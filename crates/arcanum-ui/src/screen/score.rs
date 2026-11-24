//! Fin de partie de « Première reconnaissance » (spec 063) : lignes du dernier tour et fenêtre de
//! fin avec le décompte final.

use arcanum_core::{
    Achievement, ConquestLine, ConquestScore, GameEndReason, LastTurn, PlayerView, ScenarioEnd,
};
use arcanum_i18n::Catalog;

use super::{Button, Command};

/// Fenêtre de fin : la raison, les sept lignes du décompte, le total, déjà traduits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreWindow {
    pub title: String,
    pub reason: String,
    /// « Mission accomplie ! » ou « Mission échouée… », en gras sous la raison (spec 064).
    pub outcome: Option<String>,
    /// Gloire, puis les six Hauts Faits dans l'ordre de la carte de score, même à 0 (Q3).
    pub lines: Vec<String>,
    pub total: String,
    pub no_titles: String,
    /// « Voir le plateau » : ferme la fenêtre.
    pub close: Button,
    /// « Nouvelle partie… » : ouvre la fenêtre de nouvelle partie (spec 065).
    pub new_game: Button,
}

/// Bandeau du tour de la révélation : « Cité révélée : après ce tour, encore un dernier tour. »
/// Après la conquête de la dernière cité (spec 064) : « Toutes les cités sont conquises… ».
pub(super) fn last_turn_next(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    let conquest = view
        .scenario
        .is_some_and(|scenario| scenario.end() == ScenarioEnd::AllCitiesConquered);
    (view.last_turn == LastTurn::Next && view.game_end.is_none()).then(|| {
        catalog.text(if conquest {
            "scenario.last_turn_next_conquest"
        } else {
            "scenario.last_turn_next"
        })
    })
}

/// « Cités conquises : 1/2 », pour un scénario de conquête (spec 064, C28).
pub(super) fn cities_progress(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.conquest.map(|progress| {
        catalog.format(
            "scenario.cities_progress",
            &[
                ("conquered", &progress.conquered.to_string()),
                ("total", &progress.cities.to_string()),
            ],
        )
    })
}

/// Les cinq lignes de conquête du décompte (spec 064), dans l'ordre de `ConquestLine::ALL`.
fn conquest_lines(conquest: &ConquestScore, catalog: &Catalog) -> Vec<String> {
    ConquestLine::ALL
        .into_iter()
        .map(|line| {
            let points = conquest.points(line).to_string();
            let (key, count) = match line {
                ConquestLine::Cities => ("score.cities", Some(("count", conquest.conquered))),
                ConquestLine::AllCities => ("score.all_cities", None),
                ConquestLine::UnplayedRounds => (
                    "score.unplayed_rounds",
                    Some(("rounds", conquest.unplayed_rounds)),
                ),
                ConquestLine::DummyDeck => {
                    ("score.dummy_deck", Some(("cards", conquest.dummy_deck)))
                }
                ConquestLine::RoundNotAnnounced => ("score.round_not_announced", None),
            };
            let count = count.map(|(name, value)| (name, value.to_string()));
            let mut params = vec![("points", points.as_str())];
            params.extend(count.as_ref().map(|(name, value)| (*name, value.as_str())));
            catalog.format(key, &params)
        })
        .collect()
}

/// Ligne « Dernier tour », sous la ligne de la manche, pendant le dernier tour.
pub(super) fn last_turn_line(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    (view.last_turn == LastTurn::Current && view.game_end.is_none())
        .then(|| catalog.text("scenario.last_turn"))
}

/// Message de fin du panneau latéral, d'après la raison de la fin de partie.
pub(super) fn end_message(reason: GameEndReason, catalog: &Catalog) -> String {
    match reason {
        GameEndReason::LastRound => catalog.text("game.last_round"),
        GameEndReason::CityRevealed | GameEndReason::AllCitiesConquered => {
            catalog.text("game.finished")
        }
    }
}

/// La fenêtre de fin, quand la partie est terminée et que le scénario a un décompte.
pub(super) fn score_window(view: &PlayerView, catalog: &Catalog) -> Option<ScoreWindow> {
    let score = view.final_score?;
    let reason = view.game_end?;
    let points = |achievement| score.points(achievement).to_string();
    let lines = std::iter::once(catalog.format("score.fame", &[("fame", &score.fame.to_string())]))
        .chain(Achievement::ALL.into_iter().map(|achievement| {
            let key = match achievement {
                Achievement::Knowledge => "score.knowledge",
                Achievement::Loot => "score.loot",
                Achievement::Leader => "score.leader",
                Achievement::Conqueror => "score.conqueror",
                Achievement::Adventurer => "score.adventurer",
                Achievement::Beating => "score.beating",
            };
            catalog.format(
                key,
                &[
                    ("points", &points(achievement)),
                    ("crystals", &score.crystals.to_string()),
                    ("wounds", &score.wounds.to_string()),
                    ("artifacts", &score.artifacts.to_string()),
                    ("shields", &score.shields.to_string()),
                ],
            )
        }))
        .chain(
            score
                .conquest
                .iter()
                .flat_map(|conquest| conquest_lines(conquest, catalog)),
        )
        .collect();
    Some(ScoreWindow {
        title: catalog.text("score.title"),
        reason: catalog.text(match reason {
            GameEndReason::CityRevealed => "score.reason.city_revealed",
            GameEndReason::LastRound => "score.reason.last_round",
            GameEndReason::AllCitiesConquered => "score.reason.all_cities_conquered",
        }),
        outcome: score.conquest.map(|conquest| {
            catalog.text(if conquest.all_conquered {
                "score.outcome.success"
            } else {
                "score.outcome.failure"
            })
        }),
        lines,
        total: catalog.format("score.total", &[("score", &score.total().to_string())]),
        // « Variante solo » ne convient pas à Conquête en solitaire, qui n'en est pas une.
        no_titles: catalog.text(if score.conquest.is_some() {
            "score.no_titles_solo"
        } else {
            "score.no_titles"
        }),
        close: Button {
            label: catalog.text("score.show_board"),
            command: Command::CloseScore,
            // Spec 071, C23 : Échap ferme la fenêtre de fin.
            shortcut: crate::shortcut_label(crate::Shortcut::Cancel, catalog),
        },
        new_game: Button {
            label: catalog.text("new_game.button"),
            command: Command::OpenNewGame,
            shortcut: None,
        },
    })
}

/// Bouton « Score final » du panneau, quand la fenêtre de fin est fermée.
pub(crate) fn reopen_score_button(catalog: &Catalog) -> Button {
    Button {
        label: catalog.text("score.reopen"),
        command: Command::ReopenScore,
        shortcut: None,
    }
}
