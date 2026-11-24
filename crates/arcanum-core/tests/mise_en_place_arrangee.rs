//! Outil des parcours (issue privée 318) : la mise en place arrangée garde la vraie mise en place de
//! `new_game`, avec l'ordre de certaines piles choisi par le test.

mod support;

use arcanum_core::{
    CommonCard, CommonDeck, EnemyColor, Event, GameConfig, GameRunner, Hero, Offer, Scenario,
};
use support::Parcours;
use support::arrangement::Setup;

fn config() -> GameConfig {
    GameConfig {
        seed: 416,
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

fn events(log: &arcanum_core::EventLog) -> Vec<Event> {
    log.entries()
        .iter()
        .map(|entry| entry.event.clone())
        .collect()
}

#[test]
fn an_unarranged_setup_is_the_real_one() {
    let runner = GameRunner::new(config(), Vec::new());
    let (log, positions) = Setup::new(config()).save();
    assert_eq!(events(&log), events(runner.log()));
    assert_eq!(positions, runner.rng_positions());
}

#[test]
fn an_arranged_deck_gives_the_chosen_starting_hand() {
    let setup = Setup::new(config());
    let order = setup
        .events()
        .iter()
        .find_map(|event| match event {
            Event::DeckShuffled { order, .. } => Some(order.clone()),
            _ => None,
        })
        .expect("le deck est mélangé");
    // Les cinq dernières du mélange, qu'aucune graine ne garantit en main.
    let wanted: Vec<_> = order.iter().rev().take(5).copied().collect();
    let parcours = Parcours::arranged(&setup.deck(&wanted));
    let view = parcours.view();
    assert_eq!(view.hand, wanted);
    assert_eq!(view.deck_size, order.len() - 5);
}

#[test]
fn an_arranged_offer_shows_the_chosen_card() {
    let setup = Setup::new(config());
    let spells = common_order(&setup, CommonDeck::Spells);
    // La sixième du mélange : hors de l'offre de départ, qui en a trois.
    let wanted = spells[5];
    let parcours = Parcours::arranged(&setup.offer(CommonDeck::Spells, &[wanted]));
    let view = parcours.view();
    assert!(view.offers[&Offer::Spells].contains(&wanted));
    assert_eq!(view.offers[&Offer::Spells].len(), 3);
}

#[test]
fn next_cards_come_after_the_dealt_offer() {
    let setup = Setup::new(config());
    let spells = common_order(&setup, CommonDeck::Spells);
    let wanted = spells[spells.len() - 1];
    let arranged = setup.next_cards(CommonDeck::Spells, &[wanted]);
    assert_eq!(common_order(&arranged, CommonDeck::Spells)[3], wanted);
    let mut offer = Parcours::arranged(&arranged).view().offers[&Offer::Spells].clone();
    offer.sort();
    let mut dealt = spells[..3].to_vec();
    dealt.sort();
    assert_eq!(offer, dealt);
}

#[test]
fn next_enemies_keep_what_the_setup_drew() {
    let setup = Setup::new(config());
    let placed = placed_tokens(&setup);
    let green = enemy_order(&setup, EnemyColor::Green);
    let wanted = green[green.len() - 1];
    let arranged = setup.next_enemies(EnemyColor::Green, &[wanted]);
    assert_eq!(placed_tokens(&arranged), placed);
    let first_undrawn = green
        .iter()
        .position(|token| !placed.iter().any(|(t, _)| t == token))
        .expect("la pile verte n'est pas vide");
    assert_eq!(
        enemy_order(&arranged, EnemyColor::Green)[first_undrawn],
        wanted
    );
}

#[test]
fn arranged_enemies_are_the_first_drawn_at_setup() {
    let setup = Setup::new(config());
    let green = enemy_order(&setup, EnemyColor::Green);
    let first_placed = placed_tokens(&setup)[0];
    assert_eq!(first_placed.0, green[0], "la mise en place tire du dessus");
    let wanted = green[green.len() - 1];
    let arranged = setup.enemies(EnemyColor::Green, &[wanted]);
    assert_eq!(placed_tokens(&arranged)[0], (wanted, first_placed.1));
    let parcours = Parcours::arranged(&arranged);
    assert_eq!(parcours.view().enemy_tokens[&first_placed.1], [wanted]);
}

#[test]
fn arranged_skills_are_on_top_of_the_pile() {
    let setup = Setup::new(config());
    let order = skill_order(&setup);
    let wanted = [order[4], order[7]];
    let arranged = setup.skills(&wanted);
    assert_eq!(skill_order(&arranged)[..2], wanted);
}

fn common_order(setup: &Setup, wanted: CommonDeck) -> Vec<CommonCard> {
    setup
        .events()
        .iter()
        .find_map(|event| match event {
            Event::CommonDeckShuffled { deck, order } if *deck == wanted => Some(order.clone()),
            _ => None,
        })
        .expect("pile mélangée")
}

fn enemy_order(setup: &Setup, wanted: EnemyColor) -> Vec<arcanum_core::EnemyToken> {
    setup
        .events()
        .iter()
        .find_map(|event| match event {
            Event::EnemyPileShuffled { color, order } if *color == wanted => Some(order.clone()),
            _ => None,
        })
        .expect("pile mélangée")
}

fn skill_order(setup: &Setup) -> Vec<arcanum_core::Skill> {
    setup
        .events()
        .iter()
        .find_map(|event| match event {
            Event::SkillPileShuffled { order, .. } => Some(order.clone()),
            _ => None,
        })
        .expect("pile mélangée")
}

fn placed_tokens(setup: &Setup) -> Vec<(arcanum_core::EnemyToken, arcanum_core::Hex)> {
    setup
        .events()
        .iter()
        .filter_map(|event| match event {
            Event::EnemyTokenPlaced { token, hex } | Event::DefenderPlaced { token, hex } => {
                Some((*token, *hex))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn deck_cards_take_one_copy_of_each() {
    use arcanum_core::{BasicAction, DeedCard};
    let march = DeedCard::Basic(BasicAction::March);
    let rage = DeedCard::Basic(BasicAction::Rage);
    let parcours = Parcours::arranged(&Setup::new(config()).deck_cards(&[march, march, rage]));
    let hand: Vec<DeedCard> = parcours
        .view()
        .hand
        .iter()
        .map(|card| card.card())
        .collect();
    assert_eq!(hand[..3], [march, march, rage]);
    assert_ne!(parcours.view().hand[0], parcours.view().hand[1]);
}

#[test]
fn the_figure_starts_on_the_chosen_hex() {
    let glade = arcanum_core::Hex::new(4, -3);
    let parcours = Parcours::arranged(&Setup::new(config()).figure(glade));
    assert_eq!(parcours.view().figure, Some(glade));
}
