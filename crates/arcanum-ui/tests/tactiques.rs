//! Spec 060 côté affichage (Tactiques) : fenêtre du choix en grille de 2 × 3, choix qui suivent la
//! prise, tactique du joueur dans le panneau de droite, dé volé, fenêtres de Provisions et de
//! Longue Nuit, raccourcis 1 à 6 et Entrée.

use arcanum_core::{
    Action, BasicColor, CardId, Event, GameState, Hero, ManaColor, Scenario, Tactic, TimeOfDay,
    apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Button, Command, GameScreen, ManaChoice, PurchaseButton, ScreenMode, ScreenPhase, Shortcut,
    game_screen, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{bar_of, session_of};

use arcanum_core::BasicAction::{
    Concentration, March, Promise, Rage, Stamina, Swiftness, Threaten,
};

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn tactic_hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Rage, 1),
        card(Promise, 1),
    ]
}

/// Première reconnaissance, Tovak sur une plaine, `hand` en main et, si donné, `deck` derrière ;
/// le choix de la tactique de `time` attend (comme le décor des tests d'arcanum-core).
fn decor_with(time: TimeOfDay, hand: &[CardId], deck: Option<&[CardId]>) -> Vec<Event> {
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        figure: Some(PLAIN_W),
        hand: hand.to_vec(),
        time,
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events();
    if let Some(deck) = deck {
        let mut order = hand.to_vec();
        order.extend_from_slice(deck);
        events.push(Event::DeckShuffled {
            player: SOLO,
            order,
        });
        events.extend(
            hand.iter()
                .map(|&card| Event::CardDrawn { player: SOLO, card }),
        );
    }
    events
}

fn decor(time: TimeOfDay) -> Vec<Event> {
    decor_with(time, &tactic_hand(), None)
}

fn state_of(events: &[Event]) -> GameState {
    events.iter().fold(GameState::default(), apply)
}

/// Les événements de `events`, puis ceux de `action`, jouée.
fn then(mut events: Vec<Event>, action: Action) -> Vec<Event> {
    let state = state_of(&events);
    assert!(
        legal_actions(&state, SOLO).contains(&action),
        "{action:?} doit être légale"
    );
    events.extend(events_of(&state, &action));
    events
}

fn choose(tactic: Tactic) -> Action {
    Action::ChooseTactic { tactic }
}

fn taken(time: TimeOfDay, tactic: Tactic) -> Vec<Event> {
    then(decor(time), choose(tactic))
}

fn screen_in(events: &[Event], mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
    let state = state_of(events);
    let legal = legal_actions(&state, SOLO);
    game_screen(
        &view(&state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        mode,
        catalog,
    )
}

fn screen_of(events: &[Event], catalog: &Catalog) -> GameScreen {
    screen_in(events, &ScreenMode::Normal, catalog)
}

/// La commande qui joue `action`, légale après `events`.
#[track_caller]
fn play(events: &[Event], action: &Action) -> Command {
    let index = legal_actions(&state_of(events), SOLO)
        .iter()
        .position(|legal| legal == action)
        .unwrap_or_else(|| panic!("{action:?} doit être légale"));
    Command::Play(index)
}

fn labels(buttons: &[Button]) -> Vec<(&str, Command)> {
    buttons
        .iter()
        .map(|button| (button.label.as_str(), button.command))
        .collect()
}

fn purchase_labels(buttons: &[PurchaseButton]) -> Vec<(&str, Option<Command>)> {
    buttons
        .iter()
        .map(|button| (button.label.as_str(), button.command))
        .collect()
}

// --- Fenêtre du choix ---

// Spec 060 C1 — MKUE p. 4, 3.a ; p. 3 : au début de la manche, les six tactiques du moment, chacune
// à la place de son numéro (grille 2 × 3), rien d'encadré, « Prendre » grisé.
#[test]
fn tactic_window_shows_six_day_tactics_in_place() {
    let catalog = french();
    let screen = screen_of(&decor(TimeOfDay::Day), &catalog);
    let window = screen.tactic_choice.expect("la fenêtre du choix");
    assert_eq!(window.title, "Choisir une tactique (Jour)");
    let cards: Vec<(u8, &str, bool)> = window
        .slots
        .iter()
        .map(|slot| {
            let card = slot.as_ref().expect("aucune tactique retirée");
            (card.number, card.name.as_str(), card.selected)
        })
        .collect();
    assert_eq!(
        cards,
        [
            (1, "Lève-tôt", false),
            (2, "Faux départ", false),
            (3, "Vol de mana", false),
            (4, "Planification", false),
            (5, "Départ flamboyant", false),
            (6, "Au bon moment", false),
        ]
    );
    let steal = window.slots[2].as_ref().expect("Vol de mana");
    assert_eq!(steal.text, catalog.text("tactic.mana_steal.text"));
    assert_eq!(steal.select, Some(Command::SelectTactic(Tactic::ManaSteal)));
    assert_eq!(window.take.label, "Prendre cette tactique");
    assert_eq!(window.take.command, None);
}

// Spec 060 C6, C7 — MKUE p. 19, *Special Rules* : une tactique retirée laisse un trou à sa place ;
// les autres ne bougent pas (remarque de Mathieu, grille 2 × 3).
#[test]
fn removed_tactic_leaves_a_hole() {
    let catalog = french();
    let mut events = decor(TimeOfDay::Night);
    events.insert(
        events.len() - 1,
        Event::TacticsRemoved {
            tactics: vec![Tactic::MidnightMeditation],
        },
    );
    let window = screen_of(&events, &catalog)
        .tactic_choice
        .expect("la fenêtre du choix");
    assert_eq!(window.title, "Choisir une tactique (Nuit)");
    let numbers: Vec<Option<u8>> = window
        .slots
        .iter()
        .map(|slot| slot.as_ref().map(|card| card.number))
        .collect();
    assert_eq!(numbers, [Some(1), Some(2), Some(3), None, Some(5), Some(6)]);
}

// Spec 060 C1, C3 — Interface : un clic encadre la tactique, « Prendre » la prend.
#[test]
fn framed_tactic_is_taken_by_the_take_button() {
    let catalog = french();
    let events = decor(TimeOfDay::Day);
    let screen = screen_in(
        &events,
        &ScreenMode::ChoosingTactic(Tactic::ManaSteal),
        &catalog,
    );
    let window = screen.tactic_choice.expect("la fenêtre du choix");
    let selected: Vec<bool> = window
        .slots
        .iter()
        .map(|slot| slot.as_ref().is_some_and(|card| card.selected))
        .collect();
    assert_eq!(selected, [false, false, true, false, false, false]);
    assert_eq!(
        window.take.command,
        Some(play(&events, &choose(Tactic::ManaSteal)))
    );
}

// Spec 060 C3 : la tactique prise, la fenêtre se ferme et la tactique passe au panneau de droite.
#[test]
fn taken_tactic_goes_to_the_side_panel() {
    let catalog = french();
    let screen = screen_of(&taken(TimeOfDay::Day, Tactic::Planning), &catalog);
    assert_eq!(screen.tactic_choice, None);
    let line = screen.tactic.expect("la tactique du joueur");
    assert_eq!(line.number, "N° 4");
    assert_eq!(line.name, "Planification");
    assert_eq!(line.text, catalog.text("tactic.planning.text"));
    assert_eq!(line.flipped, None);
    assert!(line.buttons.is_empty());
}

// Spec 060, Raccourcis : 1 à 6 encadrent la tactique de ce numéro, Entrée la prend ; les autres
// touches de jeu n'agissent pas, et un numéro retiré non plus.
#[test]
fn keys_frame_and_take_a_tactic() {
    let catalog = french();
    let mut events = decor(TimeOfDay::Day);
    events.insert(
        events.len() - 1,
        Event::TacticsRemoved {
            tactics: vec![Tactic::Planning],
        },
    );
    let mut session = session_of(events);
    session.shortcut(Shortcut::Take(4));
    session.shortcut(Shortcut::Confirm);
    assert!(session.screen(&catalog).tactic_choice.is_some());
    session.shortcut(Shortcut::Take(3));
    session.shortcut(Shortcut::EndTurn);
    let window = session
        .screen(&catalog)
        .tactic_choice
        .expect("encore ouverte");
    assert!(window.slots[2].as_ref().is_some_and(|card| card.selected));
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    assert_eq!(
        session.view().tactic.map(|tactic| tactic.tactic),
        Some(Tactic::ManaSteal)
    );
}

// Spec 060, Interface : un clic sur une tactique l'encadre.
#[test]
fn clicking_a_tactic_frames_it() {
    let catalog = french();
    let mut session = session_of(decor(TimeOfDay::Day));
    session.command(Command::SelectTactic(Tactic::GreatStart));
    let window = session.screen(&catalog).tactic_choice.expect("ouverte");
    assert!(window.slots[4].as_ref().is_some_and(|card| card.selected));
}

// Spec 060 — Interface ; issue du fil moteur : pendant le choix d'une nouvelle manche, la barre de
// menu montre le tour 1 de la manche, pas le dernier tour de la manche finie.
#[test]
fn menu_bar_shows_turn_one_during_the_choice() {
    let catalog = french();
    let mut events = taken(TimeOfDay::Day, Tactic::EarlyBird);
    events.extend([
        Event::TurnStarted {
            player: SOLO,
            turn: 5,
        },
        Event::RoundEnded { round: 1 },
        Event::RoundStarted {
            round: 2,
            time_of_day: TimeOfDay::Night,
        },
    ]);
    let state = state_of(&events);
    assert!(
        legal_actions(&state, SOLO).contains(&choose(Tactic::FromTheDusk)),
        "le choix de la Nuit attend"
    );
    let bar = bar_of(&view(&state, SOLO), &catalog);
    assert_eq!(bar.round, "Manche 2/4 (tour 1)");
}

// --- Jour : au choix ---

// Spec 060 C9 — Interface : Faux départ, les cartes de la main à cocher, bandeau et bouton qui
// défausse les cartes cochées.
#[test]
fn rethink_checks_hand_cards() {
    let catalog = french();
    let events = taken(TimeOfDay::Day, Tactic::Rethink);
    let screen = screen_of(&events, &catalog);
    let prompt = screen.tactic_prompt.expect("le bandeau de Faux départ");
    assert_eq!(
        prompt.text,
        "Faux départ : cochez jusqu'à 3 cartes à défausser"
    );
    let none = Action::RethinkDiscard { cards: Vec::new() };
    assert_eq!(
        labels(&prompt.buttons),
        [("Défausser et piocher 0 carte(s)", play(&events, &none))]
    );
    let toggles: Vec<(String, bool, Command)> = screen
        .hand
        .iter()
        .map(|card| {
            let toggle = card.discard_toggle.as_ref().expect("une case");
            (toggle.label.clone(), toggle.checked, toggle.command)
        })
        .collect();
    assert_eq!(
        toggles[0],
        (
            "Défausser".to_string(),
            false,
            Command::ToggleTacticCard(card(March, 1))
        )
    );
}

// Spec 060 C9, C10 : les cartes cochées font le bouton ; à 3 cartes, les autres ne se cochent plus.
#[test]
fn rethink_button_follows_checked_cards() {
    let catalog = french();
    let events = taken(TimeOfDay::Day, Tactic::Rethink);
    let two = vec![card(Stamina, 1), card(March, 1)];
    let screen = screen_in(&events, &ScreenMode::PickingTacticCards(two), &catalog);
    let wanted = Action::RethinkDiscard {
        cards: vec![card(March, 1), card(Stamina, 1)],
    };
    assert_eq!(
        labels(&screen.tactic_prompt.expect("bandeau").buttons),
        [("Défausser et piocher 2 carte(s)", play(&events, &wanted))]
    );
    let three = vec![card(March, 1), card(March, 2), card(Stamina, 1)];
    let screen = screen_in(&events, &ScreenMode::PickingTacticCards(three), &catalog);
    let checkable: Vec<Option<bool>> = screen
        .hand
        .iter()
        .map(|card| card.discard_toggle.as_ref().map(|toggle| toggle.checked))
        .collect();
    assert_eq!(checkable, [Some(true), Some(true), Some(true), None, None]);
}

// Spec 060 C9 : une case cochée dans la session passe au bouton.
#[test]
fn toggling_a_card_checks_it() {
    let catalog = french();
    let mut session = session_of(taken(TimeOfDay::Day, Tactic::Rethink));
    session.command(Command::ToggleTacticCard(card(Rage, 1)));
    let screen = session.screen(&catalog);
    let rage = screen
        .hand
        .iter()
        .find(|hand| hand.card == card(Rage, 1))
        .and_then(|hand| hand.discard_toggle.as_ref())
        .expect("la case de Rage");
    assert!(rage.checked);
    assert_eq!(
        screen.tactic_prompt.expect("bandeau").buttons[0].label,
        "Défausser et piocher 1 carte(s)"
    );
}

// Spec 060 C11 — Interface : Vol de mana, les dés de couleur de base de la Source se cliquent.
#[test]
fn mana_steal_picks_a_source_die() {
    let catalog = french();
    let events = taken(TimeOfDay::Day, Tactic::ManaSteal);
    let screen = screen_of(&events, &catalog);
    let prompt = screen.tactic_prompt.expect("le bandeau de Vol de mana");
    assert_eq!(
        prompt.text,
        "Vol de mana : choisissez un dé de couleur de base"
    );
    assert!(prompt.buttons.is_empty());
    let picks: Vec<Option<Command>> = screen.source.dice.iter().map(|die| die.pick).collect();
    let expected: Vec<Option<Command>> = (0..4)
        .map(|die| Some(play(&events, &Action::StealDie { die })))
        .collect();
    assert_eq!(picks, expected);
}

// Spec 060 C11, C12, C35 : le dé volé quitte la Source et va sur la tactique, dans le panneau.
#[test]
fn stolen_die_shows_on_the_tactic() {
    let catalog = french();
    let events = then(
        taken(TimeOfDay::Day, Tactic::ManaSteal),
        Action::StealDie { die: 2 },
    );
    let screen = screen_of(&events, &catalog);
    let colors: Vec<ManaColor> = screen.source.dice.iter().map(|die| die.color).collect();
    assert_eq!(
        colors,
        [
            ManaColor::Basic(BasicColor::Green),
            ManaColor::Basic(BasicColor::Red),
            ManaColor::Basic(BasicColor::White),
        ]
    );
    let line = screen.tactic.expect("la tactique");
    assert_eq!(
        (line.number.as_str(), line.name.as_str()),
        ("N° 3", "Vol de mana")
    );
    assert_eq!(line.notes, ["Dé volé : bleu"]);
}

// Spec 060 C12 — Interface : le dé volé, une ligne de plus dans le choix du mana.
#[test]
fn stolen_die_pays_in_the_mana_choice() {
    let catalog = french();
    let events = then(
        taken(TimeOfDay::Day, Tactic::ManaSteal),
        Action::StealDie { die: 2 },
    );
    let mode = ScreenMode::ChoosingMana(ManaChoice {
        card: card(Stamina, 1),
        option: 0,
        discard: None,
    });
    let prompt = screen_in(&events, &mode, &catalog)
        .mana_prompt
        .expect("le choix du mana");
    let payments: Vec<&str> = prompt.payments.iter().map(|b| b.label.as_str()).collect();
    assert!(payments.contains(&"Dé volé : bleu"), "{payments:?}");
}

// Spec 060 C30 — Interface : Préparation, la fenêtre des cartes du deck.
#[test]
fn preparation_lists_deck_cards() {
    let catalog = french();
    let events = taken(TimeOfDay::Night, Tactic::Preparation);
    let state = state_of(&events);
    let deck = view(&state, SOLO).preparation_choice;
    assert!(!deck.is_empty());
    let window = screen_of(&events, &catalog)
        .preparation
        .expect("la fenêtre de Préparation");
    assert_eq!(
        window.title,
        "Préparation : choisissez la carte à prendre en main"
    );
    // Une carte par sorte : deux exemplaires d'une carte du deck se valent (actions légales).
    let offered = legal_actions(&state, SOLO)
        .iter()
        .filter(|action| matches!(action, Action::PrepareCard { .. }))
        .count();
    assert!(offered < deck.len());
    assert_eq!(window.cards.len(), offered);
    assert_eq!(
        window.cards[0].command,
        play(&events, &Action::PrepareCard { card: deck[0] })
    );
    assert_eq!(
        window.cards[0].label,
        catalog.text(deck[0].card().label_key())
    );
}

// Spec 060 C20, C35 : Au bon moment, son bouton, puis le rappel et la tactique retournée.
#[test]
fn right_moment_button_then_reminder() {
    let catalog = french();
    let events = taken(TimeOfDay::Day, Tactic::TheRightMoment);
    let line = screen_of(&events, &catalog).tactic.expect("la tactique");
    assert_eq!(
        labels(&line.buttons),
        [(
            "Jouer un tour de plus",
            play(&events, &Action::AnnounceRightMoment)
        )]
    );
    let events = then(events, Action::AnnounceRightMoment);
    let line = screen_of(&events, &catalog).tactic.expect("la tactique");
    assert!(line.buttons.is_empty());
    assert_eq!(line.flipped.as_deref(), Some("Retournée : a servi"));
    assert_eq!(
        line.notes,
        ["Au bon moment : un tour de plus après celui-ci"]
    );
}

/// Longue Nuit prise ; deck de 2, une carte en défausse ; une carte jouée de côté, puis la fin du
/// tour qui défausse 3 cartes : la pioche s'arrête, deck épuisé.
fn long_night_paused() -> Vec<Event> {
    let deck = [card(Swiftness, 1), card(Swiftness, 2)];
    let mut drawn = tactic_hand();
    drawn.push(card(Threaten, 1));
    let mut events = decor_with(TimeOfDay::Night, &drawn, Some(&deck));
    events.push(Event::CardDiscarded {
        player: SOLO,
        card: card(Threaten, 1),
    });
    let events = then(events, choose(Tactic::LongNight));
    let events = then(events, sideways(card(March, 1)));
    let state = state_of(&events);
    let end = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| {
            matches!(action, Action::EndTurn { discard, .. }
                if *discard == [card(March, 2), card(Stamina, 1), card(Rage, 1)])
        })
        .expect("la fin du tour qui défausse 3 cartes");
    then(events, end)
}

// Spec 060 C24 — Q5 : la pioche arrêtée, la fenêtre de Longue Nuit et ses deux boutons.
#[test]
fn long_night_window_during_the_draw() {
    let catalog = french();
    let events = long_night_paused();
    let decision = screen_of(&events, &catalog)
        .tactic_decision
        .expect("la fenêtre de Longue Nuit");
    assert_eq!(
        decision.text,
        "Deck épuisé : utiliser Longue Nuit pour continuer la pioche ?"
    );
    assert_eq!(
        purchase_labels(&decision.buttons),
        [
            (
                "Remettre 3 cartes dans le deck",
                Some(play(&events, &Action::UseLongNight))
            ),
            (
                "Non, arrêter la pioche",
                Some(play(&events, &Action::DeclineLongNight))
            ),
        ]
    );
}

// Spec 060 C23 : pendant le tour, deck vide, le bouton de Longue Nuit dans le panneau.
#[test]
fn long_night_button_with_an_empty_deck() {
    let catalog = french();
    let events = then(long_night_paused(), Action::DeclineLongNight);
    let screen = screen_of(&events, &catalog);
    assert_eq!(screen.tactic_decision, None);
    let line = screen.tactic.expect("la tactique");
    assert_eq!(
        labels(&line.buttons),
        [(
            "Remettre 3 cartes dans le deck",
            play(&events, &Action::UseLongNight)
        )]
    );
}

/// Nuit, la Source or, noir, rouge, bleu ; Appel du mana pris.
fn mana_search() -> Vec<Event> {
    let mut events = decor(TimeOfDay::Night);
    events.push(Event::SourceRolled {
        dice: vec![
            ManaColor::Gold,
            ManaColor::Black,
            ManaColor::Basic(BasicColor::Red),
            ManaColor::Basic(BasicColor::Blue),
        ],
    });
    then(events, choose(Tactic::ManaSearch))
}

// Spec 060 C26 — Interface : Appel du mana, le bouton ouvre le choix des dés.
#[test]
fn mana_search_button_opens_the_dice_choice() {
    let catalog = french();
    let events = mana_search();
    let screen = screen_of(&events, &catalog);
    assert_eq!(screen.tactic_prompt, None);
    let line = screen.tactic.expect("la tactique");
    assert_eq!(
        labels(&line.buttons),
        [("Relancer des dés", Command::OpenManaSearch)]
    );
    let mut session = session_of(events);
    session.command(Command::OpenManaSearch);
    let prompt = session
        .screen(&catalog)
        .tactic_prompt
        .expect("le bandeau d'Appel du mana");
    assert_eq!(
        prompt.text,
        "Appel du mana : choisissez 1 ou 2 dés (or d'abord)"
    );
    assert_eq!(
        labels(&prompt.buttons),
        [("Annuler", Command::CancelChoice)]
    );
    session.shortcut(Shortcut::Cancel);
    assert_eq!(session.screen(&catalog).tactic_prompt, None);
}

// Spec 060 C26, C27 — Compendium, *Mana Search* S1 : le dé or d'abord ; cochés, le bouton relance.
#[test]
fn mana_search_checks_gold_dice_first() {
    let catalog = french();
    let events = mana_search();
    let screen = screen_in(
        &events,
        &ScreenMode::PickingTacticDice(Vec::new()),
        &catalog,
    );
    let all: Vec<(ManaColor, Option<Command>)> = screen
        .source
        .dice
        .iter()
        .chain(&screen.source.depleted)
        .map(|die| (die.color, die.pick))
        .collect();
    assert!(all.contains(&(ManaColor::Gold, Some(Command::ToggleTacticDie(0)))));
    assert!(all.contains(&(ManaColor::Basic(BasicColor::Red), None)));
    let screen = screen_in(&events, &ScreenMode::PickingTacticDice(vec![0]), &catalog);
    let gold = screen
        .source
        .dice
        .iter()
        .chain(&screen.source.depleted)
        .find(|die| die.color == ManaColor::Gold)
        .expect("le dé or");
    assert!(gold.checked);
    assert_eq!(
        labels(&screen.tactic_prompt.expect("bandeau").buttons),
        [
            (
                "Relancer 1 dé(s)",
                play(&events, &Action::ManaSearch { dice: vec![0] })
            ),
            ("Annuler", Command::CancelChoice),
        ]
    );
}

// Spec 060 C28 — Interface : Méditation nocturne, le bouton, puis les cartes cochées et le bouton
// qui les remélange.
#[test]
fn midnight_meditation_checks_hand_cards() {
    let catalog = french();
    let events = taken(TimeOfDay::Night, Tactic::MidnightMeditation);
    let screen = screen_of(&events, &catalog);
    assert!(screen.hand.iter().all(|card| card.discard_toggle.is_none()));
    let line = screen.tactic.expect("la tactique");
    assert_eq!(
        labels(&line.buttons),
        [("Remélanger des cartes", Command::OpenMidnightMeditation)]
    );
    let mode = ScreenMode::PickingTacticCards(vec![card(Rage, 1)]);
    let screen = screen_in(&events, &mode, &catalog);
    let prompt = screen.tactic_prompt.expect("le bandeau");
    assert_eq!(prompt.text, "Méditation nocturne : cochez jusqu'à 5 cartes");
    let wanted = Action::MidnightMeditation {
        cards: vec![card(Rage, 1)],
    };
    assert_eq!(
        labels(&prompt.buttons),
        [
            ("Remélanger et piocher 1 carte(s)", play(&events, &wanted)),
            ("Annuler", Command::CancelChoice),
        ]
    );
    let checked: Vec<bool> = screen
        .hand
        .iter()
        .map(|card| card.discard_toggle.as_ref().is_some_and(|t| t.checked))
        .collect();
    assert_eq!(checked, [false, false, false, true, false]);
}

// Spec 060 C31, C32 — Interface : Provisions, la fenêtre d'avant le tour ; puis le nombre de cartes
// dessous dans le panneau.
#[test]
fn sparing_power_window_before_the_turn() {
    let catalog = french();
    let events = taken(TimeOfDay::Night, Tactic::SparingPower);
    let screen = screen_of(&events, &catalog);
    let decision = screen.tactic_decision.expect("la fenêtre de Provisions");
    assert_eq!(
        decision.text,
        "Provisions (0 carte(s) dessous) : avant ce tour"
    );
    let store = Action::SparingPower { take: false };
    let take = Action::SparingPower { take: true };
    assert_eq!(
        purchase_labels(&decision.buttons),
        [
            (
                "Poser la carte du dessus du deck",
                Some(play(&events, &store))
            ),
            (
                "Retourner et prendre 0 carte(s)",
                Some(play(&events, &take))
            ),
        ]
    );
    let events = then(events, store);
    let screen = screen_of(&events, &catalog);
    assert_eq!(screen.tactic_decision, None);
    assert_eq!(
        screen.tactic.expect("la tactique").notes,
        ["Dessous : 1 carte(s)"]
    );
}

// Spec 060 C33 — Interface : deck vide, « Poser » est grisé.
#[test]
fn sparing_power_store_greyed_with_empty_deck() {
    let catalog = french();
    let events = then(
        decor_with(TimeOfDay::Night, &tactic_hand(), Some(&[])),
        choose(Tactic::SparingPower),
    );
    let decision = screen_of(&events, &catalog)
        .tactic_decision
        .expect("la fenêtre de Provisions");
    assert_eq!(decision.buttons[0].command, None);
    assert!(decision.buttons[1].command.is_some());
}

// Spec 060 C35 : sans tactique (bac à sable), rien dans le panneau.
#[test]
fn no_tactic_line_in_the_sandbox() {
    let catalog = french();
    let events = Sandbox {
        figure: Some(PLAIN_W),
        hand: vec![card(Concentration, 1)],
        ..Sandbox::default()
    }
    .events();
    let screen = screen_of(&events, &catalog);
    assert_eq!(screen.tactic, None);
    assert_eq!(screen.tactic_choice, None);
}
