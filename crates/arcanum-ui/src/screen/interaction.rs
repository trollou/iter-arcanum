//! Réputation et interaction du panneau latéral (spec 040).

use arcanum_core::{Feature, Interaction};

use super::*;
use crate::board::feature_key;

/// Un nombre signé : « +1 », « −2 » (signe moins typographique), « 0 ».
pub(super) fn signed(value: i32) -> String {
    match value {
        v if v > 0 => format!("+{v}"),
        v if v < 0 => minus(v),
        _ => "0".to_string(),
    }
}

/// Un nombre avec son signe s'il est négatif seulement : « 3 », « −2 ».
pub(super) fn with_minus(value: i32) -> String {
    if value < 0 {
        minus(value)
    } else {
        value.to_string()
    }
}

fn minus(value: i32) -> String {
    format!("\u{2212}{}", value.unsigned_abs())
}

/// « Réputation : +2 · bonus d'interaction +1 », ou « Réputation : X · interaction impossible ».
pub(super) fn reputation_line(view: &PlayerView, catalog: &Catalog) -> String {
    match view.reputation_bonus {
        Some(bonus) => catalog.format(
            "reputation.line",
            &[
                ("reputation", &signed(i32::from(view.reputation.value()))),
                ("bonus", &signed(i32::from(bonus))),
            ],
        ),
        None => catalog.text("reputation.line_x"),
    }
}

/// Le bouton « Interagir avec les habitants », quand l'interaction est légale, et la ligne qui
/// nomme le mouvement qu'elle ferait perdre (MKUE p. 6, 6.e).
pub(super) fn interact_button(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> (Option<Button>, Option<String>) {
    let Some(index) = legal_actions
        .iter()
        .position(|action| *action == Action::Interact)
    else {
        return (None, None);
    };
    let button = Button {
        label: catalog.text("interaction.button"),
        command: Command::Play(index),
        shortcut: shortcut_label(Shortcut::Interact, catalog),
    };
    // Issue #95 : l'influence jouée avant reste (variante maison), seul le mouvement se perd.
    let warning = (view.move_points > 0).then(|| {
        catalog.format(
            "interaction.lost_points",
            &[("move", &view.move_points.to_string())],
        )
    });
    (Some(button), warning)
}

/// « Interaction : Village » pendant l'interaction.
pub(super) fn interaction_header(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.interaction.map(|Interaction { site, .. }| {
        catalog.format(
            "interaction.header",
            &[("site", &catalog.text(feature_key(Feature::Site(site))))],
        )
    })
}
