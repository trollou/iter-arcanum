//! Scénario de la spec 043 (Sites fortifiés) côté sauvegarde.

use arcanum_core::{
    Assault, CardId, DeedCard, Event, EventLog, GameRunner, GameState, Hex, RngPositions, Spell,
    TileId, TilePosition,
};
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
        SaveGame::from_runner(&runner, "2026-11-04T20:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), name, &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    (runner.state().clone(), reloaded.state().clone())
}

// C34 — spec 024 : défenseurs face cachée et retourné, forteresse possédée, récompense sort en
// attente ; puis un assaut en cours.
#[test]
fn fortified_sites_survive_save_and_load() {
    // La campagne 10 en 2A + B, sa forteresse possédée ; Gardes retournés, Mages de feu cachés.
    let owned = Hex::new(6, -6);
    let spells: Vec<CardId> = [Spell::Fireball, Spell::Snowstorm, Spell::Expose]
        .map(|spell| CardId::new(DeedCard::Spell(spell), 1))
        .to_vec();
    let mut events = Fortified {
        keep: Some((guardsmen(), true)),
        figure: MAGE_TOWER,
        tower: None,
        spells,
        ..Fortified::default()
    }
    .events();
    events.push(Event::TileRevealed {
        tile: TileId::Countryside(10),
        position: TilePosition::from_lattice(2, 1),
    });
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: owned,
    });
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: MAGE_TOWER,
    });
    events.push(Event::TurnEnded { player: SOLO });
    let (before, after) = round_trip(events, "forteresse");
    assert_eq!(after, before);
    assert!(!after.is_face_down(KEEP));
    assert_eq!(after.shields().get(&owned), Some(&SOLO));
    assert_eq!(after.spell_choices(SOLO), 1);

    // Assaut en cours : moment « à engager », la tour face cachée.
    let mut events = Fortified {
        figure: NEAR_KEEP,
        ..Fortified::default()
    }
    .events();
    events.push(Event::HeroMoved {
        player: SOLO,
        from: NEAR_KEEP,
        to: KEEP,
        cost: 3,
    });
    events.push(Event::AssaultStarted {
        player: SOLO,
        site: KEEP,
        from: NEAR_KEEP,
    });
    let (before, after) = round_trip(events, "assaut");
    assert_eq!(after, before);
    assert!(after.is_face_down(MAGE_TOWER));
    assert_eq!(
        after.assault(SOLO),
        Some(Assault {
            site: KEEP,
            from: NEAR_KEEP,
        })
    );
}
