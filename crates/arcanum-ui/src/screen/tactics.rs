//! Tactiques (spec 060) : fenêtre du choix en grille de 2 × 3, choix qui suivent la prise, tactique
//! du joueur dans le panneau de droite, fenêtres de Provisions et de Longue Nuit.

use arcanum_core::{Tactic, TacticSlot};

use super::*;

pub(super) fn tactic_name(tactic: Tactic, catalog: &Catalog) -> String {
    catalog.text(&format!("tactic.{}.name", tactic.id()))
}

fn tactic_text(tactic: Tactic, catalog: &Catalog) -> String {
    catalog.text(&format!("tactic.{}.text", tactic.id()))
}

fn index_of(legal_actions: &[Action], wanted: &Action) -> Option<usize> {
    legal_actions.iter().position(|action| action == wanted)
}

/// La fenêtre du choix, pendant le choix seulement : chaque tactique à la place de son numéro,
/// un trou pour une tactique retirée (remarque de Mathieu, grille 2 × 3).
pub(super) fn tactic_window(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<TacticWindow> {
    let (first, _) = view.tactic_choice.first()?;
    let selected = match mode {
        ScreenMode::ChoosingTactic(tactic) => Some(*tactic),
        _ => None,
    }
    .and_then(|tactic| {
        index_of(legal_actions, &Action::ChooseTactic { tactic }).map(|i| (tactic, i))
    });
    let mut slots: Vec<Option<TacticCard>> = vec![None; 6];
    for &(tactic, slot) in &view.tactic_choice {
        let Some(place) = slots.get_mut(usize::from(tactic.number()).saturating_sub(1)) else {
            continue;
        };
        *place = match slot {
            TacticSlot::Removed => None,
            TacticSlot::Available | TacticSlot::Taken(_) => Some(TacticCard {
                tactic,
                number: tactic.number(),
                name: tactic_name(tactic, catalog),
                text: tactic_text(tactic, catalog),
                selected: selected.is_some_and(|(chosen, _)| chosen == tactic),
                select: (slot == TacticSlot::Available).then_some(Command::SelectTactic(tactic)),
            }),
        };
    }
    let time = catalog.text(match first.time_of_day() {
        TimeOfDay::Day => "time_of_day.day",
        TimeOfDay::Night => "time_of_day.night",
    });
    Some(TacticWindow {
        title: catalog.format("tactic.choose_title", &[("time", &time)]),
        slots,
        take: PurchaseButton {
            card: None,
            label: catalog.text("tactic.take"),
            command: selected.map(|(_, index)| Command::Play(index)),
            tooltip: None,
            shortcut: shortcut_label(Shortcut::Confirm, catalog),
        },
    })
}

/// Une carte comparée à une autre : les Blessures sont identiques (spec 034).
fn card_key(card: CardId) -> Option<CardId> {
    (!card.card().is_wound()).then_some(card)
}

fn keys(cards: &[CardId]) -> Vec<Option<CardId>> {
    let mut keys: Vec<Option<CardId>> = cards.iter().copied().map(card_key).collect();
    keys.sort();
    keys
}

/// Vrai si `cards` contient toutes les cartes de `wanted`, Blessures comptées par nombre.
fn includes(cards: &[CardId], wanted: &[CardId]) -> bool {
    let mut left = keys(cards);
    keys(wanted).into_iter().all(|key| {
        left.iter()
            .position(|other| *other == key)
            .map(|at| left.remove(at))
            .is_some()
    })
}

/// Le choix de cartes de la main en cours : Faux départ (en attente) ou Méditation nocturne
/// (ouverte par son bouton).
pub(super) struct CardPicking {
    meditation: bool,
    picks: Vec<CardId>,
    /// Les choix légaux : index de l'action et ses cartes.
    choices: Vec<(usize, Vec<CardId>)>,
}

impl CardPicking {
    fn confirm(&self) -> Option<usize> {
        self.choices
            .iter()
            .find(|(_, cards)| cards.len() == self.picks.len() && includes(cards, &self.picks))
            .map(|(index, _)| *index)
    }

    /// La case d'une carte : cochée, ou cochable si un choix légal la prend avec les cochées.
    fn toggle(&self, card: CardId, catalog: &Catalog) -> Option<DiscardToggle> {
        let checked = self.picks.contains(&card);
        let mut with = self.picks.clone();
        with.push(card);
        (checked || self.choices.iter().any(|(_, cards)| includes(cards, &with))).then(|| {
            DiscardToggle {
                // Méditation nocturne : la case seule, le bandeau dit ce qu'elle fait.
                label: if self.meditation {
                    String::new()
                } else {
                    catalog.text("end_turn.discard")
                },
                checked,
                command: Command::ToggleTacticCard(card),
            }
        })
    }
}

pub(super) fn card_picking(mode: &ScreenMode, legal_actions: &[Action]) -> Option<CardPicking> {
    let rethink: Vec<(usize, Vec<CardId>)> = legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::RethinkDiscard { cards } => Some((index, cards.clone())),
            _ => None,
        })
        .collect();
    let picks = match mode {
        ScreenMode::PickingTacticCards(picks) => Some(picks.clone()),
        _ => None,
    };
    if !rethink.is_empty() {
        return Some(CardPicking {
            meditation: false,
            picks: picks.unwrap_or_default(),
            choices: rethink,
        });
    }
    let choices: Vec<(usize, Vec<CardId>)> = legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::MidnightMeditation { cards } => Some((index, cards.clone())),
            _ => None,
        })
        .collect();
    (!choices.is_empty()).then_some(())?;
    Some(CardPicking {
        meditation: true,
        picks: picks?,
        choices,
    })
}

/// Carte de la main pendant Faux départ ou Méditation nocturne : sa case seulement.
pub(super) fn picking_hand_card(
    card: CardId,
    picking: &CardPicking,
    catalog: &Catalog,
) -> HandCard {
    HandCard {
        discard_toggle: picking.toggle(card, catalog),
        ..idle_card(card, catalog)
    }
}

/// Les choix légaux d'Appel du mana : index de l'action et ses dés.
type DiceChoices<'a> = Vec<(usize, &'a [u8])>;

/// Les dés cochés d'Appel du mana, quand son choix est ouvert, et ses choix légaux.
fn dice_picking<'a>(
    mode: &'a ScreenMode,
    legal_actions: &'a [Action],
) -> Option<(&'a [u8], DiceChoices<'a>)> {
    let ScreenMode::PickingTacticDice(picks) = mode else {
        return None;
    };
    let choices: DiceChoices = legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::ManaSearch { dice } => Some((index, dice.as_slice())),
            _ => None,
        })
        .collect();
    (!choices.is_empty()).then_some((picks.as_slice(), choices))
}

fn same_dice(a: &[u8], b: &[u8]) -> bool {
    let (mut a, mut b) = (a.to_vec(), b.to_vec());
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

/// Les dés cliquables de la Source : Vol de mana (le dé pris) ou Appel du mana (cocher), et les
/// dés cochés.
pub(super) fn tactic_die_picks(
    mode: &ScreenMode,
    legal_actions: &[Action],
) -> (Vec<(u8, Command)>, Vec<u8>) {
    let mut picks: Vec<(u8, Command)> = legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::StealDie { die } => Some((*die, Command::Play(index))),
            _ => None,
        })
        .collect();
    let Some((checked, choices)) = dice_picking(mode, legal_actions) else {
        return (picks, Vec::new());
    };
    // Un dé se coche si les cochés et lui forment un choix légal : les dés or d'abord (Compendium,
    // *Mana Search* S1).
    let mut dice: Vec<u8> = choices
        .iter()
        .flat_map(|(_, dice)| dice.iter().copied())
        .filter(|die| {
            let mut with = checked.to_vec();
            with.push(*die);
            checked.contains(die) || choices.iter().any(|(_, dice)| same_dice(dice, &with))
        })
        .collect();
    dice.sort_unstable();
    dice.dedup();
    picks.extend(
        dice.into_iter()
            .map(|die| (die, Command::ToggleTacticDie(die))),
    );
    (picks, checked.to_vec())
}

fn cancel_button(catalog: &Catalog) -> Button {
    Button {
        label: catalog.text("discard_choice.cancel"),
        command: Command::CancelChoice,
        shortcut: shortcut_label(Shortcut::Cancel, catalog),
    }
}

/// Le bandeau du choix en cours : Faux départ, Vol de mana, Appel du mana, Méditation nocturne.
pub(super) fn tactic_prompt(
    _view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<AnnouncePrompt> {
    if let Some(picking) = card_picking(mode, legal_actions) {
        let count = picking.picks.len().to_string();
        let (text, confirm) = if picking.meditation {
            ("tactic.meditation_prompt", "tactic.meditation_confirm")
        } else {
            ("tactic.rethink_prompt", "tactic.rethink_confirm")
        };
        let mut buttons: Vec<Button> = picking
            .confirm()
            .map(|index| Button {
                label: catalog.format(confirm, &[("count", &count)]),
                command: Command::Play(index),
                shortcut: None,
            })
            .into_iter()
            .collect();
        // Faux départ ne s'annule pas : la tactique est prise (Q3).
        if picking.meditation {
            buttons.push(cancel_button(catalog));
        }
        return Some(AnnouncePrompt {
            text: catalog.text(text),
            buttons,
        });
    }
    if legal_actions
        .iter()
        .any(|action| matches!(action, Action::StealDie { .. }))
    {
        return Some(AnnouncePrompt {
            text: catalog.text("tactic.steal_prompt"),
            buttons: Vec::new(),
        });
    }
    let (checked, choices) = dice_picking(mode, legal_actions)?;
    let confirm = choices.iter().find(|(_, dice)| same_dice(dice, checked));
    let mut buttons: Vec<Button> = confirm
        .map(|(index, _)| Button {
            label: catalog.format(
                "tactic.mana_search_confirm",
                &[("count", &checked.len().to_string())],
            ),
            command: Command::Play(*index),
            shortcut: None,
        })
        .into_iter()
        .collect();
    buttons.push(cancel_button(catalog));
    Some(AnnouncePrompt {
        text: catalog.text("tactic.mana_search_prompt"),
        buttons,
    })
}

/// Préparation : les cartes du deck (C30).
pub(super) fn preparation_window(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<PreparationWindow> {
    if view.preparation_choice.is_empty() {
        return None;
    }
    Some(PreparationWindow {
        title: catalog.text("tactic.preparation_prompt"),
        cards: view
            .preparation_choice
            .iter()
            .filter_map(|&card| {
                let index = index_of(legal_actions, &Action::PrepareCard { card })?;
                Some(Button {
                    label: card_name(card, catalog),
                    command: Command::Play(index),
                    shortcut: None,
                })
            })
            .collect(),
    })
}

fn purchase(label: String, command: Option<usize>) -> PurchaseButton {
    PurchaseButton {
        card: None,
        label,
        command: command.map(Command::Play),
        tooltip: None,
        shortcut: None,
    }
}

/// Provisions avant le tour, Longue Nuit pendant la pioche.
pub(super) fn tactic_decision(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<TacticDecision> {
    let take = index_of(legal_actions, &Action::SparingPower { take: true });
    let store = index_of(legal_actions, &Action::SparingPower { take: false });
    if take.is_some() || store.is_some() {
        let count = view
            .tactic
            .as_ref()
            .map_or(0, |tactic| tactic.stored)
            .to_string();
        return Some(TacticDecision {
            text: catalog.format("tactic.sparing_prompt", &[("count", &count)]),
            buttons: vec![
                purchase(catalog.text("tactic.sparing_store"), store),
                purchase(
                    catalog.format("tactic.sparing_take", &[("count", &count)]),
                    take,
                ),
            ],
        });
    }
    let decline = index_of(legal_actions, &Action::DeclineLongNight)?;
    Some(TacticDecision {
        text: catalog.text("tactic.long_night_prompt"),
        buttons: vec![
            purchase(
                catalog.text("tactic.long_night_button"),
                index_of(legal_actions, &Action::UseLongNight),
            ),
            purchase(catalog.text("tactic.long_night_decline"), Some(decline)),
        ],
    })
}

/// La tactique du joueur dans le panneau de droite, avec ses boutons légaux.
pub(super) fn tactic_line(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<TacticLine> {
    let tactic = view.tactic.as_ref()?;
    let mut notes = Vec::new();
    if let Some((_, color)) = tactic.stolen_die {
        notes.push(catalog.format(
            "tactic.stolen_die",
            &[("color", &mana_name(color, catalog))],
        ));
    }
    if tactic.stored > 0 {
        notes.push(catalog.format("tactic.stored", &[("count", &tactic.stored.to_string())]));
    }
    if tactic.right_moment {
        notes.push(catalog.text("tactic.right_moment_announced"));
    }
    let play = |action: &Action, key: &str| {
        index_of(legal_actions, action).map(|index| Button {
            label: catalog.text(key),
            command: Command::Play(index),
            shortcut: None,
        })
    };
    let open = |wanted: fn(&Action) -> bool, command: Command, key: &str| {
        legal_actions.iter().any(wanted).then(|| Button {
            label: catalog.text(key),
            command,
            shortcut: None,
        })
    };
    // Longue Nuit en pleine pioche : la fenêtre la propose, pas le panneau.
    let paused = legal_actions.contains(&Action::DeclineLongNight);
    let buttons = [
        play(&Action::AnnounceRightMoment, "tactic.right_moment_button"),
        play(&Action::UseLongNight, "tactic.long_night_button").filter(|_| !paused),
        open(
            |action| matches!(action, Action::ManaSearch { .. }),
            Command::OpenManaSearch,
            "tactic.mana_search_button",
        ),
        open(
            |action| matches!(action, Action::MidnightMeditation { .. }),
            Command::OpenMidnightMeditation,
            "tactic.meditation_button",
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    Some(TacticLine {
        number: catalog.format(
            "tactic.number",
            &[("number", &tactic.tactic.number().to_string())],
        ),
        name: tactic_name(tactic.tactic, catalog),
        text: tactic_text(tactic.tactic, catalog),
        flipped: tactic.flipped.then(|| catalog.text("tactic.flipped")),
        notes,
        buttons,
    })
}
