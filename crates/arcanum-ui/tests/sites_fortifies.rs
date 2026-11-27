//! Scénarios de la spec 043 côté affichage (Sites fortifiés) : C35 à C45.

use arcanum_core::UndoStatus;
use arcanum_core::{
    Action, BasicColor, CardId, CardPlay, DeedCard, Effect, Enemy, EnemyToken, Event, GameState,
    Hex, ManaColor, SidewaysEffect, Spell, Unit, UnitCard, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, CombatChoice, Command, FOG_IMAGE, GameScreen, OverlayKind,
    ScreenMode, ScreenPhase, Shortcut, board_screen, combat_screen, figure_image, flag_image,
    game_screen, hex_center, repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{board_input, labels_and_commands, play, session_of};

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

/// Le décor joué : ces cartes jouées pour leur effet de base.
fn playing(decor: Fortified, cards: &[CardId]) -> GameState {
    cards
        .iter()
        .fold(decor.state(), |state, &card| done(&state, basic(card)))
}

fn move_to(hex: Hex) -> Action {
    Action::MoveHero { to: hex }
}

fn spell(spell: Spell) -> CardId {
    CardId::new(DeedCard::Spell(spell), 1)
}

fn fireball() -> CardId {
    spell(Spell::Fireball)
}

fn snowstorm() -> CardId {
    spell(Spell::Snowstorm)
}

fn expose() -> CardId {
    spell(Spell::Expose)
}

fn swordsmen() -> EnemyToken {
    token(Enemy::Swordsmen, 1)
}

fn defender_image(screen: &BoardScreen, hex: Hex) -> Option<arcanum_ui::ImageId> {
    screen
        .overlays
        .iter()
        .find(|overlay| overlay.hex == hex && overlay.kind == OverlayKind::Defender)
        .map(|overlay| overlay.image)
}

/// La figurine voisine de la forteresse, quatre points de mouvement (deux Marches).
fn next_to_keep(keep: Option<(EnemyToken, bool)>) -> GameState {
    playing(
        Fortified {
            figure: NEAR_KEEP,
            keep,
            ..Fortified::default()
        },
        &[card(March, 1), card(March, 2)],
    )
}

/// La forteresse conquise, la figurine `figure`, sans défenseur.
fn own_keep(figure: Hex) -> Fortified {
    Fortified {
        figure,
        keep: None,
        conquered: vec![KEEP],
        ..Fortified::default()
    }
}

// --- Plateau ---

// C35 — Plateau : un défenseur face cachée est un dos de jeton de sa couleur avec « ? » ; retourné,
// le jeton habituel. Le site reste dessiné.
#[test]
fn board_draws_face_down_defenders() {
    let catalog = french();
    let hidden = board(&Fortified::default().state(), None, &catalog);
    assert_eq!(defender_image(&hidden, KEEP), Some(FOG_IMAGE));
    assert_eq!(defender_image(&hidden, MAGE_TOWER), Some(FOG_IMAGE));
    assert!(
        hidden
            .overlays
            .iter()
            .any(|overlay| overlay.hex == KEEP && overlay.kind == OverlayKind::Site)
    );
    let revealed = Fortified {
        keep: Some((guardsmen(), true)),
        ..Fortified::default()
    }
    .state();
    assert_eq!(
        defender_image(&board(&revealed, None, &catalog), KEEP),
        Some(figure_image(Enemy::Guardsmen))
    );
}

// C36 — Infobulle d'un site fortifié : le défenseur face cachée par sa couleur, ou son nom une fois
// retourné ; « Forteresse conquise » ; sur une cible d'assaut, l'avertissement en rouge, puis la
// fortification du défenseur et le trajet.
#[test]
fn hover_describes_fortified_sites() {
    let catalog = french();
    let assault = "⚔ Assaut : Réputation −1, combat obligatoire".to_string();

    let hidden = board(
        &next_to_keep(Some((guardsmen(), false))),
        Some(KEEP),
        &catalog,
    );
    let lines = &hidden.hover_lines;
    assert!(lines.contains(&"Forteresse".to_string()), "{lines:?}");
    assert!(
        lines.contains(&"défenseur face cachée (gardes de forteresse)".to_string()),
        "{lines:?}"
    );
    let alert = lines.iter().position(|line| *line == assault);
    assert!(alert.is_some(), "{lines:?}");
    assert_eq!(hidden.hover_alert, alert);
    assert!(
        lines.contains(&"Défenseur fortifié : siège seulement à distance".to_string()),
        "{lines:?}"
    );
    assert!(
        !lines.iter().any(|line| line.starts_with("Cité")),
        "{lines:?}"
    );
    assert!(
        !lines.iter().any(|line| line.contains("ennemi errant")),
        "{lines:?}"
    );

    // Retourné : son nom, et les Gardiens (Fortifié) sont doublement fortifiés.
    let revealed = board(
        &next_to_keep(Some((guardsmen(), true))),
        Some(KEEP),
        &catalog,
    );
    let lines = &revealed.hover_lines;
    assert!(
        lines.iter().any(|line| line.starts_with("Gardiens")),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Défenseur doublement fortifié : aucun tir à distance".to_string()),
        "{lines:?}"
    );
    assert!(lines.contains(&assault), "{lines:?}");

    // À deux pas, le trajet de l'assaut.
    let far = playing(
        Fortified {
            keep: Some((swordsmen(), true)),
            ..Fortified::default()
        },
        &[card(March, 1), card(March, 2), card(Stamina, 1)],
    );
    let lines = board(&far, Some(KEEP), &catalog).hover_lines;
    assert!(lines.contains(&assault), "{lines:?}");
    assert!(
        lines.contains(&"Défenseur fortifié : siège seulement à distance".to_string()),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("Trajet : 2 case(s)")),
        "{lines:?}"
    );

    // Conquise : une case ordinaire.
    let conquered = playing(own_keep(NEAR_KEEP), &[card(March, 1), card(March, 2)]);
    let screen = board(&conquered, Some(KEEP), &catalog);
    let lines = &screen.hover_lines;
    assert!(
        lines.contains(&"Forteresse conquise".to_string()),
        "{lines:?}"
    );
    assert!(!lines.contains(&"Forteresse".to_string()), "{lines:?}");
    assert!(!lines.contains(&assault), "{lines:?}");
    assert!(
        lines.contains(&"Coût de déplacement : 3".to_string()),
        "{lines:?}"
    );
    assert_eq!(screen.hover_alert, None);
}

// C37 — Une forteresse qu'on peut assaillir porte une pastille rouge « ⚔ coût », voisine ou au bout
// d'un trajet ; au survol, le trajet est tracé en rouge.
#[test]
fn board_marks_assault_targets() {
    let catalog = french();
    let near = next_to_keep(Some((guardsmen(), true)));
    let screen = board(&near, None, &catalog);
    let keep = screen
        .move_marks
        .iter()
        .find(|mark| mark.hex == KEEP)
        .expect("la forteresse est marquée");
    assert!(keep.assault && keep.reachable && keep.neighbor);
    assert_eq!(keep.cost, 3);
    assert!(
        screen
            .move_marks
            .iter()
            .filter(|mark| mark.hex != KEEP)
            .all(|mark| !mark.assault)
    );

    let far = playing(
        Fortified {
            keep: Some((swordsmen(), true)),
            ..Fortified::default()
        },
        &[card(March, 1), card(March, 2), card(Stamina, 1)],
    );
    let screen = board(&far, Some(KEEP), &catalog);
    let keep = screen
        .move_marks
        .iter()
        .find(|mark| mark.hex == KEEP)
        .expect("la forteresse est marquée au bout du trajet");
    assert!(keep.assault && keep.reachable && !keep.neighbor);
    assert_eq!(keep.cost, 5);
    assert_eq!(screen.route.len(), 3);
    assert!(screen.route_assault);
    assert_eq!(
        screen.click,
        Some(Command::Play(index_of(
            &far,
            &Action::FollowRoute { to: KEEP }
        )))
    );
    // Un trajet ordinaire reste de la couleur habituelle.
    let screen = board(&far, Some(NEAR_KEEP), &catalog);
    assert!(!screen.route.is_empty());
    assert!(!screen.route_assault);
}

// C38 — Un clic sur la cible d'assaut joue l'assaut aussitôt ; « Revenir en arrière » le reprend tant
// que rien n'est révélé.
#[test]
fn assault_in_one_click() {
    let catalog = french();
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: Some((guardsmen(), true)),
        ..Fortified::default()
    };
    let mut session = session_of(decor.events());
    play(&mut session, &basic(card(March, 1)));
    play(&mut session, &basic(card(March, 2)));
    let click = board(session.runner().state(), Some(KEEP), &catalog)
        .click
        .expect("un clic sur la forteresse");
    session.command(click);
    session.run_frame();
    let state = session.runner().state();
    assert_eq!(state.figure(SOLO), Some(KEEP));
    assert!(state.assault(SOLO).is_some());
    session.command(Command::Undo);
    session.run_frame();
    let state = session.runner().state();
    assert_eq!(state.figure(SOLO), Some(NEAR_KEEP));
    assert!(state.assault(SOLO).is_none());
}

// C38b — Site conquis ou monastère incendié : le drapeau du héros à la place du jeton Bouclier.
#[test]
fn conquered_sites_show_a_flag() {
    let catalog = french();
    let state = own_keep(TWO_FROM_KEEP).state();
    let hero = view(&state, SOLO).hero.expect("héros de la partie");
    let screen = board(&state, None, &catalog);
    let flags: Vec<_> = screen
        .overlays
        .iter()
        .filter(|overlay| overlay.kind == OverlayKind::Flag)
        .map(|overlay| (overlay.hex, overlay.image))
        .collect();
    assert_eq!(flags, [(KEEP, flag_image(hero))]);
    // Le drapeau reste dans la case du site : son rectangle tient dans celui de la case.
    let flag = screen
        .overlays
        .iter()
        .find(|overlay| overlay.kind == OverlayKind::Flag)
        .expect("drapeau");
    let hex = screen
        .hexes
        .iter()
        .find(|drawing| drawing.hex == KEEP)
        .expect("case de la forteresse");
    assert!(
        hex.rect.contains_rect(flag.rect),
        "{:?} {:?}",
        hex.rect,
        flag.rect
    );
}

// --- Combat ---

/// Assaut de la forteresse depuis sa voisine, défenseur `defender` face visible, les Rôdeurs
/// errants voisins des deux cases : Marche et Endurance jouées, le pas, le combat engagé.
fn assaulting(defender: EnemyToken) -> GameState {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: Some((defender, true)),
        placed: vec![(prowlers(), Hex::new(6, -4))],
        ..Fortified::default()
    };
    let state = playing(decor, &[card(March, 1), card(Stamina, 1)]);
    let state = done(&state, move_to(KEEP));
    done(&state, Action::StartCombat)
}

fn status_of(state: &GameState, hex: Hex, catalog: &Catalog) -> String {
    combat_screen(
        &view(state, SOLO),
        &legal_actions(state, SOLO),
        &CombatChoice::default(),
        UndoStatus::Possible,
        catalog,
    )
    .expect("écran de combat")
    .enemies
    .into_iter()
    .find(|enemy| enemy.hex == hex)
    .expect("ennemi du combat")
    .status
    .unwrap_or_default()
}

// C39 — Combat d'un assaut : le défenseur porte « fortifié (site) », ou « doublement fortifié :
// pas de cible à distance » ; l'ennemi errant du même combat, rien.
#[test]
fn combat_marks_site_fortification() {
    let catalog = french();
    let state = assaulting(swordsmen());
    let keep = status_of(&state, KEEP, &catalog);
    assert!(keep.contains("fortifié (site)"), "{keep}");
    assert!(keep.contains("fortifié : siège seulement"), "{keep}");
    let rampaging = status_of(&state, Hex::new(6, -4), &catalog);
    assert!(!rampaging.contains("fortifié"), "{rampaging}");

    let state = assaulting(guardsmen());
    let keep = status_of(&state, KEEP, &catalog);
    assert!(
        keep.contains("doublement fortifié : pas de cible à distance"),
        "{keep}"
    );
    assert!(!keep.contains("siège seulement"), "{keep}");
    // En phase d'attaque, il redevient une cible : seul son site est rappelé.
    let keep = status_of(&skip_to_attack_phase(&state), KEEP, &catalog);
    assert!(keep.contains("fortifié (site)"), "{keep}");
    assert!(!keep.contains("doublement"), "{keep}");
}

// Spec 070 C4 — MKUE p. 9, *Ranged and Siege Attack Phase* 3 ; spec 043, C39 : le défenseur seul,
// doublement fortifié, n'est pas une cible à distance : ni case ni « Attaquer ».
#[test]
fn fortified_single_enemy_has_no_attack_in_ranged_phase() {
    let catalog = french();
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: Some((guardsmen(), true)),
        ..Fortified::default()
    };
    let state = playing(decor, &[card(March, 1), card(Stamina, 1)]);
    let state = done(&state, move_to(KEEP));
    let state = done(&state, Action::StartCombat);
    let screen = combat_screen(
        &view(&state, SOLO),
        &legal_actions(&state, SOLO),
        &CombatChoice::default(),
        UndoStatus::Possible,
        &catalog,
    )
    .expect("écran de combat");
    assert_eq!(screen.enemies.len(), 1);
    assert_eq!(screen.enemies[0].target, None);
    assert_eq!(screen.attack, None);
}

// --- Panneau ---

// C40 — Après une retraite, « Assaut repoussé : retour sur la case de départ » jusqu'à la fin du
// tour.
#[test]
fn panel_announces_withdrawal() {
    let catalog = french();
    let fighting = skip_to_attack_phase(&assaulting(swordsmen()));
    assert_eq!(screen_of(&fighting, &catalog).withdrawal, None);
    let withdrawn = done(&fighting, Action::EndCombat);
    assert_eq!(withdrawn.figure(SOLO), Some(NEAR_KEEP));
    assert_eq!(
        screen_of(&withdrawn, &catalog).withdrawal.as_deref(),
        Some("Assaut repoussé : retour sur la case de départ")
    );
    let next_turn = done(
        &withdrawn,
        Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    );
    assert_eq!(screen_of(&next_turn, &catalog).withdrawal, None);
}

/// La tour de mage conquise, la figurine dessus, l'offre Éclair de feu, Tempête de neige,
/// Exposition ; la Source rouge, vert, blanc ; l'interaction commencée, `influence` points.
fn at_conquered_tower(influence: u8) -> GameState {
    let decor = Fortified {
        figure: MAGE_TOWER,
        dice: vec![
            ManaColor::Basic(BasicColor::Red),
            ManaColor::Basic(BasicColor::Green),
            ManaColor::Basic(BasicColor::White),
        ],
        tower: None,
        conquered: vec![MAGE_TOWER],
        spells: vec![fireball(), snowstorm(), expose()],
        ..Fortified::default()
    };
    let state = done(&decor.state(), Action::Interact);
    with_influence(&state, influence)
}

fn spell_purchases(state: &GameState, card: CardId) -> Vec<usize> {
    legal_actions(state, SOLO)
        .iter()
        .enumerate()
        .filter(|(_, action)| matches!(action, Action::BuySpell { card: c, .. } if *c == card))
        .map(|(index, _)| index)
        .collect()
}

// Spec #20, C22 : à la tour de mage conquise, chaque achat de sort porte la carte hors zone de
// son sort, celle de la fenêtre Offres.
#[test]
fn spell_purchases_show_cards() {
    let catalog = french();
    let state = at_conquered_tower(8);
    let window = arcanum_ui::offers_window(&view(&state, SOLO), &[], &catalog);
    let cards: Vec<Option<arcanum_ui::OfferCardView>> = window.columns[1]
        .cards
        .iter()
        .map(|card| Some(bare(card)))
        .collect();
    let purchases: Vec<Option<arcanum_ui::OfferCardView>> = screen_of(&state, &catalog)
        .purchases
        .into_iter()
        .map(|purchase| purchase.card)
        .collect();
    assert_eq!(purchases, cards);
}

/// Spec #20 : la carte hors zone de la fenêtre Offres, sans ses boutons ni sa marque.
fn bare(card: &arcanum_ui::OfferCardView) -> arcanum_ui::OfferCardView {
    arcanum_ui::OfferCardView {
        take: None,
        mark: None,
        uses: Vec::new(),
        ..card.clone()
    }
}

// C41 — À une tour de mage conquise : un bouton « Acheter : Éclair de feu (7 + mana rouge) » par
// sort de l'offre, grisé si l'influence ou le mana manque ; le choix du mana ensuite.
#[test]
fn panel_offers_spells_at_mage_tower() {
    let catalog = french();
    let state = at_conquered_tower(8);
    let purchases: Vec<(String, Option<Command>)> = screen_of(&state, &catalog)
        .purchases
        .into_iter()
        .map(|purchase| (purchase.label, purchase.command))
        .collect();
    assert_eq!(
        purchases,
        [
            (
                "Acheter : Éclair de feu (7 + mana rouge)".to_string(),
                Some(Command::ChooseSpellMana(fireball()))
            ),
            // Ni dé bleu ni cristal bleu.
            (
                "Acheter : Tempête de neige (7 + mana bleu)".to_string(),
                None
            ),
            (
                "Acheter : Exposition (7 + mana blanc)".to_string(),
                Some(Command::ChooseSpellMana(expose()))
            ),
        ]
    );
    // Trop peu d'influence : tout est grisé.
    assert!(
        screen_of(&at_conquered_tower(6), &catalog)
            .purchases
            .iter()
            .all(|purchase| purchase.command.is_none())
    );
    // Le choix du mana : un bouton par paiement légal, et Annuler.
    let prompt = screen_in(&state, &ScreenMode::ChoosingSpellMana(fireball()), &catalog)
        .mana_prompt
        .expect("choix du mana de l'achat");
    let commands: Vec<Command> = prompt
        .payments
        .iter()
        .map(|button| button.command)
        .collect();
    let expected: Vec<Command> = spell_purchases(&state, fireball())
        .into_iter()
        .map(Command::Play)
        .collect();
    assert!(!expected.is_empty());
    assert_eq!(commands, expected);
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
}

// C41 — Le bouton d'achat ouvre le choix du mana, dans la session.
#[test]
fn spell_purchase_opens_mana_choice() {
    let decor = Fortified {
        figure: MAGE_TOWER,
        tower: None,
        conquered: vec![MAGE_TOWER],
        spells: vec![fireball(), snowstorm(), expose()],
        ..Fortified::default()
    };
    let mut session = session_of(decor.events());
    play(&mut session, &Action::Interact);
    session.command(Command::ChooseSpellMana(fireball()));
    // Sans influence, l'achat n'est pas légal : le mode n'est pas ouvert.
    assert!(!session.is_choosing_mana());
    let mut events = decor.events();
    events.push(Event::InteractionStarted {
        player: SOLO,
        site: arcanum_core::Site::MageTower,
        bonus: 0,
        city_shields: 0,
    });
    events.push(Event::CardPlayed {
        player: SOLO,
        card: card(Promise, 1),
        play: CardPlay::Sideways(SidewaysEffect::Influence),
        effect: Effect::Influence(8),
    });
    let mut session = session_of(events);
    session.command(Command::ChooseSpellMana(fireball()));
    assert!(session.is_choosing_mana());
}

// C42 — Dans sa forteresse : « Interagir avec les habitants », puis les unités de type forteresse.
#[test]
fn keep_interaction_lists_keep_units() {
    let catalog = french();
    let state = Fortified {
        offer: vec![
            UnitCard::new(Unit::Peasants, 1),
            UnitCard::new(Unit::UtemSwordsmen, 1),
        ],
        ..own_keep(KEEP)
    }
    .state();
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        screen.interact.map(|button| button.command),
        Some(Command::Play(index_of(&state, &Action::Interact)))
    );
    let state = with_influence(&done(&state, Action::Interact), 6);
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        screen.site_menu.as_ref().map(|menu| menu.title.as_str()),
        Some("Interaction : Forteresse")
    );
    let recruits: Vec<&str> = screen
        .recruit
        .iter()
        .map(|button| button.label.as_str())
        .collect();
    assert_eq!(recruits.len(), 1, "{recruits:?}");
    assert!(recruits[0].contains(&catalog.text(Unit::UtemSwordsmen.label_key())));
}

/// Sur la tour de mage conquise ce tour (sa récompense sort due), l'offre Éclair de feu, Tempête
/// de neige, Exposition.
fn tower_conquered_this_turn() -> Vec<Event> {
    let mut events = Fortified {
        figure: MAGE_TOWER,
        tower: None,
        spells: vec![fireball(), snowstorm(), expose()],
        ..Fortified::default()
    }
    .events();
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: MAGE_TOWER,
    });
    events
}

fn end_turn() -> Action {
    Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

fn choosing_spell() -> GameState {
    let state = tower_conquered_this_turn()
        .iter()
        .fold(GameState::default(), apply);
    let state = done(&state, basic(card(March, 1)));
    done(&state, end_turn())
}

// C43 — Fin du tour avec une récompense sort : la fenêtre montre les sorts de l'offre, nom et
// texte, « Prendre » sous chacun.
#[test]
fn spell_choice_window() {
    let catalog = french();
    let state = choosing_spell();
    let window = screen_of(&state, &catalog)
        .spell_choice
        .expect("fenêtre de récompense");
    assert_eq!(window.title, "Récompense : prenez un sort");
    let names: Vec<&str> = window.cards.iter().map(|card| card.name.as_str()).collect();
    assert_eq!(names, ["Éclair de feu", "Tempête de neige", "Exposition"]);
    assert!(!window.cards[0].lines.is_empty());
    let takes: Vec<_> = window
        .cards
        .iter()
        .map(|card| card.take.clone().expect("Prendre"))
        .collect();
    let commands: Vec<_> = [fireball(), snowstorm(), expose()]
        .into_iter()
        .map(|card| Command::Play(index_of(&state, &Action::TakeSpell { card })))
        .collect();
    assert_eq!(
        labels_and_commands(&takes),
        commands
            .iter()
            .map(|command| ("Prendre", command))
            .collect::<Vec<_>>()
    );
    // Spec 071, C22 : 1 à 3, dans l'ordre de l'offre.
    assert_eq!(
        takes
            .iter()
            .map(|button| button.shortcut.as_deref())
            .collect::<Vec<_>>(),
        [Some("1"), Some("2"), Some("3")]
    );
    // Sans récompense en attente, pas de fenêtre.
    let before = tower_conquered_this_turn()
        .iter()
        .fold(GameState::default(), apply);
    assert_eq!(screen_of(&before, &catalog).spell_choice, None);
}

// Raccourcis 1 à 3 (spec 043, *Raccourcis clavier*) : prendre le 2e sort de l'offre.
#[test]
fn key_two_takes_the_second_spell() {
    let mut session = session_of(tower_conquered_this_turn());
    play(&mut session, &basic(card(March, 1)));
    play(&mut session, &end_turn());
    session.shortcut(Shortcut::Take(2));
    session.run_frame();
    let state = session.runner().state();
    assert_eq!(state.spell_choices(SOLO), 0);
    assert!(state.hand(SOLO).contains(&snowstorm()));
}

// C44 — Près d'une forteresse possédée, « Limite de main : 6 (+1 forteresse) » ; ailleurs, rien.
#[test]
fn panel_shows_keep_hand_limit() {
    let catalog = french();
    let near = own_keep(NEAR_KEEP).state();
    assert_eq!(
        screen_of(&near, &catalog).keep_hand_limit.as_deref(),
        Some("Limite de main : 6 (+1 forteresse)")
    );
    let far = own_keep(TWO_FROM_KEEP).state();
    assert_eq!(screen_of(&far, &catalog).keep_hand_limit, None);
}

// C45 — La récompense du tour s'annonce « Récompense en fin de tour : 1 sort(s) ».
#[test]
fn panel_announces_spell_reward() {
    let catalog = french();
    assert_eq!(
        screen_of(&Fortified::default().state(), &catalog).spell_reward_pending,
        None
    );
    let state = tower_conquered_this_turn()
        .iter()
        .fold(GameState::default(), apply);
    assert_eq!(
        screen_of(&state, &catalog).spell_reward_pending.as_deref(),
        Some("Récompense en fin de tour : 1 sort(s)")
    );
}

// C37 — La pastille et l'infobulle de l'assaut écrivent « ⚔ » : les polices par défaut d'egui
// doivent avoir le glyphe (emojis monochromes, décision de Mathieu, 2026-11-05).
#[test]
fn default_fonts_draw_crossed_swords() {
    let fonts = eframe::egui::FontDefinitions::default();
    assert!(
        fonts.font_data.contains_key("emoji-icon-font"),
        "{:?}",
        fonts.font_data.keys().collect::<Vec<_>>()
    );
}

// Spec 071, C22 : « Prendre » des sorts de l'offre disent 1, 2 et 3.
#[test]
fn spell_take_shows_shortcut() {
    let catalog = french();
    let window = screen_of(&choosing_spell(), &catalog)
        .spell_choice
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
    assert_eq!(keys, [Some("1".into()), Some("2".into()), Some("3".into())]);
}
