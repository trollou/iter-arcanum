//! Mode « mettre un dé sur une couleur » des cartes de mana (spec 025).

use super::*;

/// Aucun dé mis sur une couleur.
pub(super) const NO_SET_DICE: [Option<DieSetting>; 2] = [None, None];

/// Nombre de dés déjà mis sur une couleur dans `choice`.
pub(super) fn chosen_count(choice: &SetDiceChoice) -> usize {
    choice.settings.iter().flatten().count()
}

/// Les actions qui prolongent `choice` : effet renforcé de sa carte qui met sur une couleur les dés
/// déjà choisis, dans n'importe quel ordre, et au moins un autre ; avec leur index, leurs dés mis
/// (dans l'ordre de la Source) et leur paiement (spec 025).
pub(super) fn set_dice_candidates(
    legal_actions: &[Action],
    choice: &SetDiceChoice,
) -> Vec<(usize, [Option<DieSetting>; 2], ManaPayment)> {
    let chosen = chosen_count(choice);
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::PlayCard {
                card,
                play:
                    CardPlay::Strong {
                        option,
                        mana,
                        set_dice,
                        with: None,
                        ..
                    },
            } if *card == choice.card
                && *option == choice.option
                && choice
                    .settings
                    .iter()
                    .flatten()
                    .all(|setting| set_dice.contains(&Some(*setting)))
                && set_dice.iter().flatten().count() > chosen =>
            {
                Some((index, *set_dice, *mana))
            }
            _ => None,
        })
        .collect()
}

/// Bandeau du mode « mettre un dé sur une couleur » : les couleurs une fois le dé choisi. Le
/// dernier dé fixé, son bouton joue l'action s'il n'y a qu'un paiement, sinon passe au choix du
/// mana (spec 025).
pub(super) fn set_dice_prompt(
    view: &PlayerView,
    legal_actions: &[Action],
    choice: &SetDiceChoice,
    catalog: &Catalog,
) -> SetDicePrompt {
    let chosen = chosen_count(choice);
    let candidates = set_dice_candidates(legal_actions, choice);
    let colors = match choice.die {
        None => Vec::new(),
        Some(die) => BasicColor::ALL
            .into_iter()
            .filter_map(|color| {
                let setting = DieSetting { die, color };
                let (_, set_dice, _) = candidates
                    .iter()
                    .find(|(_, set_dice, _)| set_dice.contains(&Some(setting)))?;
                // Le dernier dé : l'action a exactement les dés choisis, dans l'ordre de la Source.
                let last = set_dice.iter().flatten().count() == chosen + 1;
                let settings = *set_dice;
                let command = if !last {
                    Command::PickSetColor(color)
                } else {
                    let mana = ManaChoice {
                        card: choice.card,
                        option: choice.option,
                        discard: None,
                    };
                    match payments(legal_actions, &view.source, &mana, true, settings).as_slice() {
                        [(_, _, only)] => Command::Play(*only),
                        _ => Command::ChooseSetDiceMana {
                            card: choice.card,
                            option: choice.option,
                            set_dice: settings,
                        },
                    }
                };
                Some(StrongButton {
                    label: color_name(color, catalog),
                    color,
                    command,
                    enabled: true,
                    tooltip: None,
                })
            })
            .collect(),
    };
    SetDicePrompt {
        text: catalog.text("set_die.prompt"),
        colors,
        cancel: Button {
            label: catalog.text("mana_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    }
}

/// Tempête de mana (spec 056, C13) : chaque dé de la Source que l'option peut prendre, avec le clic
/// qui joue la carte sur ce dé.
pub(super) fn die_candidates(
    legal_actions: &[Action],
    choice: &DiscardChoice,
) -> Vec<(u8, Command)> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::PlayCard {
                card,
                play:
                    CardPlay::Basic {
                        option,
                        die: Some(die),
                        ..
                    },
            } if *card == choice.card && *option == choice.option => {
                Some((*die, Command::Play(index)))
            }
            _ => None,
        })
        .collect()
}

/// Le bandeau du choix du dé de Tempête de mana : « Tempête de mana : choisissez un dé de la
/// Source. » et « Annuler », sans couleur à choisir.
pub(super) fn die_prompt(choice: &DiscardChoice, catalog: &Catalog) -> SetDicePrompt {
    SetDicePrompt {
        text: catalog.format(
            "die_choice.prompt",
            &[("card", &card_name(choice.card, catalog))],
        ),
        colors: Vec::new(),
        cancel: Button {
            label: catalog.text("mana_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    }
}
