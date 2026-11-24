//! Spec 055 (artefacts), sauvegarde et retour en arrière : C43. Une bannière assignée et retournée,
//! l'Anneau de rubis renforcé en cours, survivent au rechargement ; le retour en arrière rend la
//! bannière, pas avant un lancer de dé.

use arcanum_core::{
    Action, Artifact, ArtifactChoices, BasicAction, CardId, CardPlay, DeedCard, Event, GameState,
    PlayerId, UndoStatus, Unit, UnitCard, apply, decide,
};
use arcanum_persistance::{SaveGame, load, save};

#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

fn play_event(events: &mut Vec<Event>, state: &mut GameState, action: &Action) {
    let produced = decide(state, PlayerId::SOLO, action, &mut config(1).rngs())
        .unwrap_or_else(|error| panic!("{action:?} : {error:?}"));
    *state = produced.iter().fold(state.clone(), apply);
    events.extend(produced);
}

fn artifact_card(card: Artifact) -> CardId {
    CardId::new(DeedCard::Artifact(card), 1)
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn throw_away(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::ThrowAway {
            option: 0,
            choices: ArtifactChoices::default(),
        },
    }
}

/// Le bac à sable avec les Paysans épuisés, la Bannière de courage, l'Anneau de rubis et le Cor de
/// colère en main.
fn decor() -> (Vec<Event>, GameState) {
    let courage = artifact_card(Artifact::BannerOfCourage);
    let ruby = artifact_card(Artifact::RubyRing);
    let horn = artifact_card(Artifact::HornOfWrath);
    let mut events = Sandbox {
        figure: Some(PLAIN_W),
        hand: vec![courage, ruby, horn, card(BasicAction::March, 1)],
        placed: vec![(diggers(), ORCS_2)],
        ..Sandbox::default()
    }
    .events();
    events.extend(unit_events(&[peasants()], &[]));
    events.push(Event::UnitActivated {
        player: SOLO,
        unit: peasants(),
        ability: 0,
        option: 0,
        effect: arcanum_core::Effect::Nothing,
        enemy: None,
    });
    let state = events.iter().fold(GameState::default(), apply);
    (events, state)
}

// C43 — spec 024 : tout est rétabli au rechargement ; le retour en arrière rend la bannière, pas
// avant un lancer de dé.
#[test]
fn banners_follow_reload_and_undo() {
    let courage = artifact_card(Artifact::BannerOfCourage);
    let ruby = artifact_card(Artifact::RubyRing);
    let (mut events, mut state) = decor();
    for action in [
        Action::AssignBanner {
            card: courage,
            unit: peasants(),
        },
        Action::UseBanner {
            unit: peasants(),
            enemy: None,
        },
        throw_away(ruby),
    ] {
        play_event(&mut events, &mut state, &action);
    }
    let (runner_full, _) = runner(events.clone());
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(&runner_full, "2026-11-13T00:00:00+00:00").expect("partie");
    let path = save(dir.path(), "artefacts", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(Puppet::default())])
        .expect("partie reprise");
    assert_eq!(reloaded.state(), runner_full.state());
    let carrier = reloaded
        .state()
        .units(SOLO)
        .iter()
        .find(|owned| owned.card == peasants())
        .copied()
        .expect("Paysans");
    assert_eq!(carrier.banner, Some(courage));
    assert!(carrier.banner_flipped);

    // Retour en arrière jusqu'avant l'assignation : la bannière revient en main.
    let (events, _) = decor();
    let (mut game, puppet) = runner(events);
    play(
        &mut game,
        &puppet,
        Action::AssignBanner {
            card: courage,
            unit: peasants(),
        },
    );
    assert_eq!(game.undo(), Ok(()));
    assert!(game.state().hand(SOLO).contains(&courage));

    // Pas avant un lancer de dé (Corne de la colère).
    let horn = artifact_card(Artifact::HornOfWrath);
    let (events, _) = decor();
    let (mut game, puppet) = runner(events);
    play(&mut game, &puppet, challenge(ORCS_2));
    play(&mut game, &puppet, Action::StartCombat);
    play(&mut game, &puppet, declare(&[diggers()]));
    play(
        &mut game,
        &puppet,
        Action::PlayCard {
            card: horn,
            play: CardPlay::Artifact {
                option: 0,
                choices: ArtifactChoices::default(),
            },
        },
    );
    assert_eq!(game.undo_status(), UndoStatus::InformationRevealed);
}
