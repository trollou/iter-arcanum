//! Scénarios de la spec 040 (Interaction et réputation).

use arcanum_core::{
    Action, AttackKind, BasicAction, CardId, CardPlay, ContentError, ContentSources, Effect,
    Element, Enemy, EnemyToken, Event, GameConfig, GameContent, GameRunner, GameState, Hero, Hex,
    Interaction, Reputation, Scenario, SidewaysEffect, Site, TileId, TilePosition, apply,
    legal_actions, new_game, view,
};

use BasicAction::*;

mod arene;
use arene::*;

// --- Outils ---

fn with_reputation_file(text: &str) -> Result<GameContent, ContentError> {
    GameContent::parse(ContentSources {
        reputation: text,
        ..ContentSources::EMBEDDED
    })
}

/// L'état après ces changements de réputation, dans l'ordre.
fn reputation_after(state: GameState, changes: &[i8]) -> GameState {
    changes
        .iter()
        .map(|&change| Event::ReputationChanged {
            player: SOLO,
            change,
        })
        .fold(state, |state, event| apply(state, &event))
}

/// Les changements de réputation parmi ces événements.
fn reputation_changes(events: &[Event]) -> Vec<i8> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::ReputationChanged { change, .. } => Some(*change),
            _ => None,
        })
        .collect()
}

/// Case voisine des orques ② (3, −3) et du centre (4, −5) de la tuile royaume 4, posée en A + B.
const BETWEEN: Hex = Hex::new(3, -4);

/// Le draconien imprimé au centre de la tuile royaume 4 posée en A + B.
const DRACONUM: Hex = Hex::new(4, -5);

fn swamp_dragon() -> EnemyToken {
    token(Enemy::SwampDragon, 1)
}

/// Combat engagé contre ces ennemis défiés depuis `figure`, en phase d'attaque, avec la tuile
/// royaume 4 posée en A + B (son centre est un draconien imprimé).
fn attacking(figure: Hex, placed: &[(EnemyToken, Hex)], challenged: &[Hex]) -> GameState {
    let march = card(BasicAction::March, 1);
    let mut events = Sandbox {
        figure: Some(figure),
        hand: vec![march, card(BasicAction::Rage, 1)],
        placed: placed.to_vec(),
        next: vec![TileId::Core(4)],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::TileRevealed {
        tile: TileId::Core(4),
        position: TilePosition::from_lattice(1, 1),
    });
    let mut state = events.iter().fold(GameState::default(), apply);
    for &enemy in challenged {
        state = done(&state, challenge(enemy));
    }
    let state = done(&state, Action::StartCombat);
    skip_to_attack_phase(&state)
}

/// Une attaque de `amount` points physiques sur `targets`, par un événement : la valeur seule
/// compte ici, pas la carte.
fn attack_with(state: &GameState, targets: &[Hex], amount: u8) -> GameState {
    let targets: Vec<EnemyToken> = targets.iter().map(|&hex| foe(state, hex)).collect();
    let state = done(state, declare(&targets));
    apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: card(BasicAction::March, 1),
            play: CardPlay::Sideways(SidewaysEffect::Attack),
            effect: Effect::Attack {
                amount,
                element: Element::Physical,
                kind: AttackKind::Melee,
            },
        },
    )
}

fn legal(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
}

fn can_interact(state: &GameState) -> bool {
    legal(state).contains(&Action::Interact)
}

/// Les façons légales de jouer `card`.
fn plays_of(state: &GameState, card: CardId) -> Vec<CardPlay> {
    legal(state)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card: played, play } if played == card => Some(play),
            _ => None,
        })
        .collect()
}

fn sideways_influence(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Sideways(SidewaysEffect::Influence),
    }
}

/// Les gains de gloire parmi ces événements.
fn fame_gains(events: &[Event]) -> Vec<u32> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::FameGained { fame, .. } => Some(*fame),
            _ => None,
        })
        .collect()
}

fn threaten() -> CardId {
    card(Threaten, 1)
}

fn manners() -> CardId {
    card(NobleManners, 1)
}

/// La main de Norowas au village : Nobles manières, Menace, Promesse, Marche, Rage.
fn norowas_hand() -> [CardId; 5] {
    [
        manners(),
        threaten(),
        card(Promise, 1),
        card(March, 1),
        card(Rage, 1),
    ]
}

fn end_turn(discard: &[CardId]) -> Action {
    Action::EndTurn {
        discard: discard.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

/// Une main de tous les jours pour interagir : Marche, Promesse, Rage, Improvisation.
fn village_hand() -> [CardId; 4] {
    [
        card(March, 1),
        card(Promise, 1),
        card(Rage, 1),
        card(Improvisation, 1),
    ]
}

/// La figurine sur le village (2, −3), réputation `reputation`, la main `hand`, les Rôdeurs sur
/// les orques ② voisins.
fn at_village(hand: &[CardId], reputation: i8) -> GameState {
    reputation_after(arena(VILLAGE, hand, &[(prowlers(), ORCS_2)]), &[reputation])
}

// --- Piste de réputation ---

// C1 — MKUE p. 3 : le jeton de réputation part de la case centrale.
#[test]
fn reputation_starts_at_zero() {
    for scenario in [
        Scenario::sandbox(),
        Scenario::first_reconnaissance().without_dummy(),
    ] {
        let config = GameConfig {
            seed: 3,
            scenario,
            hero: Hero::Norowas,
            dummy_hero: Default::default(),
        };
        let state = new_game(&config, &mut config.rngs())
            .iter()
            .fold(GameState::default(), apply);
        assert_eq!(state.reputation(SOLO), Reputation::new(0), "{scenario:?}");
        assert_eq!(state.reputation_bonus(SOLO), Some(0), "{scenario:?}");
    }
}

// C2 — scan misc/fame_board.jpg, de −7 à +7.
#[test]
fn reputation_track_matches_the_board() {
    let expected = [
        None,
        Some(-5),
        Some(-3),
        Some(-2),
        Some(-1),
        Some(-1),
        Some(0),
        Some(0),
        Some(0),
        Some(1),
        Some(1),
        Some(2),
        Some(2),
        Some(3),
        Some(5),
    ];
    let track = GameContent::embedded().reputation_track();
    assert_eq!(track.spaces(), expected);
    for (value, bonus) in (-7..=7).zip(expected) {
        assert_eq!(track.bonus(Reputation::new(value)), bonus, "{value}");
    }
    assert!(Reputation::new(-7).is_x());
    assert!(!Reputation::new(-6).is_x());
}

// C2 — la piste est validée au chargement : quinze cases, la case X au bas et elle seule.
#[test]
fn reputation_track_is_validated() {
    let fourteen = "[X, Bonus(-5), Bonus(-3), Bonus(-2), Bonus(-1), Bonus(-1), Bonus(0), Bonus(0), \
                    Bonus(0), Bonus(1), Bonus(1), Bonus(2), Bonus(2), Bonus(3)]";
    let no_x = "[Bonus(-7), Bonus(-5), Bonus(-3), Bonus(-2), Bonus(-1), Bonus(-1), Bonus(0), \
                Bonus(0), Bonus(0), Bonus(1), Bonus(1), Bonus(2), Bonus(2), Bonus(3), Bonus(5)]";
    let x_on_top = "[X, Bonus(-5), Bonus(-3), Bonus(-2), Bonus(-1), Bonus(-1), Bonus(0), Bonus(0), \
                    Bonus(0), Bonus(1), Bonus(1), Bonus(2), Bonus(2), Bonus(3), X]";
    for text in [fourteen, no_x, x_on_top] {
        assert!(
            matches!(
                with_reputation_file(text),
                Err(ContentError::InvalidReputation { .. })
            ),
            "{text}"
        );
    }
}

// C3 — Compendium, *Influence* S5 : la réputation ne sort pas de la piste.
#[test]
fn reputation_stays_on_the_track() {
    let state = arena(VILLAGE, &[], &[]);
    let top = reputation_after(state.clone(), &[7]);
    assert_eq!(top.reputation(SOLO), Reputation::new(7));
    assert_eq!(
        reputation_after(top, &[2]).reputation(SOLO),
        Reputation::new(7)
    );
    let bottom = reputation_after(state, &[-7]);
    assert!(bottom.reputation(SOLO).is_x());
    assert_eq!(
        reputation_after(bottom, &[-1]).reputation(SOLO),
        Reputation::new(-7)
    );
    assert_eq!(Reputation::new(9), Reputation::new(7));
    assert_eq!(Reputation::new(-9), Reputation::new(-7));
}

// C25 — la vue donne la réputation et son bonus (tout est public).
#[test]
fn view_shows_reputation() {
    let state = reputation_after(arena(VILLAGE, &[], &[]), &[2]);
    let seen = view(&state, SOLO);
    assert_eq!(seen.reputation, Reputation::new(2));
    assert_eq!(seen.reputation_bonus, Some(1));
    let x = reputation_after(state, &[-9]);
    assert_eq!(view(&x, SOLO).reputation_bonus, None);
}

// --- Ennemis errants vaincus ---

// C4 — MKUE p. 10, *Combat Outcomes* 2.a : +1 par orques maraudeurs, avec leur gloire.
#[test]
fn defeating_orc_marauders_gains_one_reputation() {
    let state = attacking(BETWEEN, &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let state = attack_with(&state, &[ORCS_2], 3);
    let events = events_of(&state, &Action::ResolveAttack);
    assert!(matches!(events[0], Event::EnemiesDefeated { .. }));
    assert_eq!(
        events.get(1),
        Some(&Event::ReputationChanged {
            player: SOLO,
            change: 1
        })
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.reputation(SOLO), Reputation::new(1));
}

// C5 — MKUE p. 10, 2.a : +2 par draconien ; une seule attaque, un seul changement.
#[test]
fn defeating_draconum_gains_two_reputation() {
    let placed = [(swamp_dragon(), DRACONUM), (prowlers(), ORCS_2)];
    let state = attacking(BETWEEN, &placed, &[DRACONUM]);
    let dragon = attack_with(&state, &[DRACONUM], 9);
    let events = events_of(&dragon, &Action::ResolveAttack);
    assert_eq!(reputation_changes(&events), vec![2]);

    let state = attacking(BETWEEN, &placed, &[DRACONUM, ORCS_2]);
    let both = attack_with(&state, &[DRACONUM, ORCS_2], 12);
    let events = events_of(&both, &Action::ResolveAttack);
    assert_eq!(reputation_changes(&events), vec![3]);
}

// C6 — une attaque ratée, ou un combat fini sans victoire : rien.
#[test]
fn no_reputation_without_defeat() {
    let state = attacking(BETWEEN, &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let failed = attack_with(&state, &[ORCS_2], 2);
    let events = events_of(&failed, &Action::ResolveAttack);
    assert!(matches!(events[..], [Event::AttackFailed { .. }]));
    let after = events.iter().fold(failed, apply);
    let events = events_of(&after, &Action::EndCombat);
    assert!(reputation_changes(&events).is_empty());
}

// C7 — Q2 : la case imprimée compte, pas la couleur du jeton (pile de secours, spec 033).
#[test]
fn reputation_follows_the_printed_space() {
    let state = attacking(BETWEEN, &[(swamp_dragon(), ORCS_2)], &[ORCS_2]);
    let state = attack_with(&state, &[ORCS_2], 9);
    let events = events_of(&state, &Action::ResolveAttack);
    assert_eq!(reputation_changes(&events), vec![1]);
}

// --- Interagir ---

// C8 — MKUE p. 8, 1 : sur un village ou un monastère seulement.
#[test]
fn interact_only_on_a_village_or_monastery() {
    assert!(can_interact(&at_village(&village_hand(), 0)));
    let plain = arena(PLAIN_W, &village_hand(), &[]);
    assert!(!can_interact(&plain));
    refused(&plain, Action::Interact);
}

// C9 — MKUE p. 6, 6.e ; p. 8, 2.a : mouvement perdu, puis le bonus. Variante maison (issue #95) :
// l'influence jouée avant l'interaction est gardée et le bonus s'y ajoute.
#[test]
fn interacting_keeps_unspent_influence_and_adds_the_bonus() {
    let [march, promise, rage, _] = village_hand();
    let state = at_village(&village_hand(), 2);
    let state = done(&state, basic(march));
    let state = done(&state, sideways(rage));
    let state = done(&state, basic(promise));
    assert_eq!(state.move_points(SOLO), 3);
    assert_eq!(state.influence_points(SOLO), 2);
    let events = events_of(&state, &Action::Interact);
    assert_eq!(
        events,
        vec![Event::InteractionStarted {
            player: SOLO,
            site: Site::Village,
            bonus: 1,
            city_shields: 0,
        }]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.move_points(SOLO), 0);
    assert_eq!(state.influence_points(SOLO), 3);
    assert_eq!(
        state.interaction(SOLO),
        Some(Interaction {
            site: Site::Village,
            bonus: 1,
            city_shields: 0,
        })
    );
}

// C10 — Compendium, *Influence* S1 : un malus donne une influence négative.
#[test]
fn a_penalty_gives_negative_influence() {
    let [_, promise, ..] = village_hand();
    let state = done(&at_village(&village_hand(), -4), Action::Interact);
    assert_eq!(state.influence_points(SOLO), -2);
    let state = done(&state, basic(promise));
    assert_eq!(state.influence_points(SOLO), 0);
}

// C11 — MKUE p. 8, 2.a et 5.a : le bonus est compté une fois, au début.
#[test]
fn the_bonus_is_applied_once() {
    let state = done(&at_village(&village_hand(), 2), Action::Interact);
    // Réputation +5 : bonus +2 désormais.
    let state = reputation_after(state, &[3]);
    assert_eq!(state.reputation_bonus(SOLO), Some(2));
    assert_eq!(state.influence_points(SOLO), 1);
    assert_eq!(state.interaction(SOLO).map(|i| i.bonus), Some(1));
}

// C13 — MKUE p. 6, 6.d et 7 : l'action du tour, sans repos.
#[test]
fn interacting_is_the_action_of_the_turn() {
    let state = at_village(&village_hand(), 0);
    assert!(legal(&state).contains(&challenge(ORCS_2)));
    assert!(state.can_rest(SOLO));
    let state = done(&state, Action::Interact);
    // Des points de mouvement gagnés après coup ne servent plus à rien.
    let state = apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: card(March, 1),
            play: CardPlay::Basic {
                die: None,
                gain: None,
                with: None,
                option: 0,
                discard: None,
                mana: None,
            },
            effect: Effect::Move(4),
        },
    );
    let actions = legal(&state);
    assert!(
        !actions.iter().any(|action| matches!(
            action,
            Action::MoveHero { .. }
                | Action::FollowRoute { .. }
                | Action::Explore { .. }
                | Action::Challenge { .. }
                | Action::AnnounceEndOfRound
                | Action::Interact
        )),
        "{actions:?}"
    );
    assert!(!state.can_rest(SOLO));
    assert!(!view(&state, SOLO).can_rest);
    // Repos standard : une Blessure et une autre carte, refusé.
    let [_, promise, ..] = village_hand();
    refused(&state, end_turn(&[promise]).with_wound_discard());
}

trait WithWound {
    fn with_wound_discard(self) -> Action;
}

impl WithWound for Action {
    fn with_wound_discard(self) -> Action {
        match self {
            Action::EndTurn {
                mut discard,
                reclaim,
                ..
            } => {
                discard.push(wound(1));
                Action::EndTurn {
                    discard,
                    reclaim,
                    glade: None,
                    steady_tempo: false,
                }
            }
            other => other,
        }
    }
}

// C14 — MKUE p. 8, 2.b à 2.e : influence, mana et soin ; plus de mouvement.
#[test]
fn interaction_offers_influence_mana_and_healing() {
    let (march, improvisation) = (card(March, 1), card(Improvisation, 1));
    let (tranquility, concentration) = (card(Tranquility, 1), card(Concentration, 1));
    let hand = [march, improvisation, tranquility, concentration, wound(1)];
    let state = done(&at_village(&hand, 0), Action::Interact);
    assert_eq!(
        plays_of(&state, march),
        vec![CardPlay::Sideways(SidewaysEffect::Influence)]
    );
    let improvisation_effects: Vec<Effect> = plays_of(&state, improvisation)
        .into_iter()
        .filter(|play| !matches!(play, CardPlay::Sideways(_)))
        .filter_map(|play| improvisation.effect(play))
        .collect();
    assert!(!improvisation_effects.is_empty());
    assert!(
        improvisation_effects
            .iter()
            .all(|effect| matches!(effect, Effect::Influence(3) | Effect::Influence(5))),
        "{improvisation_effects:?}"
    );
    assert!(plays_of(&state, tranquility).contains(&CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option: 0,
        discard: None,
        mana: None
    }));
    assert!(plays_of(&state, concentration).contains(&CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option: 0,
        discard: None,
        mana: None
    }));
}

// C15 — Compendium, *Influence* S7 : défier ou interagir, pas les deux.
#[test]
fn challenging_forbids_interacting() {
    let state = at_village(&village_hand(), 0);
    assert!(can_interact(&state));
    assert!(legal(&state).contains(&challenge(ORCS_2)));
    let state = done(&state, challenge(ORCS_2));
    assert!(!can_interact(&state));
    refused(&state, Action::Interact);
}

// C16 — MKUE p. 6, 6.b : la provocation impose le combat.
#[test]
fn a_provocation_forbids_interacting() {
    let [march, ..] = village_hand();
    let state = arena(PLAIN_W, &village_hand(), &[(prowlers(), ORCS_2)]);
    let state = done(&state, basic(march));
    let events = events_of(&state, &Action::MoveHero { to: VILLAGE });
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert!(!can_interact(&state));
}

// C17 — MKUE p. 8, 2.a : rien sur la case X.
#[test]
fn no_interaction_on_x() {
    let state = at_village(&village_hand(), -7);
    assert!(!can_interact(&state));
    refused(&state, Action::Interact);
}

// C18 — spec 024 : revenir en arrière juste après avoir interagi.
#[test]
fn interacting_can_be_undone() {
    let [march, promise, ..] = village_hand();
    let mut events = Sandbox {
        figure: Some(VILLAGE),
        hand: village_hand().to_vec(),
        ..Sandbox::default()
    }
    .events();
    events.push(Event::ReputationChanged {
        player: SOLO,
        change: 2,
    });
    let (mut runner, puppet) = runner(events);
    play(&mut runner, &puppet, basic(march));
    play(&mut runner, &puppet, basic(promise));
    play(&mut runner, &puppet, Action::Interact);
    assert_eq!(runner.state().influence_points(SOLO), 1);
    assert_eq!(runner.undo(), Ok(()));
    let state = runner.state();
    assert_eq!(state.move_points(SOLO), 2);
    assert_eq!(state.influence_points(SOLO), 2);
    assert_eq!(state.interaction(SOLO), None);
}

/// Au village avec la main `village_hand()`, réputation +2.
fn village_runner() -> (GameRunner, Puppet) {
    let mut events = Sandbox {
        figure: Some(VILLAGE),
        hand: village_hand().to_vec(),
        ..Sandbox::default()
    }
    .events();
    events.push(Event::ReputationChanged {
        player: SOLO,
        change: 2,
    });
    runner(events)
}

// Issue privée 307 — MKUE p. 7, 1 et p. 5, Reverting : fermer le menu d'interaction sans rien y avoir
// fait reprend le début de l'interaction, et le héros peut encore se déplacer.
#[test]
fn untouched_interaction_can_be_cancelled() {
    let [march, ..] = village_hand();
    let (mut runner, puppet) = village_runner();
    play(&mut runner, &puppet, basic(march));
    play(&mut runner, &puppet, Action::Interact);
    assert!(runner.cancel_untouched_interaction());
    let state = runner.state();
    assert_eq!(state.interaction(SOLO), None);
    assert_eq!(state.move_points(SOLO), 2);
    assert!(
        legal_actions(state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::MoveHero { .. })),
        "le héros peut de nouveau se déplacer"
    );
    // La carte jouée avant l'interaction le reste.
    assert!(!state.hand(SOLO).contains(&march));
}

// Issue privée 307 : une fois de l'influence jouée dans l'interaction, elle n'est plus reprise ainsi.
#[test]
fn interaction_with_a_card_played_is_not_cancelled() {
    let [_, promise, ..] = village_hand();
    let (mut runner, puppet) = village_runner();
    play(&mut runner, &puppet, Action::Interact);
    play(&mut runner, &puppet, basic(promise));
    assert!(!runner.cancel_untouched_interaction());
    assert!(runner.state().interaction(SOLO).is_some());
}

// Issue privée 307 : sans interaction en cours, rien n'est repris.
#[test]
fn no_interaction_nothing_cancelled() {
    let [march, ..] = village_hand();
    let (mut runner, puppet) = village_runner();
    play(&mut runner, &puppet, basic(march));
    assert!(!runner.cancel_untouched_interaction());
    assert_eq!(runner.state().move_points(SOLO), 2);
}

// C19 — MKUE p. 6, 6.d : l'interaction finit avec le tour.
#[test]
fn interaction_ends_with_the_turn() {
    let [_, promise, ..] = village_hand();
    let state = done(&at_village(&village_hand(), 2), Action::Interact);
    let state = done(&state, basic(promise));
    let state = done(&state, end_turn(&[]));
    assert_eq!(state.influence_points(SOLO), 0);
    assert_eq!(state.interaction(SOLO), None);
    assert!(can_interact(&state));
}

// L1 — refusée par `decide` hors des actions légales, état inchangé.
#[test]
fn interact_is_refused_when_not_legal() {
    let twice = done(&at_village(&village_hand(), 0), Action::Interact);
    refused(&twice, Action::Interact);
}

// L2 — MKUE p. 6, 5.a : sans carte jouée, la fin du tour défausse une carte, sans repos.
#[test]
fn interacting_without_a_card_still_needs_a_discard() {
    let [march, ..] = village_hand();
    let state = done(&at_village(&village_hand(), 0), Action::Interact);
    let actions = legal(&state);
    assert!(!actions.contains(&end_turn(&[])));
    assert!(actions.contains(&end_turn(&[march])));
}

// L2 — Compendium, *Resting* S4 : une main de Blessures sans carte jouée ne peut que se reposer.
#[test]
fn a_hand_of_wounds_cannot_interact() {
    let state = at_village(&[wound(1), wound(2)], 0);
    assert!(!can_interact(&state));
}

/// Le village, deck vide au début du tour : la main seule.
fn with_empty_deck(hand: &[CardId]) -> GameState {
    let state = at_village(hand, 0);
    let mut events = vec![Event::DeckShuffled {
        player: SOLO,
        order: hand.to_vec(),
    }];
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.iter().fold(state, apply)
}

// L3 — MKUE p. 6, 3 : deck vide, annoncer ou interagir ; après l'interaction, plus d'annonce.
#[test]
fn empty_deck_announce_or_interact() {
    let state = with_empty_deck(&village_hand());
    assert!(state.deck(SOLO).is_empty());
    assert!(legal(&state).contains(&Action::AnnounceEndOfRound));
    assert!(can_interact(&state));
    let state = done(&state, Action::Interact);
    assert!(!legal(&state).contains(&Action::AnnounceEndOfRound));
    // Main et deck vides : l'annonce est obligatoire, pas d'interaction (MKUE p. 6, 3.b).
    assert!(!can_interact(&with_empty_deck(&[])));
}

// L7 — une réputation qui sort de X rend l'interaction possible au tour suivant.
#[test]
fn leaving_x_allows_interaction_next_turn() {
    let swiftness = card(Swiftness, 1);
    let hand = [swiftness, card(March, 1), card(Rage, 1)];
    let state = at_village(&hand, -7);
    assert!(!can_interact(&state));
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    let state = done(&state, Action::EndCombat);
    assert_eq!(state.reputation(SOLO), Reputation::new(-6));
    assert!(!can_interact(&state));
    let state = done(&state, end_turn(&[]));
    assert!(can_interact(&state));
}

// --- Menace et Nobles manières ---

// C20 — cartes/actions-de-base.md : Menace renforcée, Influence 5 et Réputation −1, aussitôt.
#[test]
fn threaten_strong_loses_reputation() {
    let state = arena(PLAIN_W, &norowas_hand(), &[]);
    let events = events_of(&state, &strong(threaten(), RED_DIE));
    assert_eq!(
        events.last(),
        Some(&Event::ReputationChanged {
            player: SOLO,
            change: -1
        })
    );
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(after.influence_points(SOLO), 5);
    assert_eq!(after.reputation(SOLO), Reputation::new(-1));
    let events = events_of(&state, &basic(threaten()));
    assert!(reputation_changes(&events).is_empty());
    assert_eq!(events.iter().fold(state, apply).influence_points(SOLO), 2);
}

// C21 — FAQ p1 : Nobles manières en interaction rapporte sa gloire, renforcée sa réputation.
#[test]
fn noble_manners_rewards_interaction() {
    let state = done(&at_village(&norowas_hand(), 0), Action::Interact);
    let events = events_of(&state, &basic(manners()));
    assert_eq!(fame_gains(&events), vec![1]);
    assert!(reputation_changes(&events).is_empty());
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(after.influence_points(SOLO), 2);
    assert_eq!(after.fame(SOLO), 1);

    let events = events_of(&state, &strong(manners(), WHITE_DIE));
    assert_eq!(fame_gains(&events), vec![1]);
    assert_eq!(reputation_changes(&events), vec![1]);
    let after = events.iter().fold(state, apply);
    assert_eq!(after.influence_points(SOLO), 4);
    assert_eq!(after.reputation(SOLO), Reputation::new(1));
}

// C22 — Compendium, *Noble Manners* S2 et S3 ; FAQ p2 : sans interaction, ou de côté, rien de plus.
#[test]
fn noble_manners_needs_interaction() {
    let before = at_village(&norowas_hand(), 0);
    let events = events_of(&before, &strong(manners(), WHITE_DIE));
    assert!(fame_gains(&events).is_empty());
    assert!(reputation_changes(&events).is_empty());
    let away = arena(PLAIN_W, &norowas_hand(), &[]);
    assert!(fame_gains(&events_of(&away, &basic(manners()))).is_empty());
    let state = done(&before, Action::Interact);
    let events = events_of(&state, &sideways_influence(manners()));
    assert!(fame_gains(&events).is_empty());
    assert_eq!(events.iter().fold(state, apply).influence_points(SOLO), 1);
}

// C23 — spec 035 : la gloire de Nobles manières compte pour le niveau de la fin du tour.
#[test]
fn noble_manners_fame_counts_for_level() {
    let state = reputation_after(arena_at(VILLAGE, &norowas_hand(), &[], 2, 1), &[0]);
    let state = done(&state, Action::Interact);
    let state = done(&state, basic(manners()));
    let events = events_of(&state, &end_turn(&[]));
    assert!(
        events.contains(&Event::LevelGained {
            player: SOLO,
            level: 2
        }),
        "{events:?}"
    );
}

// C12 — Compendium, *Threaten* S1 : Menace avant d'interagir baisse le bonus ; variante maison
// (issue #95) : ses 5 d'influence restent, le malus s'y ajoute.
#[test]
fn threaten_before_interacting_lowers_the_bonus() {
    let state = at_village(&norowas_hand(), -1);
    assert_eq!(state.reputation_bonus(SOLO), Some(0));
    let state = done(&state, strong(threaten(), RED_DIE));
    assert_eq!(state.reputation(SOLO), Reputation::new(-2));
    let state = done(&state, Action::Interact);
    assert_eq!(state.interaction(SOLO).map(|i| i.bonus), Some(-1));
    assert_eq!(state.influence_points(SOLO), 4);
}

// L4 — spec 063 : au dernier tour, l'interaction est permise et sa gloire compte au décompte.
#[test]
fn last_turn_interaction_counts_at_scoring() {
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: Some(VILLAGE),
        hand: norowas_hand().to_vec(),
        ..Sandbox::default()
    }
    .events();
    events.extend([
        Event::LastTurnAnnounced { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    ]);
    let state = events.iter().fold(GameState::default(), apply);
    assert!(can_interact(&state));
    let state = done(&state, Action::Interact);
    let state = done(&state, basic(manners()));
    let state = done(&state, end_turn(&[]));
    assert!(state.game_end().is_some());
    assert_eq!(arcanum_core::final_score(&state, SOLO).fame, 1);
}

// L5 — réputation +7 : la gloire, la réputation reste à +7.
#[test]
fn noble_manners_at_the_top_of_the_track() {
    let state = done(&at_village(&norowas_hand(), 7), Action::Interact);
    let state = done(&state, strong(manners(), WHITE_DIE));
    assert_eq!(state.fame(SOLO), 1);
    assert_eq!(state.reputation(SOLO), Reputation::new(7));
}

// L6 — Concentration renforcée : Influence 4 + 2, gloire et réputation une seule fois.
#[test]
fn concentration_on_noble_manners() {
    let concentration = card(Concentration, 1);
    let hand = [manners(), concentration, card(March, 1)];
    let state = done(&at_village(&hand, 0), Action::Interact);
    let boosted = legal(&state)
        .into_iter()
        .find(|action| {
            matches!(action, Action::PlayCard { card, play: CardPlay::Strong { with: Some(target), .. } }
                if *card == concentration && target.card == manners())
        })
        .expect("Concentration renforcée sur Nobles manières");
    let events = events_of(&state, &boosted);
    assert_eq!(fame_gains(&events), vec![1]);
    assert_eq!(reputation_changes(&events), vec![1]);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.influence_points(SOLO), 6);
}
