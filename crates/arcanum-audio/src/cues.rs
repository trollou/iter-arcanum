//! Table des effets sonores (spec 081) : quel événement filtré donne quel effet.

use arcanum_core::{CardPlay, PlayerEvent, PlayerId, TimeOfDay};

use crate::{Cue, Outcome, SoundSituation};

/// L'effet d'un événement entendu par le joueur humain, s'il en a un. `situation` porte l'issue
/// de la partie pour le jingle de fin (C29).
pub(crate) fn cue_of(event: &PlayerEvent, situation: SoundSituation) -> Option<Cue> {
    use PlayerEvent as E;
    // Le joueur concerné ; `None` pour un événement de la table ou de toute la partie.
    let (player, cue) = match event {
        E::RoundStarted { time_of_day, .. } => (
            None,
            match time_of_day {
                TimeOfDay::Day => Cue::Rooster,
                TimeOfDay::Night => Cue::Owl,
            },
        ),
        // Le premier jour n'a pas de `RoundStarted` : son coq vient du début de la partie.
        E::GameStarted { .. } => (None, Cue::Rooster),
        E::HeroMoved { player, .. } => (Some(*player), Cue::Footstep),
        E::TileRevealed { .. } => (None, Cue::TileRevealed),
        E::CombatStarted { player, .. } => (Some(*player), Cue::CombatStart),
        E::EnemyBlocked { player, .. } => (Some(*player), Cue::Block),
        E::AttackFailed { player } | E::BlockFailed { player } => (Some(*player), Cue::Miss),
        E::WoundsTaken { player, .. }
        | E::WoundFromCard { player, .. }
        | E::UnitWounded { player, .. } => (Some(*player), Cue::Wound),
        E::EnemiesDefeated { player, .. } | E::EnemyDestroyed { player, .. } => {
            (Some(*player), Cue::EnemyDefeated)
        }
        E::CombatEnded { player } => (Some(*player), Cue::CombatEnd),
        // Un sort joué de côté n'est pas lancé.
        E::CardPlayed {
            player, card, play, ..
        } if card.card().is_spell()
            && !matches!(play, CardPlay::Sideways(_) | CardPlay::SidewaysSkill { .. }) =>
        {
            (Some(*player), Cue::Spell)
        }
        E::SiteConquered { player, .. } => (Some(*player), Cue::Conquest),
        E::CityConquered { leader, .. } => (Some(*leader), Cue::Conquest),
        E::LevelGained { player, .. } => (Some(*player), Cue::LevelUp),
        E::UnitRecruited { player, .. } => (Some(*player), Cue::Recruit),
        E::KnockedOut { player, .. } => (Some(*player), Cue::KnockedOut),
        E::LastTurnAnnounced { player } => (Some(*player), Cue::LastTurn),
        E::GameEnded { .. } => (
            None,
            match situation {
                SoundSituation::GameOver(_, Outcome::Victory) => Cue::Victory,
                _ => Cue::Defeat,
            },
        ),
        _ => return None,
    };
    player
        .is_none_or(|player| player == PlayerId::SOLO)
        .then_some(cue)
}
