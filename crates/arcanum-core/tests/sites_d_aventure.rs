//! Scénarios de la spec 044 (Sites d'aventure).

use arcanum_core::{
    Achievement, Action, AttackKind, BasicAction, BasicColor, CardId, CardPlay, CombatPhase,
    Effect, Element, EnemyColor, Event, GameState, Hex, ManaColor, ManaPayment, Player, RngUsage,
    SidewaysEffect, Site, TimeOfDay, Unit, UnitCard, WoundPile, apply, decide, final_score,
    legal_actions, roll_die, view,
};

use BasicAction::*;

mod arene;
use arene::*;

mod premier_choix;

/// Une carte de la main jouée de côté pour `effect`, par un événement : seule la valeur compte ici.
fn played(state: &GameState, card: CardId, effect: Effect) -> GameState {
    let sideways = match effect {
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

fn melee(amount: u8) -> Effect {
    Effect::Attack {
        amount,
        element: Element::Physical,
        kind: AttackKind::Melee,
    }
}

fn block(amount: u8) -> Effect {
    Effect::Block {
        amount,
        element: Element::Physical,
    }
}

// --- Ennemis désignés par leur jeton (Q1) ---

/// Les Rôdeurs et les Creuseurs sur la même case (les orques ②), combattus ensemble.
fn two_on_one_space() -> GameState {
    fight(
        VILLAGE,
        &[card(March, 1), card(Rage, 1)],
        &[(prowlers(), ORCS_2), (diggers(), ORCS_2)],
        &[ORCS_2],
    )
}

// C1 — Compendium, *Miscellaneous* S15 : deux jetons d'une même case sont deux ennemis.
#[test]
fn two_enemies_on_one_space_are_distinct_targets() {
    let state = two_on_one_space();
    let combat = state.combat(SOLO).expect("combat engagé");
    assert_eq!(
        combat
            .enemies
            .iter()
            .map(|enemy| (enemy.hex, enemy.token))
            .collect::<Vec<_>>(),
        [(ORCS_2, prowlers()), (ORCS_2, diggers())]
    );

    // Attaque : les Rôdeurs seuls (armure 3, gloire 2) ; les Creuseurs restent en jeu.
    let attacking = skip_to_attack_phase(&state);
    let attacking = done(&attacking, declare(&[prowlers()]));
    let attacking = played(&attacking, card(March, 1), melee(3));
    let events = events_of(&attacking, &Action::ResolveAttack);
    assert_events_in_order(
        &events,
        &[Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![prowlers()],
            fame: 2,
        }],
    );
    let after = events.iter().fold(attacking, apply);
    let combat = after.combat(SOLO).expect("combat engagé");
    assert!(combat.enemy(prowlers()).is_some_and(|enemy| enemy.defeated));
    assert!(combat.enemy(diggers()).is_some_and(|enemy| !enemy.defeated));
    assert_eq!(after.enemy_tokens_at(ORCS_2), [diggers()]);

    // Parade : les Rôdeurs seuls (attaque 4) ; les Creuseurs ne sont pas parés.
    let blocking = to_block_phase(&state);
    let blocking = done(&blocking, declare_block(prowlers()));
    let blocking = played(&blocking, card(Rage, 1), block(4));
    let after = done(&blocking, Action::ResolveBlock);
    let combat = after.combat(SOLO).expect("combat engagé");
    assert!(combat.enemy(prowlers()).is_some_and(|enemy| enemy.blocked));
    assert!(combat.enemy(diggers()).is_some_and(|enemy| !enemy.blocked));
}

// --- Entrer (C2 à C6, C17, C20) ---

fn legal(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
}

fn can_enter(state: &GameState) -> bool {
    legal(state).contains(&Action::EnterAdventureSite)
}

/// Les Rôdeurs errants sur la plaine voisine du donjon.
fn dungeon_with_prowlers() -> GameState {
    Adventure {
        placed: vec![(prowlers(), NEAR_DUNGEON)],
        ..Adventure::default()
    }
    .state()
}

// C2 — MKUE p. 8, *Combat with Enemies* 1.b et 2.d ; p. 6, 6.e : un jeton marron tiré, le combat
// aussitôt, l'action du tour prise.
#[test]
fn entering_a_dungeon_starts_combat_at_once() {
    let state = dungeon_with_prowlers();
    assert!(legal(&state).contains(&challenge(NEAR_DUNGEON)));
    let state = done(&state, basic(card(March, 1)));
    let state = done(
        &state,
        Action::PlayCard {
            card: card(Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
        },
    );
    assert!(can_enter(&state));
    let events = events_of(&state, &Action::EnterAdventureSite);
    assert_eq!(
        events,
        [
            Event::AdventureSiteEntered {
                player: SOLO,
                hex: DUNGEON
            },
            Event::EnemyTokenPlaced {
                token: minotaur(),
                hex: DUNGEON
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(DUNGEON, minotaur())]
            },
        ]
    );
    let entered = events.iter().fold(state, apply);
    assert_eq!(entered.adventure(SOLO), Some(DUNGEON));
    assert_eq!(entered.enemy_tokens_at(DUNGEON), [minotaur()]);
    assert_eq!(
        (
            entered.move_points(SOLO),
            entered.influence_points(SOLO),
            entered.movement_ended(SOLO)
        ),
        (0, 0, true)
    );
    let after = done(&skip_to_attack_phase(&entered), Action::EndCombat);
    for state in [&entered, &after] {
        assert!(!legal(state).iter().any(|action| matches!(
            action,
            Action::MoveHero { .. }
                | Action::FollowRoute { .. }
                | Action::Explore { .. }
                | Action::Interact
                | Action::Challenge { .. }
                | Action::EnterAdventureSite
        )));
    }
}

// C3 — carte du tombeau : un jeton rouge (Draconien).
#[test]
fn tomb_draws_a_red_token() {
    let state = Adventure {
        figure: TOMB,
        ..Adventure::default()
    }
    .state();
    assert!(can_enter(&state));
    assert_eq!(
        events_of(&state, &Action::EnterAdventureSite),
        [
            Event::AdventureSiteEntered {
                player: SOLO,
                hex: TOMB
            },
            Event::EnemyTokenPlaced {
                token: swamp_dragon(),
                hex: TOMB
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(TOMB, swamp_dragon())]
            },
        ]
    );
}

// C4 — Compendium, *Miscellaneous* S14 : la figurine doit être sur le site.
#[test]
fn entering_needs_standing_on_the_site() {
    assert!(Site::Dungeon.is_adventure() && Site::MonsterDen.is_adventure());
    assert!(!Site::MagicalGlade.is_adventure() && !Site::Village.is_adventure());
    for figure in [NEAR_DUNGEON, GLADE, Hex::new(5, -4)] {
        let state = Adventure {
            figure,
            ..Adventure::default()
        }
        .state();
        assert!(!can_enter(&state), "{figure:?}");
        refused(&state, Action::EnterAdventureSite);
    }
}

// C5 — Compendium, *Combat* S40 ; spec 040 : pas après une interaction, un combat ou une
// provocation ; permis sur la case X de réputation.
#[test]
fn entering_is_the_action_of_the_turn() {
    let state = dungeon_with_prowlers();
    let interacted = apply(
        state.clone(),
        &Event::InteractionStarted {
            player: SOLO,
            site: Site::Village,
            bonus: 0,
            city_shields: 0,
        },
    );
    assert!(!can_enter(&interacted));
    let challenged = done(&state, challenge(NEAR_DUNGEON));
    assert!(!can_enter(&challenged));
    let fighting = done(&challenged, Action::StartCombat);
    let fought = done(&skip_to_attack_phase(&fighting), Action::EndCombat);
    assert!(!can_enter(&fought));
    let provoked = apply(
        state.clone(),
        &Event::EnemiesProvoked {
            player: SOLO,
            enemies: vec![NEAR_DUNGEON],
        },
    );
    assert!(!can_enter(&provoked));
    let on_x = apply(
        state,
        &Event::ReputationChanged {
            player: SOLO,
            change: -7,
        },
    );
    assert!(on_x.reputation(SOLO).is_x());
    assert!(can_enter(&on_x));
}

// C6 — MKUE p. 7, 4.d : un site d'aventure ne termine pas le déplacement.
#[test]
fn adventure_site_does_not_stop_movement() {
    let state = Adventure {
        figure: NEAR_DUNGEON,
        ..Adventure::default()
    }
    .state();
    let state = done(&state, basic(card(March, 1)));
    let state = done(&state, basic(card(Swiftness, 1)));
    let state = done(&state, sideways(card(Rage, 1)));
    // La clairière de la campagne 7, par le donjon (2 + 3) : le marais du centre coûte 5.
    let glade_7 = Hex::new(7, -4);
    assert!(
        legal(&state).contains(&Action::FollowRoute { to: glade_7 }),
        "{:?}",
        legal(&state)
    );
    let stepped = done(&state, Action::MoveHero { to: DUNGEON });
    assert!(!stepped.movement_ended(SOLO));
    assert!(legal(&stepped).contains(&Action::MoveHero { to: glade_7 }));
    assert!(can_enter(&stepped));
}

/// Les Gargouilles restées sur l'antre, la figurine sur la colline voisine.
fn den_with_gargoyle() -> GameState {
    Adventure {
        figure: NEAR_DEN,
        placed: vec![(gargoyle(), MONSTER_DEN)],
        ..Adventure::default()
    }
    .state()
}

// C17 — MKUE p. 7, 4.b à 4.d ; Compendium, *Combat* S15 et S40 : un jeton resté sur un antre
// n'est pas un ennemi errant.
#[test]
fn adventure_tokens_are_not_rampaging() {
    let state = den_with_gargoyle();
    let state = done(&state, basic(card(March, 1)));
    let state = done(&state, basic(card(Swiftness, 1)));
    let actions = legal(&state);
    assert!(actions.contains(&Action::MoveHero { to: MONSTER_DEN }));
    assert!(!actions.contains(&challenge(MONSTER_DEN)));
    // La mine du centre de la campagne 6 longe aussi l'antre.
    let step = events_of(
        &state,
        &Action::MoveHero {
            to: Hex::new(4, -5),
        },
    );
    assert!(
        !step
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. })),
        "{step:?}"
    );
    let entered = done(
        &done(&state, Action::MoveHero { to: MONSTER_DEN }),
        Action::EnterAdventureSite,
    );
    assert_eq!(
        entered.combat(SOLO).map(|combat| combat.enemies.len()),
        Some(1)
    );
}

// C20 — spec 043, L4 : plus aucun jeton nulle part, le site est conquis sans combat.
#[test]
fn no_token_left_conquers_without_combat() {
    let mut events = Adventure {
        figure: MONSTER_DEN,
        ..Adventure::default()
    }
    .events();
    events.extend(EnemyColor::ALL.map(|color| Event::EnemyPileShuffled {
        color,
        order: Vec::new(),
    }));
    let state = events.iter().fold(GameState::default(), apply);
    assert_eq!(
        events_of(&state, &Action::EnterAdventureSite),
        [
            Event::AdventureSiteEntered {
                player: SOLO,
                hex: MONSTER_DEN
            },
            Event::SiteConquered {
                player: SOLO,
                hex: MONSTER_DEN
            },
        ]
    );
}

// --- Donjon et tombeau : seul, de nuit (C7 à C10, L7) ---

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn activations(state: &GameState) -> usize {
    legal(state)
        .into_iter()
        .filter(|action| matches!(action, Action::ActivateUnit { .. }))
        .count()
}

// C7 — carte du donjon ; Compendium, *Combat* S43 : les unités ne participent pas, les dégâts vont
// au héros.
#[test]
fn no_units_in_dungeons_and_tombs() {
    let state = Adventure {
        owned: vec![peasants()],
        ..Adventure::default()
    }
    .state();
    let entered = done(&state, Action::EnterAdventureSite);
    let blocking = done(&entered, Action::NextCombatPhase);
    assert_eq!(
        blocking.combat(SOLO).map(|combat| combat.phase),
        Some(CombatPhase::Block)
    );
    assert_eq!(activations(&done(&blocking, declare_block(minotaur()))), 0);
    let events = events_of(&blocking, &Action::NextCombatPhase);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::DamageAssignedToUnit { .. })),
        "{events:?}"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::WoundsTaken { .. }))
    );
}

const GOLD_DIE: u8 = 0;

/// De jour, au donjon : un jeton or, la Source sans dé rouge (or, bleu, blanc, noir), Tranquillité
/// en main à la place de Détermination.
fn dungeon_with_gold(figure: Hex) -> GameState {
    let state = Adventure {
        figure,
        hand: vec![
            card(March, 1),
            card(Rage, 1),
            card(Tranquility, 1),
            card(Swiftness, 1),
            card(Promise, 1),
        ],
        dice: vec![
            ManaColor::Gold,
            ManaColor::Basic(BasicColor::Blue),
            ManaColor::Basic(BasicColor::White),
            ManaColor::Black,
        ],
        placed: if figure == MONSTER_DEN {
            vec![(gargoyle(), MONSTER_DEN)]
        } else {
            Vec::new()
        },
        owned: vec![peasants()],
        ..Adventure::default()
    }
    .state();
    apply(
        state,
        &Event::ManaTokenGained {
            player: SOLO,
            color: ManaColor::Gold,
        },
    )
}

/// L'effet renforcé de `card` payé par `mana` est-il proposé ?
fn strong_by(state: &GameState, card: CardId, mana: ManaPayment) -> bool {
    legal(state).iter().any(|action| {
        matches!(
            action,
            Action::PlayCard {
                card: played,
                play: CardPlay::Strong { mana: paid, .. },
            } if *played == card && *paid == mana
        )
    })
}

// C8 — FAQ p1 ; Compendium, *Mana* Q7, *Combat* Q2 : règles de Nuit pendant le combat du donjon,
// celles de la manche avant et après.
#[test]
fn night_rules_in_dungeon_combat() {
    let state = dungeon_with_gold(DUNGEON);
    let tranquility = card(Tranquility, 1);
    assert!(strong_by(&state, tranquility, ManaPayment::Token));
    assert!(strong_by(&state, tranquility, ManaPayment::Die(GOLD_DIE)));
    let entered = done(&state, Action::EnterAdventureSite);
    let attacking = done(&skip_to_attack_phase(&entered), declare(&[minotaur()]));
    let rage = card(Rage, 1);
    assert!(!strong_by(&attacking, rage, ManaPayment::Token));
    assert!(!strong_by(&attacking, rage, ManaPayment::Die(GOLD_DIE)));
    let after = done(&skip_to_attack_phase(&entered), Action::EndCombat);
    assert!(strong_by(&after, tranquility, ManaPayment::Token));
    assert!(strong_by(&after, tranquility, ManaPayment::Die(GOLD_DIE)));
}

// L7 — Compendium, *Mana* S3 : le dé noir de la Source n'est plus épuisé au donjon, mais il ne
// paie encore rien (sorts, spec 054).
#[test]
fn black_die_still_pays_nothing_in_dungeon() {
    let black_die = 3;
    let entered = done(&dungeon_with_gold(DUNGEON), Action::EnterAdventureSite);
    let attacking = done(&skip_to_attack_phase(&entered), declare(&[minotaur()]));
    assert!(!legal(&attacking).iter().any(|action| matches!(
        action,
        Action::PlayCard {
            play: CardPlay::Strong {
                mana: ManaPayment::Die(die),
                ..
            },
            ..
        } if *die == black_die
    )));
}

// C9 — Compendium, *Miscellaneous* S10 : la vue donne la Nuit pour le combat du donjon.
#[test]
fn view_shows_the_combat_time_of_day() {
    let state = dungeon_with_gold(DUNGEON);
    assert_eq!(view(&state, SOLO).combat_time_of_day, TimeOfDay::Day);
    let entered = done(&state, Action::EnterAdventureSite);
    let seen = view(&entered, SOLO);
    assert_eq!(
        (seen.time_of_day, seen.combat_time_of_day, seen.adventure),
        (TimeOfDay::Day, TimeOfDay::Night, Some(DUNGEON))
    );
    assert_eq!(entered.combat_time_of_day(SOLO), TimeOfDay::Night);
    let after = done(&skip_to_attack_phase(&entered), Action::EndCombat);
    assert_eq!(view(&after, SOLO).combat_time_of_day, TimeOfDay::Day);
}

// C10 — carte de l'antre : règles de la manche, unités permises.
#[test]
fn monster_den_uses_round_rules() {
    let entered = done(&dungeon_with_gold(MONSTER_DEN), Action::EnterAdventureSite);
    assert_eq!(entered.combat_time_of_day(SOLO), TimeOfDay::Day);
    let blocking = done(&entered, Action::NextCombatPhase);
    assert!(activations(&done(&blocking, declare_block(gargoyle()))) > 0);
    let attacking = done(&skip_to_attack_phase(&entered), declare(&[gargoyle()]));
    assert!(strong_by(&attacking, card(Rage, 1), ManaPayment::Token));
    assert!(strong_by(
        &attacking,
        card(Rage, 1),
        ManaPayment::Die(GOLD_DIE)
    ));
}

// --- Issue (C11 à C16, C18, C19, L1 à L3, L5) ---

/// En phase d'attaque, `targets` vaincus par une attaque de `amount` (la première carte de la main
/// jouée de côté).
fn beat(state: &GameState, targets: &[arcanum_core::EnemyToken], amount: u8) -> GameState {
    let attacking = done(&skip_to_attack_phase(state), declare(targets));
    let first = attacking.hand(SOLO)[0];
    let attacking = played(&attacking, first, melee(amount));
    done(&attacking, Action::ResolveAttack)
}

fn end_combat(state: &GameState) -> Vec<Event> {
    events_of(&skip_to_attack_phase(state), &Action::EndCombat)
}

/// La première fin de tour proposée.
fn end_turn(state: &GameState) -> GameState {
    let action = legal(state)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("fin du tour proposée");
    done(state, action)
}

fn entered(adventure: Adventure) -> GameState {
    done(&adventure.state(), Action::EnterAdventureSite)
}

// C11 — carte du donjon ; Compendium, *Mana* S14 : conquis, un dé de récompense de la réserve ;
// or ou noir, un sort ; sinon, un artefact. L5 : le noir de jour aussi.
#[test]
fn dungeon_reward_die() {
    let won = beat(&entered(Adventure::default()), &[minotaur()], 5);
    let events = end_combat(&won);
    assert_eq!(events.len(), 3, "{events:?}");
    assert_eq!(
        events[..2],
        [
            Event::CombatEnded { player: SOLO },
            Event::SiteConquered {
                player: SOLO,
                hex: DUNGEON
            },
        ]
    );
    assert!(
        matches!(events[2..], [Event::RewardDieRolled { player: SOLO, .. }]),
        "{events:?}"
    );
    let source = won.source().to_vec();
    let conquered = events[..2].iter().fold(won, apply);
    let after = apply(conquered.clone(), &events[2]);
    assert_eq!(after.source(), source);
    assert_eq!(after.shields().get(&DUNGEON), Some(&SOLO));
    assert_eq!(after.figure(SOLO), Some(DUNGEON));
    let rolled = |color| {
        let state = apply(
            conquered.clone(),
            &Event::RewardDieRolled {
                player: SOLO,
                color,
            },
        );
        (state.spell_rewards(SOLO), state.artifact_rewards(SOLO))
    };
    assert_eq!(rolled(ManaColor::Gold), (1, 0));
    assert_eq!(rolled(ManaColor::Black), (1, 0));
    assert_eq!(rolled(ManaColor::Basic(BasicColor::Green)), (0, 1));
}

// C12 — carte du tombeau ; Compendium, *Combat* S15 : un sort et un artefact, pas de réputation.
#[test]
fn tomb_rewards_spell_and_artifact() {
    let fighting = skip_to_attack_phase(&entered(Adventure {
        figure: TOMB,
        ..Adventure::default()
    }));
    let attacking = done(&fighting, declare(&[swamp_dragon()]));
    let attacking = played(&attacking, card(Swiftness, 1), melee(9));
    let defeat = events_of(&attacking, &Action::ResolveAttack);
    assert!(
        !defeat
            .iter()
            .any(|event| matches!(event, Event::ReputationChanged { .. })),
        "{defeat:?}"
    );
    let won = defeat.iter().fold(attacking, apply);
    let after = end_combat(&won).iter().fold(won, apply);
    assert_eq!(after.shields().get(&TOMB), Some(&SOLO));
    assert_eq!(
        (after.spell_rewards(SOLO), after.artifact_rewards(SOLO)),
        (1, 1)
    );
}

// C13 — cartes de l'antre et du Sombre Terrier : deux cristaux aléatoires ; un artefact et trois
// cristaux aléatoires.
#[test]
fn den_and_grounds_reward_random_crystals() {
    let den = beat(
        &entered(Adventure {
            figure: MONSTER_DEN,
            placed: vec![(gargoyle(), MONSTER_DEN)],
            ..Adventure::default()
        }),
        &[gargoyle()],
        20,
    );
    let events = end_combat(&den);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::SiteConquered {
                player: SOLO,
                hex: MONSTER_DEN
            },
        ]
    );
    let den = events.iter().fold(den, apply);
    assert_eq!(
        (den.crystal_rewards(SOLO), den.artifact_rewards(SOLO)),
        (2, 0)
    );
    assert!(den.enemy_tokens_at(MONSTER_DEN).is_empty());

    let grounds = beat(
        &entered(Adventure {
            figure: SPAWNING_GROUNDS,
            ..Adventure::default()
        }),
        &[minotaur(), medusa()],
        9,
    );
    let grounds = end_combat(&grounds).iter().fold(grounds, apply);
    assert_eq!(grounds.shields().get(&SPAWNING_GROUNDS), Some(&SOLO));
    assert_eq!(
        (
            grounds.crystal_rewards(SOLO),
            grounds.artifact_rewards(SOLO)
        ),
        (3, 1)
    );
}

// C14 — MKUE p. 10, 3.b ; Compendium, *Combat* S18 : au donjon, l'ennemi non vaincu est défaussé.
#[test]
fn failed_dungeon_discards_the_enemy() {
    let fighting = entered(Adventure::default());
    let events = end_combat(&fighting);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::EnemyTokenDiscarded {
                hex: DUNGEON,
                token: minotaur()
            },
        ]
    );
    let after = events.iter().fold(skip_to_attack_phase(&fighting), apply);
    assert!(after.enemy_tokens_at(DUNGEON).is_empty());
    assert!(after.shields().is_empty());
    assert_eq!(after.figure(SOLO), Some(DUNGEON));
    assert_eq!(after.enemy_discard(EnemyColor::Brown), [minotaur()]);
}

// C15 — MKUE p. 10, 3.c : à l'antre, l'ennemi non vaincu reste, et sera combattu sans tirage.
#[test]
fn monster_den_enemy_waits() {
    let fighting = entered(Adventure {
        figure: MONSTER_DEN,
        ..Adventure::default()
    });
    assert_eq!(fighting.enemy_tokens_at(MONSTER_DEN), [minotaur()]);
    let events = end_combat(&fighting);
    assert_eq!(events, [Event::CombatEnded { player: SOLO }]);
    let after = events.iter().fold(skip_to_attack_phase(&fighting), apply);
    assert_eq!(after.enemy_tokens_at(MONSTER_DEN), [minotaur()]);
    let next = end_turn(&after);
    assert_eq!(
        events_of(&next, &Action::EnterAdventureSite),
        [
            Event::AdventureSiteEntered {
                player: SOLO,
                hex: MONSTER_DEN
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(MONSTER_DEN, minotaur())]
            },
        ]
    );
}

// C16 — Compendium, *Miscellaneous* S15 ; Q2 : un jeton vaincu sur deux, l'autre reste ; l'entrée
// suivante complète à deux.
#[test]
fn spawning_grounds_refill_to_two() {
    let fighting = entered(Adventure {
        figure: SPAWNING_GROUNDS,
        brown: vec![minotaur(), medusa(), gargoyle()],
        ..Adventure::default()
    });
    let won = beat(&fighting, &[minotaur()], 5);
    let events = end_combat(&won);
    assert_eq!(events, [Event::CombatEnded { player: SOLO }]);
    let after = events.iter().fold(skip_to_attack_phase(&won), apply);
    assert_eq!(after.enemy_tokens_at(SPAWNING_GROUNDS), [medusa()]);
    let next = end_turn(&after);
    assert_eq!(
        events_of(&next, &Action::EnterAdventureSite),
        [
            Event::AdventureSiteEntered {
                player: SOLO,
                hex: SPAWNING_GROUNDS
            },
            Event::EnemyTokenPlaced {
                token: gargoyle(),
                hex: SPAWNING_GROUNDS
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(SPAWNING_GROUNDS, medusa()), (SPAWNING_GROUNDS, gargoyle())]
            },
        ]
    );
    let again = beat(
        &done(&next, Action::EnterAdventureSite),
        &[medusa(), gargoyle()],
        20,
    );
    assert!(end_combat(&again).contains(&Event::SiteConquered {
        player: SOLO,
        hex: SPAWNING_GROUNDS
    }));
}

// C18 — carte du donjon, *While Conquered* : un nouveau jeton, la gloire seulement.
#[test]
fn conquered_dungeon_for_fame_only() {
    let state = Adventure {
        conquered: vec![DUNGEON],
        ..Adventure::default()
    }
    .state();
    assert!(can_enter(&state));
    let won = beat(&done(&state, Action::EnterAdventureSite), &[minotaur()], 5);
    assert_eq!(won.fame(SOLO), 4);
    assert_eq!(end_combat(&won), [Event::CombatEnded { player: SOLO }]);
}

// C19 — cartes de l'antre et du Sombre Terrier : conquis, ce sont des cases ordinaires.
#[test]
fn conquered_den_is_an_ordinary_space() {
    for site in [MONSTER_DEN, SPAWNING_GROUNDS] {
        let state = Adventure {
            figure: site,
            conquered: vec![site],
            ..Adventure::default()
        }
        .state();
        assert!(!can_enter(&state), "{site:?}");
    }
}

// L1 — spec 034 : assommé dans le donjon, le combat continue ; vaincre le monstre conquiert.
#[test]
fn knocked_out_in_dungeon_still_conquers() {
    let fighting = entered(Adventure::default());
    let blocking = done(&fighting, Action::NextCombatPhase);
    let damage = events_of(&blocking, &Action::NextCombatPhase);
    assert!(
        damage
            .iter()
            .any(|event| matches!(event, Event::KnockedOut { .. })),
        "{damage:?}"
    );
    let hurt = damage.iter().fold(blocking, apply);
    assert_eq!(
        hurt.combat(SOLO).map(|combat| combat.phase),
        Some(CombatPhase::Attack)
    );
    let won = apply(
        hurt,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![minotaur()],
            fame: 4,
        },
    );
    assert!(end_combat(&won).contains(&Event::SiteConquered {
        player: SOLO,
        hex: DUNGEON
    }));
}

// L2 — spec 033 : la créature invoquée n'est pas un jeton du site.
#[test]
fn summoned_creature_is_not_a_site_token() {
    let fighting = entered(Adventure {
        figure: MONSTER_DEN,
        placed: vec![(orc_summoners(), MONSTER_DEN)],
        ..Adventure::default()
    });
    let summoned = done(&fighting, Action::NextCombatPhase);
    assert!(
        summoned
            .combat(SOLO)
            .is_some_and(|combat| combat.enemies[0].summoned.is_some())
    );
    let won = beat(&summoned, &[orc_summoners()], 4);
    assert!(end_combat(&won).contains(&Event::SiteConquered {
        player: SOLO,
        hex: MONSTER_DEN
    }));
}

// L3 — spec 043, L4 : un seul jeton nulle part pour le Sombre Terrier ; le vaincre conquiert.
#[test]
fn spawning_grounds_with_a_single_token_left() {
    let mut events = Adventure {
        figure: SPAWNING_GROUNDS,
        ..Adventure::default()
    }
    .events();
    events.extend(EnemyColor::ALL.map(|color| Event::EnemyPileShuffled {
        color,
        order: if color == EnemyColor::Brown {
            vec![minotaur()]
        } else {
            Vec::new()
        },
    }));
    let state = events.iter().fold(GameState::default(), apply);
    let fighting = done(&state, Action::EnterAdventureSite);
    assert_eq!(
        fighting.combat(SOLO).map(|combat| combat.enemies.len()),
        Some(1)
    );
    let won = beat(&fighting, &[minotaur()], 5);
    assert!(end_combat(&won).contains(&Event::SiteConquered {
        player: SOLO,
        hex: SPAWNING_GROUNDS
    }));
}

// --- Récompenses de fin de tour (C21 à C26, L6) ---

/// La première graine dont les dés de récompense donnent `colors`, dans l'ordre.
fn seed_rolling(colors: &[ManaColor]) -> u64 {
    (0..10_000)
        .find(|&seed| {
            let mut rngs = config(seed).rngs();
            colors
                .iter()
                .all(|&color| roll_die(rngs.rng(RngUsage::RewardDice)) == color)
        })
        .expect("une graine convient")
}

fn events_seeded(state: &GameState, action: &Action, seed: u64) -> Vec<Event> {
    decide(state, SOLO, action, &mut config(seed).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

/// Un tour où `site` a été conquis, Marche jouée de côté.
fn prepared(state: &GameState, site: Hex) -> GameState {
    let won = apply(
        state.clone(),
        &Event::SiteConquered {
            player: SOLO,
            hex: site,
        },
    );
    done(&won, sideways(card(March, 1)))
}

/// La fin du tour (sans défausse) de ce tour, dés tirés avec `seed`.
fn end_turn_after(state: &GameState, site: Hex, seed: u64) -> Vec<Event> {
    events_seeded(
        &prepared(state, site),
        &Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
        seed,
    )
}

const BLUE: ManaColor = ManaColor::Basic(BasicColor::Blue);
const GREEN: ManaColor = ManaColor::Basic(BasicColor::Green);

fn position(events: &[Event], wanted: impl Fn(&Event) -> bool) -> usize {
    events
        .iter()
        .position(wanted)
        .unwrap_or_else(|| panic!("absent de {events:?}"))
}

// C21 — MKUE p. 11, 5.a : un dé par cristal aléatoire, un cristal de la couleur, avant la pioche.
#[test]
fn random_crystals_at_end_of_turn() {
    let state = Adventure::default().state();
    let seed = seed_rolling(&[BLUE, GREEN]);
    let events = end_turn_after(&state, MONSTER_DEN, seed);
    let die = |color| Event::RewardDieRolled {
        player: SOLO,
        color,
    };
    let crystal = |color| Event::CrystalGained {
        player: SOLO,
        color,
    };
    assert_events_in_order(
        &events,
        &[
            Event::TurnEnded { player: SOLO },
            die(BLUE),
            crystal(BasicColor::Blue),
            die(GREEN),
            crystal(BasicColor::Green),
        ],
    );
    let last_die = position(&events, |event| *event == crystal(BasicColor::Green));
    let draw = position(&events, |event| {
        matches!(event, Event::CardDrawn { .. } | Event::TurnStarted { .. })
    });
    assert!(last_die < draw, "{events:?}");
    let after = events.iter().fold(prepared(&state, MONSTER_DEN), apply);
    assert_eq!(after.crystal_rewards(SOLO), 0);
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::Blue), 1);
}

// C22 — MKUE p. 11, 5.a et 6 : noir, 1 gloire, qui peut faire changer de niveau.
#[test]
fn black_die_gives_one_fame() {
    let state = apply(
        Adventure::default().state(),
        &Event::FameGained {
            player: SOLO,
            fame: 2,
        },
    );
    let seed = seed_rolling(&[ManaColor::Black, BLUE]);
    let events = end_turn_after(&state, MONSTER_DEN, seed);
    assert_events_in_order(
        &events,
        &[
            Event::RewardDieRolled {
                player: SOLO,
                color: ManaColor::Black,
            },
            Event::FameGained {
                player: SOLO,
                fame: 1,
            },
            Event::LevelGained {
                player: SOLO,
                level: 2,
            },
        ],
    );
}

// C23 — Compendium, *End of Turn* Q4 : or, la fin du tour attend le choix de la couleur.
#[test]
fn gold_die_waits_for_a_color() {
    let state = Adventure::default().state();
    // Le même générateur pour la fin du tour et le choix : le second dé suit le premier.
    let mut rngs = config(seed_rolling(&[ManaColor::Gold, GREEN])).rngs();
    let end = Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    };
    let events = decide(&prepared(&state, MONSTER_DEN), SOLO, &end, &mut rngs).expect("légale");
    // La fin du tour attend : rien après le dé or.
    assert_eq!(
        events.last(),
        Some(&Event::RewardDieRolled {
            player: SOLO,
            color: ManaColor::Gold
        }),
        "{events:?}"
    );
    let waiting = events.iter().fold(prepared(&state, MONSTER_DEN), apply);
    assert_eq!(waiting.crystal_choices(SOLO), 1);
    assert_eq!(
        legal(&waiting),
        [
            BasicColor::Red,
            BasicColor::Blue,
            BasicColor::White,
            BasicColor::Green
        ]
        .map(|color| Action::ChooseCrystalColor { color })
    );
    let choice = Action::ChooseCrystalColor {
        color: BasicColor::White,
    };
    let chosen = decide(&waiting, SOLO, &choice, &mut rngs).expect("légale");
    assert_eq!(
        chosen[..3],
        [
            Event::CrystalGained {
                player: SOLO,
                color: BasicColor::White
            },
            Event::RewardDieRolled {
                player: SOLO,
                color: GREEN
            },
            Event::CrystalGained {
                player: SOLO,
                color: BasicColor::Green
            },
        ]
    );
    let after = chosen.iter().fold(waiting, apply);
    assert_eq!(
        (after.crystal_choices(SOLO), after.crystal_rewards(SOLO)),
        (0, 0)
    );
    assert!(
        chosen
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. }))
    );
}

/// Trois cristaux de chaque couleur de `full`.
fn with_full(state: GameState, full: &[BasicColor]) -> GameState {
    full.iter()
        .flat_map(|&color| [color; 3])
        .fold(state, |state, color| {
            apply(
                state,
                &Event::CrystalGained {
                    player: SOLO,
                    color,
                },
            )
        })
}

// C24 — MKUE p. 11, 5.a : couleur pleine, rien ; or et tout plein, pas de choix.
#[test]
fn full_inventory_gets_nothing() {
    let state = with_full(Adventure::default().state(), &[BasicColor::Blue]);
    let events = end_turn_after(&state, MONSTER_DEN, seed_rolling(&[BLUE, BLUE]));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CrystalGained { .. })),
        "{events:?}"
    );
    let all = with_full(
        Adventure::default().state(),
        &[
            BasicColor::Red,
            BasicColor::Blue,
            BasicColor::White,
            BasicColor::Green,
        ],
    );
    let gold = seed_rolling(&[ManaColor::Gold, ManaColor::Gold]);
    let events = end_turn_after(&all, MONSTER_DEN, gold);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. })),
        "{events:?}"
    );
    let after = events.iter().fold(prepared(&all, MONSTER_DEN), apply);
    assert_eq!(after.crystal_choices(SOLO), 0);
}

// C25 — MKUE p. 11, 5 : au Sombre Terrier, les cristaux d'abord, puis les artefacts tirés.
#[test]
fn crystals_before_artifact() {
    let state = Adventure {
        figure: SPAWNING_GROUNDS,
        ..Adventure::default()
    }
    .state();
    let state = with_artifacts(state);
    let events = end_turn_after(&state, SPAWNING_GROUNDS, seed_rolling(&[BLUE, BLUE, BLUE]));
    let dice: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(_, event)| matches!(event, Event::RewardDieRolled { .. }))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(dice.len(), 3, "{events:?}");
    let drawn = position(&events, |event| {
        matches!(event, Event::ArtifactsDrawn { .. })
    });
    assert!(dice[2] < drawn, "{events:?}");
    let after = events
        .iter()
        .fold(prepared(&state, SPAWNING_GROUNDS), apply);
    assert!(
        legal(&after)
            .iter()
            .all(|action| matches!(action, Action::TakeArtifact { .. }))
    );
}

/// La pioche d'artefacts mélangée : deux artefacts sur le dessus.
fn with_artifacts(state: GameState) -> GameState {
    let order = [
        arcanum_core::Artifact::BannerOfGlory,
        arcanum_core::Artifact::EndlessBagOfGold,
    ]
    .map(|card| arcanum_core::CommonCard::Deed(artifact(card)))
    .to_vec();
    apply(
        state,
        &Event::CommonDeckShuffled {
            deck: arcanum_core::CommonDeck::Artifacts,
            order,
        },
    )
}

// C26 — spec 024 : le retour en arrière ne passe pas avant un dé de récompense.
#[test]
fn reward_die_blocks_undo() {
    assert!(
        Event::RewardDieRolled {
            player: SOLO,
            color: GREEN
        }
        .blocks_undo()
    );
}

// L6 — specs 041 et 043 : offre de sorts et pioche d'artefacts vides, les récompenses du tombeau
// sont perdues et le tour suivant commence.
#[test]
fn tomb_rewards_lost_when_nothing_to_take() {
    let state = Adventure {
        figure: TOMB,
        ..Adventure::default()
    }
    .state();
    let events = end_turn_after(&state, TOMB, 0);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. })),
        "{events:?}"
    );
}

// --- Clairière magique (C27 à C33, L8, L9) ---

fn end(discard: Vec<CardId>, glade: Option<WoundPile>) -> Action {
    Action::EndTurn {
        discard,
        reclaim: None,
        glade,
        steady_tempo: false,
    }
}

/// Sur la clairière, une Blessure en main avec Marche, Rage, Détermination et Célérité.
fn glade_with_wound() -> Adventure {
    Adventure {
        figure: GLADE,
        hand: vec![
            card(March, 1),
            card(Rage, 1),
            card(Determination, 1),
            card(Swiftness, 1),
            wound(1),
        ],
        ..Adventure::default()
    }
}

/// Marche jouée de côté : la fin du tour sans défausse est permise.
fn marched(state: &GameState) -> GameState {
    done(state, sideways(card(March, 1)))
}

fn drawn(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, Event::CardDrawn { .. }))
        .count()
}

// C27 — carte de la clairière ; MKUE p. 11, 4.a : jeter une Blessure de la main, une carte de
// plus piochée.
#[test]
fn glade_throws_a_wound_from_hand() {
    let state = marched(&glade_with_wound().state());
    let actions = legal(&state);
    let without = actions.iter().position(|a| *a == end(Vec::new(), None));
    let from_hand = actions
        .iter()
        .position(|a| *a == end(Vec::new(), Some(WoundPile::Hand)));
    assert!(
        matches!((without, from_hand), (Some(a), Some(b)) if a < b),
        "{actions:?}"
    );
    assert!(!actions.contains(&end(Vec::new(), Some(WoundPile::DiscardPile))));
    let plain = events_of(&state, &end(Vec::new(), None));
    let thrown = events_of(&state, &end(Vec::new(), Some(WoundPile::Hand)));
    assert_events_in_order(
        &thrown,
        &[
            Event::TurnEnded { player: SOLO },
            Event::WoundThrownAway {
                player: SOLO,
                from: WoundPile::Hand,
            },
        ],
    );
    assert_eq!(drawn(&thrown), drawn(&plain) + 1);
    let after = thrown.iter().fold(state, apply);
    assert!(!after.hand(SOLO).contains(&wound(1)));
    assert!(!after.discard_pile(SOLO).contains(&wound(1)));
}

// C28 — FAQ p3 ; Compendium, *Healing* S2 : la Blessure de la défausse ; sans Blessure nulle part,
// ou hors d'une clairière, pas de choix.
#[test]
fn glade_throws_from_discard_pile() {
    let state = apply(
        glade_with_wound().state(),
        &Event::CardDiscarded {
            player: SOLO,
            card: wound(1),
        },
    );
    let state = marched(&state);
    let actions = legal(&state);
    assert!(actions.contains(&end(Vec::new(), Some(WoundPile::DiscardPile))));
    assert!(!actions.contains(&end(Vec::new(), Some(WoundPile::Hand))));
    let events = events_of(&state, &end(Vec::new(), Some(WoundPile::DiscardPile)));
    assert!(events.contains(&Event::WoundThrownAway {
        player: SOLO,
        from: WoundPile::DiscardPile,
    }));
    let after = events.iter().fold(state, apply);
    assert!(!after.discard_pile(SOLO).contains(&wound(1)));

    let clean = marched(
        &Adventure {
            figure: GLADE,
            ..Adventure::default()
        }
        .state(),
    );
    let elsewhere = marched(
        &Adventure {
            figure: NEAR_DUNGEON,
            ..glade_with_wound()
        }
        .state(),
    );
    for state in [&clean, &elsewhere] {
        assert!(
            !legal(state)
                .iter()
                .any(|action| matches!(action, Action::EndTurn { glade: Some(_), .. }))
        );
    }
    refused(&elsewhere, end(Vec::new(), Some(WoundPile::Hand)));
}

// C29, L9 — Compendium, *Resting* S5, *Healing* S3 : après un repos aussi ; ce n'est pas un soin.
// Une Blessure défaussée par le repos n'est pas dans la défausse au moment du choix.
#[test]
fn glade_is_not_healing() {
    let state = Adventure {
        hand: vec![
            card(March, 1),
            card(Rage, 1),
            card(Swiftness, 1),
            wound(1),
            wound(2),
        ],
        ..glade_with_wound()
    }
    .state();
    let rest = vec![card(March, 1), wound(1)];
    let actions = legal(&state);
    assert!(actions.contains(&end(rest.clone(), Some(WoundPile::Hand))));
    assert!(!actions.contains(&end(rest.clone(), Some(WoundPile::DiscardPile))));
    assert!(!actions.contains(&end(
        vec![card(March, 1), wound(1), wound(2)],
        Some(WoundPile::Hand)
    )));
    let events = events_of(&state, &end(rest, Some(WoundPile::Hand)));
    assert!(events.contains(&Event::WoundThrownAway {
        player: SOLO,
        from: WoundPile::Hand,
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::WoundsHealed { .. } | Event::UnitHealed { .. }))
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.healing_points(SOLO), 0);
}

/// Le deck vidé : la main seule reste.
fn empty_deck(state: GameState) -> GameState {
    let hand = state.hand(SOLO).to_vec();
    let state = apply(
        state,
        &Event::DeckShuffled {
            player: SOLO,
            order: hand.clone(),
        },
    );
    hand.into_iter().fold(state, |state, card| {
        apply(state, &Event::CardDrawn { player: SOLO, card })
    })
}

fn after_turn_started(events: &[Event]) -> Option<&Event> {
    let start = position(events, |event| matches!(event, Event::TurnStarted { .. }));
    events.get(start + 1)
}

// C30, C32 — Compendium, *End of Round* S1 ; MKUE p. 6, 4 : l'annonce ne jette rien ; la nouvelle
// manche (de Nuit) donne un jeton noir après `TurnStarted`.
#[test]
fn no_glade_when_announcing_end_of_round() {
    let state = empty_deck(glade_with_wound().state());
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::WoundThrownAway { .. }))
    );
    assert_eq!(
        after_turn_started(&events),
        Some(&Event::ManaTokenGained {
            player: SOLO,
            color: ManaColor::Black,
        }),
        "{events:?}"
    );
}

// C31 — carte de la clairière ; Compendium, *Turn Start* II : or de jour, noir de nuit, rien
// ailleurs.
#[test]
fn glade_gives_a_mana_token_at_turn_start() {
    for (time, color) in [
        (TimeOfDay::Day, ManaColor::Gold),
        (TimeOfDay::Night, ManaColor::Black),
    ] {
        let state = marched(
            &Adventure {
                figure: GLADE,
                time,
                ..Adventure::default()
            }
            .state(),
        );
        let events = events_of(&state, &end(Vec::new(), None));
        assert_eq!(
            after_turn_started(&events),
            Some(&Event::ManaTokenGained {
                player: SOLO,
                color
            }),
            "{events:?}"
        );
    }
    let elsewhere = marched(
        &Adventure {
            figure: NEAR_DUNGEON,
            ..Adventure::default()
        }
        .state(),
    );
    let events = events_of(&elsewhere, &end(Vec::new(), None));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ManaTokenGained { .. }))
    );
}

// C32 — MKUE p. 6, 4 : le tour perdu par l'annonce obligatoire ne donne pas de jeton.
#[test]
fn glade_token_at_new_round_not_when_forfeiting() {
    let state = marched(&empty_deck(
        Adventure {
            figure: GLADE,
            ..Adventure::default()
        }
        .state(),
    ));
    let rest = state.hand(SOLO).to_vec();
    let events = events_of(&state, &end(rest, None));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. })),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ManaTokenGained { .. })),
        "{events:?}"
    );
}

// C33 — spec 025 : le jeton or de la clairière paie un effet renforcé de jour, et revient à la
// réserve à la fin du tour.
#[test]
fn glade_token_is_an_ordinary_token() {
    let state = marched(
        &Adventure {
            figure: GLADE,
            hand: vec![
                card(March, 1),
                card(Rage, 1),
                card(Determination, 1),
                card(Swiftness, 1),
                card(Promise, 1),
            ],
            dice: vec![
                ManaColor::Basic(BasicColor::Blue),
                ManaColor::Basic(BasicColor::Blue),
                ManaColor::Basic(BasicColor::White),
            ],
            ..Adventure::default()
        }
        .state(),
    );
    let next = done(&state, end(Vec::new(), None));
    assert_eq!(next.mana_tokens(SOLO), [ManaColor::Gold]);
    let marching = next
        .hand(SOLO)
        .iter()
        .copied()
        .find(|card| card.card() == arcanum_core::DeedCard::Basic(March));
    if let Some(march) = marching {
        assert!(strong_by(&next, march, ManaPayment::Token));
    }
    let ended = apply(next, &Event::TurnEnded { player: SOLO });
    assert!(ended.mana_tokens(SOLO).is_empty());
}

// L8 — MKUE p. 11, 4 puis 5 : la Blessure se jette avant les cristaux aléatoires.
#[test]
fn glade_before_rewards() {
    let state = prepared(&glade_with_wound().state(), MONSTER_DEN);
    let events = events_of(&state, &end(Vec::new(), Some(WoundPile::Hand)));
    let thrown = position(&events, |event| {
        matches!(event, Event::WoundThrownAway { .. })
    });
    let die = position(&events, |event| {
        matches!(event, Event::RewardDieRolled { .. })
    });
    assert!(thrown < die, "{events:?}");
}

// --- Décompte, vue (C34, C36, C37) ---

// C34 — MKUE p. 15, *The Greatest Adventurer* : 2 points par Bouclier sur un site d'aventure.
#[test]
fn adventurer_counts_adventure_shields() {
    let state = Adventure {
        conquered: vec![DUNGEON, MONSTER_DEN],
        ..Adventure::default()
    }
    .state();
    let score = final_score(&state, SOLO);
    assert_eq!((score.adventure_shields, score.shields), (2, 0));
    assert_eq!(score.points(Achievement::Adventurer), 4);
}

// C36 — la vue donne le site d'aventure en cours, les cristaux aléatoires et les choix en attente.
#[test]
fn view_shows_adventure_state() {
    let fighting = entered(Adventure {
        figure: MONSTER_DEN,
        placed: vec![(gargoyle(), MONSTER_DEN)],
        ..Adventure::default()
    });
    let seen = view(&fighting, SOLO);
    assert_eq!(seen.adventure, Some(MONSTER_DEN));
    assert_eq!(
        seen.enemy_tokens.get(&MONSTER_DEN).map(Vec::as_slice),
        Some(&[gargoyle()][..])
    );
    let won = prepared(&Adventure::default().state(), MONSTER_DEN);
    assert_eq!(view(&won, SOLO).crystal_rewards, 2);
    let waiting = [
        Event::TurnEnded { player: SOLO },
        Event::RewardDieRolled {
            player: SOLO,
            color: ManaColor::Gold,
        },
    ]
    .iter()
    .fold(won, apply);
    let seen = view(&waiting, SOLO);
    assert_eq!((seen.crystal_rewards, seen.crystal_choices), (1, 1));
}

// C45 (vue) — carte du donjon : le dé de récompense lancé à la conquête reste dans la vue
// jusqu'à la fin du tour, pour que l'écran l'annonce ; pas les dés des cristaux aléatoires.
#[test]
fn view_shows_the_dungeon_reward_die() {
    let won = skip_to_attack_phase(&beat(&entered(Adventure::default()), &[minotaur()], 5));
    let events = events_of(&won, &Action::EndCombat);
    let color = events
        .iter()
        .find_map(|event| match event {
            Event::RewardDieRolled { color, .. } => Some(*color),
            _ => None,
        })
        .expect("dé lancé");
    let conquered = events.iter().fold(won, apply);
    assert_eq!(view(&conquered, SOLO).reward_die, Some(color));
    assert_eq!(view(&end_turn(&conquered), SOLO).reward_die, None);
    let crystals = [
        Event::TurnEnded { player: SOLO },
        Event::RewardDieRolled {
            player: SOLO,
            color: ManaColor::Gold,
        },
    ]
    .iter()
    .fold(prepared(&Adventure::default().state(), MONSTER_DEN), apply);
    assert_eq!(view(&crystals, SOLO).reward_die, None);
}

// C37 — un jeton resté sur un antre est dans `enemy_tokens` de la vue, sans être un ennemi
// errant : la case reste accessible.
#[test]
fn view_lists_adventure_tokens() {
    let state = done(&den_with_gargoyle(), basic(card(March, 1)));
    let state = done(&state, basic(card(Swiftness, 1)));
    let seen = view(&state, SOLO);
    assert_eq!(
        seen.enemy_tokens.get(&MONSTER_DEN).map(Vec::as_slice),
        Some(&[gargoyle()][..])
    );
    assert!(legal(&state).contains(&Action::MoveHero { to: MONSTER_DEN }));
}

// --- Joueur de test (C38) ---

/// Le choix du joueur « premier choix » parmi les actions légales.
fn first_choice(state: &GameState) -> Action {
    let actions = legal(state);
    let mut player = premier_choix::FirstChoice::default();
    player.request(&view(state, SOLO), &actions);
    let index = player.poll().expect("un choix");
    actions[index].clone()
}

/// Toute la main jouée de côté pour l'influence : plus de carte à jouer ni de pas à faire.
fn hand_played(state: GameState) -> GameState {
    let hand: Vec<CardId> = state
        .hand(SOLO)
        .iter()
        .copied()
        .filter(|card| !card.card().is_wound())
        .collect();
    hand.into_iter().fold(state, |state, card| {
        done(
            &state,
            Action::PlayCard {
                card,
                play: CardPlay::Sideways(SidewaysEffect::Influence),
            },
        )
    })
}

// C38 — le joueur de test n'entre pas dans les sites d'aventure et finit son tour sans jeter de
// Blessure : ses parcours ne changent pas.
#[test]
fn first_choice_ignores_adventure_sites() {
    let dungeon = hand_played(Adventure::default().state());
    assert_eq!(legal(&dungeon)[0], Action::EnterAdventureSite);
    assert!(matches!(
        first_choice(&dungeon),
        Action::EndTurn { glade: None, .. }
    ));
    let glade = hand_played(glade_with_wound().state());
    assert!(legal(&glade).contains(&end(Vec::new(), Some(WoundPile::Hand))));
    assert_eq!(first_choice(&glade), end(Vec::new(), None));
}
