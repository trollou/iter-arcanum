//! Spec 050 (offres et cartes gagnées), sauvegarde : C22.

use arcanum_core::{
    Action, CardId, CommonDeck, Event, GameState, Hero, Offer, PlayerId, apply, decide,
    legal_actions, new_game,
};
use arcanum_persistance::{SaveGame, load, save};

#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

/// Joue `action`, qui doit être légale, en ajoutant ses événements au journal.
fn play(events: &mut Vec<Event>, state: &mut GameState, action: &Action) {
    let produced = decide(state, PlayerId::SOLO, action, &mut config(1).rngs())
        .unwrap_or_else(|error| panic!("{action:?} : {error:?}"));
    *state = produced.iter().fold(state.clone(), apply);
    events.extend(produced);
}

// C22 — spec 024 : offres, pioches et deck identiques après rechargement.
#[test]
fn offers_survive_save_and_reload() {
    let config = config(1);
    let mut events = new_game(&config, &mut config.rngs());
    // Deux cartes en main, deck vide : la manche se termine, les offres se renouvellent.
    let kept: Vec<CardId> = Hero::Tovak.starting_deck()[..2].to_vec();
    events.push(Event::DeckShuffled {
        player: SOLO,
        order: kept.clone(),
    });
    events.extend(
        kept.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    let mut state = events.iter().fold(GameState::default(), apply);
    play(&mut events, &mut state, &Action::AnnounceEndOfRound);
    // Niveau 2 à la fin du tour suivant : une action avancée gagnée.
    let fame = Event::EnemiesDefeated {
        player: SOLO,
        enemies: Vec::new(),
        fame: 3,
    };
    state = apply(state, &fame);
    events.push(fame);
    let first = state.hand(SOLO)[0];
    play(&mut events, &mut state, &sideways(first));
    let end_turn = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal");
    play(&mut events, &mut state, &end_turn);
    // Spec 051 : la compétence du niveau 2 d'abord.
    let skill = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::ChooseSkill { .. }))
        .expect("une compétence à choisir");
    play(&mut events, &mut state, &skill);
    let card = state.offer(Offer::AdvancedActions)[1]
        .deed()
        .expect("une action avancée");
    play(
        &mut events,
        &mut state,
        &Action::GainAdvancedAction { card },
    );
    assert!(state.hand(SOLO).contains(&card));

    let (runner, _) = runner(events);
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(&runner, "2026-10-31T00:00:00+00:00").expect("partie");
    let path = save(dir.path(), "offres", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(Puppet::default())])
        .expect("partie reprise");
    let (before, after) = (runner.state(), reloaded.state());
    assert_eq!(after, before);
    for offer in Offer::ALL {
        assert_eq!(after.offer(offer), before.offer(offer));
    }
    for deck in CommonDeck::ALL {
        assert_eq!(after.common_deck(deck), before.common_deck(deck));
    }
    assert_eq!(after.deck(SOLO), before.deck(SOLO));
}
