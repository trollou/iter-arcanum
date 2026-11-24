//! Parcours de bout en bout du jalon 2 (spec 028, ADR 0006) : une partie entière du bac à sable,
//! jouée par un joueur scripté, de la mise en place à la fin de la quatrième manche.

mod support;

use arcanum_core::{
    Action, BasicAction, BasicColor, CardPlay, Event, GameConfig, Hero, Hex, ManaColor,
    ManaPayment, PlayerView, Scenario, TilePosition, TimeOfDay, UndoError, UndoStatus, routes,
};
use support::{Parcours, end_turn, has, in_hand, sideways_move, walk_to};

/// Graine choisie pour que les préconditions du parcours tiennent (spec 028, « Graine et
/// préconditions »). Rechoisie une dernière fois pour la spec 038 (73 → 13) : chaque usage du
/// hasard a désormais son générateur, et un usage ajouté par une spec ne décale plus les autres.
/// Si un test échoue sur « graine à rechoisir », c'est qu'une spec a ajouté un tirage à un usage
/// existant (Source, deck…) : en chercher une autre.
const SEED: u64 = 13;

/// Le village de la tuile de campagne 1, d'où l'on explore au tour 1 (plateau du bac à sable :
/// tuiles de campagne dans l'ordre des numéros, MKUE p. 18).
const VILLAGE: Hex = Hex::new(2, -3);
/// La mine verte de la tuile de campagne 2 (S-O de son centre).
const GREEN_MINE: Hex = Hex::new(2, -1);
/// Emplacement explorable depuis le village, dans la forme en coin.
const EXPLORED: TilePosition = TilePosition::from_lattice(1, 1);

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        // Le héros par défaut de l'écran de nouvelle partie (spec 065).
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Spec 028, L2 : une précondition qui ne tient plus vient de la graine, pas des règles.
fn precondition(holds: bool, what: &str) {
    assert!(holds, "graine à rechoisir : {what}");
}

/// Le dé de la Source qui paie cette action, s'il y en a un.
fn die_paid(action: &Action) -> Option<u8> {
    match action {
        Action::PlayCard {
            play:
                CardPlay::Strong {
                    mana: ManaPayment::Die(die),
                    ..
                }
                | CardPlay::Basic {
                    mana: Some(ManaPayment::Die(die)),
                    ..
                },
            ..
        } => Some(*die),
        _ => None,
    }
}

fn strong(mana: ManaPayment) -> CardPlay {
    CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option: 0,
        discard: None,
        mana,
        set_dice: [None, None],
        with: None,
    }
}

/// Manche 1, tour 1 : C1 à C5.
fn day_turn_with_a_die(parcours: &mut Parcours) {
    let start = parcours.view();
    let source = start.source.clone();
    let paid = [BasicAction::March, BasicAction::Stamina]
        .into_iter()
        .filter_map(|action| in_hand(&start, action))
        .find_map(|card| {
            let color = card.card().definition()?.color;
            source
                .iter()
                .position(|die| die.can_pay(color, TimeOfDay::Day))
                .map(|die| (card, die as u8))
        });
    precondition(
        paid.is_some(),
        "Marche ou Endurance en main avec un dé de sa couleur (ou or) au tour 1",
    );
    let Some((card, die)) = paid else { return };
    let action = Action::PlayCard {
        card,
        play: strong(ManaPayment::Die(die)),
    };

    // C1 — MKUE p. 5, Using Mana 3.a ; spec 022 : un dé de la Source paie l'effet renforcé.
    let events = parcours.play(action.clone());
    assert!(has(&events, |e| matches!(e, Event::DieTaken { .. })));
    let after = parcours.view();
    assert_eq!(after.move_points, 4);
    assert_eq!(after.dice_taken, vec![die]);
    assert!(
        parcours.legal().iter().all(|a| die_paid(a).is_none()),
        "un seul dé par tour"
    );

    // C2 — MKUE p. 5, Reverting 1 ; spec 024 : la carte revient en main, le dé dans la Source.
    assert_eq!(parcours.undo(), Ok(()));
    let reverted = parcours.view();
    assert!(reverted.hand.contains(&card));
    assert!(reverted.dice_taken.is_empty());
    assert_eq!(reverted.move_points, 0);
    assert_eq!(reverted, start);
    parcours.play(action);
    assert_eq!(parcours.view(), after);

    // Le reste de la main de côté : 4 + 4 = 8 points, 6 pour le village et 2 pour explorer.
    while !parcours.view().hand.is_empty() {
        sideways_move(parcours);
    }
    assert_eq!(parcours.view().move_points, 8);

    // C3 — MKUE p. 5, Reverting 1 ; spec 024 : un déplacement se reprend.
    let view = parcours.view();
    let route = routes(&view.map, Hex::ORIGIN, TimeOfDay::Day)
        .into_iter()
        .find(|route| route.to == VILLAGE)
        .expect("le village est atteignable");
    assert_eq!(route.cost, 6);
    parcours.play(Action::MoveHero { to: route.steps[0] });
    assert_eq!(parcours.undo(), Ok(()));
    assert_eq!(parcours.view(), view);
    assert_eq!(walk_to(parcours, VILLAGE), 6);

    // C4 — MKUE p. 5, Reverting 2 ; spec 024 : on ne revient pas avant une tuile révélée.
    let events = parcours.play(Action::Explore { position: EXPLORED });
    assert!(has(&events, |e| matches!(e, Event::TileRevealed { .. })));
    assert_eq!(
        parcours.runner.undo_status(),
        UndoStatus::InformationRevealed
    );
    assert_eq!(parcours.undo(), Err(UndoError::InformationRevealed));

    // C5 — MKUE p. 5, Using Mana 3.b ; p. 11, 1 ; specs 016, 022, 024 : le dé est rendu et relancé.
    let events = end_turn(parcours);
    assert!(has(&events, |e| matches!(e, Event::DieReturned { .. })));
    let next = parcours.view();
    assert!(next.dice_taken.is_empty());
    assert_eq!(next.source.len(), 3);
    assert_eq!(next.hand.len(), 5);
    assert_eq!(next.turn, 2);
    assert_eq!(parcours.undo(), Err(UndoError::NothingSinceTurnStart));
}

/// Manche 1, tour 2 : C7 (jeton) puis C6 (cristal sur la mine).
fn day_turn_with_a_token(parcours: &mut Parcours) {
    let start = parcours.view();
    let concentration = in_hand(&start, BasicAction::Concentration);
    let stamina = in_hand(&start, BasicAction::Stamina);
    precondition(
        concentration.is_some() && stamina.is_some(),
        "Concentration et Endurance en main au tour 2",
    );
    let (Some(concentration), Some(stamina)) = (concentration, stamina) else {
        return;
    };

    // C7 — MKUE p. 5, "Gain" Effects 1 ; spec 025 : le jeton gagné paie l'effet renforcé.
    parcours.play(Action::PlayCard {
        card: concentration,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        },
    });
    assert_eq!(
        parcours.view().mana_tokens,
        vec![arcanum_core::ManaColor::Basic(BasicColor::Blue)]
    );
    let events = parcours.play(Action::PlayCard {
        card: stamina,
        play: strong(ManaPayment::Token),
    });
    assert!(has(&events, |e| matches!(e, Event::ManaTokenUsed { .. })));
    assert_eq!(parcours.view().move_points, 4);
    assert!(parcours.view().mana_tokens.is_empty());

    // Village → mine : 7 points. Le trajet à 5 points, du village droit à la plaine (2,−2), longe
    // les orques de la tuile 2 et les provoquerait ; il fait le détour par (1,−2) (spec 030, C19).
    while parcours.view().move_points < 7 && !parcours.view().hand.is_empty() {
        match in_hand(&parcours.view(), BasicAction::March) {
            Some(march) => {
                parcours.play(Action::PlayCard {
                    card: march,
                    play: CardPlay::Basic {
                        die: None,
                        gain: None,
                        with: None,
                        option: 0,
                        discard: None,
                        mana: None,
                    },
                });
            }
            None => sideways_move(parcours),
        }
    }
    precondition(
        parcours.view().move_points >= 7,
        "7 points de mouvement au tour 2 pour aller du village à la mine",
    );
    assert_eq!(walk_to(parcours, GREEN_MINE), 7);

    // C6 — MKUE p. 11, End of the Turn 4.b ; spec 021 : un cristal vert en finissant sur la mine.
    let events = end_turn(parcours);
    assert!(has(&events, |e| matches!(
        e,
        Event::CrystalGained {
            color: BasicColor::Green,
            ..
        }
    )));
    let next = parcours.view();
    assert_eq!(next.inventory.crystals(BasicColor::Green), 1);
    // C7 — MKUE p. 11, End of the Turn 3.a : aucun jeton ne reste après la fin du tour.
    assert!(next.mana_tokens.is_empty());
}

/// Fin de la manche 1 et début de la manche 2 : C8, C9.
fn into_the_night(parcours: &mut Parcours) {
    // C8 — MKUE p. 6, 3 et 4 ; p. 4, One Round 2 ; spec 023.
    let events = parcours.finish_round();
    assert!(has(&events, |e| matches!(
        e,
        Event::RoundEnded { round: 1 }
    )));
    let night = parcours.view();
    assert_eq!(night.round, 2);
    assert_eq!(night.time_of_day, TimeOfDay::Night);
    assert_eq!(night.turn, 1);
    assert_eq!(night.hand.len(), 5);
    assert_eq!(night.deck_size, 11);
    assert!(night.discard.is_empty());

    // C9 — MKUE p. 4, One Round 2.b ; p. 5, Using Mana 3.c ; specs 020, 023.
    assert_eq!(night.source.len(), 3);
    precondition(
        night.source.contains(&ManaColor::Gold),
        "un dé or dans la Source de la manche 2",
    );
    assert!(
        parcours
            .legal()
            .iter()
            .filter_map(die_paid)
            .all(|die| night.source[usize::from(die)] != ManaColor::Gold),
        "un dé or est épuisé de nuit"
    );
    assert!(night.inventory.crystals(BasicColor::Green) >= 1);
}

/// Manche 2 : C10, jusqu'au point de sauvegarde de C12 (une carte jouée en milieu de tour). Rend
/// la vue d'avant la carte jouée.
fn night_turn_with_a_crystal(parcours: &mut Parcours) -> PlayerView {
    while in_hand(&parcours.view(), BasicAction::March).is_none() {
        parcours.filler_turn();
        precondition(
            parcours.view().round == 2,
            "Marche en main pendant la manche 2",
        );
    }
    let before = parcours.view();
    let march = in_hand(&before, BasicAction::March).expect("vérifiée juste avant");

    // C10 — MKUE p. 5, Using Mana 1.b ; spec 022 : le cristal gagné de jour paie de nuit.
    let events = parcours.play(Action::PlayCard {
        card: march,
        play: strong(ManaPayment::Crystal),
    });
    assert!(has(&events, |e| matches!(e, Event::CrystalUsed { .. })));
    let after = parcours.view();
    assert_eq!(after.move_points, before.move_points + 4);
    assert_eq!(
        after.inventory.crystals(BasicColor::Green),
        before.inventory.crystals(BasicColor::Green) - 1
    );
    before
}

/// Le parcours de la mise en place jusqu'au point de sauvegarde (C1 à C10), et la vue d'avant la
/// dernière carte jouée.
fn up_to_the_save_point(seed: u64) -> (Parcours, PlayerView) {
    let mut parcours = Parcours::new(config(seed));
    day_turn_with_a_die(&mut parcours);
    day_turn_with_a_token(&mut parcours);
    into_the_night(&mut parcours);
    let before_march = night_turn_with_a_crystal(&mut parcours);
    (parcours, before_march)
}

/// Tours de remplissage jusqu'à la fin de la partie, en suivant l'alternance des manches.
fn finish_game(parcours: &mut Parcours) {
    while !parcours.is_finished() {
        let view = parcours.view();
        let expected = if view.round % 2 == 1 {
            TimeOfDay::Day
        } else {
            TimeOfDay::Night
        };
        assert_eq!(view.time_of_day, expected, "manche {}", view.round);
        parcours.filler_turn();
    }
}

// C1 à C11 : un seul parcours, de la mise en place à la fin de partie.
#[test]
fn parcours_quatre_manches_avec_la_mana() {
    let (mut parcours, _) = up_to_the_save_point(SEED);
    finish_game(&mut parcours);

    // C11 — MKUE p. 4, One Round 5 ; spec 023 : la quatrième manche finie, plus de décision.
    let end = parcours.view();
    assert_eq!(end.round, 4);
    assert_eq!(end.time_of_day, TimeOfDay::Night);
    assert!(!end.has_pending_decision);
    assert!(parcours.legal().is_empty());
    assert!(parcours.is_finished());
}

// C12 — ADR 0001 ; spec 024 : une partie reprise finit comme celle jouée d'une traite.
#[test]
fn parcours_reprise_identique() {
    let (mut played, _) = up_to_the_save_point(SEED);
    let saved = played.save();
    finish_game(&mut played);

    let mut resumed = Parcours::resume(&saved);
    finish_game(&mut resumed);

    assert_eq!(resumed.runner.log(), played.runner.log());
    assert_eq!(resumed.runner.state(), played.runner.state());
}

// C13 — MKUE p. 5, Reverting 1 ; spec 024 : juste après la reprise, on peut revenir en arrière.
#[test]
fn parcours_annulation_apres_reprise() {
    let (played, before_march) = up_to_the_save_point(SEED);
    let saved = played.save();

    let mut resumed = Parcours::resume(&saved);
    assert_eq!(resumed.runner.undo_status(), UndoStatus::Possible);
    assert_eq!(resumed.undo(), Ok(()));
    let view = resumed.view();
    assert!(in_hand(&view, BasicAction::March).is_some());
    assert_eq!(
        view.inventory.crystals(BasicColor::Green),
        played.view().inventory.crystals(BasicColor::Green) + 1
    );
    assert_eq!(view, before_march);
}

// L1 : une action voulue absente des actions légales fait échouer le test en la nommant.
#[test]
#[should_panic(expected = "action voulue absente des actions légales")]
fn scripted_player_reports_illegal_action() {
    let mut parcours = Parcours::new(config(SEED));
    parcours.play(Action::MoveHero { to: Hex::new(5, 5) });
}
