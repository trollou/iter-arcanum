//! Source de mana (spec 020) et inventaire des cristaux (spec 021).

use super::*;

/// `picks` : les dés cliquables en mode « mettre un dé sur une couleur » (spec 025) ; `checked` :
/// les dés cochés pour Appel du mana (spec 060).
pub(super) fn source_line(
    view: &PlayerView,
    picks: &[(u8, Command)],
    checked: &[u8],
    catalog: &Catalog,
) -> SourceLine {
    let time = view.time_of_day;
    // Spec 060, C11 : le dé volé n'est plus dans la Source, il est sur la tactique.
    let stolen = view
        .tactic
        .as_ref()
        .and_then(|tactic| tactic.stolen_die)
        .map(|(die, _)| die);
    // Spec 025 : un dé mis sur une couleur est rangé avec les dés pris.
    let faces = move |taken: bool| {
        (0u8..)
            .zip(view.source.iter().copied())
            .filter(move |(die, _)| stolen != Some(*die))
            .filter(move |(die, _)| {
                (view.dice_taken.contains(die)
                    || view.dice_set.contains(die)
                    || view.kept_die == Some(*die))
                    == taken
            })
    };
    let taken: Vec<(u8, ManaColor)> = faces(true).collect();
    let (depleted, usable): (Vec<_>, Vec<_>) =
        faces(false).partition(|(_, face)| face.is_depleted(time));
    let die = |(index, color): (u8, ManaColor)| SourceDie {
        pick: picks
            .iter()
            .find(|(die, _)| *die == index)
            .map(|(_, command)| *command),
        // Spec 058, C6 : Magie inutile à +3, plus de dé ce tour.
        forbidden: view.source_dice_forbidden,
        checked: checked.contains(&index),
        ..source_die(color, time, catalog)
    };
    SourceLine {
        title: catalog.text("source.title"),
        dice: usable.into_iter().map(die).collect(),
        depleted_title: (!depleted.is_empty()).then(|| catalog.text("source.depleted")),
        depleted: depleted.into_iter().map(die).collect(),
        taken_title: (!taken.is_empty()).then(|| catalog.text("source.taken")),
        taken: taken
            .into_iter()
            .map(|(index, color)| {
                let name = mana_name(color, catalog);
                let key = if view.kept_die == Some(index) {
                    // Spec 057 : le dé de Puits de mana, hors de la Source pour la manche.
                    "mana_claim.kept_die"
                } else if view.dice_set.contains(&index) {
                    "source.die.set"
                } else {
                    "source.die.taken"
                };
                SourceDie {
                    color,
                    tooltip: catalog.format(key, &[("color", &name)]),
                    name,
                    pick: None,
                    forbidden: false,
                    checked: false,
                }
            })
            .collect(),
        keep: None,
    }
}

/// Le nom d'une couleur de mana, sans balise : le mot du catalogue (« bleu »).
fn plain_mana_name(color: ManaColor, catalog: &Catalog) -> String {
    catalog.text(match color {
        ManaColor::Basic(BasicColor::Red) => "mana.red",
        ManaColor::Basic(BasicColor::Blue) => "mana.blue",
        ManaColor::Basic(BasicColor::White) => "mana.white",
        ManaColor::Basic(BasicColor::Green) => "mana.green",
        ManaColor::Gold => "mana.gold",
        ManaColor::Black => "mana.black",
    })
}

/// Le nom d'une couleur de mana dans sa balise de couleur (spec #68, C11 ; issue #106) :
/// « <blue>bleu</blue> », pour tous les textes calculés qui nomment une couleur (effets, invites
/// de paiement, infobulles), que l'écran colore.
pub(super) fn mana_name(color: ManaColor, catalog: &Catalog) -> String {
    let tag = crate::COLOR_TAGS
        .iter()
        .find(|(_, tagged)| *tagged == color)
        .map_or("", |(tag, _)| tag);
    format!("<{tag}>{}</{tag}>", plain_mana_name(color, catalog))
}

pub(super) fn color_name(color: BasicColor, catalog: &Catalog) -> String {
    mana_name(ManaColor::Basic(color), catalog)
}

pub(super) fn inventory_line(inventory: Inventory, catalog: &Catalog) -> InventoryLine {
    let groups = BasicColor::ALL
        .into_iter()
        .map(|color| {
            let count = inventory.crystals(color);
            let key = match color {
                BasicColor::Red => "inventory.crystals.red",
                BasicColor::Blue => "inventory.crystals.blue",
                BasicColor::White => "inventory.crystals.white",
                BasicColor::Green => "inventory.crystals.green",
            };
            CrystalGroup {
                color,
                count,
                limit: CRYSTAL_LIMIT,
                tooltip: catalog.format(
                    key,
                    &[
                        ("count", &count.to_string()),
                        ("limit", &CRYSTAL_LIMIT.to_string()),
                    ],
                ),
            }
        })
        .collect();
    InventoryLine {
        title: catalog.text("inventory.title"),
        groups,
    }
}

pub(super) fn source_die(color: ManaColor, time: TimeOfDay, catalog: &Catalog) -> SourceDie {
    let name = mana_name(color, catalog);
    let tooltip = if color.is_depleted(time) {
        let when = catalog.text(match time {
            TimeOfDay::Day => "source.by_day",
            TimeOfDay::Night => "source.by_night",
        });
        catalog.format(
            "source.die.depleted",
            &[("color", &name), ("time_of_day_lower", &when)],
        )
    } else {
        match color {
            ManaColor::Basic(_) => catalog.format("source.die.basic", &[("color", &name)]),
            ManaColor::Gold => catalog.text("source.die.gold_day"),
            ManaColor::Black => catalog.text("source.die.black_night"),
        }
    };
    SourceDie {
        color,
        name,
        tooltip,
        pick: None,
        forbidden: false,
        checked: false,
    }
}
