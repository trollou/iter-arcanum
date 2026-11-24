//! Fenêtre de Méditation et de Transe (spec 057, C19, C20) : les cartes en attente, leur ordre
//! choisi carte par carte, puis « Sur le deck » ou « Sous le deck ».

use super::*;

/// La fenêtre, présente tant que la vue a des cartes de Méditation en attente.
pub(super) fn meditation_window(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<MeditationWindow> {
    let pending = view.meditation.as_ref()?;
    let picked: Vec<CardId> = match mode {
        ScreenMode::Meditating(picked) => picked.clone(),
        _ => Vec::new(),
    };
    // Sans choix, des cartes tirées au hasard gardent l'ordre du tirage.
    let order = if picked.len() == pending.pick {
        Some(picked.clone())
    } else if picked.is_empty() && pending.cards.len() == pending.pick {
        Some(pending.cards.clone())
    } else {
        None
    };
    let place = |top: bool, key: &str| {
        let order = order.clone()?;
        let wanted = Action::PlaceOnDeck { top, order };
        let index = legal_actions.iter().position(|action| *action == wanted)?;
        Some(Button {
            label: catalog.text(key),
            command: Command::Play(index),
            shortcut: None,
        })
    };
    Some(MeditationWindow {
        title: catalog.text("meditation.title"),
        cards: pending
            .cards
            .iter()
            .map(|&card| MeditationCard {
                name: card_name(card, catalog),
                place: picked
                    .iter()
                    .position(|&chosen| chosen == card)
                    .map(|index| index + 1),
                pick: Some(Command::PickMeditationCard(card)),
            })
            .collect(),
        top: place(true, "meditation.top"),
        bottom: place(false, "meditation.bottom"),
    })
}
