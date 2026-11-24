//! Scénario de la spec 044 (Sites d'aventure) côté sauvegarde.

use arcanum_core::{Event, EventLog, GameRunner, GameState, ManaColor, RngPositions};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

/// La partie de ces événements, sauvegardée puis rechargée : son état avant et après.
fn round_trip(events: Vec<Event>, name: &str) -> (GameState, GameState) {
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let runner = GameRunner::resume(log, RngPositions::new(), Vec::new()).expect("journal valide");
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game =
        SaveGame::from_runner(&runner, "2026-11-05T20:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), name, &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    (runner.state().clone(), reloaded.state().clone())
}

// C35 — spec 024 : jeton resté sur un antre, deux sur un Sombre Terrier, donjon conquis, cristaux
// aléatoires et choix de couleur en attente ; puis un combat de donjon en cours.
#[test]
fn adventure_sites_survive_save_and_load() {
    let mut events = Adventure {
        figure: SPAWNING_GROUNDS,
        placed: vec![(gargoyle(), MONSTER_DEN), (minotaur(), SPAWNING_GROUNDS)],
        brown: vec![medusa()],
        conquered: vec![DUNGEON],
        ..Adventure::default()
    }
    .events();
    events.push(Event::EnemyTokenPlaced {
        token: medusa(),
        hex: SPAWNING_GROUNDS,
    });
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: MONSTER_DEN,
    });
    events.push(Event::TurnEnded { player: SOLO });
    events.push(Event::RewardDieRolled {
        player: SOLO,
        color: ManaColor::Gold,
    });
    let (before, after) = round_trip(events, "aventure");
    assert_eq!(after, before);
    assert_eq!(after.enemy_tokens_at(MONSTER_DEN), [gargoyle()]);
    assert_eq!(
        after.enemy_tokens_at(SPAWNING_GROUNDS),
        [minotaur(), medusa()]
    );
    assert_eq!(after.shields().get(&DUNGEON), Some(&SOLO));
    assert_eq!(
        (after.crystal_rewards(SOLO), after.crystal_choices(SOLO)),
        (1, 1)
    );

    // Combat de donjon en cours, règles de Nuit.
    let mut events = Adventure::default().events();
    events.push(Event::AdventureSiteEntered {
        player: SOLO,
        hex: DUNGEON,
    });
    events.push(Event::EnemyTokenPlaced {
        token: minotaur(),
        hex: DUNGEON,
    });
    events.push(Event::CombatStarted {
        player: SOLO,
        enemies: vec![(DUNGEON, minotaur())],
    });
    let (before, after) = round_trip(events, "donjon");
    assert_eq!(after, before);
    assert_eq!(after.adventure(SOLO), Some(DUNGEON));
    assert_eq!(
        after.combat_time_of_day(SOLO),
        arcanum_core::TimeOfDay::Night
    );
}
