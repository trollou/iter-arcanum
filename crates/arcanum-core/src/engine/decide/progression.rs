//! Décision des récompenses et de la progression : compétences, offres, artefacts, unités en
//! récompense.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn progression_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    match action {
        Action::UseSkill {
            skill,
            option,
            unit,
            target,
        } => use_skill_events(state, player, skill, option, unit, target, rngs),
        Action::ChooseSkill { skill } => decide_choose_skill(state, player, skill),
        Action::GainAdvancedAction { card } => {
            let offer = Offer::AdvancedActions;
            // MKUE p. 11, 6.b ; p. 5, 3.a : la carte sur le deck, puis l'offre complétée.
            let mut events = vec![Event::OfferCardGained {
                player,
                offer,
                card: CommonCard::Deed(*card),
                to: CardDestination::DeckTop,
            }];
            let top = state.common_deck(CommonDeck::AdvancedActions).first();
            events.extend(top.map(|&card| Event::OfferCardDealt { offer, card }));
            // Spec 035, L4 : un choix par niveau pair, l'offre complétée entre deux choix.
            let after = events.iter().fold(state.clone(), apply);
            events.extend(level_choices_next(&after, player));
            Ok(events)
        }
        Action::TakeSpell { card } => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            let offer = Offer::Spells;
            // MKUE p. 11, 5.c : le sort sur le deck, puis l'offre complétée (spec 050).
            let mut events = vec![Event::OfferCardGained {
                player,
                offer,
                card: CommonCard::Deed(*card),
                to: CardDestination::DeckTop,
            }];
            let top = state.common_deck(CommonDeck::Spells).first();
            events.extend(top.map(|&card| Event::OfferCardDealt { offer, card }));
            let left = state.offer(offer).len() - 1 + usize::from(top.is_some());
            if p.spell_choices > 1 && left > 0 {
                return Ok(events);
            }
            // Issue privée 317 (MKUE p. 11, 5.c puis 6.b) : les compétences du niveau pair, révélées
            // une fois le sort choisi.
            if p.skill_choices > 0 {
                let after = events.iter().fold(state.clone(), apply);
                events.extend(skills_revealed(&after, player));
                return Ok(events);
            }
            if p.advanced_action_choices > 0 || p.unit_choices > 0 {
                return Ok(events);
            }
            let deck: Vec<CardId> = std::iter::once(*card)
                .chain(p.deck.cards().iter().copied())
                .collect();
            let count = turn_end_limit(state, player, p).saturating_sub(p.hand.len());
            events.extend(turn_end_draw(state, player, &deck, count));
            Ok(events)
        }
        Action::TakeArtifact { card } => take_artifact_events(state, player, card),
        Action::TakeUnitReward { unit, disband } => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            if p.unit_choices == 0 || !unit_reward_actions(state, p).contains(action) {
                return Err(ActionError::NotLegal);
            }
            // MKUE p. 11, 5.d : gratuite, sans réputation ; l'offre n'est pas complétée.
            let mut events: Vec<Event> = disband
                .iter()
                .map(|&unit| Event::UnitDisbanded { player, unit })
                .collect();
            events.push(Event::UnitRewardTaken {
                player,
                unit: *unit,
            });
            Ok(unit_reward_end(state, player, p, events))
        }
        Action::DeclineUnitReward => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            if p.unit_choices == 0 || !unit_reward_actions(state, p).contains(action) {
                return Err(ActionError::NotLegal);
            }
            let events = vec![Event::UnitRewardDeclined { player }];
            Ok(unit_reward_end(state, player, p, events))
        }
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}

/// `Action::ChooseSkill` : la compétence gardée au gain de niveau.
fn decide_choose_skill(
    state: &GameState,
    player: PlayerId,
    skill: &Skill,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    // MKUE p. 11, 6.b : l'une gardée, l'autre dans l'offre commune ; ou une de l'offre
    // commune, les deux révélées y vont.
    let mut events = Vec::new();
    if !p.skill_choice.contains(skill) {
        events.push(Event::SkillTakenFromCommonOffer {
            player,
            skill: *skill,
        });
    }
    events.push(Event::SkillGained {
        player,
        skill: *skill,
    });
    // Texte de Loyauté sans faille ; Compendium, *Bonds of Loyalty* Q2 : deux unités
    // régulières dans l'offre, tout de suite ; autant que possible.
    if *skill == Skill::BondsOfLoyalty {
        events.extend(
            state
                .common_deck(CommonDeck::RegularUnits)
                .iter()
                .take(BONDS_UNITS)
                .map(|&card| Event::OfferCardDealt {
                    offer: Offer::Units,
                    card,
                }),
        );
    }
    events.extend(
        p.skill_choice
            .iter()
            .filter(|&other| other != skill)
            .map(|&other| Event::SkillToCommonOffer { skill: other }),
    );
    // Spec 061 (MKUE p. 15, *Skills in a Solo Game*) : la compétence du dessus de la pile du
    // fantôme rejoint l'offre commune, après le choix ; pile vide, rien.
    if let Some(&skill) = state.dummy.as_ref().and_then(|dummy| dummy.skills.first()) {
        events.push(Event::DummySkillRevealed { skill });
    }
    let after = events.iter().fold(state.clone(), apply);
    events.extend(level_choices_next(&after, player));
    Ok(events)
}

/// `Action::UseSkill` : la compétence utilisée, son effet et sa cible.
fn use_skill_events(
    state: &GameState,
    player: PlayerId,
    skill: &Skill,
    option: &u8,
    unit: &Option<UnitCard>,
    target: &Option<SkillTarget>,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    // Spec 058 : l'avantage d'Embrasure de la Source, un dé en plus et son cristal ; le dé
    // donne son mana en jeton, à dépenser ce tour.
    if *skill == Skill::SourceOpening
        && p.center_advantage(Skill::SourceOpening)
        && let Some(SkillTarget::Die(die)) = target
    {
        let Some(&ManaColor::Basic(color)) = state.source.get(usize::from(*die)) else {
            return Err(ActionError::NotLegal);
        };
        let mut inventory = p.inventory;
        return Ok(vec![
            Event::CenterAdvantageUsed {
                player,
                skill: *skill,
            },
            Event::DieTaken {
                player,
                die: *die,
                color: ManaColor::Basic(color),
            },
            Event::ManaTokenGained {
                player,
                color: ManaColor::Basic(color),
            },
            gained_crystal(&mut inventory, player, color),
        ]);
    }
    let choice = skill
        .definition()
        .options()
        .get(usize::from(*option))
        .ok_or(ActionError::NotLegal)?;
    // Le moment de la journée où elle est utilisée (Compendium, *Day Sharpshooting* S1).
    let effect = resolved_skill_effect(p, choice.effect(p.combat_time(state.time_of_day)));
    let mut events = vec![Event::SkillUsed {
        player,
        skill: *skill,
        option: *option,
        effect,
    }];
    let mut inventory = p.inventory;
    effect_events(&mut events, &mut inventory, p, player, effect);
    match (effect, unit) {
        (Effect::ReadyAnyUnit { .. }, Some(unit)) => {
            events.push(Event::UnitReadied {
                player,
                unit: *unit,
            });
        }
        // Compendium, *Inspiration* S2 : une Blessure, sans payer de Soin.
        (Effect::HealUnit, Some(unit)) => events.push(Event::UnitHealed {
            player,
            unit: *unit,
            cost: 0,
        }),
        _ => {}
    }
    events.extend(skill_target_events(state, player, effect, *target));
    // Compendium, *SOURCE Opening* S1 : le dé relancé.
    if let (Effect::SourceOpening, Some(SkillTarget::Die(die))) = (effect, target) {
        events.push(Event::DieRerolled {
            die: *die,
            color: roll_die(rngs.rng(RngUsage::Source)),
        });
    }
    let center_color = match (effect, target) {
        (Effect::ManaOverload, Some(SkillTarget::Color(color))) => Some(*color),
        _ => None,
    };
    // Spec 058 (MKUE p. 5, 4.b) : une coopérative est posée au centre.
    if skill.definition().cooperative {
        events.push(Event::SkillPlacedInCenter {
            player,
            skill: *skill,
            color: center_color,
        });
    }
    Ok(events)
}

/// `Action::TakeArtifact` : l'artefact gardé, les autres rendus, puis la suite des récompenses.
fn take_artifact_events(
    state: &GameState,
    player: PlayerId,
    card: &CardId,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    // MKUE p. 11, 5.b : l'artefact pris sur le deck, l'autre sous la pioche ; puis le
    // choix d'action avancée d'un niveau pair (6.b), ou la pioche (7).
    let mut events = vec![Event::ArtifactTaken {
        player,
        card: *card,
    }];
    // Spec 045 : plusieurs artefacts à garder, un choix après l'autre.
    if p.artifacts_to_keep > 1 && p.drawn_artifacts.len() > 1 {
        return Ok(events);
    }
    events.extend(
        p.drawn_artifacts
            .iter()
            .filter(|&other| other != card)
            .map(|&other| Event::ArtifactReturned { card: other }),
    );
    if p.spell_choices > 0 {
        return Ok(events);
    }
    // Issue privée 317 (MKUE p. 11, 5.b puis 6.b) : les compétences du niveau pair, révélées une fois
    // l'artefact choisi.
    if p.skill_choices > 0 {
        let after = events.iter().fold(state.clone(), apply);
        events.extend(skills_revealed(&after, player));
        return Ok(events);
    }
    if p.advanced_action_choices > 0 || p.unit_choices > 0 {
        return Ok(events);
    }
    let deck: Vec<CardId> = std::iter::once(*card)
        .chain(p.deck.cards().iter().copied())
        .collect();
    let count = turn_end_limit(state, player, p).saturating_sub(p.hand.len());
    events.extend(turn_end_draw(state, player, &deck, count));
    Ok(events)
}
