//! Événements pour `decide` : fin du tour (gloire, bannières perdues, clairière,
//! récompenses, niveaux, pioche).

use super::*;

/// Spec 055 (Graal doré ; Compendium, *Golden Grail* S1 ; Q7) : Gloire +1 par point de soin
/// dépensé, pour les premiers points ; renforcée, une carte piochée par Blessure de la main jetée,
/// tant que le deck en a.
pub(in crate::engine) fn with_grail(state: &GameState, events: Vec<Event>) -> Vec<Event> {
    let mut result = Vec::with_capacity(events.len());
    for event in events {
        let (player, points, hand_wounds) = match &event {
            Event::WoundsHealed { player, wounds } => (*player, wounds.len(), wounds.len()),
            Event::UnitHealed { player, cost, .. } => (*player, usize::from(*cost), 0),
            _ => {
                result.push(event);
                continue;
            }
        };
        result.push(event);
        let Some(p) = state.player(player) else {
            continue;
        };
        let left: usize = p
            .modifiers
            .iter()
            .map(|modifier| match modifier.kind {
                ModifierKind::GrailFame { left } => usize::from(left),
                _ => 0,
            })
            .sum();
        if let Ok(fame @ 1..) = u32::try_from(points.min(left)) {
            result.push(Event::FameGained { player, fame });
        }
        let draws = hand_wounds
            * p.modifiers
                .iter()
                .filter(|modifier| modifier.kind == ModifierKind::DrawPerHandWoundHealed)
                .count();
        result.extend(
            p.deck
                .cards()
                .iter()
                .take(draws)
                .map(|&card| Event::CardDrawn { player, card }),
        );
    }
    result
}

/// Spec 055 (Graal doré) : les points de soin dépensés usent la gloire du Graal.
pub(in crate::engine) fn spend_grail_fame(p: &mut PlayerState, points: u32) {
    let mut points = points;
    for modifier in &mut p.modifiers {
        if let ModifierKind::GrailFame { left } = &mut modifier.kind {
            let used = u8::try_from(points).unwrap_or(u8::MAX).min(*left);
            *left -= used;
            points -= u32::from(used);
        }
    }
    p.modifiers
        .retain(|modifier| modifier.kind != ModifierKind::GrailFame { left: 0 });
}

/// Spec 055 (Q3) : la gloire « ce tour » des artefacts, ce qui est déjà fait aussitôt, la suite au
/// fur et à mesure. Un anneau renforcé : Gloire +1 par sort de sa couleur lancé ce tour. La
/// Bannière de gloire : Gloire +1 à chaque Attaque ou Parade de l'unité porteuse ; renforcée,
/// Gloire +1 par unité qui attaque ou pare ce tour, une fois par unité. Une même action ne joue
/// pas la carte et n'active pas l'unité à la fois.
pub(in crate::engine) fn with_turn_fame(state: &GameState, events: Vec<Event>) -> Vec<Event> {
    let mut result = Vec::with_capacity(events.len());
    for event in events {
        let fame = match &event {
            Event::CardPlayed {
                player, card, play, ..
            } => state.player(*player).map_or(0, |p| {
                let rings = card.modifiers(*play).iter().filter_map(|kind| match kind {
                    ModifierKind::EndlessMana(color) => Some(*color),
                    _ => None,
                });
                let before: usize = rings
                    .map(|color| p.spells_cast.iter().filter(|c| **c == color).count())
                    .sum();
                let after = match (play, card.card().definition()) {
                    (CardPlay::Cast { .. }, Some(definition)) => p
                        .modifiers
                        .iter()
                        .filter(|modifier| {
                            modifier.kind == ModifierKind::EndlessMana(definition.color)
                        })
                        .count(),
                    _ => 0,
                };
                let units = if card
                    .modifiers(*play)
                    .contains(&ModifierKind::FamePerUnitAttackOrBlock)
                {
                    p.units_fought.len()
                } else {
                    0
                };
                let defeated = if card
                    .modifiers(*play)
                    .contains(&ModifierKind::FamePerEnemyDefeated)
                {
                    enemies_defeated_this_turn(p)
                } else {
                    0
                };
                before + after + units + defeated
            }),
            Event::EnemiesDefeated {
                player, enemies, ..
            } => state.player(*player).map_or(0, |p| {
                p.modifiers
                    .iter()
                    .filter(|modifier| modifier.kind == ModifierKind::FamePerEnemyDefeated)
                    .count()
                    * enemies.len()
            }),
            Event::UnitActivated {
                player,
                unit,
                effect,
                ..
            } if is_attack_or_block(*effect) => state.player(*player).map_or(0, |p| {
                let banner = usize::from(glory_banner(p, *unit));
                let strong = if p.units_fought.contains(unit) {
                    0
                } else {
                    p.modifiers
                        .iter()
                        .filter(|modifier| modifier.kind == ModifierKind::FamePerUnitAttackOrBlock)
                        .count()
                };
                banner + strong
            }),
            _ => 0,
        };
        let player = match &event {
            Event::CardPlayed { player, .. }
            | Event::UnitActivated { player, .. }
            | Event::EnemiesDefeated { player, .. } => Some(*player),
            _ => None,
        };
        result.push(event);
        if let (Some(player), Ok(fame @ 1..)) = (player, u32::try_from(fame)) {
            result.push(Event::FameGained { player, fame });
        }
    }
    result
}

/// Spec 055 (MKUE p. 5, 8.a) : avant qu'une unité porteuse d'une bannière soit détruite ou
/// renvoyée, sa bannière va dans la défausse.
pub(in crate::engine) fn with_banner_losses(state: &GameState, events: Vec<Event>) -> Vec<Event> {
    let mut result = Vec::with_capacity(events.len());
    for event in events {
        if let Event::UnitDestroyed { player, unit } | Event::UnitDisbanded { player, unit } = event
            && let Some(card) = state.player(player).and_then(|p| {
                p.units
                    .iter()
                    .find(|owned| owned.card == unit)
                    .and_then(|owned| owned.banner)
            })
        {
            result.push(Event::BannerLost { player, card, unit });
        }
        result.push(event);
    }
    result
}

/// Spec 044 (carte de la clairière ; Compendium, *Turn Start* II) : un tour qui commence sur une
/// clairière magique, nouvelle manche comprise, donne un jeton de mana or de jour, noir de nuit,
/// aussitôt après `TurnStarted` ; pas si le tour est perdu par l'annonce obligatoire de fin de
/// manche (MKUE p. 6, 4).
pub(in crate::engine) fn with_glade_token(
    state: &GameState,
    player: PlayerId,
    mut events: Vec<Event>,
) -> Vec<Event> {
    if !matches!(events.last(), Some(Event::TurnStarted { player: started, .. } | Event::ExtraTurnStarted { player: started }) if *started == player)
    {
        return events;
    }
    let after = events.iter().fold(state.clone(), apply);
    let Some(p) = after.player(player) else {
        return events;
    };
    if on_glade(&after, p) && !must_announce_end_of_round(p) {
        let color = match after.time_of_day {
            TimeOfDay::Day => ManaColor::Gold,
            TimeOfDay::Night => ManaColor::Black,
        };
        events.push(Event::ManaTokenGained { player, color });
    }
    // Spec 057 : Puits de mana, un jeton de la couleur du dé gardé au début de chaque tour de la
    // manche, supplémentaire compris.
    events.extend(
        p.modifiers
            .iter()
            .filter_map(|modifier| match modifier.kind {
                ModifierKind::ManaClaimPerTurn(color) => Some(Event::ManaTokenGained {
                    player,
                    color: ManaColor::Basic(color),
                }),
                _ => None,
            }),
    );
    events
}

/// Spec 045 : après le choix d'unité, le suivant ou la pioche (MKUE p. 11, 7).
pub(in crate::engine) fn unit_reward_end(
    state: &GameState,
    player: PlayerId,
    p: &PlayerState,
    mut events: Vec<Event>,
) -> Vec<Event> {
    if p.unit_choices > 1 {
        return events;
    }
    let count = turn_end_limit(state, player, p).saturating_sub(p.hand.len());
    events.extend(turn_end_draw(state, player, p.deck.cards(), count));
    events
}

/// La fin du tour après le bénéfice de la case (MKUE p. 11, 5 à 7), sur l'état `state` qui suit
/// `TurnEnded` : les cristaux aléatoires un dé à la fois (spec 044), l'artefact (spec 041), les
/// niveaux (spec 035), la défausse `discard`, les choix de sort et d'action avancée (specs 043 et
/// 050), puis la pioche. Un dé or attend le choix de la couleur : la défausse est faite avant, et
/// `ChooseCrystalColor` reprend ici avec une défausse vide.
pub(in crate::engine) fn turn_end_rewards(
    state: &GameState,
    player: PlayerId,
    discard: &[CardId],
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let mut state = state.clone();
    let mut events = Vec::new();
    let push = |state: &mut GameState, events: &mut Vec<Event>, event: Event| {
        *state = apply(std::mem::take(state), &event);
        events.push(event);
    };
    // MKUE p. 11, 5.a : un cristal de la couleur, rien si elle est pleine ; noir, 1 gloire (avant
    // le changement de niveau, 6) ; or, la couleur au choix du joueur.
    while state.player(player).is_some_and(|p| p.crystal_rewards > 0) {
        let color = roll_die(rngs.rng(RngUsage::RewardDice));
        push(
            &mut state,
            &mut events,
            Event::RewardDieRolled { player, color },
        );
        match color {
            ManaColor::Basic(color) => {
                if !state.inventory(player).is_full(color) {
                    push(
                        &mut state,
                        &mut events,
                        Event::CrystalGained { player, color },
                    );
                }
            }
            ManaColor::Black => {
                push(
                    &mut state,
                    &mut events,
                    Event::FameGained { player, fame: 1 },
                );
            }
            ManaColor::Gold => {
                if state.crystal_choices(player) > 0 {
                    events.extend(
                        discard
                            .iter()
                            .map(|&card| Event::CardDiscarded { player, card }),
                    );
                    return events;
                }
            }
        }
    }
    let Some(p) = state.player(player) else {
        return events;
    };
    // MKUE p. 11, 7.b et 7.c : jusqu'à la limite de main, rien au-delà ; 7.d : pas de remélange
    // quand le deck s'épuise. Une reprise (spec 025) défausse une carte et en remet une en main :
    // la main gardée ne change pas. Spec 035 : la pioche prend déjà la limite du niveau atteint à
    // l'étape 6.
    let reached = p.level_by_fame();
    let kept = p.hand.len() - discard.len();
    let limit = usize::from(level_token(reached).hand_limit)
        + state.hand_bonus(player)
        + draw_bonus(p)
        + planning_bonus(p, kept);
    let count = limit.saturating_sub(kept);
    // Spec 041 : la récompense artefact, après le bénéfice de la case et avant le
    // changement de niveau (MKUE p. 11, 5.b) : deux artefacts tirés attendent le choix ;
    // un seul est pris sans choix ; aucun, la récompense est perdue.
    let mut choosing_artifact = false;
    // Spec 045 : n récompenses, n + 1 artefacts tirés, n gardés (MKUE p. 11, 5.b).
    let mut reward = Vec::new();
    if p.artifact_rewards > 0 {
        let rewards = usize::from(p.artifact_rewards);
        let cards: Vec<CardId> = state
            .common_deck(CommonDeck::Artifacts)
            .iter()
            .filter_map(|card| card.deed())
            .take(rewards + 1)
            .collect();
        if cards.len() > rewards {
            events.push(Event::ArtifactsDrawn { player, cards });
            choosing_artifact = true;
        } else if !cards.is_empty() {
            events.push(Event::ArtifactsDrawn {
                player,
                cards: cards.clone(),
            });
            for &card in &cards {
                events.push(Event::ArtifactTaken { player, card });
            }
            // Chaque artefact pris va sur le deck : le dernier dessus.
            reward = cards.into_iter().rev().collect();
        }
    }
    // MKUE p. 11, 6 : un changement de niveau par ligne franchie, après le bénéfice de la
    // case et avant la pioche ; tous les niveaux manqués d'un coup (spec 035, L4).
    let levels: Vec<Event> = (p.level.max(1) + 1..=reached)
        .map(|level| Event::LevelGained { player, level })
        .collect();
    // Spec 051 : un niveau pair révèle deux compétences après le changement de niveau (MKUE
    // p. 11, 6.b) ; les niveaux pairs suivants, l'un après l'autre (spec 035, L4). Issue privée 317 :
    // pas avant le choix de l'artefact ou du sort de la récompense (5.b, 5.c), qui les révèle.
    let leveled = levels.iter().fold(state.clone(), apply);
    let choosing_skill = leveled.player(player).is_some_and(|p| p.skill_choices > 0);
    events.extend(levels);
    let choosing_spell = p.spell_choices > 0;
    if choosing_skill && !choosing_artifact && !choosing_spell {
        events.extend(skills_revealed(&leveled, player));
    }
    events.extend(
        discard
            .iter()
            .map(|&card| Event::CardDiscarded { player, card }),
    );
    // Spec 050 (Q6) : un niveau pair attend le choix d'une action avancée, avant la pioche
    // (MKUE p. 11, 6.b puis 7) ; rien à choisir si l'offre est vide (L1).
    let even_level = (p.level.max(1) + 1..=reached)
        .any(|level| level_reward(level) == LevelReward::SkillAndAdvancedAction);
    // Spec 043 : la récompense sort attend son choix, après l'artefact (MKUE p. 11, 5.c) ;
    // perdue si l'offre de sorts est vide.
    // `TurnEnded` en a fait des choix.
    if choosing_artifact
        || choosing_spell
        || p.advanced_action_choices > 0
        || p.unit_choices > 0
        || choosing_skill
        || (even_level && !state.offer(Offer::AdvancedActions).is_empty())
    {
        return events;
    }
    let deck: Vec<CardId> = reward
        .into_iter()
        .chain(p.deck.cards().iter().copied())
        .collect();
    events.extend(turn_end_draw(&state, player, &deck, count));
    events
}

/// Spec 051 : les compétences de l'offre commune que le joueur peut prendre : celles des autres
/// héros (Compendium, *Skill Tokens* S3 ; FAQ p3).
pub(in crate::engine) fn common_skills_takeable<'a>(
    state: &'a GameState,
    p: &'a PlayerState,
) -> impl Iterator<Item = &'a Skill> {
    state
        .common_skills
        .iter()
        .filter(move |skill| Some(skill.hero()) != p.hero)
}

/// Spec 051 : les deux compétences du dessus de la pile, révélées pour un niveau pair.
pub(in crate::engine) fn skills_revealed(state: &GameState, player: PlayerId) -> Option<Event> {
    let p = state.player(player)?;
    Some(Event::SkillsRevealed {
        player,
        skills: p.skill_pile.iter().take(2).copied().collect(),
    })
}

/// Spec 051 : la suite des choix de fin de tour, après une compétence ou une action avancée
/// (MKUE p. 11, 6.b puis 7) : l'action avancée du niveau dont la compétence est choisie, la
/// paire du niveau pair suivant, l'unité d'une ruine (spec 045), ou la pioche. Calculée sur
/// l'état qui suit le choix.
pub(in crate::engine) fn level_choices_next(state: &GameState, player: PlayerId) -> Vec<Event> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    if p.advanced_action_choices > p.skill_choices
        && !state.offer(Offer::AdvancedActions).is_empty()
    {
        return Vec::new();
    }
    if p.skill_choices > 0 {
        return skills_revealed(state, player).into_iter().collect();
    }
    if p.unit_choices > 0 {
        return Vec::new();
    }
    let count = turn_end_limit(state, player, p).saturating_sub(p.hand.len());
    turn_end_draw(state, player, p.deck.cards(), count)
}

/// La limite de main d'une pioche : celle du niveau, plus le bonus des forteresses possédées
/// (spec 043) ou d'une cité conquise (spec 062).
pub(in crate::engine) fn draw_limit(state: &GameState, player: PlayerId, p: &PlayerState) -> usize {
    usize::from(p.level_token().hand_limit) + state.hand_bonus(player) + draw_bonus(p)
}

/// La limite de la pioche de fin de tour, la main gardée étant la main actuelle : celle de
/// `draw_limit`, plus Planification (spec 060).
pub(in crate::engine) fn turn_end_limit(
    state: &GameState,
    player: PlayerId,
    p: &PlayerState,
) -> usize {
    draw_limit(state, player, p) + planning_bonus(p, p.hand.len())
}

/// Spec 060 (carte ; MKUE p. 11, 7.b ; FAQ p1) : Planification, une carte de plus à la pioche de
/// fin de tour si au moins 2 cartes, Blessures comprises, sont gardées en main.
fn planning_bonus(p: &PlayerState, kept: usize) -> usize {
    let planning = p
        .tactic
        .as_ref()
        .is_some_and(|tactic| tactic.tactic == Tactic::Planning);
    usize::from(planning && kept >= 2)
}

/// Spec 057 : les cartes en plus de Méditation pour la prochaine pioche, cumulées (Compendium,
/// *Meditation* O3).
pub(super) fn draw_bonus(p: &PlayerState) -> usize {
    p.modifiers
        .iter()
        .map(|modifier| match modifier.kind {
            ModifierKind::DrawBonus(bonus) => usize::from(bonus),
            _ => 0,
        })
        .sum()
}

/// Spec 057 (carte ; Compendium, *Time Bending* S7) : avec Distorsion temporelle en cours, les
/// cartes de la zone de jeu reviennent en main, sauf la carte écartée : celle du sort, ou Don pour
/// la magie s'il l'a lancé depuis l'offre (MKUE p. 5, *"Composite" Effects* 1.c).
pub(in crate::engine) fn time_bending_events(p: &PlayerState, player: PlayerId) -> Vec<Event> {
    let Some(source) =
        p.modifiers
            .iter()
            .find_map(|modifier| match (modifier.kind, modifier.source) {
                (ModifierKind::TimeBending, ModifierSource::Card(card)) => Some(card),
                _ => None,
            })
    else {
        return Vec::new();
    };
    let played: Vec<CardId> = p.play_area.iter().map(|played| played.card).collect();
    let aside = if played.contains(&source) {
        Some(source)
    } else {
        // Lancé depuis l'offre : la carte qui l'a permis (Don pour la magie), dont le
        // modificateur est déjà consommé ; la dernière jouée.
        p.play_area
            .iter()
            .rev()
            .find(|played| played.effect == Effect::CastFromOffer)
            .map(|played| played.card)
    };
    let mut events = vec![Event::CardsReturnedToHand {
        player,
        cards: played
            .iter()
            .copied()
            .filter(|&card| Some(card) != aside)
            .collect(),
    }];
    events.extend(aside.map(|card| Event::CardSetAside { player, card }));
    events
}

/// Les remises à zéro d'un nouveau tour, supplémentaire compris (spec 057).
pub(in crate::engine) fn start_turn(p: &mut PlayerState) {
    // Spec 060 : le choix qui suit la prise de la tactique est fait.
    p.tactic_follow_up = false;
    p.draw_stopped = None;
    // Spec 060 (Compendium, *Sparing Power* S2) : Provisions face visible, le choix avant le tour.
    p.sparing_pending = p
        .tactic
        .as_ref()
        .is_some_and(|tactic| tactic.tactic == Tactic::SparingPower && !tactic.flipped);
    if let Some(tactic) = p.tactic.as_mut() {
        tactic.mana_search_used = false;
    }
    // Spec 073 : les faits du tour précédent restent visibles jusqu'ici.
    p.turn_facts.clear();
    p.played_elsewhere = false;
    p.spells_cast.clear();
    p.units_fought.clear();
    p.wounds_received_hand = 0;
    p.wounds_received_discard = 0;
    // Spec 052 : le trajet du tour part de la case de la figurine.
    p.turn_path = p.figure.into_iter().collect();
    p.advanced_action_choices = 0;
    // Spec 051 : les choix de compétence ne survivent pas non plus.
    p.skill_choices = 0;
    p.skill_choice.clear();
    p.lowest_advanced_action_only = false;
    // Spec 041 : une récompense se prend à la fin du tour où elle est gagnée.
    p.artifact_rewards = 0;
    p.spell_choices = 0;
    p.artifacts_to_keep = 0;
    p.unit_choices = 0;
}

/// La fin d'un tour après le changement de niveau : la pioche de `count` cartes du dessus de
/// `deck` et le tour suivant (MKUE p. 11, 7), ou la fin de la partie au dernier tour, sans pioche
/// ni tour suivant (MKUE p. 18, *Scenario End* ; p. 4, 4.c ; spec 063).
pub(in crate::engine) fn turn_end_draw(
    state: &GameState,
    player: PlayerId,
    deck: &[CardId],
    count: usize,
) -> Vec<Event> {
    // Spec 057 : Distorsion temporelle, un tour supplémentaire sans pioche, même au dernier tour
    // (Compendium, *Time Bending* S2, S10).
    if state.player(player).is_some_and(|p| p.extra_turn_pending) {
        return vec![Event::ExtraTurnStarted { player }];
    }
    // Spec 060 (Compendium, *The Right Moment* S3) : le tour de plus se joue avant la fin.
    let right_moment = state
        .player(player)
        .and_then(|p| p.tactic.as_ref())
        .is_some_and(|tactic| tactic.right_moment);
    // Spec 061 (MKUE p. 15 ; Compendium, *Dummy Player* S3) : le dernier tour de la manche
    // annoncée par le fantôme, sans pioche ; `decide` complète la fin de la manche.
    if state.dummy.as_ref().is_some_and(|dummy| dummy.announced) && !right_moment {
        return vec![Event::RoundEnded { round: state.round }];
    }
    if state.last_turn == LastTurn::Current && !right_moment {
        return vec![Event::GameEnded {
            reason: crate::game_end::last_turn_reason(state),
        }];
    }
    let mut events: Vec<Event> = deck
        .iter()
        .take(count)
        .map(|&card| Event::CardDrawn { player, card })
        .collect();
    // Spec 057 : les cartes en plus de Méditation servent à la première pioche qui tire.
    if !events.is_empty() && state.player(player).is_some_and(|p| draw_bonus(p) > 0) {
        events.push(Event::DrawBonusUsed { player });
    }
    // Spec 060 (Q5 ; MKUE p. 11, 7.d, note) : le deck épuisé avant la limite, la pioche s'arrête
    // sur le choix de Longue Nuit.
    if deck.len() < count && state.player(player).is_some_and(long_night_usable) {
        events.push(Event::DrawStopped {
            player,
            missing: count - deck.len(),
        });
        return events;
    }
    events.push(Event::TurnStarted {
        player,
        turn: state.turn + 1,
    });
    events
}
