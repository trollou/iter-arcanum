//! Actions légales des artefacts et des bannières.

use super::*;

/// Les façons de jouer l'artefact `card` pour son effet (spec 055) : chaque option de base
/// utilisable maintenant (pas celle d'une bannière, qui s'assigne), chaque option renforcée en le
/// détruisant.
pub(super) fn artifact_actions(
    state: &GameState,
    card: CardId,
    artifact: Artifact,
    p: &PlayerState,
    playable: impl Fn(Effect, &[ModifierKind]) -> bool,
) -> Vec<Action> {
    let (basic, strong) = GameContent::embedded().artifact(artifact);
    let mut actions = Vec::new();
    for (effect, strong) in [(basic, false), (strong, true)] {
        for option in 0u8.. {
            let Some(option_effect) = effect.option(option) else {
                break;
            };
            if option_effect == Effect::BannerAssigned
                || !playable(option_effect, effect.option_modifiers(option))
            {
                continue;
            }
            let play = |choices| {
                if strong {
                    CardPlay::ThrowAway { option, choices }
                } else {
                    CardPlay::Artifact { option, choices }
                }
            };
            let choices: Vec<ArtifactChoices> = match option_effect {
                // Spec 055 : les couleurs des jetons de l'Amulette des ténèbres, sans ordre.
                Effect::AmuletOfDarkness { tokens } => {
                    multisets(&ManaColor::ALL, usize::from(tokens))
                        .into_iter()
                        .map(|colors| ArtifactChoices {
                            colors: crate::Picks::of(&colors),
                            ..ArtifactChoices::default()
                        })
                        .collect()
                }
                // Spec 055 (Bannière de peur renforcée) : 1 à `count` ennemis qui attaquent
                // encore, comme Tourbillon (L3).
                Effect::CancelAttacks(count) => {
                    let targets = match moment(p) {
                        Moment::Fighting(combat) => unit_enemy_targets(
                            combat,
                            &[ModifierKind::OnEnemy(EnemyModifierKind::DoesNotAttack)],
                        ),
                        _ => Vec::new(),
                    };
                    (1..=targets.len().min(usize::from(count)))
                        .flat_map(|size| combinations(&targets, size))
                        .map(|enemies| ArtifactChoices {
                            enemies: crate::Picks::of(&enemies),
                            ..ArtifactChoices::default()
                        })
                        .collect()
                }
                // Spec 055 (Épée de justice ; Q8) : toute partie de la main sans Blessure ni l'Épée,
                // vide comprise.
                Effect::SwordDiscard => {
                    let cards: Vec<CardId> = p
                        .hand
                        .iter()
                        .copied()
                        .filter(|other| *other != card && !other.card().is_wound())
                        .collect();
                    (0..=cards.len().min(crate::PICKS))
                        .flat_map(|size| combinations(&cards, size))
                        .map(|discard| ArtifactChoices {
                            discard: crate::Picks::of(&discard),
                            ..ArtifactChoices::default()
                        })
                        .collect()
                }
                // Spec 055 (Livre de sagesse ; Compendium, *Book of Wisdom* S1) : une carte action de
                // la main à détruire, une carte de sa couleur dans l'offre (L4).
                Effect::TrainFromOffer(_) | Effect::SpellAndCrystalFromOffer => {
                    let offer = if option_effect == Effect::SpellAndCrystalFromOffer {
                        Offer::Spells
                    } else {
                        Offer::AdvancedActions
                    };
                    p.hand
                        .iter()
                        .filter(|&&other| other != card && is_action_card(other))
                        .flat_map(|&destroyed| {
                            destroyed
                                .card()
                                .definition()
                                .map(|definition| {
                                    offer_cards_of_color(state, offer, definition.color)
                                })
                                .unwrap_or_default()
                                .into_iter()
                                .map(move |gain| ArtifactChoices {
                                    discard: crate::Picks::of(&[destroyed]),
                                    gain: Some(gain),
                                    ..ArtifactChoices::default()
                                })
                        })
                        .collect()
                }
                // Spec 055 (Corne de la colère renforcée) : le bonus, de 0 à `extra`.
                Effect::HornOfWrath { extra } if strong => (0..=extra)
                    .map(|bonus| ArtifactChoices {
                        bonus,
                        ..ArtifactChoices::default()
                    })
                    .collect(),
                // Spec 055 (Bannière de courage renforcée) : une unité épuisée au moins.
                Effect::ReadyAllUnits if p.units.iter().all(|owned| owned.ready) => Vec::new(),
                _ => vec![ArtifactChoices::default()],
            };
            actions.extend(choices.into_iter().map(|choices| Action::PlayCard {
                card,
                play: play(choices),
            }));
        }
    }
    actions
}

/// Les suites de `size` éléments de `items`, répétitions permises, chacune dans l'ordre de `items`
/// (spec 055 : les couleurs des jetons de l'Amulette des ténèbres).
fn multisets<T: Copy>(items: &[T], size: usize) -> Vec<Vec<T>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    let mut result = Vec::new();
    for (index, &item) in items.iter().enumerate() {
        for mut rest in multisets(&items[index..], size - 1) {
            rest.insert(0, item);
            result.push(rest);
        }
    }
    result
}

/// Spec 055 : les bannières qui servent. Terreur (carte, *Lost Legion* ; Compendium, *Banner of
/// Fear* S3, S4) : en phase de parade, l'unité porteuse disponible et non blessée, contre chaque
/// ennemi qui attaque encore (pas un Invocateur, issue #8). Courage (Compendium, *Banner of
/// Courage* S1) : hors combat, la bannière face visible, l'unité porteuse épuisée.
pub(super) fn banner_use_actions(p: &PlayerState) -> Vec<Action> {
    let mut actions = Vec::new();
    for owned in &p.units {
        let Some(DeedCard::Artifact(banner)) = owned.banner.map(CardId::card) else {
            continue;
        };
        match (banner, moment(p)) {
            (Artifact::BannerOfFear, Moment::Fighting(combat))
                if combat.phase == crate::CombatPhase::Block
                    && owned.ready
                    && !owned.is_wounded() =>
            {
                actions.extend(
                    unit_enemy_targets(
                        combat,
                        &[ModifierKind::OnEnemy(EnemyModifierKind::DoesNotAttack)],
                    )
                    .into_iter()
                    .map(|enemy| Action::UseBanner {
                        unit: owned.card,
                        enemy: Some(enemy),
                    }),
                );
            }
            (Artifact::BannerOfCourage, Moment::Free) if !owned.banner_flipped && !owned.ready => {
                actions.push(Action::UseBanner {
                    unit: owned.card,
                    enemy: None,
                });
            }
            _ => {}
        }
    }
    actions
}
