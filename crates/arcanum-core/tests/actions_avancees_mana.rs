//! Scénarios de la spec 056 (actions avancées : mana et cartes) : détruire une carte, cartes gagnées
//! des offres, Rituel de sang, Maîtrise du cristal, Tempête de mana, Apprentissage, Effet maximum.

use arcanum_core::{
    Action, AdvancedAction, BasicColor, BasicEffect, BoostTarget, CardDestination, CardId,
    CardPlay, CommonCard, DeedCard, DestroyedFrom, EnemyToken, Event, GameState, Hex, InProgress,
    ManaChoice, ManaColor, ManaPayment, Offer, Spell, TimeOfDay, TurnFact, apply, legal_actions,
    view,
};

mod arene;
use arene::*;

use AdvancedAction::*;
use arcanum_core::BasicAction::{
    Concentration, Crystallize, Determination, Improvisation, March, Promise, Rage, Threaten,
    Tranquility,
};

// --- Décors ---

/// Un décor de la spec 056 : le bac à sable (Arythea) de jour ou de nuit, la figurine, la main, les
/// dés de la Source, l'offre d'actions avancées et celle de sorts (dans l'ordre), puis des
/// événements de plus (cristaux…).
#[derive(Clone)]
struct Decor {
    time: TimeOfDay,
    figure: Hex,
    hand: Vec<CardId>,
    dice: Vec<ManaColor>,
    advanced: Vec<CardId>,
    spells: Vec<CardId>,
    placed: Vec<(EnemyToken, Hex)>,
    extra: Vec<Event>,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            time: TimeOfDay::Day,
            figure: PLAIN_W,
            hand: Vec::new(),
            dice: Sandbox::default().dice,
            advanced: Vec::new(),
            spells: Vec::new(),
            placed: Vec::new(),
            extra: Vec::new(),
        }
    }
}

impl Decor {
    fn events(&self) -> Vec<Event> {
        let mut events = Sandbox {
            time: self.time,
            figure: Some(self.figure),
            hand: self.hand.clone(),
            dice: self.dice.clone(),
            placed: self.placed.clone(),
            ..Sandbox::default()
        }
        .events();
        if !self.advanced.is_empty() {
            events.extend(advanced_offer(&self.advanced));
        }
        if !self.spells.is_empty() {
            events.extend(spell_offer(&self.spells));
        }
        events.extend(self.extra.iter().cloned());
        events
    }

    fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }

    /// Le décor, avec les Rôdeurs sur les orques ②, puis le combat engagé contre eux (phase à
    /// distance).
    fn fighting(&self) -> GameState {
        let decor = Decor {
            placed: vec![(prowlers(), ORCS_2)],
            ..self.clone()
        };
        let state = done(&decor.state(), challenge(ORCS_2));
        done(&state, Action::StartCombat)
    }
}

fn spell(spell: Spell) -> CardId {
    CardId::new(DeedCard::Spell(spell), 1)
}

fn crystal(color: BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

fn basic_color(color: BasicColor) -> ManaColor {
    ManaColor::Basic(color)
}

/// L'option `option` de l'effet de base de `card`, avec la carte à détruire ou défausser, la carte
/// prise dans une offre et le dé choisi.
fn basic_play(
    card: CardId,
    option: u8,
    discard: Option<CardId>,
    gain: Option<CardId>,
    die: Option<u8>,
) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            option,
            discard,
            mana: None,
            with: None,
            gain,
            die,
        },
    }
}

/// L'option `option` de l'effet renforcé de `card` payé par `mana`, avec la carte à détruire, la
/// carte prise dans une offre et le mana en plus.
fn strong_play(
    card: CardId,
    option: u8,
    mana: ManaPayment,
    discard: Option<CardId>,
    gain: Option<CardId>,
    extra_mana: Option<ManaPayment>,
) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            option,
            discard,
            mana,
            set_dice: [None, None],
            with: None,
            extra_mana,
            gain,
        },
    }
}

/// Les `PlayCard` légaux de `card` joués pour un effet (ni de côté).
fn effect_plays(state: &GameState, card: CardId) -> Vec<CardPlay> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card: played, play } if played == card => Some(play),
            _ => None,
        })
        .filter(|play| !matches!(play, CardPlay::Sideways(_)))
        .collect()
}

fn end_turn() -> Action {
    Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

/// La carte est-elle encore quelque part chez le joueur : main, défausse, deck, zone de jeu ?
fn owned_anywhere(state: &GameState, card: CardId) -> bool {
    state.hand(SOLO).contains(&card)
        || state.discard_pile(SOLO).contains(&card)
        || state.deck(SOLO).contains(&card)
        || state
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == card)
}

// --- Étape 1 : détruire une carte, Décomposition ---

// Carte Décomposition ; MKUE p. 5, *Discarding and Throwing Away* 2.b ; *"Gain" Effects* 2.
#[test]
fn decompose_two_crystals_of_destroyed_color() {
    let decompose = advanced(Decompose);
    let rage = card(Rage, 1);
    let state = Decor {
        hand: vec![decompose, rage, card(March, 1)],
        extra: vec![crystal(BasicColor::Red), crystal(BasicColor::Red)],
        ..Decor::default()
    }
    .state();
    let action = basic_play(decompose, 0, Some(rage), None, None);
    let events = events_of(&state, &action);
    assert_events_in_order(
        &events,
        &[
            Event::CardDestroyed {
                player: SOLO,
                card: rage,
                from: DestroyedFrom::Hand,
            },
            crystal(BasicColor::Red),
            Event::ManaTokenGained {
                player: SOLO,
                color: basic_color(BasicColor::Red),
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Red), 3);
    assert!(!owned_anywhere(&state, rage));
    // Spec 073 : la carte détruite entre dans les faits du tour.
    assert!(
        state
            .turn_facts(SOLO)
            .contains(&TurnFact::CardDestroyed { card: rage })
    );
    // MKUE p. 11, 1 : elle ne revient pas en fin de tour.
    let state = done(&state, end_turn());
    assert!(!owned_anywhere(&state, rage));
}

// Carte Décomposition renforcée.
#[test]
fn strong_decompose_other_colors() {
    let decompose = advanced(Decompose);
    let march = card(March, 1);
    let state = Decor {
        hand: vec![decompose, march],
        ..Decor::default()
    }
    .state();
    let action = strong_play(
        decompose,
        0,
        ManaPayment::Die(RED_DIE),
        Some(march),
        None,
        None,
    );
    let state = done(&state, action);
    let inventory = state.inventory(SOLO);
    assert_eq!(inventory.crystals(BasicColor::Red), 1);
    assert_eq!(inventory.crystals(BasicColor::Blue), 1);
    assert_eq!(inventory.crystals(BasicColor::White), 1);
    assert_eq!(inventory.crystals(BasicColor::Green), 0);
    assert!(!owned_anywhere(&state, march));
}

// MKUE p. 4, 6.a : une Blessure n'est pas une carte action ; sans carte à détruire, pas d'effet.
#[test]
fn decompose_needs_an_action_card() {
    let decompose = advanced(Decompose);
    let state = Decor {
        hand: vec![decompose, wound(1), wound(2)],
        ..Decor::default()
    }
    .state();
    assert_eq!(effect_plays(&state, decompose), Vec::new());
}

// Spec 056, L1 : un sort ou un artefact n'est pas une carte action (MKUE p. 4, 6).
#[test]
fn spells_and_artifacts_are_not_action_cards() {
    let decompose = advanced(Decompose);
    let training = advanced(Training);
    let state = Decor {
        hand: vec![
            decompose,
            training,
            spell(Spell::Fireball),
            artifact(arcanum_core::Artifact::ALL[0]),
        ],
        advanced: vec![advanced(FireBolt), advanced(PathFinding)],
        ..Decor::default()
    }
    .state();
    // Décomposition et Entraînement peuvent se détruire l'une l'autre, jamais le sort ni l'artefact.
    let destroyed: Vec<Option<CardId>> = effect_plays(&state, decompose)
        .into_iter()
        .chain(effect_plays(&state, training))
        .map(|play| match play {
            CardPlay::Basic { discard, .. } | CardPlay::Strong { discard, .. } => discard,
            _ => None,
        })
        .collect();
    assert!(!destroyed.is_empty());
    assert!(
        destroyed
            .iter()
            .all(|card| *card == Some(decompose) || *card == Some(training)),
        "{destroyed:?}"
    );
}

// --- Étape 2 : cartes gagnées hors du deck, Entraînement, Don pour la magie ---

fn training_decor() -> Decor {
    Decor {
        hand: vec![
            advanced(Training),
            card(March, 1),
            card(Rage, 1),
            card(Determination, 1),
        ],
        advanced: vec![advanced(PathFinding), advanced(FireBolt), advanced(Agility)],
        ..Decor::default()
    }
}

// Carte Entraînement ; Compendium, *Training* S3 ; MKUE p. 5, *"Gain" Effects* 3.a.
#[test]
fn training_takes_same_color_to_discard() {
    let training = advanced(Training);
    let (march, rage, determination) = (card(March, 1), card(Rage, 1), card(Determination, 1));
    let state = training_decor().state();
    let plays = effect_plays(&state, training);
    let basic = |discard, gain| CardPlay::Basic {
        option: 0,
        discard: Some(discard),
        mana: None,
        with: None,
        gain: Some(gain),
        die: None,
    };
    assert!(plays.contains(&basic(rage, advanced(FireBolt))));
    assert!(!plays.contains(&basic(march, advanced(FireBolt))));
    assert!(plays.iter().all(|play| !matches!(
        play,
        CardPlay::Basic { discard, .. } | CardPlay::Strong { discard, .. } if *discard == Some(determination)
    )));
    let events = events_of(
        &state,
        &basic_play(training, 0, Some(march), Some(advanced(PathFinding)), None),
    );
    assert_events_in_order(
        &events,
        &[
            Event::CardDestroyed {
                player: SOLO,
                card: march,
                from: DestroyedFrom::Hand,
            },
            Event::OfferCardGained {
                player: SOLO,
                offer: Offer::AdvancedActions,
                card: CommonCard::Deed(advanced(PathFinding)),
                to: CardDestination::DiscardPile,
            },
        ],
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::OfferCardDealt {
            offer: Offer::AdvancedActions,
            ..
        }
    )));
    let state = events.iter().fold(state, apply);
    assert!(state.discard_pile(SOLO).contains(&advanced(PathFinding)));
    assert!(!state.deck(SOLO).contains(&advanced(PathFinding)));
    assert!(!owned_anywhere(&state, march));
    assert_eq!(state.offer(Offer::AdvancedActions).len(), 3);
}

// Carte Entraînement renforcée : la carte prise arrive en main, jouable aussitôt.
#[test]
fn strong_training_to_hand() {
    let training = advanced(Training);
    let rage = card(Rage, 1);
    let state = training_decor().state();
    let action = strong_play(
        training,
        0,
        ManaPayment::Die(GREEN_DIE),
        Some(rage),
        Some(advanced(FireBolt)),
        None,
    );
    let state = done(&state, action);
    assert!(state.hand(SOLO).contains(&advanced(FireBolt)));
    assert!(!effect_plays(&state, advanced(FireBolt)).is_empty());
}

// Compendium, *Training* S2 : la carte détruite est une autre carte.
#[test]
fn training_cannot_destroy_itself() {
    let training = advanced(Training);
    let state = Decor {
        hand: vec![training, card(March, 1)],
        advanced: vec![advanced(PathFinding), advanced(CrushingBolt)],
        ..Decor::default()
    }
    .state();
    let plays = effect_plays(&state, training);
    assert!(!plays.is_empty());
    assert!(plays.iter().all(|play| matches!(
        play,
        CardPlay::Basic { discard, .. } | CardPlay::Strong { discard, .. } if *discard == Some(card(March, 1))
    )));
}

fn magic_talent_decor() -> Decor {
    Decor {
        hand: vec![advanced(MagicTalent)],
        spells: vec![
            spell(Spell::Fireball),
            spell(Spell::Snowstorm),
            spell(Spell::Tremor),
        ],
        extra: vec![crystal(BasicColor::Red)],
        ..Decor::default()
    }
}

// Carte Don pour la magie renforcée ; Compendium, *Magic Talent* O4.
#[test]
fn strong_magic_talent_gains_spell_to_discard() {
    let talent = advanced(MagicTalent);
    let state = magic_talent_decor().state();
    // Les options dans l'ordre rouge, bleu, blanc, vert : aucune blanche, l'offre n'en a pas.
    assert!(
        effect_plays(&state, talent)
            .iter()
            .all(|play| !matches!(play, CardPlay::Strong { option: 2, .. }))
    );
    let action = strong_play(
        talent,
        0,
        ManaPayment::Die(BLUE_DIE),
        None,
        Some(spell(Spell::Fireball)),
        Some(ManaPayment::Crystal),
    );
    let events = events_of(&state, &action);
    assert_events_in_order(
        &events,
        &[
            Event::CrystalUsed {
                player: SOLO,
                color: BasicColor::Red,
            },
            Event::OfferCardGained {
                player: SOLO,
                offer: Offer::Spells,
                card: CommonCard::Deed(spell(Spell::Fireball)),
                to: CardDestination::DiscardPile,
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert!(state.discard_pile(SOLO).contains(&spell(Spell::Fireball)));
    assert_eq!(state.offer(Offer::Spells).len(), 3);
}

// MKUE p. 5, *Using Mana* 2.a : de jour, l'or vaut toute couleur de base.
#[test]
fn magic_talent_gold_by_day() {
    let talent = advanced(MagicTalent);
    let state = Decor {
        dice: vec![
            ManaColor::Gold,
            basic_color(BasicColor::Red),
            basic_color(BasicColor::White),
        ],
        extra: vec![crystal(BasicColor::Blue)],
        ..magic_talent_decor()
    }
    .state();
    let action = strong_play(
        talent,
        3,
        ManaPayment::Crystal,
        None,
        Some(spell(Spell::Tremor)),
        Some(ManaPayment::Die(0)),
    );
    let state = done(&state, action);
    assert!(state.discard_pile(SOLO).contains(&spell(Spell::Tremor)));
}

// Spec 056, L5 : une offre vide, pas d'option.
#[test]
fn empty_offers_give_no_option() {
    let state = Decor {
        hand: vec![advanced(Training), advanced(MagicTalent), card(March, 1)],
        extra: vec![crystal(BasicColor::Red)],
        ..Decor::default()
    }
    .state();
    assert_eq!(effect_plays(&state, advanced(Training)), Vec::new());
    assert_eq!(effect_plays(&state, advanced(MagicTalent)), Vec::new());
}

// Spec 056, règle « effet utilisable » : les cartes de mana, Entraînement et Don pour la magie se
// jouent aussi en combat (MKUE p. 8, *Combat with Enemies* 6).
#[test]
fn mana_and_card_cards_play_in_combat() {
    let state = Decor {
        hand: vec![
            advanced(Decompose),
            advanced(Training),
            advanced(MagicTalent),
            card(March, 1),
        ],
        advanced: vec![advanced(PathFinding)],
        spells: vec![spell(Spell::Tremor)],
        extra: vec![crystal(BasicColor::Green)],
        ..training_decor()
    }
    .fighting();
    for card in [
        advanced(Decompose),
        advanced(Training),
        advanced(MagicTalent),
    ] {
        assert!(!effect_plays(&state, card).is_empty(), "{card:?}");
    }
}

// --- Étape 3 : Rituel de sang ---

/// Les options de base de Rituel de sang : une par couleur de jeton, dans l'ordre de
/// `ManaColor::ALL`.
fn token_option(color: ManaColor) -> u8 {
    let index = ManaColor::ALL
        .iter()
        .position(|&each| each == color)
        .expect("une couleur de mana");
    u8::try_from(index).expect("six couleurs")
}

fn wounds_in_hand(state: &GameState) -> usize {
    state
        .hand(SOLO)
        .iter()
        .filter(|card| card.card().is_wound())
        .count()
}

// Carte Rituel de sang (version Lost Legion) ; Compendium, *Blood Ritual* S3.
#[test]
fn blood_ritual_wound_crystal_and_token() {
    let ritual = advanced(BloodRitual);
    let state = Decor {
        hand: vec![ritual],
        ..Decor::default()
    }
    .state();
    let green = basic_color(BasicColor::Green);
    let events = events_of(
        &state,
        &basic_play(ritual, token_option(green), None, None, None),
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::KnockedOut { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(wounds_in_hand(&state), 1);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Red), 1);
    assert_eq!(state.mana_tokens(SOLO), [green]);
}

// Compendium, *Blood Ritual* S3 ; MKUE p. 9, 5.c : en combat, la Blessure compte pour être assommé.
#[test]
fn blood_ritual_wound_counts_in_combat() {
    let ritual = advanced(BloodRitual);
    let march = card(March, 1);
    let state = Decor {
        hand: vec![ritual, march],
        ..Decor::default()
    }
    .fighting();
    let four = Event::WoundsTaken {
        player: SOLO,
        enemy: prowlers(),
        wounds: (1..=4).map(wound).collect(),
    };
    let state = apply(state, &four);
    let events = events_of(
        &state,
        &basic_play(ritual, token_option(ManaColor::Gold), None, None, None),
    );
    assert!(events.contains(&Event::KnockedOut {
        player: SOLO,
        cards: vec![march],
    }));
}

// Compendium, *Blood Ritual* S2 : un jeton noir de jour se prend, sans rien payer.
#[test]
fn blood_ritual_black_token_by_day() {
    let ritual = advanced(BloodRitual);
    let march = card(March, 1);
    let state = Decor {
        hand: vec![ritual, march],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        basic_play(ritual, token_option(ManaColor::Black), None, None, None),
    );
    assert_eq!(state.mana_tokens(SOLO), [ManaColor::Black]);
    assert!(effect_plays(&state, march).iter().all(|play| !matches!(
        play,
        CardPlay::Strong {
            mana: ManaPayment::Token,
            ..
        }
    )));
}

fn strong_ritual_state() -> (GameState, CardId) {
    let ritual = advanced(BloodRitual);
    let state = Decor {
        hand: vec![ritual],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        strong_play(ritual, 0, ManaPayment::Die(RED_DIE), None, None, None),
    );
    (state, ritual)
}

// Carte Rituel de sang renforcée ; Compendium, *Blood Ritual* S1.
#[test]
fn strong_blood_ritual_three_tokens_then_crystal() {
    let (state, _) = strong_ritual_state();
    assert_eq!(wounds_in_hand(&state), 1);
    let choose = |color| Action::ChooseTokenColor { color };
    assert_eq!(
        legal_actions(&state, SOLO),
        ManaColor::ALL.map(choose).to_vec()
    );
    assert_eq!(
        view(&state, SOLO).mana_choice,
        Some(ManaChoice {
            tokens_left: 3,
            crystal_offered: false,
        })
    );
    let green = basic_color(BasicColor::Green);
    let state = done(&state, choose(green));
    assert_eq!(
        view(&state, SOLO).mana_choice,
        Some(ManaChoice {
            tokens_left: 2,
            crystal_offered: false,
        })
    );
    let state = done(&state, choose(green));
    let state = done(&state, choose(ManaColor::Gold));
    assert_eq!(state.mana_tokens(SOLO), [green, green, ManaColor::Gold]);
    assert_eq!(
        view(&state, SOLO).mana_choice,
        Some(ManaChoice {
            tokens_left: 0,
            crystal_offered: true,
        })
    );
    let legal = legal_actions(&state, SOLO);
    assert_eq!(legal.last(), Some(&Action::DeclineCrystal));
    assert!(legal.iter().all(|action| matches!(
        action,
        Action::PayForCrystal { .. } | Action::DeclineCrystal
    )));
    let pay = Action::PayForCrystal {
        color: BasicColor::Green,
        mana: ManaPayment::Token,
    };
    assert!(legal.contains(&pay));
    let state = done(&state, pay);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Green), 1);
    assert_eq!(state.mana_tokens(SOLO), [green, ManaColor::Gold]);
    assert_eq!(view(&state, SOLO).mana_choice, None);
}

// Compendium, *Blood Ritual* S1 : une fois, aussitôt.
#[test]
fn blood_ritual_crystal_only_once() {
    let (state, _) = strong_ritual_state();
    let mut state = state;
    for _ in 0..3 {
        state = done(
            &state,
            Action::ChooseTokenColor {
                color: basic_color(BasicColor::White),
            },
        );
    }
    let events = events_of(&state, &Action::DeclineCrystal);
    assert_eq!(events, vec![Event::CrystalDeclined { player: SOLO }]);
    let state = events.iter().fold(state, apply);
    assert_eq!(view(&state, SOLO).mana_choice, None);
    assert!(legal_actions(&state, SOLO).iter().all(|action| !matches!(
        action,
        Action::PayForCrystal { .. } | Action::DeclineCrystal | Action::ChooseTokenColor { .. }
    )));
}

// --- Étape 4 : Maîtrise du cristal ---

// Carte Maîtrise du cristal : une couleur déjà possédée.
#[test]
fn crystal_mastery_needs_an_owned_color() {
    let mastery = advanced(CrystalMastery);
    let decor = Decor {
        hand: vec![mastery],
        ..Decor::default()
    };
    // Inventaire vide : aucune option de base.
    assert!(
        effect_plays(&decor.state(), mastery)
            .iter()
            .all(|play| !matches!(play, CardPlay::Basic { .. }))
    );
    let state = Decor {
        extra: vec![crystal(BasicColor::Blue)],
        ..decor
    }
    .state();
    let basic: Vec<CardPlay> = effect_plays(&state, mastery)
        .into_iter()
        .filter(|play| matches!(play, CardPlay::Basic { .. }))
        .collect();
    // Les options dans l'ordre rouge, bleu, blanc, vert.
    assert_eq!(basic.len(), 1);
    assert!(matches!(basic[0], CardPlay::Basic { option: 1, .. }));
    let state = done(&state, basic_play(mastery, 1, None, None, None));
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Blue), 2);
}

fn mastery_decor() -> Decor {
    Decor {
        hand: vec![advanced(CrystalMastery), card(March, 1)],
        extra: vec![crystal(BasicColor::Blue), crystal(BasicColor::Green)],
        ..Decor::default()
    }
}

fn crystal_strong(card: CardId) -> Action {
    strong_play(card, 0, ManaPayment::Crystal, None, None, None)
}

// FAQ p2 : les cristaux dépensés ce tour, le cristal qui a payé la carte compris.
#[test]
fn crystal_mastery_returns_spent_crystals() {
    let state = mastery_decor().state();
    let state = done(&state, crystal_strong(advanced(CrystalMastery)));
    let state = done(&state, crystal_strong(card(March, 1)));
    assert_eq!(state.inventory(SOLO).total(), 0);
    assert_eq!(
        view(&state, SOLO).crystals_to_return,
        [BasicColor::Blue, BasicColor::Green]
    );
    let events = events_of(&state, &end_turn());
    assert_events_in_order(
        &events,
        &[
            Event::CrystalsReturned {
                player: SOLO,
                colors: vec![BasicColor::Blue, BasicColor::Green],
            },
            Event::TurnEnded { player: SOLO },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Blue), 1);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Green), 1);
}

// FAQ p2 ; Compendium, *Crystal Mastery* Q1 : un cristal dépensé avant la carte revient aussi, pas
// un dé.
#[test]
fn crystal_mastery_counts_crystals_spent_before() {
    let state = mastery_decor().state();
    let state = done(&state, crystal_strong(card(March, 1)));
    let state = done(
        &state,
        strong_play(
            advanced(CrystalMastery),
            0,
            ManaPayment::Die(BLUE_DIE),
            None,
            None,
            None,
        ),
    );
    assert_eq!(view(&state, SOLO).crystals_to_return, [BasicColor::Green]);
    let state = done(&state, end_turn());
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Green), 1);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Blue), 1);
}

// Spec 056, Q8 : au plus 3 cristaux par couleur, le surplus perdu.
#[test]
fn crystal_mastery_returns_at_most_three() {
    let state = mastery_decor().state();
    let state = done(&state, crystal_strong(advanced(CrystalMastery)));
    let state = done(&state, crystal_strong(card(March, 1)));
    let state = vec![crystal(BasicColor::Green); 3]
        .iter()
        .fold(state, apply);
    assert_eq!(view(&state, SOLO).crystals_to_return, [BasicColor::Blue]);
}

// Spec 024 : le retour en arrière et la sauvegarde suivent les cristaux dépensés.
#[test]
fn crystal_mastery_follows_undo_and_reload() {
    let (mut game, puppet) = runner(mastery_decor().events());
    play(&mut game, &puppet, crystal_strong(advanced(CrystalMastery)));
    play(&mut game, &puppet, crystal_strong(card(March, 1)));
    // Rechargée avant la fin du tour, la partie rend les deux cristaux.
    let (reloaded, _) = runner(
        game.log()
            .entries()
            .iter()
            .map(|entry| entry.event.clone())
            .collect(),
    );
    assert_eq!(
        view(reloaded.state(), SOLO).crystals_to_return,
        [BasicColor::Blue, BasicColor::Green]
    );
    // Revenue avant Marche, la partie ne rend que le cristal bleu.
    game.undo().expect("Marche se reprend");
    assert_eq!(
        view(game.state(), SOLO).crystals_to_return,
        [BasicColor::Blue]
    );
}

// --- Étape 5 : Tempête de mana ---

fn dice(colors: &[ManaColor]) -> Vec<ManaColor> {
    colors.to_vec()
}

const GREEN: ManaColor = ManaColor::Basic(BasicColor::Green);
const BLUE: ManaColor = ManaColor::Basic(BasicColor::Blue);
const WHITE: ManaColor = ManaColor::Basic(BasicColor::White);
const RED: ManaColor = ManaColor::Basic(BasicColor::Red);

/// Le dé `die` de la Source montre désormais `color` (une relance aux résultats choisis).
fn rerolled(state: GameState, die: u8, color: ManaColor) -> GameState {
    apply(state, &Event::DieRerolled { die, color })
}

// Carte Tempête de mana : un dé de couleur de base, relancé ; ce n'est pas le dé du tour.
#[test]
fn mana_storm_crystal_and_reroll() {
    let storm = advanced(ManaStorm);
    let state = Decor {
        hand: vec![storm],
        dice: dice(&[GREEN, BLUE, ManaColor::Black]),
        ..Decor::default()
    }
    .state();
    let dice_chosen: Vec<Option<u8>> = effect_plays(&state, storm)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Basic { die, .. } => Some(die),
            _ => None,
        })
        .collect();
    assert_eq!(dice_chosen, [Some(0), Some(1)]);
    let events = events_of(&state, &basic_play(storm, 0, None, None, Some(0)));
    assert_events_in_order(&events, &[crystal(BasicColor::Green)]);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::DieRerolled { die: 0, .. })),
        "{events:?}"
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Green), 1);
    assert_eq!(state.source().len(), 3);
    assert!(state.dice_taken(SOLO).is_empty());
    assert_eq!(view(&state, SOLO).dice_limit, 1);
}

// MKUE p. 5, *Reverting* 2 : un dé relancé est un point de non-retour.
#[test]
fn mana_storm_reroll_cannot_be_undone() {
    let storm = advanced(ManaStorm);
    let decor = Decor {
        hand: vec![storm, card(March, 1)],
        dice: dice(&[GREEN, BLUE, WHITE]),
        ..Decor::default()
    };
    let (mut game, puppet) = runner(decor.events());
    play(
        &mut game,
        &puppet,
        basic_play(storm, 0, None, None, Some(1)),
    );
    assert!(game.undo().is_err());
}

// Carte Tempête de mana renforcée ; Compendium, *Mana Storm* S1 : les dés encore dans la Source.
#[test]
fn strong_mana_storm_rerolls_and_three_more_dice() {
    let storm = advanced(ManaStorm);
    let state = Decor {
        hand: vec![storm],
        ..Decor::default()
    }
    .state();
    let events = events_of(
        &state,
        &strong_play(storm, 0, ManaPayment::Die(WHITE_DIE), None, None, None),
    );
    let rerolled_dice: Vec<u8> = events
        .iter()
        .filter_map(|event| match event {
            Event::DieRerolled { die, .. } => Some(*die),
            _ => None,
        })
        .collect();
    assert_eq!(rerolled_dice, [GREEN_DIE, RED_DIE, BLUE_DIE]);
    let state = events.iter().fold(state, apply);
    assert_eq!(view(&state, SOLO).dice_limit, 4);
}

// Carte Tempête de mana renforcée ; spec 056, Q9 : noir et or de toute couleur de base.
#[test]
fn strong_mana_storm_black_and_gold_any_color() {
    let storm = advanced(ManaStorm);
    let march = card(March, 1);
    for (time, face) in [
        (TimeOfDay::Day, ManaColor::Black),
        (TimeOfDay::Night, ManaColor::Gold),
    ] {
        let state = Decor {
            time,
            hand: vec![storm, march],
            dice: dice(&[WHITE, RED, BLUE]),
            ..Decor::default()
        }
        .state();
        let state = done(
            &state,
            strong_play(storm, 0, ManaPayment::Die(0), None, None, None),
        );
        let state = rerolled(state, 1, face);
        let action = strong_play(march, 0, ManaPayment::Die(1), None, None, None);
        assert!(legal_actions(&state, SOLO).contains(&action), "{time:?}");
    }
}

// Spec 056, L4 : sans dé de couleur de base dans la Source, pas d'effet de base.
#[test]
fn mana_storm_needs_a_basic_die() {
    let storm = advanced(ManaStorm);
    let state = Decor {
        hand: vec![storm],
        dice: dice(&[ManaColor::Gold, ManaColor::Black, ManaColor::Gold]),
        ..Decor::default()
    }
    .state();
    assert!(
        effect_plays(&state, storm)
            .iter()
            .all(|play| !matches!(play, CardPlay::Basic { .. }))
    );
}

// --- Étape 6 : Apprentissage ---

fn learning_decor(hand: Vec<CardId>) -> Decor {
    Decor {
        hand,
        advanced: vec![advanced(PathFinding), advanced(FireBolt), advanced(Agility)],
        ..Decor::default()
    }
}

fn learn(card: CardId) -> Action {
    Action::LearnAdvancedAction { card }
}

/// Apprentissage de base, Promesse et Menace : Influence 6.
fn learning_with_six_influence() -> GameState {
    six_influence_with(Vec::new())
}

/// Comme `learning_with_six_influence`, avec ces ennemis posés.
fn six_influence_with(placed: Vec<(EnemyToken, Hex)>) -> GameState {
    let learning = advanced(Learning);
    let (promise, threaten) = (card(Promise, 1), card(Threaten, 1));
    let state = Decor {
        placed,
        ..learning_decor(vec![learning, promise, threaten])
    }
    .state();
    let state = done(&state, basic(learning));
    let state = done(&state, basic(promise));
    done(&state, basic(threaten))
}

// Carte Apprentissage ; Compendium, *Learning* S1.
#[test]
fn learning_buys_advanced_action_to_discard() {
    let state = learning_with_six_influence();
    assert_eq!(state.influence_points(SOLO), 6);
    let events = events_of(&state, &learn(advanced(PathFinding)));
    assert_events_in_order(
        &events,
        &[
            Event::AdvancedActionLearned {
                player: SOLO,
                card: advanced(PathFinding),
                cost: 6,
            },
            Event::OfferCardGained {
                player: SOLO,
                offer: Offer::AdvancedActions,
                card: CommonCard::Deed(advanced(PathFinding)),
                to: CardDestination::DiscardPile,
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.influence_points(SOLO), 0);
    assert!(state.discard_pile(SOLO).contains(&advanced(PathFinding)));
    assert_eq!(state.offer(Offer::AdvancedActions).len(), 3);
}

// Carte Apprentissage renforcée : 9 Influence, la carte en main, jouable ce tour.
#[test]
fn strong_learning_to_hand() {
    let learning = advanced(Learning);
    let threaten = card(Threaten, 1);
    let decor = learning_decor(vec![learning, threaten]);
    let state = Decor {
        extra: vec![crystal(BasicColor::Red)],
        ..decor
    }
    .state();
    let state = done(
        &state,
        strong_play(learning, 0, ManaPayment::Die(WHITE_DIE), None, None, None),
    );
    let state = done(&state, strong_crystal(threaten, 0));
    assert_eq!(state.influence_points(SOLO), 9);
    let state = done(&state, learn(advanced(FireBolt)));
    assert!(state.hand(SOLO).contains(&advanced(FireBolt)));
    assert!(!effect_plays(&state, advanced(FireBolt)).is_empty());
}

// Carte Apprentissage : une fois par tour ; spec 056, Q7 : hors combat.
#[test]
fn learning_once_per_turn() {
    let learns = |state: &GameState| {
        legal_actions(state, SOLO)
            .into_iter()
            .filter(|action| matches!(action, Action::LearnAdvancedAction { .. }))
            .count()
    };
    let state = learning_with_six_influence();
    assert_eq!(learns(&state), 3);
    let after = done(&state, learn(advanced(Agility)));
    assert_eq!(learns(&after), 0);
    // 5 Influence : pas d'achat.
    let learning = advanced(Learning);
    let (promise, march) = (card(Promise, 1), card(March, 1));
    let five = learning_decor(vec![learning, promise, march]).state();
    let five = done(&five, basic(learning));
    let five = done(&five, basic(promise));
    let five = done(
        &five,
        Action::PlayCard {
            card: march,
            play: CardPlay::Sideways(arcanum_core::SidewaysEffect::Influence),
        },
    );
    assert_eq!(five.influence_points(SOLO), 5);
    assert_eq!(learns(&five), 0);
    // En combat : pas d'achat (l'Influence non dépensée est d'ailleurs perdue en l'engageant).
    let state = six_influence_with(vec![(prowlers(), ORCS_2)]);
    let state = done(&state, challenge(ORCS_2));
    let fighting = done(&state, Action::StartCombat);
    assert_eq!(learns(&fighting), 0);
}

// Spec 056, L2 : une carte hors de l'offre d'actions avancées ne s'apprend pas.
#[test]
fn learning_only_from_the_advanced_actions_offer() {
    let state = learning_with_six_influence();
    refused(&state, learn(advanced(IceBolt)));
}

// --- Étapes 7 et 8 : Effet maximum ---

/// Effet maximum joué avec l'option `option` de `target` : effet de base, ou renforcé payé par
/// `mana`.
fn maximal(maximal: CardId, target: CardId, option: u8, mana: Option<ManaPayment>) -> Action {
    let with = Some(BoostTarget {
        card: target,
        option,
        discard: None,
        set_dice: [None, None],
    });
    let play = match mana {
        None => CardPlay::Basic {
            option: 0,
            discard: None,
            mana: None,
            with,
            gain: None,
            die: None,
        },
        Some(mana) => CardPlay::Strong {
            option: 0,
            discard: None,
            mana,
            set_dice: [None, None],
            with,
            extra_mana: None,
            gain: None,
        },
    };
    Action::PlayCard {
        card: maximal,
        play,
    }
}

/// Les points de l'attaque ou de la parade en cours.
fn in_progress_points(state: &GameState) -> arcanum_core::ElementPoints {
    match state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress.clone())
    {
        Some(InProgress::Attack { points, .. } | InProgress::Block { points, .. }) => points,
        None => panic!("aucune attaque ni parade en cours"),
    }
}

/// Le combat contre les Rôdeurs, en phase d'attaque, l'attaque contre eux déclarée.
fn attacking(hand: Vec<CardId>) -> GameState {
    let state = Decor {
        hand,
        ..Decor::default()
    }
    .fighting();
    let state = skip_to_attack_phase(&state);
    done(&state, declare(&[prowlers()]))
}

// FAQ p1 ; Compendium, *Maximal Effect* S1, Q4 : une seule Attaque, triplée.
#[test]
fn maximal_effect_triples_basic_effect() {
    let effect = advanced(MaximalEffect);
    let rage = card(Rage, 1);
    let state = attacking(vec![effect, rage]);
    let state = done(&state, maximal(effect, rage, 0, None));
    assert_eq!(in_progress_points(&state), physical(6));
    assert!(!owned_anywhere(&state, rage));
    assert!(
        state
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == effect)
    );
}

// Carte Effet maximum renforcée : l'effet renforcé de la cible deux fois, sans la payer.
#[test]
fn strong_maximal_effect_doubles_strong_effect() {
    let effect = advanced(MaximalEffect);
    let march = card(March, 1);
    let state = Decor {
        hand: vec![effect, march],
        ..Decor::default()
    }
    .state();
    let events = events_of(
        &state,
        &maximal(effect, march, 0, Some(ManaPayment::Die(RED_DIE))),
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::DieTaken { .. }))
            .count(),
        1
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.move_points(SOLO), 8);
    assert!(!owned_anywhere(&state, march));
}

// FAQ p2 ; Compendium, *Maximal Effect* S5 : une Parade 10, un seul effet.
#[test]
fn maximal_effect_block_is_one_effect() {
    let effect = advanced(MaximalEffect);
    let determination = card(Determination, 1);
    let state = Decor {
        hand: vec![effect, determination],
        ..Decor::default()
    }
    .fighting();
    let state = to_block_phase(&state);
    let state = done(&state, declare_block(prowlers()));
    let state = done(
        &state,
        maximal(effect, determination, 0, Some(ManaPayment::Die(RED_DIE))),
    );
    assert_eq!(in_progress_points(&state), physical(10));
}

// FAQ p1 ; spec 056, Q2 : les gains répétés, une même option.
#[test]
fn maximal_effect_repeats_gains() {
    let effect = advanced(MaximalEffect);
    let concentration = card(Concentration, 1);
    let state = Decor {
        hand: vec![effect, concentration],
        ..Decor::default()
    }
    .state();
    let state = done(&state, maximal(effect, concentration, 0, None));
    assert_eq!(state.mana_tokens(SOLO), [BLUE, BLUE, BLUE]);
    let crystallize = card(Crystallize, 1);
    let state = Decor {
        hand: vec![effect, crystallize],
        ..Decor::default()
    }
    .state();
    // Les options de Cristallisation renforcée : rouge, bleu, blanc, vert.
    let state = done(
        &state,
        maximal(effect, crystallize, 2, Some(ManaPayment::Die(RED_DIE))),
    );
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::White), 2);
}

// Compendium, *Maximal Effect* S33 : une Blessure par répétition.
#[test]
fn maximal_effect_with_blood_rage_three_wounds() {
    let effect = advanced(MaximalEffect);
    let blood_rage = advanced(BloodRage);
    let state = attacking(vec![effect, blood_rage]);
    let state = done(&state, maximal(effect, blood_rage, 1, None));
    assert_eq!(in_progress_points(&state), physical(15));
    assert_eq!(wounds_in_hand(&state), 3);
}

// Carte Effet maximum avec Tranquillité : trois cartes piochées.
#[test]
fn maximal_effect_draws_three() {
    let effect = advanced(MaximalEffect);
    let tranquility = card(Tranquility, 1);
    let state = Decor {
        hand: vec![effect, tranquility],
        ..Decor::default()
    }
    .state();
    let deck: Vec<CardId> = state.deck(SOLO)[..3].to_vec();
    let state = done(&state, maximal(effect, tranquility, 1, None));
    assert_eq!(state.hand(SOLO), deck);
}

// Spec 056, Q1 et Q3 : seules les options répétables, ni Concentration renforcée.
#[test]
fn maximal_effect_targets() {
    let effect = advanced(MaximalEffect);
    let (improvisation, agility, concentration, rage) = (
        card(Improvisation, 1),
        advanced(Agility),
        card(Concentration, 1),
        card(Rage, 1),
    );
    let state = Decor {
        hand: vec![effect, improvisation, agility, concentration, rage],
        extra: vec![crystal(BasicColor::Red)],
        ..Decor::default()
    }
    .state();
    let targets: Vec<(CardId, bool)> = effect_plays(&state, effect)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Basic {
                with: Some(target), ..
            } => Some((target.card, false)),
            CardPlay::Strong {
                with: Some(target), ..
            } => Some((target.card, true)),
            _ => None,
        })
        .collect();
    // Concentration de base (ses jetons), et rien d'autre : Improvisation défausse, Agilité pose un
    // modificateur, Concentration renforcée est un `Boost`, Rage ne sert pas hors combat.
    assert!(targets.contains(&(concentration, false)));
    assert!(
        targets
            .iter()
            .all(|&target| target == (concentration, false)),
        "{targets:?}"
    );
    // Effet maximum n'est pas une cible de Concentration renforcée.
    let state = Decor {
        hand: vec![concentration, effect, card(March, 1)],
        ..Decor::default()
    }
    .state();
    assert!(
        effect_plays(&state, concentration)
            .iter()
            .all(|play| !matches!(
                play,
                CardPlay::Strong { with: Some(target), .. } if target.card == effect
            ))
    );
}

// Spec 056, L3 : renforcé, il faut un mana rouge ; la cible ne se paie jamais.
#[test]
fn strong_maximal_effect_needs_red_mana() {
    let effect = advanced(MaximalEffect);
    let march = card(March, 1);
    let state = Decor {
        hand: vec![effect, march],
        dice: dice(&[GREEN, BLUE, WHITE]),
        ..Decor::default()
    }
    .state();
    refused(&state, maximal(effect, march, 0, Some(ManaPayment::Die(0))));
}

// --- Étape 9 : contenu ---

// Spec 056 C32 : plus aucune action avancée non codée ; l'effet de base de Don pour la magie l'est
// depuis la spec 054.
#[test]
fn advanced_actions_all_supported() {
    let unsupported: Vec<(AdvancedAction, bool)> = AdvancedAction::ALL
        .into_iter()
        .flat_map(|card| {
            let definition = DeedCard::Advanced(card)
                .definition()
                .expect("une action avancée a une définition");
            [
                (card, false, definition.basic),
                (card, true, definition.strong),
            ]
        })
        .filter(|(_, _, effect)| *effect == BasicEffect::Unsupported)
        .map(|(card, strong, _)| (card, strong))
        .collect();
    assert_eq!(unsupported, []);
}

// Spec 056, Q1 (risque du plan) : pour chaque carte action du contenu, Effet maximum ne propose
// qu'une option répétable, sans modificateur, ni mana en plus, ni carte à défausser.
#[test]
fn maximal_effect_only_proposes_repeatable_options() {
    let effect = advanced(MaximalEffect);
    let cards = arcanum_core::BasicAction::ALL
        .map(|action| card(action, 1))
        .into_iter()
        .chain(AdvancedAction::ALL.map(advanced))
        .filter(|&other| other != effect);
    for other in cards {
        let state = Decor {
            hand: vec![effect, other],
            extra: vec![crystal(BasicColor::Red)],
            ..Decor::default()
        }
        .state();
        for play in effect_plays(&state, effect) {
            let (target, strong) = match play {
                CardPlay::Basic {
                    with: Some(target), ..
                } => (target, false),
                CardPlay::Strong {
                    with: Some(target), ..
                } => (target, true),
                _ => continue,
            };
            let definition = target.card.card().definition().expect("une carte action");
            let options = if strong {
                definition.strong
            } else {
                definition.basic
            };
            let BasicEffect::Options { cost, .. } = options else {
                panic!("{other:?} : effet non codé proposé");
            };
            let points = options.option(target.option).expect("une option");
            assert!(points.is_repeatable(), "{other:?} : {points:?}");
            assert!(
                options.option_modifiers(target.option).is_empty(),
                "{other:?}"
            );
            assert_eq!(options.option_mana(target.option), None, "{other:?}");
            assert_eq!(cost, arcanum_core::Cost::None, "{other:?}");
        }
    }
}
