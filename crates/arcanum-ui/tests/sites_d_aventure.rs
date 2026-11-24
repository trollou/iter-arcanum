//! Scénarios de la spec 044 côté affichage (Sites d'aventure) : C39 à C49.

use arcanum_core::UndoStatus;
use arcanum_core::{
    Action, BasicColor, CardId, CardPlay, Effect, Element, Enemy, Event, GameEndReason, GameState,
    Hex, ManaColor, SidewaysEffect, TimeOfDay, Unit, UnitCard, WoundPile, apply, final_score,
    legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, CombatChoice, Command, EndTurnChoice, GameScreen, GladeOption,
    OverlayKind, ScreenMode, ScreenPhase, Shortcut, board_screen, combat_screen, figure_image,
    flag_image, game_screen, hex_center, repository_assets_dir,
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

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn entered(adventure: Adventure) -> GameState {
    done(&adventure.state(), Action::EnterAdventureSite)
}

/// Les Gargouilles restées sur l'antre.
fn den_with_gargoyle(figure: Hex) -> Adventure {
    Adventure {
        figure,
        placed: vec![(gargoyle(), MONSTER_DEN)],
        ..Adventure::default()
    }
}

/// Le donjon conquis ce tour, le dé de récompense lancé de cette couleur.
fn dungeon_won(color: ManaColor) -> GameState {
    let fighting = skip_to_attack_phase(&entered(Adventure::default()));
    let attacking = done(&fighting, declare(&[minotaur()]));
    let first = attacking.hand(SOLO)[0];
    let attacking = apply(
        attacking,
        &Event::CardPlayed {
            player: SOLO,
            card: first,
            play: CardPlay::Sideways(SidewaysEffect::Attack),
            effect: Effect::Attack {
                amount: 5,
                element: Element::Physical,
                kind: arcanum_core::AttackKind::Melee,
            },
        },
    );
    let won = done(&attacking, Action::ResolveAttack);
    [
        Event::CombatEnded { player: SOLO },
        Event::SiteConquered {
            player: SOLO,
            hex: DUNGEON,
        },
        Event::RewardDieRolled {
            player: SOLO,
            color,
        },
    ]
    .iter()
    .fold(won, apply)
}

/// Un tour où `site` a été conquis, Marche jouée de côté.
fn prepared(state: GameState, site: Hex) -> GameState {
    let won = apply(
        state,
        &Event::SiteConquered {
            player: SOLO,
            hex: site,
        },
    );
    done(&won, sideways(card(March, 1)))
}

/// Antre conquis, fin du tour commencée, un dé or lancé : le choix de la couleur attend.
fn gold_rolled(state: GameState) -> GameState {
    [
        Event::TurnEnded { player: SOLO },
        Event::RewardDieRolled {
            player: SOLO,
            color: ManaColor::Gold,
        },
    ]
    .iter()
    .fold(prepared(state, MONSTER_DEN), apply)
}

fn with_full(state: GameState, color: BasicColor) -> GameState {
    (0..3).fold(state, |state, _| {
        apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        )
    })
}

/// Sur la clairière, une Blessure en main avec Marche, Rage, Détermination et Célérité.
fn glade_with_wound() -> Adventure {
    Adventure {
        figure: GLADE,
        hand: vec![
            card(March, 1),
            card(Rage, 1),
            card(Determination, 1),
            card(Swiftness, 1),
            wound(1),
        ],
        ..Adventure::default()
    }
}

fn marched(state: &GameState) -> GameState {
    done(state, sideways(card(March, 1)))
}

fn ending(glade: Option<Option<WoundPile>>) -> ScreenMode {
    ScreenMode::EndingTurn(EndTurnChoice {
        glade,
        ..EndTurnChoice::default()
    })
}

fn glade_option(
    label: &str,
    glade: Option<WoundPile>,
    enabled: bool,
    selected: bool,
) -> GladeOption {
    GladeOption {
        label: label.into(),
        command: Command::ChooseGlade(glade),
        enabled,
        selected,
    }
}

// --- Entrer (C39, C40) ---

// C39 — MKUE p. 6, 6.e : le bouton sur un site où l'on peut entrer, les points perdus dessous.
#[test]
fn panel_offers_to_enter_adventure_site() {
    let catalog = french();
    let state = Adventure::default().state();
    let screen = screen_of(&state, &catalog);
    let enter = screen.adventure.as_ref().expect("bouton d'entrée");
    assert_eq!(
        enter.label_and_command(),
        ("Entrer dans : Donjon", &Command::OpenAdventure)
    );
    // Spec 071, C21 : E ouvre la confirmation.
    assert_eq!(enter.shortcut.as_deref(), Some("E"));
    assert_eq!(screen.adventure_warning, None);
    let walked = done(&state, basic(card(March, 1)));
    assert_eq!(
        screen_of(&walked, &catalog).adventure_warning.as_deref(),
        Some("Entrer fait perdre : 2 mouvement, 0 influence")
    );
    let tomb = Adventure {
        figure: TOMB,
        ..Adventure::default()
    }
    .state();
    assert_eq!(
        screen_of(&tomb, &catalog)
            .adventure
            .map(|button| button.label),
        Some("Entrer dans : Tombeau".into())
    );
    // Hors d'un site, sur un antre conquis, ou pendant la confirmation : pas de bouton.
    for state in [
        Adventure {
            figure: NEAR_DUNGEON,
            ..Adventure::default()
        }
        .state(),
        Adventure {
            figure: MONSTER_DEN,
            conquered: vec![MONSTER_DEN],
            ..Adventure::default()
        }
        .state(),
    ] {
        assert_eq!(screen_of(&state, &catalog).adventure, None);
    }
    let confirming = screen_in(&state, &ScreenMode::ConfirmingAdventure, &catalog);
    assert_eq!(confirming.adventure, None);
}

// C40 — cartes de site : la confirmation dit ce qui sera combattu, les règles de Nuit, la
// récompense, et l'absence de retour en arrière quand un jeton sera tiré.
#[test]
fn enter_confirmation() {
    let catalog = french();
    let confirm = |adventure: Adventure| {
        let state = adventure.state();
        let prompt = screen_in(&state, &ScreenMode::ConfirmingAdventure, &catalog)
            .adventure_confirm
            .expect("confirmation de l'entrée");
        (state, prompt)
    };
    let (state, prompt) = confirm(Adventure::default());
    assert_eq!(
        prompt.text,
        "Combat contre 1 ennemi(s) marron, tiré(s) au hasard.\nRègles de Nuit, sans vos \
         unités.\nRécompense si vous gagnez : un sort (dé or ou noir) ou un artefact. \
         Pas de retour en arrière."
    );
    assert_eq!(
        labels_and_commands(&prompt.buttons),
        [
            (
                "Entrer",
                &Command::Play(index_of(&state, &Action::EnterAdventureSite))
            ),
            ("Annuler", &Command::CancelChoice),
        ]
    );
    // Spec 071, C23 : Échap annule.
    assert_eq!(prompt.buttons[1].shortcut.as_deref(), Some("Échap"));
    let (_, prompt) = confirm(den_with_gargoyle(MONSTER_DEN));
    assert_eq!(
        prompt.text,
        "Combat contre : Gargouille (donjon).\nRécompense si vous gagnez : deux cristaux aléatoires."
    );
    let (_, prompt) = confirm(Adventure {
        figure: SPAWNING_GROUNDS,
        placed: vec![(gargoyle(), SPAWNING_GROUNDS)],
        ..Adventure::default()
    });
    assert_eq!(
        prompt.text,
        "Combat contre : Gargouille (donjon).\nCombat contre 1 ennemi(s) marron, tiré(s) au hasard.\n\
         Récompense si vous gagnez : un artefact et trois cristaux aléatoires. Pas de retour en \
         arrière."
    );
    let (_, prompt) = confirm(Adventure {
        figure: TOMB,
        conquered: vec![TOMB],
        ..Adventure::default()
    });
    assert_eq!(
        prompt.text,
        "Combat contre 1 ennemi(s) rouge, tiré(s) au hasard.\nRègles de Nuit, sans vos unités.\n\
         Récompense si vous gagnez : la gloire seulement. Pas de retour en arrière."
    );
    // Hors confirmation, pas de bandeau.
    assert_eq!(
        screen_of(&Adventure::default().state(), &catalog).adventure_confirm,
        None
    );
}

// C40 — Q4 : un clic (ou E) ouvre la confirmation ; « Annuler » ou Échap la ferment ; « Entrer »
// joue l'action.
#[test]
fn enter_confirmation_in_session() {
    let catalog = french();
    let mut session = session_of(Adventure::default().events());
    let before = session.runner().log().len();
    session.command(Command::OpenAdventure);
    assert!(session.screen(&catalog).adventure_confirm.is_some());
    session.shortcut(Shortcut::Cancel);
    assert_eq!(session.screen(&catalog).adventure_confirm, None);
    session.shortcut(Shortcut::EnterSite);
    assert!(session.screen(&catalog).adventure_confirm.is_some());
    session.command(Command::CancelChoice);
    session.run_frame();
    assert_eq!(session.runner().log().len(), before);
    session.shortcut(Shortcut::EnterSite);
    let confirm = session
        .screen(&catalog)
        .adventure_confirm
        .expect("confirmation")
        .buttons[0]
        .command;
    session.command(confirm);
    session.run_frame();
    assert_eq!(session.runner().state().adventure(SOLO), Some(DUNGEON));
    // Hors d'un site, E ne fait rien.
    let mut elsewhere = session_of(
        Adventure {
            figure: NEAR_DUNGEON,
            ..Adventure::default()
        }
        .events(),
    );
    elsewhere.shortcut(Shortcut::EnterSite);
    assert_eq!(elsewhere.screen(&catalog).adventure_confirm, None);
}

// --- Plateau (C41 à C43) ---

// C41 — Compendium, *Miscellaneous* S15 : les jetons restés, face visible ; deux côte à côte,
// réduits, dans la case ; pas des ennemis errants (la case reste accessible, sans « ! »).
#[test]
fn board_draws_adventure_tokens() {
    let catalog = french();
    let state = den_with_gargoyle(NEAR_DEN).state();
    let screen = board(&state, None, &catalog);
    let on_den: Vec<_> = screen
        .overlays
        .iter()
        .filter(|overlay| overlay.hex == MONSTER_DEN && overlay.kind == OverlayKind::SiteEnemy)
        .collect();
    assert_eq!(on_den.len(), 1, "{:?}", screen.overlays);
    assert_eq!(on_den[0].image, figure_image(Enemy::Gargoyle));
    assert!(screen.enemy_alerts.is_empty());
    assert!(screen.move_marks.iter().any(|mark| mark.hex == MONSTER_DEN));

    let grounds = Adventure {
        figure: SPAWNING_GROUNDS,
        placed: vec![
            (gargoyle(), SPAWNING_GROUNDS),
            (minotaur(), SPAWNING_GROUNDS),
        ],
        ..Adventure::default()
    }
    .state();
    let screen = board(&grounds, None, &catalog);
    let tokens: Vec<_> = screen
        .overlays
        .iter()
        .filter(|overlay| overlay.hex == SPAWNING_GROUNDS && overlay.kind == OverlayKind::SiteEnemy)
        .collect();
    assert_eq!(tokens.len(), 2, "{:?}", screen.overlays);
    assert_eq!(
        [tokens[0].image, tokens[1].image],
        [figure_image(Enemy::Gargoyle), figure_image(Enemy::Minotaur)]
    );
    let hex = screen
        .hexes
        .iter()
        .find(|drawing| drawing.hex == SPAWNING_GROUNDS)
        .expect("case dessinée");
    for token in &tokens {
        assert!(hex.rect.contains_rect(token.rect), "{:?}", token.rect);
        assert!(token.rect.width() < on_den[0].rect.width());
    }
    assert!((tokens[0].rect.center().y - tokens[1].rect.center().y).abs() < 0.5);
    assert!(!tokens[0].rect.intersects(tokens[1].rect));
}

// C42 — spec 043 : le drapeau du héros sur les sites d'aventure conquis.
#[test]
fn conquered_adventure_sites_show_a_flag() {
    let catalog = french();
    let state = Adventure {
        figure: NEAR_DUNGEON,
        conquered: vec![DUNGEON, MONSTER_DEN],
        ..Adventure::default()
    }
    .state();
    let flags: Vec<Hex> = board(&state, None, &catalog)
        .overlays
        .iter()
        .filter(|overlay| overlay.kind == OverlayKind::Flag)
        .map(|overlay| {
            assert_eq!(overlay.image, flag_image(arcanum_core::Hero::Arythea));
            overlay.hex
        })
        .collect();
    assert_eq!(flags, [MONSTER_DEN, DUNGEON]);
}

// C43 — cartes de site : l'infobulle dit le site, son état, ce qu'on y combat et la récompense ;
// sur une clairière, ses deux effets.
#[test]
fn hover_describes_adventure_sites() {
    let catalog = french();
    let lines = hover(&Adventure::default().state(), DUNGEON, &catalog);
    assert!(lines.contains(&"Donjon".to_string()), "{lines:?}");
    assert!(
        lines.contains(
            &"Entrer : combat, récompense : un sort (dé or ou noir) ou un artefact".into()
        ),
        "{lines:?}"
    );
    let conquered = Adventure {
        conquered: vec![DUNGEON, MONSTER_DEN],
        ..Adventure::default()
    }
    .state();
    let lines = hover(&conquered, DUNGEON, &catalog);
    assert!(lines.contains(&"Donjon conquis".to_string()), "{lines:?}");
    assert!(
        lines.contains(&"Entrer : combat, récompense : la gloire seulement".into()),
        "{lines:?}"
    );
    let lines = hover(&conquered, MONSTER_DEN, &catalog);
    assert!(lines.contains(&"Antre conquis".to_string()), "{lines:?}");
    assert!(
        !lines.iter().any(|line| line.starts_with("Entrer")),
        "{lines:?}"
    );
    let lines = hover(&den_with_gargoyle(NEAR_DEN).state(), MONSTER_DEN, &catalog);
    assert!(lines.contains(&"Antre".to_string()), "{lines:?}");
    assert!(
        lines.contains(&"Vous y attendent : Gargouille (donjon)".to_string()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Entrer : combat, récompense : deux cristaux aléatoires".into()),
        "{lines:?}"
    );
    assert!(
        !lines.contains(&catalog.text("board.hover.enemy_space")),
        "{lines:?}"
    );
    let lines = hover(&Adventure::default().state(), GLADE, &catalog);
    assert!(
        lines.contains(&catalog.text("board.hover.glade")),
        "{lines:?}"
    );
}

// --- Combat (C44) ---

// C44 — carte du donjon : règles de Nuit et sans unités, en tête de l'écran de combat.
#[test]
fn combat_marks_night_rules_and_no_units() {
    let catalog = french();
    let combat = |state: &GameState| {
        combat_screen(
            &view(state, SOLO),
            &legal_actions(state, SOLO),
            &CombatChoice::default(),
            UndoStatus::Possible,
            &catalog,
        )
        .expect("écran de combat")
    };
    let dungeon = combat(&entered(Adventure {
        owned: vec![peasants()],
        ..Adventure::default()
    }));
    assert_eq!(
        dungeon.night_rules.as_deref(),
        Some("Règles de Nuit : le mana or ne paie rien")
    );
    assert_eq!(
        dungeon.units_excluded.as_deref(),
        Some("Vos unités ne participent pas à ce combat")
    );
    let den = combat(&entered(Adventure {
        owned: vec![peasants()],
        ..den_with_gargoyle(MONSTER_DEN)
    }));
    assert_eq!((den.night_rules, den.units_excluded), (None, None));
}

// --- Récompenses (C45, C46) ---

// C45 — carte du donjon ; MKUE p. 11, 5.a : le dé du donjon et les cristaux aléatoires
// s'annoncent sous les points.
#[test]
fn panel_announces_adventure_rewards() {
    let catalog = french();
    let screen = screen_of(&dungeon_won(ManaColor::Basic(BasicColor::Green)), &catalog);
    assert_eq!(
        screen.reward_die.as_deref(),
        Some("Dé de récompense : vert, un artefact")
    );
    assert_eq!(
        screen.reward_pending.as_deref(),
        Some("Récompense en fin de tour : 1 artefact(s)")
    );
    assert_eq!(
        screen_of(&dungeon_won(ManaColor::Gold), &catalog)
            .reward_die
            .as_deref(),
        Some("Dé de récompense : or, un sort")
    );
    assert_eq!(
        screen_of(&Adventure::default().state(), &catalog).reward_die,
        None
    );
    let den = prepared(Adventure::default().state(), MONSTER_DEN);
    assert_eq!(
        screen_of(&den, &catalog).crystal_reward_pending.as_deref(),
        Some("Récompense en fin de tour : 2 cristal(aux) aléatoire(s)")
    );
    assert_eq!(
        screen_of(&Adventure::default().state(), &catalog).crystal_reward_pending,
        None
    );
}

// C46 — Compendium, *End of Turn* Q4 : un dé or, la couleur du cristal au choix, sauf pleine.
#[test]
fn crystal_color_window() {
    let catalog = french();
    // Le rouge plein avant le dé : un cristal gagné pendant le choix le consommerait.
    let state = gold_rolled(with_full(Adventure::default().state(), BasicColor::Red));
    let window = screen_of(&state, &catalog)
        .crystal_choice
        .expect("fenêtre du choix de la couleur");
    assert_eq!(
        window.title,
        "Récompense : choisissez la couleur du cristal (dé or)"
    );
    let choose = |color| {
        Some(Command::Play(index_of(
            &state,
            &Action::ChooseCrystalColor { color },
        )))
    };
    assert_eq!(
        window
            .colors
            .iter()
            .map(|button| button.color)
            .collect::<Vec<_>>(),
        BasicColor::ALL
    );
    assert_eq!(
        labels_and_commands(&window.colors),
        [
            ("rouge", &None),
            ("bleu", &choose(BasicColor::Blue)),
            ("blanc", &choose(BasicColor::White)),
            ("vert", &choose(BasicColor::Green)),
        ]
    );
    // Spec 071, C22 : 1 à 4, dans l'ordre des couleurs.
    assert_eq!(
        window
            .colors
            .iter()
            .map(|button| button.shortcut.as_deref())
            .collect::<Vec<_>>(),
        [Some("1"), Some("2"), Some("3"), Some("4")]
    );
    assert_eq!(
        screen_of(&Adventure::default().state(), &catalog).crystal_choice,
        None
    );
}

// Raccourcis 1 à 4 (spec 044, *Raccourcis clavier*) : 4 choisit le vert.
#[test]
fn key_four_chooses_a_green_crystal() {
    let start = Adventure::default().events();
    let mut events = events_after(start, &[sideways(card(March, 1))]);
    events.extend([
        Event::SiteConquered {
            player: SOLO,
            hex: MONSTER_DEN,
        },
        Event::TurnEnded { player: SOLO },
        Event::RewardDieRolled {
            player: SOLO,
            color: ManaColor::Gold,
        },
    ]);
    let mut session = session_of(events);
    session.shortcut(Shortcut::Take(4));
    session.run_frame();
    let state = session.runner().state();
    assert_eq!(state.crystal_choices(SOLO), 0);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Green), 1);
}

// --- Clairière (C47, C48) ---

// C47 — carte de la clairière ; Q5 : trois choix, ceux sans Blessure grisés, la main par défaut ;
// la pioche suit le choix.
#[test]
fn end_turn_glade_choice() {
    let catalog = french();
    let state = marched(&glade_with_wound().state());
    let prompt = screen_in(&state, &ending(None), &catalog)
        .end_turn_prompt
        .expect("mode fin du tour");
    let menu = prompt.glade.expect("choix de la clairière");
    assert_eq!(menu.label, "Clairière magique : jeter une Blessure");
    assert_eq!(
        menu.options,
        [
            glade_option("de la main", Some(WoundPile::Hand), true, true),
            glade_option("de la défausse", Some(WoundPile::DiscardPile), false, false),
            glade_option("ne pas jeter", None, true, false),
        ]
    );
    // Quatre cartes en main, la Blessure jetée : trois gardées, deux piochées.
    assert_eq!(prompt.draw, "Vous piocherez 2 carte(s).");
    let kept = screen_in(&state, &ending(Some(None)), &catalog)
        .end_turn_prompt
        .expect("mode fin du tour");
    assert_eq!(kept.draw, "Vous piocherez 1 carte(s).");
    let selected: Vec<bool> = kept
        .glade
        .expect("choix de la clairière")
        .options
        .iter()
        .map(|option| option.selected)
        .collect();
    assert_eq!(selected, [false, false, true]);

    // Blessure dans la défausse seulement : la défausse par défaut.
    let discarded = marched(&apply(
        glade_with_wound().state(),
        &Event::CardDiscarded {
            player: SOLO,
            card: wound(1),
        },
    ));
    let options = screen_in(&discarded, &ending(None), &catalog)
        .end_turn_prompt
        .and_then(|prompt| prompt.glade)
        .expect("choix de la clairière")
        .options;
    assert_eq!(
        options,
        [
            glade_option("de la main", Some(WoundPile::Hand), false, false),
            glade_option("de la défausse", Some(WoundPile::DiscardPile), true, true),
            glade_option("ne pas jeter", None, true, false),
        ]
    );
    // Hors d'une clairière, pas de ligne.
    let elsewhere = marched(
        &Adventure {
            figure: NEAR_DUNGEON,
            ..glade_with_wound()
        }
        .state(),
    );
    assert_eq!(
        screen_in(&elsewhere, &ending(None), &catalog)
            .end_turn_prompt
            .and_then(|prompt| prompt.glade),
        None
    );
}

// C47 — la fin du tour envoie le choix : par défaut la main ; « ne pas jeter », rien.
#[test]
fn end_turn_sends_the_glade_choice() {
    let thrown = |choice: Option<Option<WoundPile>>| {
        let mut session = session_of(events_after(
            glade_with_wound().events(),
            &[sideways(card(March, 1))],
        ));
        session.command(Command::OpenEndTurn);
        if let Some(choice) = choice {
            session.command(Command::ChooseGlade(choice));
        }
        session.command(Command::ConfirmEndTurn);
        session.run_frame();
        session
            .runner()
            .log()
            .entries()
            .iter()
            .find_map(|logged| match logged.event {
                Event::WoundThrownAway { from, .. } => Some(from),
                _ => None,
            })
    };
    assert_eq!(thrown(None), Some(WoundPile::Hand));
    assert_eq!(thrown(Some(None)), None);
}

// C48 — carte de la clairière : le jeton du début du tour s'annonce sous les points, jusqu'à la
// première action.
#[test]
fn panel_announces_glade_token() {
    let catalog = french();
    for (time, text) in [
        (TimeOfDay::Day, "Clairière magique : +1 jeton de mana or"),
        (
            TimeOfDay::Night,
            "Clairière magique : +1 jeton de mana noir",
        ),
    ] {
        let state = marched(
            &Adventure {
                figure: GLADE,
                time,
                ..Adventure::default()
            }
            .state(),
        );
        let next = done(
            &state,
            Action::EndTurn {
                discard: Vec::new(),
                reclaim: None,
                glade: None,
                steady_tempo: false,
            },
        );
        assert_eq!(
            screen_of(&next, &catalog).glade_token.as_deref(),
            Some(text)
        );
        let card: CardId = next
            .hand(SOLO)
            .iter()
            .copied()
            .find(|card| !card.card().is_wound())
            .expect("une carte à jouer");
        let played = done(&next, sideways(card));
        assert_eq!(screen_of(&played, &catalog).glade_token, None);
    }
    let elsewhere = marched(&Adventure::default().state());
    let next = done(
        &elsewhere,
        Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    );
    assert_eq!(screen_of(&next, &catalog).glade_token, None);
}

// --- Décompte (C49) ---

// C49 — MKUE p. 15, *The Greatest Adventurer* : L'Aventurier compte ses points.
#[test]
fn final_window_shows_adventurer() {
    let catalog = french();
    let state = Adventure {
        conquered: vec![DUNGEON, MONSTER_DEN],
        ..Adventure::default()
    }
    .state();
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
    let window = screen.score_window.expect("fenêtre de fin");
    assert!(
        window.lines.contains(&"L'Aventurier : 4".to_string()),
        "{:?}",
        window.lines
    );
}

// Issue privée 127 : chaque libellé de la spec 044 existe en français.
#[test]
fn every_adventure_key_exists_in_french() {
    let keys = french().keys();
    for key in [
        "adventure.enter",
        "adventure.lost_points",
        "adventure.confirm.fight_drawn",
        "adventure.confirm.fight_waiting",
        "adventure.confirm.night_no_units",
        "adventure.confirm.reward",
        "adventure.confirm.no_undo",
        "adventure.confirm_yes",
        "adventure.cancel",
        "adventure.reward.dungeon",
        "adventure.reward.tomb",
        "adventure.reward.monster_den",
        "adventure.reward.spawning_grounds",
        "adventure.reward.fame_only",
        "site.conquered.adventure",
        "board.hover.adventure.enemies_waiting",
        "board.hover.adventure.fight",
        "board.hover.glade",
        "combat.night_rules",
        "reward.die",
        "reward.die_spell",
        "reward.die_artifact",
        "reward.pending_crystals",
        "reward.choose_crystal",
        "end_turn.glade",
        "end_turn.glade_hand",
        "end_turn.glade_discard",
        "end_turn.glade_none",
        "glade.token",
    ] {
        assert!(keys.iter().any(|known| known == key), "{key}");
    }
}

// Spec 071, C22 : rouge, bleu, blanc, vert disent 1 à 4, grisé compris.
#[test]
fn crystal_colors_show_shortcut() {
    let catalog = french();
    let state = gold_rolled(with_full(Adventure::default().state(), BasicColor::Red));
    let window = screen_of(&state, &catalog)
        .crystal_choice
        .expect("fenêtre du choix de la couleur");
    let keys: Vec<Option<&str>> = window
        .colors
        .iter()
        .map(|button| button.shortcut.as_deref())
        .collect();
    assert_eq!(keys, [Some("1"), Some("2"), Some("3"), Some("4")]);
}
