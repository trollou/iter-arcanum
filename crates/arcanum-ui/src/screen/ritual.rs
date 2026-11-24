//! Fenêtre du choix en attente de Rituel de sang renforcé (spec 056, C4, C5) : les couleurs des
//! trois jetons, une par une, puis le cristal payé d'un mana de base, ou « Non merci ».

use super::*;

/// La fenêtre, présente tant que la vue a un choix en attente (`PlayerView::mana_choice`).
pub(super) fn blood_ritual_window(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<TokenChoice> {
    let choice = view.mana_choice?;
    let mut tokens = Vec::new();
    let mut payments = Vec::new();
    let mut decline = None;
    for (index, action) in legal_actions.iter().enumerate() {
        match action {
            Action::ChooseTokenColor { color } => tokens.push(TokenButton {
                color: *color,
                label: mana_name(*color, catalog),
                command: Command::Play(index),
                shortcut: None,
            }),
            Action::PayForCrystal { color, mana } => payments.push(Button {
                label: format!(
                    "{} · {}",
                    catalog.format(
                        "effect.gain_crystal",
                        &[("color", &color_name(*color, catalog))]
                    ),
                    payment_label(view, *mana, ManaColor::Basic(*color), catalog)
                ),
                command: Command::Play(index),
                shortcut: None,
            }),
            Action::DeclineCrystal => {
                decline = Some(Button {
                    label: catalog.text("choice.decline"),
                    command: Command::Play(index),
                    shortcut: None,
                });
            }
            _ => {}
        }
    }
    let title = if choice.tokens_left > 0 {
        catalog.format(
            "choice.token_color",
            &[("left", &choice.tokens_left.to_string())],
        )
    } else {
        catalog.text("choice.crystal")
    };
    Some(TokenChoice {
        title,
        tokens,
        payments,
        decline,
    })
}
