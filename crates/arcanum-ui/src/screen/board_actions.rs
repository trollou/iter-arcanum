//! Boutons d'action posés sur les cases du plateau (spec 075, issue privée 169) : ceux de l'ancienne
//! section Ici, avec leur case et leur infobulle.

use arcanum_core::{Action, PlayerView};
use arcanum_i18n::Catalog;

use super::{BoardActionButton, BoardActionKind, Button, Command, GameScreen, PurchaseButton};
use crate::enemy_text::token_names;
use crate::hover_text;

/// Les boutons du plateau d'un écran de partie hors des modes : d'abord ceux du lieu du héros
/// (interagir, piller, incendier, entrer, offrande), puis « Défier » sur chaque ennemi défiable
/// et « Engager le combat » sur chaque ennemi à combattre (Q9).
pub(super) fn board_action_buttons(
    view: &PlayerView,
    legal_actions: &[Action],
    screen: &GameScreen,
    catalog: &Catalog,
) -> Vec<BoardActionButton> {
    let mut buttons = Vec::new();
    if let Some(hex) = view.figure {
        // Pendant l'interaction, le bouton bulle replie ou déplie le menu du site (C30).
        if let Some(menu) = screen
            .site_menu
            .as_ref()
            .filter(|_| screen.interact.is_none())
        {
            buttons.push(BoardActionButton {
                hex: menu.hex,
                kind: BoardActionKind::Interact,
                label: catalog.text(label_key(BoardActionKind::Interact)),
                command: Some(Command::ToggleSiteMenu),
                tooltip: vec![menu.title.clone()],
            });
        }
        let mut site = |kind, button: Option<&Button>, warning: &Option<String>| {
            if let Some(button) = button {
                buttons.push(BoardActionButton {
                    hex,
                    kind,
                    label: catalog.text(label_key(kind)),
                    command: Some(button.command),
                    tooltip: tooltip(
                        &button.label,
                        [warning.as_deref()],
                        button.shortcut.as_deref(),
                        catalog,
                    ),
                });
            }
        };
        site(
            BoardActionKind::Interact,
            screen.interact.as_ref(),
            &screen.interaction_warning,
        );
        site(BoardActionKind::Plunder, screen.plunder.as_ref(), &None);
        site(
            BoardActionKind::Burn,
            screen.burn.as_ref(),
            &screen.burn_warning,
        );
        site(
            BoardActionKind::Enter,
            screen.adventure.as_ref(),
            &screen.adventure_warning,
        );
        if let Some(altar) = &screen.altar {
            buttons.push(tribute(hex, altar, &screen.altar_warning, catalog));
        }
    }
    for (index, action) in legal_actions.iter().enumerate() {
        match action {
            Action::Challenge { enemy } => {
                let names = view
                    .enemy_tokens
                    .get(enemy)
                    .map(|tokens| token_names(tokens.iter().copied(), catalog))
                    .unwrap_or_default();
                buttons.push(BoardActionButton {
                    hex: *enemy,
                    kind: BoardActionKind::Challenge,
                    label: catalog.text(label_key(BoardActionKind::Challenge)),
                    command: Some(Command::Play(index)),
                    tooltip: vec![catalog.format("combat.challenge", &[("enemy", &names)])],
                });
            }
            Action::StartCombat => {
                // L'ancien bandeau « À combattre : … » devient l'infobulle (spec 075).
                let banner = super::provoked_banner(view, catalog);
                for &hex in &view.enemies_to_fight {
                    buttons.push(BoardActionButton {
                        hex,
                        kind: BoardActionKind::StartCombat,
                        label: catalog.text(label_key(BoardActionKind::StartCombat)),
                        command: Some(Command::Play(index)),
                        tooltip: banner
                            .iter()
                            .cloned()
                            .chain([catalog.text("combat.start")])
                            .collect(),
                    });
                }
            }
            _ => {}
        }
    }
    buttons
}

/// L'offrande à l'autel, grisée sans le mana (spec 045, C33).
fn tribute(
    hex: arcanum_core::Hex,
    altar: &PurchaseButton,
    warning: &Option<String>,
    catalog: &Catalog,
) -> BoardActionButton {
    let reason = altar
        .tooltip
        .as_deref()
        .filter(|reason| warning.as_deref() != Some(*reason));
    BoardActionButton {
        hex,
        kind: BoardActionKind::Tribute,
        label: catalog.text(label_key(BoardActionKind::Tribute)),
        command: altar.command,
        tooltip: tooltip(
            &altar.label,
            [warning.as_deref(), reason],
            altar.shortcut.as_deref(),
            catalog,
        ),
    }
}

/// Libellé, avertissements, puis « Raccourci : … » (spec 075, C26).
fn tooltip<const N: usize>(
    label: &str,
    warnings: [Option<&str>; N],
    shortcut: Option<&str>,
    catalog: &Catalog,
) -> Vec<String> {
    std::iter::once(label.to_owned())
        .chain(warnings.into_iter().flatten().map(str::to_owned))
        .chain(hover_text(None, shortcut, catalog))
        .collect()
}

pub(crate) fn label_key(kind: BoardActionKind) -> &'static str {
    match kind {
        BoardActionKind::Challenge => "board.action.challenge",
        BoardActionKind::StartCombat => "board.action.start_combat",
        BoardActionKind::Interact => "board.action.interact",
        BoardActionKind::Plunder => "board.action.plunder",
        BoardActionKind::Burn => "board.action.burn",
        BoardActionKind::Enter => "board.action.enter",
        BoardActionKind::Tribute => "board.action.tribute",
    }
}
