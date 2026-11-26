//! Boutons Revenir en arrière, Sauvegarder, Charger… et fenêtre de chargement (spec 024) :
//! calculés sans egui, déjà traduits.

use arcanum_core::UndoStatus;
use arcanum_i18n::Catalog;
use arcanum_persistance::{LoadError, SaveEntry, SaveSummary};

use crate::saves::display_date;
use crate::{Button, Command, Shortcut, shortcut_label};

/// Bouton qui peut être grisé, avec son info-bulle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlButton {
    pub label: String,
    pub hint: String,
    pub enabled: bool,
    pub command: Command,
    /// « Ctrl+Z » : sa touche, écrite en face du libellé dans le Menu (spec 074).
    pub shortcut: Option<String>,
}

/// Les commandes de la partie reprises par le Menu (spec 074), et le message du dernier essai.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controls {
    pub undo: ControlButton,
    pub save: ControlButton,
    pub load: ControlButton,
    /// « Nouvelle partie… » (spec 065), grisé comme les deux autres.
    pub new_game: ControlButton,
    pub notice: Option<String>,
}

/// Ce qu'on dit au joueur après une sauvegarde ou un chargement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    Saved(String),
    SaveFailed,
    AutosaveFailed,
    LoadFailed,
    /// La partie abandonnée pour une nouvelle a été sauvegardée sous ce nom (spec 065).
    PreviousSaved(String),
}

/// `in_mode` : un mode de choix ou la fenêtre de chargement est ouvert (spec 024, L7).
pub fn controls_screen(
    undo: UndoStatus,
    in_mode: bool,
    notice: Option<&Notice>,
    catalog: &Catalog,
) -> Controls {
    let undo_hint = match undo {
        UndoStatus::Possible => "undo.hint",
        UndoStatus::NothingSinceTurnStart => "undo.blocked.turn_start",
        UndoStatus::InformationRevealed => "undo.blocked.revealed",
    };
    // Spec 071, C18 : la touche s'affiche sous la description, même grisé.
    let button = |key: &str, hint: String, enabled: bool, command, shortcut| ControlButton {
        label: catalog.text(&format!("{key}.button")),
        hint,
        enabled,
        command,
        shortcut: shortcut_label(shortcut, catalog),
    };
    Controls {
        undo: button(
            "undo",
            catalog.text(undo_hint),
            !in_mode && undo == UndoStatus::Possible,
            Command::Undo,
            Shortcut::Undo,
        ),
        save: button(
            "save",
            catalog.text("save.hint"),
            !in_mode,
            Command::Save,
            Shortcut::Save,
        ),
        load: button(
            "load",
            catalog.text("load.hint"),
            !in_mode,
            Command::OpenLoad,
            Shortcut::OpenLoad,
        ),
        new_game: button(
            "new_game",
            catalog.text("new_game.hint"),
            !in_mode,
            Command::OpenNewGame,
            Shortcut::NewGame,
        ),
        notice: notice.map(|notice| match notice {
            Notice::Saved(name) => catalog.format("save.done", &[("name", name)]),
            Notice::SaveFailed => catalog.text("save.failed"),
            Notice::AutosaveFailed => catalog.text("save.auto_failed"),
            Notice::LoadFailed => catalog.text("load.failed"),
            Notice::PreviousSaved(name) => {
                catalog.format("new_game.previous_saved", &[("name", name)])
            }
        }),
    }
}

/// Fenêtre de chargement : une ligne par sauvegarde, dans l'ordre de `arcanum_persistance::list`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadWindow {
    pub title: String,
    /// Présent quand il n'y a aucune sauvegarde.
    pub empty: Option<String>,
    pub lines: Vec<LoadLine>,
    pub close: Button,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadLine {
    pub text: String,
    /// Absent pour un fichier illisible, grisé avec son erreur.
    pub load: Option<Button>,
}

pub fn load_window_screen(entries: &[SaveEntry], catalog: &Catalog) -> LoadWindow {
    let lines = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| match &entry.summary {
            Ok(summary) => {
                let date = if entry.is_auto() {
                    catalog.text("load.auto")
                } else {
                    display_date(&summary.saved_at)
                };
                LoadLine {
                    text: catalog.format(
                        "load.entry",
                        &[
                            ("date", &date),
                            ("scenario", &catalog.text(summary.scenario.label_key())),
                            ("hero", &catalog.text(summary.hero.label_key())),
                            ("line", &turn_line(summary, catalog)),
                        ],
                    ),
                    load: Some(Button {
                        label: catalog.text("load.load"),
                        command: Command::Load(index),
                        shortcut: None,
                    }),
                }
            }
            Err(error) => {
                let name = entry
                    .path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let reason = catalog.text(match error {
                    LoadError::Io(_) => "load.error.unreadable",
                    // Squelette (tests rouges) : raisons de la spec #52 pas encore distinguées.
                    LoadError::NewerVersion { .. }
                    | LoadError::TooOld { .. }
                    | LoadError::NotMigratable { .. }
                    | LoadError::Invalid(_)
                    | LoadError::Resume(_) => "load.error.invalid",
                });
                LoadLine {
                    text: catalog
                        .format("load.unreadable", &[("name", &name), ("reason", &reason)]),
                    load: None,
                }
            }
        })
        .collect::<Vec<_>>();
    LoadWindow {
        title: catalog.text("load.title"),
        empty: lines.is_empty().then(|| catalog.text("load.empty")),
        lines,
        close: Button {
            label: catalog.text("load.close"),
            command: Command::CloseLoad,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    }
}

/// « Manche 1/4 · Jour · Tour 1 » d'une sauvegarde.
fn turn_line(summary: &SaveSummary, catalog: &Catalog) -> String {
    let time = match summary.time_of_day {
        arcanum_core::TimeOfDay::Day => catalog.text("time_of_day.day"),
        arcanum_core::TimeOfDay::Night => catalog.text("time_of_day.night"),
    };
    catalog.format(
        "turn.line",
        &[
            ("round", &summary.round.to_string()),
            ("limit", &summary.round_limit.to_string()),
            ("time_of_day", &time),
            ("number", &summary.turn.to_string()),
        ],
    )
}

/// Résumé d'une sauvegarde sans sa date (spec 065, « Continuer ») : les champs de `load.entry`
/// qui suivent la date, dans le même ordre.
pub(crate) fn summary_without_date(summary: &SaveSummary, catalog: &Catalog) -> String {
    let entry = catalog.format(
        "load.entry",
        &[
            ("date", ""),
            ("scenario", &catalog.text(summary.scenario.label_key())),
            ("hero", &catalog.text(summary.hero.label_key())),
            ("line", &turn_line(summary, catalog)),
        ],
    );
    entry.trim_start_matches([' ', '·']).to_string()
}
