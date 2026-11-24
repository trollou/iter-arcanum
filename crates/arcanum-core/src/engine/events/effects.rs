//! Événements pour `decide` : un effet résolu, de carte, d'unité ou de compétence.

use super::*;

/// Un cristal gagné, ou un jeton si l'inventaire en a déjà trois (MKUE p. 5, « Gain » 2).
pub(in crate::engine) fn gained_crystal(
    inventory: &mut Inventory,
    player: PlayerId,
    color: BasicColor,
) -> Event {
    if inventory.is_full(color) {
        Event::ManaTokenGained {
            player,
            color: ManaColor::Basic(color),
        }
    } else {
        inventory.add(color);
        Event::CrystalGained { player, color }
    }
}

/// Ce que fait un effet joué, carte ou unité (specs 025, 034, 040, 042) : le mana qu'il fait
/// gagner, les cartes qu'il fait piocher, la réputation.
pub(in crate::engine) fn effect_events(
    events: &mut Vec<Event>,
    inventory: &mut Inventory,
    p: &PlayerState,
    player: PlayerId,
    effect: Effect,
) {
    // Spec 025 : le mana gagné. Un cristal de trop devient un jeton (MKUE p. 5, « Gain »
    // 2), compté après le paiement, qui a pu libérer une place (L1).
    match effect {
        Effect::GainToken(color) => events.push(Event::ManaTokenGained {
            player,
            color: ManaColor::Basic(color),
        }),
        Effect::GainCrystal(color) | Effect::PayForCrystal(color) => {
            events.push(if inventory.is_full(color) {
                Event::ManaTokenGained {
                    player,
                    color: ManaColor::Basic(color),
                }
            } else {
                inventory.add(color);
                Event::CrystalGained { player, color }
            });
        }
        // Spec 042 : jeton et cristal (Mages de feu et de glace), deux jetons (Mages d'Altem).
        Effect::TokenAndCrystal(color) => {
            events.push(Event::ManaTokenGained {
                player,
                color: ManaColor::Basic(color),
            });
            events.push(if inventory.is_full(color) {
                Event::ManaTokenGained {
                    player,
                    color: ManaColor::Basic(color),
                }
            } else {
                inventory.add(color);
                Event::CrystalGained { player, color }
            });
        }
        Effect::GainTokens(first, second) => {
            for color in [first, second] {
                events.push(Event::ManaTokenGained { player, color });
            }
        }
        // Spec 056 : Rituel de sang, le cristal puis le jeton ; Maîtrise du cristal.
        Effect::WoundCrystalAndToken { crystal, token } => {
            events.push(gained_crystal(inventory, player, crystal));
            events.push(Event::ManaTokenGained {
                player,
                color: token,
            });
        }
        Effect::GainOwnedCrystal(color) => events.push(gained_crystal(inventory, player, color)),
        // Le dessus du deck, ce qu'il y en a ; jamais la défausse (spec 016).
        Effect::DrawCards(count) => events.extend(
            p.deck
                .cards()
                .iter()
                .take(usize::from(count))
                .map(|&card| Event::CardDrawn { player, card }),
        ),
        // Les dés de plus s'ajoutent avec `CardPlayed` ; la carte de Concentration suit.
        // Spec 040 : la réputation change aussitôt (Q3) ; Nobles manières seulement pendant une
        // interaction (FAQ p1 ; Compendium, *Noble Manners* S2).
        Effect::InfluenceReputation { reputation, .. } => events.push(Event::ReputationChanged {
            player,
            change: reputation,
        }),
        Effect::InfluenceInInteraction {
            fame, reputation, ..
        } => {
            if p.interaction.is_some() {
                if fame > 0 {
                    events.push(Event::FameGained {
                        player,
                        fame: u32::from(fame),
                    });
                }
                if reputation != 0 {
                    events.push(Event::ReputationChanged {
                        player,
                        change: reputation,
                    });
                }
            }
        }
        // Le soin alimente la réserve avec l'effet joué (spec 042, Q1).
        Effect::Heal(_)
        | Effect::Move(_)
        | Effect::Influence(_)
        | Effect::Attack { .. }
        | Effect::Block { .. }
        | Effect::IceBlockPerAbility(_)
        | Effect::ExtraDie
        | Effect::ExtraDieBlackAnyColor
        | Effect::Boost(_)
        | Effect::SetDice { .. }
        | Effect::SwiftBlock(_)
        | Effect::ReadyUnit(_)
        | Effect::Nothing
        | Effect::AttackWithWound { .. }
        | Effect::MoveAndHeal { .. }
        | Effect::HealAndReady { .. }
        | Effect::InfluencePerWound { .. } => {}
        // Spec 056 : écrit avec chaque carte ; Décomposition dans `played_events`, la carte
        // gagnée dans `play_card_events`, les jetons de Rituel de sang renforcé par le choix.
        Effect::WoundAndTokens(_)
        | Effect::CrystalsFromDestroyed(_)
        | Effect::CrystalsOtherColors
        | Effect::CrystalFromDie
        | Effect::RerollSource
        | Effect::Repeat(_)
        | Effect::GainSpell(_)
        | Effect::TrainFromOffer(_) => {}
        // Spec 054 : écrit dans `play_card_events` et `apply`.
        Effect::AttackPerEnemy { .. }
        | Effect::BlockPerEnemy { .. }
        | Effect::DestroyEnemy
        | Effect::CastFromOffer => {}
        // Spec 057.
        Effect::WoundsPerCrystal(_)
        | Effect::OthersOnly
        | Effect::HealByTerrain { .. }
        | Effect::HealAndReadyLevels { .. }
        | Effect::ReadyAnyUnit { .. }
        | Effect::UndergroundMove { .. }
        | Effect::FlyMove { .. }
        | Effect::DoesNotAttackPerMove
        | Effect::DiscardToDeck { .. }
        | Effect::ClaimDie { .. } => {}
        // Spec 055 : la gloire de la Bourse sans fond, aussitôt (Compendium, *Endless Bag of Gold*
        // Q1, Q2).
        Effect::InfluenceAndFame { fame, .. } => events.push(Event::FameGained {
            player,
            fame: u32::from(fame),
        }),
        Effect::BannerAssigned => {}
        Effect::AmuletOfSun { .. } | Effect::AmuletOfDarkness { .. } => {}
        Effect::CancelAttacks(_) | Effect::SwordDiscard => {}
        Effect::HornOfWrath { .. } | Effect::GemPouchDice | Effect::TokensEachColorAndTime => {}
        Effect::SpellAndCrystalFromOffer => {}
        // Spec 055 : Bannière de courage renforcée, blessées ou non.
        Effect::ReadyAllUnits => {
            events.extend(p.units.iter().filter(|owned| !owned.ready).map(|owned| {
                Event::UnitReadied {
                    player,
                    unit: owned.card,
                }
            }))
        }
        // Spec 051 : le cristal (un jeton si l'inventaire est plein, spec 025), puis le jeton.
        Effect::CrystalAndToken { crystal, token } => {
            events.push(gained_crystal(inventory, player, crystal));
            events.push(Event::ManaTokenGained {
                player,
                color: token,
            });
        }
        // Spec 051 : Motivation, le dessus du deck sans remélanger, et toujours le jeton
        // (Compendium, *Motivation* S1, S3).
        Effect::DrawAndToken { cards, token } => {
            events.extend(
                p.deck
                    .cards()
                    .iter()
                    .take(usize::from(cards))
                    .map(|&card| Event::CardDrawn { player, card }),
            );
            events.push(Event::ManaTokenGained {
                player,
                color: ManaColor::Basic(token),
            });
        }
        // Spec 051 : l'unité soignée est écrite par `UseSkill`.
        Effect::HealUnit => {}
        // Spec 058 : arrivent résolues ; l'Influence d'interaction, par `apply`.
        Effect::CrystalColorInfluence
        | Effect::MovePerUnit { .. }
        | Effect::InteractionInfluence(_) => {}
        // Spec 058 : selon la cible de `UseSkill` (`skill_target_events`).
        Effect::Flight { .. }
        | Effect::ResistanceBreak
        | Effect::SourceOpening
        | Effect::CalmWeather
        | Effect::ManaOverload => {}
        // Spec 058 : Rituel de guérison, dans la pile des Blessures (Compendium, *Ritual of Pain*
        // S1).
        Effect::ThrowWounds(count) => events.extend((0..count).map(|_| Event::WoundThrownAway {
            player,
            from: WoundPile::Hand,
        })),
        // Spec 058 : Même pas mal, une Blessure dans la défausse, puis le dessus du deck, sans
        // remélanger (spec 034).
        Effect::DiscardWoundToDraw => {
            if let Some(&card) = p.hand.iter().find(|card| card.card().is_wound()) {
                events.push(Event::CardDiscarded { player, card });
                events.extend(
                    p.deck
                        .cards()
                        .first()
                        .map(|&card| Event::CardDrawn { player, card }),
                );
            }
        }
        // Spec 055 : jeton, cristal (un jeton si l'inventaire est plein) et Gloire +1.
        Effect::TokenCrystalAndFame(color) => {
            events.push(Event::ManaTokenGained {
                player,
                color: ManaColor::Basic(color),
            });
            events.push(gained_crystal(inventory, player, color));
            events.push(Event::FameGained { player, fame: 1 });
        }
    }
}
