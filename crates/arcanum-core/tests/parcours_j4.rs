//! Parcours de bout en bout du jalon 4 (spec 049, ADR 0006) : une partie complète de Première
//! reconnaissance en solo, du recrutement d'une unité au décompte final.

mod support;

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, CombatPhase, CommonCard, DamageTarget, DeedCard, Effect,
    EnemyColor, Event, EventLog, GameConfig, GameContent, GameEndReason, Hero, Hex, LastTurn,
    Offer, PlayerView, RecruitSite, RngPositions, Scenario, SidewaysEffect, Site, Terrain, TileId,
    TilePosition, TimeOfDay, UnitCard, UnitRank,
};
use support::{
    Parcours, SOLO, all_events, card_plays, combat, expect_terrain, has, influence_amount,
    melee_attack, move_amount, precondition, route_to, single_card, step_cost, unit_to_recruit,
};

/// Graine choisie pour que les préconditions du parcours tiennent (spec 049, « Graine et
/// préconditions »). Si un test échoue sur « graine à rechoisir », en chercher une autre avec
/// `chercher_une_graine`.
const SEED: u64 = 38;

const PORTAL: Hex = Hex::new(0, 0);
const VILLAGE: Hex = Hex::new(4, -2);
/// La clairière magique de la tuile de campagne 2, voisine de l'emplacement 2A.
const GLADE: Hex = Hex::new(4, -3);
/// La plaine de la campagne 3 voisine de la forteresse.
const NEXT_TO_KEEP: Hex = Hex::new(6, -5);
const KEEP: Hex = Hex::new(7, -5);
/// Les orques de la tuile de campagne 2, que le trajet évite.
const ORCS_2: Hex = Hex::new(3, -3);

/// L'emplacement 2A (`docs/reference/bac-a-sable.md`).
fn position_2a() -> TilePosition {
    TilePosition::from_lattice(2, 0)
}

/// Un tour ne suffit pas toujours pour marcher : borne des tours de marche.
const MAX_WALKING_TURNS: u32 = 6;
/// Borne des tours d'exploration : au-delà, l'explorateur est bloqué (plan, *Risques*).
const MAX_EXPLORER_TURNS: u32 = 60;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::first_reconnaissance().without_dummy(),
        // Le héros de la partie lancée par l'interface, comme les autres parcours.
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Les cartes gardées pour l'influence du recrutement et pour le combat de l'assaut, tant qu'ils
/// ne sont pas faits : jamais jouées pour se déplacer.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Keep {
    Influence,
    Combat,
}

fn kept(card: CardId, keep: Keep) -> bool {
    let influence = matches!(
        card.card(),
        DeedCard::Basic(BasicAction::Promise | BasicAction::Threaten)
    );
    let fight = matches!(
        card.card(),
        DeedCard::Basic(
            BasicAction::BattleVersatility
                | BasicAction::Determination
                | BasicAction::Improvisation
        )
    );
    match keep {
        Keep::Influence => influence || fight,
        Keep::Combat => fight,
    }
}

/// Joue des cartes jusqu'à avoir `points` points (de mouvement ou d'influence, selon `amount`) :
/// à chaque fois celle qui en donne le plus, renforcée si le mana est là, en gardant les cartes de
/// `keep`. Rend faux s'il en manque.
fn gather(
    parcours: &mut Parcours,
    points: u32,
    keep: Keep,
    current: fn(&PlayerView) -> u32,
    amount: fn(Effect) -> Option<u8>,
) -> bool {
    while current(&parcours.view()) < points {
        let best = card_plays(parcours)
            .into_iter()
            .filter(|(_, card, play, _)| !kept(*card, keep) && single_card(play))
            .filter_map(|(action, _, _, effect)| amount(effect).map(|amount| (amount, action)))
            .max_by_key(|(amount, _)| *amount);
        let Some((_, action)) = best else {
            return false;
        };
        parcours.play(action);
    }
    true
}

fn gather_move(parcours: &mut Parcours, points: u32, keep: Keep) -> bool {
    gather(parcours, points, keep, |view| view.move_points, move_amount)
}

/// Joue des cartes jusqu'à ce que l'attaque ou la parade en cours atteigne `needed` : à chaque
/// fois la carte qui rapporte le plus de points par carte quittant la main.
fn gather_combat(parcours: &mut Parcours, needed: u32, what: &str) {
    loop {
        let current = combat(&parcours.view());
        if current.in_progress_total().unwrap_or(0) >= needed {
            return;
        }
        let best = card_plays(parcours)
            .into_iter()
            .filter(|(_, _, _, effect)| current.efficiency(*effect).is_some())
            .filter_map(|(action, _, play, effect)| {
                let points = current.counted_points(effect)?;
                let cards = match play {
                    CardPlay::Basic {
                        discard: Some(_), ..
                    }
                    | CardPlay::Strong {
                        discard: Some(_), ..
                    } => 2,
                    _ => 1,
                };
                Some((points * 10 / cards, action))
            })
            .max_by_key(|(score, _)| *score);
        precondition(best.is_some(), &format!("assez de cartes pour {what}"));
        let Some((_, action)) = best else { return };
        parcours.play(action);
    }
}

/// La fin de tour légale sans défausse ni Blessure jetée ; sans carte jouée, en défaussant une
/// carte qui n'est pas une Blessure (MKUE p. 6, 5.a).
fn finish_turn(parcours: &mut Parcours, keep: Keep) -> Vec<Event> {
    let view = parcours.view();
    // Deck vide et rien joué : le tour passe pour annoncer la fin de la manche (spec 023).
    if view.play_area.is_empty() && parcours.legal().contains(&Action::AnnounceEndOfRound) {
        return parcours.play(Action::AnnounceEndOfRound);
    }
    let discard = if view.play_area.is_empty() {
        let card = view
            .hand
            .iter()
            .copied()
            .find(|card| !card.card().is_wound() && !kept(*card, keep))
            .or_else(|| view.hand.iter().copied().find(|c| !c.card().is_wound()));
        card.into_iter().collect()
    } else {
        Vec::new()
    };
    let mut events = parcours.play(Action::EndTurn {
        discard,
        reclaim: None,
        glade: None,
        steady_tempo: false,
    });
    events.extend(support::take_pending_reward(parcours));
    events
}

/// Marche case par case, sur un ou plusieurs tours, le long de `steps`, en gardant `extra` points
/// de mouvement pour la suite du tour à l'arrivée ; le tour n'est pas fini à l'arrivée. Rend les
/// événements des pas.
fn walk(parcours: &mut Parcours, steps: &[Hex], extra: u32, keep: Keep) -> Vec<Event> {
    let mut events = Vec::new();
    let mut next = 0;
    for _ in 0..MAX_WALKING_TURNS {
        let view = parcours.view();
        let remaining: u32 = steps[next..].iter().map(|hex| step_cost(&view, *hex)).sum();
        gather_move(parcours, remaining + extra, keep);
        while let Some(&hex) = steps.get(next) {
            if !parcours.legal().contains(&Action::MoveHero { to: hex }) {
                break;
            }
            events.extend(parcours.play(Action::MoveHero { to: hex }));
            next += 1;
        }
        if next == steps.len() && gather_move(parcours, extra, keep) {
            return events;
        }
        finish_turn(parcours, keep);
    }
    precondition(false, "assez de cartes de Mouvement pour marcher");
    events
}

/// C1 — MKUE p. 18, *Special Rules* ; specs 050 (Q3), 063 (C1, C19).
fn setup(parcours: &Parcours) {
    let view = parcours.view();
    assert_eq!(
        view.scenario,
        Some(Scenario::first_reconnaissance().without_dummy())
    );
    assert_eq!((view.fame, view.level), (0, 1));
    assert_eq!(view.reputation.value(), 0);
    assert!(view.units.is_empty());
    assert_eq!(view.final_score, None);
    let units: Vec<UnitCard> = view.offers[&Offer::Units]
        .iter()
        .filter_map(|card| card.unit())
        .collect();
    assert!(units.iter().any(|unit| {
        unit.unit
            .definition()
            .recruit
            .contains(&RecruitSite::Village)
    }));
    assert!(
        units
            .iter()
            .all(|unit| unit.unit.definition().rank == UnitRank::Regular)
    );
    for (hex, terrain) in [
        (PORTAL, Terrain::Plains),
        (VILLAGE, Terrain::Plains),
        (GLADE, Terrain::Forest),
    ] {
        expect_terrain(parcours, hex, terrain);
    }
}

/// C2 et C3 : du portail au village, interaction et recrutement. Rend l'unité recrutée.
fn recruit(parcours: &mut Parcours) -> UnitCard {
    let unit = unit_to_recruit(&parcours.view());
    precondition(
        unit.is_some(),
        "une unité de village avec une Attaque dans l'offre",
    );
    let unit = unit.expect("vérifié");
    let cost = u32::from(unit.unit.definition().influence);

    // On y arrive par (1, 0), (2, -1), (3, -1), sans passer à côté des orques de (3, -3).
    let steps = [Hex::new(1, 0), Hex::new(2, -1), Hex::new(3, -1), VILLAGE];
    assert_eq!(route_to(parcours, VILLAGE), steps);
    let events = walk(parcours, &steps, 0, Keep::Influence);
    assert!(!has(&events, |e| matches!(
        e,
        Event::EnemiesProvoked { .. }
    )));
    // Un tour neuf pour l'interaction, si la main a déjà servi à marcher.
    let influence_in_hand = |view: &PlayerView| -> u32 {
        view.hand
            .iter()
            .filter(|card| !card.card().is_wound())
            .map(|card| match card.card() {
                DeedCard::Basic(BasicAction::Promise | BasicAction::Threaten) => 2,
                _ => 1,
            })
            .sum()
    };
    if influence_in_hand(&parcours.view()) < cost {
        finish_turn(parcours, Keep::Influence);
    }
    precondition(
        influence_in_hand(&parcours.view()) >= cost,
        "de quoi payer le recrutement",
    );

    // C2 — MKUE p. 8, *Interacting with Locals* ; spec 040.
    let events = parcours.play(Action::Interact);
    assert!(has(&events, |e| matches!(
        e,
        Event::InteractionStarted {
            site: Site::Village,
            ..
        }
    )));
    let start = parcours.view().influence_points;
    let mut played = 0;
    while parcours.view().influence_points < cost as i32 {
        let best = card_plays(parcours)
            .into_iter()
            .filter(|(_, _, play, _)| single_card(play))
            .filter_map(|(action, _, _, effect)| {
                influence_amount(effect).map(|amount| (amount, action))
            })
            .max_by_key(|(amount, _)| *amount);
        let (amount, action) =
            best.unwrap_or_else(|| panic!("graine à rechoisir : de quoi payer le recrutement"));
        parcours.play(action);
        played += i32::from(amount);
        assert_eq!(parcours.view().influence_points, start + played);
    }

    // C3 — MKUE p. 8, 4.a ; p. 5, *Using Units* 2 ; spec 042.
    let before = parcours.view().influence_points;
    let events = parcours.play(Action::RecruitUnit {
        unit,
        disband: None,
        bonds: false,
    });
    assert!(has(&events, |e| matches!(
        e,
        Event::UnitRecruited { unit: u, .. } if *u == unit
    )));
    let view = parcours.view();
    let owned: Vec<(UnitCard, bool)> = view.units.iter().map(|u| (u.card, u.ready)).collect();
    assert_eq!(owned, [(unit, true)]);
    assert!(!view.offers[&Offer::Units].contains(&CommonCard::Unit(unit)));
    assert_eq!(view.influence_points, before - cost as i32);
    finish_turn(parcours, Keep::Combat);
    unit
}

/// C4 et C5 : de la clairière, exploration de 2A, puis la case voisine de la forteresse.
fn explore_2a(parcours: &mut Parcours) {
    let to_glade = route_to(parcours, GLADE);
    walk(parcours, &to_glade, 2, Keep::Combat);
    assert_eq!(parcours.view().figure, Some(GLADE));

    // C4 — MKUE p. 18 ; carte Keep ; specs 043 (C1), 063 (C3).
    let fame = parcours.view().fame;
    let events = parcours.play(Action::Explore {
        position: position_2a(),
    });
    assert!(events.contains(&Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: position_2a(),
    }));
    assert!(has(&events, |e| matches!(
        e,
        Event::DefenderPlaced { hex, token } if *hex == KEEP && token.color() == EnemyColor::Grey
    )));
    assert!(events.contains(&Event::FameGained {
        player: SOLO,
        fame: 1,
    }));
    let view = parcours.view();
    assert_eq!(view.fame, fame + 1);
    expect_terrain(parcours, NEXT_TO_KEEP, Terrain::Plains);
    expect_terrain(parcours, KEEP, Terrain::Hills);
    // De jour, la clairière n'est pas voisine de la forteresse : le défenseur reste face cachée.
    assert_eq!(
        view.face_down_defenders.get(&KEEP),
        Some(&vec![EnemyColor::Grey])
    );
    assert!(!view.enemy_tokens.contains_key(&KEEP));

    // C5 — carte Keep ; spec 043 (C4) : de jour, le pas qui arrive à côté retourne le défenseur.
    let steps = route_to(parcours, NEXT_TO_KEEP);
    assert!(steps.iter().all(|hex| !hex.is_adjacent(ORCS_2)));
    let events = walk(parcours, &steps, 0, Keep::Combat);
    assert_eq!(parcours.view().figure, Some(NEXT_TO_KEEP));
    if parcours.view().time_of_day == TimeOfDay::Day {
        assert!(has(&events, |e| matches!(
            e,
            Event::EnemyTokenRevealed { hex, .. } if *hex == KEEP
        )));
        let view = parcours.view();
        assert!(!view.face_down_defenders.contains_key(&KEEP));
        assert_eq!(view.enemy_tokens[&KEEP][0].color(), EnemyColor::Grey);
    }
    finish_turn(parcours, Keep::Combat);
}

/// C6 à C9 : l'assaut, le combat avec l'unité, la conquête et le niveau 2. Rend l'action avancée
/// gagnée.
fn conquer(parcours: &mut Parcours, unit: UnitCard) -> CardId {
    // Faute de Mouvement en main, le tour passe en défaussant une carte, pour repiocher.
    let mut ready = false;
    for _ in 0..MAX_WALKING_TURNS {
        if gather_move(parcours, step_cost(&parcours.view(), KEEP), Keep::Combat) {
            ready = true;
            break;
        }
        finish_turn(parcours, Keep::Combat);
    }
    precondition(ready, "de quoi payer le pas de l'assaut");
    let fame = parcours.view().fame;

    // C6 — MKUE p. 7, 4.a ; p. 8, 1.a ; spec 043 (C11, C15).
    let events = parcours.play(Action::MoveHero { to: KEEP });
    assert!(events.contains(&Event::AssaultStarted {
        player: SOLO,
        site: KEEP,
        from: NEXT_TO_KEEP,
    }));
    assert!(events.contains(&Event::ReputationChanged {
        player: SOLO,
        change: -1,
    }));
    let events = parcours.play(Action::StartCombat);
    // De nuit, le défenseur se retourne à l'engagement (spec 043, C17) : C5 se vérifie là.
    if !parcours.view().face_down_defenders.is_empty() {
        assert!(has(&events, |e| matches!(
            e,
            Event::EnemyTokenRevealed { hex, .. } if *hex == KEEP
        )));
    }
    let defender = combat(&parcours.view()).enemies[0].clone();
    assert_eq!(defender.hex, KEEP);
    assert!(defender.site_fortified);
    let enemy = defender.token.enemy();
    precondition(
        enemy.definition().resistances.is_empty(),
        "un défenseur sans résistance (pas des Golems)",
    );
    let fortress_fame = u32::from(enemy.definition().fame);

    // Sans attaque de siège, le défenseur fortifié ne se combat qu'au corps à corps.
    parcours.play(Action::NextCombatPhase);
    parcours.play(Action::NextCombatPhase);
    let mut fight = Vec::new();
    // Les dégâts non parés vont au héros : l'unité garde ses Blessures à 0 pour le décompte.
    while let Some(action) = parcours.legal().into_iter().find(|action| {
        matches!(
            action,
            Action::AssignDamage {
                to: DamageTarget::Hero,
                ..
            }
        )
    }) {
        fight.extend(parcours.play(action));
    }
    if combat(&parcours.view()).phase != CombatPhase::Attack {
        fight.extend(parcours.play(Action::NextCombatPhase));
    }
    assert_eq!(combat(&parcours.view()).phase, CombatPhase::Attack);

    // C7 — MKUE p. 5, *Using Units* 3 ; specs 042, 043 (C19).
    parcours.play(Action::DeclareAttack {
        targets: vec![defender.token],
    });
    let (ability, option, amount) = melee_attack(unit).expect("unité choisie pour son Attaque");
    let events = parcours.play(Action::ActivateUnit {
        enemy: None,
        unit,
        ability,
        option,
        mana: Vec::new(),
        target: None,
        leadership: false,
    });
    assert!(has(&events, |e| matches!(
        e,
        Event::UnitActivated { unit: u, .. } if *u == unit
    )));
    assert_eq!(
        combat(&parcours.view()).in_progress_total(),
        Some(u32::from(amount))
    );
    assert!(!parcours.view().units[0].ready);
    let armor = u32::from(enemy.definition().armor);
    gather_combat(parcours, armor, "vaincre le défenseur");
    let events = parcours.play(Action::ResolveAttack);
    assert!(has(&events, |e| matches!(
        e,
        Event::EnemiesDefeated { enemies, .. } if enemies == &vec![defender.token]
    )));

    // C8 — MKUE p. 10, *Combat Outcomes* 2.c ; spec 043 (C20, C21).
    let events = parcours.play(Action::EndCombat);
    assert!(events.contains(&Event::SiteConquered {
        player: SOLO,
        hex: KEEP,
    }));
    let view = parcours.view();
    assert_eq!(view.shields.get(&KEEP), Some(&SOLO));
    assert_eq!(view.figure, Some(KEEP));
    assert_eq!(view.fame, fame + fortress_fame);
    assert_eq!(view.fame, 1 + fortress_fame);

    // L2 — spec 034 : jamais assommé pendant l'assaut.
    assert!(!has(&fight, |e| matches!(e, Event::KnockedOut { .. })));
    assert!(!view.combat.as_ref().is_some_and(|c| c.knocked_out));

    // C9 — MKUE p. 11, *End of the Turn* 6 ; specs 035, 050.
    let events = parcours.play(Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    });
    assert!(events.contains(&Event::LevelGained {
        player: SOLO,
        level: 2,
    }));
    // Spec 051 : la compétence du niveau 2 d'abord ; le parcours garde la première.
    support::choose_first_skill(parcours);
    let view = parcours.view();
    assert_eq!(view.advanced_action_choices, 1);
    let card = view.offers[&Offer::AdvancedActions][0]
        .deed()
        .expect("une action avancée");
    let number = match card.card() {
        DeedCard::Advanced(action) => GameContent::embedded().advanced_action_number(action),
        other => panic!("pas une action avancée : {other:?}"),
    };
    assert!((1..=16).contains(&number));
    let offer_size = view.offers[&Offer::AdvancedActions].len();
    let events = parcours.play(Action::GainAdvancedAction { card });
    assert!(events.contains(&Event::OfferCardGained {
        to: arcanum_core::CardDestination::DeckTop,
        player: SOLO,
        offer: Offer::AdvancedActions,
        card: CommonCard::Deed(card),
    }));
    let view = parcours.view();
    assert!(view.hand.contains(&card));
    assert_eq!(view.offers[&Offer::AdvancedActions].len(), offer_size);
    assert!(!view.offers[&Offer::AdvancedActions].contains(&CommonCard::Deed(card)));
    assert_eq!(view.level, 2);
    card
}

/// Du début de la partie à la sauvegarde, après le tour du niveau 2.
fn up_to_the_save_point(seed: u64) -> (Parcours, UnitCard, CardId) {
    let mut parcours = Parcours::new(config(seed));
    setup(&parcours);
    let unit = recruit(&mut parcours);
    explore_2a(&mut parcours);
    let card = conquer(&mut parcours, unit);
    (parcours, unit, card)
}

/// C10 — MKUE p. 4, 3.e ; spec 050 : l'action avancée gagnée se joue, pour son effet si le moteur
/// le sait, sinon de côté.
fn play_advanced_action(parcours: &mut Parcours, card: CardId) {
    let plays: Vec<Action> = parcours
        .legal()
        .into_iter()
        .filter(|action| matches!(action, Action::PlayCard { card: c, .. } if *c == card))
        .collect();
    let action = plays
        .iter()
        .find(|action| {
            matches!(
                action,
                Action::PlayCard {
                    play: CardPlay::Basic {
                        discard: None,
                        mana: None,
                        ..
                    },
                    ..
                }
            )
        })
        .or_else(|| {
            plays.iter().find(|action| {
                matches!(
                    action,
                    Action::PlayCard {
                        play: CardPlay::Sideways(SidewaysEffect::Move),
                        ..
                    }
                )
            })
        })
        .cloned()
        .expect("l'action avancée se joue, au moins de côté");
    let events = parcours.play(action);
    assert!(has(&events, |e| matches!(
        e,
        Event::CardPlayed { card: c, .. } if *c == card
    )));
    assert!(!parcours.view().hand.contains(&card));
}

/// De la sauvegarde à la fin de la partie : C10 à C15. Rend les événements des tours
/// d'exploration.
fn explore_to_the_city(parcours: &mut Parcours, unit: UnitCard, card: CardId) {
    let conquest = parcours.runner.log().len();
    let fame = parcours.view().fame;
    let conquest_round = parcours.view().round;
    let mut advanced_action_played = false;
    let mut checked_new_round = false;
    let mut turns = 0;
    let mut city_turn = None;
    while !parcours.is_finished() {
        turns += 1;
        precondition(
            turns <= MAX_EXPLORER_TURNS,
            "l'explorateur n'est pas bloqué",
        );
        // Deck vide : l'annonce passe avant la carte, jouée au tour d'après.
        if !advanced_action_played
            && parcours.view().hand.contains(&card)
            && !parcours.legal().contains(&Action::AnnounceEndOfRound)
        {
            play_advanced_action(parcours, card);
            advanced_action_played = true;
        }
        let events = parcours.explorer_turn();
        let view = parcours.view();

        // C11 — MKUE p. 4, 2.g ; specs 042, 050 : au début de la manche qui suit la conquête (la
        // 2, de Nuit, si l'assaut a lieu de jour).
        if !checked_new_round && view.round == conquest_round + 1 {
            checked_new_round = true;
            assert!(events.contains(&Event::UnitsReadied { player: SOLO }));
            assert_eq!(
                view.time_of_day == TimeOfDay::Night,
                view.round.is_multiple_of(2)
            );
            let owned: Vec<(UnitCard, bool)> =
                view.units.iter().map(|u| (u.card, u.ready)).collect();
            assert_eq!(owned, [(unit, true)]);
            assert_eq!(view.shields.get(&KEEP), Some(&SOLO));
            assert!(view.level >= 2 && view.fame >= fame);
            // Un monastère révélé y ajoute une action avancée (spec 041) : seules les unités comptent.
            assert!(
                view.offers[&Offer::Units]
                    .iter()
                    .filter_map(|card| card.unit())
                    .all(|unit| unit.unit.definition().rank == UnitRank::Regular)
            );
        }

        // C12 — MKUE p. 18 ; p. 11, 6 ; specs 035, 063 (C3, C5).
        for (i, event) in events.iter().enumerate() {
            if matches!(event, Event::TileRevealed { .. }) {
                assert_eq!(
                    events.get(i + 1..).and_then(|rest| rest
                        .iter()
                        .find(|e| matches!(e, Event::FameGained { .. }))),
                    Some(&Event::FameGained {
                        player: SOLO,
                        fame: 1,
                    }),
                    "1 gloire par tuile révélée"
                );
            }
            if matches!(event, Event::LevelGained { .. }) {
                assert!(
                    events[..i]
                        .iter()
                        .any(|e| matches!(e, Event::TurnEnded { .. })),
                    "un niveau ne vient qu'à une fin de tour"
                );
            }
        }

        // C13 — MKUE p. 18, *Scenario End* ; spec 063 (C7).
        let city = events
            .iter()
            .position(|e| matches!(e, Event::TileRevealed { tile, .. } if tile.is_city()));
        if let Some(city) = city {
            city_turn = Some(view.turn);
            let announced = events
                .iter()
                .position(|e| matches!(e, Event::LastTurnAnnounced { .. }));
            assert!(announced.is_some_and(|announced| announced > city));
            let fame_gained = events[city..]
                .iter()
                .position(|e| matches!(e, Event::FameGained { .. }));
            assert!(fame_gained.is_some_and(|f| city + f < announced.unwrap_or(0)));
            precondition(
                view.round <= 4,
                "la cité révélée avant la fin de la 4e manche",
            );

            // C14 — spec 063 (C8) : la fin de ce tour n'arrête pas la partie.
            let ended = events
                .iter()
                .position(|e| matches!(e, Event::TurnEnded { .. }))
                .expect("le tour de la révélation finit");
            assert!(ended > announced.unwrap_or(0));
            assert!(has(&events[ended..], |e| matches!(
                e,
                Event::TurnStarted { .. }
            )));
            assert!(!has(&events, |e| matches!(e, Event::GameEnded { .. })));
            assert_eq!(parcours.view().last_turn, LastTurn::Current);
            assert_eq!(parcours.view().game_end, None);
        }
    }
    precondition(city_turn.is_some(), "la cité révélée");
    assert!(advanced_action_played && checked_new_round);

    // C15 — MKUE p. 4, 4.c et 4.d ; spec 063 (C9, C10).
    let view = parcours.view();
    assert_eq!(view.game_end, Some(GameEndReason::CityRevealed));
    assert!(parcours.legal().is_empty());
    let events = all_events(parcours);
    assert!(events.ends_with(&[Event::GameEnded {
        reason: GameEndReason::CityRevealed,
    }]));

    // L3 — spec 049 : les tours d'exploration ne provoquent rien et n'assaillent aucun site.
    let after = &events[conquest..];
    assert!(!has(after, |e| matches!(
        e,
        Event::EnemiesProvoked { .. } | Event::AssaultStarted { .. }
    )));
}

/// C16 — MKUE p. 15 ; p. 18, *Scoring* ; specs 063 (C14), 042, 043, 050 : le score de la vue,
/// recompté depuis les événements du parcours et la vue.
fn check_score(parcours: &Parcours, unit: UnitCard) {
    let view = parcours.view();
    let score = view.final_score.expect("le décompte de la partie finie");
    let events = all_events(parcours);
    let advanced_actions = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Event::OfferCardGained {
                    offer: Offer::AdvancedActions,
                    ..
                }
            )
        })
        .count() as u32;
    let wounds_taken: usize = events
        .iter()
        .map(|e| match e {
            Event::WoundsTaken { wounds, .. } | Event::PoisonWoundsDiscarded { wounds, .. } => {
                wounds.len()
            }
            _ => 0,
        })
        .sum();
    let wounds_gone: usize = events
        .iter()
        .map(|e| match e {
            Event::WoundsHealed { wounds, .. } => wounds.len(),
            Event::WoundThrownAway { .. } => 1,
            _ => 0,
        })
        .sum();
    let wounds = (wounds_taken - wounds_gone) as u32;
    let owned = view.units.iter().find(|u| u.card == unit).expect("l'unité");
    let level = u32::from(unit.unit.definition().level);
    let leader = if owned.is_wounded() { level / 2 } else { level };
    let crystals = view.inventory.total();

    assert_eq!(score.fame, view.fame);
    assert_eq!(score.advanced_actions, advanced_actions);
    assert_eq!(score.crystals, crystals);
    assert_eq!(score.unit_levels, leader);
    assert_eq!(score.shields, 1);
    assert_eq!(score.adventure_shields, 0);
    assert_eq!((score.spells, score.artifacts), (0, 0));
    assert_eq!(score.wounds, wounds);
    use arcanum_core::Achievement::*;
    assert_eq!(score.points(Knowledge), i64::from(advanced_actions));
    assert_eq!(score.points(Loot), i64::from(crystals / 2));
    assert_eq!(score.points(Leader), i64::from(leader));
    assert_eq!(score.points(Conqueror), 2);
    assert_eq!(score.points(Adventurer), 0);
    assert_eq!(score.points(Beating), -2 * i64::from(wounds));
    assert_eq!(
        score.total(),
        i64::from(view.fame)
            + i64::from(advanced_actions)
            + i64::from(crystals / 2)
            + i64::from(leader)
            + 2
            - 2 * i64::from(wounds)
    );
}

fn after_the_save_point(parcours: &mut Parcours, unit: UnitCard, card: CardId) {
    explore_to_the_city(parcours, unit, card);
    check_score(parcours, unit);
}

// C1 à C16 : un seul parcours, de la mise en place au décompte final.
#[test]
fn parcours_premiere_reconnaissance() {
    let (mut parcours, unit, card) = up_to_the_save_point(SEED);
    after_the_save_point(&mut parcours, unit, card);
}

// C17 — ADR 0001 ; specs 024, 038 : reprise depuis la sauvegarde faite après le tour du niveau 2.
#[test]
fn parcours_reprise_apres_la_conquete() {
    let (mut played, unit, card) = up_to_the_save_point(SEED);
    let saved: (EventLog, RngPositions) = played.save();
    after_the_save_point(&mut played, unit, card);

    let mut resumed = Parcours::resume(&saved);
    after_the_save_point(&mut resumed, unit, card);
    assert_eq!(resumed.runner.log(), played.runner.log());
    assert_eq!(resumed.runner.state(), played.runner.state());
    assert_eq!(resumed.view().final_score, played.view().final_score);
}

/// Outil pour rechoisir la graine : `cargo test -p arcanum-core --test parcours_j4 -- --ignored
/// --nocapture` liste les graines qui passent, et la raison de l'échec des autres.
#[test]
#[ignore]
fn chercher_une_graine() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        let line = info.location().map(|l| l.line()).unwrap_or(0);
        eprintln!("ECHEC {line} {}", msg.lines().next().unwrap_or(""));
    }));
    let mut found = Vec::new();
    for seed in 0..200 {
        let ok = std::panic::catch_unwind(|| {
            let (mut parcours, unit, card) = up_to_the_save_point(seed);
            let conquest = parcours.view().round;
            after_the_save_point(&mut parcours, unit, card);
            let view = parcours.view();
            eprintln!(
                "graine {seed} : conquête manche {conquest}, fin manche {} tour {}, unité {:?}, score {:?}",
                view.round,
                view.turn,
                unit.unit,
                view.final_score.map(|s| s.total())
            );
        });
        if ok.is_ok() {
            found.push(seed);
        }
    }
    let _ = std::panic::take_hook();
    println!("graines : {found:?}");
}
