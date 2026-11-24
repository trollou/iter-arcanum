//! Outils des parcours de bout en bout (ADR 0006, spec 028) : joueur scripté et tour de remplissage.

// Module partagé par plusieurs crates de test (parcours_j1, parcours_j2) : chacune n'en utilise
// qu'une partie.
#![allow(dead_code)]

pub mod arrangement;

use std::cell::RefCell;
use std::rc::Rc;

use arcanum_core::{
    Action, AttackKind, BasicAction, CardId, CardPlay, Combat, DeedCard, Effect, Event, EventLog,
    GameConfig, GameRunner, Hex, Offer, Placement, PlayerId, PlayerView, RecruitSite, RngPositions,
    SidewaysEffect, StepOutcome, Tactic, Terrain, TilePosition, TimeOfDay, UndoError, UnitCard,
    free_positions_touching, legal_actions, placement, routes_avoiding, view,
};

pub const SOLO: PlayerId = PlayerId::SOLO;

/// Ce que le test et le joueur scripté partagent : l'action voulue, et ce que la dernière
/// demande a montré.
#[derive(Default)]
struct Shared {
    wanted: Option<Action>,
}

/// Joueur scripté : il répond l'index de l'action voulue, retrouvée par sa valeur dans les
/// actions légales de chaque demande (jamais un index gardé d'une décision à l'autre).
pub struct ScriptedPlayer {
    shared: Rc<RefCell<Shared>>,
    legal: Vec<Action>,
}

impl arcanum_core::Player for ScriptedPlayer {
    fn request(&mut self, _view: &PlayerView, legal_actions: &[Action]) {
        self.legal = legal_actions.to_vec();
    }

    fn poll(&mut self) -> Option<usize> {
        let wanted = self.shared.borrow_mut().wanted.take()?;
        let index = self.legal.iter().position(|action| *action == wanted);
        // Spec 028, L1 : le test échoue en nommant l'action voulue et les actions légales.
        assert!(
            index.is_some(),
            "action voulue absente des actions légales : {wanted:?}\nactions légales : {:#?}",
            self.legal
        );
        index
    }
}

/// Une partie jouée par un joueur scripté, avec les vérifications communes des parcours.
pub struct Parcours {
    pub runner: GameRunner,
    shared: Rc<RefCell<Shared>>,
    /// Spec 069 : faux quand le script choisit lui-même ses tactiques (`choosing_tactics`).
    auto_tactics: bool,
    /// Spec 069 : chaque décision prise, gardée pour les vérifications du parcours, si demandé.
    steps: Option<Vec<Step>>,
}

/// Spec 069 : une décision prise, avec la vue et les actions légales d'avant, et ses événements.
pub struct Step {
    pub before: PlayerView,
    pub legal: Vec<Action>,
    pub action: Action,
    pub events: Vec<Event>,
    pub after: PlayerView,
}

impl Parcours {
    fn player() -> (Box<ScriptedPlayer>, Rc<RefCell<Shared>>) {
        let shared = Rc::new(RefCell::new(Shared::default()));
        let player = ScriptedPlayer {
            shared: Rc::clone(&shared),
            legal: Vec::new(),
        };
        (Box::new(player), shared)
    }

    pub fn new(config: GameConfig) -> Self {
        let (player, shared) = Self::player();
        let mut parcours = Self {
            runner: GameRunner::new(config, vec![player]),
            shared,
            auto_tactics: true,
            steps: None,
        };
        parcours.past_tactic_choice();
        parcours
    }

    /// Spec 060 : prend la tactique sans effet si son choix attend (Lève-tôt, Au crépuscule, ou une
    /// fois retirées Au bon moment, Appel du mana, qui n'agissent que si on s'en sert), pour que
    /// les parcours d'avant la spec jouent leurs tours comme avant.
    pub fn past_tactic_choice(&mut self) -> Vec<Event> {
        if !self.auto_tactics {
            return Vec::new();
        }
        let tactics = match self.view().time_of_day {
            TimeOfDay::Day => [Tactic::EarlyBird, Tactic::TheRightMoment],
            TimeOfDay::Night => [Tactic::FromTheDusk, Tactic::ManaSearch],
        };
        let legal = self.legal();
        let tactic = tactics
            .map(|tactic| Action::ChooseTactic { tactic })
            .into_iter()
            .find(|action| legal.contains(action));
        match tactic {
            Some(action) => self.play(action),
            None => Vec::new(),
        }
    }

    /// Reprend une partie depuis une sauvegarde en mémoire (spec 028, Q2), avec un nouveau
    /// joueur scripté.
    pub fn resume(saved: &(EventLog, RngPositions)) -> Self {
        let (player, shared) = Self::player();
        let runner = GameRunner::resume(saved.0.clone(), saved.1.clone(), vec![player])
            .expect("la sauvegarde vient d'une vraie partie");
        let mut parcours = Self {
            runner,
            shared,
            auto_tactics: true,
            steps: None,
        };
        parcours.past_tactic_choice();
        parcours
    }

    /// Spec 069 : comme `resume`, mais le script choisit lui-même chaque tactique (aucun choix
    /// d'office, ni à la reprise ni aux nouvelles manches).
    pub fn resume_choosing_tactics(saved: &(EventLog, RngPositions)) -> Self {
        let (player, shared) = Self::player();
        let runner = GameRunner::resume(saved.0.clone(), saved.1.clone(), vec![player])
            .expect("la sauvegarde vient d'une vraie partie");
        Self {
            runner,
            shared,
            auto_tactics: false,
            steps: None,
        }
    }

    /// Spec 069 : une mise en place arrangée où le script choisit ses tactiques.
    pub fn arranged_choosing_tactics(setup: &arrangement::Setup) -> Self {
        Self::resume_choosing_tactics(&setup.save())
    }

    /// Part d'une mise en place arrangée (issue privée 318), avec un nouveau joueur scripté.
    pub fn arranged(setup: &arrangement::Setup) -> Self {
        Self::resume(&setup.save())
    }

    /// Ce qu'une sauvegarde enregistre : le journal et les positions des générateurs (spec 024, 038).
    pub fn save(&self) -> (EventLog, RngPositions) {
        (self.runner.log().clone(), self.runner.rng_positions())
    }

    pub fn view(&self) -> PlayerView {
        view(self.runner.state(), SOLO)
    }

    pub fn legal(&self) -> Vec<Action> {
        legal_actions(self.runner.state(), SOLO)
    }

    /// Spec 069 : garder désormais chaque décision prise (`take_steps`).
    pub fn record_steps(&mut self) {
        self.steps.get_or_insert_with(Vec::new);
    }

    /// Les décisions gardées depuis le dernier appel.
    pub fn take_steps(&mut self) -> Vec<Step> {
        self.steps.as_mut().map(std::mem::take).unwrap_or_default()
    }

    /// Joue l'action voulue et rend les événements qu'elle a produits.
    pub fn play(&mut self, action: Action) -> Vec<Event> {
        let recorded = self
            .steps
            .is_some()
            .then(|| (self.view(), self.legal(), action.clone()));
        let before = self.runner.log().len();
        self.shared.borrow_mut().wanted = Some(action);
        let outcome = self.runner.step().expect("action acceptée par le moteur");
        assert_eq!(
            outcome,
            StepOutcome::Advanced,
            "la décision n'a pas été prise"
        );
        let mut events: Vec<Event> = self.runner.log().entries()[before..]
            .iter()
            .map(|entry| entry.event.clone())
            .collect();
        if let Some((view, legal, action)) = recorded {
            let after = self.view();
            if let Some(steps) = self.steps.as_mut() {
                steps.push(Step {
                    before: view,
                    legal,
                    action,
                    events: events.clone(),
                    after,
                });
            }
        }
        // Spec 060 : la nouvelle manche s'ouvre sur la tactique sans effet, comme avant la spec.
        events.extend(self.past_tactic_choice());
        events
    }

    pub fn undo(&mut self) -> Result<(), UndoError> {
        self.runner.undo()
    }

    /// Partie finie : plus de décision, `step` répond `Finished`.
    pub fn is_finished(&mut self) -> bool {
        self.runner.step().expect("pas d'erreur de la boucle") == StepOutcome::Finished
    }

    /// Tour de remplissage (spec 028, Modèle) : annoncer la fin de la manche si c'est légal ;
    /// sinon jouer la première carte de la main de côté (Mouvement 1), puis finir le tour sans
    /// défausser. Si une carte a déjà été jouée ce tour, il la complète de la même façon.
    pub fn filler_turn(&mut self) -> Vec<Event> {
        if self.legal().contains(&Action::AnnounceEndOfRound) {
            return self.play(Action::AnnounceEndOfRound);
        }
        let mut events = Vec::new();
        let view = self.view();
        // Une Blessure ne se joue pas (spec 031) : la première autre carte.
        if let Some(card) = first_playable(&view) {
            events.extend(self.play(Action::PlayCard {
                card,
                play: CardPlay::Sideways(SidewaysEffect::Move),
            }));
        }
        // Main faite de Blessures et aucune carte jouée : le repos prolongé en défausse une
        // (MKUE p. 6, 7.a ; spec 031, Q12).
        let slow_recovery = first_playable(&view).is_none() && view.play_area.is_empty();
        events.extend(
            self.play(Action::EndTurn {
                discard: view
                    .hand
                    .first()
                    .copied()
                    .filter(|_| slow_recovery)
                    .into_iter()
                    .collect(),
                reclaim: None,
                glade: None,
                steady_tempo: false,
            }),
        );
        events
    }

    /// Tours de remplissage jusqu'à la fin de la manche en cours (ou de la partie).
    pub fn finish_round(&mut self) -> Vec<Event> {
        let round = self.view().round;
        let mut events = Vec::new();
        while self.view().round == round && !self.is_finished() {
            events.extend(self.filler_turn());
        }
        events.extend(self.past_tactic_choice());
        events
    }
}

/// Le premier exemplaire en main d'une carte d'action de base.
pub fn in_hand(view: &PlayerView, action: BasicAction) -> Option<CardId> {
    view.hand
        .iter()
        .copied()
        .find(|card| card.card() == DeedCard::Basic(action))
}

/// La première carte de la main qui n'est pas une Blessure : une Blessure ne se joue pas
/// (spec 031).
pub fn first_playable(view: &PlayerView) -> Option<CardId> {
    view.hand
        .iter()
        .copied()
        .find(|card| !card.card().is_wound())
}

/// Joue la première carte de la main (hors Blessure) de côté, pour Mouvement 1.
pub fn sideways_move(parcours: &mut Parcours) {
    let card = first_playable(&parcours.view()).expect("une carte à jouer dans la main");
    parcours.play(Action::PlayCard {
        card,
        play: CardPlay::Sideways(SidewaysEffect::Move),
    });
}

/// Suit le trajet le moins cher vers `to` (spec 017), case par case, et rend son coût. Le trajet
/// évite les ennemis errants et les pas qui les provoqueraient (spec 030, C19).
pub fn walk_to(parcours: &mut Parcours, to: Hex) -> u32 {
    let view = parcours.view();
    let from = view.figure.expect("la figurine est posée");
    let enemies: Vec<Hex> = view.enemy_tokens.keys().copied().collect();
    let route = routes_avoiding(&view.map, from, view.time_of_day, &enemies)
        .into_iter()
        .find(|route| route.to == to)
        .expect("la case est atteignable");
    for hex in route.steps {
        parcours.play(Action::MoveHero { to: hex });
    }
    route.cost
}

pub fn has(events: &[Event], wanted: impl Fn(&Event) -> bool) -> bool {
    events.iter().any(wanted)
}

/// Finit le tour sans défausser.
pub fn end_turn(parcours: &mut Parcours) -> Vec<Event> {
    parcours.play(Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    })
}

/// Les emplacements où la figurine, sur `hex`, pourrait révéler la prochaine tuile (MKUE p. 7,
/// 5.a et 5.e ; spec 015).
pub fn explorable_from(view: &PlayerView, hex: Hex) -> Vec<TilePosition> {
    let Some(shape) = view.scenario.map(|scenario| scenario.shape()) else {
        return Vec::new();
    };
    free_positions_touching(&view.map, hex)
        .into_iter()
        .filter(|position| {
            placement(&view.map, shape, *position, view.next_tile) == Placement::Allowed
        })
        .collect()
}

/// La décision qui attend en fin de tour, s'il y en a une : une compétence puis une action avancée
/// d'un niveau pair (specs 050, 051) ; l'explorateur prend la première proposée (spec 049, L4).
pub fn take_pending_reward(parcours: &mut Parcours) -> Vec<Event> {
    let mut events = Vec::new();
    while let Some(action) = parcours.legal().into_iter().find(|action| {
        matches!(
            action,
            Action::GainAdvancedAction { .. } | Action::ChooseSkill { .. }
        )
    }) {
        events.extend(parcours.play(action));
    }
    events
}

impl Parcours {
    /// Tour d'exploration (spec 049, « L'explorateur scripté ») : annoncer la fin de la manche si
    /// c'est légal ; sinon jouer de côté pour Mouvement les cartes qui ne sont pas des Blessures
    /// jusqu'à pouvoir explorer ; explorer si c'est légal, sinon marcher vers la case la moins
    /// chère d'où explorer, aussi loin que les points le permettent ; finir le tour sans défausser
    /// (repos prolongé si la main n'a que des Blessures), et prendre la récompense d'un niveau pair.
    /// Si une carte a déjà été jouée ce tour, il la complète de la même façon.
    pub fn explorer_turn(&mut self) -> Vec<Event> {
        if self.legal().contains(&Action::AnnounceEndOfRound) {
            return self.play(Action::AnnounceEndOfRound);
        }
        let mut events = Vec::new();
        loop {
            let view = self.view();
            let figure = view.figure.expect("la figurine est posée");
            if let Some(position) = explorable_from(&view, figure).into_iter().find(|position| {
                self.legal().contains(&Action::Explore {
                    position: *position,
                })
            }) {
                events.extend(self.play(Action::Explore { position }));
                continue;
            }
            // Le trajet le moins cher vers une case d'où explorer : la figurine sur une telle case
            // n'a besoin que des 2 points de l'exploration.
            let enemies: Vec<Hex> = view.enemy_tokens.keys().copied().collect();
            let target = if explorable_from(&view, figure).is_empty() {
                routes_avoiding(&view.map, figure, view.time_of_day, &enemies)
                    .into_iter()
                    .filter(|route| !explorable_from(&view, route.to).is_empty())
                    .min_by_key(|route| route.cost)
            } else {
                None
            };
            let needed = target.as_ref().map_or(0, |route| route.cost) + 2;
            if view.move_points < needed
                && let Some(card) = first_playable(&view)
            {
                events.extend(self.play(Action::PlayCard {
                    card,
                    play: CardPlay::Sideways(SidewaysEffect::Move),
                }));
                continue;
            }
            // Plus de carte à jouer, ou assez de points : marcher aussi loin que possible.
            let mut moved = false;
            if let Some(route) = target {
                for hex in route.steps {
                    if !self.legal().contains(&Action::MoveHero { to: hex }) {
                        break;
                    }
                    events.extend(self.play(Action::MoveHero { to: hex }));
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        let view = self.view();
        let slow_recovery = first_playable(&view).is_none() && view.play_area.is_empty();
        events.extend(
            self.play(Action::EndTurn {
                discard: view
                    .hand
                    .first()
                    .copied()
                    .filter(|_| slow_recovery)
                    .into_iter()
                    .collect(),
                reclaim: None,
                glade: None,
                steady_tempo: false,
            }),
        );
        events.extend(take_pending_reward(self));
        events
    }
}

/// Spec 051 : la première compétence révélée à un niveau pair, gardée ; ses événements.
pub fn choose_first_skill(parcours: &mut Parcours) -> Vec<Event> {
    let action = parcours
        .legal()
        .into_iter()
        .find(|action| matches!(action, Action::ChooseSkill { .. }))
        .expect("une compétence à choisir au niveau pair");
    parcours.play(action)
}

// Aides des parcours du J4 et du J5 (spec 059, Modèle : deux utilisateurs, règle de la 049).

/// Spec 049, L1 : une précondition qui ne tient plus vient de la graine, pas des règles.
pub fn precondition(holds: bool, what: &str) {
    assert!(holds, "graine à rechoisir : {what}");
}

/// Le plateau de jeu traversé ne dépend pas de la graine.
pub fn expect_terrain(parcours: &Parcours, hex: Hex, terrain: Terrain) {
    let space = parcours.view().map.space(hex);
    assert_eq!(
        space.map(|space| space.terrain),
        Some(terrain),
        "case {hex:?} : la mise en place a changé"
    );
}

pub fn combat(view: &PlayerView) -> Combat {
    view.combat.clone().expect("un combat en cours")
}

pub fn all_events(parcours: &Parcours) -> Vec<Event> {
    parcours
        .runner
        .log()
        .entries()
        .iter()
        .map(|entry| entry.event.clone())
        .collect()
}

/// Une carte jouée légale et l'effet qu'elle donne.
pub fn card_plays(parcours: &Parcours) -> Vec<(Action, CardId, CardPlay, Effect)> {
    parcours
        .legal()
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } => card
                .effect(play)
                .map(|effect| (action.clone(), card, play, effect)),
            _ => None,
        })
        .collect()
}

/// Les cartes jouées sans défausser d'autre carte ni mettre de dé sur une couleur.
pub fn single_card(play: &CardPlay) -> bool {
    match play {
        CardPlay::Basic { discard, .. } => discard.is_none(),
        CardPlay::Sideways(_) => true,
        CardPlay::Strong {
            discard,
            set_dice,
            with,
            ..
        } => discard.is_none() && with.is_none() && set_dice.iter().all(Option::is_none),
        CardPlay::Boosted { .. } => false,
        // Spec 054 : un sort lancé seul.
        CardPlay::Cast { .. } => true,
        // Spec 055 : le parcours ne joue pas ses artefacts pour leur effet (il reste celui d'avant).
        CardPlay::Artifact { .. } | CardPlay::ThrowAway { .. } => false,
        // Spec 058 : les parcours ne jouent pas de côté avec une compétence.
        CardPlay::SidewaysSkill { .. } => false,
    }
}

/// Le coût d'un pas sur cette case, au moment de la journée de la vue.
pub fn step_cost(view: &PlayerView, hex: Hex) -> u32 {
    let space = view.map.space(hex).expect("case du trajet");
    match arcanum_core::move_cost(space, view.time_of_day) {
        // Le pas d'un assaut coûte le terrain du site (spec 043).
        arcanum_core::MoveCost::Points(points) | arcanum_core::MoveCost::Fortified(points) => {
            u32::from(points)
        }
        other => panic!("pas infranchissable sur le trajet : {other:?}"),
    }
}

/// Le trajet le moins cher vers `to`, qui évite les ennemis errants (spec 030, C19).
pub fn route_to(parcours: &Parcours, to: Hex) -> Vec<Hex> {
    let view = parcours.view();
    let from = view.figure.expect("la figurine est posée");
    if from == to {
        return Vec::new();
    }
    let enemies: Vec<Hex> = view.enemy_tokens.keys().copied().collect();
    routes_avoiding(&view.map, from, view.time_of_day, &enemies)
        .into_iter()
        .find(|route| route.to == to)
        .expect("la case est atteignable")
        .steps
}

pub fn move_amount(effect: Effect) -> Option<u8> {
    match effect {
        Effect::Move(amount) => Some(amount),
        _ => None,
    }
}

pub fn influence_amount(effect: Effect) -> Option<u8> {
    match effect {
        Effect::Influence(amount) => Some(amount),
        _ => None,
    }
}

/// L'unité recrutable au village, utile à l'assaut (une capacité d'Attaque), la moins chère.
pub fn unit_to_recruit(view: &PlayerView) -> Option<UnitCard> {
    view.offers[&Offer::Units]
        .iter()
        .filter_map(|card| card.unit())
        .filter(|unit| {
            let definition = unit.unit.definition();
            definition.recruit.contains(&RecruitSite::Village) && melee_attack(*unit).is_some()
        })
        .min_by_key(|unit| unit.unit.definition().influence)
}

/// La première option d'Attaque (corps à corps, sans mana) de l'unité : (capacité, option, valeur).
pub fn melee_attack(unit: UnitCard) -> Option<(u8, u8, u8)> {
    unit.unit
        .definition()
        .abilities
        .iter()
        .enumerate()
        .filter(|(_, ability)| ability.mana.is_none() && !ability.wounds_unit())
        .find_map(|(a, ability)| {
            ability
                .options()
                .iter()
                .enumerate()
                .find_map(|(o, option)| match option.effect {
                    Effect::Attack {
                        amount,
                        kind: AttackKind::Melee,
                        ..
                    } if option.extra_mana.is_empty() => Some((a as u8, o as u8, amount)),
                    _ => None,
                })
        })
}

/// Spec 059, Q7 (ADR 0006, « Départ d'un parcours ») : joue `run` sur chaque graine, dit sur une
/// ligne pourquoi chaque échec échoue, et rend les graines qui passent. Pour un test ignoré
/// `balayer_les_graines` du parcours.
pub fn sweep_seeds(seeds: std::ops::Range<u64>, run: impl Fn(u64)) -> Vec<u64> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        let line = info.location().map_or(0, |location| location.line());
        eprintln!(
            "échec ligne {line} : {}",
            message.lines().next().unwrap_or("")
        );
    }));
    let total = seeds.end - seeds.start;
    let mut passed = Vec::new();
    for seed in seeds {
        eprint!("graine {seed} : ");
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(seed)));
        if outcome.is_ok() {
            eprintln!("passe");
            passed.push(seed);
        }
    }
    std::panic::set_hook(previous);
    println!("{} graines sur {total} passent", passed.len());
    passed
}
