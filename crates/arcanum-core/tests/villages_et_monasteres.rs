//! Scénarios de la spec 041 (Villages et monastères).

use arcanum_core::{
    Action, AdvancedAction, Artifact, BasicAction, CardId, CardPlay, CommonCard, CommonDeck,
    Effect, Event, GameConfig, GameState, Hero, MapShape, Offer, Scenario, ScenarioEnd, ScenarioId,
    SidewaysEffect, Site, TileDeckRecipe, TileId, Unit, UnitCard, apply, legal_actions, new_game,
};
use arcanum_core::{CombatPhase, CountrysideOrder, EnemyColor, EnemyToken, FinalScoring, Hex};

use BasicAction::*;

mod arene;
use arene::*;

// --- Outils ---

fn legal(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
}

fn three_units() -> Vec<UnitCard> {
    vec![
        UnitCard::new(Unit::Peasants, 1),
        UnitCard::new(Unit::Herbalists, 1),
        UnitCard::new(Unit::NorthernMonks, 1),
    ]
}

fn ice_shield() -> CardId {
    advanced(AdvancedAction::IceShield)
}

fn units_offer_deeds(state: &GameState) -> Vec<CardId> {
    state
        .offer(Offer::Units)
        .iter()
        .filter_map(|card| card.deed())
        .collect()
}

/// Deck et main vidés : le joueur doit annoncer la fin de la manche (MKUE p. 6, 3.b).
fn empty_deck(state: GameState) -> GameState {
    apply(
        state,
        &Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    )
}

/// La nouvelle manche qui suit l'annonce : ses événements et l'état après.
fn new_round(state: &GameState) -> (Vec<Event>, GameState) {
    let state = empty_deck(state.clone());
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    let after = events.iter().fold(state, apply);
    (events, after)
}

/// Explorer A + B depuis le village, après une Marche (Mouvement 2).
fn explore_from_village(decor: Monastery) -> Vec<Event> {
    let state = done(&decor.state(), basic(card(March, 1)));
    events_of(
        &state,
        &Action::Explore {
            position: MONASTERY_POSITION,
        },
    )
}

fn dealt_to_units(events: &[Event]) -> Vec<CommonCard> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::OfferCardDealt {
                offer: Offer::Units,
                card,
            } => Some(*card),
            _ => None,
        })
        .collect()
}

// --- Actions avancées des monastères ---

// C1 — MKUE p. 7, 5.f : la carte du dessus de la pioche d'actions avancées entre dans l'offre
// d'unités, après la tuile révélée.
#[test]
fn revealed_monastery_adds_an_advanced_action() {
    let decor = Monastery {
        figure: VILLAGE,
        revealed: false,
        offer: three_units(),
        monastery_cards: Vec::new(),
        ..Monastery::default()
    };
    let state = done(&decor.state(), basic(card(March, 1)));
    let top = state.common_deck(CommonDeck::AdvancedActions)[0];
    let events = events_of(
        &state,
        &Action::Explore {
            position: MONASTERY_POSITION,
        },
    );
    assert_events_in_order(
        &events,
        &[
            Event::TileRevealed {
                tile: TileId::Countryside(5),
                position: MONASTERY_POSITION,
            },
            Event::OfferCardDealt {
                offer: Offer::Units,
                card: top,
            },
        ],
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.offer(Offer::Units).len(), 4);
    assert_eq!(after.offer(Offer::Units)[3], top);
    assert!(
        top.deed()
            .is_some_and(|card| card.card().is_advanced_action())
    );
}

// C2 — MKUE p. 7, 5.f : sans monastère, ou pioche d'actions avancées vide, rien n'entre.
#[test]
fn no_monastery_no_advanced_action() {
    let without = explore_from_village(Monastery {
        figure: VILLAGE,
        revealed: false,
        tile: TileId::Countryside(3),
        monastery_cards: Vec::new(),
        ..Monastery::default()
    });
    assert!(
        without
            .iter()
            .any(|e| matches!(e, Event::TileRevealed { .. }))
    );
    assert!(dealt_to_units(&without).is_empty());

    let decor = Monastery {
        figure: VILLAGE,
        revealed: false,
        monastery_cards: Vec::new(),
        ..Monastery::default()
    };
    let state = apply(
        done(&decor.state(), basic(card(March, 1))),
        &Event::CommonDeckShuffled {
            deck: CommonDeck::AdvancedActions,
            order: Vec::new(),
        },
    );
    let events = events_of(
        &state,
        &Action::Explore {
            position: MONASTERY_POSITION,
        },
    );
    assert!(dealt_to_units(&events).is_empty());
}

/// Un scénario de test : le plateau de jeu du bac à sable, tuiles de campagne mélangées.
fn shuffled_countryside() -> Scenario {
    Scenario::new(
        ScenarioId::Sandbox,
        MapShape::Wedge,
        TileDeckRecipe {
            countryside: 8,
            countryside_order: CountrysideOrder::Random,
            core_city: 1,
            core_non_city: 2,
        },
        4,
        0,
        ScenarioEnd::RoundLimit,
        FinalScoring::None,
    )
    .expect("scénario de test valide")
}

// C3 — Compendium, *Map Revealing* S1 : un monastère de la mise en place met une action avancée
// dans l'offre d'unités, après ses unités.
#[test]
fn setup_monastery_adds_an_advanced_action() {
    let monastery = |tile: &TileId| matches!(tile, TileId::Countryside(5 | 7));
    let (events, state) = (0..200)
        .map(|seed| {
            let config = GameConfig {
                seed,
                scenario: shuffled_countryside(),
                hero: Hero::Tovak,
                dummy_hero: Default::default(),
            };
            let events = new_game(&config, &mut config.rngs());
            let state = events.iter().fold(GameState::default(), apply);
            (events, state)
        })
        .find(|(events, _)| {
            events
                .iter()
                .any(|event| matches!(event, Event::TileRevealed { tile, .. } if monastery(tile)))
        })
        .expect("une graine pose un monastère à la mise en place");
    let dealt = dealt_to_units(&events);
    assert_eq!(dealt.len(), 4);
    assert!(dealt[..3].iter().all(|card| card.unit().is_some()));
    assert!(
        dealt[3]
            .deed()
            .is_some_and(|card| card.card().is_advanced_action())
    );
    assert_eq!(state.offer(Offer::Units), dealt);
}

// C4 — MKUE p. 4, 2.c ; FAQ p3 : l'action avancée retourne sous sa pioche avec les unités ; une
// nouvelle entre après les unités, avant le renouvellement de l'offre d'actions avancées.
#[test]
fn new_round_renews_monastery_advanced_actions() {
    let state = Monastery {
        offer: three_units(),
        ..Monastery::default()
    }
    .state();
    let next = state.common_deck(CommonDeck::AdvancedActions)[0];
    let (events, after) = new_round(&state);
    let returned = Event::OfferCardReturned {
        offer: Offer::Units,
        card: CommonCard::Deed(ice_shield()),
    };
    let dealt = Event::OfferCardDealt {
        offer: Offer::Units,
        card: next,
    };
    assert_events_in_order(&events, &[returned, dealt.clone()]);
    let dealt_at = events.iter().position(|event| *event == dealt);
    let renewal_at = events.iter().position(|event| {
        matches!(
            event,
            Event::OfferCardDealt {
                offer: Offer::AdvancedActions,
                ..
            }
        )
    });
    assert!(dealt_at < renewal_at, "{events:?}");
    let units = dealt_to_units(&events);
    assert_eq!(units.len(), 4);
    assert!(units[..3].iter().all(|card| card.unit().is_some()));
    assert_eq!(
        units_offer_deeds(&after),
        [next.deed().expect("carte Acte")]
    );
    assert_eq!(
        after.common_deck(CommonDeck::AdvancedActions).last(),
        Some(&CommonCard::Deed(ice_shield()))
    );
}

// C6 — Compendium, *Training* S4 : ni recrutable, ni au choix d'action avancée d'un niveau pair.
#[test]
fn monastery_advanced_actions_are_not_units_nor_offer() {
    let state = Monastery {
        offer: three_units(),
        ..Monastery::default()
    }
    .state();
    let interacting = done(&state, Action::Interact);
    let rich = apply(
        interacting,
        &Event::CardPlayed {
            player: SOLO,
            card: card(Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(20),
        },
    );
    assert!(!legal(&rich).iter().any(|action| matches!(
        action,
        Action::RecruitUnit { unit, .. } if !three_units().contains(unit)
    )));
    // Niveau 2 gagné à la fin du tour : l'offre d'actions avancées est vide, aucun choix.
    let leveling = apply(
        done(&state, sideways(card(March, 1))),
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame: 3,
        },
    );
    let end = legal(&leveling)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, .. } if discard.is_empty()))
        .expect("fin du tour sans défausse");
    let after = events_of(&leveling, &end)
        .iter()
        .fold(leveling.clone(), apply);
    assert_eq!(after.level(SOLO), 2);
    assert_eq!(after.advanced_action_choices(SOLO), 0);
    assert!(!legal(&after).iter().any(|action| matches!(
        action,
        Action::GainAdvancedAction { card } if *card == ice_shield()
    )));
}

// --- Acheter ---

/// L'interaction commencée sur ce site, sans bonus (spec 040), puis `influence` points de plus
/// (Promesse jouée de côté, pour une influence de la valeur voulue), ou de moins.
fn interacting(state: &GameState, site: Site, influence: i32) -> GameState {
    let bonus = if influence < 0 {
        i8::try_from(influence).unwrap_or(i8::MIN)
    } else {
        0
    };
    let state = apply(
        state.clone(),
        &Event::InteractionStarted {
            player: SOLO,
            site,
            bonus,
            city_shields: 0,
        },
    );
    if influence <= 0 {
        return state;
    }
    apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: card(Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(u8::try_from(influence).unwrap_or(u8::MAX)),
        },
    )
}

fn at_village() -> GameState {
    Monastery {
        figure: VILLAGE,
        offer: three_units(),
        ..Monastery::default()
    }
    .state()
}

fn at_monastery() -> GameState {
    Monastery {
        offer: three_units(),
        ..Monastery::default()
    }
    .state()
}

fn buy(card: CardId) -> Action {
    Action::BuyAdvancedAction { card }
}

fn purchases(state: &GameState) -> Vec<Action> {
    legal(state)
        .into_iter()
        .filter(|action| {
            matches!(
                action,
                Action::BuyHealing | Action::BuyAdvancedAction { .. }
            )
        })
        .collect()
}

// C7 — MKUE p. 8, 4.b ; Compendium, *Healing* S4 : 3 influence le point au village, autant de fois
// que l'influence le permet. L2 : sans Blessure ni unité blessée, l'achat reste permis.
#[test]
fn village_sells_healing_for_three() {
    let state = interacting(&at_village(), Site::Village, 6);
    assert_eq!(
        events_of(&state, &Action::BuyHealing),
        [Event::HealingBought {
            player: SOLO,
            cost: 3
        }]
    );
    let once = done(&state, Action::BuyHealing);
    let twice = done(&once, Action::BuyHealing);
    assert_eq!(twice.influence_points(SOLO), 0);
    assert_eq!(twice.healing_points(SOLO), 2);
    assert!(!legal(&twice).contains(&Action::BuyHealing));
}

// C7b — variante maison (issue #95), contre MKUE p. 6, 6.e : l'influence jouée avant l'interaction
// reste et paie les achats, comme si elle avait été jouée après.
#[test]
fn influence_played_before_interacting_still_buys() {
    let state = apply(
        at_village(),
        &Event::CardPlayed {
            player: SOLO,
            card: card(Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(6),
        },
    );
    let state = done(&state, Action::Interact);
    assert_eq!(state.influence_points(SOLO), 6);
    let twice = done(&done(&state, Action::BuyHealing), Action::BuyHealing);
    assert_eq!(twice.healing_points(SOLO), 2);
}

// C8 — carte du monastère : 2 influence le point.
#[test]
fn monastery_sells_healing_for_two() {
    let state = interacting(&at_monastery(), Site::Monastery, 2);
    assert_eq!(
        events_of(&state, &Action::BuyHealing),
        [Event::HealingBought {
            player: SOLO,
            cost: 2
        }]
    );
    let after = done(&state, Action::BuyHealing);
    assert_eq!(
        (after.influence_points(SOLO), after.healing_points(SOLO)),
        (0, 1)
    );
}

// C9 — MKUE p. 11, 3.c ; spec 042 : le point acheté rejoint la réserve des points de soin.
#[test]
fn bought_healing_joins_the_reserve() {
    let mut hand = Units::default().hand;
    hand.push(wound(1));
    let state = Monastery {
        figure: VILLAGE,
        hand,
        ..Monastery::default()
    }
    .state();
    let bought = done(&interacting(&state, Site::Village, 3), Action::BuyHealing);
    let heal = Action::SpendHealing {
        on: arcanum_core::HealingTarget::Wound,
    };
    assert!(legal(&bought).contains(&heal));
    assert!(!done(&bought, heal).hand(SOLO).contains(&wound(1)));

    let monks = UnitCard::new(Unit::NorthernMonks, 1);
    let state = apply(
        Monastery {
            level: 3,
            owned: vec![monks],
            ..Monastery::default()
        }
        .state(),
        &Event::UnitWounded {
            player: SOLO,
            unit: monks,
            wounds: 1,
        },
    );
    let state = interacting(&state, Site::Monastery, 4);
    let one = done(&state, Action::BuyHealing);
    let unit_heal = Action::SpendHealing {
        on: arcanum_core::HealingTarget::Unit(monks),
    };
    assert!(!legal(&one).contains(&unit_heal));
    let two = done(&one, Action::BuyHealing);
    assert!(legal(&two).contains(&unit_heal));
}

// C10 — MKUE p. 8, 4.c ; Compendium, *Miscellaneous* S7 : 6 influence, sur le dessus du deck,
// l'offre d'unités n'est pas complétée. L1 : la même carte ne s'achète pas deux fois.
#[test]
fn monastery_sells_advanced_actions() {
    let state = interacting(&at_monastery(), Site::Monastery, 6);
    let events = events_of(&state, &buy(ice_shield()));
    assert_eq!(
        events,
        [Event::AdvancedActionBought {
            player: SOLO,
            card: ice_shield(),
            cost: 6
        }]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.deck(SOLO).first(), Some(&ice_shield()));
    assert_eq!(after.influence_points(SOLO), 0);
    assert_eq!(after.offer(Offer::Units).len(), 3);
    assert!(units_offer_deeds(&after).is_empty());
    refused(&after, buy(ice_shield()));
}

// C11 — carte du monastère ; Compendium, *Map Revealing* S2 : toutes les actions avancées de
// l'offre d'unités s'achètent au même monastère.
#[test]
fn any_monastery_sells_every_monastery_card() {
    let fire_bolt = advanced(AdvancedAction::FireBolt);
    let state = Monastery {
        monastery_cards: vec![ice_shield(), fire_bolt],
        ..Monastery::default()
    }
    .state();
    let state = interacting(&state, Site::Monastery, 12);
    assert_eq!(
        purchases(&state)
            .into_iter()
            .filter(|action| matches!(action, Action::BuyAdvancedAction { .. }))
            .collect::<Vec<_>>(),
        [buy(ice_shield()), buy(fire_bolt)]
    );
    let after = done(&done(&state, buy(fire_bolt)), buy(ice_shield()));
    assert_eq!(after.deck(SOLO)[..2], [ice_shield(), fire_bolt]);
    assert_eq!(after.influence_points(SOLO), 0);
}

// C12 — MKUE p. 8, 4 : pas d'action avancée au village ; il faut le site et l'influence.
#[test]
fn buying_needs_the_site_and_enough_influence() {
    let village = interacting(&at_village(), Site::Village, 20);
    assert_eq!(purchases(&village), [Action::BuyHealing]);
    let poor = interacting(&at_monastery(), Site::Monastery, 5);
    assert_eq!(purchases(&poor), [Action::BuyHealing]);
    let negative = interacting(&at_monastery(), Site::Monastery, -1);
    assert!(negative.influence_points(SOLO) < 0);
    assert!(purchases(&negative).is_empty());
}

// C13 — MKUE p. 8, 4 : hors interaction, aucun achat.
#[test]
fn buying_needs_an_interaction() {
    let state = apply(
        at_monastery(),
        &Event::CardPlayed {
            player: SOLO,
            card: card(Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(10),
        },
    );
    assert!(purchases(&state).is_empty());
    refused(&state, Action::BuyHealing);
    refused(&state, buy(ice_shield()));
}

// C14 — MKUE p. 8, 5 ; Compendium, *Influence* S8 : achats, recrutement et unités se mêlent.
#[test]
fn purchases_mix_in_one_interaction() {
    let peasants = UnitCard::new(Unit::Peasants, 1);
    let monks = UnitCard::new(Unit::NorthernMonks, 1);
    let state = Monastery {
        level: 3,
        owned: vec![peasants],
        offer: vec![monks],
        ..Monastery::default()
    }
    .state();
    let state = interacting(&state, Site::Monastery, 13);
    let recruited = done(
        &state,
        Action::RecruitUnit {
            unit: monks,
            disband: None,
            bonds: false,
        },
    );
    let healed = done(&recruited, Action::BuyHealing);
    let activated = done(
        &healed,
        Action::ActivateUnit {
            enemy: None,
            unit: peasants,
            ability: 1,
            option: 0,
            mana: Vec::new(),
            target: None,
            leadership: false,
        },
    );
    assert_eq!(activated.influence_points(SOLO), 6);
    let bought = done(&activated, buy(ice_shield()));
    assert_eq!(bought.influence_points(SOLO), 0);
    assert_eq!(bought.healing_points(SOLO), 1);
    assert_eq!(bought.units(SOLO).len(), 2);
    assert_eq!(bought.deck(SOLO).first(), Some(&ice_shield()));
}

// --- Incendier un monastère ---

/// Ce jeton sur le dessus de la pile violette.
fn violet_on_top(state: GameState, token: EnemyToken) -> GameState {
    let mut order = vec![token];
    order.extend(
        state
            .enemy_pile(EnemyColor::Violet)
            .iter()
            .copied()
            .filter(|&other| other != token),
    );
    apply(
        state,
        &Event::EnemyPileShuffled {
            color: EnemyColor::Violet,
            order,
        },
    )
}

/// Au monastère, les Moines sur le dessus de la pile violette.
fn burnable() -> GameState {
    violet_on_top(at_monastery(), monks())
}

/// L'incendie commencé, puis la phase d'attaque sans dégâts.
fn burning_in_attack_phase(state: &GameState) -> GameState {
    skip_to_attack_phase(&done(state, Action::BurnMonastery))
}

/// Les Moines (armure 5) vaincus en phase d'attaque : Rage (Attaque 2) et trois cartes de côté.
fn defeat_monks(state: &GameState) -> GameState {
    let state = done(state, declare(&[foe(state, MONASTERY)]));
    let state = done(&state, option(card(Rage, 1), 0));
    let state = [card(Promise, 1), card(Threaten, 1), card(March, 1)]
        .into_iter()
        .fold(state, |state, card| done(&state, sideways_attack(card)));
    done(&state, Action::ResolveAttack)
}

fn reputation(state: GameState, change: i8) -> GameState {
    apply(
        state,
        &Event::ReputationChanged {
            player: SOLO,
            change,
        },
    )
}

// C15 — MKUE p. 8, *Combat with Enemies* 1.c ; carte du monastère : Réputation −3, un jeton violet
// tiré sur le monastère, le combat engagé contre lui seul.
#[test]
fn burning_starts_a_combat_against_a_violet_enemy() {
    let state = burnable();
    assert!(legal(&state).contains(&Action::BurnMonastery));
    let events = events_of(&state, &Action::BurnMonastery);
    assert_eq!(
        events,
        [
            Event::MonasteryBurnStarted {
                player: SOLO,
                hex: MONASTERY
            },
            Event::ReputationChanged {
                player: SOLO,
                change: -3
            },
            Event::EnemyTokenPlaced {
                token: monks(),
                hex: MONASTERY
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(MONASTERY, monks())]
            },
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.reputation(SOLO).value(), -3);
    assert_eq!(after.burning(SOLO), Some(MONASTERY));
    assert_eq!(after.enemy_tokens_at(MONASTERY), [monks()]);
}

// C16 — MKUE p. 6, 6.e ; Compendium, *Influence* S7 : points perdus, plus de déplacement,
// d'interaction ni de repos.
#[test]
fn burning_is_the_action_of_the_turn() {
    let state = done(&burnable(), basic(card(March, 1)));
    let state = done(
        &state,
        Action::PlayCard {
            card: card(Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
        },
    );
    let burning = done(&state, Action::BurnMonastery);
    assert_eq!(
        (
            burning.move_points(SOLO),
            burning.influence_points(SOLO),
            burning.can_rest(SOLO)
        ),
        (0, 0, false)
    );
    let after = done(&skip_to_attack_phase(&burning), Action::EndCombat);
    assert!(!legal(&after).iter().any(|action| matches!(
        action,
        Action::MoveHero { .. }
            | Action::FollowRoute { .. }
            | Action::Explore { .. }
            | Action::Interact
            | Action::BurnMonastery
    )));
}

// C17 — Compendium, *Combat* S17 : ni après une interaction, un défi ou une provocation ; pendant
// l'incendie, pas de défi.
#[test]
fn burning_excludes_other_combats() {
    let near = Hex::new(5, -5);
    let state = violet_on_top(
        Monastery {
            offer: three_units(),
            placed: vec![(prowlers(), near)],
            ..Monastery::default()
        }
        .state(),
        monks(),
    );
    assert!(legal(&state).contains(&Action::BurnMonastery));
    assert!(legal(&state).contains(&challenge(near)));
    let interacted = done(&state, Action::Interact);
    assert!(!legal(&interacted).contains(&Action::BurnMonastery));
    let challenged = done(&state, challenge(near));
    assert!(!legal(&challenged).contains(&Action::BurnMonastery));
    let provoked = apply(
        state.clone(),
        &Event::EnemiesProvoked {
            player: SOLO,
            enemies: vec![near],
        },
    );
    assert!(!legal(&provoked).contains(&Action::BurnMonastery));
    let burning = done(&state, Action::BurnMonastery);
    assert!(
        !legal(&burning)
            .iter()
            .any(|action| matches!(action, Action::Challenge { .. }))
    );
    assert_eq!(
        burning.combat(SOLO).map(|combat| combat.enemies.len()),
        Some(1)
    );
}

// C18 — Compendium, *Influence* S5 ; spec 040, C3 : sur la case X aussi.
#[test]
fn burning_works_on_x() {
    let on_x = reputation(burnable(), -7);
    assert!(on_x.reputation(SOLO).is_x());
    assert!(legal(&on_x).contains(&Action::BurnMonastery));
    let low = done(&reputation(burnable(), -5), Action::BurnMonastery);
    assert_eq!(low.reputation(SOLO).value(), -7);
}

// C19 — FAQ p3 ; Compendium, *Combat* S9 : l'ennemi du monastère n'est pas fortifié.
#[test]
fn monastery_defender_is_not_fortified() {
    let burning = done(&burnable(), Action::BurnMonastery);
    let declared = done(&burning, declare(&[foe(&burning, MONASTERY)]));
    let combat = declared.combat(SOLO).expect("combat engagé");
    assert_eq!(combat.phase, CombatPhase::RangedAndSiege);
    assert!(combat.accepts(Effect::Attack {
        amount: 3,
        element: arcanum_core::Element::Physical,
        kind: arcanum_core::AttackKind::Ranged,
    }));
}

// C20 — carte du monastère ; Compendium, *Combat* S43 : les unités ne participent pas ; après le
// combat, elles s'activent à nouveau.
#[test]
fn units_sit_out_the_burning() {
    // Spec 052 : après le combat, l'Influence ne sert plus (hors interaction) ; les Illusionnistes
    // gardent une capacité qui sert encore (le cristal blanc).
    let peasants = UnitCard::new(Unit::Illusionists, 1);
    let state = violet_on_top(
        Monastery {
            owned: vec![peasants],
            ..Monastery::default()
        }
        .state(),
        monks(),
    );
    let activations = |state: &GameState| {
        legal(state)
            .into_iter()
            .filter(|action| matches!(action, Action::ActivateUnit { .. }))
            .count()
    };
    assert!(activations(&state) > 0);
    let burning = done(&state, Action::BurnMonastery);
    let blocking = done(&burning, Action::NextCombatPhase);
    assert_eq!(activations(&blocking), 0);
    let events = events_of(&blocking, &Action::NextCombatPhase);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::DamageAssignedToUnit { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::WoundsTaken { .. }))
    );
    let attacking = events.iter().fold(blocking, apply);
    assert_eq!(
        attacking.combat(SOLO).map(|c| c.phase),
        Some(CombatPhase::Attack)
    );
    assert_eq!(activations(&attacking), 0);
    assert!(!attacking.units(SOLO)[0].is_wounded());
    let after = done(&attacking, Action::EndCombat);
    assert!(activations(&after) > 0);
}

// C21 — carte du monastère ; Compendium, *Combat* S36 : victoire, Bouclier, monastère incendié,
// gloire entière, une récompense artefact.
#[test]
fn victory_burns_the_monastery() {
    let defeated = defeat_monks(&burning_in_attack_phase(&burnable()));
    assert_eq!(defeated.fame(SOLO), 4);
    let events = events_of(&defeated, &Action::EndCombat);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::MonasteryBurned {
                player: SOLO,
                hex: MONASTERY
            },
        ]
    );
    let after = events.iter().fold(defeated, apply);
    assert_eq!(after.shields().get(&MONASTERY), Some(&SOLO));
    assert_eq!(after.burned_monasteries(), [MONASTERY]);
    assert_eq!(after.artifact_rewards(SOLO), 1);
    assert_eq!(after.enemy_discard(EnemyColor::Violet), [monks()]);
}

// C22 — FAQ p3 ; Compendium, *Combat* S39 : ennemi défaussé, monastère intact, réputation perdue ;
// un nouvel incendie tire un autre jeton.
#[test]
fn failed_burning_discards_the_enemy() {
    let attacking = burning_in_attack_phase(&burnable());
    let events = events_of(&attacking, &Action::EndCombat);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::EnemyTokenDiscarded {
                hex: MONASTERY,
                token: monks(),
            },
        ]
    );
    let after = events.iter().fold(attacking, apply);
    assert!(after.enemy_tokens().is_empty());
    assert_eq!(after.enemy_discard(EnemyColor::Violet), [monks()]);
    assert!(after.shields().is_empty());
    assert_eq!(after.reputation(SOLO).value(), -3);
    assert_eq!(after.artifact_rewards(SOLO), 0);
    let next_turn = [
        Event::TurnEnded { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    ]
    .iter()
    .fold(after, apply);
    let next = next_turn.enemy_pile(EnemyColor::Violet)[0];
    assert_ne!(next, monks());
    assert!(
        events_of(&next_turn, &Action::BurnMonastery).contains(&Event::EnemyTokenPlaced {
            token: next,
            hex: MONASTERY
        })
    );
}

fn burned(state: GameState) -> GameState {
    apply(
        state,
        &Event::MonasteryBurned {
            player: SOLO,
            hex: MONASTERY,
        },
    )
}

// C23 — Compendium, *Combat* S17 : une case vide de son terrain.
#[test]
fn burned_monastery_is_an_empty_space() {
    let state = burned(at_monastery());
    assert!(!legal(&state).contains(&Action::Interact));
    assert!(!legal(&state).contains(&Action::BurnMonastery));
    let nearby = burned(
        Monastery {
            figure: NEAR_MONASTERY,
            ..Monastery::default()
        }
        .state(),
    );
    let walking = done(&nearby, basic(card(March, 1)));
    assert!(legal(&walking).contains(&Action::MoveHero { to: MONASTERY }));
    let arrived = done(&walking, Action::MoveHero { to: MONASTERY });
    assert_eq!(arrived.move_points(SOLO), 0);
    assert!(!legal(&arrived).contains(&Action::Interact));
}

// C5 — Compendium, *Map Revealing* S2 : la carte reste jusqu'à la nouvelle manche, qui n'en
// ajoute plus.
#[test]
fn burned_monastery_adds_nothing_at_new_round() {
    let state = burned(at_monastery());
    assert_eq!(units_offer_deeds(&state), [ice_shield()]);
    let (events, after) = new_round(&state);
    assert_eq!(
        dealt_to_units(&events)
            .iter()
            .filter(|card| card.deed().is_some())
            .count(),
        0
    );
    assert!(units_offer_deeds(&after).is_empty());
}

// L7 — spec 033 : pile violette et défausse vides, la pile de secours fournit le jeton.
#[test]
fn empty_violet_pile_uses_another_pile() {
    let state = apply(
        at_monastery(),
        &Event::EnemyPileShuffled {
            color: EnemyColor::Violet,
            order: Vec::new(),
        },
    );
    let events = events_of(&state, &Action::BurnMonastery);
    let placed = events.iter().find_map(|event| match event {
        Event::EnemyTokenPlaced { token, hex } if *hex == MONASTERY => Some(*token),
        _ => None,
    });
    assert!(placed.is_some_and(|token| token.color() != EnemyColor::Violet));
}

// --- Récompense : un artefact ---

fn ruby_ring() -> CardId {
    artifact(Artifact::RubyRing)
}

fn amulet_of_sun() -> CardId {
    artifact(Artifact::AmuletOfSun)
}

/// Le monastère incendié, combat fini ; la pioche d'artefacts a `artifacts` sur le dessus. Les
/// Moines (gloire 4) font passer au niveau 2 ; l'offre d'actions avancées est vide (aucun choix).
fn rewarded(artifacts: Vec<CardId>) -> GameState {
    let state = violet_on_top(
        Monastery {
            artifacts,
            ..Monastery::default()
        }
        .state(),
        monks(),
    );
    done(
        &defeat_monks(&burning_in_attack_phase(&state)),
        Action::EndCombat,
    )
}

fn end_turn_without_discard(state: &GameState) -> Action {
    legal(state)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, .. } if discard.is_empty()))
        .expect("fin du tour sans défausse")
}

fn take(card: CardId) -> Action {
    Action::TakeArtifact { card }
}

// C24 — MKUE p. 11, 5.b ; Compendium, *End of Turn* S5 : deux tirés, un pris (dessus du deck),
// l'autre sous la pioche, puis la pioche de fin de tour.
#[test]
fn artifact_reward_draws_two_takes_one() {
    let state = rewarded(vec![ruby_ring(), amulet_of_sun()]);
    let events = events_of(&state, &end_turn_without_discard(&state));
    let drawn = Event::ArtifactsDrawn {
        player: SOLO,
        cards: vec![ruby_ring(), amulet_of_sun()],
    };
    assert!(events.contains(&drawn));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. } | Event::TurnStarted { .. }))
    );
    let choosing = events.iter().fold(state, apply);
    assert_eq!(
        choosing.drawn_artifacts(SOLO),
        [ruby_ring(), amulet_of_sun()]
    );
    assert_eq!(choosing.artifact_rewards(SOLO), 0);
    assert_eq!(legal(&choosing), [take(ruby_ring()), take(amulet_of_sun())]);
    let events = events_of(&choosing, &take(amulet_of_sun()));
    assert_eq!(
        events[..3],
        [
            Event::ArtifactTaken {
                player: SOLO,
                card: amulet_of_sun()
            },
            Event::ArtifactReturned { card: ruby_ring() },
            Event::CardDrawn {
                player: SOLO,
                card: amulet_of_sun()
            },
        ]
    );
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
    let after = events.iter().fold(choosing, apply);
    assert!(after.hand(SOLO).contains(&amulet_of_sun()));
    assert!(after.drawn_artifacts(SOLO).is_empty());
    assert_eq!(
        after.common_deck(CommonDeck::Artifacts).last(),
        Some(&CommonCard::Deed(ruby_ring()))
    );
}

// C25 — MKUE p. 11, 5 et 6 : l'artefact d'abord, puis l'action avancée du niveau pair, puis la
// pioche.
#[test]
fn artifact_reward_comes_before_level_up() {
    let state = rewarded(vec![ruby_ring(), amulet_of_sun()]);
    let offered = state.common_deck(CommonDeck::AdvancedActions)[0];
    let state = apply(
        state,
        &Event::OfferCardDealt {
            offer: Offer::AdvancedActions,
            card: offered,
        },
    );
    let events = events_of(&state, &end_turn_without_discard(&state));
    assert_events_in_order(
        &events,
        &[
            Event::ArtifactsDrawn {
                player: SOLO,
                cards: vec![ruby_ring(), amulet_of_sun()],
            },
            Event::LevelGained {
                player: SOLO,
                level: 2,
            },
        ],
    );
    let choosing = events.iter().fold(state, apply);
    assert_eq!(legal(&choosing), [take(ruby_ring()), take(amulet_of_sun())]);
    let events = events_of(&choosing, &take(ruby_ring()));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    let gaining = events.iter().fold(choosing, apply);
    let gained = offered.deed().expect("carte Acte");
    assert_eq!(
        legal(&gaining),
        [Action::GainAdvancedAction { card: gained }]
    );
    let after = done(&gaining, Action::GainAdvancedAction { card: gained });
    assert_eq!(after.hand(SOLO)[..2], [gained, ruby_ring()]);
}

// Issue privée 317 — MKUE p. 11, 5.b puis 6.b : les compétences du niveau pair ne sont révélées qu'après
// le choix de l'artefact de la récompense, pas dès la fin du tour.
#[test]
fn skills_revealed_after_artifact_reward() {
    use arcanum_core::Skill;
    let pile = vec![
        Skill::DarkPaths,
        Skill::BurningPower,
        Skill::HotSwordsmanship,
        Skill::DarkNegotiation,
    ];
    let state = apply(
        rewarded(vec![ruby_ring(), amulet_of_sun()]),
        &Event::SkillPileShuffled {
            player: SOLO,
            order: pile.clone(),
        },
    );
    let events = events_of(&state, &end_turn_without_discard(&state));
    assert!(events.contains(&Event::LevelGained {
        player: SOLO,
        level: 2,
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SkillsRevealed { .. }))
    );
    let choosing = events.iter().fold(state, apply);
    assert_eq!(legal(&choosing), [take(ruby_ring()), take(amulet_of_sun())]);
    let events = events_of(&choosing, &take(ruby_ring()));
    assert!(events.contains(&Event::SkillsRevealed {
        player: SOLO,
        skills: pile[..2].to_vec(),
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    let after = events.iter().fold(choosing, apply);
    assert_eq!(
        legal(&after),
        [
            Action::ChooseSkill { skill: pile[0] },
            Action::ChooseSkill { skill: pile[1] }
        ]
    );
}

// C26 — interprétation : une seule carte, prise sans choix ; aucune, rien.
#[test]
fn short_artifact_deck() {
    let one = apply(
        rewarded(Vec::new()),
        &Event::CommonDeckShuffled {
            deck: CommonDeck::Artifacts,
            order: vec![CommonCard::Deed(ruby_ring())],
        },
    );
    let events = events_of(&one, &end_turn_without_discard(&one));
    assert_events_in_order(
        &events,
        &[
            Event::ArtifactsDrawn {
                player: SOLO,
                cards: vec![ruby_ring()],
            },
            Event::ArtifactTaken {
                player: SOLO,
                card: ruby_ring(),
            },
            Event::CardDrawn {
                player: SOLO,
                card: ruby_ring(),
            },
        ],
    );
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));

    let none = apply(
        rewarded(Vec::new()),
        &Event::CommonDeckShuffled {
            deck: CommonDeck::Artifacts,
            order: Vec::new(),
        },
    );
    let events = events_of(&none, &end_turn_without_discard(&none));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ArtifactsDrawn { .. }))
    );
    let after = events.iter().fold(none, apply);
    assert_eq!(after.artifact_rewards(SOLO), 0);
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
}

// C27 — spec 050, Q5 : l'artefact pris se joue de côté seulement.
#[test]
fn taken_artifact_plays_sideways() {
    let state = rewarded(vec![ruby_ring(), amulet_of_sun()]);
    let choosing = done(&state, end_turn_without_discard(&state));
    let after = done(&choosing, take(ruby_ring()));
    let plays: Vec<CardPlay> = legal(&after)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } if card == ruby_ring() => Some(play),
            _ => None,
        })
        .collect();
    // Spec 055 (C2, test modifié) : l'artefact se joue aussi pour son effet, plus seulement de
    // côté.
    assert!(
        plays
            .iter()
            .any(|play| matches!(play, CardPlay::Sideways(_)))
    );
    assert!(
        plays
            .iter()
            .any(|play| matches!(play, CardPlay::Artifact { .. }))
    );
}

// --- Décompte, retour en arrière, sauvegarde, vue ---

// C28 — MKUE p. 15 : Le Collectionneur 2 par artefact plus 1 par deux cristaux ; Le Conquérant 2
// par Bouclier sur un monastère.
#[test]
fn final_score_counts_artifacts_and_shields() {
    let mut state = burned(at_monastery());
    let mut deck = state.deck(SOLO).to_vec();
    deck.insert(0, ruby_ring());
    state = apply(
        state,
        &Event::DeckShuffled {
            player: SOLO,
            order: deck,
        },
    );
    for color in [
        arcanum_core::BasicColor::Red,
        arcanum_core::BasicColor::Blue,
    ] {
        state = apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        );
    }
    let score = arcanum_core::final_score(&state, SOLO);
    assert_eq!((score.artifacts, score.shields), (1, 1));
    assert_eq!(score.points(arcanum_core::Achievement::Loot), 3);
    assert_eq!(score.points(arcanum_core::Achievement::Conqueror), 2);
}

// C29 — spec 024 : les achats et le choix de l'artefact se reprennent, pas l'incendie (jeton
// tiré) ni le tirage des artefacts.
#[test]
fn purchases_undo_burning_does_not() {
    let mut setup = Monastery::default().events();
    setup.push(Event::InteractionStarted {
        player: SOLO,
        site: Site::Monastery,
        bonus: 0,
        city_shields: 0,
    });
    setup.push(Event::CardPlayed {
        player: SOLO,
        card: card(Promise, 1),
        play: CardPlay::Sideways(SidewaysEffect::Influence),
        effect: Effect::Influence(8),
    });
    undo_log(setup.clone(), &[Action::BuyHealing]);
    undo_log(setup, &[buy(ice_shield())]);

    assert!(
        events_of(&burnable(), &Action::BurnMonastery)
            .iter()
            .any(Event::blocks_undo)
    );
    let state = rewarded(vec![ruby_ring(), amulet_of_sun()]);
    let drawn = events_of(&state, &end_turn_without_discard(&state));
    assert!(
        drawn
            .iter()
            .any(|event| matches!(event, Event::ArtifactsDrawn { .. }) && event.blocks_undo())
    );
    for event in [
        Event::HealingBought {
            player: SOLO,
            cost: 2,
        },
        Event::ArtifactTaken {
            player: SOLO,
            card: ruby_ring(),
        },
        Event::ArtifactReturned { card: ruby_ring() },
        Event::MonasteryBurned {
            player: SOLO,
            hex: MONASTERY,
        },
    ] {
        assert!(!event.blocks_undo(), "{event:?}");
    }
}

/// Le journal : la mise en place `setup` en décision 0, puis une décision par action ; la
/// dernière est reprise, l'état revient à celui d'avant.
fn undo_log(setup: Vec<Event>, actions: &[Action]) {
    let mut log = arcanum_core::EventLog::default();
    let mut state = GameState::default();
    for event in setup {
        state = apply(state, &event);
        log.push(0, event);
    }
    let mut before = state.clone();
    for (decision, action) in (1..).zip(actions) {
        before = state.clone();
        for event in events_of(&state, action) {
            state = apply(state, &event);
            log.push(decision, event);
        }
    }
    let mut runner =
        arcanum_core::GameRunner::resume(log, arcanum_core::RngPositions::new(), Vec::new())
            .expect("journal valide");
    assert_eq!(runner.state(), &state);
    runner.undo().expect("retour en arrière possible");
    assert_eq!(runner.state(), &before);
}

// C31 — la vue donne les Boucliers, les monastères incendiés, les récompenses et les artefacts
// tirés.
#[test]
fn view_shows_shields_and_rewards() {
    let burning = done(&burnable(), Action::BurnMonastery);
    assert_eq!(arcanum_core::view(&burning, SOLO).burning, Some(MONASTERY));
    let state = rewarded(vec![ruby_ring(), amulet_of_sun()]);
    let view = arcanum_core::view(&state, SOLO);
    assert_eq!(view.shields.get(&MONASTERY), Some(&SOLO));
    assert_eq!(view.burned_monasteries, [MONASTERY]);
    assert_eq!(view.artifact_rewards, 1);
    let choosing = done(&state, end_turn_without_discard(&state));
    assert_eq!(
        arcanum_core::view(&choosing, SOLO).drawn_artifacts,
        [ruby_ring(), amulet_of_sun()]
    );
}

// --- Piller un village ---

// C32 — carte du village ; Compendium, *Turn Start* S1 : deux cartes, Réputation −1, en tête des
// actions au début du tour.
#[test]
fn plundering_draws_two_cards() {
    let state = at_village();
    assert_eq!(legal(&state).first(), Some(&Action::PlunderVillage));
    let deck = state.deck(SOLO).to_vec();
    let events = events_of(&state, &Action::PlunderVillage);
    assert_eq!(
        events,
        [
            Event::VillagePlundered { player: SOLO },
            Event::ReputationChanged {
                player: SOLO,
                change: -1
            },
            Event::CardDrawn {
                player: SOLO,
                card: deck[0]
            },
            Event::CardDrawn {
                player: SOLO,
                card: deck[1]
            },
        ]
    );
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(after.hand(SOLO).len(), state.hand(SOLO).len() + 2);
    assert_eq!(after.reputation(SOLO).value(), -1);
    // Compendium, *Influence* S5 : sur la case X aussi.
    assert!(legal(&reputation(at_village(), -7)).contains(&Action::PlunderVillage));
}

// C33 — Compendium, *Turn Start* S1 ; MKUE p. 11, 7.d : seulement avant toute autre chose, une fois ;
// deck à une carte, une seule piochée.
#[test]
fn plundering_only_before_the_turn() {
    let state = at_village();
    let played = done(&state, sideways(card(March, 1)));
    assert!(!legal(&played).contains(&Action::PlunderVillage));
    let peasants = UnitCard::new(Unit::Peasants, 1);
    let with_unit = Monastery {
        figure: VILLAGE,
        owned: vec![peasants],
        ..Monastery::default()
    }
    .state();
    assert!(legal(&with_unit).contains(&Action::PlunderVillage));
    let activated = done(
        &with_unit,
        Action::ActivateUnit {
            enemy: None,
            unit: peasants,
            ability: 2,
            option: 0,
            mana: Vec::new(),
            target: None,
            leadership: false,
        },
    );
    assert!(!legal(&activated).contains(&Action::PlunderVillage));
    let plundered = done(&state, Action::PlunderVillage);
    assert!(!legal(&plundered).contains(&Action::PlunderVillage));
    assert!(!legal(&at_monastery()).contains(&Action::PlunderVillage));

    let short = apply(
        state,
        &Event::DeckShuffled {
            player: SOLO,
            order: vec![card(March, 2)],
        },
    );
    let events = events_of(&short, &Action::PlunderVillage);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::CardDrawn { .. }))
            .count(),
        1
    );
}
