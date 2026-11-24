//! `apply` : combat (engagement, phases, attaques, parades, Blessures reçues, ennemis).

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn combat_event(state: &mut GameState, event: &Event) {
    match event {
        Event::AssaultStarted { player, site, from } => {
            let defended = state.has_enemy_token(*site);
            if let Some(p) = state.player_mut(*player) {
                // MKUE p. 6, 6.b et 6.e : l'action du tour commence, les points restants sont
                // perdus ; le défenseur est à combattre (spec 043).
                p.move_points = 0;
                p.influence_points = 0;
                p.movement_ended = true;
                p.moved_or_fought = true;
                p.assault = Some(crate::Assault {
                    site: *site,
                    from: *from,
                });
                if defended {
                    p.enemies_to_fight.push(*site);
                }
            }
        }
        Event::EnemiesProvoked { player, enemies } => {
            if let Some(p) = state.player_mut(*player) {
                // MKUE p. 6, 6.e : l'action commence, les points restants sont perdus.
                p.movement_ended = true;
                p.move_points = 0;
                p.influence_points = 0;
                p.enemies_to_fight.extend(enemies.iter().copied());
            }
        }
        Event::EnemyChallenged { player, enemy } => {
            if let Some(p) = state.player_mut(*player) {
                // MKUE p. 6, 6.d et 6.e : défier commence l'action, comme une provocation.
                p.movement_ended = true;
                p.move_points = 0;
                p.influence_points = 0;
                p.enemies_to_fight.push(*enemy);
                p.moved_or_fought = true;
            }
        }
        Event::CombatStarted { player, enemies } => {
            let state_city: std::collections::BTreeMap<Hex, crate::BasicColor> = state
                .cities
                .iter()
                .map(|(&hex, city)| (hex, city.color))
                .collect();
            if let Some(p) = state.player_mut(*player) {
                let mut combat = Combat::new(enemies);
                // Spec 043 : le défenseur de l'assaut est fortifié par son site.
                // Spec 057 : l'Attaque souterraine les ignore (`IgnoreSiteFortifications`).
                let ignored = p
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.kind == ModifierKind::IgnoreSiteFortifications);
                if let Some(assault) = p.assault {
                    // Spec 062 : les défenseurs d'une cité ont son bonus.
                    let city = state_city.get(&assault.site).copied();
                    for enemy in &mut combat.enemies {
                        enemy.site_fortified = enemy.hex == assault.site && !ignored;
                        if enemy.hex == assault.site {
                            enemy.city = city;
                        }
                    }
                }
                p.combat = Some(combat);
                // MKUE p. 11, 3.d : les points de soin sont perdus en entrant en combat.
                p.healing = 0;
                // Spec 042 : des dégâts une fois par combat.
                for owned in &mut p.units {
                    owned.damaged = false;
                }
                p.moved_or_fought = true;
            }
        }
        Event::CombatPhaseStarted { player, phase } => {
            if let Some(combat) = state.player_mut(*player).and_then(|p| p.combat.as_mut()) {
                combat.start_phase(*phase);
            }
        }
        Event::CombatEnded { player } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(combat) = p.combat.as_mut()
            {
                combat.ended = true;
                // Spec 053 : les modificateurs d'ennemis disparaissent avec le combat.
                for enemy in &mut combat.enemies {
                    enemy.modifiers.clear();
                }
                // ADR 0008 : la fin du combat retire les modificateurs du combat.
                p.modifiers
                    .retain(|modifier| modifier.duration != ModifierDuration::Combat);
            }
        }
        Event::AttackDeclared { player, targets } => {
            if let Some(combat) = state.player_mut(*player).and_then(|p| p.combat.as_mut()) {
                combat.declare_attack(targets);
            }
        }
        Event::EnemiesDefeated {
            player,
            enemies,
            fame,
        } => apply_defeat(state, *player, enemies, *fame),
        Event::AttackFailed { player } => {
            if let Some(combat) = state.player_mut(*player).and_then(|p| p.combat.as_mut()) {
                combat.fail_attack();
            }
        }
        Event::BlockDeclared { player, target } => {
            if let Some(combat) = state.player_mut(*player).and_then(|p| p.combat.as_mut()) {
                combat.declare_block(*target);
            }
        }
        Event::EnemyBlocked { player, enemy } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(combat) = p.combat.as_mut()
            {
                combat.block(*enemy);
                // Spec 054 : la parade réussie pose l'attaque différée du Bouclier ardent.
                let deferred: Vec<Modifier> = std::mem::take(&mut combat.on_block_success)
                    .into_iter()
                    .filter(|(kind, _)| matches!(kind, ModifierKind::DeferredAttack { .. }))
                    .map(|(kind, source)| Modifier {
                        kind,
                        source,
                        duration: ModifierDuration::Combat,
                    })
                    .collect();
                p.modifiers.extend(deferred);
            }
        }
        Event::BlockFailed { player } => {
            if let Some(combat) = state.player_mut(*player).and_then(|p| p.combat.as_mut()) {
                combat.on_block_success.clear();
                combat.fail_block();
            }
        }
        Event::WoundsTaken {
            player,
            enemy,
            wounds,
        } => {
            if let Some(p) = state.player_mut(*player) {
                p.wounds_received_hand += wounds.len() as u32;
            }
            apply_wounds(state, *player, *enemy, wounds)
        }
        // Spec 053 : la Blessure de Coup de sang, comptée dans les Blessures du combat.
        Event::WoundFromCard { player, wound, .. } => {
            if let Some(p) = state.player_mut(*player) {
                p.hand.push(*wound);
                p.wounds_received_hand += 1;
                p.wounds_given = p.wounds_given.saturating_add(1);
                if let Some(combat) = p.combat.as_mut().filter(|combat| !combat.ended) {
                    combat.wounds_to_hand += 1;
                }
            }
        }
        Event::PoisonWoundsDiscarded {
            player,
            enemy,
            wounds,
        } => {
            if let Some(p) = state.player_mut(*player) {
                p.wounds_received_discard += wounds.len() as u32;
            }
            apply_poison(state, *player, *enemy, wounds)
        }
        Event::HandParalyzed { player, cards } => apply_paralysis(state, *player, cards),
        Event::EnemySummoned {
            player,
            summoner,
            token,
        } => apply_summon(state, *player, *summoner, *token),
        Event::SummonedEnemyDiscarded { player, summoner } => {
            apply_dismiss(state, *player, *summoner)
        }
        Event::KnockedOut { player, cards } => apply_knock_out(state, *player, cards),
        // Spec 052 (Agilité, version Lost Legion) : 1 Mouvement pour Attaque 1 physique, 2 pour
        // Attaque à distance 1.
        Event::MoveSpentAsAttack { player, ranged } => {
            let (rate, kind) = if *ranged {
                (2, AttackKind::Ranged)
            } else {
                (1, AttackKind::Melee)
            };
            if let Some(p) = state.player_mut(*player)
                && let Some(left) = p.move_points.checked_sub(rate)
            {
                let effect = with_attack_changes(
                    &p.modifiers,
                    p.combat.as_ref(),
                    Effect::Attack {
                        amount: 1,
                        element: Element::Physical,
                        kind,
                    },
                );
                if let Some(combat) = p.combat.as_mut() {
                    p.move_points = left;
                    combat.add_converted(effect);
                }
            }
        }
        // Spec 052 (Diplomatie) : 1 Influence pour Parade 1 de l'élément.
        Event::InfluenceSpentAsBlock { player, element } => {
            if let Some(p) = state.player_mut(*player)
                && p.influence_points >= 1
                && let Some(combat) = p.combat.as_mut()
            {
                p.influence_points -= 1;
                combat.add_converted(Effect::Block {
                    amount: 1,
                    element: *element,
                });
            }
        }
        // Spec 058 : Botte secrète, jamais sous 1, pour le combat (spec 053, `ArmorReduction`).
        Event::ResistanceBroken { player, enemy, by } => {
            if let Some(fought) = state
                .player_mut(*player)
                .and_then(|p| p.combat.as_mut())
                .and_then(|combat| combat.enemies.iter_mut().find(|e| e.token == *enemy))
            {
                fought.modifiers.push(crate::EnemyModifier {
                    kind: EnemyModifierKind::ArmorReduction { by: *by, min: 1 },
                    source: ModifierSource::Skill(Skill::ResistanceBreak),
                });
            }
        }
        Event::EnemyDestroyed { player, enemy } => apply_destroyed(state, *player, *enemy),
        Event::DeferredAttackUsed {
            player,
            amount,
            element,
        } => {
            if let Some(p) = state.player_mut(*player) {
                let used = p.modifiers.iter().position(|modifier| {
                    modifier.kind
                        == ModifierKind::DeferredAttack {
                            amount: *amount,
                            element: *element,
                        }
                });
                if let Some(index) = used {
                    p.modifiers.remove(index);
                }
                if let Some(combat) = p.combat.as_mut() {
                    combat.add_converted(Effect::Attack {
                        amount: *amount,
                        element: *element,
                        kind: AttackKind::Melee,
                    });
                }
            }
        }
        _ => {}
    }
}
