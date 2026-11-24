//! Parcours de bout en bout du jalon 1 (spec 019, ADR 0006) : une manche de jour du bac à sable,
//! jouée par un joueur scripté, pour se déplacer et explorer.

mod support;

use std::collections::BTreeSet;

use arcanum_core::{
    Action, BasicAction, CardPlay, Effect, Event, GameConfig, Hero, Hex, NextTile,
    STARTING_HAND_LIMIT, Scenario, Terrain, TileDeckCounts, TileId, TilePosition, TimeOfDay,
};
use support::{Parcours, first_playable, has, in_hand, sideways_move, walk_to};

/// Graine choisie pour que les préconditions du parcours tiennent (spec 019, « Graine et
/// préconditions »). Si un test échoue sur « graine à rechoisir », en chercher une autre.
const SEED: u64 = 10;

const DECK_SIZE: usize = 16;

const PORTAL: Hex = Hex::ORIGIN;
const FIRST_STEP: Hex = Hex::new(0, -1);
const VILLAGE: Hex = Hex::new(2, -3);
const LAKE: Hex = Hex::new(2, -4);
const NEW_VILLAGE: Hex = Hex::new(3, -4);
const EXPLORE_FROM: Hex = Hex::new(4, -6);
const KEEP: Hex = Hex::new(5, -6);
const FOREST: Hex = Hex::new(4, -5);
const MOUNTAIN: Hex = Hex::new(3, -6);

/// `A + B` et `2B` (specs 012 et 015).
const A_PLUS_B: TilePosition = TilePosition::from_lattice(1, 1);
const TWO_B: TilePosition = TilePosition::from_lattice(0, 2);

const MOVE_TWO: [BasicAction; 3] = [
    BasicAction::March,
    BasicAction::Stamina,
    BasicAction::Swiftness,
];

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        // Le héros de la partie lancée par l'interface, comme la 028.
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Spec 019, L2 : une précondition qui ne tient plus vient de la graine, pas des règles.
fn precondition(holds: bool, what: &str) {
    assert!(holds, "graine à rechoisir : {what}");
}

/// Spec 019, L3 : le plateau de jeu traversé ne dépend pas de la graine.
fn expect_terrain(parcours: &Parcours, hex: Hex, terrain: Terrain) {
    let space = parcours.view().map.space(hex);
    assert_eq!(
        space.map(|space| space.terrain),
        Some(terrain),
        "case {hex:?} : la mise en place a changé"
    );
}

/// Les cases où un `MoveHero` est légal.
fn move_targets(parcours: &Parcours) -> BTreeSet<Hex> {
    parcours
        .legal()
        .into_iter()
        .filter_map(|action| match action {
            Action::MoveHero { to } => Some(to),
            _ => None,
        })
        .collect()
}

fn explore_targets(parcours: &Parcours) -> Vec<TilePosition> {
    parcours
        .legal()
        .into_iter()
        .filter_map(|action| match action {
            Action::Explore { position } => Some(position),
            _ => None,
        })
        .collect()
}

/// L'effet qu'une action légale de carte donnerait.
fn effect_of(action: &Action) -> Option<Effect> {
    let Action::PlayCard { card, play } = action else {
        return None;
    };
    card.effect(*play)
}

fn basic_move_two(parcours: &mut Parcours, card: arcanum_core::CardId) {
    parcours.play(Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        },
    });
}

/// Joue des cartes jusqu'à avoir `points` points de mouvement : une carte de Mouvement 2 pour
/// son effet de base si la main en a une, sinon la première de côté.
fn gather(parcours: &mut Parcours, points: u32) {
    while parcours.view().move_points < points {
        let view = parcours.view();
        precondition(
            first_playable(&view).is_some(),
            &format!("assez de cartes pour {points} points au tour {}", view.turn),
        );
        match MOVE_TWO.iter().find_map(|&action| in_hand(&view, action)) {
            Some(card) => basic_move_two(parcours, card),
            None => sideways_move(parcours),
        }
    }
}

/// Les cartes qui ne sont pas des Blessures.
fn without_wounds(cards: &[arcanum_core::CardId]) -> usize {
    cards.iter().filter(|card| !card.card().is_wound()).count()
}

/// Les Blessures reçues depuis le début de la partie, d'après le journal.
fn wounds_taken(parcours: &Parcours) -> usize {
    parcours
        .runner
        .log()
        .entries()
        .iter()
        .map(|entry| match &entry.event {
            Event::WoundsTaken { wounds, .. } => wounds.len(),
            // Spec 033 : les Blessures toxiques vont dans la défausse.
            Event::PoisonWoundsDiscarded { wounds, .. } => wounds.len(),
            _ => 0,
        })
        .sum()
}

/// Les 16 cartes sont toujours là, plus les Blessures reçues, que le deck ne compte pas (spec 019,
/// Comportements attendus ; spec 031).
fn all_cards_accounted_for(parcours: &Parcours) {
    let view = parcours.view();
    let played: Vec<_> = view.play_area.iter().map(|played| played.card).collect();
    assert_eq!(
        without_wounds(&view.hand) + view.deck_size + played.len() + without_wounds(&view.discard),
        DECK_SIZE,
        "tour {}",
        view.turn
    );
    let wounds: usize = [view.hand.as_slice(), &played, &view.discard]
        .iter()
        .map(|zone| zone.len() - without_wounds(zone))
        .sum();
    assert_eq!(wounds, wounds_taken(parcours), "tour {}", view.turn);
}

fn moved_costs(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::HeroMoved { cost, .. } => Some(*cost),
            _ => None,
        })
        .collect()
}

/// C1 — MKUE p. 3 ; specs 012, 013, 015.
fn setup(parcours: &Parcours) {
    let view = parcours.view();
    assert_eq!(view.figure, Some(PORTAL));
    assert_eq!(view.hand.len(), 5);
    assert_eq!(view.deck_size, 11);
    assert!(view.discard.is_empty());
    assert_eq!((view.round, view.turn), (1, 1));
    assert_eq!(view.time_of_day, TimeOfDay::Day);
    assert_eq!(view.move_points, 0);
    assert_eq!(view.map.tiles().count(), 3);
    assert_eq!(
        view.tile_deck,
        TileDeckCounts {
            countryside: 6,
            core: 3
        }
    );
    assert_eq!(view.next_tile, NextTile::Countryside);
    assert!(move_targets(parcours).is_empty());
    assert!(explore_targets(parcours).is_empty());
    for (hex, terrain) in [
        (PORTAL, Terrain::Plains),
        (Hex::new(-1, 0), Terrain::Sea),
        (Hex::new(-1, 1), Terrain::Sea),
        (Hex::new(0, 1), Terrain::Sea),
        (FIRST_STEP, Terrain::Plains),
        (VILLAGE, Terrain::Plains),
        (LAKE, Terrain::Lake),
    ] {
        expect_terrain(parcours, hex, terrain);
    }
}

/// Tour 1 : C2, C3, C5.
fn first_turn(parcours: &mut Parcours) {
    let start = parcours.view();
    let move_two = MOVE_TWO.iter().find_map(|&action| in_hand(&start, action));
    precondition(
        move_two.is_some(),
        "une carte de Mouvement 2 dans la main de départ",
    );
    let Some(move_two) = move_two else { return };

    // C2 — MKUE p. 4, Deed Cards 3.a, 3.e et 4 ; spec 013 : effet de base + de côté = 3 points.
    basic_move_two(parcours, move_two);
    sideways_move(parcours);
    let view = parcours.view();
    assert_eq!(view.move_points, 3);
    assert_eq!(view.play_area.len(), 2);
    assert!(
        parcours
            .legal()
            .iter()
            .filter_map(effect_of)
            .all(|effect| !matches!(effect, Effect::Attack { .. } | Effect::Block { .. })),
        "ni Attaque ni Parade hors combat"
    );

    // C3 — MKUE p. 7, 3 et 3.a ; spec 014 : les trois voisines de Littoral sont infranchissables.
    assert_eq!(
        move_targets(parcours),
        BTreeSet::from([Hex::new(1, 0), Hex::new(1, -1), FIRST_STEP])
    );
    let events = parcours.play(Action::MoveHero { to: FIRST_STEP });
    assert_eq!(moved_costs(&events), vec![2]);
    assert_eq!(parcours.view().move_points, 1);
    assert!(move_targets(parcours).is_empty(), "il manque des points");

    // C5 — MKUE p. 6, 6.e et 8 ; p. 11, 3, 7.a et 7.b ; spec 016.
    let discarded = parcours.view().hand[0];
    assert!(!finish_turn(parcours, vec![discarded]));
    let next = parcours.view();
    assert_eq!(next.discard.len(), 3);
    assert!(next.discard.contains(&discarded));
    assert_eq!(next.move_points, 0);
    assert_eq!(next.hand.len(), 5);
    assert_eq!(next.deck_size, 8);
    assert_eq!(next.turn, 2);
    all_cards_accounted_for(parcours);
}

/// Tour 2 : marcher jusqu'au village (C4, première partie).
fn second_turn(parcours: &mut Parcours) -> bool {
    let view = parcours.view();
    let route = arcanum_core::routes(&view.map, FIRST_STEP, TimeOfDay::Day)
        .into_iter()
        .find(|route| route.to == VILLAGE)
        .expect("le village est atteignable");
    gather(parcours, route.cost);
    let before = parcours.runner.log().len();

    // C4 — MKUE p. 7, 3 ; specs 014, 017 : chaque pas coûte le terrain où l'on entre.
    assert_eq!(walk_to(parcours, VILLAGE), route.cost);
    let events: Vec<Event> = parcours.runner.log().entries()[before..]
        .iter()
        .map(|entry| entry.event.clone())
        .collect();
    let costs = moved_costs(&events);
    assert_eq!(costs.len(), route.steps.len());
    assert_eq!(costs.iter().map(|&c| u32::from(c)).sum::<u32>(), route.cost);
    assert_eq!(parcours.view().figure, Some(VILLAGE));
    finish_turn(parcours, Vec::new())
}

/// Tour 3 : C4 (le Lac), C6, C7.
fn third_turn(parcours: &mut Parcours) -> bool {
    // C6 — MKUE p. 7, 5.b : pas d'exploration à moins de 2 points.
    assert!(explore_targets(parcours).is_empty());
    gather(parcours, 4);

    // C4 — MKUE p. 7, 3.a ; spec 014 : le Lac voisin n'est jamais légal.
    let targets = move_targets(parcours);
    assert!(!targets.is_empty());
    assert!(!targets.contains(&LAKE));

    // C6 — MKUE p. 7, 5.a à 5.e ; spec 015 : seul `A + B` borde le village.
    assert_eq!(explore_targets(parcours), vec![A_PLUS_B]);
    let points = parcours.view().move_points;
    let events = parcours.play(Action::Explore { position: A_PLUS_B });
    assert!(has(&events, |e| matches!(
        e,
        Event::Explored { cost: 2, .. }
    )));
    assert!(events.contains(&Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: A_PLUS_B,
    }));
    let view = parcours.view();
    assert_eq!(view.move_points, points - 2);
    assert_eq!(view.figure, Some(VILLAGE));
    assert_eq!(view.tile_deck.countryside, 5);
    for (hex, terrain) in [
        (NEW_VILLAGE, Terrain::Plains),
        (Hex::new(3, -5), Terrain::Plains),
        (EXPLORE_FROM, Terrain::Plains),
        (KEEP, Terrain::Hills),
        (FOREST, Terrain::Forest),
    ] {
        expect_terrain(parcours, hex, terrain);
    }

    // C7 — MKUE p. 7, 3 ; specs 014, 015 : la tuile révélée se parcourt.
    assert!(move_targets(parcours).contains(&NEW_VILLAGE));
    let events = parcours.play(Action::MoveHero { to: NEW_VILLAGE });
    assert_eq!(moved_costs(&events), vec![2]);
    // Spec 031 : ce pas longe les orques imprimés de la tuile 2, provoqués (spec 030) ; le combat
    // est obligatoire, mené sans attaque ni parade avant de finir le tour.
    assert!(has(&events, |e| matches!(e, Event::EnemiesProvoked { .. })));
    let events = fight_without_cards(parcours);
    // MKUE p. 9, *Assign Damage Phase* 5.b : une Mégère maudite (attaque 3) non parée donne deux
    // Blessures (armure 2, MKUE p. 3), qui arrivent dans la main.
    let wounds: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::WoundsTaken { wounds, .. } => Some(wounds.len()),
            _ => None,
        })
        .collect();
    precondition(
        wounds == [2],
        "les orques de la tuile 2 sont des Mégères maudites (attaque 3)",
    );
    // Spec 033 C33 — MKUE p. 10, 6.a : elles sont toxiques, 2 Blessures de plus vont dans la
    // défausse.
    let poisoned: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::PoisonWoundsDiscarded { wounds, .. } => Some(wounds.len()),
            _ => None,
        })
        .collect();
    assert_eq!(poisoned, [2]);
    assert_eq!(
        parcours
            .view()
            .discard
            .iter()
            .filter(|c| c.card().is_wound())
            .count(),
        2
    );
    assert_eq!(
        parcours
            .view()
            .hand
            .iter()
            .filter(|c| c.card().is_wound())
            .count(),
        2
    );
    finish_turn(parcours, Vec::new())
}

/// Engage le combat et passe ses phases sans jouer de carte, jusqu'à le terminer (spec 031).
fn fight_without_cards(parcours: &mut Parcours) -> Vec<Event> {
    [
        Action::StartCombat,
        Action::NextCombatPhase,
        Action::NextCombatPhase,
        Action::EndCombat,
    ]
    .into_iter()
    .flat_map(|action| parcours.play(action))
    .collect()
}

/// Tour 4 : marcher jusqu'à (4, -6).
fn fourth_turn(parcours: &mut Parcours) -> bool {
    gather(parcours, 4);
    assert_eq!(walk_to(parcours, EXPLORE_FROM), 4);
    finish_turn(parcours, Vec::new())
}

/// Tour 5 : C8, C9.
fn fifth_turn(parcours: &mut Parcours) -> bool {
    // C8 — MKUE p. 7, 4.a ; spec 014 (Q2) : la Forêt est atteignable. Depuis la spec 043, la
    // forteresse aussi : y entrer est un assaut (le parcours ne le joue pas).
    gather(parcours, 3);
    let targets = move_targets(parcours);
    assert!(targets.contains(&KEEP));
    assert!(targets.contains(&FOREST));

    // C9 — MKUE p. 7, 3.a, 5.c et 5.e ; specs 014, 015 : `2B` touche les tuiles 1 et 3.
    let events = parcours.play(Action::Explore { position: TWO_B });
    assert!(events.contains(&Event::TileRevealed {
        tile: TileId::Countryside(4),
        position: TWO_B,
    }));
    expect_terrain(parcours, MOUNTAIN, Terrain::Mountain);
    gather(parcours, 2);
    assert!(!move_targets(parcours).contains(&MOUNTAIN));
    finish_turn(parcours, Vec::new())
}

/// Finit le tour en défaussant `discard`, et vérifie C10 si le deck ne peut plus remplir la main.
/// Rend vrai si cette fin de tour a pioché moins que nécessaire.
fn finish_turn(parcours: &mut Parcours, discard: Vec<arcanum_core::CardId>) -> bool {
    let before = parcours.view();
    let kept = before.hand.len() - discard.len();
    let needed = STARTING_HAND_LIMIT.saturating_sub(kept);
    let events = parcours.play(Action::EndTurn {
        discard,
        reclaim: None,
        glade: None,
        steady_tempo: false,
    });
    all_cards_accounted_for(parcours);
    let after = parcours.view();
    assert_eq!(after.move_points, 0, "points perdus");
    if before.deck_size >= needed {
        return false;
    }
    // C10 — MKUE p. 11, 7.c et 7.d ; spec 016 : le deck s'épuise sans remélange.
    let drawn = events
        .iter()
        .filter(|e| matches!(e, Event::CardDrawn { .. }))
        .count();
    assert_eq!(drawn, before.deck_size, "toutes les cartes restantes");
    assert_eq!(after.deck_size, 0);
    assert_eq!(after.hand.len(), kept + drawn);
    assert!(after.hand.len() < STARTING_HAND_LIMIT);
    // Les Blessures (spec 031) ne sont pas dans les 16 cartes du deck.
    assert_eq!(
        without_wounds(&after.discard),
        DECK_SIZE - without_wounds(&after.hand),
        "pas de remélange"
    );
    assert_eq!(after.round, 1);
    true
}

/// C10 : si le deck n'est pas encore épuisé, jouer toute sa main de côté jusqu'à ce qu'il le soit.
fn exhaust_the_deck(parcours: &mut Parcours, mut exhausted: bool) {
    while !exhausted {
        // Les Blessures restent en main (spec 031) : on joue tout le reste.
        while first_playable(&parcours.view()).is_some() {
            sideways_move(parcours);
        }
        // Aucune carte jouée et que des Blessures en main : le repos prolongé en défausse une
        // (MKUE p. 6, 7.a ; spec 031, C28 bis).
        let view = parcours.view();
        let discard = view
            .hand
            .first()
            .copied()
            .filter(|_| view.play_area.is_empty());
        exhausted = finish_turn(parcours, discard.into_iter().collect());
    }
}

// C1 à C10 : un seul parcours, de la mise en place au deck épuisé.
#[test]
fn parcours_se_deplacer_et_explorer() {
    let mut parcours = Parcours::new(config(SEED));
    setup(&parcours);
    first_turn(&mut parcours);
    let mut exhausted = second_turn(&mut parcours);
    exhausted |= third_turn(&mut parcours);
    exhausted |= fourth_turn(&mut parcours);
    exhausted |= fifth_turn(&mut parcours);
    exhaust_the_deck(&mut parcours, exhausted);
}
