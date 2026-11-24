//! Spec 054 (sorts), sauvegarde : C32. Un sort lancé (ennemi visé, mana noir) et un ennemi détruit
//! survivent à la sauvegarde et au rechargement. Spec 057 : C28.

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, CastTargets, DeedCard, Event, GameState,
    ManaColor, ManaPayment, PlayerId, Spell, TimeOfDay, apply, decide, legal_actions,
};
use arcanum_persistance::{SaveGame, load, save};

#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

fn play(events: &mut Vec<Event>, state: &mut GameState, action: &Action) {
    let produced = decide(state, PlayerId::SOLO, action, &mut config(1).rngs())
        .unwrap_or_else(|error| panic!("{action:?} : {error:?}"));
    *state = produced.iter().fold(state.clone(), apply);
    events.extend(produced);
}

// C32 — spec 024 : le sort lancé et l'ennemi détruit, identiques après rechargement.
#[test]
fn destroyed_enemy_survives_save_and_reload() {
    let whirlwind = CardId::new(DeedCard::Spell(Spell::Whirlwind), 1);
    let mut events = Sandbox {
        time: TimeOfDay::Night,
        figure: Some(PLAIN_W),
        hand: vec![whirlwind],
        placed: vec![(prowlers(), ORCS_2)],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::CrystalGained {
        player: SOLO,
        color: BasicColor::White,
    });
    events.push(Event::ManaTokenGained {
        player: SOLO,
        color: ManaColor::Black,
    });
    let mut state = events.iter().fold(GameState::default(), apply);
    for action in [
        challenge(ORCS_2),
        Action::StartCombat,
        Action::NextCombatPhase,
        Action::NextCombatPhase,
        Action::PlayCard {
            card: whirlwind,
            play: CardPlay::Cast {
                option: 0,
                mana: ManaPayment::Crystal,
                black: Some(ManaPayment::Token),
                extra_mana: None,
                enemy: Some(prowlers()),
                from_offer: false,
                targets: Default::default(),
            },
        },
    ] {
        play(&mut events, &mut state, &action);
    }
    assert!(events.contains(&Event::EnemyDestroyed {
        player: SOLO,
        enemy: prowlers(),
    }));

    let (runner, _) = runner(events);
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(&runner, "2026-11-12T00:00:00+00:00").expect("partie");
    let path = save(dir.path(), "sorts", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(Puppet::default())])
        .expect("partie reprise");
    assert_eq!(reloaded.state(), runner.state());
}

fn cast(card: CardId, option: u8, black: bool, targets: CastTargets) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option,
            mana: ManaPayment::Crystal,
            black: black.then_some(ManaPayment::Token),
            extra_mana: None,
            enemy: None,
            from_offer: false,
            targets,
        },
    }
}

// Spec 057 C28 — spec 024 : un tour supplémentaire, un dé gardé et une carte écartée, identiques
// après rechargement ; le Voyage souterrain se reprend.
#[test]
fn extra_turn_and_kept_die_follow_reload() {
    let spell = |spell| CardId::new(DeedCard::Spell(spell), 1);
    let (claim, bending, underground) = (
        spell(Spell::ManaClaim),
        spell(Spell::SpaceBending),
        spell(Spell::UndergroundTravel),
    );
    let mut events = Sandbox {
        time: TimeOfDay::Night,
        figure: Some(PLAIN_W),
        hand: vec![claim, bending, underground, card(BasicAction::March, 1)],
        dice: vec![
            ManaColor::Basic(BasicColor::Blue),
            ManaColor::Basic(BasicColor::Red),
            ManaColor::Black,
        ],
        ..Sandbox::default()
    }
    .events();
    for color in [BasicColor::Blue, BasicColor::Blue, BasicColor::Green] {
        events.push(Event::CrystalGained {
            player: SOLO,
            color,
        });
    }
    events.push(Event::ManaTokenGained {
        player: SOLO,
        color: ManaColor::Black,
    });
    let (mut runner, puppet) = runner(events);
    arene::play(
        &mut runner,
        &puppet,
        cast(
            claim,
            0,
            false,
            CastTargets {
                die: Some(RED_DIE),
                ..CastTargets::default()
            },
        ),
    );
    arene::play(
        &mut runner,
        &puppet,
        cast(bending, 0, true, CastTargets::default()),
    );
    let end_turn = legal_actions(runner.state(), SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("fin du tour");
    arene::play(&mut runner, &puppet, end_turn);
    arene::play(
        &mut runner,
        &puppet,
        cast(
            underground,
            0,
            false,
            CastTargets {
                destination: Some(VILLAGE),
                ..CastTargets::default()
            },
        ),
    );
    let state = runner.state();
    assert!(state.extra_turn(SOLO));
    assert_eq!(state.kept_die(SOLO), Some(RED_DIE));
    assert_eq!(state.set_aside(SOLO), [bending]);
    assert_eq!(state.figure(SOLO), Some(VILLAGE));

    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(&runner, "2026-11-13T00:00:00+00:00").expect("partie");
    let path = save(dir.path(), "sorts-057", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(Puppet::default())])
        .expect("partie reprise");
    assert_eq!(reloaded.state(), runner.state());
    runner.undo().expect("le Voyage souterrain se reprend");
    assert_eq!(runner.state().figure(SOLO), Some(PLAIN_W));
}
