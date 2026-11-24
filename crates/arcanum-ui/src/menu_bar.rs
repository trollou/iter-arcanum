//! Barre de menu (spec 074) : le Menu des commandes de la partie, puis le scénario, la manche,
//! les pioches de tuiles et le message de la dernière sauvegarde. Calculée sans egui.

use arcanum_core::{PlayerView, TimeOfDay};
use arcanum_i18n::Catalog;

use crate::{Clock, Command, ControlButton, Controls, Icon, IconValue, Shortcut, shortcut_label};

/// Ce que la session sait en plus de la partie : fenêtre de nouvelle partie ouverte (L1) et
/// plein écran (C11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuState {
    pub new_game_open: bool,
    pub fullscreen: bool,
}

/// Une ligne du Menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuItem {
    Entry(MenuEntry),
    Separator,
}

/// Un élément du Menu, sa touche écrite en face du libellé.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuEntry {
    pub label: String,
    /// Sa description seule, en infobulle (raison du grisé comprise).
    pub hint: Option<String>,
    /// « Ctrl+Z », « Début », « F11 ».
    pub shortcut: Option<String>,
    pub enabled: bool,
    /// Plein écran : coché ou non ; `None` pour un élément sans case.
    pub checked: Option<bool>,
    pub command: Command,
}

/// La barre, de gauche à droite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuBar {
    /// « Menu ».
    pub title: String,
    pub items: Vec<MenuItem>,
    /// Le nom du scénario seul (C4).
    pub scenario: Option<String>,
    /// Soleil ou lune (C5).
    pub clock: Clock,
    /// « Manche 1/4 (tour 3) ».
    pub round: String,
    /// La pioche de campagne puis la pioche royaume, ou leurs tuiles écartées (C6, C7).
    pub tile_decks: Vec<IconValue>,
    /// Le message de la dernière sauvegarde ou du dernier chargement (C9).
    pub notice: Option<String>,
}

pub fn menu_bar(
    view: &PlayerView,
    controls: &Controls,
    state: MenuState,
    catalog: &Catalog,
) -> MenuBar {
    // Les commandes de la partie, telles que les boutons de `Controls` (C2) ; grisées derrière la
    // fenêtre de nouvelle partie (L1).
    let game_entry = |button: &ControlButton| {
        MenuItem::Entry(MenuEntry {
            label: button.label.clone(),
            hint: Some(button.hint.clone()),
            shortcut: button.shortcut.clone(),
            enabled: button.enabled && !state.new_game_open,
            checked: None,
            command: button.command,
        })
    };
    // Recentrer, Plein écran, Préférences et Quitter : toujours actifs, sans description (C3).
    let window_entry = |key: &str, shortcut: Option<String>, checked, command| {
        MenuItem::Entry(MenuEntry {
            label: catalog.text(key),
            hint: None,
            shortcut,
            enabled: true,
            checked,
            command,
        })
    };
    let items = vec![
        game_entry(&controls.undo),
        MenuItem::Separator,
        game_entry(&controls.save),
        game_entry(&controls.load),
        game_entry(&controls.new_game),
        MenuItem::Separator,
        // La touche Début est lue par la caméra, hors de `KEY_BINDINGS`.
        window_entry(
            "board.recenter",
            Some(catalog.text("shortcut.key.home")),
            None,
            Command::Recenter,
        ),
        window_entry(
            "menu.fullscreen",
            shortcut_label(Shortcut::Fullscreen, catalog),
            Some(state.fullscreen),
            Command::ToggleFullscreen,
        ),
        MenuItem::Separator,
        // Spec 080 : toujours actif, comme Plein écran (C14).
        window_entry(
            "preferences.button",
            shortcut_label(Shortcut::Preferences, catalog),
            None,
            Command::OpenPreferences,
        ),
        MenuItem::Separator,
        window_entry(
            "menu.quit",
            shortcut_label(Shortcut::Quit, catalog),
            None,
            Command::Quit,
        ),
    ];
    // Spec 023 : sans scénario (partie pas commencée), la manche en cours tient lieu de limite.
    let round_limit = view
        .scenario
        .map_or(view.round, |scenario| u32::from(scenario.rounds()));
    MenuBar {
        title: catalog.text("menu.title"),
        items,
        scenario: view
            .scenario
            .map(|scenario| catalog.text(scenario.id().label_key())),
        clock: Clock {
            time_of_day: view.time_of_day,
            tooltip: catalog.text(match view.time_of_day {
                TimeOfDay::Day => "time_of_day.icon.day",
                TimeOfDay::Night => "time_of_day.icon.night",
            }),
        },
        round: {
            let round = catalog.format(
                "menu.round",
                &[
                    ("round", &view.round.to_string()),
                    ("limit", &round_limit.to_string()),
                    // Spec 060 : pendant le choix de la tactique, le tour 1 de la manche qui
                    // commence, pas le dernier de la manche finie.
                    (
                        "number",
                        &if view.tactic_choice.is_empty() {
                            view.turn
                        } else {
                            1
                        }
                        .to_string(),
                    ),
                ],
            );
            // Spec 057 : le tour supplémentaire de Distorsion temporelle.
            if view.extra_turn {
                format!("{round} · {}", catalog.text("turn.extra"))
            } else {
                round
            }
        },
        tile_decks: tile_decks(view, catalog),
        notice: controls.notice.clone(),
    }
}

/// Les deux pioches ; vides toutes deux, les tuiles écartées encore révélables (spec 015, C7).
fn tile_decks(view: &PlayerView, catalog: &Catalog) -> Vec<IconValue> {
    let empty = view.tile_deck.countryside + view.tile_deck.core == 0;
    let (counts, keys) = if empty {
        (
            view.removed_tiles,
            ["menu.tiles.countryside_removed", "menu.tiles.core_removed"],
        )
    } else {
        (
            view.tile_deck,
            ["menu.tiles.countryside", "menu.tiles.core"],
        )
    };
    [
        (Icon::TileCountryside, counts.countryside, keys[0]),
        (Icon::TileCore, counts.core, keys[1]),
    ]
    .into_iter()
    .map(|(icon, count, key)| {
        let count = count.to_string();
        IconValue {
            icon,
            tooltip: catalog.format(key, &[("count", &count)]),
            value: count,
        }
    })
    .collect()
}
