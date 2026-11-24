//! Situation sonore (spec 081) : ce que montre l'écran, pour choisir musique et ambiance. Lue dans
//! la vue et dans l'écran, jamais dans les événements : elle se rattrape d'elle-même après un
//! retour en arrière, un chargement ou une nouvelle partie.

use arcanum_audio::{Outcome, SoundSituation};
use arcanum_core::{GameEndReason, PlayerView, ScenarioId};

/// La situation d'une partie vue par le joueur ; `title` : écran de lancement ou de nouvelle
/// partie par-dessus.
pub fn situation_of(view: Option<&PlayerView>, title: bool) -> SoundSituation {
    let Some(view) = view.filter(|_| !title) else {
        return SoundSituation::Title;
    };
    let time = view.time_of_day;
    if let Some(reason) = view.game_end {
        let scenario = view.scenario.as_ref().map(|scenario| scenario.id());
        SoundSituation::GameOver(time, game_outcome(scenario, reason))
    // Un combat fini reste dans la vue jusqu'à la fin du tour (spec 031), l'écran est déjà
    // revenu au plateau (issue #18).
    } else if view.combat.as_ref().is_some_and(|combat| !combat.ended) {
        SoundSituation::Combat(time)
    } else {
        SoundSituation::Exploration(time)
    }
}

/// Victoire ou défaite d'une partie finie, selon son scénario (table de la spec, C28).
pub fn game_outcome(scenario: Option<ScenarioId>, reason: GameEndReason) -> Outcome {
    match (scenario, reason) {
        // Q11 : le bac à sable n'a pas d'objectif, rien à perdre.
        (Some(ScenarioId::Sandbox) | None, _) => Outcome::Victory,
        // Mission accomplie (MKUE p. 19, *Scoring*).
        (Some(ScenarioId::SoloConquest), GameEndReason::AllCitiesConquered) => Outcome::Victory,
        // La cité trouvée est la mission (MKUE p. 18, *your task is to locate its Capital*).
        (Some(ScenarioId::FirstReconnaissance), GameEndReason::CityRevealed) => Outcome::Victory,
        _ => Outcome::Defeat,
    }
}
