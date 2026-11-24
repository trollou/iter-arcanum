//! Scénarios de la spec 041 côté affichage (Villages et monastères) : C34 à C43.

use arcanum_core::UndoStatus;
use arcanum_core::{
    Action, AdvancedAction, Artifact, BasicAction, BasicColor, CardId, CardPlay, Effect, Event,
    GameEndReason, GameState, SidewaysEffect, Site, Unit, UnitCard, apply, final_score,
    legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, CombatChoice, Command, GameScreen, OverlayKind, ScreenMode,
    ScreenPhase, Shortcut, board_screen, combat_screen, game_screen, hex_center, offers_window,
    repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{LabelAndCommand, board_input, labels_and_commands, session_of};

use BasicAction::*;

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

fn ice_shield() -> CardId {
    advanced(AdvancedAction::IceShield)
}

fn ruby_ring() -> CardId {
    artifact(Artifact::RubyRing)
}

fn amulet_of_sun() -> CardId {
    artifact(Artifact::AmuletOfSun)
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn at_village() -> GameState {
    Monastery {
        figure: VILLAGE,
        ..Monastery::default()
    }
    .state()
}

fn at_monastery() -> GameState {
    Monastery {
        owned: vec![peasants()],
        ..Monastery::default()
    }
    .state()
}

/// L'interaction commencée sur ce site, sans bonus, avec `influence` points (Promesse jouée de
/// côté pour une influence de la valeur voulue).
fn interacting(state: &GameState, site: Site, influence: u8) -> GameState {
    let state = apply(
        state.clone(),
        &Event::InteractionStarted {
            player: SOLO,
            site,
            bonus: 0,
            city_shields: 0,
        },
    );
    apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: card(Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(influence),
        },
    )
}

fn burned(state: GameState) -> GameState {
    apply(
        state,
        &Event::MonasteryBurned {
            player: SOLO,
            hex: MONASTERY,
        },
    )
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

fn board(state: &GameState, cursor: Option<arcanum_core::Hex>, catalog: &Catalog) -> BoardScreen {
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

// --- Acheter ---

// C34 — MKUE p. 8, 4.b : au village, le point de soin à 3, grisé faute d'influence ; plus de
// « Rien à acheter ici ».
#[test]
fn panel_offers_healing_purchase() {
    let catalog = french();
    let poor = interacting(&at_village(), Site::Village, 2);
    assert_eq!(
        labels_and_commands(&screen_of(&poor, &catalog).purchases),
        [("Acheter un point de soin (3)", &None)]
    );
    let rich = interacting(&at_village(), Site::Village, 3);
    assert_eq!(
        labels_and_commands(&screen_of(&rich, &catalog).purchases),
        [(
            "Acheter un point de soin (3)",
            &Some(Command::Play(index_of(&rich, &Action::BuyHealing)))
        )]
    );
    // Hors interaction, aucun achat.
    assert!(screen_of(&at_village(), &catalog).purchases.is_empty());
}

// C35 — MKUE p. 8, 4.b et 4.c : au monastère, chaque action avancée de l'offre d'unités à 6, avec
// le texte de la carte en infobulle, puis le point de soin à 2.
#[test]
fn panel_offers_advanced_actions_at_monastery() {
    let catalog = french();
    let text = {
        let window = offers_window(&view(&at_monastery(), SOLO), &[], &catalog);
        let units = window.columns.last().expect("colonne des unités");
        units
            .cards
            .last()
            .expect("Bouclier de glace dans l'offre d'unités")
            .lines
            .join("\n")
    };
    let poor = interacting(&at_monastery(), Site::Monastery, 5);
    let purchases = screen_of(&poor, &catalog).purchases;
    assert_eq!(
        labels_and_commands(&purchases),
        [
            ("Acheter : Bouclier de glace (6)", &None),
            (
                "Acheter un point de soin (2)",
                &Some(Command::Play(index_of(&poor, &Action::BuyHealing)))
            ),
        ]
    );
    // Le texte de l'action avancée en infobulle, pas celui du soin.
    assert_eq!(purchases[0].tooltip.as_ref(), Some(&text));
    assert_eq!(purchases[1].tooltip, None);
    let rich = interacting(&at_monastery(), Site::Monastery, 6);
    let purchases = screen_of(&rich, &catalog).purchases;
    assert_eq!(
        purchases[0].command,
        Some(Command::Play(index_of(
            &rich,
            &Action::BuyAdvancedAction { card: ice_shield() }
        )))
    );
    assert_eq!(purchases[0].tooltip, Some(text));
}

// C36 — MKUE p. 7, 5.f : l'action avancée du monastère suit les unités, marquée « monastère ».
#[test]
fn offers_window_shows_monastery_cards() {
    let catalog = french();
    let state = Monastery {
        offer: vec![peasants()],
        ..Monastery::default()
    }
    .state();
    let window = offers_window(&view(&state, SOLO), &legal_actions(&state, SOLO), &catalog);
    let units = window.columns.last().expect("colonne des unités");
    let names: Vec<&str> = units.cards.iter().map(|card| card.name.as_str()).collect();
    assert_eq!(names, ["Paysans", "Bouclier de glace"]);
    let marks: Vec<Option<&str>> = units
        .cards
        .iter()
        .map(|card| card.mark.as_deref())
        .collect();
    assert_eq!(marks, [None, Some("monastère")]);
    assert!(units.cards.iter().all(|card| card.take.is_none()));
}

// --- Incendier ---

// C37 — carte du monastère : le bouton, les points perdus, la confirmation qui dit le prix.
#[test]
fn burn_button_asks_confirmation() {
    let catalog = french();
    let state = at_monastery();
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        screen
            .burn
            .as_ref()
            .map(arcanum_ui::Button::label_and_command),
        Some(("Incendier le monastère", &Command::OpenBurn))
    );
    assert_eq!(screen.burn_warning, None);
    assert_eq!(screen.burn_confirm, None);
    let walked = done(&state, basic(card(March, 1)));
    assert_eq!(
        screen_of(&walked, &catalog).burn_warning.as_deref(),
        Some("Incendier fait perdre : 2 mouvement, 0 influence")
    );
    let confirming = screen_in(&state, &ScreenMode::ConfirmingBurn, &catalog);
    let prompt = confirming.burn_confirm.expect("confirmation de l'incendie");
    assert_eq!(
        prompt.text,
        "Incendier le monastère ? Réputation \u{2212}3, combat contre un ennemi violet, sans vos \
         unités. Pas de retour en arrière."
    );
    assert_eq!(
        labels_and_commands(&prompt.buttons),
        [
            (
                "Incendier",
                &Command::Play(index_of(&state, &Action::BurnMonastery))
            ),
            ("Annuler", &Command::CancelChoice),
        ]
    );
    // Spec 071, C23 : Échap annule.
    assert_eq!(prompt.buttons[1].shortcut.as_deref(), Some("Échap"));
    assert_eq!(confirming.burn, None);
    // Au village, pas d'incendie.
    assert_eq!(screen_of(&at_village(), &catalog).burn, None);
}

// C37 — un clic ouvre la confirmation ; « Annuler » ou Échap ne font rien.
#[test]
fn burn_confirmation_cancels() {
    let catalog = french();
    let mut session = session_of(
        Monastery {
            owned: vec![peasants()],
            ..Monastery::default()
        }
        .events(),
    );
    let before = session.runner().log().len();
    session.command(Command::OpenBurn);
    assert!(session.screen(&catalog).burn_confirm.is_some());
    session.shortcut(Shortcut::Cancel);
    session.run_frame();
    assert_eq!(session.screen(&catalog).burn_confirm, None);
    session.command(Command::OpenBurn);
    session.command(Command::CancelChoice);
    session.run_frame();
    assert_eq!(session.runner().log().len(), before);
    assert_eq!(session.runner().state().burning(SOLO), None);
    // Confirmé, l'incendie part.
    session.command(Command::OpenBurn);
    let confirm = session
        .screen(&catalog)
        .burn_confirm
        .expect("confirmation")
        .buttons[0]
        .command;
    session.command(confirm);
    session.run_frame();
    assert_eq!(session.runner().state().burning(SOLO), Some(MONASTERY));
}

// C38 — carte du monastère ; Compendium, *Combat* S43 : les unités ne participent pas.
#[test]
fn burning_combat_hides_units() {
    let catalog = french();
    let state = done(&at_monastery(), Action::BurnMonastery);
    let combat = combat_screen(
        &view(&state, SOLO),
        &legal_actions(&state, SOLO),
        &CombatChoice::default(),
        UndoStatus::Possible,
        &catalog,
    )
    .expect("écran de combat");
    assert_eq!(
        combat.units_excluded.as_deref(),
        Some("Vos unités ne participent pas à ce combat")
    );
    // Un combat ordinaire laisse les unités.
    let ordinary = fight(VILLAGE, &move_hand(), &sandbox_tokens(), &[ORCS_2]);
    let combat = combat_screen(
        &view(&ordinary, SOLO),
        &legal_actions(&ordinary, SOLO),
        &CombatChoice::default(),
        UndoStatus::Possible,
        &catalog,
    )
    .expect("écran de combat");
    assert_eq!(combat.units_excluded, None);
}

// C39 — un jeton Bouclier sur la case du monastère incendié, dessiné comme un drapeau depuis la
// spec 043 (C38b) ; l'infobulle le dit.
#[test]
fn board_shows_burned_monastery() {
    let catalog = french();
    let intact = Monastery {
        figure: NEAR_MONASTERY,
        ..Monastery::default()
    }
    .state();
    let before = board(&intact, Some(MONASTERY), &catalog);
    assert!(
        !before
            .overlays
            .iter()
            .any(|overlay| overlay.kind == OverlayKind::Flag)
    );
    assert!(before.hover_lines.contains(&"Monastère".to_string()));
    let after = board(&burned(intact), Some(MONASTERY), &catalog);
    let shields: Vec<_> = after
        .overlays
        .iter()
        .filter(|overlay| overlay.kind == OverlayKind::Flag)
        .map(|overlay| overlay.hex)
        .collect();
    assert_eq!(shields, [MONASTERY]);
    assert!(
        after
            .hover_lines
            .contains(&"Monastère incendié".to_string()),
        "{:?}",
        after.hover_lines
    );
    assert!(!after.hover_lines.contains(&"Monastère".to_string()));
}

// --- Récompense ---

/// Les deux artefacts tirés à la fin du tour, en attente du choix.
fn choosing_artifact() -> Vec<Event> {
    let mut events = Monastery {
        artifacts: vec![ruby_ring(), amulet_of_sun()],
        ..Monastery::default()
    }
    .events();
    events.push(Event::ArtifactsDrawn {
        player: SOLO,
        cards: vec![ruby_ring(), amulet_of_sun()],
    });
    events
}

// C40 — MKUE p. 11, 5.b : deux artefacts tirés, nom et texte, « Prendre » sous chacun.
#[test]
fn artifact_choice_window() {
    let catalog = french();
    let state = choosing_artifact().iter().fold(GameState::default(), apply);
    let window = screen_of(&state, &catalog)
        .artifact_choice
        .expect("fenêtre de récompense");
    assert_eq!(window.title, "Récompense : prenez un artefact");
    let names: Vec<&str> = window.cards.iter().map(|card| card.name.as_str()).collect();
    assert_eq!(names, ["Anneau de rubis", "Amulette du soleil"]);
    assert!(
        window.cards[0]
            .lines
            .iter()
            // Spec 055 : l'artefact est codé, son texte écrit seul (test modifié, C1).
            .any(|line| line == "Jeton et cristal rouges ; Gloire +1"),
        "{:?}",
        window.cards[0].lines
    );
    let takes: Vec<_> = window
        .cards
        .iter()
        .map(|card| card.take.clone().expect("Prendre"))
        .collect();
    assert_eq!(
        labels_and_commands(&takes),
        [
            (
                "Prendre",
                &Command::Play(index_of(
                    &state,
                    &Action::TakeArtifact { card: ruby_ring() }
                ))
            ),
            (
                "Prendre",
                &Command::Play(index_of(
                    &state,
                    &Action::TakeArtifact {
                        card: amulet_of_sun()
                    }
                ))
            ),
        ]
    );
    // Sans tirage, pas de fenêtre.
    assert_eq!(screen_of(&at_monastery(), &catalog).artifact_choice, None);
}

// Raccourcis 1 et 2 (spec 041, *Raccourcis clavier*) : prendre le 2e artefact tiré.
#[test]
fn key_two_takes_the_second_artifact() {
    let mut session = session_of(choosing_artifact());
    session.shortcut(Shortcut::Take(2));
    session.run_frame();
    let state = session.runner().state();
    assert!(state.drawn_artifacts(SOLO).is_empty());
    assert!(state.hand(SOLO).contains(&amulet_of_sun()));
}

// C43 — la récompense gagnée ce tour s'annonce sous les points.
#[test]
fn panel_announces_pending_reward() {
    let catalog = french();
    assert_eq!(screen_of(&at_monastery(), &catalog).reward_pending, None);
    let state = burned(at_monastery());
    assert_eq!(
        screen_of(&state, &catalog).reward_pending.as_deref(),
        Some("Récompense en fin de tour : 1 artefact(s)")
    );
}

// --- Décompte ---

// C41 — MKUE p. 15 : Le Collectionneur et Le Conquérant comptent artefacts et Boucliers.
#[test]
fn score_shows_artifacts_and_shields() {
    let catalog = french();
    let mut state = burned(at_monastery());
    let mut deck = state.deck(SOLO).to_vec();
    deck.insert(0, ruby_ring());
    state = apply(
        state,
        &Event::DeckShuffled {
            player: SOLO,
            order: deck,
        },
    );
    for color in [BasicColor::Red, BasicColor::Blue] {
        state = apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        );
    }
    let finished = arcanum_core::PlayerView {
        final_score: Some(final_score(&state, SOLO)),
        game_end: Some(GameEndReason::LastRound),
        ..view(&state, SOLO)
    };
    let screen = game_screen(
        &finished,
        ScreenPhase::Finished,
        &ScreenMode::Normal,
        &catalog,
    );
    let lines = screen.score_window.expect("fenêtre de fin").lines;
    // Pluriel en « (s) » (Mathieu, 2026-11-04) : arcanum-i18n n'accorde pas.
    assert!(
        lines.contains(&"Le Collectionneur (1 artefact(s), 2 cristaux) : 3".to_string()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Le Conquérant (1 Bouclier(s)) : 2".to_string()),
        "{lines:?}"
    );
}

// --- Piller ---

// C42 — Compendium, *Turn Start* S1 : au début du tour seulement, sur un village.
#[test]
fn plunder_button_at_turn_start() {
    let catalog = french();
    let state = at_village();
    assert_eq!(
        screen_of(&state, &catalog)
            .plunder
            .as_ref()
            .map(arcanum_ui::Button::label_and_command),
        Some((
            "Piller le village (2 cartes, Réputation \u{2212}1)",
            &Command::Play(index_of(&state, &Action::PlunderVillage))
        ))
    );
    let played = done(&state, basic(card(March, 1)));
    assert_eq!(screen_of(&played, &catalog).plunder, None);
    assert_eq!(screen_of(&at_monastery(), &catalog).plunder, None);
}

// Spec 071, C22 : « Prendre » du 1er et du 2e artefact disent 1 et 2.
#[test]
fn artifact_take_shows_shortcut() {
    let catalog = french();
    let state = choosing_artifact().iter().fold(GameState::default(), apply);
    let window = screen_of(&state, &catalog)
        .artifact_choice
        .expect("fenêtre de récompense");
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
