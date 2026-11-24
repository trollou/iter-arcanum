//! Scénarios de la spec 045 (Ruines).

use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, CardId, CardPlay, CombatPhase, ContentError,
    ContentSources, Effect, Element, EnemyColor, EnemyToken, Event, Feature, GameContent,
    GameState, Hero, Hex, ManaColor, ManaPayment, PlayerEvent, RngUsage, RuinKind, RuinReward,
    RuinToken, SidewaysEffect, Site, TileId, TimeOfDay, UndoStatus, Unit, UnitCard, apply,
    filter_event, legal_actions, new_game, view,
};

use BasicAction::*;

mod arene;
use arene::*;

mod premier_choix;
use premier_choix::FirstChoice;

// --- Jetons de ruines (C1) ---

// C1 — MKUE p. 3, *Enemy and Ruin token piles* ; `../mage-knight/docs/regles/cartes/ennemis.md`, *Jetons de
// ruines* : 12 jetons, au moins autant que de ruines sur toutes les tuiles.
#[test]
fn ruin_tokens_match_the_content() {
    use EnemyColor::*;
    use RuinReward::*;
    let content = GameContent::embedded();
    let tokens = content.ruin_tokens();
    assert_eq!(tokens, RuinToken::ALL);
    let kinds: Vec<RuinKind> = tokens
        .iter()
        .map(|&token| content.ruin_token(token).kind.clone())
        .collect();
    let enemies = |colors: &[EnemyColor], rewards: &[RuinReward]| RuinKind::Enemies {
        colors: colors.to_vec(),
        rewards: rewards.to_vec(),
    };
    assert_eq!(
        kinds,
        [
            RuinKind::Altar(BasicColor::Green),
            RuinKind::Altar(BasicColor::Blue),
            RuinKind::Altar(BasicColor::White),
            RuinKind::Altar(BasicColor::Red),
            enemies(&[Brown, Red], &[Artifact, Artifact]),
            enemies(&[Green, Red], &[Artifact, AdvancedAction]),
            enemies(&[Grey, White], &[Artifact, Spell]),
            enemies(&[Grey, Brown], &[Artifact]),
            enemies(&[Grey, Violet], &[Unit]),
            enemies(&[Green, Brown], &[Artifact]),
            enemies(&[Green, Green], &[CrystalSet]),
            enemies(&[Brown, Violet], &[Spell, CrystalSet]),
        ]
    );
    assert_eq!(tokens.iter().filter(|token| token.is_altar()).count(), 4);
    // Plus de jetons que de ruines : la pile ne se vide jamais, pas de remélange à coder.
    let ruins = content
        .tiles()
        .iter()
        .flat_map(|tile| std::iter::once(tile.center).chain(tile.sides))
        .filter(|space| space.feature == Some(Feature::Site(Site::Ruins)))
        .count();
    assert_eq!(ruins, 7);
    assert!(tokens.len() >= ruins);
}

// C1 — ADR 0005 : le fichier des jetons de ruines est validé (jeton manquant, en double).
#[test]
fn ruin_content_is_validated() {
    let parse = |ruins: &str| {
        GameContent::parse(ContentSources {
            ruins,
            ..ContentSources::EMBEDDED
        })
    };
    let embedded = ContentSources::EMBEDDED.ruins;
    let without_red_altar = embedded.replace("(id: RedAltar, kind: Altar(Red)),", "");
    assert_eq!(
        parse(&without_red_altar).err(),
        Some(ContentError::Missing {
            file: "ruins.ron",
            id: "RedAltar".into()
        })
    );
    let twice = embedded.replace(
        "(id: RedAltar, kind: Altar(Red)),",
        "(id: RedAltar, kind: Altar(Red)), (id: RedAltar, kind: Altar(Red)),",
    );
    assert_eq!(
        parse(&twice).err(),
        Some(ContentError::Duplicate {
            file: "ruins.ron",
            id: "RedAltar".into()
        })
    );
}

// --- Pile et pose (C2 à C4) ---

/// La pile des jetons de ruines de la mise en place de cette graine.
fn setup_ruin_pile(seed: u64) -> Vec<RuinToken> {
    let config = config(seed);
    let events = new_game(&config, &mut config.rngs());
    let piles: Vec<Vec<RuinToken>> = events
        .iter()
        .filter_map(|event| match event {
            Event::RuinPileShuffled { order } => Some(order.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(piles.len(), 1, "un seul mélange de la pile des ruines");
    piles[0].clone()
}

// C2 — MKUE p. 3 ; ADR 0007 : la pile des jetons de ruines, mélangée avec son propre générateur.
#[test]
fn ruin_pile_is_shuffled_at_setup() {
    let first = setup_ruin_pile(0);
    let mut sorted = first.clone();
    sorted.sort();
    assert_eq!(sorted, RuinToken::ALL);
    assert_ne!(first, setup_ruin_pile(1));
    // Un usage à part, au numéro de flux figé : les autres tirages ne bougent pas.
    assert_eq!(RngUsage::RuinPile.stream(), 4);
    assert_eq!(RngUsage::RuinPile.to_string(), "ruin_pile");
    assert_eq!("ruin_pile".parse(), Ok(RngUsage::RuinPile));
    let config = config(0);
    let mut rngs = config.rngs();
    new_game(&config, &mut rngs);
    assert!(rngs.positions().contains_key(&RngUsage::RuinPile));
}

/// Explorer l'emplacement 2A (campagne 8, ruines) depuis la colline (4, −2) après une Marche, avec
/// `BrownViolet` sur le dessus de la pile des jetons de ruines.
fn explore_ruins(time: TimeOfDay) -> (Vec<Event>, GameState) {
    let state = Sandbox {
        figure: Some(Hex::new(4, -2)),
        hand: vec![card(March, 1)],
        next: vec![TileId::Countryside(8)],
        time,
        ruins: vec![RuinToken::BrownViolet],
        ..Sandbox::default()
    }
    .state();
    let state = done(&state, basic(card(March, 1)));
    let events = events_of(
        &state,
        &Action::Explore {
            position: RUINS_POSITION,
        },
    );
    let after = events.iter().fold(state, apply);
    (events, after)
}

// C3 — Carte *Ancient Ruins*, *When Revealed* : de jour, le jeton du dessus face visible.
#[test]
fn ruin_token_placed_face_up_by_day() {
    let (events, state) = explore_ruins(TimeOfDay::Day);
    assert_events_in_order(
        &events,
        &[
            Event::TileRevealed {
                tile: TileId::Countryside(8),
                position: RUINS_POSITION,
            },
            Event::RuinTokenPlaced {
                token: RuinToken::BrownViolet,
                hex: RUINS,
                face_up: true,
            },
        ],
    );
    assert_eq!(
        state.map().space(RUINS).and_then(|space| space.feature),
        Some(Feature::Site(Site::Ruins))
    );
    let placed = state.ruins().get(&RUINS).expect("un jeton sur la ruine");
    assert_eq!(
        (placed.token, placed.face_up),
        (RuinToken::BrownViolet, true)
    );
    assert!(!state.ruin_pile().contains(&RuinToken::BrownViolet));
    assert_eq!(state.ruin_pile().len(), 11);
    let seen = view(&state, SOLO);
    assert_eq!(seen.ruins.get(&RUINS), Some(&Some(RuinToken::BrownViolet)));
    assert_eq!(seen.ruin_pile, 11);
}

// C4 — Carte *Ancient Ruins* : de nuit, face cachée ; la vue ne montre qu'une ruine face cachée.
#[test]
fn ruin_token_placed_face_down_at_night() {
    let (events, state) = explore_ruins(TimeOfDay::Night);
    let placed = Event::RuinTokenPlaced {
        token: RuinToken::BrownViolet,
        hex: RUINS,
        face_up: false,
    };
    assert_events_in_order(&events, std::slice::from_ref(&placed));
    assert_eq!(
        filter_event(&placed, SOLO),
        PlayerEvent::RuinTokenPlaced {
            token: None,
            hex: RUINS,
            face_up: false,
        }
    );
    assert!(
        state
            .ruins()
            .get(&RUINS)
            .is_some_and(|placed| !placed.face_up)
    );
    assert_eq!(view(&state, SOLO).ruins.get(&RUINS), Some(&None));
}

// --- Révélation (C5 à C9, L5) ---

/// Une carte de la main jouée de côté pour `effect`, par un événement : seule la valeur compte ici.
fn played(state: &GameState, card: CardId, effect: Effect) -> GameState {
    let sideways = match effect {
        Effect::Move(_) => SidewaysEffect::Move,
        Effect::Block { .. } => SidewaysEffect::Block,
        _ => SidewaysEffect::Attack,
    };
    apply(
        state.clone(),
        &Event::CardPlayed {
            player: SOLO,
            card,
            play: CardPlay::Sideways(sideways),
            effect,
        },
    )
}

/// La ruine de nuit, jeton gris + violet face cachée, la figurine sur `figure` avec `points` de
/// Mouvement (une Marche jouée).
fn night_ruins(figure: Hex, points: u8) -> GameState {
    let state = Ruins {
        figure,
        time: TimeOfDay::Night,
        token: RuinToken::GreyViolet,
        face_up: false,
        ..Ruins::default()
    }
    .state();
    played(&state, card(March, 1), Effect::Move(points))
}

fn revealed() -> Event {
    Event::RuinTokenRevealed {
        hex: RUINS,
        token: RuinToken::GreyViolet,
    }
}

fn is_ruin_reveal(event: &Event) -> bool {
    matches!(event, Event::RuinTokenRevealed { .. })
}

// C5 — Compendium, *Movement* S8 : de nuit, un pas sur la ruine retourne son jeton.
#[test]
fn stepping_on_ruins_reveals_the_token() {
    let state = night_ruins(NEAR_RUINS, 10);
    let events = events_of(&state, &Action::MoveHero { to: RUINS });
    assert!(matches!(events[0], Event::HeroMoved { to: RUINS, .. }));
    assert_eq!(events[1], revealed());
    let after = events.iter().fold(state.clone(), apply);
    assert!(
        after
            .ruins()
            .get(&RUINS)
            .is_some_and(|placed| placed.face_up)
    );
    assert_eq!(
        view(&after, SOLO).ruins.get(&RUINS),
        Some(&Some(RuinToken::GreyViolet))
    );
    // Un pas sur une case voisine de la ruine ne la retourne pas.
    let beside = events_of(
        &state,
        &Action::MoveHero {
            to: Hex::new(6, -4),
        },
    );
    assert!(!beside.iter().any(is_ruin_reveal));
}

// C6 — Compendium, *Movement* S13 ; Q2 : un trajet qui retourne le jeton s'arrête sur la ruine,
// les points restants gardés.
#[test]
fn route_stops_on_revealed_ruins() {
    let state = night_ruins(RUINS_GLADE, 10);
    let events = events_of(&state, &Action::FollowRoute { to: NEAR_RUINS });
    assert_eq!(events.last(), Some(&revealed()));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::HeroMoved { to, .. } if *to == NEAR_RUINS))
    );
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(after.figure(SOLO), Some(RUINS));
    let step = match events[0] {
        Event::HeroMoved { cost, .. } => u32::from(cost),
        _ => panic!("un pas d'abord"),
    };
    assert_eq!(after.move_points(SOLO), 10 - step);
    assert!(!after.movement_ended(SOLO));
}

// C7, L5 — Compendium, *End of Round* S6 : une manche de Jour retourne tous les jetons face
// cachée, figurine dessus comprise ; une manche de Nuit, rien.
#[test]
fn day_round_reveals_ruin_tokens() {
    let deck = Hero::Arythea.starting_deck();
    let at_night = Ruins {
        figure: RUINS,
        hand: deck.clone(),
        time: TimeOfDay::Night,
        token: RuinToken::GreyViolet,
        face_up: false,
        ..Ruins::default()
    }
    .state();
    let events = events_of(&at_night, &Action::AnnounceEndOfRound);
    assert_events_in_order(
        &events,
        &[
            Event::RoundStarted {
                round: 3,
                time_of_day: TimeOfDay::Day,
            },
            revealed(),
        ],
    );
    let by_day = Ruins {
        hand: deck,
        token: RuinToken::GreyViolet,
        face_up: false,
        ..Ruins::default()
    }
    .state();
    let events = events_of(&by_day, &Action::AnnounceEndOfRound);
    assert!(!events.iter().any(is_ruin_reveal));
}

// C8 — Spec 024 : on ne revient pas en arrière avant un jeton de ruines retourné.
#[test]
fn ruin_reveal_blocks_undo() {
    assert!(revealed().blocks_undo());
    let (mut runner, puppet) = runner(
        Ruins {
            figure: NEAR_RUINS,
            time: TimeOfDay::Night,
            token: RuinToken::GreyViolet,
            face_up: false,
            ..Ruins::default()
        }
        .events(),
    );
    play(&mut runner, &puppet, basic(card(March, 1)));
    play(&mut runner, &puppet, basic(card(Swiftness, 1)));
    play(
        &mut runner,
        &puppet,
        Action::PlayCard {
            card: card(Rage, 1),
            play: CardPlay::Sideways(SidewaysEffect::Move),
        },
    );
    play(&mut runner, &puppet, Action::MoveHero { to: RUINS });
    assert_eq!(runner.undo_status(), UndoStatus::InformationRevealed);
}

// C9 — FAQ p3 ; MKUE p. 7, 4.d : la ruine révélée n'arrête pas le déplacement et n'oblige à rien.
#[test]
fn ruins_can_be_ignored() {
    let state = night_ruins(NEAR_RUINS, 10);
    let state = done(&state, Action::MoveHero { to: RUINS });
    assert!(
        state
            .ruins()
            .get(&RUINS)
            .is_some_and(|placed| placed.face_up)
    );
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&Action::MoveHero { to: RUINS_GLADE }));
    assert!(
        legal
            .iter()
            .any(|action| matches!(action, Action::EndTurn { .. }))
    );
}

// --- Autel (C10 à C16, L3) ---

fn blue() -> ManaColor {
    ManaColor::Basic(BasicColor::Blue)
}

/// L'autel bleu, figurine dessus, à ce moment, avec ces dés dans la Source, `crystals` cristaux
/// bleus et ces jetons de mana.
fn altar(time: TimeOfDay, dice: &[ManaColor], crystals: u8, tokens: &[ManaColor]) -> GameState {
    let mut events = Ruins {
        time,
        dice: dice.to_vec(),
        ..Ruins::default()
    }
    .events();
    events.extend((0..crystals).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    events.extend(tokens.iter().map(|&color| Event::ManaTokenGained {
        player: SOLO,
        color,
    }));
    events.iter().fold(GameState::default(), apply)
}

/// Les paiements de l'offrande parmi les actions légales.
fn tributes(state: &GameState) -> Vec<Vec<ManaPayment>> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PayAltarTribute { mana } => Some(mana),
            _ => None,
        })
        .collect()
}

fn tribute(mana: &[ManaPayment]) -> Action {
    Action::PayAltarTribute {
        mana: mana.to_vec(),
    }
}

const BLUE_IN_SOURCE: u8 = 2;

// C10 — Carte *Ancient Ruins*, *Altar* : 3 mana bleus en offrande, 7 gloire, la ruine conquise.
#[test]
fn altar_tribute_gives_seven_fame() {
    let state = altar(TimeOfDay::Day, &Sandbox::default().dice, 1, &[blue()]);
    let state = done(&state, basic(card(March, 1)));
    let paid = [
        ManaPayment::Die(BLUE_IN_SOURCE),
        ManaPayment::Crystal,
        ManaPayment::Token,
    ];
    assert_eq!(tributes(&state), [paid.to_vec()]);
    let events = events_of(&state, &tribute(&paid));
    assert_eq!(
        events,
        [
            Event::AltarTributePaid {
                player: SOLO,
                hex: RUINS
            },
            Event::DieTaken {
                player: SOLO,
                die: BLUE_IN_SOURCE,
                color: blue()
            },
            Event::CrystalUsed {
                player: SOLO,
                color: BasicColor::Blue
            },
            Event::ManaTokenUsed {
                player: SOLO,
                color: blue()
            },
            Event::SiteConquered {
                player: SOLO,
                hex: RUINS
            },
            Event::FameGained {
                player: SOLO,
                fame: 7
            },
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.shields().get(&RUINS), Some(&SOLO));
    assert!(!after.ruins().contains_key(&RUINS));
    assert_eq!(after.ruin_discard(), [RuinToken::BlueAltar]);
    assert_eq!(after.fame(SOLO), 7);
    assert_eq!(after.move_points(SOLO), 0);
    assert_eq!(after.influence_points(SOLO), 0);
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::Blue), 0);
    assert!(after.mana_tokens(SOLO).is_empty());
}

// C11 — Carte : sans 3 mana de la couleur, pas d'offrande (pas de paiement partiel).
#[test]
fn altar_needs_three_mana() {
    let no_blue_die = [
        ManaColor::Basic(BasicColor::Green),
        ManaColor::Basic(BasicColor::Red),
    ];
    let state = altar(TimeOfDay::Day, &no_blue_die, 1, &[blue()]);
    assert!(tributes(&state).is_empty());
}

// C12 — Compendium, *Mana* S4 : un dé or paie de jour, pas de nuit ; un dé noir jamais.
#[test]
fn altar_gold_pays_by_day_only() {
    let by_day = altar(TimeOfDay::Day, &[ManaColor::Gold], 2, &[]);
    assert_eq!(
        tributes(&by_day),
        [vec![
            ManaPayment::Die(0),
            ManaPayment::Crystal,
            ManaPayment::Crystal
        ]]
    );
    let at_night = altar(TimeOfDay::Night, &[ManaColor::Gold], 2, &[]);
    assert!(tributes(&at_night).is_empty());
    for time in [TimeOfDay::Day, TimeOfDay::Night] {
        assert!(tributes(&altar(time, &[ManaColor::Black], 2, &[])).is_empty());
    }
}

// C13 — Spec 025 : un dé de la Source par tour, sans dé de plus ; jamais deux dés proposés.
#[test]
fn altar_respects_the_dice_limit() {
    let state = altar(TimeOfDay::Day, &[blue(), blue()], 2, &[]);
    let payments = tributes(&state);
    assert_eq!(
        payments,
        [
            vec![
                ManaPayment::Die(0),
                ManaPayment::Crystal,
                ManaPayment::Crystal
            ],
            vec![
                ManaPayment::Die(1),
                ManaPayment::Crystal,
                ManaPayment::Crystal
            ],
        ]
    );
}

// C14 — Compendium, *Movement* S11 : l'offrande est l'action du tour.
#[test]
fn altar_is_the_action_of_the_turn() {
    // Une provocation en entrant sur l'autel : les orques du centre de la campagne 8 touchent les
    // deux cases.
    let orcs = Hex::new(6, -4);
    let mut events = Ruins {
        figure: NEAR_RUINS,
        placed: vec![(prowlers(), orcs)],
        ..Ruins::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    let state = events.iter().fold(GameState::default(), apply);
    let state = played(&state, card(March, 1), Effect::Move(5));
    let provoked = done(&state, Action::MoveHero { to: RUINS });
    assert!(!provoked.enemies_to_fight(SOLO).is_empty());
    assert!(tributes(&provoked).is_empty());
    // Un défi depuis l'autel.
    let mut events = Ruins {
        placed: vec![(prowlers(), orcs)],
        ..Ruins::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    let state = events.iter().fold(GameState::default(), apply);
    assert!(!tributes(&state).is_empty());
    let challenged = done(&state, challenge(orcs));
    assert!(tributes(&challenged).is_empty());
    // Après l'offrande : ni pas, ni exploration, ni entrée, même avec du Mouvement joué ensuite.
    let state = Ruins::default().events();
    let mut state = state.iter().fold(GameState::default(), apply);
    for _ in 0..3 {
        state = apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color: BasicColor::Blue,
            },
        );
    }
    let paid = done(
        &state,
        tribute(&[
            ManaPayment::Crystal,
            ManaPayment::Crystal,
            ManaPayment::Crystal,
        ]),
    );
    let paid = played(&paid, card(March, 1), Effect::Move(4));
    assert!(legal_actions(&paid, SOLO).iter().all(|action| !matches!(
        action,
        Action::MoveHero { .. }
            | Action::FollowRoute { .. }
            | Action::Explore { .. }
            | Action::Interact
            | Action::EnterAdventureSite
            | Action::PayAltarTribute { .. }
            | Action::Challenge { .. }
    )));
}

// C33 bis (issue privée 141) — La vue dit si l'action du tour est encore possible, comme le moteur :
// une provocation la prend (MKUE p. 6, 6.b à 6.d), une main de Blessures sans carte jouée l'interdit
// (Compendium, *Resting* S4).
#[test]
fn view_tells_turn_action_taken() {
    let state = altar(TimeOfDay::Day, &Sandbox::default().dice, 3, &[]);
    assert!(!view(&state, SOLO).turn_action_taken);

    let orcs = Hex::new(6, -4);
    let state = Ruins {
        placed: vec![(prowlers(), orcs)],
        ..Ruins::default()
    }
    .state();
    let challenged = done(&state, challenge(orcs));
    assert!(view(&challenged, SOLO).turn_action_taken);

    let mut events = Ruins {
        hand: vec![wound(1), wound(2)],
        ..Ruins::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    let wounded = events.iter().fold(GameState::default(), apply);
    assert!(tributes(&wounded).is_empty());
    assert!(view(&wounded, SOLO).turn_action_taken);
}

// C15 — Compendium, *Miscellaneous* S6 : l'autel conquis est une case ordinaire, marquée du
// drapeau (Bouclier) du joueur.
#[test]
fn fulfilled_altar_is_an_ordinary_space() {
    let mut events = Ruins {
        conquered: true,
        ..Ruins::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    let state = events.iter().fold(GameState::default(), apply);
    assert_eq!(state.shields().get(&RUINS), Some(&SOLO));
    assert!(!state.ruins().contains_key(&RUINS));
    assert!(!view(&state, SOLO).ruins.contains_key(&RUINS));
    assert!(tributes(&state).is_empty());
    assert!(!legal_actions(&state, SOLO).contains(&Action::EnterAdventureSite));
}

// C16 — MKUE p. 11, 6 : la gloire de l'autel change le niveau en fin de tour, pas avant.
#[test]
fn altar_fame_levels_up_at_end_of_turn() {
    let state = altar(TimeOfDay::Day, &Sandbox::default().dice, 1, &[blue()]);
    let events = events_of(
        &state,
        &tribute(&[
            ManaPayment::Die(BLUE_IN_SOURCE),
            ManaPayment::Crystal,
            ManaPayment::Token,
        ]),
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::LevelGained { .. }))
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.level(SOLO), 1);
    assert_eq!(after.pending_level(SOLO), Some(2));
    let end = legal_actions(&after, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("une fin de tour");
    assert_events_in_order(
        &events_of(&after, &end),
        &[Event::LevelGained {
            player: SOLO,
            level: 2,
        }],
    );
}

// L3 — Compendium, *Mana* S4 : de jour, un jeton de mana or paie l'offrande.
#[test]
fn altar_takes_a_gold_token_by_day() {
    let state = altar(TimeOfDay::Day, &[], 2, &[ManaColor::Gold]);
    let paid = [
        ManaPayment::Crystal,
        ManaPayment::Crystal,
        ManaPayment::Token,
    ];
    assert_eq!(tributes(&state), [paid.to_vec()]);
    assert_events_in_order(
        &events_of(&state, &tribute(&paid)),
        &[Event::ManaTokenUsed {
            player: SOLO,
            color: ManaColor::Gold,
        }],
    );
}

// --- Ennemis des ruines (C17 à C21, L1, L2) ---

fn melee(amount: u8) -> Effect {
    Effect::Attack {
        amount,
        element: Element::Physical,
        kind: AttackKind::Melee,
    }
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn can_enter(state: &GameState) -> bool {
    legal_actions(state, SOLO).contains(&Action::EnterAdventureSite)
}

fn entered(ruins: Ruins) -> GameState {
    done(&ruins.state(), Action::EnterAdventureSite)
}

/// En phase d'attaque, `targets` vaincus par une attaque de `amount` (la première carte de la main
/// jouée de côté).
fn beat(state: &GameState, targets: &[EnemyToken], amount: u8) -> GameState {
    let attacking = done(&skip_to_attack_phase(state), declare(targets));
    let first = attacking.hand(SOLO)[0];
    let attacking = played(&attacking, first, melee(amount));
    done(&attacking, Action::ResolveAttack)
}

fn end_combat(state: &GameState) -> Vec<Event> {
    events_of(&skip_to_attack_phase(state), &Action::EndCombat)
}

fn activations(state: &GameState) -> usize {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::ActivateUnit { .. }))
        .count()
}

// C17 — Carte *Ancient Ruins*, *Enemies* ; MKUE p. 8, 1.b : un jeton par couleur montrée, le
// combat aussitôt, unités permises.
#[test]
fn entering_ruins_draws_the_token_enemies() {
    let state = Ruins {
        token: RuinToken::BrownRed,
        tops: vec![minotaur(), swamp_dragon()],
        owned: vec![peasants()],
        ..Ruins::default()
    }
    .state();
    assert!(can_enter(&state));
    let events = events_of(&state, &Action::EnterAdventureSite);
    assert_eq!(
        events,
        [
            Event::AdventureSiteEntered {
                player: SOLO,
                hex: RUINS
            },
            Event::EnemyTokenPlaced {
                token: minotaur(),
                hex: RUINS
            },
            Event::EnemyTokenPlaced {
                token: swamp_dragon(),
                hex: RUINS
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(RUINS, minotaur()), (RUINS, swamp_dragon())],
            },
        ]
    );
    let fighting = events.iter().fold(state, apply);
    let blocking = done(&fighting, Action::NextCombatPhase);
    assert!(activations(&done(&blocking, declare_block(minotaur()))) > 0);

    let green = Ruins {
        token: RuinToken::GreenGreen,
        tops: vec![prowlers(), diggers()],
        ..Ruins::default()
    }
    .state();
    assert_events_in_order(
        &events_of(&green, &Action::EnterAdventureSite),
        &[
            Event::EnemyTokenPlaced {
                token: prowlers(),
                hex: RUINS,
            },
            Event::EnemyTokenPlaced {
                token: diggers(),
                hex: RUINS,
            },
        ],
    );
}

// C18 — MKUE p. 10, 3.c : les ennemis non vaincus restent face visible ; à l'entrée suivante, on
// combat ceux-là seuls, sans tirage.
#[test]
fn ruins_enemies_are_not_replaced() {
    let fighting = entered(Ruins {
        token: RuinToken::GreyViolet,
        tops: vec![guardsmen(), monks()],
        ..Ruins::default()
    });
    let won = beat(&fighting, &[guardsmen()], 7);
    let events = end_combat(&won);
    assert_eq!(events, [Event::CombatEnded { player: SOLO }]);
    let after = events.iter().fold(skip_to_attack_phase(&won), apply);
    assert_eq!(after.enemy_tokens_at(RUINS), [monks()]);
    assert!(after.ruins().contains_key(&RUINS));
    assert!(!after.shields().contains_key(&RUINS));

    let again = Ruins {
        token: RuinToken::GreyViolet,
        placed: vec![(monks(), RUINS)],
        ..Ruins::default()
    }
    .state();
    assert!(can_enter(&again));
    assert_eq!(
        events_of(&again, &Action::EnterAdventureSite),
        [
            Event::AdventureSiteEntered {
                player: SOLO,
                hex: RUINS
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(RUINS, monks())],
            },
        ]
    );
}

// C19 — MKUE p. 10, 2.b ; FAQ p3 : tous vaincus, la ruine est conquise, sa récompense attend la
// fin du tour ; pas de réputation pour des Orques des ruines.
#[test]
fn ruins_conquest_and_no_reputation() {
    let fighting = entered(Ruins {
        token: RuinToken::GreenBrown,
        tops: vec![prowlers(), minotaur()],
        ..Ruins::default()
    });
    let attacking = done(
        &skip_to_attack_phase(&fighting),
        declare(&[prowlers(), minotaur()]),
    );
    let first = attacking.hand(SOLO)[0];
    let attacking = played(&attacking, first, melee(8));
    let events = events_of(&attacking, &Action::ResolveAttack);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::EnemiesDefeated { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ReputationChanged { .. }))
    );
    let won = events.iter().fold(attacking, apply);
    let events = end_combat(&won);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::SiteConquered {
                player: SOLO,
                hex: RUINS
            },
        ]
    );
    let after = events.iter().fold(skip_to_attack_phase(&won), apply);
    assert_eq!(after.shields().get(&RUINS), Some(&SOLO));
    assert!(!after.ruins().contains_key(&RUINS));
    assert_eq!(after.ruin_discard(), [RuinToken::GreenBrown]);
    assert_eq!(after.artifact_rewards(SOLO), 1);
}

// C20 — Compendium, *Miscellaneous* S16 : un combat ordinaire, aux règles de la manche ; les
// ennemis ne sont pas fortifiés par le site.
#[test]
fn ruins_combat_uses_round_rules() {
    let fighting = entered(Ruins {
        token: RuinToken::BrownRed,
        tops: vec![minotaur(), swamp_dragon()],
        ..Ruins::default()
    });
    assert_eq!(fighting.combat_time_of_day(SOLO), TimeOfDay::Day);
    let combat = fighting.combat(SOLO).expect("combat engagé");
    assert_eq!(combat.phase, CombatPhase::RangedAndSiege);
    assert!(combat.enemies.iter().all(|enemy| !enemy.site_fortified));
    assert!(legal_actions(&fighting, SOLO).contains(&declare(&[minotaur()])));
}

// C21 — Spec 044, C17 : un jeton resté sur la ruine n'est pas un ennemi errant.
#[test]
fn ruins_tokens_are_not_rampaging() {
    let state = Ruins {
        figure: RUINS_GLADE,
        token: RuinToken::GreyViolet,
        placed: vec![(monks(), RUINS)],
        ..Ruins::default()
    }
    .state();
    let state = played(&state, card(March, 1), Effect::Move(10));
    let legal = legal_actions(&state, SOLO);
    assert!(!legal.contains(&challenge(RUINS)));
    assert!(legal.contains(&Action::MoveHero { to: RUINS }));
    let beside = events_of(
        &state,
        &Action::MoveHero {
            to: Hex::new(6, -4),
        },
    );
    assert!(
        !beside
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. }))
    );
}

// L1 — Spec 034 : assommé dans une ruine, le combat continue ; tout vaincre conquiert.
#[test]
fn knocked_out_in_ruins_still_conquers() {
    let fighting = entered(Ruins {
        token: RuinToken::GreenBrown,
        tops: vec![prowlers(), minotaur()],
        ..Ruins::default()
    });
    let blocking = done(&fighting, Action::NextCombatPhase);
    let damage = events_of(&blocking, &Action::NextCombatPhase);
    assert!(
        damage
            .iter()
            .any(|event| matches!(event, Event::KnockedOut { .. })),
        "{damage:?}"
    );
    let hurt = damage.iter().fold(blocking, apply);
    let won = apply(
        hurt,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![prowlers(), minotaur()],
            fame: 6,
        },
    );
    assert!(end_combat(&won).contains(&Event::SiteConquered {
        player: SOLO,
        hex: RUINS
    }));
}

// L2 — Spec 044, L2 : la créature invoquée n'est pas un jeton de la ruine.
#[test]
fn summoned_creature_is_not_a_ruins_token() {
    let fighting = entered(Ruins {
        token: RuinToken::GreenBrown,
        tops: vec![orc_summoners(), minotaur()],
        ..Ruins::default()
    });
    let summoned = done(&fighting, Action::NextCombatPhase);
    assert!(
        summoned
            .combat(SOLO)
            .is_some_and(|combat| combat.enemies[0].summoned.is_some())
    );
    let won = beat(&summoned, &[orc_summoners(), minotaur()], 9);
    assert!(end_combat(&won).contains(&Event::SiteConquered {
        player: SOLO,
        hex: RUINS
    }));
}

// --- Récompenses de fin de tour (C22 à C27, L4, L7, L8) ---

fn artifacts(count: usize) -> Vec<CardId> {
    arcanum_core::Artifact::ALL[..count]
        .iter()
        .map(|&card| artifact(card))
        .collect()
}

fn advanced_actions(count: usize) -> Vec<CardId> {
    arcanum_core::AdvancedAction::ALL[..count]
        .iter()
        .map(|&card| advanced(card))
        .collect()
}

fn spell(card: arcanum_core::Spell) -> CardId {
    CardId::new(arcanum_core::DeedCard::Spell(card), 1)
}

fn units(count: usize) -> Vec<UnitCard> {
    [Unit::Foresters, Unit::Herbalists, Unit::NorthernMonks][..count]
        .iter()
        .map(|&unit| UnitCard::new(unit, 1))
        .collect()
}

/// La ruine conquise ce tour (récompense en attente), figurine dessus.
fn conquered(ruins: Ruins) -> GameState {
    Ruins {
        conquered: true,
        ..ruins
    }
    .state()
}

/// La première fin de tour proposée.
fn end_turn(state: &GameState) -> Action {
    legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("fin du tour proposée")
}

fn has_turn_started(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::TurnStarted { .. }))
}

// C22 — MKUE p. 11, 5.a : un cristal de chaque couleur non pleine, avant le choix du sort.
#[test]
fn crystal_set_reward() {
    let mut events = Ruins {
        token: RuinToken::BrownViolet,
        conquered: true,
        spells: vec![spell(arcanum_core::Spell::ALL[0])],
        ..Ruins::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Red,
    }));
    let state = events.iter().fold(GameState::default(), apply);
    assert_eq!(state.crystal_set_rewards(SOLO), 1);
    assert_eq!(state.spell_rewards(SOLO), 1);
    let events = events_of(&state, &end_turn(&state));
    let gained: Vec<BasicColor> = events
        .iter()
        .filter_map(|event| match event {
            Event::CrystalGained { color, .. } => Some(*color),
            _ => None,
        })
        .collect();
    assert_eq!(
        gained,
        [BasicColor::Blue, BasicColor::White, BasicColor::Green]
    );
    assert!(!has_turn_started(&events));
    let after = events.iter().fold(state, apply);
    assert_eq!(after.crystal_set_rewards(SOLO), 0);
    assert!(
        legal_actions(&after, SOLO)
            .iter()
            .all(|action| matches!(action, Action::TakeSpell { .. }))
    );
}

// C23 — MKUE p. 11, 5.b : deux artefacts, trois tirés ; deux gardés, le dernier sous la pioche.
#[test]
fn two_artifacts_draw_three() {
    let drawn = artifacts(3);
    let state = conquered(Ruins {
        token: RuinToken::BrownRed,
        artifacts: drawn.clone(),
        ..Ruins::default()
    });
    assert_eq!(state.artifact_rewards(SOLO), 2);
    let events = events_of(&state, &end_turn(&state));
    assert_events_in_order(
        &events,
        &[Event::ArtifactsDrawn {
            player: SOLO,
            cards: drawn.clone(),
        }],
    );
    let choosing = events.iter().fold(state, apply);
    let take = |card: CardId| Action::TakeArtifact { card };
    assert_eq!(
        legal_actions(&choosing, SOLO),
        drawn.iter().map(|&card| take(card)).collect::<Vec<_>>()
    );
    let first = events_of(&choosing, &take(drawn[1]));
    assert_eq!(
        first,
        [Event::ArtifactTaken {
            player: SOLO,
            card: drawn[1],
        }]
    );
    let choosing = first.iter().fold(choosing, apply);
    assert_eq!(
        legal_actions(&choosing, SOLO),
        [take(drawn[0]), take(drawn[2])]
    );
    let second = events_of(&choosing, &take(drawn[2]));
    assert_eq!(
        second[..2],
        [
            Event::ArtifactTaken {
                player: SOLO,
                card: drawn[2],
            },
            Event::ArtifactReturned { card: drawn[0] },
        ]
    );
    assert!(has_turn_started(&second));
    let after = second.iter().fold(choosing, apply);
    assert_eq!(
        after
            .common_deck(arcanum_core::CommonDeck::Artifacts)
            .last(),
        Some(&arcanum_core::CommonCard::Deed(drawn[0]))
    );
    // Les deux artefacts gardés sur le deck, le dernier pris dessus : la pioche (une carte ici,
    // la main gardée en a quatre) prend le dernier.
    assert!(after.hand(SOLO).contains(&drawn[2]));
    assert_eq!(after.deck(SOLO).first(), Some(&drawn[1]));
}

// C24 — MKUE p. 11, 5.c : l'action avancée de la récompense, de l'offre d'actions avancées (pas
// une action avancée de monastère) ; avec un niveau pair ce tour, deux choix successifs.
#[test]
fn advanced_action_reward() {
    let offer = advanced_actions(3);
    let monastery = advanced_actions(4)[3];
    let state = conquered(Ruins {
        token: RuinToken::GreenRed,
        artifacts: artifacts(2),
        advanced: offer.clone(),
        monastery_advanced: vec![monastery],
        fame: 3,
        ..Ruins::default()
    });
    assert_eq!(state.advanced_action_rewards(SOLO), 1);
    let events = events_of(&state, &end_turn(&state));
    assert_events_in_order(
        &events,
        &[Event::LevelGained {
            player: SOLO,
            level: 2,
        }],
    );
    let choosing = events.iter().fold(state, apply);
    let taken = done(
        &choosing,
        Action::TakeArtifact {
            card: artifacts(2)[0],
        },
    );
    let gain = |card: CardId| Action::GainAdvancedAction { card };
    assert_eq!(
        legal_actions(&taken, SOLO),
        offer.iter().map(|&card| gain(card)).collect::<Vec<_>>()
    );
    assert_eq!(taken.advanced_action_choices(SOLO), 2);
    let first = events_of(&taken, &gain(offer[0]));
    assert!(!has_turn_started(&first));
    let again = first.iter().fold(taken, apply);
    assert!(!legal_actions(&again, SOLO).contains(&gain(monastery)));
    let second_card = match legal_actions(&again, SOLO)[0] {
        Action::GainAdvancedAction { card } => card,
        ref other => panic!("choix d'action avancée attendu : {other:?}"),
    };
    assert!(has_turn_started(&events_of(&again, &gain(second_card))));
}

// C25, L7 — MKUE p. 11, 5.d ; Compendium, *Miscellaneous* S7 : une unité de l'offre, gratuite,
// disponible ; l'offre n'est pas complétée ; ni influence, ni réputation.
#[test]
fn unit_reward_from_the_offer() {
    let offer = units(2);
    let monastery = advanced_actions(1)[0];
    let state = conquered(Ruins {
        token: RuinToken::GreyViolet,
        offer: offer.clone(),
        monastery_advanced: vec![monastery],
        ..Ruins::default()
    });
    assert_eq!(state.unit_rewards(SOLO), 1);
    let events = events_of(&state, &end_turn(&state));
    assert!(!has_turn_started(&events));
    let choosing = events.iter().fold(state, apply);
    assert_eq!(choosing.unit_choices(SOLO), 1);
    let take = |unit: UnitCard| Action::TakeUnitReward {
        unit,
        disband: None,
    };
    assert_eq!(
        legal_actions(&choosing, SOLO),
        [take(offer[0]), take(offer[1])]
    );
    let events = events_of(&choosing, &take(offer[1]));
    assert_eq!(
        events[0],
        Event::UnitRewardTaken {
            player: SOLO,
            unit: offer[1],
        }
    );
    assert!(has_turn_started(&events));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::OfferCardDealt { .. } | Event::ReputationChanged { .. }
    )));
    let after = events.iter().fold(choosing, apply);
    assert_eq!(after.units(SOLO).len(), 1);
    assert_eq!(after.units(SOLO)[0].card, offer[1]);
    assert!(after.units(SOLO)[0].ready);
    assert_eq!(
        after.offer(arcanum_core::Offer::Units),
        [
            arcanum_core::CommonCard::Unit(offer[0]),
            arcanum_core::CommonCard::Deed(monastery)
        ]
    );
    assert_eq!(after.unit_choices(SOLO), 0);
}

// C26, L8 — MKUE p. 11, 5.d : sans jeton libre, renvoyer une unité (même blessée) ou renoncer ;
// exception : le jeton de commandement du niveau gagné ce tour sert déjà.
#[test]
fn unit_reward_after_level_up() {
    let offer = units(2);
    let mut events = Ruins {
        token: RuinToken::GreyViolet,
        conquered: true,
        owned: vec![peasants()],
        offer: offer.clone(),
        ..Ruins::default()
    }
    .events();
    events.push(Event::UnitWounded {
        player: SOLO,
        unit: peasants(),
        wounds: 1,
    });
    let full = events.iter().fold(GameState::default(), apply);
    let choosing = done(&full, end_turn(&full));
    let take = |unit: UnitCard, disband: Option<UnitCard>| Action::TakeUnitReward { unit, disband };
    assert_eq!(
        legal_actions(&choosing, SOLO),
        [
            take(offer[0], Some(peasants())),
            take(offer[1], Some(peasants())),
            Action::DeclineUnitReward,
        ]
    );
    let swapped = events_of(&choosing, &take(offer[0], Some(peasants())));
    assert_eq!(
        swapped[..2],
        [
            Event::UnitDisbanded {
                player: SOLO,
                unit: peasants(),
            },
            Event::UnitRewardTaken {
                player: SOLO,
                unit: offer[0],
            },
        ]
    );
    let declined = events_of(&choosing, &Action::DeclineUnitReward);
    assert_eq!(declined[0], Event::UnitRewardDeclined { player: SOLO });
    assert!(has_turn_started(&declined));
    let after = declined.iter().fold(choosing, apply);
    assert_eq!(after.units(SOLO).len(), 1);
    assert_eq!(after.unit_choices(SOLO), 0);

    // Niveau 2 à 3 ce tour : un second jeton de commandement, le choix sans renvoi.
    let leveling = conquered(Ruins {
        token: RuinToken::GreyViolet,
        owned: vec![peasants()],
        offer: offer.clone(),
        fame: 8,
        level: 2,
        ..Ruins::default()
    });
    let events = events_of(&leveling, &end_turn(&leveling));
    assert_events_in_order(
        &events,
        &[Event::LevelGained {
            player: SOLO,
            level: 3,
        }],
    );
    let choosing = events.iter().fold(leveling, apply);
    assert_eq!(
        legal_actions(&choosing, SOLO),
        [take(offer[0], None), take(offer[1], None)]
    );
}

// C27 — Compendium, *Miscellaneous* S11 : sans unité dans l'offre, la récompense est perdue.
#[test]
fn unit_reward_lost_without_units() {
    let state = conquered(Ruins {
        token: RuinToken::GreyViolet,
        ..Ruins::default()
    });
    assert_eq!(state.unit_rewards(SOLO), 1);
    let events = events_of(&state, &end_turn(&state));
    assert!(has_turn_started(&events));
    let after = events.iter().fold(state, apply);
    assert_eq!(after.unit_choices(SOLO), 0);
    assert_eq!(after.unit_rewards(SOLO), 0);
}

// L4 — Specs 041 et 050 : un seul artefact pour deux récompenses, pris sans choix ; offre
// d'actions avancées vide, la récompense perdue.
#[test]
fn short_artifact_deck_and_empty_offer() {
    let only = artifacts(1)[0];
    let mut events = Ruins {
        token: RuinToken::BrownRed,
        conquered: true,
        ..Ruins::default()
    }
    .events();
    events.push(Event::CommonDeckShuffled {
        deck: arcanum_core::CommonDeck::Artifacts,
        order: vec![arcanum_core::CommonCard::Deed(only)],
    });
    let state = events.iter().fold(GameState::default(), apply);
    let events = events_of(&state, &end_turn(&state));
    assert_events_in_order(
        &events,
        &[
            Event::ArtifactsDrawn {
                player: SOLO,
                cards: vec![only],
            },
            Event::ArtifactTaken {
                player: SOLO,
                card: only,
            },
        ],
    );
    assert!(has_turn_started(&events));

    let mut events = Ruins {
        token: RuinToken::GreenRed,
        conquered: true,
        ..Ruins::default()
    }
    .events();
    events.push(Event::CommonDeckShuffled {
        deck: arcanum_core::CommonDeck::Artifacts,
        order: vec![arcanum_core::CommonCard::Deed(only)],
    });
    let state = events.iter().fold(GameState::default(), apply);
    assert!(has_turn_started(&events_of(&state, &end_turn(&state))));
}

// --- Décompte et vue (C28, C29) ---

// C28 — MKUE p. 15, *The Greatest Adventurer* : un autel conquis compte pour L'Aventurier.
#[test]
fn adventurer_counts_ruins() {
    let state = altar(TimeOfDay::Day, &[], 3, &[]);
    let paid = [ManaPayment::Crystal; 3];
    let after = done(&state, tribute(&paid));
    let score = arcanum_core::final_score(&after, SOLO);
    assert_eq!(score.adventure_shields, 1);
    assert_eq!(score.points(arcanum_core::Achievement::Adventurer), 2);
}

// C29 — Information cachée de la spec 045 : la vue donne les ruines (jeton ou face cachée), le
// nombre de jetons de la pile, les récompenses en attente et les ruines révélées par le dernier
// pas.
#[test]
fn view_shows_ruins() {
    let state = night_ruins(NEAR_RUINS, 10);
    let seen = view(&state, SOLO);
    assert_eq!(seen.ruins.get(&RUINS), Some(&None));
    assert_eq!(seen.ruin_pile, state.ruin_pile().len());
    assert!(seen.revealed_ruins.is_empty());
    let stepped = done(&state, Action::MoveHero { to: RUINS });
    assert_eq!(view(&stepped, SOLO).revealed_ruins, [RUINS]);
    assert_eq!(stepped.revealed_ruins(SOLO), [RUINS]);
    // L'action suivante efface l'annonce.
    let next = done(&stepped, Action::MoveHero { to: NEAR_RUINS });
    assert!(view(&next, SOLO).revealed_ruins.is_empty());

    // Récompenses en attente : un lot de cristaux, une action avancée, une unité.
    let state = conquered(Ruins {
        token: RuinToken::BrownViolet,
        ..Ruins::default()
    });
    let seen = view(&state, SOLO);
    assert_eq!(seen.crystal_set_rewards, state.crystal_set_rewards(SOLO));
    assert_eq!(seen.crystal_set_rewards, 1);
    assert_eq!(seen.spell_rewards, 1);
    let state = conquered(Ruins {
        token: RuinToken::GreenRed,
        advanced: advanced_actions(1),
        ..Ruins::default()
    });
    assert_eq!(view(&state, SOLO).advanced_action_rewards, 1);
    let state = conquered(Ruins {
        token: RuinToken::GreyViolet,
        offer: units(1),
        ..Ruins::default()
    });
    assert_eq!(view(&state, SOLO).unit_rewards, 1);
    let choosing = done(&state, end_turn(&state));
    assert_eq!(view(&choosing, SOLO).unit_choices, 1);
    let state = conquered(Ruins {
        token: RuinToken::BrownRed,
        artifacts: artifacts(3),
        ..Ruins::default()
    });
    let choosing = done(&state, end_turn(&state));
    assert_eq!(view(&choosing, SOLO).artifacts_to_keep, 2);
}

// --- Joueur de test (C30) ---

/// Le choix du joueur « premier choix » dans cet état.
fn first_choice(state: &GameState) -> Action {
    let legal = legal_actions(state, SOLO);
    let mut player = FirstChoice::default();
    arcanum_core::Player::request(&mut player, &view(state, SOLO), &legal);
    let index = arcanum_core::Player::poll(&mut player).expect("un choix");
    legal[index].clone()
}

// C30 — Les parcours ne changent pas (risques du plan) : ni offrande ni entrée dans une ruine ; la
// première unité de récompense.
#[test]
fn first_choice_ignores_ruins() {
    let mut events = Ruins {
        hand: Vec::new(),
        ..Ruins::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Blue,
    }));
    let state = events.iter().fold(GameState::default(), apply);
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::PayAltarTribute { .. }))
    );
    assert!(matches!(first_choice(&state), Action::EndTurn { .. }));

    let state = Ruins {
        token: RuinToken::GreyViolet,
        hand: Vec::new(),
        ..Ruins::default()
    }
    .state();
    assert!(can_enter(&state));
    assert!(matches!(first_choice(&state), Action::EndTurn { .. }));

    let offer = units(2);
    let state = conquered(Ruins {
        token: RuinToken::GreyViolet,
        offer: offer.clone(),
        ..Ruins::default()
    });
    let choosing = done(&state, end_turn(&state));
    assert_eq!(
        first_choice(&choosing),
        Action::TakeUnitReward {
            unit: offer[0],
            disband: None,
        }
    );
}
