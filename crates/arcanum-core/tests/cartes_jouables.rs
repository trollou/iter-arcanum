//! Ce qui se joue maintenant, d'après la vue : la même règle que les actions légales (issue privée 111).

use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, CardPlay, Effect, Element, Event, GameState,
    SidewaysEffect, Unit, UnitCard, apply, legal_actions, view,
};

use BasicAction::*;

mod arene;
use arene::*;

fn attack(kind: AttackKind) -> Effect {
    Effect::Attack {
        amount: 2,
        element: Element::Physical,
        kind,
    }
}

/// Vrai si la vue dit l'effet jouable maintenant.
fn playable(state: &GameState, effect: Effect) -> bool {
    view(state, SOLO).is_playable_now(effect)
}

// MKUE p. 6, 5.c : hors combat, ni attaque ni parade ; le mouvement et l'influence, oui.
#[test]
fn out_of_combat_move_but_no_attack() {
    let state = arena(VILLAGE, &[card(March, 1)], &[]);
    assert!(playable(&state, Effect::Move(2)));
    assert!(playable(&state, Effect::Influence(2)));
    assert!(!playable(&state, attack(AttackKind::Melee)));
}

// MKUE p. 8, 2 (spec 040, C14) : en interaction, le mouvement ne sert plus, l'influence si.
#[test]
fn no_move_during_interaction() {
    let state = done(&arena(VILLAGE, &[card(March, 1)], &[]), Action::Interact);
    assert!(!playable(&state, Effect::Move(2)));
    assert!(playable(&state, Effect::Influence(2)));
}

// MKUE p. 9, *Ranged and Siege Attack Phase* 8 et p. 8, 6 : dans une attaque à distance en cours,
// la distance mais pas la mêlée ; le mana toujours, le mouvement jamais (p. 6, 6.e).
#[test]
fn in_combat_follows_the_attack_in_progress() {
    let state = fight(
        VILLAGE,
        &[card(Swiftness, 1)],
        &[(prowlers(), ORCS_2)],
        &[ORCS_2],
    );
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    assert!(playable(&state, attack(AttackKind::Ranged)));
    assert!(!playable(&state, attack(AttackKind::Melee)));
    assert!(playable(&state, Effect::GainToken(BasicColor::Blue)));
    assert!(!playable(&state, Effect::Move(2)));
}

/// Le combat contre les orques fini depuis la phase d'attaque, avec `hand` en main.
fn after_combat(hand: &[arcanum_core::CardId]) -> GameState {
    let state = fight(VILLAGE, hand, &[(prowlers(), ORCS_2)], &[ORCS_2]);
    done(&to_attack_phase(&state), Action::EndCombat)
}

/// Vrai si une action légale génère du Mouvement : effet Mouvement d'une carte ou d'une unité,
/// ou carte de côté pour Mouvement.
fn offers_move(state: &GameState) -> bool {
    legal_actions(state, SOLO)
        .iter()
        .any(|action| match action {
            Action::PlayCard {
                play: CardPlay::Sideways(SidewaysEffect::Move),
                ..
            } => true,
            Action::PlayCard {
                card,
                play: CardPlay::Basic { option, .. },
            } => card
                .card()
                .definition()
                .and_then(|definition| definition.basic.option(*option))
                .is_some_and(|effect| matches!(effect, Effect::Move(_))),
            Action::ActivateUnit {
                unit,
                ability,
                option,
                ..
            } => unit
                .unit
                .definition()
                .abilities
                .get(usize::from(*ability))
                .and_then(|definition| {
                    definition
                        .options()
                        .get(usize::from(*option))
                        .map(|choice| choice.effect)
                })
                .is_some_and(|effect| matches!(effect, Effect::Move(_))),
            _ => false,
        })
}

// MKUE p. 6, 5 (dernier tiret), 6.d et 6.e (issue privée 164) : le combat fini, l'action du tour est
// faite ; le mouvement ne sert plus, ni par une carte ni de côté.
#[test]
fn no_move_after_combat() {
    let state = after_combat(&[card(March, 1), card(Promise, 1)]);
    assert!(!playable(&state, Effect::Move(2)));
    assert!(!offers_move(&state));
}

// MKUE p. 6, 5 (dernier tiret) (issue privée 164) : avant l'action, une unité donne du Mouvement ;
// le combat fini, plus.
#[test]
fn no_unit_move_after_combat() {
    let peasants = Event::UnitRecruited {
        player: SOLO,
        unit: UnitCard::new(Unit::Peasants, 1),
        cost: 0,
    };
    let before = apply(arena(VILLAGE, &[card(Promise, 1)], &[]), &peasants);
    assert!(offers_move(&before));
    let state = apply(after_combat(&[card(Promise, 1)]), &peasants);
    assert!(!offers_move(&state));
}

/// Chaque option sans mana de l'effet de base d'une carte de la main est une action légale si et
/// seulement si la vue la dit jouable.
fn assert_view_agrees_with_legal_actions(state: &GameState) {
    let legal = legal_actions(state, SOLO);
    for held in view(state, SOLO).hand {
        let Some(definition) = held.card().definition() else {
            continue;
        };
        for option in 0u8.. {
            let Some(effect) = definition.basic.option(option) else {
                break;
            };
            if effect.mana_cost().is_some() {
                continue;
            }
            let offered = legal.iter().any(|action| {
                matches!(action, Action::PlayCard { card, play: CardPlay::Basic { option: o, .. } }
                    if *card == held && *o == option)
            });
            assert_eq!(
                offered,
                playable(state, effect),
                "{held:?}, option {option} : {effect:?}"
            );
        }
    }
}

#[test]
fn view_agrees_with_legal_actions() {
    let hand = [
        card(March, 1),
        card(Rage, 1),
        card(Promise, 1),
        card(Improvisation, 1),
        card(Swiftness, 1),
    ];
    let free = arena(VILLAGE, &hand, &[]);
    let interacting = done(&free, Action::Interact);
    let fighting = fight(VILLAGE, &hand, &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let attacking = done(&fighting, declare(&[foe(&fighting, ORCS_2)]));
    let ended = done(&to_attack_phase(&fighting), Action::EndCombat);
    for state in [&free, &interacting, &fighting, &attacking, &ended] {
        assert_view_agrees_with_legal_actions(state);
    }
}
