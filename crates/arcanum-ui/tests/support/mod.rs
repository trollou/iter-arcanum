//! Outils partagés des tests d'arcanum-ui : entrée du plateau de jeu construite d'un bloc (issue privée 51),
//! plateau de démonstration de la spec 011, et comparaison des boutons sur leur libellé et leur
//! commande (issue privée 157).

// Module inclus par plusieurs crates de test : chacune n'en utilise qu'une partie.
#![allow(dead_code)]

pub mod logs;

use std::collections::BTreeMap;

use arcanum_core::{
    Action, EnemyColor, EnemyToken, Event, EventLog, GameConfig, GameRunner, Hero, Hex, Map,
    MapShape, PlayerId, PlayerView, RngPositions, RuinToken, Scenario, Tactic, TileId,
    TilePosition,
};
use arcanum_i18n::Catalog;
use arcanum_ui::{
    BoardActionButton, BoardActionKind, BoardCamera, BoardInput, BoardMovement, Button, Command,
    CrystalButton, GameScreen, GameSession, MenuBar, MenuState, PurchaseButton, controls_screen,
    human_player, menu_bar,
};
use eframe::egui::{Pos2, Rect};

/// La barre de menu d'une vue (spec 074), commandes de la partie comme au début d'un tour.
pub fn bar_of(view: &PlayerView, catalog: &Catalog) -> MenuBar {
    let controls = controls_screen(
        arcanum_core::UndoStatus::NothingSinceTurnStart,
        false,
        None,
        catalog,
    );
    menu_bar(view, &controls, MenuState::default(), catalog)
}

/// Une session humaine qui reprend le journal `events` (issue privée 203).
pub fn session_of(events: Vec<Event>) -> GameSession {
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let (player, human) = human_player();
    let runner = GameRunner::resume(log, RngPositions::new(), vec![Box::new(player)])
        .expect("journal valide");
    let mut session = GameSession::with_runner(runner, human);
    session.run_frame();
    session
}

/// Joue l'action légale `wanted` dans la session.
#[track_caller]
pub fn play(session: &mut GameSession, wanted: &Action) {
    let index = session
        .legal_actions()
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"));
    session.command(Command::Play(index));
    session.run_frame();
}

/// Aucun jeton d'ennemi, pour un plateau hors partie.
static NO_ENEMY: BTreeMap<Hex, Vec<EnemyToken>> = BTreeMap::new();

/// Aucun jeton Bouclier, pour un plateau hors partie (spec 041).
static NO_SHIELD: BTreeMap<Hex, PlayerId> = BTreeMap::new();

/// Aucun défenseur face cachée, pour un plateau hors partie (spec 043).
static NO_DEFENDER: BTreeMap<Hex, Vec<EnemyColor>> = BTreeMap::new();
static NO_CITY: BTreeMap<Hex, arcanum_core::CityState> = BTreeMap::new();

/// Aucune ruine, pour un plateau hors partie (spec 045).
static NO_RUIN: BTreeMap<Hex, Option<RuinToken>> = BTreeMap::new();

/// Configuration des tests qui ne dépendent pas de l'écran de nouvelle partie (spec 065) :
/// Première reconnaissance et Arythea, l'ancienne configuration par défaut, pour garder leurs
/// graines.
pub fn first_reconnaissance_config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Spec 060 : prend la tactique sans effet si son choix attend (Lève-tôt, Au crépuscule, ou une fois
/// retirées Au bon moment, Appel du mana), pour les tests d'avant la spec ; rend vrai si elle l'a
/// prise.
pub fn past_tactic_choice(session: &mut GameSession) -> bool {
    let legal = session.legal_actions();
    let tactic = [
        Tactic::EarlyBird,
        Tactic::FromTheDusk,
        Tactic::TheRightMoment,
        Tactic::ManaSearch,
    ]
    .map(|tactic| Action::ChooseTactic { tactic })
    .into_iter()
    .find(|action| legal.contains(action));
    match tactic {
        Some(action) => {
            play(session, &action);
            true
        }
        None => false,
    }
}

/// Spec 060 : une partie de `config` dont la tactique sans effet est prise, pour les tests d'avant
/// la spec qui commencent au tour 1.
pub fn started_session(config: GameConfig) -> GameSession {
    let mut session = GameSession::new(config);
    session.run_frame();
    past_tactic_choice(&mut session);
    session
}

/// Entrée du plateau de jeu d'une partie, remplie comme l'application la remplit (`app/board.rs`) à
/// partir de la vue joueur et des actions légales. Un test ne change ensuite que ce qui le
/// concerne : `BoardInput { movement: None, ..board_input(…) }`. Un champ ajouté par une spec ne
/// se remplit donc qu'ici.
pub fn board_input<'a>(
    view: &'a PlayerView,
    legal_actions: &'a [Action],
    camera: BoardCamera,
    area: Rect,
    cursor: Option<Pos2>,
) -> BoardInput<'a> {
    BoardInput {
        map: &view.map,
        figure: view.figure,
        hero: view.hero,
        shape: view
            .scenario
            .map_or(MapShape::Wedge, |scenario| scenario.shape()),
        camera,
        area,
        cursor,
        inventory: Some(view.inventory),
        movement: Some(board_movement(view, legal_actions)),
        enemy_tokens: &view.enemy_tokens,
        shields: &view.shields,
        face_down_defenders: &view.face_down_defenders,
        cities: &view.cities,
        ruins: &view.ruins,
        actions: &[],
        site_anchor: None,
        time: 0.0,
        walk: None,
        departure: None,
        animations: true,
        destinations: &[],
    }
}

/// Ce qu'il faut au déplacement, tiré de la vue joueur comme dans l'application.
pub fn board_movement<'a>(view: &'a PlayerView, legal_actions: &'a [Action]) -> BoardMovement<'a> {
    BoardMovement {
        move_rules: &view.move_rules,
        move_points: view.move_points,
        next_tile: view.next_tile,
        legal_actions,
        movement_ended: view.movement_ended,
        enemies_to_fight: &view.enemies_to_fight,
    }
}

/// Entrée d'un plateau seul, hors partie : ni figurine, ni inventaire, ni déplacement, ni jeton.
pub fn map_input(
    map: &Map,
    camera: BoardCamera,
    area: Rect,
    cursor: Option<Pos2>,
) -> BoardInput<'_> {
    BoardInput {
        map,
        figure: None,
        hero: None,
        shape: MapShape::Wedge,
        camera,
        area,
        cursor,
        inventory: None,
        movement: None,
        enemy_tokens: &NO_ENEMY,
        shields: &NO_SHIELD,
        face_down_defenders: &NO_DEFENDER,
        cities: &NO_CITY,
        ruins: &NO_RUIN,
        actions: &[],
        site_anchor: None,
        time: 0.0,
        walk: None,
        departure: None,
        animations: true,
        destinations: &[],
    }
}

/// Plateau de démonstration de la spec 011. Depuis la spec 012, l'application affiche le plateau
/// de jeu de la partie : ce plateau ne sert plus qu'aux tests de la spec 011, qui gardent leur sens.
pub struct DemoBoard {
    pub map: Map,
    pub figure: Hex,
}

/// Tuile de départ A, puis campagne 1 à 11 et royaume 1 à 8, chacune à l'emplacement libre le
/// plus proche de l'origine, pour un plateau compact. Ce n'est pas une partie : aucune règle d'exploration.
pub fn demo_board() -> DemoBoard {
    let mut map = Map::new();
    let starting = map.place(TileId::StartingA, TilePosition::ORIGIN);
    debug_assert!(starting.is_ok(), "plateau vide");
    let order = (1..=11)
        .map(TileId::Countryside)
        .chain((1..=8).map(TileId::Core));
    for id in order {
        // Le plus proche de l'origine, le premier de la liste en cas d'égalité : `min_by_key`
        // garde le premier minimum.
        let nearest = map
            .free_neighbor_positions()
            .into_iter()
            .min_by_key(|position| distance_from_origin(position.center()));
        if let Some(position) = nearest {
            // Emplacement libre et tuile jamais posée : la pose ne peut pas échouer.
            let placed = map.place(id, position);
            debug_assert!(placed.is_ok(), "{id:?} en {position:?}");
        }
    }
    DemoBoard {
        map,
        figure: Hex::ORIGIN,
    }
}

/// Distance en nombre de cases depuis `(0, 0)`.
fn distance_from_origin(hex: Hex) -> i32 {
    (hex.q.abs() + hex.r.abs() + (hex.q + hex.r).abs()) / 2
}

/// Libellé et commande d'un bouton, ce que compare un test qui ne porte pas sur ses autres champs
/// (raccourci, infobulle…) : un champ ajouté au bouton ne touche alors que ses propres tests
/// (issue privée 157). Un test qui porte sur un autre champ le compare à part.
pub trait LabelAndCommand {
    type Command;
    fn label_and_command(&self) -> (&str, &Self::Command);
}

impl LabelAndCommand for Button {
    type Command = Command;
    fn label_and_command(&self) -> (&str, &Command) {
        (&self.label, &self.command)
    }
}

impl LabelAndCommand for PurchaseButton {
    /// `None` pour un achat grisé (spec 041).
    type Command = Option<Command>;
    fn label_and_command(&self) -> (&str, &Option<Command>) {
        (&self.label, &self.command)
    }
}

impl LabelAndCommand for CrystalButton {
    /// `None` pour une couleur pleine (spec 044).
    type Command = Option<Command>;
    fn label_and_command(&self) -> (&str, &Option<Command>) {
        (&self.label, &self.command)
    }
}

/// Libellé et commande de chaque bouton, dans l'ordre.
pub fn labels_and_commands<B: LabelAndCommand>(buttons: &[B]) -> Vec<(&str, &B::Command)> {
    buttons.iter().map(B::label_and_command).collect()
}

/// « Finir le tour… » du bas du panneau quand il est actif hors du mode fin du tour, comme l'ancien
/// champ `GameScreen::end_turn`, que l'écran ne dessinait plus (issue privée 259).
pub fn end_turn_button(screen: &GameScreen) -> Option<&PurchaseButton> {
    screen
        .bottom_buttons
        .last()
        .filter(|button| button.command.is_some() && screen.end_turn_prompt.is_none())
}

/// « Finir la manche… » du bas du panneau, quand l'annonce est possible ; son infobulle porte le
/// texte de l'ancien bandeau (spec 073, C33 ; issue privée 259).
pub fn announce_button<'a>(
    screen: &'a GameScreen,
    catalog: &Catalog,
) -> Option<&'a PurchaseButton> {
    let label = catalog.text("round.announce.button_short");
    screen
        .bottom_buttons
        .iter()
        .find(|button| button.label == label)
}

/// Les boutons d'action du plateau d'un genre (spec 075, C24 à C29).
pub fn board_actions_of(screen: &GameScreen, kind: BoardActionKind) -> Vec<&BoardActionButton> {
    screen
        .board_actions
        .iter()
        .filter(|button| button.kind == kind)
        .collect()
}

/// « Défier … » et « Engager le combat », par la dernière ligne de l'infobulle des boutons du
/// plateau, qui ont remplacé les boutons de combat du panneau (spec 075, Q9 ; issue privée 259).
pub fn combat_buttons(screen: &GameScreen) -> Vec<&BoardActionButton> {
    screen
        .board_actions
        .iter()
        .filter(|button| {
            matches!(
                button.kind,
                BoardActionKind::Challenge | BoardActionKind::StartCombat
            )
        })
        .collect()
}

/// Les libellés d'avant de `combat_buttons` : la dernière ligne de leur infobulle.
pub fn combat_labels(screen: &GameScreen) -> Vec<String> {
    combat_buttons(screen)
        .into_iter()
        .filter_map(|button| button.tooltip.last().cloned())
        .collect()
}

/// « À combattre : … », l'ancien bandeau, en tête de l'infobulle d'« Engager le combat »
/// (spec 075 ; issue privée 259).
pub fn to_fight_banner(screen: &GameScreen) -> Option<String> {
    board_actions_of(screen, BoardActionKind::StartCombat)
        .first()
        .filter(|button| button.tooltip.len() > 1)
        .map(|button| button.tooltip[0].clone())
}

/// Libellé d'une touche avec modificateur sur le système de compilation : « Ctrl+Z », « Cmd+Z » sous
/// macOS (spec #7, C7).
pub fn ctrl(key: &str) -> String {
    let modifier = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };
    format!("{modifier}+{key}")
}
