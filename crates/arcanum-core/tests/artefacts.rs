//! Scénarios de la spec 055 (artefacts) : jouer un artefact pour son effet de base, le détruire pour
//! son effet renforcé, les bannières assignées aux unités, et les seize artefacts du jeu de base.

use arcanum_core::{
    Action, AdvancedAction, Artifact, ArtifactChoices, CardId, CardPlay, DeedCard, DestroyedFrom,
    Effect, EnemyColor, EnemyToken, Event, GameState, Hex, ManaColor, TimeOfDay, Unit, UnitCard, apply,
    legal_actions,
};

mod arene;
use arene::*;

use Artifact::*;
use arcanum_core::BasicAction::{Concentration, March, Rage, Stamina};

// --- Décors ---

/// Un décor de la spec 055 : le bac à sable (Arythea) de jour ou de nuit, la figurine, la main, les
/// dés de la Source, les ennemis posés, les unités déjà recrutées (dans l'ordre) et celles de
/// l'offre, puis des événements de plus (gloire, cristaux, offres…).
#[derive(Clone)]
struct Decor {
    time: TimeOfDay,
    figure: Hex,
    hand: Vec<CardId>,
    dice: Vec<ManaColor>,
    placed: Vec<(EnemyToken, Hex)>,
    owned: Vec<UnitCard>,
    offer: Vec<UnitCard>,
    extra: Vec<Event>,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            time: TimeOfDay::Day,
            figure: PLAIN_W,
            hand: Vec::new(),
            dice: Sandbox::default().dice,
            placed: vec![(prowlers(), ORCS_2)],
            owned: Vec::new(),
            offer: Vec::new(),
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
            placed: self.placed.clone(),
            dice: self.dice.clone(),
            ..Sandbox::default()
        }
        .events();
        events.extend(unit_events(&self.owned, &self.offer));
        events.extend(self.extra.iter().cloned());
        events
    }

    fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }

    /// Le décor, puis le combat engagé contre les ennemis des orques ② (phase à distance).
    #[allow(dead_code)]
    fn fighting(&self) -> GameState {
        let state = done(&self.state(), challenge(ORCS_2));
        done(&state, Action::StartCombat)
    }
}

#[allow(dead_code)]
fn unit(unit: Unit, copy: u8) -> UnitCard {
    UnitCard::new(unit, copy)
}

/// L'effet de base de l'artefact (option `option`), avec ses choix.
fn basic_with(card: CardId, option: u8, choices: ArtifactChoices) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Artifact { option, choices },
    }
}

fn basic_artifact(card: CardId) -> Action {
    basic_with(card, 0, ArtifactChoices::default())
}

/// L'effet renforcé de l'artefact (option `option`), en le détruisant, avec ses choix.
fn throw_with(card: CardId, option: u8, choices: ArtifactChoices) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::ThrowAway { option, choices },
    }
}

fn throw_away(card: CardId) -> Action {
    throw_with(card, 0, ArtifactChoices::default())
}

/// Les façons légales de jouer `card`.
fn plays_of(state: &GameState, card: CardId) -> Vec<CardPlay> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card: played, play } if played == card => Some(play),
            _ => None,
        })
        .collect()
}

fn fame(amount: u32) -> Event {
    Event::FameGained {
        player: SOLO,
        fame: amount,
    }
}

fn end_turn() -> Action {
    Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

// --- Contenu et règles communes ---

// Spec 055 C2 — MKUE p. 4, 3.c, 3.e : effet de base sans mana, effet renforcé en détruisant la
// carte, ou de côté.
#[test]
fn artifact_plays_basic_strong_or_sideways() {
    let bag = artifact(EndlessBagOfGold);
    let state = Decor {
        hand: vec![bag, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let plays = plays_of(&state, bag);
    assert!(plays.contains(&CardPlay::Artifact {
        option: 0,
        choices: ArtifactChoices::default(),
    }));
    assert!(plays.contains(&CardPlay::ThrowAway {
        option: 0,
        choices: ArtifactChoices::default(),
    }));
    assert!(
        plays
            .iter()
            .any(|play| matches!(play, CardPlay::Sideways(_)))
    );

    let events = events_of(&state, &basic_artifact(bag));
    assert_events_in_order(
        &events,
        &[
            Event::CardPlayed {
                player: SOLO,
                card: bag,
                play: CardPlay::Artifact {
                    option: 0,
                    choices: ArtifactChoices::default(),
                },
                effect: Effect::Influence(4),
            },
            fame(2),
        ],
    );
    let after = events.iter().fold(state.clone(), apply);
    assert!(
        after
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == bag)
    );
    assert_eq!(after.influence_points(SOLO), 4);

    let events = events_of(&state, &throw_away(bag));
    assert_events_in_order(
        &events,
        &[
            fame(3),
            Event::CardDestroyed {
                player: SOLO,
                card: bag,
                from: DestroyedFrom::PlayArea,
            },
        ],
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.influence_points(SOLO), 9);
    assert!(
        !after
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == bag)
    );
    assert!(!after.hand(SOLO).contains(&bag));
    assert!(!after.discard_pile(SOLO).contains(&bag));
    assert!(!after.deck(SOLO).contains(&bag));
}

// Spec 055 C3 — Compendium, *Minimum Turn* S2 : un artefact détruit compte comme une carte jouée.
#[test]
fn thrown_away_artifact_counts_as_played() {
    let bag = artifact(EndlessBagOfGold);
    let state = Decor {
        hand: vec![bag, card(March, 1), card(Rage, 1)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, throw_away(bag));
    assert!(legal_actions(&state, SOLO).contains(&end_turn()));
}

// Spec 055 C4 — Compendium, *Endless Bag of Gold* Q1 ; spec 035 : la gloire aussitôt, le niveau à
// la fin du tour.
#[test]
fn artifact_fame_is_immediate_level_at_end_of_turn() {
    let bag = artifact(EndlessBagOfGold);
    let mut decor = Decor {
        hand: vec![bag, card(March, 1)],
        ..Decor::default()
    };
    decor.extra = progress(1, 1);
    let state = done(&decor.state(), basic_artifact(bag));
    assert_eq!(state.fame(SOLO), 3);
    assert_eq!(state.level(SOLO), 1);
    let state = done(&state, end_turn());
    assert_eq!(state.level(SOLO), 2);
}

// Spec 055 C5 — Compendium, *Tome of All Spells* S4 : un artefact n'est ni une carte action ni une
// carte de couleur.
#[test]
fn artifacts_are_not_boosted_nor_colored() {
    let ring = artifact(RubyRing);
    let bag = artifact(EndlessBagOfGold);
    let concentration = card(Concentration, 1);
    let state = Decor {
        hand: vec![concentration, ring, bag, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let targets: Vec<CardId> = plays_of(&state, concentration)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Strong {
                with: Some(target), ..
            } => Some(target.card),
            _ => None,
        })
        .collect();
    assert!(!targets.is_empty());
    assert!(!targets.contains(&ring) && !targets.contains(&bag));

    let talent = advanced(AdvancedAction::MagicTalent);
    let mut decor = Decor {
        hand: vec![talent, ring, bag, card(Rage, 1)],
        ..Decor::default()
    };
    decor.extra = spell_offer(&[CardId::new(
        DeedCard::Spell(arcanum_core::Spell::Fireball),
        1,
    )]);
    let state = decor.state();
    let discards: Vec<CardId> = plays_of(&state, talent)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Basic {
                discard: Some(discard),
                ..
            } => Some(discard),
            _ => None,
        })
        .collect();
    assert_eq!(discards, vec![card(Rage, 1)]);
}

// --- Bannières ---

fn peasants() -> UnitCard {
    unit(Unit::Peasants, 1)
}

fn guardsmen() -> UnitCard {
    unit(Unit::UtemGuardsmen, 1)
}

fn assign_banner(card: CardId, unit: UnitCard) -> Action {
    Action::AssignBanner { card, unit }
}

/// L'unité épuisée (activée pour rien).
fn exhaust(unit: UnitCard) -> Event {
    Event::UnitActivated {
        player: SOLO,
        unit,
        ability: 0,
        option: 0,
        effect: Effect::Nothing,
        enemy: None,
    }
}

fn wound_unit(unit: UnitCard) -> Event {
    Event::UnitWounded {
        player: SOLO,
        unit,
        wounds: 1,
    }
}

fn owned(state: &GameState, of: UnitCard) -> arcanum_core::OwnedUnit {
    *state
        .units(SOLO)
        .iter()
        .find(|owned| owned.card == of)
        .unwrap_or_else(|| panic!("{of:?} absente"))
}

// Spec 055 C6 — MKUE p. 5, 8 ; Compendium, *Minimum Turn* S2 : une bannière s'assigne à toute unité,
// même en combat ; c'est une carte jouée.
#[test]
fn banner_assigned_to_any_unit() {
    let glory = artifact(BannerOfGlory);
    let decor = Decor {
        hand: vec![glory, card(March, 1)],
        owned: vec![peasants()],
        extra: vec![exhaust(peasants()), wound_unit(peasants())],
        ..Decor::default()
    };
    assert!(legal_actions(&decor.fighting(), SOLO).contains(&assign_banner(glory, peasants())));
    let state = decor.state();
    let events = events_of(&state, &assign_banner(glory, peasants()));
    assert_eq!(
        events,
        vec![Event::BannerAssigned {
            player: SOLO,
            card: glory,
            unit: peasants(),
        }]
    );
    let state = events.iter().fold(state, apply);
    assert!(!state.hand(SOLO).contains(&glory));
    assert_eq!(owned(&state, peasants()).banner, Some(glory));
    assert!(legal_actions(&state, SOLO).contains(&end_turn()));
}

// Spec 055 C7 — MKUE p. 5, 8.a : une seconde bannière défausse la première.
#[test]
fn second_banner_discards_the_first() {
    let glory = artifact(BannerOfGlory);
    let protection = artifact(BannerOfProtection);
    let state = Decor {
        hand: vec![glory, protection, card(March, 1)],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .state();
    let state = done(&state, assign_banner(glory, peasants()));
    let events = events_of(&state, &assign_banner(protection, peasants()));
    assert_events_in_order(
        &events,
        &[
            Event::BannerLost {
                player: SOLO,
                card: glory,
                unit: peasants(),
            },
            Event::BannerAssigned {
                player: SOLO,
                card: protection,
                unit: peasants(),
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(owned(&state, peasants()).banner, Some(protection));
    assert!(state.discard_pile(SOLO).contains(&glory));
}

// Spec 055 C8 — MKUE p. 5, 8.a : l'unité détruite ou renvoyée, sa bannière va dans la défausse.
#[test]
fn banner_discarded_with_its_unit() {
    let glory = artifact(BannerOfGlory);
    let lost = Event::BannerLost {
        player: SOLO,
        card: glory,
        unit: peasants(),
    };
    // Détruits : les dégâts de la Méduse (Paralysant).
    let decor = Decor {
        hand: vec![glory, card(March, 1)],
        owned: vec![peasants()],
        placed: vec![(medusa(), ORCS_2)],
        ..Decor::default()
    };
    let state = done(&decor.fighting(), assign_banner(glory, peasants()));
    let state = done(&to_block_phase(&state), Action::NextCombatPhase);
    let assign = Action::AssignDamage {
        enemy: medusa(),
        to: arcanum_core::DamageTarget::Unit(peasants()),
    };
    let events = events_of(&state, &assign);
    assert_events_in_order(
        &events,
        &[
            lost.clone(),
            Event::UnitDestroyed {
                player: SOLO,
                unit: peasants(),
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert!(state.discard_pile(SOLO).contains(&glory));

    // Renvoyés pour recruter les Gardes d'Utem au village.
    let state = Decor {
        figure: VILLAGE,
        hand: vec![glory, card(March, 1), card(Rage, 1)],
        owned: vec![peasants()],
        offer: vec![guardsmen()],
        ..Decor::default()
    }
    .state();
    let state = done(&state, assign_banner(glory, peasants()));
    let state = apply(
        state,
        &Event::InteractionStarted {
            player: SOLO,
            site: arcanum_core::Site::Village,
            bonus: 0,
            city_shields: 0,
        },
    );
    let state = apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: card(Rage, 1),
            play: CardPlay::Sideways(arcanum_core::SidewaysEffect::Influence),
            effect: Effect::Influence(9),
        },
    );
    let recruit = Action::RecruitUnit {
        unit: guardsmen(),
        disband: Some(peasants()),
        bonds: false,
    };
    let events = events_of(&state, &recruit);
    assert_events_in_order(
        &events,
        &[
            lost,
            Event::UnitDisbanded {
                player: SOLO,
                unit: peasants(),
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert!(state.discard_pile(SOLO).contains(&glory));
}

// Spec 055 C10 — MKUE p. 15, *The Greatest Loot* : une bannière sur une unité compte, un artefact
// détruit non.
#[test]
fn loot_counts_banners_on_units() {
    let glory = artifact(BannerOfGlory);
    let bag = artifact(EndlessBagOfGold);
    let state = Decor {
        hand: vec![glory, bag, card(March, 1)],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .state();
    let state = done(&state, assign_banner(glory, peasants()));
    let state = done(&state, throw_away(bag));
    assert_eq!(arcanum_core::final_score(&state, SOLO).artifacts, 1);
}

// Spec 055 C11, L1 — Compendium, *Artifact Cards* S1 : l'effet de base d'une bannière ne se joue que
// par `AssignBanner` ; une fois assignée, plus aucune façon de la jouer ; refus sans unité, avec
// une carte qui n'est pas une bannière ou absente de la main.
#[test]
fn banner_strong_only_from_hand() {
    let courage = artifact(BannerOfCourage);
    let bag = artifact(EndlessBagOfGold);
    let decor = Decor {
        hand: vec![courage, bag, card(March, 1)],
        ..Decor::default()
    };
    let state = decor.state();
    assert!(
        !plays_of(&state, courage)
            .iter()
            .any(|play| matches!(play, CardPlay::Artifact { .. }))
    );
    refused(&state, assign_banner(courage, peasants()));
    let state = Decor {
        owned: vec![peasants()],
        ..decor
    }
    .state();
    refused(&state, assign_banner(bag, peasants()));
    refused(&state, assign_banner(artifact(BannerOfFear), peasants()));
    let state = done(&state, assign_banner(courage, peasants()));
    assert!(plays_of(&state, courage).is_empty());
}

/// Le décor d'une fin de manche : le deck vide (la fin de manche s'annonce), Marche et `banners` en
/// main, Paysans et Gardes d'Utem recrutés.
fn round_end_decor(banners: &[CardId]) -> GameState {
    let mut hand = vec![card(March, 1)];
    hand.extend_from_slice(banners);
    let mut extra = vec![Event::DeckShuffled {
        player: SOLO,
        order: hand.clone(),
    }];
    extra.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    Decor {
        hand: hand.clone(),
        owned: vec![peasants(), guardsmen()],
        extra,
        ..Decor::default()
    }
    .state()
}

fn reclaim(banners: &[CardId]) -> Action {
    Action::ReclaimBanners {
        banners: banners.to_vec(),
    }
}

// Spec 055 C9, L5 — MKUE p. 4, 2.g ; p. 5, 8.b ; Q2 : à la fin de la manche, le choix des bannières
// à reprendre ; une reprise rejoint le deck mélangé, l'autre reste sur son unité.
#[test]
fn round_end_reclaims_chosen_banners() {
    let glory = artifact(BannerOfGlory);
    let courage = artifact(BannerOfCourage);
    let state = round_end_decor(&[glory, courage]);
    refused(&state, reclaim(&[]));
    let state = done(&state, assign_banner(glory, peasants()));
    let state = done(&state, assign_banner(courage, guardsmen()));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_eq!(events, vec![Event::RoundEnded { round: 1 }]);
    let state = events.iter().fold(state, apply);
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![
            reclaim(&[]),
            reclaim(&[glory]),
            reclaim(&[courage]),
            reclaim(&[glory, courage]),
        ]
    );
    let events = events_of(&state, &reclaim(&[glory]));
    assert_eq!(
        events.first(),
        Some(&Event::BannersReclaimed {
            player: SOLO,
            banners: vec![glory],
        })
    );
    let shuffled = events.iter().find_map(|event| match event {
        Event::DeckShuffled { order, .. } => Some(order.clone()),
        _ => None,
    });
    assert!(shuffled.is_some_and(|order| order.contains(&glory) && !order.contains(&courage)));
    let state = events.iter().fold(state, apply);
    assert_eq!(state.round(), 2);
    assert_eq!(owned(&state, peasants()).banner, None);
    assert_eq!(owned(&state, guardsmen()).banner, Some(courage));

    // Sans bannière assignée, la manche suivante commence aussitôt.
    let state = round_end_decor(&[]);
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::RoundStarted { .. }))
    );
}

// --- Amulettes ---

fn token_gained(color: ManaColor) -> Event {
    Event::ManaTokenGained {
        player: SOLO,
        color,
    }
}

fn has_modifier(state: &GameState, kind: arcanum_core::ModifierKind) -> bool {
    state
        .modifiers(SOLO)
        .iter()
        .any(|modifier| modifier.kind == kind)
}

/// L'effet renforcé de Endurance (Mouvement 4, bleu) payé par `mana`.
fn strong_stamina(mana: arcanum_core::ManaPayment) -> Action {
    Action::PlayCard {
        card: card(Stamina, 1),
        play: CardPlay::Strong {
            option: 0,
            discard: None,
            mana,
            set_dice: [None, None],
            with: None,
            extra_mana: None,
            gain: None,
        },
    }
}

/// La Source des tests d'amulettes : vert, or, noir.
fn gold_and_black_dice() -> Vec<ManaColor> {
    vec![
        ManaColor::Basic(arcanum_core::BasicColor::Green),
        ManaColor::Gold,
        ManaColor::Black,
    ]
}

const GOLD_DIE: u8 = 1;
const BLACK_DIE: u8 = 2;

fn darkness_colors(colors: &[ManaColor]) -> ArtifactChoices {
    ArtifactChoices {
        colors: arcanum_core::Picks::of(colors),
        ..ArtifactChoices::default()
    }
}

// Spec 055 C39 — carte ; Compendium, *Amulet of the Sun* S1 ; Q6 : de nuit, un jeton or utilisable,
// forêts à 3, dé or de la Source, ruines face cachée retournées.
#[test]
fn amulet_of_sun_day_rules_at_night() {
    let sun = artifact(AmuletOfSun);
    let ruins = Hex::new(1, -1);
    // Le jeton du dessus de la pile de jetons de ruines du bac à sable.
    let ruin = arcanum_core::GameContent::embedded().ruin_tokens()[0];
    let mut decor = Decor {
        time: TimeOfDay::Night,
        figure: Hex::ORIGIN,
        hand: vec![sun, card(Stamina, 1), card(March, 1)],
        dice: gold_and_black_dice(),
        ..Decor::default()
    };
    decor.extra = vec![Event::RuinTokenPlaced {
        token: ruin,
        hex: ruins,
        face_up: false,
    }];
    let state = decor.state();
    assert_eq!(
        state
            .move_rules(SOLO)
            .terrain_cost(arcanum_core::Terrain::Forest),
        Some(5)
    );
    let events = events_of(&state, &basic_artifact(sun));
    assert_events_in_order(
        &events,
        &[
            token_gained(ManaColor::Gold),
            Event::RuinTokenRevealed {
                hex: ruins,
                token: ruin,
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert!(has_modifier(&state, arcanum_core::ModifierKind::DayRules));
    assert_eq!(
        state
            .move_rules(SOLO)
            .terrain_cost(arcanum_core::Terrain::Forest),
        Some(3)
    );
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&strong_stamina(arcanum_core::ManaPayment::Token)));
    assert!(legal.contains(&strong_stamina(arcanum_core::ManaPayment::Die(GOLD_DIE))));
}

// Spec 055 C40 — carte ; Compendium, *Amulet of Darkness* S1 ; Q10 : de jour, un jeton noir choisi,
// déserts à 3, sort renforcé, dé noir de la Source ; l'or sert encore.
#[test]
fn amulet_of_darkness_night_mana_by_day() {
    let darkness = artifact(AmuletOfDarkness);
    let meditation = CardId::new(DeedCard::Spell(arcanum_core::Spell::Meditation), 1);
    let mut decor = Decor {
        hand: vec![darkness, meditation, card(Stamina, 1), card(March, 1)],
        dice: gold_and_black_dice(),
        ..Decor::default()
    };
    decor.extra = vec![Event::CrystalGained {
        player: SOLO,
        color: arcanum_core::BasicColor::Green,
    }];
    let state = decor.state();
    let black = basic_with(darkness, 0, darkness_colors(&[ManaColor::Black]));
    assert!(legal_actions(&state, SOLO).contains(&black));
    let events = events_of(&state, &black);
    assert_events_in_order(&events, &[token_gained(ManaColor::Black)]);
    let state = events.iter().fold(state, apply);
    assert!(has_modifier(&state, arcanum_core::ModifierKind::NightMana));
    assert_eq!(
        state
            .move_rules(SOLO)
            .terrain_cost(arcanum_core::Terrain::Desert),
        Some(3)
    );
    let casts: Vec<Option<arcanum_core::ManaPayment>> = plays_of(&state, meditation)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Cast { black, .. } => black,
            _ => None,
        })
        .map(Some)
        .collect();
    assert!(casts.contains(&Some(arcanum_core::ManaPayment::Token)));
    assert!(casts.contains(&Some(arcanum_core::ManaPayment::Die(BLACK_DIE))));
    assert!(
        legal_actions(&state, SOLO)
            .contains(&strong_stamina(arcanum_core::ManaPayment::Die(GOLD_DIE)))
    );
}

// Spec 055 C41, L6 — cartes : trois jetons de couleurs choisies ; l'Amulette du soleil de jour, trois
// jetons or sans règles de jour ; celle des ténèbres de nuit, sans `NightMana`.
#[test]
fn amulets_strong_three_tokens() {
    let darkness = artifact(AmuletOfDarkness);
    let sun = artifact(AmuletOfSun);
    let state = Decor {
        hand: vec![darkness, sun, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let colors = [
        ManaColor::Basic(arcanum_core::BasicColor::Red),
        ManaColor::Gold,
        ManaColor::Black,
    ];
    let play = throw_with(darkness, 0, darkness_colors(&colors));
    assert!(legal_actions(&state, SOLO).contains(&play));
    let events = events_of(&state, &play);
    let tokens: Vec<Event> = events
        .iter()
        .filter(|event| matches!(event, Event::ManaTokenGained { .. }))
        .cloned()
        .collect();
    assert_eq!(tokens, colors.map(token_gained).to_vec());

    let state = done(&state, throw_away(sun));
    assert_eq!(
        state
            .mana_tokens(SOLO)
            .iter()
            .filter(|&&token| token == ManaColor::Gold)
            .count(),
        3
    );
    assert!(!has_modifier(&state, arcanum_core::ModifierKind::DayRules));

    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![darkness, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        basic_with(darkness, 0, darkness_colors(&[ManaColor::Black])),
    );
    assert!(!has_modifier(&state, arcanum_core::ModifierKind::NightMana));
}

// --- Anneaux ---

fn spell_card(card: arcanum_core::Spell) -> CardId {
    CardId::new(DeedCard::Spell(card), 1)
}

fn cast(
    card: CardId,
    option: u8,
    mana: arcanum_core::ManaPayment,
    black: Option<arcanum_core::ManaPayment>,
) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana: None,
            enemy: None,
            from_offer: false,
            targets: arcanum_core::CastTargets::default(),
        },
    }
}

fn crystal(color: arcanum_core::BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

const RED: arcanum_core::BasicColor = arcanum_core::BasicColor::Red;

// Spec 055 C24 — carte ; MKUE p. 5, « Gain » 2 : un jeton, un cristal (un jeton si l'inventaire est
// plein), Gloire +1.
#[test]
fn ring_token_crystal_and_fame() {
    let ruby = artifact(RubyRing);
    let decor = Decor {
        hand: vec![ruby, card(March, 1)],
        ..Decor::default()
    };
    let events = events_of(&decor.state(), &basic_artifact(ruby));
    assert_events_in_order(
        &events,
        &[token_gained(ManaColor::Basic(RED)), crystal(RED), fame(1)],
    );
    let full = Decor {
        extra: vec![crystal(RED), crystal(RED), crystal(RED)],
        ..decor
    };
    let events = events_of(&full.state(), &basic_artifact(ruby));
    let tokens = events
        .iter()
        .filter(|event| **event == token_gained(ManaColor::Basic(RED)))
        .count();
    assert_eq!(tokens, 2);
    assert!(!events.contains(&crystal(RED)));
}

// Spec 055 C25 — carte ; Q3 : mana rouge et noir illimité, Gloire +1 par sort rouge lancé ce tour,
// avant comme après l'anneau.
#[test]
fn ruby_ring_endless_mana_and_fame() {
    let ruby = artifact(RubyRing);
    let fireball = spell_card(arcanum_core::Spell::Fireball);
    let flame_wall = spell_card(arcanum_core::Spell::FlameWall);
    let mut decor = Decor {
        time: TimeOfDay::Night,
        hand: vec![ruby, fireball, flame_wall, card(March, 1)],
        placed: vec![(fire_dragon(), ORCS_2)],
        ..Decor::default()
    };
    decor.extra = vec![token_gained(ManaColor::Basic(RED))];
    let state = decor.fighting();
    let state = done(&state, declare(&[fire_dragon()]));
    let state = done(
        &state,
        cast(fireball, 0, arcanum_core::ManaPayment::Token, None),
    );
    let events = events_of(&state, &throw_away(ruby));
    assert_events_in_order(&events, &[fame(1)]);
    let state = events.iter().fold(state, apply);
    let state = done(&state, Action::ResolveAttack);
    let state = skip_to_attack_phase(&state);
    let state = done(&state, declare(&[fire_dragon()]));
    let endless = cast(
        flame_wall,
        0,
        arcanum_core::ManaPayment::Endless(ManaColor::Basic(RED)),
        Some(arcanum_core::ManaPayment::Endless(ManaColor::Black)),
    );
    assert!(legal_actions(&state, SOLO).contains(&endless));
    let events = events_of(&state, &endless);
    assert!(events.contains(&fame(1)));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::DieTaken { .. } | Event::ManaTokenUsed { .. } | Event::CrystalUsed { .. }
    )));
}

// Spec 055 C26 — Compendium, *Rings* S1 : le noir illimité seulement là où le noir sert.
#[test]
fn ring_black_mana_follows_night_rules() {
    let sapphire = artifact(SapphireRing);
    let darkness = artifact(AmuletOfDarkness);
    let meditation = spell_card(arcanum_core::Spell::Meditation);
    let state = Decor {
        hand: vec![sapphire, darkness, meditation, card(March, 1)],
        extra: vec![crystal(arcanum_core::BasicColor::Green)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, throw_away(sapphire));
    let endless_black = |state: &GameState| {
        plays_of(state, meditation).into_iter().any(|play| {
            matches!(
                play,
                CardPlay::Cast {
                    black: Some(arcanum_core::ManaPayment::Endless(ManaColor::Black)),
                    ..
                }
            )
        })
    };
    assert!(!endless_black(&state));
    let state = done(
        &state,
        basic_with(darkness, 0, darkness_colors(&[ManaColor::Gold])),
    );
    assert!(endless_black(&state));
}

// --- Bannière de gloire ---

fn activate(of: UnitCard, ability: u8, option: u8) -> Action {
    Action::ActivateUnit {
        unit: of,
        ability,
        option,
        mana: Vec::new(),
        target: None,
        enemy: None,
        leadership: false,
    }
}

fn in_progress_points(state: &GameState) -> arcanum_core::ElementPoints {
    match state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress.clone())
    {
        Some(
            arcanum_core::InProgress::Attack { points, .. }
            | arcanum_core::InProgress::Block { points, .. },
        ) => points,
        None => panic!("ni attaque ni parade en cours"),
    }
}

fn fame_gained(events: &[Event]) -> u32 {
    events
        .iter()
        .map(|event| match event {
            Event::FameGained { fame, .. } => *fame,
            _ => 0,
        })
        .sum()
}

// Spec 055 C12 — carte ; Compendium, *Miscellaneous* S12 : +1 à la parade de l'unité porteuse,
// Gloire +1 même si la parade ne suffit pas ; pas de +1 à l'Influence.
#[test]
fn banner_of_glory_bonus_and_fame() {
    let glory = artifact(BannerOfGlory);
    let decor = Decor {
        hand: vec![glory, card(March, 1)],
        owned: vec![peasants()],
        placed: vec![(ironclads(), ORCS_2)],
        ..Decor::default()
    };
    let state = done(&decor.fighting(), assign_banner(glory, peasants()));
    let state = done(&to_block_phase(&state), declare_block(ironclads()));
    let events = events_of(&state, &activate(peasants(), 0, 1));
    assert_eq!(fame_gained(&events), 1);
    let state = events.iter().fold(state, apply);
    assert_eq!(in_progress_points(&state), physical(3));

    // L'Influence au village : 2, sans gloire.
    let state = Decor {
        figure: VILLAGE,
        placed: Vec::new(),
        ..decor
    }
    .state();
    let state = done(&state, assign_banner(glory, peasants()));
    let state = apply(
        state,
        &Event::InteractionStarted {
            player: SOLO,
            site: arcanum_core::Site::Village,
            bonus: 0,
            city_shields: 0,
        },
    );
    let before = state.influence_points(SOLO);
    let events = events_of(&state, &activate(peasants(), 1, 0));
    assert_eq!(fame_gained(&events), 0);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.influence_points(SOLO) - before, 2);
}

// Spec 055 C13 — carte ; spec 042 : Armure +1 pour l'unité porteuse.
#[test]
fn banner_of_glory_armor() {
    let glory = artifact(BannerOfGlory);
    let decor = Decor {
        hand: vec![glory, card(March, 1)],
        owned: vec![peasants()],
        ..Decor::default()
    };
    let assign = Action::AssignDamage {
        enemy: prowlers(),
        to: arcanum_core::DamageTarget::Unit(peasants()),
    };
    let wounds_to_hero = |state: &GameState| {
        let state = done(&to_block_phase(state), Action::NextCombatPhase);
        events_of(&state, &assign)
            .iter()
            .any(|event| matches!(event, Event::WoundsTaken { .. }))
    };
    assert!(wounds_to_hero(&decor.fighting()));
    let state = done(&decor.fighting(), assign_banner(glory, peasants()));
    assert!(!wounds_to_hero(&state));
}

// Spec 055 C14 — carte ; Q3 : renforcée, +1 aux unités et Gloire +1 par unité qui attaque ou pare
// ce tour, avant comme après la bannière.
#[test]
fn banner_of_glory_strong_fame_per_unit() {
    let glory = artifact(BannerOfGlory);
    let decor = Decor {
        hand: vec![glory, card(March, 1)],
        owned: vec![peasants(), guardsmen()],
        ..Decor::default()
    };
    let block = to_block_phase(&decor.fighting());
    let state = done(&block, throw_away(glory));
    let state = done(&state, declare_block(prowlers()));
    let events = events_of(&state, &activate(guardsmen(), 1, 0));
    assert_eq!(fame_gained(&events), 1);
    let state = events.iter().fold(state, apply);
    assert_eq!(in_progress_points(&state), physical(5));
    let state = done(&state, Action::ResolveBlock);
    let state = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    let events = events_of(&state, &activate(peasants(), 0, 0));
    assert_eq!(fame_gained(&events), 1);
    let state = events.iter().fold(state, apply);
    assert_eq!(in_progress_points(&state), physical(3));

    // Une unité activée avant la bannière compte aussi, à la bannière.
    let state = done(&block, declare_block(prowlers()));
    let state = done(&state, activate(guardsmen(), 1, 0));
    let events = events_of(&state, &throw_away(glory));
    assert_eq!(fame_gained(&events), 1);
}

// --- Bannières de terreur et de courage ---

fn use_banner(unit: UnitCard, enemy: Option<EnemyToken>) -> Action {
    Action::UseBanner { unit, enemy }
}

fn swordsmen() -> EnemyToken {
    EnemyToken::new(arcanum_core::Enemy::Swordsmen, 1)
}

fn banner_uses(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::UseBanner { .. }))
        .collect()
}

fn does_not_attack(state: &GameState, enemy: EnemyToken) -> bool {
    state.combat(SOLO).is_some_and(|combat| {
        combat
            .enemies
            .iter()
            .any(|e| e.token == enemy && e.does_not_attack())
    })
}

fn readied(unit: UnitCard) -> Event {
    Event::UnitReadied { player: SOLO, unit }
}

// Spec 055 C15 — carte (Lost Legion) ; Compendium, *Banner of Fear* S3 : en phase de parade,
// l'unité porteuse épuisée annule une attaque, Gloire +1.
#[test]
fn banner_of_fear_cancels_attack() {
    let fear = artifact(BannerOfFear);
    let decor = Decor {
        hand: vec![fear, card(March, 1)],
        owned: vec![peasants()],
        placed: vec![(swordsmen(), ORCS_2)],
        ..Decor::default()
    };
    let state = done(&decor.fighting(), assign_banner(fear, peasants()));
    let state = to_block_phase(&state);
    let action = use_banner(peasants(), Some(swordsmen()));
    assert_eq!(banner_uses(&state), vec![action.clone()]);
    let events = events_of(&state, &action);
    assert_events_in_order(
        &events,
        &[
            Event::BannerUsed {
                player: SOLO,
                unit: peasants(),
                enemy: Some(swordsmen()),
            },
            fame(1),
        ],
    );
    let state = events.iter().fold(state, apply);
    assert!(!owned(&state, peasants()).ready);
    assert!(does_not_attack(&state, swordsmen()));
    let events = events_of(&state, &Action::NextCombatPhase);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::WoundsTaken { .. })),
        "{events:?}"
    );
}

// Spec 055 C16 — Compendium, *Banner of Fear* S4 : unité disponible et non blessée, en phase de
// parade, contre un ennemi pas paré.
#[test]
fn banner_of_fear_needs_ready_unwounded_unit() {
    let fear = artifact(BannerOfFear);
    let decor = Decor {
        hand: vec![fear, card(March, 1)],
        owned: vec![peasants(), guardsmen()],
        ..Decor::default()
    };
    let ranged = done(&decor.fighting(), assign_banner(fear, peasants()));
    assert!(banner_uses(&ranged).is_empty());
    let block = to_block_phase(&ranged);
    assert!(!banner_uses(&block).is_empty());
    assert!(banner_uses(&apply(block.clone(), &exhaust(peasants()))).is_empty());
    assert!(banner_uses(&apply(block.clone(), &wound_unit(peasants()))).is_empty());
    // Prowlers parés par les Gardes d'Utem (Parade rapide 4).
    let state = done(&block, declare_block(prowlers()));
    let state = done(
        &state,
        Action::ActivateUnit {
            unit: guardsmen(),
            ability: 1,
            option: 0,
            mana: Vec::new(),
            target: None,
            enemy: None,
            leadership: false,
        },
    );
    let state = done(&state, Action::ResolveBlock);
    assert!(banner_uses(&state).is_empty());
    let attack = skip_to_attack_phase(&block);
    assert!(banner_uses(&attack).is_empty());
}

fn summoned_by(state: &GameState, summoner: EnemyToken) -> Option<EnemyToken> {
    state.combat(SOLO).and_then(|combat| {
        combat
            .enemies
            .iter()
            .find(|enemy| enemy.token == summoner)
            .and_then(|enemy| enemy.summoned)
    })
}

/// Un combat contre des Orc Summoners, les Paysans portant la Bannière de terreur.
fn fear_against_summoners() -> GameState {
    let fear = artifact(BannerOfFear);
    let decor = Decor {
        hand: vec![fear, card(March, 1)],
        owned: vec![peasants()],
        placed: vec![(orc_summoners(), ORCS_2)],
        ..Decor::default()
    };
    done(&decor.fighting(), assign_banner(fear, peasants()))
}

// Spec 055 C17 ; issue #8 — MKUE p. 9, *Block Phase* 2 ; Compendium, *Banner of Fear* Q5,
// *Combat* S23 : au début de la phase de parade, l'invocation attend le choix de la Bannière de
// terreur. Annulée, aucune créature n'est tirée, Gloire +1.
#[test]
fn banner_of_fear_cancels_summoning_before_draw() {
    let state = to_block_phase(&fear_against_summoners());
    assert_eq!(summoned_by(&state, orc_summoners()), None);
    let cancel = use_banner(peasants(), Some(orc_summoners()));
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![cancel.clone(), Action::CallSummons]
    );
    let brown = state.enemy_pile(EnemyColor::Brown).len();
    let fame_before = state.fame(SOLO);
    let state = done(&state, cancel);
    assert!(does_not_attack(&state, orc_summoners()));
    assert_eq!(summoned_by(&state, orc_summoners()), None);
    assert_eq!(state.enemy_pile(EnemyColor::Brown).len(), brown);
    assert_eq!(state.fame(SOLO), fame_before + 1);
    // Plus de bannière à jouer : la phase de parade reprend son cours.
    assert!(legal_actions(&state, SOLO).contains(&Action::NextCombatPhase));
    assert!(!legal_actions(&state, SOLO).contains(&Action::CallSummons));
}

// Spec 055 C17 ; issue #8 — Compendium, *Banner of Fear* Q6, *Combat* S23 : l'invocation
// laissée faire, la créature tirée n'est pas une cible de la bannière.
#[test]
fn banner_of_fear_not_against_summoned_creature() {
    let state = to_block_phase(&fear_against_summoners());
    let state = done(&state, Action::CallSummons);
    assert!(summoned_by(&state, orc_summoners()).is_some());
    assert!(banner_uses(&state).is_empty());
    assert!(!legal_actions(&state, SOLO).contains(&Action::CallSummons));
}

// Issue #8 — MKUE p. 9, *Block Phase* 2 : sans Bannière de terreur utilisable (unité blessée),
// la créature est tirée aussitôt, comme avant.
#[test]
fn summoning_not_held_without_usable_banner_of_fear() {
    let state = apply(fear_against_summoners(), &wound_unit(peasants()));
    let state = to_block_phase(&state);
    assert!(summoned_by(&state, orc_summoners()).is_some());
    assert!(!legal_actions(&state, SOLO).contains(&Action::CallSummons));
}

// Spec 055 C18, L3 — carte (Lost Legion) : renforcée, jusqu'à 3 ennemis n'attaquent pas.
#[test]
fn banner_of_fear_strong_three_enemies() {
    let fear = artifact(BannerOfFear);
    let state = Decor {
        hand: vec![fear, card(March, 1)],
        placed: vec![
            (prowlers(), ORCS_2),
            (swordsmen(), ORCS_2),
            (wolf_riders(), ORCS_2),
            (diggers(), ORCS_2),
        ],
        ..Decor::default()
    }
    .fighting();
    let three = [prowlers(), swordsmen(), diggers()];
    let enemies = |targets: &[EnemyToken]| ArtifactChoices {
        enemies: arcanum_core::Picks::of(targets),
        ..ArtifactChoices::default()
    };
    let after = done(&state, throw_with(fear, 0, enemies(&three)));
    for enemy in three {
        assert!(does_not_attack(&after, enemy), "{enemy:?}");
    }
    assert!(!does_not_attack(&after, wolf_riders()));
    refused(
        &state,
        throw_with(
            fear,
            0,
            enemies(&[prowlers(), swordsmen(), wolf_riders(), diggers()]),
        ),
    );
    refused(
        &state,
        throw_with(fear, 0, enemies(&[prowlers(), prowlers()])),
    );
    // L3 : sans ennemi qui attaque encore, pas proposée.
    let attack = skip_to_attack_phase(&state);
    assert!(
        !plays_of(&attack, fear)
            .iter()
            .any(|play| matches!(play, CardPlay::ThrowAway { .. }))
    );
}

// Spec 055 L2 — une unité sans bannière, ou dont la bannière n'a pas de capacité : refusé.
#[test]
fn banner_use_needs_a_banner_with_ability() {
    let glory = artifact(BannerOfGlory);
    let state = Decor {
        hand: vec![glory, card(March, 1)],
        owned: vec![peasants(), guardsmen()],
        ..Decor::default()
    }
    .fighting();
    let state = to_block_phase(&done(&state, assign_banner(glory, peasants())));
    refused(&state, use_banner(peasants(), Some(prowlers())));
    refused(&state, use_banner(guardsmen(), Some(prowlers())));
}

// Spec 055 C22 — carte ; Compendium, *Banner of Courage* S1 : hors combat, une fois par manche,
// rendre disponible l'unité porteuse, blessée ou non.
#[test]
fn banner_of_courage_once_per_round() {
    let courage = artifact(BannerOfCourage);
    let decor = Decor {
        hand: vec![courage, card(March, 1)],
        owned: vec![peasants()],
        extra: vec![exhaust(peasants()), wound_unit(peasants())],
        ..Decor::default()
    };
    let state = done(&decor.state(), assign_banner(courage, peasants()));
    let action = use_banner(peasants(), None);
    assert_eq!(banner_uses(&state), vec![action.clone()]);
    let events = events_of(&state, &action);
    assert_events_in_order(
        &events,
        &[
            Event::BannerUsed {
                player: SOLO,
                unit: peasants(),
                enemy: None,
            },
            readied(peasants()),
        ],
    );
    let state = events.iter().fold(state, apply);
    let owned_now = owned(&state, peasants());
    assert!(owned_now.ready && owned_now.is_wounded() && owned_now.banner_flipped);
    assert!(banner_uses(&apply(state, &exhaust(peasants()))).is_empty());
    // En combat, jamais.
    let fighting = done(&decor.fighting(), assign_banner(courage, peasants()));
    assert!(banner_uses(&fighting).is_empty());
}

// Spec 055 C23 — carte : renforcée, hors combat, toutes les unités épuisées redeviennent
// disponibles.
#[test]
fn banner_of_courage_strong_readies_all() {
    let courage = artifact(BannerOfCourage);
    let third = unit(Unit::Peasants, 2);
    let decor = Decor {
        hand: vec![courage, card(March, 1)],
        owned: vec![peasants(), guardsmen(), third],
        extra: vec![exhaust(peasants()), exhaust(guardsmen()), exhaust(third)],
        ..Decor::default()
    };
    let events = events_of(&decor.state(), &throw_away(courage));
    assert_events_in_order(
        &events,
        &[readied(peasants()), readied(guardsmen()), readied(third)],
    );
    assert!(
        !plays_of(&decor.fighting(), courage)
            .iter()
            .any(|play| matches!(play, CardPlay::ThrowAway { .. }))
    );
}

// --- Bannière de protection ---

fn thrown_away(from: arcanum_core::WoundPile) -> Event {
    Event::WoundThrownAway { player: SOLO, from }
}

fn throws(events: &[Event], from: arcanum_core::WoundPile) -> usize {
    events
        .iter()
        .filter(|event| **event == thrown_away(from))
        .count()
}

/// Le combat joué jusqu'au bout sans parade ni unité : les dégâts au héros (la phase d'attaque
/// suit aussitôt), puis la fin du combat.
fn take_damage_and_leave(state: &GameState) -> GameState {
    let state = done(&to_block_phase(state), Action::NextCombatPhase);
    done(&state, Action::EndCombat)
}

fn wounds_in(cards: &[CardId]) -> usize {
    cards.iter().filter(|card| card.card().is_wound()).count()
}

// Spec 055 C19 — carte ; MKUE p. 9, 4.b ; spec 042 : Armure +1 et résistances au Feu et à la
// Glace pour l'unité porteuse.
#[test]
fn banner_of_protection_resistances() {
    let protection = artifact(BannerOfProtection);
    let golems = EnemyToken::new(arcanum_core::Enemy::IceGolems, 1);
    let decor = Decor {
        hand: vec![protection, card(March, 1)],
        owned: vec![peasants()],
        placed: vec![(golems, ORCS_2)],
        ..Decor::default()
    };
    let assign = Action::AssignDamage {
        enemy: golems,
        to: arcanum_core::DamageTarget::Unit(peasants()),
    };
    let damage = |state: &GameState| {
        let state = done(&to_block_phase(state), Action::NextCombatPhase);
        events_of(&state, &assign)
    };
    let destroyed = Event::UnitDestroyed {
        player: SOLO,
        unit: peasants(),
    };
    assert!(damage(&decor.fighting()).contains(&destroyed));
    let state = done(&decor.fighting(), assign_banner(protection, peasants()));
    let events = damage(&state);
    assert!(!events.contains(&destroyed), "{events:?}");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::UnitWounded { .. } | Event::WoundsTaken { .. })),
        "{events:?}"
    );
}

// Spec 055 C20 — Compendium, *Banner of Protection* Q1 ; Q5 : les Blessures reçues ce tour, en
// main et dans la défausse, jetées au début de la fin du tour ; pas celles d'avant.
#[test]
fn banner_of_protection_throws_received_wounds() {
    use arcanum_core::WoundPile::{DiscardPile, Hand};
    let protection = artifact(BannerOfProtection);
    let hags = EnemyToken::new(arcanum_core::Enemy::CursedHags, 1);
    let decor = Decor {
        hand: vec![protection, wound(101), wound(102), card(March, 1)],
        placed: vec![(hags, ORCS_2)],
        ..Decor::default()
    };
    let state = done(&decor.fighting(), throw_away(protection));
    let state = take_damage_and_leave(&state);
    assert_eq!(wounds_in(state.hand(SOLO)), 4);
    assert_eq!(wounds_in(state.discard_pile(SOLO)), 2);
    let events = events_of(&state, &end_turn());
    assert_eq!(throws(&events, Hand), 2);
    assert_eq!(throws(&events, DiscardPile), 2);
    assert_eq!(events.first(), Some(&thrown_away(Hand)));
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(wounds_in(after.hand(SOLO)), 2);
    assert_eq!(wounds_in(after.discard_pile(SOLO)), 0);

    // Une Blessure soignée entre-temps : une seule jetée de la main.
    let in_hand = *state
        .hand(SOLO)
        .iter()
        .rev()
        .find(|card| card.card().is_wound())
        .expect("Blessure en main");
    let healed = apply(
        state.clone(),
        &Event::WoundsHealed {
            player: SOLO,
            wounds: vec![in_hand],
        },
    );
    let events = events_of(&healed, &end_turn());
    assert_eq!(throws(&events, Hand), 1);
    // Sans la bannière (Marche jouée pour le tour minimal), rien n'est jeté.
    let plain = done(&decor.state(), sideways(card(March, 1)));
    let plain = done(&done(&plain, challenge(ORCS_2)), Action::StartCombat);
    let plain = take_damage_and_leave(&plain);
    assert_eq!(throws(&events_of(&plain, &end_turn()), Hand), 0);
}

// Spec 055 C21 — Compendium, *Banner of Protection* S2 : le héros est assommé quand même ; les
// Blessures reçues sont jetées à la fin du tour.
#[test]
fn banner_of_protection_does_not_prevent_knockout() {
    use arcanum_core::WoundPile::Hand;
    let protection = artifact(BannerOfProtection);
    let state = Decor {
        hand: vec![protection, card(March, 1), card(Rage, 1)],
        placed: vec![(fire_dragon(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, throw_away(protection));
    let events = events_of(&to_block_phase(&state), &Action::NextCombatPhase);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::KnockedOut { .. })),
        "{events:?}"
    );
    let state = take_damage_and_leave(&state);
    let received = wounds_in(state.hand(SOLO));
    assert!(received >= 5);
    assert_eq!(throws(&events_of(&state, &end_turn()), Hand), received);
}

// --- Épée de justice ---

fn sword_discarding(sword: CardId, cards: &[CardId]) -> Action {
    basic_with(
        sword,
        0,
        ArtifactChoices {
            discard: arcanum_core::Picks::of(cards),
            ..ArtifactChoices::default()
        },
    )
}

fn rage_basic() -> Action {
    option(card(Rage, 1), 0)
}

// Spec 055 C27 — carte ; Compendium, *Sword of Justice* Q4, S8 : Attaque 3 par carte défaussée,
// pas une Blessure ni l'Épée ; Gloire +1 par ennemi vaincu ce tour.
#[test]
fn sword_of_justice_discards_for_attack() {
    use arcanum_core::BasicAction::Promise;
    let sword = artifact(SwordOfJustice);
    let state = Decor {
        hand: vec![sword, card(March, 1), card(Promise, 1), wound(101)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    refused(&state, sword_discarding(sword, &[wound(101)]));
    refused(&state, sword_discarding(sword, &[sword]));
    let action = sword_discarding(sword, &[card(March, 1), card(Promise, 1)]);
    let events = events_of(&state, &action);
    assert_events_in_order(
        &events,
        &[
            Event::CardDiscarded {
                player: SOLO,
                card: card(March, 1),
            },
            Event::CardDiscarded {
                player: SOLO,
                card: card(Promise, 1),
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(in_progress_points(&state), physical(6));
    let events = events_of(&state, &Action::ResolveAttack);
    assert_events_in_order(
        &events,
        &[
            Event::EnemiesDefeated {
                player: SOLO,
                enemies: vec![prowlers()],
                fame: 2,
            },
            fame(1),
        ],
    );
}

// Spec 055 C28 — Compendium, *Sword of Justice* S2 : la base en phase d'attaque seulement.
#[test]
fn sword_of_justice_basic_only_in_attack_phase() {
    let sword = artifact(SwordOfJustice);
    let state = Decor {
        hand: vec![sword, card(March, 1)],
        ..Decor::default()
    }
    .fighting();
    let basic = |state: &GameState| {
        plays_of(state, sword)
            .iter()
            .any(|play| matches!(play, CardPlay::Artifact { .. }))
    };
    assert!(!basic(&state));
    let attack = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    assert!(basic(&attack));
    assert!(legal_actions(&attack, SOLO).contains(&sword_discarding(sword, &[])));
}

// Spec 055 C29 — exemple ; Compendium, *Sword of Justice* S1, S7 : renforcée, l'Attaque physique
// compte double et les ennemis perdent leur résistance physique ; une Attaque de feu non.
#[test]
fn sword_of_justice_doubles_physical() {
    let sword = artifact(SwordOfJustice);
    let fireball = spell_card(arcanum_core::Spell::Fireball);
    let decor = Decor {
        hand: vec![sword, card(Rage, 1), fireball, card(March, 1)],
        placed: vec![(ironclads(), ORCS_2)],
        extra: vec![token_gained(ManaColor::Basic(RED))],
        ..Decor::default()
    };
    let attack = skip_to_attack_phase(&decor.fighting());
    let state = done(&attack, throw_away(sword));
    let state = done(&state, declare(&[ironclads()]));
    let state = done(&state, rage_basic());
    assert_eq!(in_progress_points(&state), physical(4));
    let events = events_of(&state, &Action::ResolveAttack);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::EnemiesDefeated { .. }))
    );
    assert!(events.contains(&fame(1)), "{events:?}");
    // Le feu n'est pas doublé.
    let state = done(&done(&attack, throw_away(sword)), declare(&[ironclads()]));
    let state = done(
        &state,
        cast(fireball, 0, arcanum_core::ManaPayment::Token, None),
    );
    assert_eq!(
        in_progress_points(&state),
        arcanum_core::ElementPoints {
            fire: 5,
            ..arcanum_core::ElementPoints::default()
        }
    );
}

// Spec 055 C30 — Compendium, *Sword of Justice* S1, S3 : en phase à distance, la résistance
// perdue aussitôt ; le doublement en phase d'attaque seulement, attaque d'unité comprise.
#[test]
fn sword_of_justice_strong_timing() {
    let sword = artifact(SwordOfJustice);
    let state = Decor {
        hand: vec![sword, card(March, 1)],
        owned: vec![peasants()],
        placed: vec![(ironclads(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, throw_away(sword));
    let resistances = state
        .combat(SOLO)
        .and_then(|combat| combat.enemies.first())
        .map(|enemy| enemy.effective_resistances())
        .unwrap_or_default();
    assert!(!resistances.contains(&arcanum_core::Resistance::Physical));
    let state = done(&skip_to_attack_phase(&state), declare(&[ironclads()]));
    let state = done(&state, activate(peasants(), 0, 0));
    assert_eq!(in_progress_points(&state), physical(4));
}

// --- Corne de la colère et Poche de mana ---

/// Les événements de l'action avec la graine `seed` (les dés lancés en dépendent).
fn events_seeded(state: &GameState, action: &Action, seed: u64) -> Vec<Event> {
    arcanum_core::decide(state, SOLO, action, &mut config(seed).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

fn die_faces(events: &[Event]) -> Vec<ManaColor> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::ArtifactDieRolled { color, .. } => Some(*color),
            _ => None,
        })
        .collect()
}

fn wounds_from_card(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, Event::WoundFromCard { .. }))
        .count()
}

fn red_or_black(faces: &[ManaColor]) -> usize {
    faces
        .iter()
        .filter(|face| matches!(face, ManaColor::Black | ManaColor::Basic(RED)))
        .count()
}

fn horn(bonus: u8) -> ArtifactChoices {
    ArtifactChoices {
        bonus,
        ..ArtifactChoices::default()
    }
}

// Spec 055 C31 — carte (Lost Legion) ; Compendium, *Horn of Wrath* S1 : Attaque de siège 5 ; un dé,
// noir ou rouge : une Blessure en main.
#[test]
fn horn_of_wrath_siege_and_die() {
    let horn_card = artifact(HornOfWrath);
    let state = Decor {
        hand: vec![horn_card, card(March, 1)],
        placed: vec![(diggers(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, declare(&[diggers()]));
    let mut wounded = false;
    for seed in 0..40 {
        let events = events_seeded(&state, &basic_artifact(horn_card), seed);
        let faces = die_faces(&events);
        assert_eq!(faces.len(), 1);
        assert_eq!(wounds_from_card(&events), red_or_black(&faces));
        wounded |= red_or_black(&faces) > 0;
        let after = events.iter().fold(state.clone(), apply);
        assert_eq!(in_progress_points(&after), physical(5));
        let defeated = events_of(&after, &Action::ResolveAttack);
        assert!(
            defeated
                .iter()
                .any(|event| matches!(event, Event::EnemiesDefeated { .. }))
        );
    }
    assert!(wounded, "aucun dé rouge ou noir sur 40 graines");
}

// Spec 055 C32 — carte ; Compendium, *Horn of Wrath* S2 ; spec 024 : renforcé, +0 à +5, un dé par
// point ajouté ; le lancer est un point de non-retour.
#[test]
fn horn_of_wrath_strong_extra_dice() {
    let horn_card = artifact(HornOfWrath);
    let state = Decor {
        hand: vec![horn_card, card(March, 1)],
        placed: vec![(diggers(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, declare(&[diggers()]));
    refused(&state, throw_with(horn_card, 0, horn(6)));
    let events = events_of(&state, &throw_with(horn_card, 0, horn(3)));
    let faces = die_faces(&events);
    assert_eq!(faces.len(), 3);
    assert_eq!(wounds_from_card(&events), red_or_black(&faces));
    assert!(
        events
            .iter()
            .filter(|event| matches!(event, Event::ArtifactDieRolled { .. }))
            .all(Event::blocks_undo)
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(in_progress_points(&after), physical(8));
}

// Spec 055 C33 — carte : deux dés ; une couleur de base, son cristal ; or, la couleur au choix
// (après les autres dés) ; noir, Gloire +1.
#[test]
fn gem_pouch_rolls_two_dice() {
    let pouch = artifact(EndlessGemPouch);
    let state = Decor {
        hand: vec![pouch, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let (mut gold, mut black) = (false, false);
    for seed in 0..60 {
        let events = events_seeded(&state, &basic_artifact(pouch), seed);
        let faces = die_faces(&events);
        assert_eq!(faces.len(), 2);
        for face in &faces {
            if let ManaColor::Basic(color) = face {
                assert!(events.contains(&crystal(*color)), "{events:?}");
            }
        }
        let blacks = faces.iter().filter(|f| **f == ManaColor::Black).count();
        assert_eq!(fame_gained(&events), u32::try_from(blacks).unwrap_or(0));
        black |= blacks > 0;
        let golds = faces.iter().filter(|f| **f == ManaColor::Gold).count();
        if golds > 0 {
            gold = true;
            // L'or en dernier, puis le choix de la couleur seul.
            assert_eq!(faces.last(), Some(&ManaColor::Gold));
            let after = events.iter().fold(state.clone(), apply);
            let actions = legal_actions(&after, SOLO);
            assert!(!actions.is_empty());
            assert!(
                actions
                    .iter()
                    .all(|action| matches!(action, Action::ChooseCrystalColor { .. }))
            );
            let chosen = events_of(
                &after,
                &Action::ChooseCrystalColor {
                    color: arcanum_core::BasicColor::Green,
                },
            );
            assert_eq!(
                chosen.first(),
                Some(&crystal(arcanum_core::BasicColor::Green))
            );
            assert!(
                !chosen
                    .iter()
                    .any(|event| matches!(event, Event::CardDrawn { .. }))
            );
        }
    }
    assert!(gold && black, "or : {gold}, noir : {black}");
}

// Spec 055 C34 — carte ; Compendium, *Endless Gem Pouch* S1 : un jeton de chaque couleur de base,
// plus or de jour, noir de nuit.
#[test]
fn gem_pouch_strong_tokens_by_time() {
    let pouch = artifact(EndlessGemPouch);
    let tokens = |time: TimeOfDay| {
        let state = Decor {
            time,
            hand: vec![pouch, card(March, 1)],
            ..Decor::default()
        }
        .state();
        events_of(&state, &throw_away(pouch))
            .iter()
            .filter_map(|event| match event {
                Event::ManaTokenGained { color, .. } => Some(*color),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let basics: Vec<ManaColor> = arcanum_core::BasicColor::ALL
        .into_iter()
        .map(ManaColor::Basic)
        .collect();
    let mut day = basics.clone();
    day.push(ManaColor::Gold);
    let mut night = basics;
    night.push(ManaColor::Black);
    assert_eq!(tokens(TimeOfDay::Day), day);
    assert_eq!(tokens(TimeOfDay::Night), night);
}

// --- Graal doré et Livre de sagesse ---

fn herbalists() -> UnitCard {
    unit(Unit::Herbalists, 1)
}

fn spend_on_wound() -> Action {
    Action::SpendHealing {
        on: arcanum_core::HealingTarget::Wound,
    }
}

// Spec 055 C35 — carte ; Compendium, *Golden Grail* S1 ; Q7 : Gloire +1 par point de son Soin
// dépensé, ses points en premier.
#[test]
fn golden_grail_fame_per_point() {
    use arcanum_core::BasicAction::Tranquility;
    let grail = artifact(GoldenGrail);
    let state = Decor {
        hand: vec![
            grail,
            wound(101),
            wound(102),
            card(Tranquility, 1),
            card(March, 1),
        ],
        owned: vec![herbalists()],
        extra: vec![wound_unit(herbalists())],
        ..Decor::default()
    }
    .state();
    let state = done(&state, basic_artifact(grail));
    let events = events_of(&state, &spend_on_wound());
    assert_eq!(fame_gained(&events), 1);
    let state = events.iter().fold(state, apply);
    let events = events_of(
        &state,
        &Action::SpendHealing {
            on: arcanum_core::HealingTarget::Unit(herbalists()),
        },
    );
    assert_eq!(fame_gained(&events), 1);
    let state = events.iter().fold(state, apply);
    let state = done(&state, option(card(Tranquility, 1), 0));
    assert_eq!(fame_gained(&events_of(&state, &spend_on_wound())), 0);
}

// Spec 055 C36 — carte : renforcée, Soin 6 et une carte piochée par Blessure de la main jetée,
// tant que le deck en a.
#[test]
fn golden_grail_strong_draws_per_wound() {
    let grail = artifact(GoldenGrail);
    let hand = vec![grail, wound(101), wound(102), wound(103), card(March, 1)];
    let mut order = hand.clone();
    order.extend([card(Stamina, 1), card(Rage, 1)]);
    let mut extra = vec![Event::DeckShuffled {
        player: SOLO,
        order,
    }];
    extra.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    let decor = Decor {
        hand: hand.clone(),
        extra,
        ..Decor::default()
    };
    let state = decor.state();
    assert_eq!(state.deck(SOLO).len(), 2);
    let mut state = done(&state, throw_away(grail));
    let mut drawn = 0;
    for _ in 0..3 {
        let events = events_of(&state, &spend_on_wound());
        drawn += events
            .iter()
            .filter(|event| matches!(event, Event::CardDrawn { .. }))
            .count();
        state = events.iter().fold(state, apply);
    }
    assert_eq!(drawn, 2);
    assert!(state.deck(SOLO).is_empty());
    // En combat : pas proposée.
    assert!(
        !plays_of(&decor.fighting(), grail)
            .iter()
            .any(|play| matches!(play, CardPlay::ThrowAway { .. }))
    );
}

fn book_with(book: CardId, strong: bool, destroyed: CardId, gain: CardId) -> Action {
    let choices = ArtifactChoices {
        discard: arcanum_core::Picks::of(&[destroyed]),
        gain: Some(gain),
        ..ArtifactChoices::default()
    };
    if strong {
        throw_with(book, 0, choices)
    } else {
        basic_with(book, 0, choices)
    }
}

// Spec 055 C37, L4 — carte ; spec 056 : détruire une carte action de la main, une action avancée de
// sa couleur de l'offre en main ; l'offre se remplit.
#[test]
fn book_of_wisdom_gains_advanced_action() {
    let book = artifact(BookOfWisdom);
    let blood_rage = advanced(AdvancedAction::BloodRage);
    let state = Decor {
        hand: vec![book, card(Rage, 1), card(March, 1)],
        extra: advanced_offer(&[blood_rage]),
        ..Decor::default()
    }
    .state();
    let action = book_with(book, false, card(Rage, 1), blood_rage);
    let events = events_of(&state, &action);
    assert_events_in_order(
        &events,
        &[
            Event::CardDestroyed {
                player: SOLO,
                card: card(Rage, 1),
                from: DestroyedFrom::Hand,
            },
            Event::OfferCardGained {
                player: SOLO,
                offer: arcanum_core::Offer::AdvancedActions,
                card: arcanum_core::CommonCard::Deed(blood_rage),
                to: arcanum_core::CardDestination::Hand,
            },
        ],
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::OfferCardDealt { .. }))
    );
    let after = events.iter().fold(state.clone(), apply);
    assert!(after.hand(SOLO).contains(&blood_rage));
    // Marche (verte) : rien de vert dans l'offre, pas proposée.
    assert!(!plays_of(&state, book).iter().any(|play| matches!(
        play,
        CardPlay::Artifact { choices, .. } if choices.discard.iter().any(|c| c == card(March, 1))
    )));
}

// Spec 055 C38 — carte : renforcé, un sort de la couleur de la carte détruite en main, et un cristal
// de cette couleur.
#[test]
fn book_of_wisdom_strong_gains_spell_and_crystal() {
    let book = artifact(BookOfWisdom);
    let fireball = spell_card(arcanum_core::Spell::Fireball);
    let state = Decor {
        hand: vec![book, card(Rage, 1), card(March, 1)],
        extra: spell_offer(&[fireball]),
        ..Decor::default()
    }
    .state();
    let events = events_of(&state, &book_with(book, true, card(Rage, 1), fireball));
    assert_events_in_order(
        &events,
        &[
            Event::CardDestroyed {
                player: SOLO,
                card: card(Rage, 1),
                from: DestroyedFrom::Hand,
            },
            Event::OfferCardGained {
                player: SOLO,
                offer: arcanum_core::Offer::Spells,
                card: arcanum_core::CommonCard::Deed(fireball),
                to: arcanum_core::CardDestination::Hand,
            },
            crystal(RED),
        ],
    );
}

// --- Contenu, vue ---

// Spec 055 C1 — contenu : les seize artefacts ont leurs effets de base et renforcé.
#[test]
fn all_artifacts_supported() {
    for artifact in Artifact::ALL {
        let (basic, strong) = arcanum_core::GameContent::embedded().artifact(artifact);
        assert!(basic.option(0).is_some(), "{artifact:?} : base");
        assert!(strong.option(0).is_some(), "{artifact:?} : renforcé");
    }
}

// Spec 055 C42 — spec 052 : la vue donne les bannières, les modificateurs, le dé or du Sac en
// attente et le choix des bannières à reprendre.
#[test]
fn view_shows_banners_and_artifact_states() {
    let courage = artifact(BannerOfCourage);
    let ruby = artifact(RubyRing);
    let pouch = artifact(EndlessGemPouch);
    let state = round_end_decor(&[courage, ruby]);
    let state = apply(state, &exhaust(peasants()));
    let state = done(&state, assign_banner(courage, peasants()));
    let state = done(&state, use_banner(peasants(), None));
    let state = done(&state, throw_away(ruby));
    let view = arcanum_core::view(&state, SOLO);
    let carrier = view
        .units
        .iter()
        .find(|owned| owned.card == peasants())
        .copied()
        .expect("Paysans");
    assert_eq!(carrier.banner, Some(courage));
    assert!(carrier.banner_flipped);
    assert!(
        view.modifiers
            .iter()
            .any(|modifier| modifier.kind == arcanum_core::ModifierKind::EndlessMana(RED))
    );
    assert!(!view.banner_reclaim);
    let gold = apply(
        state.clone(),
        &Event::ArtifactDieRolled {
            player: SOLO,
            card: pouch,
            color: ManaColor::Gold,
        },
    );
    assert_eq!(arcanum_core::view(&gold, SOLO).crystal_choices, 1);
    let state = done(&state, Action::AnnounceEndOfRound);
    assert!(arcanum_core::view(&state, SOLO).banner_reclaim);
}

// Spec 055, Interface (zone de jeu) : les dés lancés par un artefact restent visibles le temps du
// tour, Corne de la colère compris (ses dés ne changent pas l'état).
#[test]
fn view_shows_rolled_artifact_dice() {
    let horn = artifact(HornOfWrath);
    let state = Decor {
        hand: vec![horn, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let rolled = apply(
        state,
        &Event::ArtifactDieRolled {
            player: SOLO,
            card: horn,
            color: ManaColor::Basic(arcanum_core::BasicColor::Blue),
        },
    );
    assert!(arcanum_core::view(&rolled, SOLO).turn_facts.contains(
        &arcanum_core::TurnFact::ArtifactDieRolled {
            card: horn,
            color: ManaColor::Basic(arcanum_core::BasicColor::Blue),
        }
    ));
}

// --- Épée de justice et Mages d'Altem (issue privée 272) ---

fn altem_mages() -> UnitCard {
    unit(Unit::AltemMages, 1)
}

/// La capacité au mana noir des Mages d'Altem : option 0, feu glacé ; option 1, siège. Payée par
/// le dé noir en tête de la Source.
fn altem_black(option: u8) -> Action {
    Action::ActivateUnit {
        unit: altem_mages(),
        ability: 2,
        option,
        mana: vec![arcanum_core::ManaPayment::Die(0)],
        target: None,
        enemy: None,
        leadership: false,
    }
}

/// Un combat de nuit contre les Rôdeurs, dé noir en tête de la Source, Mages d'Altem recrutés,
/// l'Épée de justice, Rage et Marche en main.
fn sword_and_altem() -> GameState {
    use arcanum_core::BasicColor;
    Decor {
        time: TimeOfDay::Night,
        hand: vec![artifact(SwordOfJustice), card(Rage, 1), card(March, 1)],
        dice: vec![
            ManaColor::Black,
            ManaColor::Basic(BasicColor::Green),
            ManaColor::Basic(BasicColor::Red),
        ],
        owned: vec![altem_mages()],
        ..Decor::default()
    }
    .fighting()
}

fn cold_fire(amount: u32) -> arcanum_core::ElementPoints {
    arcanum_core::ElementPoints {
        cold_fire: amount,
        ..arcanum_core::ElementPoints::default()
    }
}

// Issue privée 272 — Compendium, *Sword of Justice* S6 (et S1) : l'Épée renforcée ne double qu'en phase
// d'attaque, où il n'y a plus de siège ; une Attaque devenue de siège en phase à distance n'est
// pas doublée.
#[test]
fn sword_of_justice_strong_not_doubled_with_altem_siege() {
    let sword = artifact(SwordOfJustice);
    let state = done(&sword_and_altem(), altem_black(1));
    let state = done(&state, throw_away(sword));
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, rage_basic());
    assert_eq!(in_progress_points(&state), physical(2));
}

// Issue privée 272 — Compendium, *Sword of Justice* S6 : les Attaques physiques doublées par l'Épée
// renforcée peuvent devenir de feu glacé (Mages d'Altem) ; MKUE p. 9, 4.a.
#[test]
fn sword_of_justice_doubled_attack_becomes_cold_fire() {
    let sword = artifact(SwordOfJustice);
    let attack = skip_to_attack_phase(&sword_and_altem());
    let state = done(&attack, throw_away(sword));
    let state = done(&state, altem_black(0));
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, rage_basic());
    assert_eq!(in_progress_points(&state), cold_fire(4));
}

// Issue privée 272 — Compendium, *Sword of Justice* S6, S2 : l'Attaque de base de l'Épée peut devenir de
// feu glacé (Mages d'Altem).
#[test]
fn sword_of_justice_basic_becomes_cold_fire() {
    let sword = artifact(SwordOfJustice);
    let attack = skip_to_attack_phase(&sword_and_altem());
    let state = done(&attack, altem_black(0));
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, sword_discarding(sword, &[card(March, 1)]));
    assert_eq!(in_progress_points(&state), cold_fire(3));
}

// Issue privée 272 — Compendium, *Sword of Justice* S6, S2 (et *Bow of Starsdawn* O5) : l'Attaque de base
// de l'Épée peut devenir de siège (Mages d'Altem), donc se jouer en phase à distance.
#[test]
fn sword_of_justice_basic_becomes_siege() {
    let sword = artifact(SwordOfJustice);
    let state = done(&sword_and_altem(), altem_black(1));
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, sword_discarding(sword, &[card(March, 1)]));
    assert_eq!(in_progress_points(&state), physical(3));
}
