//! Scénario de la spec 041 (Villages et monastères) côté sauvegarde.

use arcanum_core::{
    AdvancedAction, Artifact, CardId, CommonCard, CommonDeck, DeedCard, Event, EventLog,
    GameRunner, Offer, RngPositions,
};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

// C30 — spec 024 : une action avancée dans l'offre d'unités, un monastère incendié et deux
// artefacts tirés en attente du choix.
#[test]
fn monasteries_survive_save_and_load() {
    let ruby = CardId::new(DeedCard::Artifact(Artifact::RubyRing), 1);
    let amulet = CardId::new(DeedCard::Artifact(Artifact::AmuletOfSun), 1);
    let mut events = Monastery {
        monastery_cards: vec![
            CardId::new(DeedCard::Advanced(AdvancedAction::IceShield), 1),
            CardId::new(DeedCard::Advanced(AdvancedAction::FireBolt), 1),
        ],
        artifacts: vec![ruby, amulet],
        ..Monastery::default()
    }
    .events();
    events.push(Event::MonasteryBurned {
        player: SOLO,
        hex: MONASTERY,
    });
    events.push(Event::ArtifactsDrawn {
        player: SOLO,
        cards: vec![ruby, amulet],
    });
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let runner = GameRunner::resume(log, RngPositions::new(), Vec::new()).expect("journal valide");

    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game =
        SaveGame::from_runner(&runner, "2026-11-04T20:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), "monastere", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    let state = reloaded.state();
    assert_eq!(state, runner.state());
    assert_eq!(state.offer(Offer::Units).len(), 2);
    assert_eq!(state.burned_monasteries(), [MONASTERY]);
    assert_eq!(state.drawn_artifacts(SOLO), [ruby, amulet]);
    assert!(
        !state
            .common_deck(CommonDeck::Artifacts)
            .contains(&CommonCard::Deed(ruby))
    );
}
