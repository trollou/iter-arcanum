//! Scénarios de la spec 042 (Unités).

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, CommonCard, CommonDeck, DamageTarget,
    Effect, Enemy, EnemyToken, Event, GameState, HealingTarget, Hex, ManaColor, ManaPayment, Offer,
    OwnedUnit, SidewaysEffect, Site, Unit, UnitCard, apply, legal_actions,
};

use BasicAction::*;

mod arene;
use arene::*;

// --- Outils ---

fn unit(unit: Unit, copy: u8) -> UnitCard {
    UnitCard::new(unit, copy)
}

fn peasants() -> UnitCard {
    unit(Unit::Peasants, 1)
}

fn guardsmen() -> UnitCard {
    unit(Unit::UtemGuardsmen, 1)
}

fn illusionists() -> UnitCard {
    unit(Unit::Illusionists, 1)
}

fn herbalists() -> UnitCard {
    unit(Unit::Herbalists, 1)
}

/// L'interaction commencée sur ce site, sans bonus (spec 040), puis `influence` points.
fn interacting(state: &GameState, site: Site, influence: i32) -> GameState {
    let state = apply(
        state.clone(),
        &Event::InteractionStarted {
            player: SOLO,
            site,
            bonus: 0,
            city_shields: 0,
        },
    );
    with_influence(state, influence)
}

/// `amount` points d'influence de plus (ou de moins), par un événement : la valeur seule compte.
fn with_influence(state: GameState, amount: i32) -> GameState {
    if amount == 0 {
        return state;
    }
    if amount < 0 {
        // Un malus : l'interaction recommence avec ce bonus négatif.
        let site = state.interaction(SOLO).map_or(Site::Village, |i| i.site);
        return apply(
            state,
            &Event::InteractionStarted {
                player: SOLO,
                site,
                bonus: i8::try_from(amount).unwrap_or(i8::MIN),
                city_shields: 0,
            },
        );
    }
    let played = state.hand(SOLO)[0];
    apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: played,
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(u8::try_from(amount).unwrap_or(u8::MAX)),
        },
    )
}

fn legal(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
}

fn recruit(unit: UnitCard) -> Action {
    Action::RecruitUnit {
        unit,
        disband: None,
        bonds: false,
    }
}

fn recruit_disbanding(unit: UnitCard, disband: UnitCard) -> Action {
    Action::RecruitUnit {
        unit,
        disband: Some(disband),
        bonds: false,
    }
}

/// Les unités recrutables parmi les actions légales, dans l'ordre.
fn recruitable(state: &GameState) -> Vec<UnitCard> {
    legal(state)
        .into_iter()
        .filter_map(|action| match action {
            Action::RecruitUnit { unit, .. } => Some(unit),
            _ => None,
        })
        .collect()
}

fn offered_units(state: &GameState) -> Vec<UnitCard> {
    state
        .offer(Offer::Units)
        .iter()
        .filter_map(|card| card.unit())
        .collect()
}

// --- Recruter ---

// C1 — MKUE p. 8, 4.a ; p. 5, 4.
#[test]
fn recruiting_takes_a_unit_from_the_offer() {
    let state = Units {
        offer: vec![peasants(), guardsmen(), herbalists()],
        ..Units::default()
    }
    .state();
    let state = interacting(&state, Site::Village, 5);
    let events = events_of(&state, &recruit(guardsmen()));
    assert_eq!(
        events,
        vec![Event::UnitRecruited {
            player: SOLO,
            unit: guardsmen(),
            cost: 5
        }]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.influence_points(SOLO), 0);
    assert_eq!(state.units(SOLO), &[OwnedUnit::recruited(guardsmen())]);
    assert_eq!(offered_units(&state), vec![peasants(), herbalists()]);
}

// C2 — MKUE p. 8, 4.a : les lieux de recrutement de la carte.
#[test]
fn recruiting_needs_a_matching_site() {
    let state = Units {
        offer: vec![peasants(), illusionists(), herbalists()],
        ..Units::default()
    }
    .state();
    let village = interacting(&state, Site::Village, 9);
    assert_eq!(recruitable(&village), vec![peasants(), herbalists()]);
    refused(&village, recruit(illusionists()));
    let monastery = interacting(&state, Site::Monastery, 9);
    assert_eq!(recruitable(&monastery), vec![illusionists(), herbalists()]);
    refused(&monastery, recruit(peasants()));
}

// C3 — MKUE p. 8, 4.a ; spec 040 : l'influence signée doit atteindre le coût.
#[test]
fn recruiting_needs_enough_influence() {
    let state = Units {
        offer: vec![guardsmen(), herbalists()],
        ..Units::default()
    }
    .state();
    let four = interacting(&state, Site::Village, 4);
    assert_eq!(recruitable(&four), vec![herbalists()]);
    refused(&four, recruit(guardsmen()));
    let penalty = interacting(&state, Site::Village, -2);
    assert_eq!(penalty.influence_points(SOLO), -2);
    assert!(recruitable(&penalty).is_empty());
}

// C4 — MKUE p. 5, 4.a : jetons tous pris, recruter renvoie une unité.
#[test]
fn recruiting_beyond_command_disbands_a_unit() {
    let state = Units {
        owned: vec![peasants()],
        offer: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    let state = interacting(&state, Site::Village, 5);
    refused(&state, recruit(guardsmen()));
    assert!(legal(&state).contains(&recruit_disbanding(guardsmen(), peasants())));
    let events = events_of(&state, &recruit_disbanding(guardsmen(), peasants()));
    assert_eq!(
        events,
        vec![
            Event::UnitDisbanded {
                player: SOLO,
                unit: peasants()
            },
            Event::UnitRecruited {
                player: SOLO,
                unit: guardsmen(),
                cost: 5
            },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.units(SOLO), &[OwnedUnit::recruited(guardsmen())]);
    // Ni offre ni pioche : l'unité renvoyée quitte la partie.
    assert!(offered_units(&state).is_empty());
    assert!(
        !state
            .common_deck(CommonDeck::RegularUnits)
            .contains(&CommonCard::Unit(peasants()))
    );
}

// C4 — un jeton libre : pas de renvoi.
#[test]
fn recruiting_with_a_free_token_disbands_nothing() {
    let state = Units {
        level: 3,
        owned: vec![peasants()],
        offer: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    let state = interacting(&state, Site::Village, 5);
    assert!(legal(&state).contains(&recruit(guardsmen())));
    refused(&state, recruit_disbanding(guardsmen(), peasants()));
}

// C5 — MKUE p. 8, 4 : seulement pendant une interaction.
#[test]
fn recruiting_needs_an_interaction() {
    let state = Units {
        offer: vec![peasants()],
        ..Units::default()
    }
    .state();
    let state = with_influence(state, 9);
    assert!(recruitable(&state).is_empty());
    refused(&state, recruit(peasants()));
}

// C7 — FAQ p3 : le jeton d'un changement de niveau n'arrive qu'à la fin du tour.
#[test]
fn new_command_token_after_end_of_turn() {
    let state = Units {
        owned: vec![peasants()],
        offer: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    // Gloire du niveau 3 (deux jetons) gagnée pendant le tour.
    let state = apply(
        state,
        &Event::FameGained {
            player: SOLO,
            fame: 8,
        },
    );
    assert_eq!(state.pending_level(SOLO), Some(3));
    let state = interacting(&state, Site::Village, 5);
    refused(&state, recruit(guardsmen()));
    assert!(legal(&state).contains(&recruit_disbanding(guardsmen(), peasants())));
}

// L1 — une unité recrutée n'est plus dans l'offre.
#[test]
fn a_unit_is_recruited_once() {
    let state = Units {
        level: 3,
        offer: vec![peasants(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = interacting(&state, Site::Village, 9);
    let state = done(&state, recruit(peasants()));
    refused(&state, recruit(peasants()));
    assert_eq!(recruitable(&state), vec![guardsmen()]);
    assert_eq!(state.influence_points(SOLO), 5);
}

// --- Activer hors combat ---

fn fire_golems() -> UnitCard {
    unit(Unit::FireGolems, 1)
}

fn fire_mages() -> UnitCard {
    unit(Unit::FireMages, 1)
}

fn altem_mages() -> UnitCard {
    unit(Unit::AltemMages, 1)
}

fn activate(unit: UnitCard, ability: u8, option: u8) -> Action {
    Action::ActivateUnit {
        enemy: None,
        unit,
        ability,
        option,
        mana: Vec::new(),
        target: None,
        leadership: false,
    }
}

fn activate_paid(unit: UnitCard, ability: u8, option: u8, mana: &[ManaPayment]) -> Action {
    Action::ActivateUnit {
        enemy: None,
        unit,
        ability,
        option,
        mana: mana.to_vec(),
        target: None,
        leadership: false,
    }
}

/// Les activations légales de cette unité : (capacité, option, paiements, cible).
fn activations(
    state: &GameState,
    of: UnitCard,
) -> Vec<(u8, u8, Vec<ManaPayment>, Option<UnitCard>)> {
    legal(state)
        .into_iter()
        .filter_map(|action| match action {
            Action::ActivateUnit {
                unit,
                ability,
                option,
                mana,
                target,
                enemy: _,
                leadership: false,
            } if unit == of => Some((ability, option, mana, target)),
            _ => None,
        })
        .collect()
}

/// Les capacités activables de cette unité, chacune une fois, dans l'ordre.
fn active_abilities(state: &GameState, of: UnitCard) -> Vec<u8> {
    let mut abilities: Vec<u8> = activations(state, of)
        .into_iter()
        .map(|(ability, ..)| ability)
        .collect();
    abilities.dedup();
    abilities
}

/// L'unité épuisée par une activation, par l'événement : seul l'état compte ici.
fn spent(state: GameState, unit: UnitCard) -> GameState {
    apply(
        state,
        &Event::UnitActivated {
            enemy: None,
            player: SOLO,
            unit,
            ability: 0,
            option: 0,
            effect: Effect::Move(0),
        },
    )
}

fn wounded(state: GameState, unit: UnitCard, wounds: u8) -> GameState {
    apply(
        state,
        &Event::UnitWounded {
            player: SOLO,
            unit,
            wounds,
        },
    )
}

fn owned(state: &GameState, card: UnitCard) -> OwnedUnit {
    *state
        .units(SOLO)
        .iter()
        .find(|owned| owned.card == card)
        .unwrap_or_else(|| panic!("{card:?} absente de la zone"))
}

// C6 — MKUE p. 8, 4.a ; Compendium, *Silver Units* Q3.
#[test]
fn a_new_unit_can_be_used_at_once() {
    let state = Units {
        level: 3,
        offer: vec![peasants(), herbalists()],
        ..Units::default()
    }
    .state();
    let state = interacting(&state, Site::Village, 5);
    let state = done(&state, recruit(peasants()));
    assert_eq!(state.influence_points(SOLO), 1);
    let state = done(&state, activate(peasants(), 1, 0));
    assert_eq!(state.influence_points(SOLO), 3);
    let state = done(&state, recruit(herbalists()));
    assert_eq!(state.units(SOLO).len(), 2);
}

// C8 — MKUE p. 5, 5.a et 9.
#[test]
fn activating_spends_the_unit_until_the_round_ends() {
    let state = Units {
        owned: vec![peasants()],
        ..Units::default()
    }
    .state();
    let events = events_of(&state, &activate(peasants(), 2, 0));
    assert_eq!(
        events,
        vec![Event::UnitActivated {
            enemy: None,
            player: SOLO,
            unit: peasants(),
            ability: 2,
            option: 0,
            effect: Effect::Move(2),
        }]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.move_points(SOLO), 2);
    assert!(!owned(&state, peasants()).ready);
    assert!(activations(&state, peasants()).is_empty());
    // La fin du tour ne la rend pas disponible.
    let state = apply(state, &Event::TurnEnded { player: SOLO });
    let state = apply(
        state,
        &Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    );
    assert!(!owned(&state, peasants()).ready);
    assert!(activations(&state, peasants()).is_empty());
}

// C8 — hors combat, Mouvement et Influence ; ni Attaque ni Parade.
#[test]
fn peasants_offer_move_and_influence_out_of_combat() {
    let state = Units {
        owned: vec![peasants()],
        ..Units::default()
    }
    .state();
    assert_eq!(
        activations(&state, peasants()),
        vec![(1, 0, vec![], None), (2, 0, vec![], None)]
    );
    // En interaction, plus de mouvement (spec 040).
    let state = interacting(&state, Site::Village, 0);
    assert_eq!(activations(&state, peasants()), vec![(1, 0, vec![], None)]);
}

// C9 — MKUE p. 11, 2.a.
#[test]
fn a_wounded_unit_cannot_be_activated() {
    let state = Units {
        owned: vec![peasants()],
        ..Units::default()
    }
    .state();
    assert!(!activations(&state, peasants()).is_empty());
    let state = wounded(state, peasants(), 1);
    assert_eq!(owned(&state, peasants()).wounds, 1);
    assert!(activations(&state, peasants()).is_empty());
    refused(&state, activate(peasants(), 2, 0));
}

// C10 (hors combat) — MKUE p. 5, 5.b : un mana de la couleur, comme un effet renforcé.
#[test]
fn paid_abilities_need_mana() {
    let state = Units {
        owned: vec![herbalists()],
        ..Units::default()
    }
    .state();
    let heal = |state: &GameState| -> Vec<Vec<ManaPayment>> {
        activations(state, herbalists())
            .into_iter()
            .filter(|(ability, ..)| *ability == 0)
            .map(|(_, _, mana, _)| mana)
            .collect()
    };
    assert_eq!(heal(&state), vec![vec![ManaPayment::Die(GREEN_DIE)]]);
    let events = events_of(
        &state,
        &activate_paid(herbalists(), 0, 0, &[ManaPayment::Die(GREEN_DIE)]),
    );
    assert_eq!(
        events[..2],
        [
            Event::UnitActivated {
                enemy: None,
                player: SOLO,
                unit: herbalists(),
                ability: 0,
                option: 0,
                effect: Effect::Heal(2),
            },
            Event::DieTaken {
                player: SOLO,
                die: GREEN_DIE,
                color: ManaColor::Basic(BasicColor::Green),
            },
        ]
    );
    let crystal = apply(
        state.clone(),
        &Event::CrystalGained {
            player: SOLO,
            color: BasicColor::Green,
        },
    );
    assert_eq!(
        heal(&crystal),
        vec![
            vec![ManaPayment::Die(GREEN_DIE)],
            vec![ManaPayment::Crystal]
        ]
    );
    // Le dé du tour déjà pris : plus de dé.
    let taken = apply(
        state,
        &Event::DieTaken {
            player: SOLO,
            die: RED_DIE,
            color: ManaColor::Basic(BasicColor::Red),
        },
    );
    assert!(heal(&taken).is_empty());
    refused(
        &taken,
        activate_paid(herbalists(), 0, 0, &[ManaPayment::Die(GREEN_DIE)]),
    );
}

// C15 — cartes/unites.md ; Compendium, *ALTEM MAGES* Q3.
#[test]
fn unit_mana_abilities() {
    let state = Units {
        owned: vec![fire_mages(), altem_mages()],
        ..Units::default()
    }
    .state();
    let events = events_of(&state, &activate(fire_mages(), 2, 0));
    assert_eq!(
        events[1..],
        [
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Basic(BasicColor::Red),
            },
            Event::CrystalGained {
                player: SOLO,
                color: BasicColor::Red,
            },
        ]
    );
    // Les 21 paires de couleurs, la dernière : noir et noir, même de jour.
    let pairs: Vec<u8> = activations(&state, altem_mages())
        .into_iter()
        .filter(|(ability, ..)| *ability == 0)
        .map(|(_, option, ..)| option)
        .collect();
    assert_eq!(pairs, (0..21).collect::<Vec<u8>>());
    let events = events_of(&state, &activate(altem_mages(), 0, 20));
    assert_eq!(
        events[1..],
        [
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Black,
            },
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Black,
            },
        ]
    );
}

// MKUE p. 5, 3.c : un jeton or paie une couleur de base de jour ; un noir, rien encore.
#[test]
fn gold_tokens_pay_any_basic_color_by_day() {
    let state = Units {
        owned: vec![herbalists(), altem_mages()],
        ..Units::default()
    }
    .state();
    // Le dé vert pris : seul un jeton peut payer.
    let state = apply(
        state,
        &Event::DieTaken {
            player: SOLO,
            die: GREEN_DIE,
            color: ManaColor::Basic(BasicColor::Green),
        },
    );
    let black = done(&state, activate(altem_mages(), 0, 20));
    assert!(!active_abilities(&black, herbalists()).contains(&0));
    // Or et noir.
    let gold = done(&state, activate(altem_mages(), 0, 19));
    let heal = activate_paid(herbalists(), 0, 0, &[ManaPayment::Token]);
    let events = events_of(&gold, &heal);
    assert_eq!(
        events[1],
        Event::ManaTokenUsed {
            player: SOLO,
            color: ManaColor::Gold,
        }
    );
    let gold = events.iter().fold(gold, apply);
    assert_eq!(gold.mana_tokens(SOLO), &[ManaColor::Black]);
}

// C16 — spec 042, Q2.
#[test]
fn unsupported_abilities_are_not_offered() {
    // Spec 053 : la capacité blanche des Illusionnistes vise un ennemi du combat, jamais hors
    // combat.
    let state = Units {
        owned: vec![illusionists()],
        ..Units::default()
    }
    .state();
    refused(&state, activate(illusionists(), 1, 0));
    // Influence 4 et le cristal blanc, pas la capacité blanche.
    assert_eq!(active_abilities(&state, illusionists()), vec![0, 2]);
}

// C17 — FAQ p2 : Concentration renforcée ne vise que les cartes de la main.
#[test]
fn concentration_does_not_boost_units() {
    let concentration = card(Concentration, 1);
    let state = Units {
        owned: vec![peasants()],
        hand: vec![concentration, card(March, 1)],
        ..Units::default()
    }
    .state();
    let targets: Vec<CardId> = legal(&state)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard {
                play:
                    CardPlay::Strong {
                        with: Some(target), ..
                    },
                ..
            } => Some(target.card),
            _ => None,
        })
        .collect();
    assert!(!targets.is_empty());
    assert!(targets.iter().all(|target| *target == card(March, 1)));
}

// C18 — MKUE p. 5, 6 ; Compendium, *Silver Units* S10.
#[test]
fn herbalists_ready_a_unit() {
    let state = Units {
        owned: vec![peasants(), herbalists(), fire_golems()],
        ..Units::default()
    }
    .state();
    // Aucune unité épuisée : rien à rendre disponible.
    assert!(!active_abilities(&state, herbalists()).contains(&1));
    let state = spent(spent(state, peasants()), fire_golems());
    let readying: Vec<Option<UnitCard>> = activations(&state, herbalists())
        .into_iter()
        .filter(|(ability, ..)| *ability == 1)
        .map(|(.., target)| target)
        .collect();
    // Les Golems de feu, niveau III, ne sont pas une cible.
    assert_eq!(readying, vec![Some(peasants())]);
    let action = Action::ActivateUnit {
        enemy: None,
        unit: herbalists(),
        ability: 1,
        option: 0,
        mana: Vec::new(),
        target: Some(peasants()),
        leadership: false,
    };
    let events = events_of(&state, &action);
    assert_eq!(
        events,
        vec![
            Event::UnitActivated {
                enemy: None,
                player: SOLO,
                unit: herbalists(),
                ability: 1,
                option: 0,
                effect: Effect::ReadyUnit(2),
            },
            Event::UnitReadied {
                player: SOLO,
                unit: peasants(),
            },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert!(owned(&state, peasants()).ready);
    assert!(!activations(&state, peasants()).is_empty());
}

// C18 — une unité blessée rendue disponible reste inactivable (Compendium, *Silver Units* S10).
#[test]
fn a_readied_wounded_unit_stays_inactive() {
    let state = Units {
        owned: vec![peasants(), herbalists()],
        ..Units::default()
    }
    .state();
    let state = wounded(spent(state, peasants()), peasants(), 1);
    let state = done(
        &state,
        Action::ActivateUnit {
            enemy: None,
            unit: herbalists(),
            ability: 1,
            option: 0,
            mana: Vec::new(),
            target: Some(peasants()),
            leadership: false,
        },
    );
    assert!(owned(&state, peasants()).ready);
    assert!(activations(&state, peasants()).is_empty());
}

fn ready_with(card: CardId, play: CardPlay, target: UnitCard) -> Action {
    Action::PlayCardOnUnit { card, play, target }
}

// C19 — cartes/actions-de-base.md : Rajeunissement rend disponible une unité.
#[test]
fn rejuvenate_readies_a_unit() {
    let rejuvenate = card(Rejuvenate, 1);
    let state = Units {
        owned: vec![peasants(), fire_golems()],
        hand: vec![rejuvenate, card(March, 1)],
        ..Units::default()
    }
    .state();
    let state = spent(spent(state, peasants()), fire_golems());
    let basic = CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option: 3,
        discard: None,
        mana: None,
    };
    let strong = CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option: 3,
        discard: None,
        mana: ManaPayment::Die(GREEN_DIE),
        set_dice: [None, None],
        with: None,
    };
    assert!(legal(&state).contains(&ready_with(rejuvenate, basic, peasants())));
    refused(&state, ready_with(rejuvenate, basic, fire_golems()));
    let events = events_of(&state, &ready_with(rejuvenate, strong, fire_golems()));
    assert_eq!(
        events.last(),
        Some(&Event::UnitReadied {
            player: SOLO,
            unit: fire_golems(),
        })
    );
    let state = events.iter().fold(state, apply);
    assert!(owned(&state, fire_golems()).ready);
    assert_eq!(state.play_area(SOLO).len(), 1);
}

// C20 — MKUE p. 4, 2.g.
#[test]
fn units_are_readied_at_the_new_round() {
    let state = Units {
        owned: vec![peasants(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = spent(spent(state, peasants()), guardsmen());
    let state = wounded(state, guardsmen(), 1);
    // Deck vide au début du tour : la fin de manche s'annonce.
    let deck = state.deck(SOLO).to_vec();
    let state = deck.into_iter().fold(state, |state, card| {
        apply(state, &Event::CardDrawn { player: SOLO, card })
    });
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert!(events.contains(&Event::UnitsReadied { player: SOLO }));
    let state = events.iter().fold(state, apply);
    assert!(owned(&state, peasants()).ready);
    assert!(owned(&state, guardsmen()).ready);
    assert_eq!(owned(&state, guardsmen()).wounds, 1);
}

// L2 — MKUE p. 5, 4.a : une unité épuisée ou blessée se renvoie, Blessures comprises.
#[test]
fn a_spent_or_wounded_unit_can_be_disbanded() {
    let state = Units {
        owned: vec![peasants()],
        offer: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    let state = wounded(spent(state, peasants()), peasants(), 2);
    let state = interacting(&state, Site::Village, 5);
    let state = done(&state, recruit_disbanding(guardsmen(), peasants()));
    assert_eq!(state.units(SOLO), &[OwnedUnit::recruited(guardsmen())]);
}

// L3 — spec 040 : l'influence d'avant l'interaction est perdue.
#[test]
fn influence_before_interacting_is_lost() {
    let state = Units {
        owned: vec![peasants()],
        ..Units::default()
    }
    .state();
    let state = done(&state, activate(peasants(), 1, 0));
    assert_eq!(state.influence_points(SOLO), 2);
    let state = done(&state, Action::Interact);
    assert_eq!(state.influence_points(SOLO), 0);
}

// L8 — Compendium, *Resting* S3 : activer une unité n'empêche pas le repos.
#[test]
fn activating_a_unit_keeps_resting_possible() {
    let state = Units {
        owned: vec![peasants()],
        ..Units::default()
    }
    .state();
    let state = done(&state, activate(peasants(), 2, 0));
    assert!(state.can_rest(SOLO));
}

// --- Unités en combat ---

fn red_cape_monks() -> UnitCard {
    unit(Unit::RedCapeMonks, 1)
}

fn crossbowmen() -> UnitCard {
    unit(Unit::UtemCrossbowmen, 1)
}

fn catapults() -> UnitCard {
    unit(Unit::Catapults, 1)
}

fn guardian_golems() -> UnitCard {
    unit(Unit::GuardianGolems, 1)
}

fn swordsmen() -> UnitCard {
    unit(Unit::UtemSwordsmen, 1)
}

/// Combat engagé contre cet ennemi, posé sur les orques ②, voisins du village, avec ces unités.
fn fighting(enemy: Enemy, owned: &[UnitCard]) -> GameState {
    let state = Units {
        owned: owned.to_vec(),
        placed: vec![(token(enemy, 1), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

/// L'attaque en cours contre l'ennemi, en phase d'attaque.
fn attacking_with(enemy: Enemy, owned: &[UnitCard]) -> GameState {
    let state = skip_to_attack_phase(&fighting(enemy, owned));
    done(&state, declare(&[foe(&state, ORCS_2)]))
}

/// La parade en cours contre l'ennemi.
fn blocking_with(enemy: Enemy, owned: &[UnitCard]) -> GameState {
    let state = to_block_phase(&fighting(enemy, owned));
    done(&state, declare_block(foe(&state, ORCS_2)))
}

fn in_progress_total(state: &GameState) -> Option<u32> {
    state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress_total())
}

// C10 — MKUE p. 5, 5.b : Attaque de feu 4 des Moines à cape rouge, avec un mana rouge.
#[test]
fn mana_abilities_need_mana() {
    let state = attacking_with(Enemy::Prowlers, &[red_cape_monks()]);
    let fire = |state: &GameState| -> Vec<Vec<ManaPayment>> {
        activations(state, red_cape_monks())
            .into_iter()
            .filter(|(ability, option, ..)| (*ability, *option) == (1, 0))
            .map(|(_, _, mana, _)| mana)
            .collect()
    };
    assert_eq!(fire(&state), vec![vec![ManaPayment::Die(RED_DIE)]]);
    let action = activate_paid(red_cape_monks(), 1, 0, &[ManaPayment::Die(RED_DIE)]);
    let events = events_of(&state, &action);
    assert_eq!(
        events[1],
        Event::DieTaken {
            player: SOLO,
            die: RED_DIE,
            color: ManaColor::Basic(BasicColor::Red),
        }
    );
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(in_progress_total(&after), Some(4));
    // Le dé compte comme le dé du tour : plus de dé ensuite.
    assert!(after.dice_taken(SOLO).contains(&RED_DIE));
    // Cristal et jeton rouges.
    let crystal = apply(
        state.clone(),
        &Event::CrystalGained {
            player: SOLO,
            color: BasicColor::Red,
        },
    );
    assert!(fire(&crystal).contains(&vec![ManaPayment::Crystal]));
    let token = apply(
        state.clone(),
        &Event::ManaTokenGained {
            player: SOLO,
            color: ManaColor::Basic(BasicColor::Red),
        },
    );
    assert!(fire(&token).contains(&vec![ManaPayment::Token]));
    // Sans mana rouge : refusée.
    let without = apply(
        state,
        &Event::DieTaken {
            player: SOLO,
            die: GREEN_DIE,
            color: ManaColor::Basic(BasicColor::Green),
        },
    );
    assert!(fire(&without).is_empty());
    refused(&without, action);
}

// C10 — le dé doré paie toute couleur de base de jour (spec 022).
#[test]
fn a_gold_die_pays_a_unit_ability_by_day() {
    let state = Units {
        owned: vec![red_cape_monks()],
        placed: vec![(token(Enemy::Prowlers, 1), ORCS_2)],
        dice: vec![ManaColor::Gold, ManaColor::Basic(BasicColor::Blue)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = skip_to_attack_phase(&done(&state, Action::StartCombat));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    assert!(legal(&state).contains(&activate_paid(
        red_cape_monks(),
        1,
        0,
        &[ManaPayment::Die(0)]
    )));
}

// C11 — MKUE p. 9, 3.a et 4.a : les capacités suivent les phases du combat.
#[test]
fn unit_abilities_follow_combat_phases() {
    // Phase d'attaque à distance : pas d'attaque de mêlée, l'Attaque à distance si.
    let ranged = fighting(Enemy::Prowlers, &[peasants(), crossbowmen()]);
    assert!(activations(&ranged, peasants()).is_empty());
    let ranged = done(&ranged, declare(&[foe(&ranged, ORCS_2)]));
    assert!(activations(&ranged, peasants()).is_empty());
    assert_eq!(active_abilities(&ranged, crossbowmen()), vec![1]);
    // Un fortifié : siège seulement.
    let fortified = done(
        &fighting(Enemy::Diggers, &[crossbowmen(), catapults()]),
        declare(&[token(Enemy::Diggers, 1)]),
    );
    assert!(active_abilities(&fortified, crossbowmen()).is_empty());
    // Les trois capacités des Catapultes sont des sièges (rouge et bleu payés par les dés).
    assert_eq!(active_abilities(&fortified, catapults()), vec![0, 1, 2]);
    // Phase d'attaque : toute attaque, dans une attaque en cours.
    let attack = attacking_with(Enemy::Prowlers, &[peasants()]);
    assert_eq!(activations(&attack, peasants()), vec![(0, 0, vec![], None)]);
    // Parade en cours : la parade seulement.
    let block = blocking_with(Enemy::Prowlers, &[peasants()]);
    assert_eq!(activations(&block, peasants()), vec![(0, 1, vec![], None)]);
    // Sans parade en cours : rien.
    let phase = to_block_phase(&fighting(Enemy::Prowlers, &[peasants()]));
    assert!(activations(&phase, peasants()).is_empty());
}

// C12 — MKUE p. 9, *Block Phase* 5 : les parades d'unités ont leur élément.
#[test]
fn unit_blocks_have_elements() {
    let state = blocking_with(Enemy::FireMages, &[guardian_golems(), peasants()]);
    let state = done(
        &state,
        activate_paid(guardian_golems(), 2, 0, &[ManaPayment::Die(BLUE_DIE)]),
    );
    assert_eq!(in_progress_total(&state), Some(4));
    let state = done(&state, activate(peasants(), 0, 1));
    // 4 de glace efficaces, 2 physiques divisés par deux.
    assert_eq!(in_progress_total(&state), Some(5));
}

// C13 — cartes/unites.md, Lost Legion : la parade des Gardes d'Utem compte double contre Vif.
#[test]
fn guardsmen_block_counts_twice_against_swift() {
    let state = blocking_with(Enemy::Crossbowmen, &[guardsmen(), peasants()]);
    let guarded = done(&state, activate(guardsmen(), 1, 0));
    assert_eq!(in_progress_total(&guarded), Some(8));
    let events = events_of(&guarded, &Action::ResolveBlock);
    assert_eq!(
        events,
        vec![Event::EnemyBlocked {
            player: SOLO,
            enemy: foe(&state, ORCS_2)
        }]
    );
    let peasants_only = done(&state, activate(peasants(), 0, 1));
    assert_eq!(in_progress_total(&peasants_only), Some(2));
    // Contre un ennemi sans Vif, Parade 4.
    let plain = done(
        &blocking_with(Enemy::Prowlers, &[guardsmen()]),
        activate(guardsmen(), 1, 0),
    );
    assert_eq!(in_progress_total(&plain), Some(4));
}

// C14 — Compendium, *UTEM SWORDSMEN* S2 : la Blessure n'est pas un dégât assigné.
#[test]
fn swordsmen_strong_ability_wounds_them() {
    let state = blocking_with(Enemy::Medusa, &[swordsmen()]);
    let events = events_of(&state, &activate(swordsmen(), 1, 1));
    assert_eq!(
        events.last(),
        Some(&Event::UnitWounded {
            player: SOLO,
            unit: swordsmen(),
            wounds: 1,
        })
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(in_progress_total(&state), Some(6));
    let state = done(&state, Action::ResolveBlock);
    assert_eq!(owned(&state, swordsmen()).wounds, 1);
    assert!(!state.units(SOLO).is_empty());
}

// C18 — Compendium, *HERBALIST* S1 : ni soin ni « rendre disponible » en combat ; le jeton vert si.
#[test]
fn herbalists_heal_and_ready_out_of_combat_only() {
    let state = fighting(Enemy::Prowlers, &[peasants(), herbalists()]);
    let state = spent(state, peasants());
    assert_eq!(active_abilities(&state, herbalists()), vec![2]);
}

// --- Assigner les dégâts ---

fn altem_mages_unit() -> UnitCard {
    altem_mages()
}

/// Combat engagé contre ces ennemis, chacun sur sa case voisine du village, défiés dans l'ordre.
fn fighting_all(enemies: &[(Enemy, Hex)], owned: &[UnitCard]) -> GameState {
    let mut state = Units {
        owned: owned.to_vec(),
        placed: enemies
            .iter()
            .map(|&(enemy, hex)| (token(enemy, 1), hex))
            .collect(),
        ..Units::default()
    }
    .state();
    for &(_, hex) in enemies {
        state = done(&state, challenge(hex));
    }
    done(&state, Action::StartCombat)
}

/// Depuis la phase de parade, sans parer : la phase d'assignation des dégâts.
fn to_damage(state: &GameState) -> GameState {
    let state = to_block_phase(state);
    done(&state, Action::NextCombatPhase)
}

fn assign(enemy: EnemyToken, unit: UnitCard) -> Action {
    Action::AssignDamage {
        enemy,
        to: DamageTarget::Unit(unit),
    }
}

fn to_hero(enemy: EnemyToken) -> Action {
    Action::AssignDamage {
        enemy,
        to: DamageTarget::Hero,
    }
}

fn assignments(state: &GameState) -> Vec<Action> {
    legal(state)
        .into_iter()
        .filter(|action| matches!(action, Action::AssignDamage { .. }))
        .collect()
}

fn unit_damage(enemy: EnemyToken, unit: UnitCard, wounds: u8) -> Event {
    Event::DamageAssignedToUnit {
        player: SOLO,
        enemy,
        unit,
        wounds,
    }
}

fn hero_wounds(enemy: EnemyToken, copies: std::ops::RangeInclusive<u16>) -> Event {
    Event::WoundsTaken {
        player: SOLO,
        enemy,
        wounds: copies.map(wound).collect(),
    }
}

fn attack_phase() -> Event {
    Event::CombatPhaseStarted {
        player: SOLO,
        phase: arcanum_core::CombatPhase::Attack,
    }
}

fn wounds_in_hand(state: &GameState) -> usize {
    state
        .hand(SOLO)
        .iter()
        .filter(|card| card.card().is_wound())
        .count()
}

// C21 — MKUE p. 9, 4.a et 5 ; exemple de la spec.
#[test]
fn damage_goes_to_units_then_hero() {
    let state = fighting(Enemy::Prowlers, &[peasants(), guardsmen()]);
    let state = to_block_phase(&state);
    assert_eq!(
        events_of(&state, &Action::NextCombatPhase),
        vec![Event::CombatPhaseStarted {
            player: SOLO,
            phase: arcanum_core::CombatPhase::AssignDamage,
        }]
    );
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(
        assignments(&state),
        vec![
            assign(foe(&state, ORCS_2), peasants()),
            assign(foe(&state, ORCS_2), guardsmen()),
            to_hero(foe(&state, ORCS_2))
        ]
    );
    let events = events_of(&state, &assign(foe(&state, ORCS_2), peasants()));
    assert_eq!(
        events,
        vec![unit_damage(foe(&state, ORCS_2), peasants(), 1)]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(owned(&state, peasants()).wounds, 1);
    // Il reste 1 dégât : aux Gardes ou au héros.
    assert_eq!(
        assignments(&state),
        vec![
            assign(foe(&state, ORCS_2), guardsmen()),
            to_hero(foe(&state, ORCS_2))
        ]
    );
    let events = events_of(&state, &to_hero(foe(&state, ORCS_2)));
    assert_eq!(
        events,
        vec![hero_wounds(foe(&state, ORCS_2), 1..=1), attack_phase()]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(wounds_in_hand(&state), 1);
}

// C22 — MKUE p. 9, 4.a : une unité absorbe tous les dégâts.
#[test]
fn a_unit_can_absorb_all_damage() {
    let state = to_damage(&fighting(Enemy::Prowlers, &[guardsmen()]));
    let events = events_of(&state, &assign(foe(&state, ORCS_2), guardsmen()));
    assert_eq!(
        events,
        vec![
            unit_damage(foe(&state, ORCS_2), guardsmen(), 1),
            attack_phase()
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(wounds_in_hand(&state), 0);
}

// L6 — plus aucune unité possible : le reste va au héros, sans décision.
#[test]
fn the_rest_goes_to_the_hero_without_a_unit_left() {
    let state = to_damage(&fighting(Enemy::Prowlers, &[peasants()]));
    let events = events_of(&state, &assign(foe(&state, ORCS_2), peasants()));
    assert_eq!(
        events,
        vec![
            unit_damage(foe(&state, ORCS_2), peasants(), 1),
            hero_wounds(foe(&state, ORCS_2), 1..=1),
            attack_phase()
        ]
    );
}

// C23 — MKUE p. 9, 4 : épuisée oui, blessée non.
#[test]
fn spent_units_can_take_damage() {
    let state = fighting(Enemy::Prowlers, &[peasants(), guardsmen()]);
    let state = wounded(spent(state, guardsmen()), peasants(), 1);
    let state = spent(state, guardsmen());
    let state = to_damage(&state);
    assert_eq!(
        assignments(&state),
        vec![
            assign(foe(&state, ORCS_2), guardsmen()),
            to_hero(foe(&state, ORCS_2))
        ]
    );
}

// C24 — MKUE p. 9, 4.b ; Compendium, *Combat* S34.
#[test]
fn resistant_units_absorb_twice_their_armor() {
    // 3 dégâts physiques : absorbés sans Blessure.
    let state = to_damage(&fighting(Enemy::Diggers, &[guardian_golems()]));
    assert_eq!(
        events_of(&state, &assign(foe(&state, ORCS_2), guardian_golems())),
        vec![
            unit_damage(foe(&state, ORCS_2), guardian_golems(), 0),
            attack_phase()
        ]
    );
    // 4 : une Blessure, plus rien.
    let state = to_damage(&fighting(Enemy::Prowlers, &[guardian_golems()]));
    assert_eq!(
        events_of(&state, &assign(foe(&state, ORCS_2), guardian_golems())),
        vec![
            unit_damage(foe(&state, ORCS_2), guardian_golems(), 1),
            attack_phase()
        ]
    );
    // 8 (brutal) : une Blessure, 2 pour le héros.
    let state = to_damage(&fighting(Enemy::Ironclads, &[guardian_golems()]));
    assert_eq!(
        events_of(&state, &assign(foe(&state, ORCS_2), guardian_golems())),
        vec![
            unit_damage(foe(&state, ORCS_2), guardian_golems(), 1),
            hero_wounds(foe(&state, ORCS_2), 1..=1),
            attack_phase()
        ]
    );
}

// C25 — FAQ p3 ; Compendium, *Combat* S35 : des dégâts une fois par combat.
#[test]
fn a_unit_takes_damage_once_per_combat() {
    let state = fighting_all(
        &[(Enemy::Diggers, ORCS_2), (Enemy::Prowlers, PLAIN_W)],
        &[guardian_golems(), peasants()],
    );
    let state = to_damage(&state);
    let state = done(&state, assign(foe(&state, ORCS_2), guardian_golems()));
    assert_eq!(owned(&state, guardian_golems()).wounds, 0);
    assert_eq!(
        assignments(&state),
        vec![
            assign(foe(&state, PLAIN_W), peasants()),
            to_hero(foe(&state, PLAIN_W))
        ]
    );
    let state = done(&state, to_hero(foe(&state, PLAIN_W)));
    // Non blessés, les Golems restent activables en phase d'attaque.
    let state = done(&state, declare(&[foe(&state, PLAIN_W)]));
    assert!(active_abilities(&state, guardian_golems()).contains(&0));
}

// C26 — MKUE p. 9, 4.b : le feu glacé demande les deux résistances.
#[test]
fn cold_fire_needs_both_resistances() {
    // Feu glacé 6, brutal : 12. Golems de feu (armure 4) : Blessure d'emblée, reste 8.
    let state = to_damage(&fighting(Enemy::HighDragon, &[fire_golems()]));
    let events = events_of(&state, &assign(foe(&state, ORCS_2), fire_golems()));
    assert_eq!(
        events[..2],
        [
            unit_damage(foe(&state, ORCS_2), fire_golems(), 1),
            hero_wounds(foe(&state, ORCS_2), 1..=4)
        ]
    );
    // Mages d'Altem (armure 5, feu et glace) : 12 − 5 − 5, une Blessure, reste 2.
    let state = to_damage(&fighting(Enemy::HighDragon, &[altem_mages_unit()]));
    let events = events_of(&state, &assign(foe(&state, ORCS_2), altem_mages_unit()));
    assert_eq!(
        events[..2],
        [
            unit_damage(foe(&state, ORCS_2), altem_mages_unit(), 1),
            hero_wounds(foe(&state, ORCS_2), 1..=1)
        ]
    );
}

// C27 — MKUE p. 10, 6.a et 6.b ; Compendium, *Combat* S47.
#[test]
fn poison_and_paralyze_on_units() {
    let state = to_damage(&fighting(Enemy::CursedHags, &[guardsmen()]));
    let state = done(&state, assign(foe(&state, ORCS_2), guardsmen()));
    assert_eq!(owned(&state, guardsmen()).wounds, 2);
    // Méduse (6, paralysant) contre les Gardes (armure 5) : détruits, 1 pour le héros.
    let state = to_damage(&fighting(Enemy::Medusa, &[guardsmen()]));
    let events = events_of(&state, &assign(foe(&state, ORCS_2), guardsmen()));
    assert_eq!(
        events[..3],
        [
            unit_damage(foe(&state, ORCS_2), guardsmen(), 1),
            Event::UnitDestroyed {
                player: SOLO,
                unit: guardsmen(),
            },
            hero_wounds(foe(&state, ORCS_2), 1..=1),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert!(state.units(SOLO).is_empty());
}

// C28 — MKUE p. 9, 1.b ; Compendium, *Combat* S34 : l'ordre est choisi.
#[test]
fn damage_order_is_chosen() {
    let state = fighting_all(
        &[(Enemy::Prowlers, ORCS_2), (Enemy::WolfRiders, PLAIN_W)],
        &[peasants(), guardsmen()],
    );
    let state = to_damage(&state);
    assert_eq!(
        assignments(&state),
        vec![
            assign(foe(&state, ORCS_2), peasants()),
            assign(foe(&state, ORCS_2), guardsmen()),
            to_hero(foe(&state, ORCS_2)),
            assign(foe(&state, PLAIN_W), peasants()),
            assign(foe(&state, PLAIN_W), guardsmen()),
            to_hero(foe(&state, PLAIN_W)),
        ]
    );
    // Les loups d'abord, au héros : plus d'unité pour eux.
    let events = events_of(&state, &to_hero(foe(&state, PLAIN_W)));
    assert_eq!(events, vec![hero_wounds(foe(&state, PLAIN_W), 1..=2)]);
    let state = events.iter().fold(state, apply);
    assert_eq!(
        assignments(&state),
        vec![
            assign(foe(&state, ORCS_2), peasants()),
            assign(foe(&state, ORCS_2), guardsmen()),
            to_hero(foe(&state, ORCS_2))
        ]
    );
    // Un ennemi commencé est fini avant le suivant.
    let state = to_damage(&fighting_all(
        &[(Enemy::Prowlers, ORCS_2), (Enemy::WolfRiders, PLAIN_W)],
        &[peasants(), guardsmen()],
    ));
    let state = done(&state, assign(foe(&state, ORCS_2), peasants()));
    assert_eq!(
        assignments(&state),
        vec![
            assign(foe(&state, ORCS_2), guardsmen()),
            to_hero(foe(&state, ORCS_2))
        ]
    );
}

// C29 — spec 031, Q3 : sans unité possible, la phase se résout seule.
#[test]
fn no_eligible_unit_resolves_damage_alone() {
    let state = fighting(Enemy::Prowlers, &[peasants()]);
    let state = to_block_phase(&wounded(state, peasants(), 1));
    assert_eq!(
        events_of(&state, &Action::NextCombatPhase),
        vec![
            Event::CombatPhaseStarted {
                player: SOLO,
                phase: arcanum_core::CombatPhase::AssignDamage,
            },
            hero_wounds(foe(&state, ORCS_2), 1..=2),
            attack_phase()
        ]
    );
}

// C30 — MKUE p. 9, 5.c ; Compendium, *Combat* S45 : assommé, les unités combattent encore.
#[test]
fn units_fight_on_when_knocked_out() {
    let state = fighting_all(
        &[(Enemy::Minotaur, ORCS_2), (Enemy::Prowlers, PLAIN_W)],
        &[peasants(), guardsmen()],
    );
    let state = to_damage(&state);
    // Minotaure brutal : 10 dégâts, 5 Blessures, assommé.
    let events = events_of(&state, &to_hero(foe(&state, ORCS_2)));
    assert!(events.iter().any(|e| matches!(e, Event::KnockedOut { .. })));
    let state = events.iter().fold(state, apply);
    assert!(state.combat(SOLO).is_some_and(|combat| combat.knocked_out));
    let state = done(&state, assign(foe(&state, PLAIN_W), peasants()));
    let state = done(&state, to_hero(foe(&state, PLAIN_W)));
    let state = done(&state, declare(&[foe(&state, PLAIN_W)]));
    assert!(active_abilities(&state, guardsmen()).contains(&0));
}

// L7 — spec 033 : les dégâts de la créature invoquée s'assignent aussi aux unités.
#[test]
fn summoned_damage_goes_to_units() {
    let state = fighting(Enemy::OrcSummoners, &[guardsmen()]);
    let state = to_damage(&state);
    let summoned = state
        .combat(SOLO)
        .and_then(|combat| combat.enemies[0].summoned);
    assert!(summoned.is_some());
    let events = events_of(&state, &assign(foe(&state, ORCS_2), guardsmen()));
    assert_eq!(events[0], unit_damage(foe(&state, ORCS_2), guardsmen(), 1));
    assert!(events.contains(&Event::SummonedEnemyDiscarded {
        player: SOLO,
        summoner: foe(&state, ORCS_2),
    }));
}

// --- Points de soin ---

fn tranquility() -> CardId {
    card(Tranquility, 1)
}

/// `amount` points de soin de plus, par un événement : la valeur seule compte.
fn with_healing(state: GameState, amount: u8) -> GameState {
    let played = state.hand(SOLO)[0];
    apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: played,
            play: CardPlay::Sideways(SidewaysEffect::Move),
            effect: Effect::Heal(amount),
        },
    )
}

fn heal(on: HealingTarget) -> Action {
    Action::SpendHealing { on }
}

fn healings(state: &GameState) -> Vec<Action> {
    legal(state)
        .into_iter()
        .filter(|action| matches!(action, Action::SpendHealing { .. }))
        .collect()
}

// C31 — MKUE p. 11, 3.c ; Q1.
#[test]
fn healing_points_are_a_reserve() {
    let state = Units {
        hand: vec![tranquility(), wound(1), card(March, 1)],
        ..Units::default()
    }
    .state();
    let events = events_of(&state, &option(tranquility(), 0));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::WoundsHealed { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.healing_points(SOLO), 1);
    assert_eq!(wounds_in_hand(&state), 1);
    assert_eq!(healings(&state), vec![heal(HealingTarget::Wound)]);
    let events = events_of(&state, &heal(HealingTarget::Wound));
    assert_eq!(
        events,
        vec![Event::WoundsHealed {
            player: SOLO,
            wounds: vec![wound(1)],
        }]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.healing_points(SOLO), 0);
    assert_eq!(wounds_in_hand(&state), 0);
    assert!(healings(&state).is_empty());
}

// C32 — MKUE p. 11, 3.c.
#[test]
fn healing_a_unit_costs_its_level() {
    let state = Units {
        owned: vec![guardsmen(), herbalists()],
        ..Units::default()
    }
    .state();
    let state = wounded(state, guardsmen(), 1);
    let state = done(
        &state,
        activate_paid(herbalists(), 0, 0, &[ManaPayment::Die(GREEN_DIE)]),
    );
    assert_eq!(state.healing_points(SOLO), 2);
    let on_guardsmen = heal(HealingTarget::Unit(guardsmen()));
    assert_eq!(healings(&state), vec![on_guardsmen.clone()]);
    let events = events_of(&state, &on_guardsmen);
    assert_eq!(
        events,
        vec![Event::UnitHealed {
            player: SOLO,
            unit: guardsmen(),
            cost: 2,
        }]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.healing_points(SOLO), 0);
    assert_eq!(owned(&state, guardsmen()).wounds, 0);

    // Un seul point : pas assez pour une unité de niveau II.
    let state = Units {
        owned: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    let state = with_healing(wounded(state, guardsmen(), 1), 1);
    assert!(healings(&state).is_empty());
    refused(&state, heal(HealingTarget::Unit(guardsmen())));
}

// C33 — FAQ p3.
#[test]
fn a_unit_heals_one_wound_at_a_time() {
    let state = Units {
        owned: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    let state = with_healing(wounded(state, guardsmen(), 2), 4);
    let on_guardsmen = heal(HealingTarget::Unit(guardsmen()));
    let state = done(&state, on_guardsmen.clone());
    assert_eq!(owned(&state, guardsmen()).wounds, 1);
    assert_eq!(state.healing_points(SOLO), 2);
    let state = done(&state, on_guardsmen.clone());
    assert_eq!(owned(&state, guardsmen()).wounds, 0);
    assert_eq!(state.healing_points(SOLO), 0);
    refused(&state, on_guardsmen);
}

// C34 — MKUE p. 11, 3.d.
#[test]
fn healing_points_are_lost() {
    let state = Units {
        hand: vec![tranquility(), wound(1), card(March, 1)],
        placed: vec![(token(Enemy::Prowlers, 1), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, option(tranquility(), 0));
    assert_eq!(state.healing_points(SOLO), 1);
    let fighting = done(&done(&state, challenge(ORCS_2)), Action::StartCombat);
    assert_eq!(fighting.healing_points(SOLO), 0);
    // Pas de soin en combat.
    assert!(healings(&fighting).is_empty());

    let ended = done(
        &state,
        Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    );
    assert_eq!(ended.healing_points(SOLO), 0);
}

// L4 — spec 034, *Soin en trop*.
#[test]
fn healing_points_without_wounds_wait_for_the_end_of_turn() {
    let state = Units {
        hand: vec![tranquility(), card(March, 1)],
        owned: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    let state = done(&state, option(tranquility(), 0));
    assert_eq!(state.healing_points(SOLO), 1);
    assert!(healings(&state).is_empty());
    let state = done(&state, sideways(card(March, 1)));
    assert_eq!(state.healing_points(SOLO), 1);
}

// --- Décompte, retour en arrière, vue ---

// C35 — MKUE p. 15, *Greatest Leader*.
#[test]
fn leader_counts_unit_levels() {
    let state = Units {
        owned: vec![peasants(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = wounded(state, guardsmen(), 1);
    let score = arcanum_core::final_score(&state, SOLO);
    // Paysans (I) : 1 ; Gardes d'Utem (II) blessés : 2 / 2 = 1.
    assert_eq!(score.unit_levels, 2);
    assert_eq!(score.points(arcanum_core::Achievement::Leader), 2);
}

/// Le journal : la mise en place en décision 0, puis une décision par action ; la dernière est
/// reprise, l'état revient à celui d'avant.
fn undo_restores(setup: Vec<Event>, actions: &[Action]) {
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

// C36 — spec 024.
#[test]
fn units_can_be_undone() {
    let decor = Units {
        offer: vec![peasants()],
        ..Units::default()
    };
    let mut setup = decor.events();
    setup.push(Event::InteractionStarted {
        player: SOLO,
        site: Site::Village,
        bonus: 4,
        city_shields: 0,
    });
    undo_restores(setup, &[recruit(peasants())]);

    let decor = Units {
        owned: vec![peasants()],
        ..Units::default()
    };
    undo_restores(decor.events(), &[activate(peasants(), 2, 0)]);

    let decor = Units {
        owned: vec![peasants()],
        placed: vec![(token(Enemy::Prowlers, 1), ORCS_2)],
        ..Units::default()
    };
    undo_restores(
        decor.events(),
        &[
            challenge(ORCS_2),
            Action::StartCombat,
            Action::NextCombatPhase,
            Action::NextCombatPhase,
            assign(token(Enemy::Prowlers, 1), peasants()),
        ],
    );
}

// C38 — spec 042, *Vue du joueur*.
#[test]
fn view_shows_units() {
    let state = Units {
        owned: vec![peasants(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = with_healing(wounded(spent(state, peasants()), guardsmen(), 1), 2);
    let view = arcanum_core::view(&state, SOLO);
    assert_eq!(
        view.units,
        vec![
            OwnedUnit {
                ready: false,
                ..OwnedUnit::recruited(peasants())
            },
            OwnedUnit {
                wounds: 1,
                ..OwnedUnit::recruited(guardsmen())
            },
        ]
    );
    assert_eq!(view.healing_points, 2);

    // En combat : les dégâts en cours d'assignation sont dans le combat de la vue.
    let state = to_damage(&fighting(Enemy::Prowlers, &[peasants(), guardsmen()]));
    let state = done(&state, assign(foe(&state, ORCS_2), peasants()));
    let view = arcanum_core::view(&state, SOLO);
    let combat = view.combat.expect("combat en cours");
    assert_eq!(combat.assigning, Some((foe(&state, ORCS_2), 1)));
    assert_eq!(view.units[0].wounds, 1);
}

// C45 — Issue privée 341 ; MKUE p. 8, 4.a et 4.e : la vue donne les unités de l'offre recrutables dans
// le lieu de l'interaction, quelle que soit l'Influence, pour que l'écran les montre grisées.
#[test]
fn view_lists_units_recruitable_here_whatever_the_influence() {
    let state = Units {
        offer: vec![peasants(), illusionists(), guardsmen(), herbalists()],
        ..Units::default()
    }
    .state();
    assert!(
        arcanum_core::view(&state, SOLO)
            .recruitable_units
            .is_empty()
    );
    let village = interacting(&state, Site::Village, 0);
    assert!(recruitable(&village).is_empty());
    assert_eq!(
        arcanum_core::view(&village, SOLO).recruitable_units,
        vec![peasants(), guardsmen(), herbalists()]
    );
    let penalty = interacting(&state, Site::Monastery, -2);
    assert_eq!(
        arcanum_core::view(&penalty, SOLO).recruitable_units,
        vec![illusionists(), herbalists()]
    );
    // MKUE p. 8, 4.e : dans la cité blanche, toutes les unités.
    let white = interacting(&state, Site::City(BasicColor::White), 0);
    assert_eq!(
        arcanum_core::view(&white, SOLO).recruitable_units,
        offered_units(&state)
    );
}
