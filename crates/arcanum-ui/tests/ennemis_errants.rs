//! Scénarios de la spec 030 côté affichage (jetons, provocation, panneau), sans fenêtre.

use std::collections::BTreeSet;
use std::fs;

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, DeedCard, Enemy, EnemyColor, EnemyToken, Event,
    GameConfig, GameContent, GameState, Hero, Hex, PlayerId, Scenario, TileId, TilePosition, apply,
    decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, Command, OverlayKind, ScreenMode, ScreenPhase, board_screen,
    figure_image, game_screen, hex_center, repository_assets_dir, token_lines,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

mod support;

use support::{board_input, combat_buttons, end_turn_button, to_fight_banner};

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;

const VILLAGE: Hex = Hex::new(2, -3);
const ORCS_1: Hex = Hex::new(1, -4);
const ORCS_2: Hex = Hex::new(3, -3);
const PLAIN_W: Hex = Hex::new(2, -2);
const PLAIN_SW: Hex = Hex::new(1, -2);

fn language(code: &str) -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language(code.into()),
    )
    .expect("fichier de langue du dépôt")
}

fn french() -> Catalog {
    language("fr")
}

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

fn config() -> GameConfig {
    GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
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

fn basic(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        },
    }
}

fn done(state: GameState, action: Action) -> GameState {
    decide(&state, SOLO, &action, &mut config().rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
        .iter()
        .fold(state, apply)
}

/// Bac à sable construit par événements : Cogneurs sur les orques ①, Rôdeurs sur les ②,
/// figurine au village, deux Marche jouées (4 points).
fn at_the_village() -> GameState {
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![Event::GameStarted { config: config() }];
    events.extend(
        [
            (TileId::StartingA, TilePosition::ORIGIN),
            (TileId::Countryside(1), TilePosition::from_lattice(0, 1)),
            (TileId::Countryside(2), TilePosition::from_lattice(1, 0)),
        ]
        .map(|(tile, position)| Event::TilePlaced { tile, position }),
    );
    events.extend([
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: VILLAGE,
        },
        Event::DeckShuffled {
            player: SOLO,
            order,
        },
    ]);
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    let placed = [
        (EnemyToken::new(Enemy::Ironclads, 1), ORCS_1),
        (EnemyToken::new(Enemy::Prowlers, 1), ORCS_2),
    ];
    let mut green: Vec<EnemyToken> = placed.iter().map(|&(token, _)| token).collect();
    let rest: Vec<EnemyToken> = GameContent::embedded()
        .enemy_tokens(EnemyColor::Green)
        .into_iter()
        .filter(|token| !green.contains(token))
        .collect();
    green.extend(rest);
    events.push(Event::EnemyPileShuffled {
        color: EnemyColor::Green,
        order: green,
    });
    events.extend(
        placed
            .iter()
            .map(|&(token, hex)| Event::EnemyTokenPlaced { token, hex }),
    );
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(state, basic(card(March, 1)));
    done(state, basic(card(March, 2)))
}

/// Au village, puis le pas vers la plaine (2,−2) qui provoque les Rôdeurs.
fn after_provocation() -> GameState {
    done(at_the_village(), Action::MoveHero { to: PLAIN_W })
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

fn panel(state: &GameState, catalog: &Catalog) -> arcanum_ui::GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        catalog,
    )
}

// C23 — MKUE p. 7, 4.b
#[test]
fn hover_describes_the_enemy_token() {
    let catalog = french();
    let screen = board(&at_the_village(), Some(ORCS_2), &catalog);
    assert_eq!(
        screen.hover_lines,
        [
            "Tuile de campagne 2",
            "Colline",
            "Rôdeurs (orques maraudeurs) : armure 3, attaque 4, gloire 2",
            "Case occupée par un ennemi errant",
        ]
    );

    let screen = board(&at_the_village(), Some(ORCS_1), &catalog);
    assert_eq!(
        screen.hover_lines[2..],
        [
            "Cogneurs (orques maraudeurs) : armure 3, attaque 4, gloire 4",
            "Capacités : brutal",
            "Résistances : physique",
            "Case occupée par un ennemi errant",
        ]
    );
}

// C23 : l'attaque s'écrit avec son élément, ou « invocation ».
#[test]
fn attack_is_written_with_its_element() {
    let catalog = french();
    let line = |enemy| token_lines(EnemyToken::new(enemy, 1), &catalog)[0].clone();
    assert_eq!(
        line(Enemy::HighDragon),
        "Dragons supérieurs (draconien) : armure 9, attaque feu glacé 6, gloire 9"
    );
    assert_eq!(
        line(Enemy::FireDragon),
        "Dragons de feu (draconien) : armure 7, attaque feu 9, gloire 8"
    );
    assert_eq!(
        line(Enemy::OrcSummoners),
        "Invocateurs orques (orques maraudeurs) : armure 4, attaque invocation, gloire 4"
    );
}

// C23 bis
#[test]
fn each_rampaging_enemy_has_its_own_image() {
    let rampaging: Vec<Enemy> = Enemy::ALL
        .into_iter()
        .filter(|enemy| {
            matches!(
                enemy.definition().color,
                EnemyColor::Green | EnemyColor::Red
            )
        })
        .collect();
    assert_eq!(rampaging.len(), 10);
    let images: BTreeSet<_> = rampaging.iter().map(|enemy| figure_image(*enemy)).collect();
    assert_eq!(images.len(), 10);
    let images_dir = repository_assets_dir().join("images");
    let mut contents = BTreeSet::new();
    for image in &images {
        let path = image.path(&images_dir);
        let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?} manque"));
        assert!(text.starts_with("<svg"), "{path:?} n'est pas un SVG");
        contents.insert(text);
    }
    assert_eq!(contents.len(), 10, "deux visuels identiques");

    let screen = board(&at_the_village(), None, &french());
    let on_orcs: Vec<_> = screen
        .overlays
        .iter()
        // Spec 075 : le décor de relief de la colline n'est pas un ennemi.
        .filter(|overlay| overlay.hex == ORCS_2 && overlay.kind != OverlayKind::Relief)
        .collect();
    assert_eq!(on_orcs.len(), 1);
    assert_eq!(on_orcs[0].kind, OverlayKind::Rampaging);
    assert_eq!(on_orcs[0].image, figure_image(Enemy::Prowlers));
}

// C24 — MKUE p. 7, 4.c
#[test]
fn provoking_moves_are_marked() {
    let catalog = french();
    let screen = board(&at_the_village(), Some(PLAIN_W), &catalog);
    let mark = |hex| {
        screen
            .move_marks
            .iter()
            .find(|mark| mark.hex == hex)
            .unwrap_or_else(|| panic!("{hex:?} sans marque"))
    };
    assert!(mark(PLAIN_W).provokes);
    assert!(mark(PLAIN_W).reachable);
    assert!(!mark(PLAIN_SW).provokes);
    assert!(
        screen
            .move_marks
            .iter()
            .filter(|mark| !mark.neighbor)
            .all(|mark| !mark.provokes)
    );
    assert!(
        screen
            .hover_lines
            .contains(&"Provoque : Rôdeurs (orques maraudeurs)".to_owned())
    );
}

/// Cases des jetons qui portent le « ! » orange.
fn alerted(screen: &BoardScreen) -> Vec<Hex> {
    screen.enemy_alerts.iter().map(|alert| alert.hex).collect()
}

// C24 bis — MKUE p. 7, 4.c
#[test]
fn threatened_enemy_is_marked() {
    // Du village, la plaine (2,−2) provoque les Rôdeurs ; les Cogneurs ne touchent pas le village.
    let screen = board(&at_the_village(), None, &french());
    assert_eq!(alerted(&screen), [ORCS_2]);
    let alert = &screen.enemy_alerts[0];
    let token = screen
        .overlays
        .iter()
        .find(|overlay| overlay.hex == ORCS_2)
        .expect("jeton dessiné");
    // Sur le jeton, pas sur la case qui provoque.
    assert!(token.rect.expand(alert.radius).contains(alert.center));
}

// C25 — MKUE p. 7, 4.b
#[test]
fn enemy_space_has_no_move_mark() {
    let screen = board(&at_the_village(), Some(ORCS_2), &french());
    assert!(screen.move_marks.iter().all(|mark| mark.hex != ORCS_2));
    assert_eq!(screen.click, None);
    assert!(screen.route.is_empty());
}

// C26 — Q1
#[test]
fn panel_lists_provoked_enemies() {
    let catalog = french();
    // Spec 075 : le bandeau est en tête de l'infobulle d'« Engager le combat » (issue privée 259).
    assert_eq!(to_fight_banner(&panel(&at_the_village(), &catalog)), None);

    let state = after_provocation();
    assert_eq!(
        to_fight_banner(&panel(&state, &catalog)).as_deref(),
        // Spec 031 : le bandeau dit « À combattre », qu'il s'agisse d'ennemis provoqués ou défiés.
        Some("À combattre : Rôdeurs (orques maraudeurs)")
    );
    let screen = board(&state, None, &catalog);
    assert!(screen.move_marks.is_empty());
    assert!(screen.explore_ghosts.is_empty());
    // Le « ! » reste sur les seuls ennemis provoqués jusqu'à la fin du tour.
    assert_eq!(alerted(&screen), [ORCS_2]);
}

// C26 bis — MKUE p. 6, 6.c : après la provocation, plus de pas ni d'exploration ce tour (issue privée 65).
#[test]
fn hover_says_movement_is_over() {
    let catalog = french();
    let state = after_provocation();
    let over = "Déplacement terminé pour ce tour".to_owned();
    let missing = |lines: &[String]| lines.iter().any(|line| line.starts_with("Il vous manque"));

    // Case voisine de la figurine : le coût reste, le pas n'est plus proposé.
    let lines = board(&state, Some(PLAIN_SW), &catalog).hover_lines;
    assert!(
        lines.contains(&"Coût de déplacement : 2".to_owned()),
        "{lines:?}"
    );
    assert!(lines.contains(&over), "{lines:?}");
    assert!(!missing(&lines), "{lines:?}");

    // Toute case avec un coût, voisine ou lointaine, dit pareil.
    let around: Vec<Hex> = PLAIN_W
        .neighbors()
        .into_iter()
        .flat_map(|hex| hex.neighbors())
        .flat_map(|hex| hex.neighbors())
        .collect();
    for hex in around {
        let lines = board(&state, Some(hex), &catalog).hover_lines;
        assert!(!missing(&lines), "{hex:?} : {lines:?}");
        assert!(
            !lines.iter().any(|line| line.starts_with("Cliquez")
                || line.starts_with("Trajet")
                || line.starts_with("Provoque")),
            "{hex:?} : {lines:?}"
        );
        let costs = lines
            .iter()
            .any(|line| line.starts_with("Coût de déplacement"));
        assert_eq!(costs, lines.contains(&over), "{hex:?} : {lines:?}");
    }
}

/// Les libellés d'avant des boutons du combat, posés sur le plateau (spec 075 ; issue privée 259).
fn combat_labels(state: &GameState, catalog: &Catalog) -> Vec<String> {
    support::combat_labels(&panel(state, catalog))
}

// Spec 031 C35 : sous le bandeau, « Défier … » et « Engager le combat » ; les phases sont sur
// l'écran de combat.
#[test]
fn provoked_banner_offers_the_combat_steps() {
    let catalog = french();
    // Les Rôdeurs sont voisins du village : on peut les défier.
    assert_eq!(
        combat_labels(&at_the_village(), &catalog),
        ["Défier Rôdeurs (orques maraudeurs)"]
    );
    let state = after_provocation();
    assert_eq!(combat_labels(&state, &catalog), ["Engager le combat"]);
    let screen = panel(&state, &catalog);
    let Some(Command::Play(index)) = combat_buttons(&screen)[0].command else {
        panic!("le bouton joue une action légale");
    };
    assert_eq!(legal_actions(&state, SOLO)[index], Action::StartCombat);

    let state = done(state, Action::StartCombat);
    assert!(combat_labels(&state, &catalog).is_empty());
    let state = done(state, Action::NextCombatPhase);
    let state = done(state, Action::NextCombatPhase);
    let state = done(state, Action::EndCombat);
    assert!(combat_labels(&state, &catalog).is_empty());
    assert!(end_turn_button(&panel(&state, &catalog)).is_some());
}

// C28
#[test]
fn every_enemy_key_exists_in_french() {
    for catalog in [french(), language("en")] {
        board(&at_the_village(), Some(ORCS_1), &catalog);
        board(&at_the_village(), Some(PLAIN_W), &catalog);
        panel(&after_provocation(), &catalog);
        for enemy in Enemy::ALL {
            token_lines(EnemyToken::new(enemy, 1), &catalog);
        }
        assert_eq!(catalog.gaps(), vec![]);
    }
}
