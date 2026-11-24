//! Événements sur le combat : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        // Spec 033 : la créature invoquée est révélée.
        Event::EnemySummoned { .. } => true,
        // Spec 030 : rien de caché, aucun hasard ; on reprend le pas qui a provoqué.
        Event::EnemiesProvoked { .. }
        // Spec 043 : public, aucun hasard.
        | Event::AssaultStarted { .. }
        // Spec 031, Q9 : tout le combat se reprend.
        | Event::EnemyChallenged { .. }
        | Event::CombatStarted { .. }
        | Event::CombatPhaseStarted { .. }
        | Event::CombatEnded { .. }
        | Event::AttackDeclared { .. }
        | Event::EnemiesDefeated { .. }
        | Event::AttackFailed { .. }
        | Event::BlockDeclared { .. }
        | Event::EnemyBlocked { .. }
        | Event::BlockFailed { .. }
        // La pile de blessures est face visible : rien n'est tiré (spec 031, Q9).
        | Event::WoundsTaken { .. }
        | Event::WoundFromCard { .. }
        // Spec 033 : rien de caché, aucun hasard.
        | Event::PoisonWoundsDiscarded { .. }
        | Event::HandParalyzed { .. }
        | Event::SummonedEnemyDiscarded { .. }
        // Spec 034 : rien de caché, aucun hasard.
        | Event::KnockedOut { .. }
        // Spec 058 : publics, aucun hasard.
        | Event::ResistanceBroken { .. }
        // Spec 052 : publics, aucun hasard (la fin du tour est déjà un point de non-retour).
        | Event::MoveSpentAsAttack { .. }
        | Event::InfluenceSpentAsBlock { .. }
        // Spec 054 : publics, aucun hasard.
        | Event::EnemyDestroyed { .. }
        | Event::DeferredAttackUsed { .. } => false,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème combat"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, _viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::AssaultStarted { player, site, from } => PlayerEvent::AssaultStarted {
            player: *player,
            site: *site,
            from: *from,
        },
        Event::EnemiesProvoked { player, enemies } => PlayerEvent::EnemiesProvoked {
            player: *player,
            enemies: enemies.clone(),
        },
        Event::EnemyChallenged { player, enemy } => PlayerEvent::EnemyChallenged {
            player: *player,
            enemy: *enemy,
        },
        Event::CombatStarted { player, enemies } => PlayerEvent::CombatStarted {
            player: *player,
            enemies: enemies.clone(),
        },
        Event::CombatPhaseStarted { player, phase } => PlayerEvent::CombatPhaseStarted {
            player: *player,
            phase: *phase,
        },
        Event::CombatEnded { player } => PlayerEvent::CombatEnded { player: *player },
        Event::AttackDeclared { player, targets } => PlayerEvent::AttackDeclared {
            player: *player,
            targets: targets.clone(),
        },
        Event::EnemiesDefeated {
            player,
            enemies,
            fame,
        } => PlayerEvent::EnemiesDefeated {
            player: *player,
            enemies: enemies.clone(),
            fame: *fame,
        },
        Event::AttackFailed { player } => PlayerEvent::AttackFailed { player: *player },
        Event::BlockDeclared { player, target } => PlayerEvent::BlockDeclared {
            player: *player,
            target: *target,
        },
        Event::EnemyBlocked { player, enemy } => PlayerEvent::EnemyBlocked {
            player: *player,
            enemy: *enemy,
        },
        Event::BlockFailed { player } => PlayerEvent::BlockFailed { player: *player },
        Event::WoundsTaken {
            player,
            enemy,
            wounds,
        } => PlayerEvent::WoundsTaken {
            player: *player,
            enemy: *enemy,
            wounds: wounds.clone(),
        },
        Event::WoundFromCard {
            player,
            card,
            wound,
        } => PlayerEvent::WoundFromCard {
            player: *player,
            card: *card,
            wound: *wound,
        },
        Event::PoisonWoundsDiscarded {
            player,
            enemy,
            wounds,
        } => PlayerEvent::PoisonWoundsDiscarded {
            player: *player,
            enemy: *enemy,
            wounds: wounds.clone(),
        },
        // La défausse est face visible (MKUE p. 3, *Player Area*) : les cartes se voient.
        Event::HandParalyzed { player, cards } => PlayerEvent::HandParalyzed {
            player: *player,
            cards: cards.clone(),
        },
        Event::EnemySummoned {
            player,
            summoner,
            token,
        } => PlayerEvent::EnemySummoned {
            player: *player,
            summoner: *summoner,
            token: *token,
        },
        Event::SummonedEnemyDiscarded { player, summoner } => PlayerEvent::SummonedEnemyDiscarded {
            player: *player,
            summoner: *summoner,
        },
        // Spec 034 : les cartes défaussées sont publiques.
        Event::KnockedOut { player, cards } => PlayerEvent::KnockedOut {
            player: *player,
            cards: cards.clone(),
        },
        Event::ResistanceBroken { player, enemy, by } => PlayerEvent::ResistanceBroken {
            player: *player,
            enemy: *enemy,
            by: *by,
        },
        Event::EnemyDestroyed { player, enemy } => PlayerEvent::EnemyDestroyed {
            player: *player,
            enemy: *enemy,
        },
        Event::DeferredAttackUsed {
            player,
            amount,
            element,
        } => PlayerEvent::DeferredAttackUsed {
            player: *player,
            amount: *amount,
            element: *element,
        },
        // Spec 052 : publics.
        Event::MoveSpentAsAttack { player, ranged } => PlayerEvent::MoveSpentAsAttack {
            player: *player,
            ranged: *ranged,
        },
        Event::InfluenceSpentAsBlock { player, element } => PlayerEvent::InfluenceSpentAsBlock {
            player: *player,
            element: *element,
        },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème combat"),
    }
}
