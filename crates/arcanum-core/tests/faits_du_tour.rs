//! Scénarios de la spec 073 (faits du tour et jetons dépensés) : ce que la zone de jeu de l'écran
//! raconte du tour. Aucune règle nouvelle : ces données suivent les événements (MKUE p. 3,
//! *Play Area* ; p. 11, *End of the Turn*).

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, CommonCard, CommonDeck, DeedCard, Effect,
    Event, GameState, ManaColor, ManaPayment, Offer, SidewaysEffect, Site, TilePosition, TurnFact,
    Unit, UnitCard, apply, legal_actions, view,
};

use BasicAction::*;

mod arene;
use arene::*;

// --- Outils ---

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

/// La carte mise dans l'offre, depuis sa pioche (spec 050).
fn in_offer(offer: Offer, card: CommonCard) -> Vec<Event> {
    vec![
        Event::CommonDeckShuffled {
            deck: card.deck(),
            order: vec![card],
        },
        Event::OfferCardDealt { offer, card },
    ]
}

/// L'artefact tiré de sa pioche pour une récompense, en attente du choix (spec 041).
fn drawn_artifact(card: CardId) -> Vec<Event> {
    vec![
        Event::CommonDeckShuffled {
            deck: CommonDeck::Artifacts,
            order: vec![CommonCard::Deed(card)],
        },
        Event::ArtifactsDrawn {
            player: SOLO,
            cards: vec![card],
        },
    ]
}

/// Des Paysans de plus, à recruter.
fn other_peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 2)
}

fn facts(state: &GameState) -> Vec<TurnFact> {
    view(state, SOLO).turn_facts
}

fn applied(state: GameState, events: &[Event]) -> GameState {
    events.iter().fold(state, apply)
}

/// L'interaction commencée au village, sans bonus (spec 040), avec `influence` points d'une carte
/// jouée de côté.
fn interacting_events(state: &GameState, influence: u8) -> Vec<Event> {
    vec![
        Event::InteractionStarted {
            player: SOLO,
            site: Site::Village,
            bonus: 0,
            city_shields: 0,
        },
        Event::CardPlayed {
            player: SOLO,
            card: state.hand(SOLO)[0],
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(influence),
        },
    ]
}

fn recruit(unit: UnitCard) -> Action {
    Action::RecruitUnit {
        unit,
        disband: None,
        bonds: false,
    }
}

/// Le décor du recrutement : au village, Paysans dans l'offre d'unités.
fn village_with_peasants() -> GameState {
    Units {
        offer: vec![peasants()],
        ..Units::default()
    }
    .state()
}

const EXPLORED: TilePosition = TilePosition::from_lattice(1, 1);

// --- Faits du tour ---

// Spec 073 C1
#[test]
fn defeating_enemies_is_a_turn_fact() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 5 : l'ennemi vaincu rapporte sa gloire.
    let swiftness = card(Swiftness, 1);
    let state = fight(VILLAGE, &[swiftness], &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    assert!(
        facts(&state).contains(&TurnFact::EnemiesDefeated {
            enemies: vec![prowlers()],
            fame: 2,
        }),
        "{:?}",
        facts(&state)
    );
}

// Spec 073 C2
#[test]
fn turn_facts_keep_event_order() {
    let state = village_with_peasants();
    let state = applied(
        state,
        &[Event::Explored {
            player: SOLO,
            position: EXPLORED,
            cost: 0,
        }],
    );
    let events = interacting_events(&state, 4);
    let state = applied(state, &events);
    let state = done(&state, recruit(peasants()));
    assert_eq!(
        facts(&state),
        vec![
            TurnFact::TileRevealed { position: EXPLORED },
            TurnFact::UnitGained { unit: peasants() },
        ]
    );
}

// Spec 073 C3
#[test]
fn each_listed_event_makes_a_fact() {
    // Le décor recrute les Paysans par événement : le tour commence après lui. Une Blessure en
    // main pour la soigner.
    let mut hand = Units::default().hand;
    hand.push(wound(1));
    let state = Units {
        owned: vec![peasants()],
        hand,
        ..Units::default()
    }
    .state();
    let state = apply(
        state,
        &Event::TurnStarted {
            player: SOLO,
            turn: 1,
        },
    );
    let fire_bolt = advanced(arcanum_core::AdvancedAction::FireBolt);
    let fireball = CardId::new(DeedCard::Spell(arcanum_core::Spell::Fireball), 1);
    let banner = artifact(arcanum_core::Artifact::BannerOfGlory);
    let wound_cards = vec![wound(1), wound(2)];
    let cases = [
        (
            Event::Explored {
                player: SOLO,
                position: EXPLORED,
                cost: 0,
            },
            TurnFact::TileRevealed { position: EXPLORED },
        ),
        (
            Event::EnemiesDefeated {
                player: SOLO,
                enemies: vec![prowlers()],
                fame: 2,
            },
            TurnFact::EnemiesDefeated {
                enemies: vec![prowlers()],
                fame: 2,
            },
        ),
        (
            Event::FameGained {
                player: SOLO,
                fame: 1,
            },
            TurnFact::FameGained { fame: 1 },
        ),
        (
            Event::ReputationChanged {
                player: SOLO,
                change: -1,
            },
            TurnFact::ReputationChanged { change: -1 },
        ),
        (
            Event::WoundsTaken {
                player: SOLO,
                enemy: prowlers(),
                wounds: wound_cards.clone(),
            },
            TurnFact::WoundsTaken { count: 2 },
        ),
        (
            Event::WoundsHealed {
                player: SOLO,
                wounds: vec![wound(1)],
            },
            TurnFact::WoundsHealed { count: 1 },
        ),
        (
            Event::CrystalGained {
                player: SOLO,
                color: BasicColor::Green,
            },
            TurnFact::CrystalGained {
                color: BasicColor::Green,
            },
        ),
        (
            Event::OfferCardGained {
                to: arcanum_core::CardDestination::DeckTop,
                player: SOLO,
                offer: Offer::AdvancedActions,
                card: CommonCard::Deed(fire_bolt),
            },
            TurnFact::CardGained { card: fire_bolt },
        ),
        (
            Event::AdvancedActionBought {
                player: SOLO,
                card: fire_bolt,
                cost: 0,
            },
            TurnFact::CardGained { card: fire_bolt },
        ),
        (
            Event::SpellBought {
                player: SOLO,
                card: fireball,
                cost: 0,
            },
            TurnFact::CardGained { card: fireball },
        ),
        (
            Event::ArtifactTaken {
                player: SOLO,
                card: banner,
            },
            TurnFact::CardGained { card: banner },
        ),
        (
            Event::UnitRecruited {
                player: SOLO,
                unit: other_peasants(),
                cost: 0,
            },
            TurnFact::UnitGained {
                unit: other_peasants(),
            },
        ),
        (
            Event::UnitRewardTaken {
                player: SOLO,
                unit: other_peasants(),
            },
            TurnFact::UnitGained {
                unit: other_peasants(),
            },
        ),
        (
            Event::UnitDisbanded {
                player: SOLO,
                unit: peasants(),
            },
            TurnFact::UnitLost { unit: peasants() },
        ),
        (
            Event::UnitDestroyed {
                player: SOLO,
                unit: peasants(),
            },
            TurnFact::UnitLost { unit: peasants() },
        ),
        (
            Event::UnitWounded {
                player: SOLO,
                unit: peasants(),
                wounds: 1,
            },
            TurnFact::UnitWounded { unit: peasants() },
        ),
        (
            Event::UnitHealed {
                player: SOLO,
                unit: peasants(),
                cost: 1,
            },
            TurnFact::UnitHealed { unit: peasants() },
        ),
        (
            Event::SiteConquered {
                player: SOLO,
                hex: VILLAGE,
            },
            TurnFact::SiteConquered { hex: VILLAGE },
        ),
        (
            Event::VillagePlundered { player: SOLO },
            TurnFact::VillagePlundered,
        ),
        (
            Event::MonasteryBurned {
                player: SOLO,
                hex: MONASTERY,
            },
            TurnFact::MonasteryBurned { hex: MONASTERY },
        ),
    ];
    for (event, fact) in cases {
        // Ce qu'il faut à l'événement pour avoir un objet : un événement sans objet ne change
        // rien, il ne fait pas de fait.
        let before: Vec<Event> = match &event {
            Event::UnitHealed { unit, .. } => vec![Event::UnitWounded {
                player: SOLO,
                unit: *unit,
                wounds: 1,
            }],
            Event::ArtifactTaken { card, .. } => drawn_artifact(*card),
            Event::OfferCardGained { offer, card, .. } => in_offer(*offer, *card),
            Event::AdvancedActionBought { card, .. } => {
                in_offer(Offer::Units, CommonCard::Deed(*card))
            }
            Event::SpellBought { card, .. } => in_offer(Offer::Spells, CommonCard::Deed(*card)),
            Event::UnitRecruited { unit, .. } | Event::UnitRewardTaken { unit, .. } => {
                in_offer(Offer::Units, CommonCard::Unit(*unit))
            }
            _ => Vec::new(),
        };
        let prepared = applied(state.clone(), &before);
        let prepared = apply(
            prepared,
            &Event::TurnStarted {
                player: SOLO,
                turn: 2,
            },
        );
        let after = apply(prepared, &event);
        assert_eq!(facts(&after), vec![fact], "{event:?}");
    }
}

// Spec 073 C3
#[test]
fn moves_and_plays_make_no_fact() {
    let march = card(March, 1);
    let state = arena(PLAIN_W, &[march, card(Rage, 1)], &[]);
    // Carte jouée, dé pris de la Source : la zone de jeu les montre déjà.
    let state = done(&state, strong(march, GREEN_DIE));
    // Déplacement.
    let state = done(&state, Action::MoveHero { to: HILL });
    assert_eq!(state.figure(SOLO), Some(HILL));
    assert_eq!(facts(&state), Vec::new());
}

// Spec 073 C4
#[test]
fn turn_facts_cleared_at_turn_start() {
    let state = village_with_peasants();
    let state = applied(
        state,
        &[
            Event::ReputationChanged {
                player: SOLO,
                change: -1,
            },
            Event::TurnEnded { player: SOLO },
        ],
    );
    let state = applied(
        state,
        &drawn_artifact(artifact(arcanum_core::Artifact::BannerOfGlory)),
    );
    let state = applied(
        state,
        &[
            // Récompenses de la fin du tour : après `TurnEnded` (MKUE p. 11, 5).
            Event::CrystalGained {
                player: SOLO,
                color: BasicColor::Green,
            },
            Event::ArtifactTaken {
                player: SOLO,
                card: artifact(arcanum_core::Artifact::BannerOfGlory),
            },
        ],
    );
    assert_eq!(
        facts(&state),
        vec![
            TurnFact::ReputationChanged { change: -1 },
            TurnFact::CrystalGained {
                color: BasicColor::Green
            },
            TurnFact::CardGained {
                card: artifact(arcanum_core::Artifact::BannerOfGlory)
            },
        ]
    );
    let state = apply(
        state,
        &Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    );
    assert_eq!(facts(&state), Vec::new());
}

// Spec 073 C5
#[test]
fn undo_removes_turn_fact() {
    // MKUE p. 5, *Reverting* ; spec 024.
    let decor = village_with_peasants();
    let mut events = Units {
        offer: vec![peasants()],
        ..Units::default()
    }
    .events();
    events.extend(interacting_events(&decor, 4));
    let (mut runner, puppet) = runner(events);
    play(&mut runner, &puppet, recruit(peasants()));
    assert_eq!(
        facts(runner.state()),
        vec![TurnFact::UnitGained { unit: peasants() }]
    );
    runner.undo().expect("le recrutement se reprend");
    assert_eq!(facts(runner.state()), Vec::new());
}

// --- Jetons dépensés ---

// Spec 073 C6
#[test]
fn spent_tokens_are_listed_until_turn_end() {
    // MKUE p. 5, *Using mana* 1 : le jeton paie l'effet renforcé ; p. 11, 3.a : les jetons
    // retournent à la réserve à la fin du tour. Endurance (bleue, Mouvement) plutôt que
    // Détermination (Parade, seulement en combat) : le paiement est le même.
    let concentration = card(Concentration, 1);
    let stamina = card(Stamina, 1);
    let state = state_with(&[concentration, stamina]);
    let state = done(&state, option(concentration, 0));
    let blue = ManaColor::Basic(BasicColor::Blue);
    assert_eq!(view(&state, SOLO).mana_tokens, vec![blue]);
    let state = done(
        &state,
        Action::PlayCard {
            card: stamina,
            play: CardPlay::Strong {
                gain: None,
                extra_mana: None,
                option: 0,
                discard: None,
                mana: ManaPayment::Token,
                set_dice: [None, None],
                with: None,
            },
        },
    );
    let seen = view(&state, SOLO);
    assert_eq!(seen.mana_tokens, Vec::new());
    assert_eq!(seen.spent_mana_tokens, vec![blue]);
    let end_turn = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, .. } if discard.is_empty()))
        .expect("fin du tour sans défausse");
    let state = done(&state, end_turn);
    let seen = view(&state, SOLO);
    assert_eq!(seen.mana_tokens, Vec::new());
    assert_eq!(seen.spent_mana_tokens, Vec::new());
}
