//! Actions légales des cartes : jouer une carte d'action, de côté ou renforcée, avec ses choix.

use super::*;

/// Pour chaque carte de la main, ses options d'effet de base jouables, ses options d'effet
/// renforcé avec chaque paiement possible (spec 022), puis de côté Mouvement 1 et Influence 1
/// (spec 013).
pub(super) fn card_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    let combat = match moment(p) {
        Moment::Fighting(combat) => Some(combat),
        Moment::Free | Moment::Engaging => None,
    };
    let playable =
        |effect: Effect, modifiers: &[ModifierKind]| effect_playable(state, p, effect, modifiers);
    // Spec 040 et issue privée 164 : la phase de mouvement finie, plus de mouvement de côté non plus ;
    // spec 057 (Q4) : en combat, si Ailes de la nuit peut être lancé.
    let fly = combat.is_some() && wings_of_night_castable(state, p);
    let movement_over = movement_over(p.movement_ended, p.interaction.is_some()) && !fly;
    let mut actions = Vec::new();
    let sideways_kinds: Vec<SidewaysEffect> = sideways_effects(combat, fly)
        .into_iter()
        .filter(|sideways| !(movement_over && *sideways == SidewaysEffect::Move))
        .collect();
    // Les Blessures sont identiques : la première seule se propose (spec 058).
    let first_wound = p.hand.iter().copied().find(|card| card.card().is_wound());
    for &card in &p.hand {
        let play = |play| Action::PlayCard { card, play };
        // Spec 058 : de côté avec une compétence (Q6, la carte porte la compétence).
        if !card.card().is_wound() || Some(card) == first_wound {
            actions.extend(
                skill_sideways_plays(state, p, card, &sideways_kinds)
                    .into_iter()
                    .map(play),
            );
        }
        // MKUE p. 4, 3.d : une Blessure ne se joue d'aucune façon, pas même de côté (sauf avec une
        // compétence, spec 058).
        if card.card().is_wound() {
            continue;
        }
        // Spec 055 : un artefact, sans couleur, se joue pour son effet de base ou se détruit pour
        // son effet renforcé, sans mana.
        if let DeedCard::Artifact(artifact) = card.card() {
            actions.extend(artifact_actions(state, card, artifact, p, playable));
        }
        // Spec 041 (C27) : un artefact se joue aussi de côté.
        let Some(definition) = card.card().definition() else {
            actions.extend(
                sideways_effects(combat, fly)
                    .into_iter()
                    .filter(|sideways| !(movement_over && *sideways == SidewaysEffect::Move))
                    .map(|sideways| play(CardPlay::Sideways(sideways))),
            );
            continue;
        };
        // Spec 054 : un sort se lance, payé de sa couleur (et du noir, renforcé).
        if card.card().is_spell() {
            actions.extend(cast_actions(state, p, card, definition, false));
            actions.extend(
                sideways_effects(combat, fly)
                    .into_iter()
                    .filter(|sideways| !(movement_over && *sideways == SidewaysEffect::Move))
                    .map(|sideways| play(CardPlay::Sideways(sideways))),
            );
            continue;
        }
        // Spec 034 (Q3) : une carte de soin ne joue aucun effet en combat, seulement de côté.
        let healing_in_combat = definition.healing && combat.is_some();
        let playable = |effect: Effect, modifiers: &[ModifierKind]| {
            !healing_in_combat && playable(effect, modifiers)
        };
        for (option, discard) in playable_options(definition.basic, card, &p.hand, playable) {
            // Spec 056 : Effet maximum, avec chaque cible et son option répétable.
            if let Some(Effect::Repeat(_)) = definition.basic.option(option) {
                actions.extend(
                    maximal_targets(p, card, false, combat.is_some(), playable)
                        .into_iter()
                        .map(|with| {
                            play(CardPlay::Basic {
                                option,
                                discard,
                                mana: None,
                                with: Some(with),
                                gain: None,
                                die: None,
                            })
                        }),
                );
                continue;
            }
            // Spec 025 : une option qui coûte du mana, avec chaque paiement possible.
            match definition.basic.option_mana(option) {
                Some(color) => {
                    actions.extend(mana_payments(state, p, color).into_iter().map(|mana| {
                        play(CardPlay::Basic {
                            option,
                            discard,
                            mana: Some(mana),
                            with: None,
                            gain: None,
                            die: None,
                        })
                    }))
                }
                None => actions.push(play(CardPlay::Basic {
                    option,
                    discard,
                    mana: None,
                    with: None,
                    gain: None,
                    die: None,
                })),
            }
        }
        let payments = mana_payments(state, p, definition.color);
        for (option, discard) in playable_options(definition.strong, card, &p.hand, playable) {
            let effect = definition.strong.option(option);
            for &mana in &payments {
                // Spec 056 : Effet maximum renforcé, avec chaque cible et son option renforcée.
                if let Some(Effect::Repeat(_)) = effect {
                    let targets = maximal_targets(p, card, true, combat.is_some(), playable);
                    actions.extend(targets.into_iter().map(|with| {
                        play(CardPlay::Strong {
                            extra_mana: None,
                            gain: None,
                            option,
                            discard,
                            mana,
                            set_dice: [None, None],
                            with: Some(with),
                        })
                    }));
                    continue;
                }
                if let Some(Effect::Boost(_)) = effect {
                    // Spec 025 : chaque carte jouable avec Concentration, et ses propres choix.
                    let targets = boost_targets(state, p, card, mana, combat.is_some(), playable);
                    actions.extend(targets.into_iter().map(|with| {
                        play(CardPlay::Strong {
                            extra_mana: None,
                            gain: None,
                            option,
                            discard,
                            mana,
                            set_dice: [None, None],
                            with: Some(with),
                        })
                    }));
                    continue;
                }
                // Spec 053 : une option qui coûte un mana en plus, chaque façon de payer les deux.
                if let Some(extra) = definition.strong.option_mana(option) {
                    extra_mana_plays(
                        &mut actions,
                        state,
                        p,
                        card,
                        option,
                        discard,
                        mana,
                        definition.color,
                        extra,
                    );
                    continue;
                }
                // Spec 025 : chaque dé libre (ou paire, dans l'ordre de la Source), hors celui qui
                // paie, et chaque couleur de base.
                for set_dice in dice_settings(state, p, mana, set_dice_count(effect)) {
                    actions.push(play(CardPlay::Strong {
                        extra_mana: None,
                        gain: None,
                        option,
                        discard,
                        mana,
                        set_dice,
                        with: None,
                    }));
                }
            }
        }
        // MKUE p. 4, 3.e : de côté, ce que le moment permet.
        actions.extend(
            sideways_effects(combat, fly)
                .into_iter()
                .filter(|sideways| !(movement_over && *sideways == SidewaysEffect::Move))
                .map(|sideways| play(CardPlay::Sideways(sideways))),
        );
    }
    // Spec 055 (MKUE p. 5, 8 ; Compendium, *Artifact Cards* S1) : une bannière de la main
    // s'assigne à toute unité, disponible ou épuisée, blessée ou non, à tout moment du tour.
    for &card in &p.hand {
        if matches!(card.card(), DeedCard::Artifact(artifact) if artifact.is_banner()) {
            actions.extend(p.units.iter().map(|owned| Action::AssignBanner {
                card,
                unit: owned.card,
            }));
        }
    }
    // Spec 054 : avec Don pour la magie, un sort de sa couleur de l'offre de sorts se lance comme
    // s'il était en main.
    let mut colors: Vec<BasicColor> = Vec::new();
    for modifier in &p.modifiers {
        if let ModifierKind::CastFromOffer(color) = modifier.kind
            && !colors.contains(&color)
        {
            colors.push(color);
        }
    }
    for color in colors {
        for card in offer_cards_of_color(state, Offer::Spells, color) {
            if let Some(definition) = card.card().definition() {
                actions.extend(cast_actions(state, p, card, definition, true));
            }
        }
    }
    // Spec 042 : un effet qui rend disponible une unité se joue sur chaque unité qu'il peut viser,
    // et pas du tout sans cible.
    actions
        .into_iter()
        .flat_map(|action| match action {
            Action::PlayCard { card, play } => match card.effect(play) {
                Some(Effect::ReadyUnit(level)) => ready_targets(p, level)
                    .into_iter()
                    .map(|target| Action::PlayCardOnUnit { card, play, target })
                    .collect(),
                // Spec 053 : Régénération sur chaque unité qu'elle peut viser, ou pour son Soin
                // seul sans cible.
                Some(Effect::HealAndReady { level, .. }) => {
                    let targets = ready_targets(p, level);
                    if targets.is_empty() {
                        vec![Action::PlayCard { card, play }]
                    } else {
                        targets
                            .into_iter()
                            .map(|target| Action::PlayCardOnUnit { card, play, target })
                            .collect()
                    }
                }
                // Spec 056 : Tempête de mana, chaque dé de la Source de couleur de base, ni pris ni
                // mis ce tour.
                Some(Effect::CrystalFromDie) => (0u8..)
                    .zip(&state.source)
                    .filter(|&(die, face)| face.is_basic() && die_free(p, die))
                    .map(|(die, _)| Action::PlayCard {
                        card,
                        play: with_die(play, die),
                    })
                    .collect(),
                // Spec 056 : Maîtrise du cristal, une couleur déjà possédée.
                Some(Effect::GainOwnedCrystal(color)) if p.inventory.crystals(color) == 0 => {
                    Vec::new()
                }
                // Spec 056 : Entraînement, une action avancée de la couleur de la carte détruite ;
                // spec 055 : le Livre de sagesse a ses choix.
                Some(Effect::TrainFromOffer(_))
                    if !matches!(play, CardPlay::Artifact { .. } | CardPlay::ThrowAway { .. }) =>
                {
                    play_discard(play)
                        .and_then(|destroyed| destroyed.card().definition())
                        .map(|definition| {
                            offer_cards_of_color(state, Offer::AdvancedActions, definition.color)
                        })
                        .unwrap_or_default()
                        .into_iter()
                        .map(|gain| Action::PlayCard {
                            card,
                            play: with_gain(play, gain),
                        })
                        .collect()
                }
                // Spec 054 : Don pour la magie de base, chaque carte de couleur de la main à
                // défausser, autre que lui (carte, version Lost Legion ; Compendium, *Magic
                // Talent* S3), s'il y a dans l'offre un sort codé de sa couleur (effet utilisable,
                // spec 052 ; comme une offre vide, spec 056, L5).
                Some(Effect::CastFromOffer) => p
                    .hand
                    .iter()
                    .filter(|&&other| {
                        other != card
                            && other.card().definition().is_some_and(|definition| {
                                offer_cards_of_color(state, Offer::Spells, definition.color)
                                    .iter()
                                    .any(|spell| {
                                        spell.card().definition().is_some_and(|spell| {
                                            spell.basic != BasicEffect::Unsupported
                                        })
                                    })
                            })
                    })
                    .map(|&other| Action::PlayCard {
                        card,
                        play: with_discard(play, other),
                    })
                    .collect(),
                // Spec 057 : Renaissance, chaque ensemble d'unités épuisées dans la limite de
                // niveaux (5 en forêt), aucune comprise ; Flux d'énergie, chaque unité épuisée.
                Some(Effect::HealAndReadyLevels {
                    levels,
                    forest_levels,
                    ..
                }) => {
                    let limit = if hero_in_forest(state, p) {
                        forest_levels
                    } else {
                        levels
                    };
                    let exhausted = ready_targets(p, u8::MAX);
                    (0..=exhausted.len())
                        .flat_map(|size| combinations(&exhausted, size))
                        .filter(|units| {
                            units
                                .iter()
                                .map(|unit| u32::from(unit.unit.definition().level))
                                .sum::<u32>()
                                <= u32::from(limit)
                        })
                        .map(|units| Action::PlayCard {
                            card,
                            play: with_units(play, &units),
                        })
                        .collect()
                }
                // Spec 057 : Ailes du vent, chaque arrivée payable ; Ailes de la nuit, chaque
                // ensemble d'ennemis payable.
                // Spec 057 : Puits de mana, chaque dé de couleur de base libre de la Source, sauf
                // celui qui paie le sort.
                Some(Effect::ClaimDie { .. }) => {
                    let paid = match play {
                        CardPlay::Cast { mana, black, .. } => [Some(mana), black],
                        _ => [None, None],
                    };
                    (0u8..)
                        .zip(&state.source)
                        .filter(|&(die, face)| {
                            face.is_basic()
                                && die_free(p, die)
                                && !paid.contains(&Some(ManaPayment::Die(die)))
                        })
                        .map(|(die, _)| Action::PlayCard {
                            card,
                            play: with_claimed_die(play, die),
                        })
                        .collect()
                }
                Some(Effect::FlyMove { max }) => fly_destinations(state, p, max)
                    .into_iter()
                    .map(|(hex, _)| Action::PlayCard {
                        card,
                        play: with_destination(play, hex),
                    })
                    .collect(),
                Some(Effect::DoesNotAttackPerMove) => {
                    let targets = match moment(p) {
                        Moment::Fighting(combat) => unit_enemy_targets(
                            combat,
                            &[ModifierKind::OnEnemy(EnemyModifierKind::DoesNotAttack)],
                        ),
                        _ => Vec::new(),
                    };
                    (1..=targets.len().min(crate::PICKS))
                        .filter(|&size| wings_of_night_cost(size) <= p.move_points)
                        .flat_map(|size| combinations(&targets, size))
                        .map(|enemies| Action::PlayCard {
                            card,
                            play: with_enemies(play, &enemies),
                        })
                        .collect()
                }
                // Spec 057 : Voyage souterrain, chaque arrivée.
                Some(Effect::UndergroundMove { assault }) => {
                    underground_destinations(state, p, assault)
                        .into_iter()
                        .map(|hex| Action::PlayCard {
                            card,
                            play: with_destination(play, hex),
                        })
                        .collect()
                }
                Some(Effect::ReadyAnyUnit { .. }) => ready_targets(p, u8::MAX)
                    .into_iter()
                    .map(|unit| Action::PlayCard {
                        card,
                        play: with_units(play, &[unit]),
                    })
                    .collect(),
                // Spec 056 : Don pour la magie, un sort de la couleur payée.
                Some(Effect::GainSpell(color)) => offer_cards_of_color(state, Offer::Spells, color)
                    .into_iter()
                    .map(|gain| Action::PlayCard {
                        card,
                        play: with_gain(play, gain),
                    })
                    .collect(),
                _ => vec![Action::PlayCard { card, play }],
            },
            other => vec![other],
        })
        .collect()
}

/// Vrai si l'option (effet et modificateurs posés) se joue maintenant : en combat selon sa phase
/// (spec 031), sinon hors combat, sans mouvement pendant une interaction (spec 040) ; ou si l'un
/// de ses modificateurs sert (spec 052).
pub(super) fn effect_playable(
    state: &GameState,
    p: &PlayerState,
    effect: Effect,
    modifiers: &[ModifierKind],
) -> bool {
    // Spec 058 (L9) : après Magie inutile à +3, plus de dé en plus.
    if p.source_dice_forbidden && matches!(effect, Effect::ExtraDie | Effect::ExtraDieBlackAnyColor)
    {
        return false;
    }
    is_playable_now(effect, modifiers, play_moment(state, p))
}

/// Vrai si l'offre d'unités a une unité (spec 057), pas seulement des actions avancées de monastère.
fn unit_offer(state: &GameState) -> bool {
    state
        .offer(Offer::Units)
        .iter()
        .any(|card| card.unit().is_some())
}

/// Le moment de jeu du joueur, pour `is_playable_now`.
fn play_moment<'a>(state: &GameState, p: &'a PlayerState) -> PlayMoment<'a> {
    let combat = match moment(p) {
        Moment::Fighting(combat) => Some(combat),
        Moment::Free | Moment::Engaging => None,
    };
    PlayMoment {
        combat,
        movement_ended: p.movement_ended,
        interacting: p.interaction.is_some(),
        modifiers: &p.modifiers,
        units_fight: !p.units.is_empty() && !p.units_sit_out(),
        fly_in_combat: wings_of_night_castable(state, p),
        extra_turn: p.extra_turn,
        unit_offer: unit_offer(state),
        offer_units_fight: unit_offer(state) && !(combat.is_some() && p.units_sit_out()),
    }
}

/// Vrai si la case du héros est une forêt, site compris (Restauration, spec 057).
pub(in crate::engine) fn hero_in_forest(state: &GameState, p: &PlayerState) -> bool {
    p.figure
        .and_then(|hex| state.map.space(hex))
        .is_some_and(|space| space.terrain == crate::Terrain::Forest)
}

/// Hors combat, Mouvement 1 et Influence 1 (spec 013) ; en combat, ni l'un ni l'autre (MKUE
/// p. 6, 6.e), mais Attaque 1 dans une attaque (spec 031).
fn sideways_effects(combat: Option<&Combat>, fly: bool) -> Vec<SidewaysEffect> {
    match combat {
        None => vec![SidewaysEffect::Move, SidewaysEffect::Influence],
        // Spec 057 (Q4) : Mouvement 1 de côté, pour Ailes de la nuit.
        Some(combat) if fly => {
            let mut effects = vec![SidewaysEffect::Move];
            effects.extend(sideways_in_combat(combat));
            effects
        }
        Some(combat) => sideways_in_combat(combat),
    }
}

/// Le nombre de dés que l'effet met sur une couleur (spec 025).
fn set_dice_count(effect: Option<Effect>) -> usize {
    match effect {
        Some(Effect::SetDice { dice, .. }) => usize::from(dice),
        _ => 0,
    }
}

/// Les cibles d'Effet maximum (spec 056, Q1) dans l'ordre de la main : une autre carte action, avec
/// chaque option (de base, ou renforcée si `strong`) répétable qui sert maintenant, sans
/// modificateur, ni mana en plus, ni carte à défausser ou détruire.
fn maximal_targets(
    p: &PlayerState,
    maximal: CardId,
    strong: bool,
    in_combat: bool,
    playable: impl Fn(Effect, &[ModifierKind]) -> bool + Copy,
) -> Vec<BoostTarget> {
    let mut targets = Vec::new();
    for &card in p.hand.iter().filter(|&&other| other != maximal) {
        if !is_action_card(card) {
            continue;
        }
        let Some(definition) = card.card().definition() else {
            continue;
        };
        if definition.healing && in_combat {
            continue;
        }
        let effect = if strong {
            definition.strong
        } else {
            definition.basic
        };
        let BasicEffect::Options { options, cost, .. } = effect else {
            continue;
        };
        if cost != Cost::None {
            continue;
        }
        for (option, &points) in (0u8..).zip(options) {
            if points.is_repeatable()
                && effect.option_modifiers(option).is_empty()
                && effect.option_mana(option).is_none()
                && playable(points, &[])
            {
                targets.push(BoostTarget {
                    card,
                    option,
                    discard: None,
                    set_dice: [None, None],
                });
            }
        }
    }
    targets
}

/// Les cartes jouables avec `concentration` renforcée payée par `mana`, dans l'ordre de la main,
/// chacune avec chaque option renforcée, carte défaussée et dé mis possibles (spec 025).
fn boost_targets(
    state: &GameState,
    p: &PlayerState,
    concentration: CardId,
    mana: ManaPayment,
    in_combat: bool,
    playable: impl Fn(Effect, &[ModifierKind]) -> bool + Copy,
) -> Vec<BoostTarget> {
    // Concentration ne peut pas être la carte défaussée par la cible.
    let hand: Vec<CardId> = p
        .hand
        .iter()
        .copied()
        .filter(|&other| other != concentration)
        .collect();
    let mut targets = Vec::new();
    for &card in &hand {
        // Spec 054 : un sort n'est jamais la cible de Concentration (Compendium, *Basic Action
        // Cards* S6).
        if card.card().is_spell() {
            continue;
        }
        let Some(definition) = card.card().definition() else {
            continue;
        };
        // Spec 034 : une carte de soin ne se renforce pas en combat (MKUE p. 5, *Composite* 1.b).
        if definition.healing && in_combat {
            continue;
        }
        let strong = definition.strong;
        // Q5 : pas de Concentration sur une autre Concentration ; spec 056, Q3 : ni sur Effet
        // maximum.
        if let BasicEffect::Options { options, .. } = strong
            && options
                .iter()
                .any(|effect| matches!(effect, Effect::Boost(_) | Effect::Repeat(_)))
        {
            continue;
        }
        // Spec 056 : Entraînement prend une carte d'une offre, que Concentration ne choisit pas.
        let playable = |effect: Effect, modifiers: &[ModifierKind]| {
            !matches!(
                effect,
                Effect::ReadyUnit(_) | Effect::HealAndReady { .. } | Effect::TrainFromOffer(_)
            ) && playable(effect, modifiers)
        };
        for (option, discard) in playable_options(strong, card, &hand, playable) {
            // Spec 053 : une option qui coûte un mana en plus ne se joue pas par Concentration,
            // qui n'a pas de second paiement.
            if strong.option_mana(option).is_some() {
                continue;
            }
            let count = set_dice_count(strong.option(option));
            targets.extend(
                dice_settings(state, p, mana, count)
                    .into_iter()
                    .map(|set_dice| BoostTarget {
                        card,
                        option,
                        discard,
                        set_dice,
                    }),
            );
        }
    }
    targets
}

/// Les options `playable` d'un effet de `card`, chacune avec chaque carte à défausser si
/// l'option le demande.
fn playable_options(
    effect: BasicEffect,
    card: CardId,
    hand: &[CardId],
    playable: impl Fn(Effect, &[ModifierKind]) -> bool,
) -> Vec<(u8, Option<CardId>)> {
    let BasicEffect::Options { options, cost, .. } = effect else {
        return Vec::new();
    };
    let mut plays = Vec::new();
    for (option, points) in (0u8..).zip(options) {
        // MKUE p. 6, 5.c : hors combat, pas d'attaque ni de parade ; en combat, spec 031 ; ou un
        // modificateur qui sert (spec 052).
        if !playable(*points, effect.option_modifiers(option)) {
            continue;
        }
        match cost {
            Cost::None => plays.push((option, None)),
            // MKUE p. 4, 6 : la carte défaussée est une autre carte de la main, jamais une
            // Blessure (6.a).
            // Spec 056 : une autre carte action de la main (MKUE p. 4, 6 et 6.a ; Compendium,
            // *Training* S2).
            Cost::DestroyActionCard => plays.extend(
                hand.iter()
                    .filter(|&&other| other != card && is_action_card(other))
                    .map(|&other| (option, Some(other))),
            ),
            Cost::DiscardAnotherCard => plays.extend(
                hand.iter()
                    .filter(|&&other| other != card && !other.card().is_wound())
                    .map(|&other| (option, Some(other))),
            ),
        }
    }
    plays
}

/// Une carte action : une action de base ou avancée, jamais une Blessure, un sort ni un artefact
/// (MKUE p. 4, 6.a ; spec 056).
pub(super) fn is_action_card(card: CardId) -> bool {
    matches!(card.card(), DeedCard::Basic(_) | DeedCard::Advanced(_))
}

/// La carte d'une offre de cette couleur, dans l'ordre de l'offre (spec 056).
pub(super) fn offer_cards_of_color(
    state: &GameState,
    offer: Offer,
    color: BasicColor,
) -> Vec<CardId> {
    state
        .offer(offer)
        .iter()
        .filter_map(|card| card.deed())
        .filter(|card| {
            card.card()
                .definition()
                .is_some_and(|definition| definition.color == color)
        })
        .collect()
}

/// La façon de jouer `play`, avec `gain` pour carte prise dans une offre (spec 056).
fn with_gain(play: CardPlay, gain: CardId) -> CardPlay {
    match play {
        CardPlay::Basic {
            option,
            discard,
            mana,
            with,
            die,
            ..
        } => CardPlay::Basic {
            option,
            discard,
            mana,
            with,
            gain: Some(gain),
            die,
        },
        CardPlay::Strong {
            option,
            discard,
            mana,
            set_dice,
            with,
            extra_mana,
            ..
        } => CardPlay::Strong {
            option,
            discard,
            mana,
            set_dice,
            with,
            extra_mana,
            gain: Some(gain),
        },
        other => other,
    }
}

/// La façon de jouer l'effet de base `play`, avec `discard` pour carte défaussée (spec 054).
fn with_discard(play: CardPlay, discard: CardId) -> CardPlay {
    match play {
        CardPlay::Basic {
            option,
            mana,
            with,
            gain,
            die,
            ..
        } => CardPlay::Basic {
            option,
            discard: Some(discard),
            mana,
            with,
            gain,
            die,
        },
        other => other,
    }
}

/// La façon de jouer `play`, avec le dé `die` choisi dans la Source (spec 056).
fn with_die(play: CardPlay, die: u8) -> CardPlay {
    match play {
        CardPlay::Basic {
            option,
            discard,
            mana,
            with,
            gain,
            ..
        } => CardPlay::Basic {
            option,
            discard,
            mana,
            with,
            gain,
            die: Some(die),
        },
        other => other,
    }
}

/// La carte à défausser ou détruire d'une façon de jouer.
pub(super) fn play_discard(play: CardPlay) -> Option<CardId> {
    match play {
        CardPlay::Basic { discard, .. }
        | CardPlay::Strong { discard, .. }
        | CardPlay::Boosted { discard, .. } => discard,
        CardPlay::Sideways(_)
        | CardPlay::SidewaysSkill { .. }
        | CardPlay::Cast { .. }
        | CardPlay::Artifact { .. }
        | CardPlay::ThrowAway { .. } => None,
    }
}

/// La carte prise dans une offre d'une façon de jouer (spec 056).
pub(in crate::engine) fn play_gain(play: CardPlay) -> Option<CardId> {
    match play {
        CardPlay::Basic { gain, .. } | CardPlay::Strong { gain, .. } => gain,
        // Spec 055 : le Livre de sagesse.
        CardPlay::Artifact { choices, .. } | CardPlay::ThrowAway { choices, .. } => choices.gain,
        CardPlay::Boosted { .. }
        | CardPlay::Sideways(_)
        | CardPlay::SidewaysSkill { .. }
        | CardPlay::Cast { .. } => None,
    }
}

/// Les façons de mettre `count` dés sur une couleur (spec 025) : les dés ni pris ni déjà mis, même
/// épuisés (Q2), sauf celui qui paie, dans l'ordre de la Source ; pour chacun, chaque couleur de
/// base (jamais l'or, ni le noir : Q4). Sans dé à mettre, une seule façon : aucun.
fn dice_settings(
    state: &GameState,
    p: &PlayerState,
    payment: ManaPayment,
    count: usize,
) -> Vec<[Option<DieSetting>; 2]> {
    let free: Vec<u8> = (0u8..)
        .zip(&state.source)
        .map(|(die, _)| die)
        .filter(|die| die_free(p, *die) && payment != ManaPayment::Die(*die))
        .collect();
    let settings = |die| BasicColor::ALL.map(|color| DieSetting { die, color });
    match count {
        0 => vec![[None, None]],
        1 => free
            .iter()
            .flat_map(|&die| settings(die).map(|first| [Some(first), None]))
            .collect(),
        2 => {
            let mut all = Vec::new();
            for (index, &first_die) in free.iter().enumerate() {
                for &second_die in &free[index + 1..] {
                    for first in settings(first_die) {
                        for second in settings(second_die) {
                            all.push([Some(first), Some(second)]);
                        }
                    }
                }
            }
            all
        }
        // Le chargement du contenu refuse plus de deux dés.
        _ => Vec::new(),
    }
}
