//! Spec 061 : le joueur fantôme dans le panneau de droite (section, ordre du tour) et la fenêtre
//! de sa défausse.

use arcanum_core::{DummyView, RoundOrder};

use super::*;

/// La section « Joueur fantôme » (C37), absente sans fantôme.
pub(super) fn dummy_section(view: &PlayerView, catalog: &Catalog) -> Option<DummySection> {
    let dummy = view.dummy.as_ref()?;
    let count = |key: &str, count: usize| catalog.format(key, &[("count", &count.to_string())]);
    Some(DummySection {
        portrait: HeroPortrait {
            image: hero_figure_image(dummy.hero),
            fallback: hero_color(dummy.hero),
            name: catalog.text(dummy.hero.label_key()),
        },
        title: catalog.format(
            "dummy.title",
            &[("hero", &catalog.text(dummy.hero.label_key()))],
        ),
        tactic: dummy.tactic.map(|tactic| {
            catalog.format(
                "dummy.tactic",
                &[
                    ("number", &tactic.number().to_string()),
                    ("name", &tactic_name(tactic, catalog)),
                ],
            )
        }),
        deck: count("dummy.deck", dummy.deck_size),
        discard: PurchaseButton {
            card: None,
            label: count("dummy.discard", dummy.discard.len()),
            command: (!dummy.discard.is_empty()).then_some(Command::OpenDummyDiscard),
            tooltip: None,
            shortcut: None,
        },
        crystals: dummy_crystals(dummy, catalog),
        skills: count("dummy.skills", dummy.skill_count),
        last_turn: last_turn_text(dummy, catalog),
        announced: dummy.announced.then(|| catalog.text("dummy.announced")),
    })
}

/// Ses cristaux (retour de Mathieu) : l'icône de l'inventaire, une par cristal, rangées par
/// couleur.
fn dummy_crystals(dummy: &DummyView, catalog: &Catalog) -> DummyCrystals {
    let held: Vec<(BasicColor, u8)> = BasicColor::ALL
        .into_iter()
        .map(|color| (color, dummy.crystals(color)))
        .filter(|(_, count)| *count > 0)
        .collect();
    let tooltip: Vec<String> = held
        .iter()
        .map(|&(color, count)| {
            catalog.format(
                "dummy.crystal_count",
                &[
                    ("color", &color_name(color, catalog)),
                    ("count", &count.to_string()),
                ],
            )
        })
        .collect();
    DummyCrystals {
        title: catalog.text("dummy.crystals"),
        crystals: held
            .iter()
            .flat_map(|&(color, count)| std::iter::repeat_n(color, usize::from(count)))
            .collect(),
        tooltip: tooltip.join(", "),
    }
}

/// Ce qu'il a retourné à son dernier tour ; au-delà de 3 cartes, la couleur de la 3e et le
/// nombre de cartes en plus (spec 061, Libellés).
fn last_turn_text(dummy: &DummyView, catalog: &Catalog) -> String {
    let names: Vec<String> = dummy
        .last_turn
        .iter()
        .map(|&card| card_name(card, catalog))
        .collect();
    let cards = names.join(", ");
    let third_color = dummy
        .last_turn
        .get(2)
        .and_then(|card| card.card().definition())
        .map(|definition| definition.color);
    match (names.len(), third_color) {
        (0, _) => catalog.text("dummy.last_turn_none"),
        (len, Some(color)) if len > 3 => catalog.format(
            "dummy.last_turn_extra",
            &[
                ("cards", &cards),
                ("color", &color_name(color, catalog)),
                ("count", &(len - 3).to_string()),
            ],
        ),
        _ => catalog.format("dummy.last_turn", &[("cards", &cards)]),
    }
}

/// La ligne d'ordre du tour (C39), absente sans fantôme ou avant les tactiques.
pub(super) fn round_order_line(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.round_order.map(|order| {
        catalog.text(match order {
            RoundOrder::PlayerFirst => "round_order.player_first",
            RoundOrder::DummyFirst => "round_order.dummy_first",
        })
    })
}

/// La fenêtre de la défausse du fantôme (C38), comme celle du joueur (spec 073) : la dernière
/// carte retournée en tête ; aucune fenêtre quand sa défausse est vide.
pub fn dummy_discard_window(view: &PlayerView, catalog: &Catalog) -> Option<DiscardWindow> {
    let dummy = view.dummy.as_ref()?;
    pile_window(catalog.text("dummy.discard_title"), &dummy.discard, catalog)
}
