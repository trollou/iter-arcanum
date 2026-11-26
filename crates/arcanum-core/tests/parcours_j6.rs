//! Parcours de bout en bout du jalon 6 (spec 069, ADR 0006) : Conquête en solitaire, Tovak contre
//! le fantôme Goldyx. Le parcours principal joue une partie entière (tactiques, fantôme, cités
//! révélées, mission échouée) ; l'épisode posé prend la seconde cité (retraite puis conquête,
//! mission accomplie) ; la reprise rejoue la fin du parcours principal.

mod support;

use arcanum_core::{
    Action, BasicColor, CardId, CommonCard, DeedCard, DummyHeroChoice, DummyView, EnemyColor,
    Event, EventLog, GameConfig, GameEndReason, Hero, NextTile, Offer, PlayerView, RngPositions,
    RoundOrder, Scenario, Tactic, TileId, TileKind, TimeOfDay,
};
use support::arrangement::Setup;
use support::{Parcours, SOLO, Step, all_events, precondition, take_pending_reward};

/// Graine du parcours principal (spec 069, « Départ et préconditions ») ; `balayer_les_graines`
/// dit lesquelles passent.
const SEED: u64 = 22;
/// Spec 069, Q6 : au moins 90 graines sur 100.
const SWEEP_THRESHOLD: usize = 90;
/// Borne des décisions d'une partie : au-delà, le script tourne en rond.
const MAX_DECISIONS: usize = 4_000;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::solo_conquest(),
        hero: Hero::Tovak,
        dummy_hero: DummyHeroChoice::Hero(Hero::Goldyx),
    }
}

/// Les tuiles de cité de la pioche de tuiles d'une mise en place.
fn city_tiles(setup: &Setup) -> Vec<TileId> {
    setup
        .events()
        .iter()
        .find_map(|event| match event {
            Event::TileDeckFormed { order } => Some(order.clone()),
            _ => None,
        })
        .expect("pioche de tuiles formée")
        .into_iter()
        .filter(|tile| tile.is_city())
        .collect()
}

/// Spec 069 : la vraie mise en place, les deux tuiles de cité en tête des tuiles royaume.
fn main_setup(seed: u64) -> Setup {
    let setup = Setup::new(config(seed));
    let cities = city_tiles(&setup);
    setup.core_tiles(&cities)
}

// --- Le choix des tactiques par le script (spec 069, Modèle) ---

const DAY_PREFERENCES: [Tactic; 6] = [
    Tactic::GreatStart,
    Tactic::EarlyBird,
    Tactic::TheRightMoment,
    Tactic::Planning,
    Tactic::Rethink,
    Tactic::ManaSteal,
];

const NIGHT_PREFERENCES: [Tactic; 6] = [
    Tactic::Preparation,
    Tactic::FromTheDusk,
    Tactic::ManaSearch,
    Tactic::MidnightMeditation,
    Tactic::SparingPower,
    Tactic::LongNight,
];

/// Les cercles de défenseurs des cités aux niveaux 5 et 8 (`cities.ron`).
fn city_defenders(color: BasicColor, level: u8) -> Vec<EnemyColor> {
    use EnemyColor::*;
    let level_5 = match color {
        BasicColor::Blue => vec![Violet, Violet, White],
        BasicColor::Green => vec![Brown, Brown, White],
        BasicColor::Red => vec![Brown, Violet, White],
        BasicColor::White => vec![Grey, White, White],
    };
    match level {
        5 => level_5,
        8 => level_5.into_iter().chain([White]).collect(),
        other => panic!("niveau {other} hors du scénario"),
    }
}

fn competitive(spell: CommonCard) -> bool {
    use arcanum_core::Spell::*;
    matches!(
        spell,
        CommonCard::Deed(card) if matches!(
            card.card(),
            DeedCard::Spell(ManaMeltdown | ManaClaim | MindRead | EnergyFlow)
        )
    )
}

/// La couleur d'une carte (son cadre) : celle qui compte pour le tour du fantôme.
fn card_color(card: CardId) -> Option<BasicColor> {
    card.card().definition().map(|definition| definition.color)
}

fn dummy(view: &PlayerView) -> DummyView {
    view.dummy.clone().expect("le fantôme est en jeu")
}

fn position_of(events: &[Event], wanted: impl Fn(&Event) -> bool) -> Option<usize> {
    events.iter().position(wanted)
}

/// Ce que le parcours principal a vu, gardé d'une décision à l'autre (et recopié à la reprise).
#[derive(Clone, Default)]
struct Seen {
    /// Les tactiques prises, par le joueur puis le fantôme, manche par manche.
    taken: Vec<Tactic>,
    great_start: bool,
    preparation: bool,
    dummy_announced: bool,
    /// Le fantôme a annoncé la fin de la manche en cours : le joueur ne peut plus l'annoncer.
    announced_this_round: bool,
    /// La taille du deck du fantôme remélangé à la dernière nouvelle manche.
    dummy_deck: usize,
    cities: Vec<u8>,
    new_rounds: u32,
}

struct J6 {
    p: Parcours,
    seen: Seen,
    /// Graine du test : les deux cités révélées et une annonce du fantôme sont des préconditions ;
    /// au balayage, C5 et C9 ne se vérifient que si cela arrive (Mathieu, 2026-11-19).
    strict: bool,
}

impl J6 {
    fn view(&self) -> PlayerView {
        self.p.view()
    }

    /// Une décision du script, ou un tour entier de l'explorateur, puis les vérifications de
    /// chaque décision prise.
    fn advance(&mut self) {
        let legal = self.p.legal();
        let has = |wanted: fn(&Action) -> bool| legal.iter().find(|action| wanted(action)).cloned();
        if has(|a| matches!(a, Action::ChooseTactic { .. })).is_some() {
            self.choose_tactic(&legal);
        } else if has(|a| matches!(a, Action::RethinkDiscard { .. })).is_some() {
            // Faux départ : défausser 0 carte.
            self.p.play(Action::RethinkDiscard { cards: Vec::new() });
        } else if has(|a| matches!(a, Action::PrepareCard { .. })).is_some() {
            self.prepare(&legal);
        } else if let Some(steal) = has(|a| matches!(a, Action::StealDie { .. })) {
            // Vol de mana : le premier dé de couleur de base.
            self.p.play(steal);
        } else if legal.contains(&Action::DeclineLongNight) {
            self.p.play(Action::DeclineLongNight);
        } else if legal.contains(&Action::SparingPower { take: true }) {
            // Provisions : rien posé dessous, la tactique retournée sans rien prendre.
            self.p.play(Action::SparingPower { take: true });
        } else if has(|a| {
            matches!(
                a,
                Action::GainAdvancedAction { .. } | Action::ChooseSkill { .. }
            )
        })
        .is_some()
        {
            take_pending_reward(&mut self.p);
        } else {
            self.p.explorer_turn();
        }
        for step in self.p.take_steps() {
            self.check_step(&step);
        }
    }

    /// C2, C3, C8 : la première décision de la manche, et elle seule, est le choix de la tactique.
    fn choose_tactic(&mut self, legal: &[Action]) {
        let view = self.view();
        let offered: Vec<Tactic> = legal
            .iter()
            .map(|action| match action {
                Action::ChooseTactic { tactic } => *tactic,
                other => panic!("C2 : {other:?} pendant le choix de la tactique"),
            })
            .collect();
        let expected: Vec<Tactic> = Tactic::of(view.time_of_day)
            .filter(|tactic| !view.removed_tactics.contains(tactic))
            .collect();
        assert_eq!(offered, expected, "C2 : tactiques du moment non retirées");
        // C8 : aucune tactique retirée n'est proposée ; aux manches 5 et 6, il en reste deux.
        assert!(
            offered
                .iter()
                .all(|tactic| !self.seen.taken.contains(tactic))
        );
        if view.round >= 5 {
            assert_eq!(offered.len(), 2, "C8 : manche {}", view.round);
        }
        let preferences = match view.time_of_day {
            TimeOfDay::Day => DAY_PREFERENCES,
            TimeOfDay::Night => NIGHT_PREFERENCES,
        };
        let tactic = preferences
            .into_iter()
            .find(|tactic| offered.contains(tactic))
            .expect("une tactique proposée");
        let events = self.p.play(Action::ChooseTactic { tactic });
        let chosen = position_of(&events, |e| {
            *e == Event::TacticChosen {
                player: SOLO,
                tactic,
            }
        })
        .expect("C2 : TacticChosen");
        let (drawn, dummy_tactic) = events
            .iter()
            .enumerate()
            .find_map(|(i, e)| match e {
                Event::DummyTacticDrawn { tactic } => Some((i, *tactic)),
                _ => None,
            })
            .expect("C2 : DummyTacticDrawn");
        assert!(chosen < drawn, "C2 : le fantôme tire après le joueur");
        assert_ne!(dummy_tactic, tactic);
        assert!(
            offered.contains(&dummy_tactic),
            "C2 : tactique du même moment"
        );
        self.seen.taken.extend([tactic, dummy_tactic]);
        // C3 : l'ordre du tour suit les numéros.
        let after = self.view();
        let dummy_first = dummy_tactic.number() < tactic.number();
        assert_eq!(
            after.round_order,
            Some(if dummy_first {
                RoundOrder::DummyFirst
            } else {
                RoundOrder::PlayerFirst
            })
        );
        // C2 : Départ flamboyant pioche 2 cartes, main de 7.
        if tactic == Tactic::GreatStart {
            let drawn = events
                .iter()
                .filter(|e| matches!(e, Event::CardDrawn { .. }))
                .count();
            assert_eq!(drawn, 2, "C2 : Départ flamboyant");
            assert_eq!(after.hand.len(), 7, "C2 : main de 7");
            self.seen.great_start = true;
        }
    }

    /// C7 : Préparation, la première carte du deck qui n'est pas une Blessure.
    fn prepare(&mut self, legal: &[Action]) {
        let cards: Vec<CardId> = legal
            .iter()
            .map(|action| match action {
                Action::PrepareCard { card } => *card,
                other => panic!("C7 : {other:?} pendant Préparation"),
            })
            .collect();
        let card = cards
            .iter()
            .copied()
            .find(|card| !card.card().is_wound())
            .unwrap_or(cards[0]);
        let events = self.p.play(Action::PrepareCard { card });
        assert!(events.contains(&Event::CardSearched { player: SOLO, card }));
        assert!(self.view().hand.contains(&card), "C7 : la carte en main");
        self.seen.preparation = true;
    }

    fn check_step(&mut self, step: &Step) {
        self.check_dummy_turn(step);
        self.check_announces(step);
        self.check_new_round(step);
        self.check_tiles(step);
    }

    /// C3, C4 : un tour du fantôme retourne 3 cartes, plus une par cristal de la couleur de la
    /// troisième (moins s'il en reste moins).
    fn check_dummy_turn(&mut self, step: &Step) {
        for (i, event) in step.events.iter().enumerate() {
            let Event::DummyTurnPlayed { cards } = event else {
                continue;
            };
            let before = dummy(&step.before);
            let first = before.deck_size.min(3);
            let extra = if first == 0 {
                0
            } else {
                card_color(cards[first - 1]).map_or(0, |color| before.crystals(color))
            };
            let expected = (first + usize::from(extra)).min(before.deck_size);
            assert_eq!(cards.len(), expected, "C4 : cartes retournées");
            // C3 : le tour du fantôme précède un tour du joueur, ou la fin de la manche.
            let next = &step.events[i + 1..];
            assert!(
                next.iter()
                    .any(|e| matches!(e, Event::TurnStarted { .. } | Event::RoundEnded { .. })),
                "C3 : {:?}",
                step.events
            );
            if !step
                .events
                .iter()
                .any(|e| matches!(e, Event::DummyDeckShuffled { .. }))
            {
                let after = dummy(&step.after);
                assert_eq!(&after.last_turn, cards, "C4 : DummyView::last_turn");
                assert_eq!(after.deck_size, before.deck_size - cards.len());
            }
        }
    }

    /// C5 : l'annonce du fantôme, et celle du joueur.
    fn check_announces(&mut self, step: &Step) {
        if self.seen.announced_this_round {
            assert!(
                !step.legal.contains(&Action::AnnounceEndOfRound),
                "C5 : le joueur ne peut plus annoncer"
            );
            assert!(
                !step
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::DummyTurnPlayed { .. })),
                "C5 : plus de tour du fantôme après son annonce"
            );
        }
        if step.events.contains(&Event::DummyAnnouncedEndOfRound) {
            assert_eq!(dummy(&step.before).deck_size, 0, "C5 : deck vide");
            self.seen.dummy_announced = true;
            self.seen.announced_this_round = true;
        }
        if step.action == Action::AnnounceEndOfRound {
            // Le fantôme joue son dernier tour, puis la manche finit.
            let turn = position_of(&step.events, |e| matches!(e, Event::DummyTurnPlayed { .. }))
                .expect("C5 : dernier tour du fantôme");
            let end = position_of(&step.events, |e| matches!(e, Event::RoundEnded { .. }))
                .expect("C5 : RoundEnded");
            assert!(turn < end);
        }
        if step
            .events
            .iter()
            .any(|e| matches!(e, Event::RoundEnded { .. }))
        {
            self.seen.announced_this_round = false;
        }
    }

    /// C6 : la fin de la manche, avant la suivante.
    fn check_new_round(&mut self, step: &Step) {
        let Some(started) = position_of(&step.events, |e| matches!(e, Event::RoundStarted { .. }))
        else {
            return;
        };
        let before = &step.before;
        let removed = step
            .events
            .iter()
            .find_map(|e| match e {
                Event::TacticsRemoved { tactics } => Some(tactics.clone()),
                _ => None,
            })
            .expect("C6 : TacticsRemoved");
        let mut expected = vec![
            before.tactic.as_ref().expect("tactique du joueur").tactic,
            dummy(before).tactic.expect("tactique du fantôme"),
        ];
        expected.sort();
        let mut removed_sorted = removed.clone();
        removed_sorted.sort();
        assert_eq!(removed_sorted, expected, "C6 : les deux tactiques retirées");
        let last_advanced = *before.offers[&Offer::AdvancedActions]
            .last()
            .expect("offre d'actions avancées");
        let CommonCard::Deed(last_advanced) = last_advanced else {
            panic!("une action avancée en dernière place")
        };
        assert!(
            step.events.contains(&Event::DummyCardAdded {
                card: last_advanced
            }),
            "C6 : DummyCardAdded"
        );
        let last_spell = *before.offers[&Offer::Spells]
            .last()
            .expect("offre de sorts");
        assert!(!competitive(last_spell), "C6 : jamais un sort compétitif");
        let CommonCard::Deed(spell) = last_spell else {
            panic!("un sort en dernière place")
        };
        let color = card_color(spell).expect("couleur du sort");
        assert!(
            step.events.contains(&Event::DummyCrystalGained { color }),
            "C6 : DummyCrystalGained"
        );
        let shuffled = step
            .events
            .iter()
            .find_map(|e| match e {
                Event::DummyDeckShuffled { order } => Some(order.len()),
                _ => None,
            })
            .expect("C6 : DummyDeckShuffled");
        let previous = if self.seen.new_rounds == 0 {
            16
        } else {
            self.seen.dummy_deck
        };
        assert_eq!(shuffled, previous + 1, "C6 : une carte de plus");
        self.seen.dummy_deck = shuffled;
        self.seen.new_rounds += 1;
        // Le retrait avant `RoundStarted` ; le fantôme prend sa carte et son cristal dans la
        // préparation de la nouvelle manche, avant le choix de la tactique (spec 061, C22 à C24).
        let removed_at = position_of(&step.events, |e| matches!(e, Event::TacticsRemoved { .. }));
        assert!(
            removed_at < Some(started),
            "C6 : retrait avant RoundStarted"
        );
        let ended = position_of(&step.events, |e| matches!(e, Event::RoundEnded { .. }))
            .expect("C6 : RoundEnded");
        assert!(
            !step.events[ended..]
                .iter()
                .any(|e| matches!(e, Event::TurnStarted { .. } | Event::DummyTurnPlayed { .. })),
            "C6 : nouvelle manche du fantôme avant tout tour : {:?}",
            step.events
        );
        let Event::RoundStarted { time_of_day, .. } = step.events[started] else {
            unreachable!()
        };
        assert_ne!(time_of_day, before.time_of_day, "C6 : le moment suivant");
    }

    /// C9, C10 : cités révélées, côte.
    fn check_tiles(&mut self, step: &Step) {
        // C10 : jamais d'exploration royaume vers la côte.
        if matches!(step.before.next_tile, NextTile::Core) {
            for action in &step.legal {
                if let Action::Explore { position } = action {
                    let (i, j) = position.lattice();
                    assert!(i != 0 && j != 0, "C10 : {position:?} sur la côte");
                }
            }
        }
        for (k, event) in step.events.iter().enumerate() {
            let Event::CityPlaced { hex, color, level } = *event else {
                continue;
            };
            assert!(
                step.events[..k]
                    .iter()
                    .any(|e| matches!(e, Event::TileRevealed { tile, .. } if tile.is_city())),
                "C9 : TileRevealed puis CityPlaced : {:?}",
                step.events
            );
            let expected_level = if self.seen.cities.is_empty() { 5 } else { 8 };
            assert_eq!(level, expected_level, "C9 : niveau de la cité");
            let mut defenders = step.after.face_down_defenders[&hex].clone();
            defenders.sort();
            let mut expected = city_defenders(color, level);
            expected.sort();
            assert_eq!(defenders, expected, "C9 : défenseurs de la cité");
            self.seen.cities.push(level);
        }
        if step
            .events
            .iter()
            .any(|e| matches!(e, Event::TileRevealed { .. }))
        {
            assert_eq!(step.after.fame, step.before.fame, "C9 : aucune gloire");
        }
    }
}

/// C1 — MKUE p. 19 ; p. 15 ; specs 061 (C3), 064 (C1, C3, C22).
fn check_setup(view: &PlayerView) {
    assert_eq!(view.scenario, Some(Scenario::solo_conquest()));
    assert_eq!((view.round, view.time_of_day), (1, TimeOfDay::Day));
    let conquest = view.conquest.expect("progression des cités");
    assert_eq!((conquest.conquered, conquest.cities), (0, 2));
    let dummy = dummy(view);
    assert_eq!(dummy.hero, Hero::Goldyx);
    assert_eq!(dummy.deck_size, 16);
    assert_eq!(
        (
            dummy.crystals(BasicColor::Green),
            dummy.crystals(BasicColor::Blue)
        ),
        (2, 1)
    );
    let spells =
        view.offers[&Offer::Spells].len() + view.common_decks[&arcanum_core::CommonDeck::Spells];
    assert_eq!(spells, 16, "C1 : 16 sorts");
    assert!(
        view.offers[&Offer::Spells]
            .iter()
            .all(|card| !competitive(*card))
    );
    let offered: Vec<Tactic> = view
        .tactic_choice
        .iter()
        .map(|(tactic, _)| *tactic)
        .collect();
    assert_eq!(offered, Tactic::of(TimeOfDay::Day).collect::<Vec<_>>());
    assert!(view.removed_tactics.is_empty());
}

fn new_main(seed: u64, strict: bool) -> J6 {
    let mut p = Parcours::arranged_choosing_tactics(&main_setup(seed));
    p.record_steps();
    check_setup(&p.view());
    J6 {
        p,
        seen: Seen::default(),
        strict,
    }
}

/// Joue jusqu'au choix de la tactique de la manche `round` (ou la fin de la partie).
fn play_until_round(j: &mut J6, round: u32) {
    for _ in 0..MAX_DECISIONS {
        let view = j.view();
        let choosing =
            j.p.legal()
                .iter()
                .any(|action| matches!(action, Action::ChooseTactic { .. }));
        if view.game_end.is_some() || (view.round == round && choosing) {
            return;
        }
        j.advance();
    }
    panic!("le script tourne en rond");
}

/// C8, C10, C11, L3 : la fin de la partie et le décompte.
fn check_end(j: &J6) {
    let view = j.view();
    let events = all_events(&j.p);
    // C11 : la 6e manche finie, la partie aussi.
    let end = events.len() - 1;
    assert_eq!(
        events[end],
        Event::GameEnded {
            reason: GameEndReason::LastRound
        }
    );
    assert!(events[..end].contains(&Event::RoundEnded { round: 6 }));
    assert!(j.p.legal().is_empty(), "C11 : plus aucune action");
    // C8 : les douze tactiques, chacune une fois.
    let mut taken = j.seen.taken.clone();
    taken.sort();
    let mut all = Tactic::ALL.to_vec();
    all.sort();
    assert_eq!(taken, all, "C8 : chaque tactique prise une fois");
    // C10 : aucune tuile royaume sur la côte.
    for (position, tile) in view.map.tiles() {
        if tile.kind() == TileKind::Core {
            let (i, j) = position.lattice();
            assert!(i != 0 && j != 0, "C10 : {tile:?} en {position:?}");
        }
    }
    // L3 : ni assaut ni ennemi provoqué.
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::AssaultStarted { .. } | Event::EnemiesProvoked { .. }
    )));
    // Préconditions vérifiées en route (L1).
    if j.strict {
        precondition(
            j.seen.cities == [5, 8],
            "départ à revoir : les deux cités révélées",
        );
        precondition(
            j.seen.dummy_announced,
            "départ à revoir : une annonce du fantôme",
        );
    }
    precondition(
        j.seen.great_start && j.seen.preparation,
        "départ à revoir : Départ flamboyant et Préparation pris",
    );
    // C11 : le décompte de la mission échouée.
    let score = view.final_score.expect("le décompte de la partie finie");
    let conquest = score.conquest.expect("partie « conquête » du décompte");
    assert_eq!((conquest.conquered, conquest.cities), (0, 2));
    assert!(!conquest.all_conquered, "C11 : mission échouée");
    assert_eq!(conquest.total(), 0);
    assert_eq!(score.fame, view.fame);
    use arcanum_core::Achievement::*;
    assert_eq!(
        score.total(),
        i64::from(view.fame)
            + [Knowledge, Loot, Leader, Conqueror, Adventurer, Beating]
                .into_iter()
                .map(|a| score.points(a))
                .sum::<i64>()
    );
}

/// Du début de la partie à la sauvegarde : au choix de la tactique de la manche 3.
fn up_to_the_save_point(seed: u64, strict: bool) -> J6 {
    let mut j = new_main(seed, strict);
    play_until_round(&mut j, 3);
    assert_eq!(j.view().round, 3, "départ à revoir : la manche 3 atteinte");
    j
}

fn after_the_save_point(j: &mut J6) {
    play_until_round(j, u32::MAX);
    check_end(j);
}

fn parcours(seed: u64, strict: bool) -> J6 {
    let mut j = up_to_the_save_point(seed, strict);
    after_the_save_point(&mut j);
    j
}

// C1 à C11 : le parcours principal, de la mise en place arrangée au décompte de la mission échouée.
#[test]
fn parcours_conquete_en_solitaire() {
    parcours(SEED, true);
}

// C12 — ADR 0001 ; specs 024, 038, 061 (C34) : reprise au début de la manche 3.
#[test]
fn parcours_reprise_avec_le_fantome() {
    let mut played = up_to_the_save_point(SEED, true);
    let saved: (EventLog, RngPositions) = played.p.save();
    let seen = played.seen.clone();
    let played_removed = played.view().removed_tactics;
    assert_eq!(played_removed.len(), 4);
    after_the_save_point(&mut played);

    let mut p = Parcours::resume_choosing_tactics(&saved);
    // Les tactiques retirées des deux premières manches reviennent du journal.
    assert_eq!(
        p.view().removed_tactics,
        played_removed,
        "C12 : tactiques retirées à la reprise"
    );
    p.record_steps();
    let mut resumed = J6 {
        p,
        seen,
        strict: true,
    };
    after_the_save_point(&mut resumed);
    assert_eq!(resumed.p.runner.log(), played.p.runner.log());
    assert_eq!(resumed.p.runner.state(), played.p.runner.state());
    assert_eq!(resumed.view().final_score, played.view().final_score);
}

/// Spec 069, Q6 : `cargo test -p arcanum-core --test parcours_j6 -- --ignored --nocapture` joue le
/// parcours principal sur les graines 0 à 99 et dit pourquoi les autres échouent.
#[test]
#[ignore]
fn balayer_les_graines() {
    let passed = support::sweep_seeds(0..100, |seed| {
        parcours(seed, false);
        episode(seed);
    });
    assert!(
        passed.len() >= SWEEP_THRESHOLD,
        "{} graines sur 100, moins que {SWEEP_THRESHOLD}",
        passed.len()
    );
}

// --- Épisode de la conquête (C13 à C19) ---

use arcanum_core::{
    BasicAction, CardPlay, CombatPhase, CommonDeck, DamageTarget, Enemy, EnemyToken, Hex, LastTurn,
    Spell, TilePosition, Unit, UnitCard, routes_avoiding,
};

/// Les emplacements posés des cinq tuiles de campagne, de la première cité, et celui où la seconde
/// sera révélée (`docs/reference/bac-a-sable.md` pour le réseau des emplacements).
const COUNTRYSIDE_POSITIONS: [(i32, i32); 5] = [(2, 0), (0, 2), (1, 2), (3, 0), (0, 3)];
const FIRST_CITY_POSITION: (i32, i32) = (2, 1);
const SECOND_CITY_POSITION: (i32, i32) = (1, 1);
/// Les tuiles de campagne de l'épisode, dans l'ordre (les deux de la mise en place d'abord) : le
/// terrain autour de la seconde cité ne dépend pas de la graine.
const EPISODE_COUNTRYSIDE: [u8; 7] = [10, 1, 4, 3, 2, 8, 7];
/// Gloire posée : le niveau 9 (4 armure, limite de main 7, 5 jetons de commandement).
const EPISODE_FAME: u32 = 80;

fn catapults(copy: u8) -> UnitCard {
    UnitCard {
        unit: Unit::Catapults,
        copy,
    }
}

fn gunners(copy: u8) -> UnitCard {
    UnitCard {
        unit: Unit::AmotepGunners,
        copy,
    }
}

fn spell_card(spell: Spell) -> CardId {
    CardId::new(DeedCard::Spell(spell), 1)
}

fn city_color(tile: TileId) -> BasicColor {
    match tile {
        TileId::Core(5) => BasicColor::Green,
        TileId::Core(6) => BasicColor::Blue,
        TileId::Core(7) => BasicColor::White,
        TileId::Core(8) => BasicColor::Red,
        other => panic!("{other:?} n'est pas une tuile de cité"),
    }
}

/// Un jeton de chaque couleur des défenseurs de la première cité, pour ses Boucliers posés.
fn shield_token(color: EnemyColor, copy: u8) -> EnemyToken {
    let enemy = match color {
        EnemyColor::Grey => Enemy::Swordsmen,
        EnemyColor::Violet => Enemy::FireMages,
        EnemyColor::Brown => Enemy::Gargoyle,
        EnemyColor::White => Enemy::Gunners,
        other => panic!("pas de défenseur de cité {other:?}"),
    };
    EnemyToken::new(enemy, copy)
}

/// Spec 069, « Épisode de la conquête » : la mise en place arrangée, puis la première cité conquise
/// et le héros armé, posés (Notes d'implémentation).
fn episode_setup(seed: u64) -> Setup {
    use BasicAction::*;
    let setup = Setup::new(config(seed));
    let cities = city_tiles(&setup);
    let first = cities
        .iter()
        .copied()
        .find(|tile| *tile != TileId::Core(8))
        .expect("une cité autre que la rouge");
    let setup = setup
        .countryside_tiles(&EPISODE_COUNTRYSIDE.map(TileId::Countryside))
        .core_tiles(&[first, TileId::Core(8)])
        // Les défenseurs de la cité rouge au niveau 8 : brun, violet, blanc, blanc.
        .next_enemy_kinds(EnemyColor::Brown, &[Enemy::Minotaur])
        // Les tours de mage de la mise en place gardent leurs Mages de glace : les Moines restent.
        .enemies(
            EnemyColor::Violet,
            &[
                EnemyToken::new(Enemy::IceMages, 1),
                EnemyToken::new(Enemy::IceMages, 2),
            ],
        )
        .next_enemy_kinds(EnemyColor::Violet, &[Enemy::Monks])
        .next_enemy_kinds(EnemyColor::White, &[Enemy::Freezers, Enemy::Freezers])
        // Main du premier tour, puis la suite du deck.
        .deck_cards(&[
            DeedCard::Basic(March),
            DeedCard::Basic(Stamina),
            DeedCard::Basic(Stamina),
            DeedCard::Basic(Swiftness),
            DeedCard::Basic(Swiftness),
            DeedCard::Basic(March),
            DeedCard::Basic(Rage),
            DeedCard::Basic(Tranquility),
        ]);
    let deck = setup.tile_deck();
    let countryside: Vec<TileId> = deck
        .iter()
        .copied()
        .filter(|tile| tile.kind() == TileKind::Countryside)
        .skip(2)
        .collect();
    let mut posed: Vec<Event> = countryside
        .iter()
        .zip(COUNTRYSIDE_POSITIONS)
        .map(|(&tile, (i, j))| Event::TileRevealed {
            tile,
            position: TilePosition::from_lattice(i, j),
        })
        .collect();
    let (i, j) = FIRST_CITY_POSITION;
    let position = TilePosition::from_lattice(i, j);
    let hex = position.center();
    let color = city_color(first);
    posed.push(Event::TileRevealed {
        tile: first,
        position,
    });
    posed.push(Event::CityPlaced {
        hex,
        color,
        level: 5,
    });
    posed.extend(
        city_defenders(color, 5)
            .into_iter()
            .zip(1..)
            .map(|(color, copy)| Event::CityShieldPlaced {
                player: SOLO,
                hex,
                enemy: shield_token(color, copy),
            }),
    );
    posed.push(Event::CityConquered { hex, leader: SOLO });
    posed.push(Event::FameGained {
        player: SOLO,
        fame: EPISODE_FAME,
    });
    posed.extend((2..=9).map(|level| Event::LevelGained {
        player: SOLO,
        level,
    }));
    posed.extend(
        [
            BasicColor::Red,
            BasicColor::Red,
            BasicColor::Blue,
            BasicColor::Blue,
            BasicColor::Blue,
            BasicColor::Green,
            BasicColor::Green,
            BasicColor::White,
        ]
        .map(|color| Event::CrystalGained {
            player: SOLO,
            color,
        }),
    );
    setup
        .pose(posed)
        .recruit(catapults(1))
        .recruit(catapults(2))
        .recruit(gunners(1))
        .recruit(gunners(2))
        // Gagnés sur le dessus du deck : la main du deuxième tour.
        .gain_on_deck(
            CommonDeck::Spells,
            CommonCard::Deed(spell_card(Spell::Chill)),
        )
        .gain_on_deck(
            CommonDeck::Spells,
            CommonCard::Deed(spell_card(Spell::Whirlwind)),
        )
}

/// La case des tuiles posées d'où explorer l'emplacement de la seconde cité : sans site, ni
/// infranchissable, voisine de la colline N-E de la tuile royaume 8, la seule case voisine de la
/// cité d'où l'assaillir sans provoquer ses Draconum (O et S-E, `tiles.ron`).
fn episode_start(setup: &Setup) -> Hex {
    let view = Parcours::arranged_choosing_tactics(setup).view();
    let (i, j) = SECOND_CITY_POSITION;
    let target = TilePosition::from_lattice(i, j);
    let hills = target.hex_at(arcanum_core::Direction::NE);
    view.map
        .tiles()
        .flat_map(|(position, _)| position.hexes())
        .find(|hex| {
            hex.is_adjacent(hills)
                && view.map.space(*hex).is_some_and(|space| {
                    space.feature.is_none()
                        && matches!(
                            arcanum_core::move_cost(space, TimeOfDay::Day),
                            arcanum_core::MoveCost::Points(_)
                        )
                })
                && support::explorable_from(&view, *hex).contains(&target)
        })
        .expect("une case d'où explorer la seconde cité")
}

/// Joue les cartes de la main qui donnent du Mouvement, la plus forte d'abord, jusqu'à `points`,
/// sans renforcer une carte bleue : le mana bleu est gardé pour les Catapultes et Frisson.
fn gather_move(p: &mut Parcours, points: u32) {
    while p.view().move_points < points {
        let best = support::card_plays(p)
            .into_iter()
            .filter(|(_, card, play, _)| {
                support::single_card(play)
                    && !(play.is_strong() && card_color(*card) == Some(BasicColor::Blue))
            })
            .filter_map(|(action, _, _, effect)| {
                support::move_amount(effect).map(|amount| (amount, action))
            })
            .max_by_key(|(amount, _)| *amount);
        let (_, action) = best.expect("assez de Mouvement en main");
        p.play(action);
    }
}

/// Active `unit` pour sa capacité `ability`, la première façon de la payer.
fn activate(p: &mut Parcours, unit: UnitCard, ability: u8) -> Vec<Event> {
    let action = p
        .legal()
        .into_iter()
        .find(|action| {
            matches!(action, Action::ActivateUnit { unit: u, ability: a, .. }
                if *u == unit && *a == ability)
        })
        .unwrap_or_else(|| panic!("{unit:?} activable pour sa capacité {ability}"));
    p.play(action)
}

/// Lance `spell` (effet de base) sur `enemy`.
fn cast_at(p: &mut Parcours, spell: Spell, enemy: EnemyToken) -> Vec<Event> {
    let card = spell_card(spell);
    let action = p
        .legal()
        .into_iter()
        .find(|action| {
            matches!(action, Action::PlayCard { card: c, play: CardPlay::Cast { black: None, enemy: Some(e), .. } }
                if *c == card && *e == enemy)
        })
        .unwrap_or_else(|| panic!("{spell:?} lançable sur {enemy:?}"));
    p.play(action)
}

/// Passe aux phases suivantes jusqu'à la phase d'attaque, les dégâts non parés au héros.
fn to_attack_phase(p: &mut Parcours) -> Vec<Event> {
    let mut events = Vec::new();
    while support::combat(&p.view()).phase != CombatPhase::Attack {
        let assign = p.legal().into_iter().find(|action| {
            matches!(
                action,
                Action::AssignDamage {
                    to: DamageTarget::Hero,
                    ..
                }
            )
        });
        events.extend(p.play(assign.unwrap_or(Action::NextCombatPhase)));
    }
    events
}

fn shields_placed(events: &[Event]) -> Vec<EnemyToken> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::CityShieldPlaced { enemy, .. } => Some(*enemy),
            _ => None,
        })
        .collect()
}

fn knocked_out(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::KnockedOut { .. }))
}

/// Rend la partie jouée : `scripts/temoins-release.sh` la sauvegarde comme témoin (issue #84).
fn episode(seed: u64) -> Parcours {
    let base = episode_setup(seed);
    let setup = base.clone().figure(episode_start(&base));
    let mut p = Parcours::arranged_choosing_tactics(&setup);
    let view = p.view();
    let conquest = view.conquest.expect("progression des cités");
    precondition(
        (conquest.conquered, conquest.cities) == (1, 2) && view.level == 9,
        "départ à revoir : première cité conquise, niveau 9",
    );
    precondition(view.units.len() == 4, "départ à revoir : quatre unités");
    let fame_before = view.fame;
    p.play(Action::ChooseTactic {
        tactic: Tactic::EarlyBird,
    });

    // C13 — MKUE p. 7, 5.g ; p. 19, *Cities* ; specs 062 (C4, C5), 064 (C22).
    gather_move(&mut p, 2);
    let (i, j) = SECOND_CITY_POSITION;
    let position = TilePosition::from_lattice(i, j);
    let city = position.center();
    let events = p.play(Action::Explore { position });
    assert!(events.contains(&Event::TileRevealed {
        tile: TileId::Core(8),
        position
    }));
    assert!(events.contains(&Event::CityPlaced {
        hex: city,
        color: BasicColor::Red,
        level: 8,
    }));
    let defenders: Vec<EnemyToken> = events
        .iter()
        .filter_map(|event| match event {
            Event::DefenderPlaced { token, hex } if *hex == city => Some(*token),
            _ => None,
        })
        .collect();
    let colors: Vec<EnemyColor> = defenders
        .iter()
        .map(|token| token.enemy().definition().color)
        .collect();
    assert_eq!(colors, city_defenders(BasicColor::Red, 8), "C13");
    let conquest = p.view().conquest.expect("progression");
    assert_eq!((conquest.conquered, conquest.cities), (1, 2));
    let [minotaur, monks, freezers_1, freezers_2] = defenders[..] else {
        panic!("quatre défenseurs")
    };

    // La case voisine de la cité d'où l'assaillir, sans provoquer d'ennemi errant.
    let view = p.view();
    let enemies: Vec<Hex> = view.enemy_tokens.keys().copied().collect();
    let from = routes_avoiding(
        &view.map,
        view.figure.expect("figurine"),
        view.time_of_day,
        &enemies,
    )
    .into_iter()
    .filter(|route| {
        route.to.is_adjacent(city) && arcanum_core::provoked(route.to, city, &enemies).is_empty()
    })
    .min_by_key(|route| route.cost)
    .expect("une case voisine de la cité");
    let cost = from.cost + support::step_cost(&view, city);
    gather_move(&mut p, cost);
    for hex in from.steps.clone() {
        p.play(Action::MoveHero { to: hex });
    }
    let from = from.to;

    // C14 — MKUE p. 10, *City Assaults*, 1.a, 3.d ; Compendium, *Cities* S23 ; spec 062.
    let events = p.play(Action::MoveHero { to: city });
    assert!(events.contains(&Event::AssaultStarted {
        player: SOLO,
        site: city,
        from,
    }));
    assert!(events.contains(&Event::ReputationChanged {
        player: SOLO,
        change: -1,
    }));
    let mut fight = p.play(Action::StartCombat);
    let combat = support::combat(&p.view());
    let tokens: Vec<EnemyToken> = combat.enemies.iter().map(|enemy| enemy.token).collect();
    assert_eq!(tokens, defenders, "C14 : les quatre défenseurs");
    assert!(combat.enemies.iter().all(|enemy| enemy.site_fortified));
    let monks_in_combat = &combat.enemies[1];
    assert!(
        monks_in_combat
            .city_abilities()
            .contains(&arcanum_core::EnemyAbility::Brutal),
        "C14 : bonus de la cité rouge"
    );
    // Phase de tir : les Catapultes (siège) battent le Minotaure et les Moines.
    fight.extend(p.play(Action::DeclareAttack {
        targets: vec![minotaur, monks],
    }));
    fight.extend(activate(&mut p, catapults(1), 1));
    fight.extend(activate(&mut p, catapults(2), 2));
    fight.extend(p.play(Action::ResolveAttack));
    assert!(fight.iter().any(|e| matches!(
        e,
        Event::EnemiesDefeated { enemies, .. } if *enemies == vec![minotaur, monks]
    )));
    fight.extend(to_attack_phase(&mut p));
    let events = p.play(Action::EndCombat);
    fight.extend(events.clone());
    assert_eq!(shields_placed(&events), vec![minotaur, monks], "C14");
    let shield = position_of(&events, |e| matches!(e, Event::CityShieldPlaced { .. }));
    let withdrew = position_of(&events, |e| {
        *e == Event::HeroWithdrew {
            player: SOLO,
            from: city,
            to: from,
        }
    });
    assert!(withdrew.is_some() && shield < withdrew, "C14 : {events:?}");
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::CityConquered { .. } | Event::LastTurnAnnounced { .. }
    )));
    let view = p.view();
    assert_eq!(view.cities[&city].shields, vec![SOLO, SOLO]);
    assert_eq!(view.conquest.map(|c| c.conquered), Some(1));
    assert!(!knocked_out(&fight), "L2");
    let events = support::end_turn(&mut p);
    assert!(
        !events.contains(&Event::DummyAnnouncedEndOfRound),
        "L6 : le fantôme n'annonce pas"
    );

    // C15 — MKUE p. 10, 2.c ; p. 19, *Scenario End* ; specs 062 (C20), 064 (C11).
    let cost = support::step_cost(&p.view(), city);
    gather_move(&mut p, cost);
    let events = p.play(Action::MoveHero { to: city });
    assert!(events.contains(&Event::ReputationChanged {
        player: SOLO,
        change: -1,
    }));
    let mut fight = p.play(Action::StartCombat);
    let tokens: Vec<EnemyToken> = support::combat(&p.view())
        .enemies
        .iter()
        .map(|enemy| enemy.token)
        .collect();
    assert_eq!(tokens, vec![freezers_1, freezers_2], "C15 : les restants");
    fight.extend(cast_at(&mut p, Spell::Whirlwind, freezers_1));
    fight.extend(cast_at(&mut p, Spell::Chill, freezers_2));
    fight.extend(to_attack_phase(&mut p));
    fight.extend(p.play(Action::DeclareAttack {
        targets: vec![freezers_1, freezers_2],
    }));
    fight.extend(activate(&mut p, gunners(1), 0));
    fight.extend(activate(&mut p, gunners(2), 0));
    let rage = support::card_plays(&p)
        .into_iter()
        .filter(|(_, card, play, _)| {
            card.card() == DeedCard::Basic(BasicAction::Rage) && play.is_strong()
        })
        .map(|(action, ..)| action)
        .next()
        .expect("Rage renforcée");
    fight.extend(p.play(rage));
    fight.extend(p.play(Action::ResolveAttack));
    let events = p.play(Action::EndCombat);
    fight.extend(events.clone());
    assert_eq!(shields_placed(&events), vec![freezers_1, freezers_2]);
    let conquered = position_of(&events, |e| {
        *e == Event::CityConquered {
            hex: city,
            leader: SOLO,
        }
    })
    .expect("C15 : CityConquered");
    assert!(matches!(
        events.get(conquered + 1),
        Some(Event::LastTurnAnnounced { player: SOLO, .. })
    ));
    let view = p.view();
    assert_eq!(view.cities[&city].shields, vec![SOLO; 4]);
    assert_eq!(view.conquest.map(|c| c.conquered), Some(2));
    assert!(!knocked_out(&fight), "L2");

    // C16 — MKUE p. 19, *the Dummy player does not* ; Compendium, *Cities* S21 ; spec 064 (C12).
    // Le niveau 10 atteint : la compétence et l'action avancée prises avant la pioche (L4).
    let mut events = support::end_turn(&mut p);
    events.extend(take_pending_reward(&mut p));
    assert!(events.iter().any(|e| matches!(e, Event::TurnEnded { .. })));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TurnStarted { .. })),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::DummyTurnPlayed { .. })),
        "C16 : pas de tour du fantôme"
    );
    assert_eq!(p.view().last_turn, LastTurn::Current);

    // C17 — MKUE p. 19 ; spec 064 (C12) : une carte de côté, puis la fin du dernier tour.
    let events = p.filler_turn();
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded {
            reason: GameEndReason::AllCitiesConquered
        })
    );
    assert!(p.legal().is_empty());

    // C18 — MKUE p. 19, *Scoring* ; spec 064 (C16, C17).
    let view = p.view();
    precondition(
        !all_events(&p).iter().any(|e| {
            matches!(
                e,
                Event::DummyAnnouncedEndOfRound | Event::RoundEnded { .. }
            )
        }),
        "départ à revoir : la manche 1 sans annonce",
    );
    let score = view.final_score.expect("décompte");
    let conquest = score.conquest.expect("partie « conquête »");
    assert_eq!((conquest.conquered, conquest.cities), (2, 2));
    assert!(conquest.all_conquered);
    assert_eq!(conquest.unplayed_rounds, 6 - view.round);
    assert_eq!(
        conquest.dummy_deck as usize,
        dummy(&view).deck_size,
        "C18 : deck du fantôme"
    );
    assert!(!conquest.round_announced);
    use arcanum_core::Achievement::*;
    let achievements: i64 = [Knowledge, Loot, Leader, Conqueror, Adventurer, Beating]
        .into_iter()
        .map(|a| score.points(a))
        .sum();
    assert_eq!(
        score.total(),
        i64::from(view.fame)
            + achievements
            + 10 * 2
            + 15
            + 30 * i64::from(6 - view.round)
            + dummy(&view).deck_size as i64
            + 5
    );

    // C19 — Compendium, *Final Scoring* S2, *Cities* S23 ; specs 062, 064 (C20).
    assert_eq!(
        score.shields, 0,
        "C19 : les Boucliers des cités hors du Conquérant"
    );
    let defeated: u32 = defenders
        .iter()
        .map(|token| u32::from(token.enemy().definition().fame))
        .sum();
    assert_eq!(
        view.fame,
        fame_before + defeated,
        "C19 : gloire des défenseurs"
    );
    p
}

// C13 à C19 : l'épisode de la conquête, depuis l'état posé.
#[test]
fn parcours_conquete_des_cites() {
    episode(SEED);
}
