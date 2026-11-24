//! Spec 065 (Nouvelle partie), côté arcanum-core : la liste des scénarios jouables et les cartes
//! propres des héros, lues dans le contenu.

use arcanum_core::{BasicAction, DeedCard, Hero, OwnCard, Scenario, ScenarioId};

use BasicAction::*;

// MKUE p. 1, Game Flow 1 : l'ordre de l'écran de nouvelle partie.
#[test]
fn playable_scenarios_in_screen_order() {
    assert_eq!(
        ScenarioId::ALL,
        // Spec 064, Q4 : Conquête en solitaire entre les deux.
        [
            ScenarioId::FirstReconnaissance,
            ScenarioId::SoloConquest,
            ScenarioId::Sandbox
        ]
    );
}

#[test]
fn scenario_of_an_id() {
    assert_eq!(
        Scenario::of(ScenarioId::FirstReconnaissance),
        Scenario::first_reconnaissance()
    );
    assert_eq!(Scenario::of(ScenarioId::Sandbox), Scenario::sandbox());
    for id in ScenarioId::ALL {
        assert_eq!(Scenario::of(id).id(), id);
    }
}

// MKUE p. 3 ; docs/reference/decks-de-depart.md : deux cartes propres par héros, dans l'ordre du
// fichier de contenu.
#[test]
fn own_cards_of_every_hero() {
    let own = |card, replaced| OwnCard { card, replaced };
    assert_eq!(
        Hero::Arythea.own_cards(),
        vec![own(BattleVersatility, Rage), own(ManaPull, ManaDraw)]
    );
    assert_eq!(
        Hero::Goldyx.own_cards(),
        vec![own(WillFocus, Concentration), own(CrystalJoy, Crystallize)]
    );
    assert_eq!(
        Hero::Norowas.own_cards(),
        vec![own(NobleManners, Promise), own(Rejuvenate, Tranquility)]
    );
    assert_eq!(
        Hero::Tovak.own_cards(),
        vec![
            own(ColdToughness, Determination),
            own(Instinct, Improvisation)
        ]
    );
}

// Les cartes propres sont bien dans le deck de départ (MKUE p. 3).
#[test]
fn own_cards_are_in_the_starting_deck() {
    for hero in Hero::ALL {
        let deck: Vec<BasicAction> = hero
            .starting_deck()
            .into_iter()
            .filter_map(|card| match card.card() {
                DeedCard::Basic(action) => Some(action),
                _ => None,
            })
            .collect();
        for own in hero.own_cards() {
            assert!(deck.contains(&own.card), "{hero:?} {:?}", own.card);
        }
    }
}
