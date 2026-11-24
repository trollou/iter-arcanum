//! Spec 050 côté affichage : fenêtre Offres, choix de l'action avancée, main.

use arcanum_core::{
    Action, CardId, DeedCard, Event, EventLog, GameConfig, GameRunner, GameState, Hero, Offer,
    PlayerId, RngPositions, Scenario, apply, decide, legal_actions, new_game, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, GameSession, ScreenMode, ScreenPhase, Shortcut, game_screen, human_player,
    offers_window, repository_assets_dir,
};

const SOLO: PlayerId = PlayerId::SOLO;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn config() -> GameConfig {
    GameConfig {
        seed: 4,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

fn play(events: &mut Vec<Event>, state: &mut GameState, action: &Action) {
    let produced = decide(state, SOLO, action, &mut config().rngs())
        .unwrap_or_else(|error| panic!("{action:?} : {error:?}"));
    *state = produced.iter().fold(state.clone(), apply);
    events.extend(produced);
}

/// La mise en place du bac à sable ; avec `choosing`, 3 de gloire et le tour fini : le choix de
/// l'action avancée du niveau 2 attend.
fn events(choosing: bool) -> Vec<Event> {
    let config = config();
    let mut events = new_game(&config, &mut config.rngs());
    if choosing {
        events.push(Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame: 3,
        });
        let mut state = events.iter().fold(GameState::default(), apply);
        let sideways = legal_actions(&state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::PlayCard { .. }))
            .expect("une carte jouable");
        play(&mut events, &mut state, &sideways);
        let end_turn = legal_actions(&state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::EndTurn { .. }))
            .expect("finir le tour");
        play(&mut events, &mut state, &end_turn);
        // Spec 051 : la compétence du niveau 2 d'abord (fenêtre du choix de compétence).
        let skill = legal_actions(&state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::ChooseSkill { .. }))
            .expect("une compétence à choisir");
        play(&mut events, &mut state, &skill);
    }
    events
}

fn opened(choosing: bool) -> GameSession {
    let mut log = EventLog::default();
    for event in events(choosing) {
        log.push(0, event);
    }
    let (player, human) = human_player();
    let runner = GameRunner::resume(log, RngPositions::new(), vec![Box::new(player)])
        .expect("journal valide");
    let mut session = GameSession::with_runner(runner, human);
    session.run_frame();
    session
}

// Spec 070 C17 : plus de taille de pioche dans la fenêtre Offres (« Pioche : 25 », « Élite : 20 »).
#[test]
fn offers_window_shows_no_deck_size() {
    let catalog = french();
    let state = events(false).iter().fold(GameState::default(), apply);
    let window = offers_window(&view(&state, SOLO), &[], &catalog);
    let shown = format!("{window:?}");
    assert!(!shown.contains("Pioche"), "{shown}");
    assert!(!shown.contains("Élite :"), "{shown}");
}

// Fenêtre Offres : trois colonnes, noms, effets non codés signalés.
#[test]
fn offers_window_lists_the_three_offers() {
    let catalog = french();
    let state = events(false).iter().fold(GameState::default(), apply);
    let seen = view(&state, SOLO);
    let window = offers_window(&seen, &[], &catalog);
    assert_eq!(window.title, "Offres");
    assert!(window.closable);
    let titles: Vec<&str> = window.columns.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(titles, ["Actions avancées", "Sorts", "Unités"]);
    for column in &window.columns {
        assert_eq!(column.cards.len(), 3);
        assert!(column.cards.iter().all(|card| card.take.is_none()));
    }
    // Spec 057 : tous les sorts sont codés, aucun effet ne le dit plus.
    let spell = &window.columns[1].cards[0];
    assert!(
        !spell
            .lines
            .iter()
            .any(|line| line.contains("pas encore codé"))
    );
    // Une unité dit si elle est régulière, son coût et ses lieux de recrutement.
    let unit = &window.columns[2].cards[0];
    assert!(unit.lines.iter().any(|line| line.contains("Régulière")));
    assert!(unit.lines.iter().any(|line| line.contains("Influence")));
}

// Choix de l'action avancée : titre du niveau, un bouton « Prendre » par carte, pas de fermeture.
#[test]
fn choice_window_offers_take_buttons() {
    let catalog = french();
    let session = opened(true);
    let window = session.offers_window(&catalog).expect("ouverte d'office");
    assert_eq!(window.title, "Niveau 2 : choisissez une action avancée");
    assert!(!window.closable);
    let takes: Vec<Command> = window.columns[0]
        .cards
        .iter()
        .filter_map(|card| card.take.as_ref().map(|button| button.command))
        .collect();
    assert_eq!(
        takes,
        [Command::Play(0), Command::Play(1), Command::Play(2)]
    );
    assert!(
        window.columns[1]
            .cards
            .iter()
            .all(|card| card.take.is_none())
    );
}

// Raccourcis : O ouvre et ferme, Échap ferme ; pendant le choix, ni l'un ni l'autre, 2 prend la
// deuxième carte.
#[test]
fn offers_shortcuts() {
    let catalog = french();
    let mut session = opened(false);
    assert!(session.offers_window(&catalog).is_none());
    session.shortcut(Shortcut::Offers);
    assert!(session.offers_window(&catalog).is_some());
    session.shortcut(Shortcut::Offers);
    assert!(session.offers_window(&catalog).is_none());
    session.command(Command::OpenOffers);
    session.shortcut(Shortcut::Cancel);
    assert!(session.offers_window(&catalog).is_none());

    let mut session = opened(true);
    let second = session.view().offers[&Offer::AdvancedActions][1]
        .deed()
        .expect("action avancée");
    session.shortcut(Shortcut::Offers);
    session.shortcut(Shortcut::Cancel);
    assert!(session.offers_window(&catalog).is_some());
    session.shortcut(Shortcut::Take(2));
    session.run_frame();
    assert!(session.view().hand.contains(&second));
    assert!(session.offers_window(&catalog).is_none());
}

// Main : une action avancée a la couleur de sa carte ; un effet non codé dit son texte.
#[test]
fn hand_shows_advanced_action_color_and_artifact_text() {
    let catalog = french();
    // Spec 056 : toutes les actions avancées sont codées ; spec 057 : tous les sorts aussi ; spec
    // 055 : les artefacts aussi, sans couleur, leur texte écrit (test modifié, C1).
    let agility = CardId::new(DeedCard::Artifact(arcanum_core::Artifact::ALL[0]), 1);
    let mut events = events(false);
    events.push(Event::DeckShuffled {
        player: SOLO,
        order: vec![agility],
    });
    events.push(Event::CardDrawn {
        player: SOLO,
        card: agility,
    });
    let state = events.iter().fold(GameState::default(), apply);
    let legal = legal_actions(&state, SOLO);
    let screen = game_screen(
        &view(&state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        &catalog,
    );
    let card = &screen.hand[0];
    assert_eq!(card.color, None);
    assert!(!card.effect.contains("pas encore codé"), "{}", card.effect);
    assert!(card.plays.is_empty() && card.strong_plays.is_empty());
    assert!(!card.sideways.is_empty());
    assert_eq!(screen.bottom_buttons[0].label, "Offres…");
}

// Spec 071, C22 : « Prendre » de la 2e action avancée dit « Raccourci : 2 ».
#[test]
fn numbered_choices_show_shortcut() {
    let catalog = french();
    let session = opened(true);
    let window = session.offers_window(&catalog).expect("ouverte d'office");
    let keys: Vec<Option<String>> = window.columns[0]
        .cards
        .iter()
        .filter_map(|card| card.take.as_ref().map(|button| button.shortcut.clone()))
        .collect();
    assert_eq!(keys, [Some("1".into()), Some("2".into()), Some("3".into())]);
    assert_eq!(window.close.shortcut, None);
}
