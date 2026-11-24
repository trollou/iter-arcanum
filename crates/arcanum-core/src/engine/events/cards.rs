//! Événements pour `decide` : le jeu d'une carte et ses conséquences.

use super::*;

/// Les jetons de Puits de mana pris aussitôt (carte ; spec 057).
const MANA_CLAIM_TOKENS: usize = 3;

/// Le dé de la Source visé par un sort lancé (Puits de mana, spec 057).
pub(in crate::engine) fn claimed_die(play: &CardPlay) -> Option<u8> {
    match play {
        CardPlay::Cast { targets, .. } => targets.die,
        _ => None,
    }
}

/// Le nombre de cartes de la défausse remises par Méditation (carte ; spec 057).
pub(in crate::engine) const MEDITATION_CARDS: usize = 2;

/// Les événements d'une carte jouée (specs 013, 022, 025) : le mana payé, la carte, puis la carte
/// jouée avec Concentration renforcée.
pub(in crate::engine) fn play_card_events(
    state: &GameState,
    player: PlayerId,
    card: &CardId,
    play: &CardPlay,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let effect = card.effect(*play).ok_or(ActionError::NotLegal)?;
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let mut inventory = p.inventory;
    let mut events = Vec::new();
    // Spec 058 : la compétence de côté, utilisée avec la carte (ses points viennent de la carte),
    // puis le mana de Pouvoir universel.
    if let CardPlay::SidewaysSkill {
        skill,
        option,
        mana,
        ..
    } = play
    {
        // L'avantage du Rituel de guérison se consomme ; la compétence reste au centre.
        events.push(if *skill == Skill::RitualOfPain {
            Event::CenterAdvantageUsed {
                player,
                skill: *skill,
            }
        } else {
            Event::SkillUsed {
                player,
                skill: *skill,
                option: *option,
                effect: Effect::Nothing,
            }
        });
        if let Some(mana) = mana {
            events.extend(match mana.color {
                ManaColor::Basic(color) => {
                    payment_events(state, p, player, mana.payment, color, &mut inventory)?
                }
                _ => black_payment_events(state, player, mana.payment)?,
            });
        }
    }
    // Le mana se paie au moment où on joue l'effet (MKUE p. 5, 3.b) : la couleur de la
    // carte pour l'effet renforcé, celle de l'option pour un effet de base payant.
    let payment = match play {
        CardPlay::Strong { mana, .. } => Some((
            *mana,
            card.card().definition().ok_or(ActionError::NotLegal)?.color,
        )),
        CardPlay::Basic {
            mana: Some(mana),
            option,
            ..
        } => Some((
            *mana,
            card.card()
                .definition()
                .and_then(|definition| definition.basic.option_mana(*option))
                .ok_or(ActionError::NotLegal)?,
        )),
        CardPlay::Basic { mana: None, .. }
        | CardPlay::Sideways(_)
        | CardPlay::SidewaysSkill { .. }
        | CardPlay::Boosted { .. }
        | CardPlay::Cast { .. }
        | CardPlay::Artifact { .. }
        | CardPlay::ThrowAway { .. } => None,
    };
    if let Some((mana, color)) = payment {
        events.extend(payment_events(
            state,
            p,
            player,
            mana,
            color,
            &mut inventory,
        )?);
    }
    // Spec 054 : un sort lancé, sa couleur, le noir, puis le mana en plus de l'option.
    if let CardPlay::Cast {
        option,
        mana,
        black,
        extra_mana,
        ..
    } = play
    {
        let definition = card.card().definition().ok_or(ActionError::NotLegal)?;
        let extra_color = match black {
            Some(_) => definition.strong.option_mana(*option),
            None => definition.basic.option_mana(*option),
        };
        let mut paying = p.clone();
        let mut payments = vec![(*mana, ManaColor::Basic(definition.color))];
        payments.extend(black.map(|black| (black, ManaColor::Black)));
        if let (Some(extra), Some(color)) = (extra_mana, extra_color) {
            payments.push((*extra, ManaColor::Basic(color)));
        }
        for (payment, color) in payments {
            paying.inventory = inventory;
            let paid = match color {
                ManaColor::Basic(color) => {
                    payment_events(state, &paying, player, payment, color, &mut inventory)?
                }
                _ => black_payment_events(state, player, payment)?,
            };
            paying = after_payment(state, &paying, payment, color);
            events.extend(paid);
        }
    }
    // Spec 053 : le mana en plus de l'option renforcée, payé ensuite (Q3).
    if let CardPlay::Strong {
        option,
        extra_mana: Some(extra),
        ..
    } = play
    {
        let color = card
            .card()
            .definition()
            .and_then(|definition| definition.strong.option_mana(*option))
            .ok_or(ActionError::NotLegal)?;
        let mut paying = p.clone();
        for event in &events {
            match event {
                Event::ManaTokenUsed { color, .. } => {
                    if let Some(index) = paying.mana_tokens.iter().position(|t| t == color) {
                        paying.mana_tokens.remove(index);
                    }
                }
                Event::DieTaken { die, .. } => paying.dice_taken.push(*die),
                _ => {}
            }
        }
        paying.inventory = inventory;
        events.extend(payment_events(
            state,
            &paying,
            player,
            *extra,
            color,
            &mut inventory,
        )?);
    }
    // Spec 057 : Restauration et Renaissance, le Soin selon le terrain de la case du héros.
    let forest = hero_in_forest(state, p);
    let played = match effect {
        Effect::HealByTerrain { heal, forest: more }
        | Effect::HealAndReadyLevels {
            heal,
            forest_heal: more,
            ..
        } => Effect::Heal(if forest { more } else { heal }),
        other => other,
    };
    played_events(
        &mut events,
        &mut inventory,
        p,
        player,
        *card,
        *play,
        played,
        1,
    );
    // Spec 057 : les unités rendues disponibles (Renaissance, Flux d'énergie), soignées d'une
    // Blessure par Vol d'énergie.
    if let CardPlay::Cast { targets, .. } = play
        && matches!(
            effect,
            Effect::HealAndReadyLevels { .. } | Effect::ReadyAnyUnit { .. }
        )
    {
        for unit in targets.units.iter() {
            events.push(Event::UnitReadied { player, unit });
            let wounded = p
                .units
                .iter()
                .any(|owned| owned.card == unit && owned.is_wounded());
            if effect == (Effect::ReadyAnyUnit { heal: true }) && wounded {
                events.push(Event::UnitHealed {
                    player,
                    unit,
                    cost: 0,
                });
            }
        }
    }
    // Spec 057 : le déplacement par effet, d'un coup, sans payer ni provoquer (MKUE p. 7, 7.c) ;
    // l'arrivée révèle ce qu'une arrivée ordinaire révèle (Q8) ; un site fortifié, assaut depuis la
    // case de départ (Compendium, *Underground Attack* S1).
    let moved_by_effect = match effect {
        Effect::UndergroundMove { assault } => Some((assault, None)),
        Effect::FlyMove { max } => Some((false, Some(max))),
        _ => None,
    };
    if let Some((assault, fly)) = moved_by_effect {
        let (Some(from), CardPlay::Cast { targets, .. }) = (p.figure, play) else {
            return Err(ActionError::NotLegal);
        };
        let to = targets.destination.ok_or(ActionError::NotLegal)?;
        // Ailes du vent : un point de Mouvement par case.
        let cost = match fly {
            Some(max) => fly_destinations(state, p, max)
                .into_iter()
                .find(|&(hex, _)| hex == to)
                .map(|(_, cost)| cost)
                .ok_or(ActionError::NotLegal)?,
            None => 0,
        };
        events.push(Event::HeroMoved {
            player,
            from,
            to,
            cost,
        });
        events.extend(ruin_reveal(state, to));
        events.extend(reveals_next_to(state, reveal_time(state, player), to, &[]));
        if assault {
            events.extend(assault_events(state, player, from, to));
        }
    }
    // Spec 057 : Puits de mana, le dé gardé, puis trois jetons de sa couleur aussitôt.
    if let Effect::ClaimDie { each_turn } = effect {
        let Some(die) = claimed_die(play) else {
            return Err(ActionError::NotLegal);
        };
        let Some(&color) = state.source.get(usize::from(die)) else {
            return Err(ActionError::NotLegal);
        };
        events.push(Event::DieKept { player, die });
        if !each_turn {
            events.extend((0..MANA_CLAIM_TOKENS).map(|_| Event::ManaTokenGained { player, color }));
        }
    }
    // Spec 057 : Méditation, deux cartes tirées au hasard dans la défausse (ADR 0007).
    if effect == (Effect::DiscardToDeck { random: true }) && !p.discard_pile.is_empty() {
        let mut cards = p.discard_pile.clone();
        cards.shuffle(rngs.rng(RngUsage::DiscardPick(player)));
        cards.truncate(MEDITATION_CARDS);
        events.push(Event::DiscardPicked { player, cards });
    }
    // Spec 054 : Tornade et Désintégration détruisent l'ennemi visé, après les modificateurs
    // posés ; la réputation d'un ennemi errant (spec 040).
    if effect == Effect::DestroyEnemy {
        let Some(enemy) = (match play {
            CardPlay::Cast { enemy, .. } => *enemy,
            _ => None,
        }) else {
            return Err(ActionError::NotLegal);
        };
        events.push(Event::EnemyDestroyed { player, enemy });
        if let Some(combat) = p.combat.as_ref() {
            events.extend(rampaging_reputation(state, player, combat, &[enemy]));
        }
    }
    // Spec 056 : Tempête de mana, le cristal du dé choisi, puis le dé relancé (ADR 0007) ;
    // renforcée, les dés encore dans la Source relancés (Compendium, *Mana Storm* S1).
    match effect {
        Effect::CrystalFromDie => {
            let die = match play {
                CardPlay::Basic { die: Some(die), .. } => *die,
                _ => return Err(ActionError::NotLegal),
            };
            let Some(&ManaColor::Basic(color)) = state.source.get(usize::from(die)) else {
                return Err(ActionError::NotLegal);
            };
            events.push(gained_crystal(&mut inventory, player, color));
            events.push(Event::DieRerolled {
                die,
                color: roll_die(rngs.rng(RngUsage::Source)),
            });
        }
        Effect::RerollSource => {
            let paid: Vec<u8> = events
                .iter()
                .filter_map(|event| match event {
                    Event::DieTaken { die, .. } => Some(*die),
                    _ => None,
                })
                .collect();
            for (die, _) in (0u8..).zip(&state.source) {
                if die_free(p, die) && !paid.contains(&die) {
                    events.push(Event::DieRerolled {
                        die,
                        color: roll_die(rngs.rng(RngUsage::Source)),
                    });
                }
            }
        }
        _ => {}
    }
    // Spec 056 : la carte prise dans une offre (MKUE p. 5, *"Gain" Effects* 3.a), puis l'offre
    // complétée.
    // Spec 055 : le Livre de sagesse détruit la carte action choisie, d'abord.
    if let (
        Effect::TrainFromOffer(_) | Effect::SpellAndCrystalFromOffer,
        CardPlay::Artifact { choices, .. } | CardPlay::ThrowAway { choices, .. },
    ) = (effect, play)
    {
        let destroyed = choices.discard.iter().next().ok_or(ActionError::NotLegal)?;
        events.push(Event::CardDestroyed {
            player,
            card: destroyed,
            from: DestroyedFrom::Hand,
        });
    }
    let gained = match effect {
        Effect::TrainFromOffer(to) => {
            Some((Offer::AdvancedActions, CommonDeck::AdvancedActions, to))
        }
        Effect::SpellAndCrystalFromOffer => {
            Some((Offer::Spells, CommonDeck::Spells, CardDestination::Hand))
        }
        Effect::GainSpell(_) => Some((
            Offer::Spells,
            CommonDeck::Spells,
            CardDestination::DiscardPile,
        )),
        _ => None,
    };
    if let Some((offer, deck, to)) = gained {
        let gain = play_gain(*play).ok_or(ActionError::NotLegal)?;
        events.push(Event::OfferCardGained {
            player,
            offer,
            card: CommonCard::Deed(gain),
            to,
        });
        let top = state.common_deck(deck).first();
        events.extend(top.map(|&card| Event::OfferCardDealt { offer, card }));
    }
    // Spec 025 : la carte jouée avec Concentration, renforcée sans payer, avec le bonus.
    if let (
        CardPlay::Strong {
            with: Some(target), ..
        },
        Effect::Boost(bonus),
    ) = (play, effect)
    {
        let boosted = CardPlay::Boosted {
            option: target.option,
            discard: target.discard,
            set_dice: target.set_dice,
        };
        let target_effect = target.card.effect(boosted).ok_or(ActionError::NotLegal)?;
        played_events(
            &mut events,
            &mut inventory,
            p,
            player,
            target.card,
            boosted,
            target_effect.with_bonus(bonus),
            1,
        );
    }
    // Spec 056 : la carte jouée avec Effet maximum, son option répétée sans payer, puis détruite
    // (MKUE p. 5, *"Composite" Effects* 1.a et 1.b).
    if let (
        CardPlay::Basic {
            with: Some(target), ..
        }
        | CardPlay::Strong {
            with: Some(target), ..
        },
        Effect::Repeat(times),
    ) = (play, effect)
    {
        let target_play = match play {
            CardPlay::Strong { .. } => CardPlay::Boosted {
                option: target.option,
                discard: None,
                set_dice: [None, None],
            },
            _ => CardPlay::Basic {
                option: target.option,
                discard: None,
                mana: None,
                with: None,
                gain: None,
                die: None,
            },
        };
        let target_effect = target
            .card
            .effect(target_play)
            .ok_or(ActionError::NotLegal)?;
        played_events(
            &mut events,
            &mut inventory,
            p,
            player,
            target.card,
            target_play,
            target_effect,
            times,
        );
        events.push(Event::CardDestroyed {
            player,
            card: target.card,
            from: DestroyedFrom::PlayArea,
        });
    }
    // Spec 055 : les jetons des amulettes ; de nuit, l'Amulette du soleil révèle comme de jour les
    // ruines face cachée et les défenseurs voisins (Q6).
    match (effect, play) {
        (Effect::AmuletOfSun { tokens }, _) => {
            events.extend((0..tokens).map(|_| Event::ManaTokenGained {
                player,
                color: ManaColor::Gold,
            }));
            if state.time_of_day == TimeOfDay::Night {
                events.extend(
                    state
                        .ruins
                        .iter()
                        .filter(|(_, placed)| !placed.face_up)
                        .map(|(&hex, placed)| Event::RuinTokenRevealed {
                            hex,
                            token: placed.token,
                        }),
                );
                if let Some(figure) = p.figure {
                    events.extend(reveals_next_to(state, TimeOfDay::Day, figure, &[]));
                }
            }
        }
        (
            Effect::AmuletOfDarkness { tokens },
            CardPlay::Artifact { choices, .. } | CardPlay::ThrowAway { choices, .. },
        ) => {
            if choices.colors.len() != usize::from(tokens) {
                return Err(ActionError::NotLegal);
            }
            events.extend(
                choices
                    .colors
                    .iter()
                    .map(|color| Event::ManaTokenGained { player, color }),
            );
        }
        // Spec 055 (Corne de la colère) : un dé, ou un par point ajouté ; une Blessure par noir ou rouge,
        // comptée pour être assommé.
        (Effect::HornOfWrath { extra }, _) => {
            let dice = match play {
                CardPlay::ThrowAway { choices, .. } if choices.bonus <= extra => choices.bonus,
                CardPlay::ThrowAway { .. } => return Err(ActionError::NotLegal),
                _ => 1,
            };
            let mut hits = 0u8;
            for _ in 0..dice {
                let color = roll_die(rngs.rng(RngUsage::ArtifactDice));
                if matches!(color, ManaColor::Black | ManaColor::Basic(BasicColor::Red)) {
                    hits += 1;
                }
                events.push(Event::ArtifactDieRolled {
                    player,
                    card: *card,
                    color,
                });
            }
            if hits > 0 {
                let wounds = wounds_from_card(&events, p, player, *card, hits);
                events.extend(wounds);
            }
        }
        // Spec 055 (Poche de mana) : deux dés ; l'or en dernier, son choix attend.
        (Effect::GemPouchDice, _) => {
            let mut faces: Vec<ManaColor> = (0..2)
                .map(|_| roll_die(rngs.rng(RngUsage::ArtifactDice)))
                .collect();
            faces.sort_by_key(|face| *face == ManaColor::Gold);
            for color in faces {
                events.push(Event::ArtifactDieRolled {
                    player,
                    card: *card,
                    color,
                });
                match color {
                    ManaColor::Basic(color) => {
                        events.push(gained_crystal(&mut inventory, player, color));
                    }
                    ManaColor::Black => events.push(Event::FameGained { player, fame: 1 }),
                    ManaColor::Gold => {}
                }
            }
        }
        // Spec 055 (Livre de sagesse renforcé) : le cristal de la couleur de la carte détruite.
        (
            Effect::SpellAndCrystalFromOffer,
            CardPlay::ThrowAway { choices, .. } | CardPlay::Artifact { choices, .. },
        ) => {
            if let Some(definition) = choices
                .discard
                .iter()
                .next()
                .and_then(|destroyed| destroyed.card().definition())
            {
                events.push(gained_crystal(&mut inventory, player, definition.color));
            }
        }
        // Spec 055 (Compendium, *Endless Gem Pouch* S1) : or de jour, noir de nuit ou dans un
        // donjon.
        (Effect::TokensEachColorAndTime, _) => {
            events.extend(
                BasicColor::ALL
                    .into_iter()
                    .map(|color| Event::ManaTokenGained {
                        player,
                        color: ManaColor::Basic(color),
                    }),
            );
            events.push(Event::ManaTokenGained {
                player,
                color: match p.combat_time(state.time_of_day) {
                    TimeOfDay::Day => ManaColor::Gold,
                    TimeOfDay::Night => ManaColor::Black,
                },
            });
        }
        _ => {}
    }
    // Spec 055 : l'artefact joué pour son effet renforcé est détruit (MKUE p. 4, 3.c).
    if let CardPlay::ThrowAway { .. } = play {
        events.push(Event::CardDestroyed {
            player,
            card: *card,
            from: DestroyedFrom::PlayArea,
        });
    }
    Ok(events)
}

/// Une carte jouée, la carte qu'elle fait défausser, puis le mana qu'elle fait gagner, les
/// Blessures qu'elle jette ou les cartes qu'elle fait piocher (spec 034).
#[allow(clippy::too_many_arguments)]
pub(super) fn played_events(
    events: &mut Vec<Event>,
    inventory: &mut Inventory,
    p: &PlayerState,
    player: PlayerId,
    card: CardId,
    play: CardPlay,
    effect: Effect,
    times: u8,
) {
    // Spec 056 : Effet maximum, un seul effet de points répété ; jetons, cristaux et Blessures un
    // par un.
    let base = effect;
    let effect = effect.repeated(times);
    // Spec 053 : Besoin d'aide compte les Blessures en main et sur les unités en jouant la carte.
    let effect = match effect {
        Effect::InfluencePerWound { base, per } => {
            let wounds = p.hand.iter().filter(|card| card.card().is_wound()).count()
                + p.units
                    .iter()
                    .map(|owned| usize::from(owned.wounds))
                    .sum::<usize>();
            let per_wound = usize::from(per).saturating_mul(wounds);
            Effect::Influence(base.saturating_add(u8::try_from(per_wound).unwrap_or(u8::MAX)))
        }
        // Spec 055 : la Corne de la colère, Attaque de siège 5 plus le bonus choisi.
        Effect::HornOfWrath { .. } => Effect::Attack {
            amount: 5u8.saturating_add(match play {
                CardPlay::ThrowAway { choices, .. } => choices.bonus,
                _ => 0,
            }),
            element: Element::Physical,
            kind: AttackKind::Siege,
        },
        // Spec 055 : l'Épée de justice, Attaque physique 3 par carte défaussée.
        Effect::SwordDiscard => match play {
            CardPlay::Artifact { choices, .. } => Effect::Attack {
                amount: u8::try_from(3 * choices.discard.iter().count()).unwrap_or(u8::MAX),
                element: Element::Physical,
                kind: AttackKind::Melee,
            },
            _ => Effect::Nothing,
        },
        // Spec 054, Q4 : Déluge de flammes compte les jetons ennemis affrontés en jouant la carte.
        Effect::AttackPerEnemy { .. } | Effect::BlockPerEnemy { .. } => {
            effect.against_enemies(p.combat.as_ref().map_or(0, Combat::enemies_faced))
        }
        other => other,
    };
    // Spec 058 (Q5) : l'avantage de Surcharge de mana, +4 à la première carte alimentée de sa
    // couleur qui donne elle-même des points (Compendium, *Mana Overload* S1 à S6).
    let mut played = effect.resolved();
    if let Some(color) = powered_color(card, &play)
        && p.center_advantages.iter().any(|advantage| {
            advantage.skill == Skill::ManaOverload
                && advantage.active
                && !advantage.used
                && advantage.color == Some(ManaColor::Basic(color))
        })
        && played.with_bonus(MANA_OVERLOAD_BONUS) != played
    {
        events.push(Event::CenterAdvantageUsed {
            player,
            skill: Skill::ManaOverload,
        });
        played = played.with_bonus(MANA_OVERLOAD_BONUS);
    }
    events.push(Event::CardPlayed {
        player,
        card,
        play,
        effect: played,
    });
    // Spec 055 : les cartes défaussées par l'Épée de justice.
    if base == Effect::SwordDiscard
        && let CardPlay::Artifact { choices, .. } = play
    {
        events.extend(
            choices
                .discard
                .iter()
                .map(|card| Event::CardDiscarded { player, card }),
        );
    }
    // Spec 053 : la Blessure de Coup de sang, en main ; elle compte pour être assommé (spec 034).
    // Spec 056 : celle de Rituel de sang aussi (Compendium, *Blood Ritual* S3).
    if let Effect::AttackWithWound { .. }
    | Effect::WoundCrystalAndToken { .. }
    | Effect::WoundAndTokens(_) = effect
    {
        events.extend(wounds_from_card(events, p, player, card, times));
    }
    if let CardPlay::Basic {
        discard: Some(discarded),
        ..
    }
    | CardPlay::Strong {
        discard: Some(discarded),
        ..
    }
    | CardPlay::Boosted {
        discard: Some(discarded),
        ..
    } = play
    {
        // Spec 056 : Décomposition et Entraînement détruisent la carte au lieu de la défausser.
        let destroys = card.card().definition().is_some_and(|definition| {
            matches!(
                definition.basic,
                BasicEffect::Options {
                    cost: Cost::DestroyActionCard,
                    ..
                }
            )
        });
        events.push(if destroys {
            Event::CardDestroyed {
                player,
                card: discarded,
                from: DestroyedFrom::Hand,
            }
        } else {
            Event::CardDiscarded {
                player,
                card: discarded,
            }
        });
        // Spec 056 : les cristaux de Décomposition, selon la couleur de la carte détruite.
        let color = discarded
            .card()
            .definition()
            .map(|definition| definition.color);
        let colors: Vec<BasicColor> = match (effect, color) {
            (Effect::CrystalsFromDestroyed(count), Some(color)) => {
                vec![color; usize::from(count)]
            }
            (Effect::CrystalsOtherColors, Some(color)) => BasicColor::ALL
                .into_iter()
                .filter(|&other| other != color)
                .collect(),
            _ => Vec::new(),
        };
        for color in colors {
            events.push(gained_crystal(inventory, player, color));
        }
    }
    // Spec 057 : Brûlure de mana, une Blessure par cristal de la couleur, puis deux cristaux.
    if let Effect::WoundsPerCrystal(color) = effect {
        let count = inventory.crystals(color);
        if count > 0 {
            events.extend(wounds_from_card(events, p, player, card, count));
        }
        for _ in 0..2 {
            events.push(gained_crystal(inventory, player, color));
        }
    }
    if let Effect::GainToken(_) | Effect::GainCrystal(_) = base {
        for _ in 0..times {
            effect_events(events, inventory, p, player, base);
        }
    } else {
        effect_events(events, inventory, p, player, effect);
    }
    if let Effect::SetDice { tokens, .. } = effect
        && let CardPlay::Strong { set_dice, .. } | CardPlay::Boosted { set_dice, .. } = play
    {
        // Chaque dé mis, puis ses jetons (spec 025).
        for setting in set_dice.iter().flatten() {
            events.push(Event::DieSet {
                player,
                die: setting.die,
                color: setting.color,
            });
            events.extend((0..tokens).map(|_| Event::ManaTokenGained {
                player,
                color: ManaColor::Basic(setting.color),
            }));
        }
    }
}

/// La Blessure prise par `card` (Coup de sang, spec 053), puis l'assommé si elle atteint la limite
/// de main du jeton de niveau dans ce combat (MKUE p. 9, 5.c) : les cartes de la main qui ne sont
/// ni des Blessures ni jouées par cette action vont dans la défausse ; l'Attaque reste.
fn wounds_from_card(
    events: &[Event],
    p: &PlayerState,
    player: PlayerId,
    card: CardId,
    count: u8,
) -> Vec<Event> {
    // Spec 056 : une Blessure par répétition d'Effet maximum (Compendium, *Maximal Effect* S33).
    let mut result: Vec<Event> = (1..=u16::from(count))
        .map(|nth| Event::WoundFromCard {
            player,
            card,
            wound: CardId::new(DeedCard::Wound, p.wounds_given.saturating_add(nth)),
        })
        .collect();
    if let Some(combat) = p.combat.as_ref().filter(|combat| !combat.ended)
        && !combat.knocked_out
        && combat.wounds_to_hand + u32::from(count) >= u32::from(p.level_token().hand_limit)
    {
        let played: Vec<CardId> = events
            .iter()
            .filter_map(|event| match event {
                Event::CardPlayed { card, .. } | Event::CardDiscarded { card, .. } => Some(*card),
                _ => None,
            })
            .collect();
        result.push(Event::KnockedOut {
            player,
            cards: p
                .hand
                .iter()
                .copied()
                .filter(|other| !other.card().is_wound() && !played.contains(other))
                .collect(),
        });
    }
    result
}
