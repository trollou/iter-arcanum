//! Décision des actions de combat : défi, phases, attaques, parades, dégâts, conversions.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn combat_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    match action {
        Action::Challenge { enemy } => Ok(vec![Event::EnemyChallenged {
            player,
            enemy: *enemy,
        }]),
        Action::StartCombat => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            // MKUE p. 8, 3 : les ennemis errants sont déjà face visible, rien n'est tiré.
            let enemies: Vec<(Hex, EnemyToken)> = p
                .enemies_to_fight
                .iter()
                .flat_map(|&hex| {
                    state
                        .enemy_tokens_at(hex)
                        .iter()
                        .map(move |&token| (hex, token))
                })
                .collect();
            // Spec 043 : un défenseur encore face cachée est retourné à l'engagement, après les
            // défis (Compendium, *Combat* Q1.a et S14).
            let mut events: Vec<Event> = enemies
                .iter()
                .filter(|(hex, _)| state.is_face_down(*hex))
                .map(|&(hex, token)| Event::EnemyTokenRevealed { hex, token })
                .collect();
            events.push(Event::CombatStarted { player, enemies });
            Ok(events)
        }
        Action::NextCombatPhase => next_phase_events(state, player, rngs),
        Action::EndCombat => decide_end_combat(state, player, rngs),
        Action::DeclareAttack { targets } => Ok(vec![Event::AttackDeclared {
            player,
            targets: targets.clone(),
        }]),
        Action::ResolveAttack => resolve_attack(state, player),
        Action::DeclareBlock { target } => Ok(vec![Event::BlockDeclared {
            player,
            target: *target,
        }]),
        Action::ResolveBlock => resolve_block(state, player),
        Action::AssignDamage { enemy, to } => assign_damage_events(state, player, *enemy, *to),
        Action::SpendMoveAsAttack => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            let combat = p.combat.as_ref().ok_or(ActionError::NotLegal)?;
            let (kind, _) = move_conversion(p, combat).ok_or(ActionError::NotLegal)?;
            Ok(vec![Event::MoveSpentAsAttack {
                player,
                ranged: kind != AttackKind::Melee,
            }])
        }
        Action::SpendInfluenceAsBlock => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            let combat = p.combat.as_ref().ok_or(ActionError::NotLegal)?;
            let element = influence_conversion(p, combat).ok_or(ActionError::NotLegal)?;
            Ok(vec![Event::InfluenceSpentAsBlock { player, element }])
        }
        // Spec 054 : l'attaque différée du Bouclier ardent, sans mana.
        Action::UseDeferredAttack => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            let (amount, element) = deferred_attack(p).ok_or(ActionError::NotLegal)?;
            Ok(vec![Event::DeferredAttackUsed {
                player,
                amount,
                element,
            }])
        }
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}

/// `Action::EndCombat` : la fin du combat et son issue.
fn decide_end_combat(
    state: &GameState,
    player: PlayerId,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let mut events = vec![Event::CombatEnded { player }];
    // Spec 041 : la fin d'un incendie, victoire ou non (carte du monastère ; FAQ p3).
    if let Some(hex) = state.burning(player) {
        let defeated = state.combat(player).is_some_and(|combat| {
            combat
                .enemies
                .iter()
                .any(|enemy| enemy.hex == hex && enemy.defeated)
        });
        if defeated {
            events.push(Event::MonasteryBurned { player, hex });
        } else {
            events.extend(
                state
                    .enemy_tokens_at(hex)
                    .iter()
                    .map(|&token| Event::EnemyTokenDiscarded { hex, token }),
            );
        }
    }
    // Spec 062 : la fin de l'assaut d'une cité.
    if let Some(assault) = state.assault(player)
        && let Some(city) = state.city(assault.site)
    {
        if !city.is_conquered() {
            events.extend(city_assault_outcome(state, player, assault, &city.shields));
            // Spec 064 : la dernière cité conquise annonce le dernier tour.
            events.extend(crate::game_end::conquest_last_turn(state, player, &events));
        }
    } else if let Some(assault) = state.assault(player)
        && !state.shields.contains_key(&assault.site)
    {
        // Spec 043 : la fin d'un assaut, conquête ou retraite (MKUE p. 10, 2.c et 3.d).
        let defeated = state.combat(player).is_some_and(|combat| {
            combat
                .enemies
                .iter()
                .any(|enemy| enemy.hex == assault.site && enemy.defeated)
        });
        if defeated {
            events.push(Event::SiteConquered {
                player,
                hex: assault.site,
            });
        } else if state.has_enemy_token(assault.site) {
            events.push(Event::HeroWithdrew {
                player,
                from: assault.site,
                to: assault.from,
            });
        }
    }
    if let Some(hex) = state.adventure(player) {
        events.extend(adventure_outcome_events(state, player, hex, rngs));
    }
    Ok(events)
}

/// La fin de l'assaut d'une cité (spec 062) : un Bouclier par défenseur vaincu, dans l'ordre où
/// ils l'ont été (MKUE p. 10, 1.a), puis la conquête s'il n'en reste aucun (2.c, *City Assaults*
/// 2), sinon la retraite (3.d). `shields` : les Boucliers déjà sur la cité.
fn city_assault_outcome(
    state: &GameState,
    player: PlayerId,
    assault: crate::combat::Assault,
    shields: &[PlayerId],
) -> Vec<Event> {
    let site = assault.site;
    let Some(combat) = state.combat(player) else {
        return Vec::new();
    };
    let defeated: Vec<EnemyToken> = combat
        .defeat_order
        .iter()
        .copied()
        .filter(|&token| combat.enemy(token).is_some_and(|enemy| enemy.hex == site))
        .collect();
    let mut events: Vec<Event> = defeated
        .iter()
        .map(|&enemy| Event::CityShieldPlaced {
            player,
            hex: site,
            enemy,
        })
        .collect();
    if state.has_enemy_token(site) {
        events.push(Event::HeroWithdrew {
            player,
            from: site,
            to: assault.from,
        });
    } else {
        let mut all = shields.to_vec();
        all.extend(defeated.iter().map(|_| player));
        events.push(Event::CityConquered {
            hex: site,
            leader: crate::CityState::leader_of(&all).unwrap_or(player),
        });
    }
    events
}
