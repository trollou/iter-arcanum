//! Actions légales des sorts : lancer un sort, ses paiements et ses cibles (ennemis, arrivée, unités, dé).

use super::*;

/// Les effets renforcés de l'option `option` de `card` payés par `mana` (la couleur de la carte),
/// avec chaque façon de payer ensuite le mana `extra` de l'option (Magie pure, Mélodie du vent ;
/// spec 053).
#[allow(clippy::too_many_arguments)]
pub(super) fn extra_mana_plays(
    actions: &mut Vec<Action>,
    state: &GameState,
    p: &PlayerState,
    card: CardId,
    option: u8,
    discard: Option<CardId>,
    mana: ManaPayment,
    color: BasicColor,
    extra: BasicColor,
) {
    for list in payment_lists(state, p, &[color, extra]) {
        if let [first, second] = list[..]
            && first == mana
        {
            actions.push(Action::PlayCard {
                card,
                play: CardPlay::Strong {
                    option,
                    discard,
                    mana,
                    set_dice: [None, None],
                    with: None,
                    extra_mana: Some(second),
                    gain: None,
                },
            });
        }
    }
}

/// Les façons de lancer le sort `card` (spec 054), dans l'ordre : effet de base puis renforcé,
/// leurs options, les paiements (couleur du sort, noir, mana en plus de l'option), puis les
/// ennemis visés. Depuis l'offre de sorts si `from_offer` (Don pour la magie).
pub(super) fn cast_actions(
    state: &GameState,
    p: &PlayerState,
    card: CardId,
    definition: CardDefinition,
    from_offer: bool,
) -> Vec<Action> {
    let mut actions = Vec::new();
    for strong in [false, true] {
        let effect = if strong {
            definition.strong
        } else {
            definition.basic
        };
        let BasicEffect::Options { options, .. } = effect else {
            continue;
        };
        for (option, &points) in (0u8..).zip(options) {
            let kinds = effect.option_modifiers(option);
            if !effect_playable(state, p, points, kinds) {
                continue;
            }
            let enemies = cast_enemies(p, points, kinds, definition.color);
            let payments = cast_payments(
                state,
                p,
                definition.color,
                strong,
                effect.option_mana(option),
            );
            for (mana, black, extra_mana) in payments {
                actions.extend(enemies.iter().map(|&enemy| Action::PlayCard {
                    card,
                    play: CardPlay::Cast {
                        option,
                        mana,
                        black,
                        extra_mana,
                        enemy,
                        from_offer,
                        targets: CastTargets::default(),
                    },
                }));
            }
        }
    }
    actions
}

/// Les ennemis que vise l'option d'un sort (spec 054) : `None` seul si elle n'en vise pas ; aucun
/// hors combat ou sans cible.
fn cast_enemies(
    p: &PlayerState,
    effect: Effect,
    kinds: &[ModifierKind],
    color: BasicColor,
) -> Vec<Option<EnemyToken>> {
    let destroys = effect == Effect::DestroyEnemy;
    if !destroys && !kinds.iter().any(|kind| kind.targets_enemy()) {
        return vec![None];
    }
    let Moment::Fighting(combat) = moment(p) else {
        return Vec::new();
    };
    let enemy_kinds: Vec<EnemyModifierKind> = kinds
        .iter()
        .filter_map(|kind| match *kind {
            ModifierKind::OnTargetEnemy(kind) => Some(kind),
            ModifierKind::OnTargetEnemyByFortification { plain, .. } => Some(plain),
            _ => None,
        })
        .collect();
    spell_enemy_targets(combat, &enemy_kinds, destroys, color)
        .into_iter()
        .map(Some)
        .collect()
}

/// Les façons de payer un sort de `color` (spec 054 ; MKUE p. 4, 3.b ; p. 5, 2.b) : sa couleur,
/// puis le noir pour l'effet renforcé, puis le mana `extra` de l'option (Éclair de mana).
fn cast_payments(
    state: &GameState,
    p: &PlayerState,
    color: BasicColor,
    strong: bool,
    extra: Option<BasicColor>,
) -> Vec<(ManaPayment, Option<ManaPayment>, Option<ManaPayment>)> {
    let mut result = Vec::new();
    for mana in mana_payments(state, p, color) {
        let after = after_payment(state, p, mana, ManaColor::Basic(color));
        let blacks: Vec<Option<ManaPayment>> = if strong {
            black_payments(state, &after)
                .into_iter()
                // Spec 058 : de nuit, l'or polarisé paie le noir d'un sort renforcé.
                .chain(polarized_payments(state, &after, ManaColor::Black))
                .map(Some)
                .collect()
        } else {
            vec![None]
        };
        for black in blacks {
            let after = match black {
                Some(payment) => after_payment(state, &after, payment, ManaColor::Black),
                None => after.clone(),
            };
            match extra {
                None => result.push((mana, black, None)),
                Some(extra) => result.extend(
                    mana_payments(state, &after, extra)
                        .into_iter()
                        .map(|payment| (mana, black, Some(payment))),
                ),
            }
        }
    }
    result
}

/// Vrai si Ailes de la nuit peut être lancé maintenant (spec 057, Q4) : en main, ou dans l'offre
/// avec Don pour la magie blanc ; en phase à distance ou de parade, un ennemi à viser, le mana
/// blanc et noir payables.
pub(crate) fn wings_of_night_castable(state: &GameState, p: &PlayerState) -> bool {
    let Moment::Fighting(combat) = moment(p) else {
        return false;
    };
    if !matches!(
        combat.phase,
        crate::CombatPhase::RangedAndSiege | crate::CombatPhase::Block
    ) {
        return false;
    }
    let wings = DeedCard::Spell(crate::Spell::WingsOfWind);
    let in_hand = p.hand.iter().any(|card| card.card() == wings);
    let from_offer = p
        .modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::CastFromOffer(BasicColor::White))
        && state
            .offer(Offer::Spells)
            .iter()
            .any(|card| card.deed().is_some_and(|card| card.card() == wings));
    (in_hand || from_offer)
        && !unit_enemy_targets(
            combat,
            &[ModifierKind::OnEnemy(EnemyModifierKind::DoesNotAttack)],
        )
        .is_empty()
        && !cast_payments(state, p, BasicColor::White, true, None).is_empty()
}

/// Les arrivées de Voyage souterrain (spec 057 ; carte ; Compendium, *Underground Travel* S1 à
/// S4) : 3 cases au plus par des cases révélées, ni marais ni lac (le littoral compte comme un
/// lac) ; une case sécurisée sans ennemi errant, ou pour l'Attaque souterraine un site fortifié non
/// conquis (*Underground Attack* S1, S3).
pub(super) fn underground_destinations(
    state: &GameState,
    p: &PlayerState,
    assault: bool,
) -> Vec<Hex> {
    let Some(from) = p.figure else {
        return Vec::new();
    };
    let rampaging = rampaging_enemies(state);
    let conquered = state.conquered_sites();
    crate::reach(&state.map, from, 3, |space| {
        !matches!(
            space.terrain,
            crate::Terrain::Swamp | crate::Terrain::Lake | crate::Terrain::Sea
        )
    })
    .into_iter()
    .map(|(hex, _)| hex)
    .filter(|&hex| {
        if assault {
            is_assault_target(state, hex)
        } else {
            is_safe_space(&state.map, hex, &conquered) && !rampaging.contains(&hex)
        }
    })
    .collect()
}

/// Les arrivées d'Ailes du vent et leur coût (spec 057 ; carte ; Compendium, *Wings of Wind* S2 à
/// S5) : `max` cases au plus et pas plus que les points de Mouvement, par toutes les cases révélées ;
/// une case sécurisée sans ennemi errant.
pub(in crate::engine) fn fly_destinations(
    state: &GameState,
    p: &PlayerState,
    max: u8,
) -> Vec<(Hex, u8)> {
    let Some(from) = p.figure else {
        return Vec::new();
    };
    let points = u8::try_from(p.move_points).unwrap_or(u8::MAX).min(max);
    let rampaging = rampaging_enemies(state);
    let conquered = state.conquered_sites();
    crate::reach(&state.map, from, points, |_| true)
        .into_iter()
        .filter(|&(hex, _)| is_safe_space(&state.map, hex, &conquered) && !rampaging.contains(&hex))
        .collect()
}

/// Le Mouvement que coûtent `count` ennemis visés par Ailes de la nuit (spec 057) : 0 pour le
/// premier, 1 pour le deuxième, 2 pour le troisième…
pub fn wings_of_night_cost(count: usize) -> u32 {
    let count = u32::try_from(count).unwrap_or(u32::MAX);
    count.saturating_mul(count.saturating_sub(1)) / 2
}

/// Le sort lancé `play` visant les ennemis `enemies` (spec 057).
pub(super) fn with_enemies(play: CardPlay, enemies: &[EnemyToken]) -> CardPlay {
    match play {
        CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets,
        } => CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets: CastTargets {
                enemies: crate::Picks::of(enemies),
                ..targets
            },
        },
        other => other,
    }
}

/// Le sort lancé `play` vers l'arrivée `hex` (spec 057).
pub(super) fn with_destination(play: CardPlay, hex: Hex) -> CardPlay {
    match play {
        CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets,
        } => CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets: CastTargets {
                destination: Some(hex),
                ..targets
            },
        },
        other => other,
    }
}

/// Le sort lancé `play` sur le dé `die` de la Source (Puits de mana, spec 057).
pub(super) fn with_claimed_die(play: CardPlay, die: u8) -> CardPlay {
    match play {
        CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets,
        } => CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets: CastTargets {
                die: Some(die),
                ..targets
            },
        },
        other => other,
    }
}

/// Le sort lancé `play` avec les unités visées `units` (spec 057).
pub(super) fn with_units(play: CardPlay, units: &[UnitCard]) -> CardPlay {
    match play {
        CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets,
        } => CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer,
            targets: CastTargets {
                units: crate::Picks::of(units),
                ..targets
            },
        },
        other => other,
    }
}
