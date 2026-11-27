//! Scénarios de la spec 045 côté affichage (Ruines) : C31 à C40.

use arcanum_core::{
    Action, BasicColor, CardId, CardPlay, Effect, EnemyColor, Event, GameState, Hex, ManaColor,
    RuinToken, SidewaysEffect, TimeOfDay, Unit, UnitCard, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, Button, Command, GameScreen, OverlayKind, RuinPip, ScreenMode,
    ScreenPhase, Shortcut, board_screen, figure_image, flag_image, game_screen, hex_center,
    repository_assets_dir, token_names,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{LabelAndCommand, board_input, labels_and_commands, session_of};

use arcanum_core::BasicAction::*;

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn screen_in(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        mode,
        catalog,
    )
}

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    screen_in(state, &ScreenMode::Normal, catalog)
}

/// L'index de cette action parmi les actions légales.
fn index_of(state: &GameState, wanted: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"))
}

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

fn camera() -> BoardCamera {
    BoardCamera {
        center: Pos2::ZERO,
        zoom: 30.0,
    }
}

fn board(state: &GameState, cursor: Option<Hex>, catalog: &Catalog) -> BoardScreen {
    let seen = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    let camera = camera();
    board_screen(
        &board_input(
            &seen,
            &legal,
            camera,
            area(),
            cursor.map(|hex| camera.to_screen(area(), hex_center(hex))),
        ),
        catalog,
    )
}

fn hover(state: &GameState, hex: Hex, catalog: &Catalog) -> Vec<String> {
    board(state, Some(hex), catalog).hover_lines
}

/// Les événements du décor suivis de ceux de ces actions, jouées dans l'ordre.
fn events_after(start: Vec<Event>, actions: &[Action]) -> Vec<Event> {
    let mut events = start;
    let mut state = events.iter().fold(GameState::default(), apply);
    for action in actions {
        let produced = events_of(&state, action);
        state = produced.iter().fold(state, apply);
        events.extend(produced);
    }
    events
}

fn blue() -> ManaColor {
    ManaColor::Basic(BasicColor::Blue)
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn units(count: usize) -> Vec<UnitCard> {
    [Unit::Foresters, Unit::Herbalists, Unit::NorthernMonks][..count]
        .iter()
        .map(|&unit| UnitCard::new(unit, 1))
        .collect()
}

fn artifacts(count: usize) -> Vec<CardId> {
    arcanum_core::Artifact::ALL[..count]
        .iter()
        .map(|&card| artifact(card))
        .collect()
}

/// Les événements de l'autel bleu, figurine dessus, de jour, avec ces dés dans la Source,
/// `crystals` cristaux bleus et ces jetons de mana.
fn altar_events(dice: &[ManaColor], crystals: u8, tokens: &[ManaColor]) -> Vec<Event> {
    let mut events = Ruins {
        dice: dice.to_vec(),
        ..Ruins::default()
    }
    .events();
    events.extend((0..crystals).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    events.extend(tokens.iter().map(|&color| Event::ManaTokenGained {
        player: SOLO,
        color,
    }));
    events
}

/// L'autel bleu payable : un dé bleu dans la Source, un cristal bleu, un jeton bleu ; une Marche
/// jouée (2 mouvement).
fn payable_altar_events() -> Vec<Event> {
    events_after(
        altar_events(&Sandbox::default().dice, 1, &[blue()]),
        &[basic(card(March, 1))],
    )
}

fn state_of(events: &[Event]) -> GameState {
    events.iter().fold(GameState::default(), apply)
}

/// Les offrandes parmi les actions légales.
fn tributes(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::PayAltarTribute { .. }))
        .collect()
}

/// La ruine conquise ce tour (récompense en attente), figurine dessus.
fn conquered(ruins: Ruins) -> GameState {
    Ruins {
        conquered: true,
        ..ruins
    }
    .state()
}

/// La première fin de tour proposée.
fn end_turn(state: &GameState) -> Action {
    legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("fin du tour proposée")
}

/// Une carte de la main jouée de côté pour `effect`, par un événement.
fn played(state: &GameState, card: CardId, effect: Effect) -> GameState {
    apply(
        state.clone(),
        &Event::CardPlayed {
            player: SOLO,
            card,
            play: CardPlay::Sideways(SidewaysEffect::Move),
            effect,
        },
    )
}

/// Aucune ligne de l'écran ne dit « Bouclier » (spec 045, Q6 : à l'écran, c'est le drapeau).
fn no_shield_word(lines: &[String]) {
    assert!(
        lines.iter().all(|line| !line.contains("Bouclier")),
        "{lines:?}"
    );
}

// --- C31 : jeton de ruines sur le plateau ---

// C31 — Carte *Ancient Ruins* ; Q5 : le jeton dessiné par le code, pastilles de l'autel ou des
// piles d'ennemis, face cachée unie ; les ennemis restés comme au Sombre Terrier.
#[test]
fn board_draws_ruin_tokens() {
    let catalog = french();
    let marks_at = |ruins: Ruins| {
        board(&ruins.state(), None, &catalog)
            .ruin_tokens
            .into_iter()
            .filter(|mark| mark.hex == RUINS)
            .collect::<Vec<_>>()
    };

    let altar = marks_at(Ruins::default());
    assert_eq!(altar.len(), 1);
    assert!(altar[0].face_up);
    assert_eq!(altar[0].pips, [RuinPip::Mana(BasicColor::Blue); 3]);
    assert!(altar[0].radius > 0.0);

    let enemies = marks_at(Ruins {
        token: RuinToken::GreyViolet,
        ..Ruins::default()
    });
    assert_eq!(
        enemies[0].pips,
        [
            RuinPip::Enemy(EnemyColor::Grey),
            RuinPip::Enemy(EnemyColor::Violet)
        ]
    );

    let hidden = marks_at(Ruins {
        token: RuinToken::GreyViolet,
        face_up: false,
        time: TimeOfDay::Night,
        ..Ruins::default()
    });
    assert_eq!(hidden.len(), 1);
    assert!(!hidden[0].face_up);
    assert!(hidden[0].pips.is_empty());

    // Le jeton resté sur la ruine, à la place du symbole imprimé ; le jeton de ruines reste.
    let waiting = Ruins {
        token: RuinToken::GreyViolet,
        placed: vec![(monks(), RUINS)],
        ..Ruins::default()
    };
    let screen = board(&waiting.state(), None, &catalog);
    assert!(screen.overlays.iter().any(|overlay| overlay.hex == RUINS
        && overlay.kind == OverlayKind::SiteEnemy
        && overlay.image == figure_image(monks().enemy())));
    assert_eq!(marks_at(waiting).len(), 1);

    // Conquise : plus de jeton de ruines.
    assert!(
        marks_at(Ruins {
            conquered: true,
            ..Ruins::default()
        })
        .is_empty()
    );
}

// --- C32 : infobulle ---

// C32 — Carte *Ancient Ruins* : autel, ennemis et récompense, ennemis restés, ou face cachée.
#[test]
fn hover_describes_ruins() {
    let catalog = french();
    let lines = |ruins: Ruins| hover(&ruins.state(), RUINS, &catalog);

    let altar = lines(Ruins::default());
    assert!(altar.contains(&"Ruines".to_string()), "{altar:?}");
    assert!(altar.contains(&"Autel : offrande de 3 mana bleu pour 7 Gloire".to_string()));
    no_shield_word(&altar);

    let unit = lines(Ruins {
        token: RuinToken::GreyViolet,
        ..Ruins::default()
    });
    assert!(
        unit.contains(
            &"Ennemis : gardes de forteresse, tour de mage ; récompense : une unité de l'offre"
                .to_string()
        ),
        "{unit:?}"
    );
    let two = lines(Ruins {
        token: RuinToken::BrownRed,
        ..Ruins::default()
    });
    assert!(
        two.contains(&"Ennemis : donjon, draconien ; récompense : 2 artefact(s)".to_string()),
        "{two:?}"
    );
    let mixed = lines(Ruins {
        token: RuinToken::BrownViolet,
        ..Ruins::default()
    });
    assert!(
        mixed.contains(
            &"Ennemis : donjon, tour de mage ; récompense : un sort, un cristal de chaque couleur"
                .to_string()
        ),
        "{mixed:?}"
    );

    let waiting = lines(Ruins {
        token: RuinToken::GreyViolet,
        placed: vec![(monks(), RUINS)],
        ..Ruins::default()
    });
    let names = token_names(vec![monks()], &catalog);
    assert!(
        waiting.contains(&format!("Vous y attendent : {names}")),
        "{waiting:?}"
    );

    let hidden = lines(Ruins {
        token: RuinToken::GreyViolet,
        face_up: false,
        time: TimeOfDay::Night,
        ..Ruins::default()
    });
    assert!(hidden.contains(
        &"Jeton de ruines face cachée (révélé au prochain Jour, ou en y entrant)".to_string()
    ));
    assert!(!hidden.iter().any(|line| line.starts_with("Ennemis")));
}

// --- C33, C34 : offrande ---

// C33 — Carte *Ancient Ruins*, *Altar* ; Q1 : le bouton de l'offrande et les points perdus ;
// grisé sans 3 mana ; absent ailleurs.
#[test]
fn panel_offers_altar_tribute() {
    let catalog = french();
    let screen = screen_of(&state_of(&payable_altar_events()), &catalog);
    let altar = screen.altar.as_ref().expect("bouton de l'autel");
    assert_eq!(
        altar.label_and_command(),
        ("Faire une offrande à l'autel", &Some(Command::OpenAltar))
    );
    assert_eq!(altar.tooltip, None);
    // Spec 071, C21 : E ouvre la fenêtre.
    assert_eq!(altar.shortcut.as_deref(), Some("E"));
    assert_eq!(
        screen.altar_warning.as_deref(),
        Some("Entrer fait perdre : 2 mouvement, 0 influence")
    );
    assert_eq!(screen.adventure, None);

    let red = ManaColor::Basic(BasicColor::Red);
    let poor = state_of(&altar_events(&[red, red, red], 1, &[]));
    assert!(tributes(&poor).is_empty());
    let screen = screen_of(&poor, &catalog);
    let button = screen.altar.expect("bouton grisé");
    assert_eq!(button.command, None);
    assert_eq!(screen.altar_warning.as_deref(), Some("Il faut 3 mana bleu"));

    let elsewhere = Ruins {
        figure: NEAR_RUINS,
        ..Ruins::default()
    }
    .state();
    assert_eq!(screen_of(&elsewhere, &catalog).altar, None);
}

// C33 bis (issue privée 141) — Pas de bouton grisé quand l'action du tour n'est pas possible : main de
// Blessures sans carte jouée (Compendium, *Resting* S4), même avec 3 cristaux bleus.
#[test]
fn no_altar_button_once_turn_action_unavailable() {
    let catalog = french();
    let mut events = Ruins {
        hand: vec![wound(1), wound(2)],
        ..Ruins::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    let wounded = state_of(&events);
    assert!(tributes(&wounded).is_empty());
    let screen = screen_of(&wounded, &catalog);
    assert_eq!(screen.altar, None);
    assert_eq!(screen.altar_warning, None);
}

// C34 — Q1 : la fenêtre de l'offrande, une ligne par paiement légal ; un clic paie ; 1 à 9 ;
// Échap ou « Annuler » ferme.
#[test]
fn altar_payment_window() {
    let catalog = french();
    let state = state_of(&payable_altar_events());
    assert_eq!(screen_of(&state, &catalog).altar_window, None);
    let window = screen_in(&state, &ScreenMode::PayingTribute, &catalog)
        .altar_window
        .expect("fenêtre de l'offrande");
    assert_eq!(window.title, "Offrande : 3 mana bleu, 7 Gloire");
    let tributes = tributes(&state);
    assert_eq!(window.payments.len(), tributes.len());
    for (button, action) in window.payments.iter().zip(&tributes) {
        assert_eq!(button.command, Command::Play(index_of(&state, action)));
    }
    let labels: Vec<&str> = window
        .payments
        .iter()
        .map(|button| button.label.as_str())
        .collect();
    assert!(
        labels.contains(&"dé bleu, cristal bleu, jeton bleu"),
        "{labels:?}"
    );
    assert_eq!(window.note, "La gloire est gagnée aussitôt.");
    assert_eq!(
        window.cancel.label_and_command(),
        ("Annuler", &Command::CancelChoice)
    );
    // Spec 071, C23 : Échap ferme la fenêtre.
    assert_eq!(window.cancel.shortcut.as_deref(), Some("Échap"));

    // E ouvre la fenêtre, Échap la ferme, 1 paie.
    let mut session = session_of(payable_altar_events());
    session.shortcut(Shortcut::EnterSite);
    assert!(session.screen(&catalog).altar_window.is_some());
    session.shortcut(Shortcut::Cancel);
    assert!(session.screen(&catalog).altar_window.is_none());
    session.command(Command::OpenAltar);
    assert!(session.screen(&catalog).altar_window.is_some());
    session.shortcut(Shortcut::Take(1));
    session.run_frame();
    assert_eq!(session.view().fame, 7);
    assert!(session.view().shields.contains_key(&RUINS));
    assert!(session.screen(&catalog).altar_window.is_none());
}

// --- C35 : entrer ---

// C35 — Carte *Ancient Ruins*, *Enemies* : le bouton de la spec 044, la confirmation donne les
// couleurs à tirer (ou les ennemis restés) et la récompense du jeton.
#[test]
fn ruins_enter_confirmation() {
    let catalog = french();
    let fresh = Ruins {
        token: RuinToken::GreyViolet,
        ..Ruins::default()
    }
    .state();
    let screen = screen_of(&fresh, &catalog);
    assert_eq!(
        screen.adventure.map(|button| button.label),
        Some("Entrer dans : Ruines".to_string())
    );
    assert_eq!(screen.altar, None);
    let confirm = screen_in(&fresh, &ScreenMode::ConfirmingAdventure, &catalog)
        .adventure_confirm
        .expect("confirmation");
    assert_eq!(
        confirm.text,
        "Combat contre des ennemis tirés au hasard : gardes de forteresse, tour de mage.\n\
         Récompense si vous gagnez : une unité de l'offre. Pas de retour en arrière."
    );
    assert_eq!(
        labels_and_commands(&confirm.buttons),
        [
            (
                "Entrer",
                &Command::Play(index_of(&fresh, &Action::EnterAdventureSite))
            ),
            ("Annuler", &Command::CancelChoice),
        ]
    );
    // Spec 071, C23 : Échap annule.
    assert_eq!(confirm.buttons[1].shortcut.as_deref(), Some("Échap"));

    let again = Ruins {
        token: RuinToken::GreyViolet,
        placed: vec![(monks(), RUINS)],
        ..Ruins::default()
    }
    .state();
    let text = screen_in(&again, &ScreenMode::ConfirmingAdventure, &catalog)
        .adventure_confirm
        .expect("confirmation")
        .text;
    let names = token_names(vec![monks()], &catalog);
    assert_eq!(
        text,
        format!("Combat contre : {names}.\nRécompense si vous gagnez : une unité de l'offre.")
    );
}

// --- C36 : drapeau ---

// C36 — Q6 : une ruine conquise montre le drapeau du héros, et se dit « conquise ».
#[test]
fn fulfilled_ruins_show_a_flag() {
    let catalog = french();
    let state = conquered(Ruins::default());
    let hero = view(&state, SOLO).hero.expect("un héros");
    let screen = board(&state, Some(RUINS), &catalog);
    assert!(screen.overlays.iter().any(|overlay| overlay.hex == RUINS
        && overlay.kind == OverlayKind::Flag
        && overlay.image == flag_image(hero)));
    assert!(
        screen.hover_lines.contains(&"Ruines conquises".to_string()),
        "{:?}",
        screen.hover_lines
    );
    assert!(
        !screen
            .hover_lines
            .iter()
            .any(|line| line.starts_with("Autel"))
    );
    no_shield_word(&screen.hover_lines);
}

// --- C37 : récompenses en attente ---

// C37 — MKUE p. 11, 5 : la récompense du jeton sous les points, à la place des lignes séparées.
#[test]
fn panel_announces_ruin_rewards() {
    let catalog = french();
    let pending = |token: RuinToken| {
        let screen = screen_of(
            &conquered(Ruins {
                token,
                ..Ruins::default()
            }),
            &catalog,
        );
        assert_eq!(screen.reward_pending, None, "{token:?}");
        assert_eq!(screen.spell_reward_pending, None, "{token:?}");
        screen.ruin_reward_pending
    };
    assert_eq!(
        pending(RuinToken::GreenRed).as_deref(),
        Some("Récompense en fin de tour : un artefact, une action avancée")
    );
    assert_eq!(
        pending(RuinToken::BrownRed).as_deref(),
        Some("Récompense en fin de tour : 2 artefact(s)")
    );
    assert_eq!(
        pending(RuinToken::BrownViolet).as_deref(),
        Some("Récompense en fin de tour : un sort, un cristal de chaque couleur")
    );
    assert_eq!(
        pending(RuinToken::GreyViolet).as_deref(),
        Some("Récompense en fin de tour : une unité de l'offre")
    );
    // L'autel ne laisse rien en attente : sa gloire est gagnée aussitôt.
    assert_eq!(pending(RuinToken::BlueAltar), None);
    // Ailleurs, rien.
    let screen = screen_of(&Ruins::default().state(), &catalog);
    assert_eq!(screen.ruin_reward_pending, None);
}

// --- C38 : deux artefacts parmi trois ---

// C38 — MKUE p. 11, 5.b : trois artefacts tirés, deux pris un à un ; 1 à 3.
#[test]
fn artifact_window_keeps_two() {
    let catalog = french();
    let drawn = artifacts(3);
    let state = conquered(Ruins {
        token: RuinToken::BrownRed,
        artifacts: drawn.clone(),
        ..Ruins::default()
    });
    let choosing = done(&state, end_turn(&state));
    let window = screen_of(&choosing, &catalog)
        .artifact_choice
        .expect("choix d'artefact");
    assert_eq!(window.cards.len(), 3);
    assert!(window.cards.iter().all(|card| card.take.is_some()));

    let kept = done(&choosing, Action::TakeArtifact { card: drawn[0] });
    let window = screen_of(&kept, &catalog)
        .artifact_choice
        .expect("le second choix reste");
    assert_eq!(window.cards.len(), 2);
    assert!(window.cards.iter().all(|card| card.take.is_some()));

    // 3 prend le troisième artefact tiré.
    let mut session = session_of(events_after(
        Ruins {
            token: RuinToken::BrownRed,
            artifacts: drawn.clone(),
            conquered: true,
            ..Ruins::default()
        }
        .events(),
        &[end_turn(&state)],
    ));
    session.shortcut(Shortcut::Take(3));
    session.run_frame();
    assert_eq!(session.view().drawn_artifacts, drawn[..2]);
}

// --- C39 : récompense unité ---

// C39 — MKUE p. 11, 5.d ; Q4. Spec #20, C12 (test modifié : la fenêtre de choix au lieu de la
// fenêtre Offres) : les unités de l'offre seulement, « Prendre » sur chacune ; 1 à 3.
#[test]
fn unit_reward_in_choice_window() {
    let catalog = french();
    let offer = units(2);
    let start = Ruins {
        token: RuinToken::GreyViolet,
        conquered: true,
        offer: offer.clone(),
        ..Ruins::default()
    };
    let state = start.state();
    let events = events_after(start.events(), &[end_turn(&state)]);
    let choosing = state_of(&events);
    let mut session = session_of(events);
    assert!(session.offers_window(&catalog).is_none());
    let window = session
        .screen(&catalog)
        .unit_reward
        .expect("fenêtre de l'unité");
    assert_eq!(window.title, "Récompense : prenez une unité de l'offre");
    assert_eq!(window.close, None);
    assert_eq!(window.cards.len(), offer.len());
    for (place, (card, &unit)) in window.cards.iter().zip(&offer).enumerate() {
        assert_eq!(card.name, catalog.text(unit.unit.label_key()));
        let wanted = Action::TakeUnitReward {
            unit,
            disband: None,
        };
        let take = card.take.as_ref().expect("bouton Prendre");
        assert_eq!(
            take.label_and_command(),
            ("Prendre", &Command::Play(index_of(&choosing, &wanted)))
        );
        assert_eq!(take.shortcut, Some((place + 1).to_string()));
    }
    // Échap ne ferme pas la fenêtre ; 2 prend la deuxième unité.
    session.shortcut(Shortcut::Cancel);
    assert!(session.screen(&catalog).unit_reward.is_some());
    session.shortcut(Shortcut::Take(2));
    session.run_frame();
    let seen = session.view();
    assert_eq!(seen.units.len(), 1);
    assert_eq!(seen.units[0].card, offer[1]);
    assert!(session.screen(&catalog).unit_reward.is_none());
    assert!(session.offers_window(&catalog).is_none());
}

// C39 — MKUE p. 11, 5.d ; Q4. Spec #20, C12 (test modifié : la fenêtre de choix) : sans jeton
// libre, « Prendre » ouvre le choix de l'unité à renvoyer, et « Renoncer » est proposé.
#[test]
fn unit_reward_window_without_free_token() {
    let catalog = french();
    let offer = units(2);
    let start = Ruins {
        token: RuinToken::GreyViolet,
        conquered: true,
        owned: vec![peasants()],
        offer: offer.clone(),
        ..Ruins::default()
    };
    let state = start.state();
    let events = events_after(start.events(), &[end_turn(&state)]);
    let choosing = state_of(&events);
    let mut session = session_of(events);
    let window = session
        .screen(&catalog)
        .unit_reward
        .expect("fenêtre de l'unité");
    let take = window.cards[0].take.as_ref().expect("bouton Prendre");
    assert_eq!(
        take.label_and_command(),
        ("Prendre", &Command::ChooseDisband(offer[0]))
    );
    assert_eq!(take.shortcut.as_deref(), Some("1"));
    assert_eq!(
        window.close.as_ref().map(Button::label_and_command),
        Some((
            "Renoncer",
            &Command::Play(index_of(&choosing, &Action::DeclineUnitReward))
        ))
    );

    session.command(Command::ChooseDisband(offer[0]));
    let screen = session.screen(&catalog);
    assert!(screen.disband_prompt.is_some());
    let wanted = Action::TakeUnitReward {
        unit: offer[0],
        disband: Some(peasants()),
    };
    assert_eq!(
        screen.units.units[0]
            .disband
            .as_ref()
            .map(Button::label_and_command),
        Some(("Renvoyer", &Command::Play(index_of(&choosing, &wanted))))
    );
    // Échap revient au choix de l'unité, sans rien jouer.
    session.shortcut(Shortcut::Cancel);
    assert!(session.screen(&catalog).disband_prompt.is_none());
    assert!(session.screen(&catalog).unit_reward.is_some());
    session.command(Command::ChooseDisband(offer[0]));
    session.command(Command::Play(index_of(&choosing, &wanted)));
    session.run_frame();
    assert_eq!(session.view().units[0].card, offer[0]);
}

// Spec #20, C13 — spec 045 : une action avancée de monastère de l'offre d'unités n'est pas une
// unité de récompense.
#[test]
fn unit_reward_leaves_out_monastery_card() {
    let catalog = french();
    let offer = units(2);
    let start = Ruins {
        token: RuinToken::GreyViolet,
        conquered: true,
        offer: offer.clone(),
        advanced: advanced_actions(3),
        monastery_advanced: vec![advanced(arcanum_core::AdvancedAction::IceShield)],
        ..Ruins::default()
    };
    let state = start.state();
    let events = events_after(start.events(), &[end_turn(&state)]);
    let window = session_of(events)
        .screen(&catalog)
        .unit_reward
        .expect("fenêtre de l'unité");
    let names: Vec<&str> = window.cards.iter().map(|card| card.name.as_str()).collect();
    let wanted: Vec<String> = offer
        .iter()
        .map(|unit| catalog.text(unit.unit.label_key()))
        .collect();
    assert_eq!(names, wanted);
}

// --- C40 : trajet arrêté ---

// C40 — Compendium, *Movement* S13 ; Q2 : le message du trajet arrêté par une ruine révélée,
// jusqu'à l'action suivante.
#[test]
fn panel_announces_revealed_ruins() {
    let catalog = french();
    let state = Ruins {
        figure: RUINS_GLADE,
        time: TimeOfDay::Night,
        token: RuinToken::GreyViolet,
        face_up: false,
        ..Ruins::default()
    }
    .state();
    let state = played(&state, card(March, 1), Effect::Move(10));
    assert_eq!(screen_of(&state, &catalog).revealed_ruins, None);
    let stopped = done(&state, Action::FollowRoute { to: NEAR_RUINS });
    assert_eq!(view(&stopped, SOLO).figure, Some(RUINS));
    assert_eq!(
        screen_of(&stopped, &catalog).revealed_ruins.as_deref(),
        Some("Ruines révélées : le trajet s'arrête")
    );
    let next = done(&stopped, Action::MoveHero { to: NEAR_RUINS });
    assert_eq!(screen_of(&next, &catalog).revealed_ruins, None);
}

// --- C41 : récompense action avancée (issue privée 140) ---

fn advanced_actions(count: usize) -> Vec<CardId> {
    arcanum_core::AdvancedAction::ALL[..count]
        .iter()
        .map(|&card| advanced(card))
        .collect()
}

// C41 — MKUE p. 11, 5.c ; C24. Spec #20, C11 (test modifié : la fenêtre de choix au lieu de la
// fenêtre Offres) : l'action avancée de la récompense, seulement les cartes de l'offre d'actions
// avancées, sans bouton de fermeture ; elle passe avant le niveau pair gagné le même tour.
#[test]
fn advanced_action_reward_in_choice_window() {
    let catalog = french();
    let offer = advanced_actions(3);
    let drawn = artifacts(2);
    let start = Ruins {
        token: RuinToken::GreenRed,
        conquered: true,
        artifacts: drawn.clone(),
        advanced: offer.clone(),
        monastery_advanced: vec![advanced(arcanum_core::AdvancedAction::IceShield)],
        fame: 3,
        ..Ruins::default()
    };
    let state = start.state();
    let gain = |card: CardId| Action::GainAdvancedAction { card };
    let events = events_after(
        start.events(),
        &[end_turn(&state), Action::TakeArtifact { card: drawn[0] }],
    );
    let choosing = state_of(&events);
    let session = session_of(events.clone());
    assert!(session.offers_window(&catalog).is_none());
    let window = session
        .screen(&catalog)
        .advanced_action_reward
        .expect("fenêtre de la récompense");
    assert_eq!(window.title, "Récompense : prenez une action avancée");
    assert_eq!(window.close, None);
    let shown: Vec<(String, Option<Command>)> = window
        .cards
        .iter()
        .map(|card| {
            (
                card.name.clone(),
                card.take.as_ref().map(|button| button.command),
            )
        })
        .collect();
    let wanted: Vec<(String, Option<Command>)> = offer
        .iter()
        .map(|&card| {
            (
                catalog.text(card.card().label_key()),
                Some(Command::Play(index_of(&choosing, &gain(card)))),
            )
        })
        .collect();
    assert_eq!(shown, wanted);

    // La récompense prise, sa fenêtre se ferme et la fenêtre Offres reste fermée.
    let events = events_after(events, &[gain(offer[0])]);
    let session = session_of(events);
    assert!(session.screen(&catalog).advanced_action_reward.is_none());
    assert!(session.offers_window(&catalog).is_none());

    // Sans niveau pair, le seul choix est celui de la récompense.
    let start = Ruins { fame: 0, ..start };
    let state = start.state();
    let events = events_after(
        start.events(),
        &[end_turn(&state), Action::TakeArtifact { card: drawn[0] }],
    );
    let window = session_of(events)
        .screen(&catalog)
        .advanced_action_reward
        .expect("fenêtre de la récompense");
    assert_eq!(window.title, "Récompense : prenez une action avancée");
}

// Spec #20, C17 — spec #32 : les cartes d'une fenêtre de choix sont celles de la fenêtre Offres, à
// « Prendre » près.
#[test]
fn choice_cards_match_offer_cards() {
    let catalog = french();
    let offer = advanced_actions(3);
    let start = Ruins {
        token: RuinToken::GreenRed,
        conquered: true,
        artifacts: artifacts(2),
        advanced: offer.clone(),
        ..Ruins::default()
    };
    let state = start.state();
    let events = events_after(
        start.events(),
        &[
            end_turn(&state),
            Action::TakeArtifact {
                card: artifacts(2)[0],
            },
        ],
    );
    let choosing = state_of(&events);
    let window = session_of(events)
        .screen(&catalog)
        .advanced_action_reward
        .expect("fenêtre de la récompense");
    let offers = arcanum_ui::offers_window(
        &view(&choosing, SOLO),
        &legal_actions(&choosing, SOLO),
        &catalog,
    );
    let in_offers = &offers.columns[0].cards[1];
    let in_choice = &window.cards[1];
    assert_eq!(
        (
            &in_choice.name,
            in_choice.color,
            &in_choice.lines,
            &in_choice.strong_heading,
            &in_choice.strong_colors,
            &in_choice.illustration,
        ),
        (
            &in_offers.name,
            in_offers.color,
            &in_offers.lines,
            &in_offers.strong_heading,
            &in_offers.strong_colors,
            &in_offers.illustration,
        )
    );
}

// Spec 071, C21 : « Faire une offrande à l'autel » dit E, comme « Entrer dans ».
#[test]
fn altar_button_shows_shortcut() {
    let catalog = french();
    let screen = screen_of(&state_of(&payable_altar_events()), &catalog);
    let altar = screen.altar.expect("offrande");
    assert_eq!(altar.shortcut.as_deref(), Some("E"));
}

// Spec 071, C22 et C23 : les paiements de l'offrande disent 1 à 9, « Annuler » Échap.
#[test]
fn altar_payments_show_shortcut() {
    let catalog = french();
    // Trois cristaux et deux jetons bleus : plusieurs paiements.
    let state = state_of(&events_after(
        altar_events(&Sandbox::default().dice, 3, &[blue(), blue()]),
        &[basic(card(March, 1))],
    ));
    let window = screen_in(&state, &ScreenMode::PayingTribute, &catalog)
        .altar_window
        .expect("fenêtre de l'offrande");
    assert!(window.payments.len() >= 2, "{:?}", window.payments);
    for (place, button) in window.payments.iter().enumerate() {
        assert_eq!(button.shortcut, Some((place + 1).to_string()));
    }
    assert_eq!(window.cancel.shortcut.as_deref(), Some("Échap"));
}

// Spec 071, C22 : « Prendre » des unités de récompense disent 1 et 2.
#[test]
fn unit_reward_take_shows_shortcut() {
    let catalog = french();
    let start = Ruins {
        token: RuinToken::GreyViolet,
        conquered: true,
        offer: units(2),
        ..Ruins::default()
    };
    let state = start.state();
    let session = session_of(events_after(start.events(), &[end_turn(&state)]));
    // Spec #20, C12 (test modifié) : dans la fenêtre de choix.
    let window = session
        .screen(&catalog)
        .unit_reward
        .expect("fenêtre de l'unité");
    let keys: Vec<Option<String>> = window
        .cards
        .iter()
        .map(|card| {
            card.take
                .as_ref()
                .and_then(|button| button.shortcut.clone())
        })
        .collect();
    assert_eq!(keys, [Some("1".into()), Some("2".into())]);
}
