//! Scénarios de la spec 058 (compétences : second lot).

mod arene;

use arcanum_core::{
    Action, AdvancedAction, Artifact, AttackKind, BasicAction, BasicColor, CardId, CardPlay,
    CombatPhase, DeedCard, Effect, Element, Event, GameRunner, GameState, Hex, ManaColor,
    ManaPayment, Offer, PolarSource, SidewaysEffect, Skill, SkillMana, SkillState, SkillTarget,
    Spell, TimeOfDay, Unit, UnitCard, apply, legal_actions,
};
use arene::*;

fn spell(card: Spell) -> CardId {
    CardId::new(DeedCard::Spell(card), 1)
}

// --- Aides ---

fn owning(state: GameState, skills: &[Skill]) -> GameState {
    skills.iter().fold(state, |state, &skill| {
        apply(
            state,
            &Event::SkillGained {
                player: SOLO,
                skill,
            },
        )
    })
}

fn use_skill(skill: Skill, option: u8) -> Action {
    Action::UseSkill {
        skill,
        option,
        unit: None,
        target: None,
    }
}

fn skill_uses(state: &GameState, skill: Skill) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::UseSkill { skill: used, .. } if *used == skill))
        .collect()
}

fn used_effect(state: &GameState, action: &Action) -> Option<Effect> {
    events_of(state, action)
        .iter()
        .find_map(|event| match event {
            Event::SkillUsed { effect, .. } => Some(*effect),
            _ => None,
        })
}

fn crystals(state: GameState, colors: &[BasicColor]) -> GameState {
    colors.iter().fold(state, |state, &color| {
        apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        )
    })
}

/// Combat engagé contre les Rôdeurs depuis le village.
fn fighting(skills: &[Skill]) -> GameState {
    owning(
        fight(VILLAGE, &move_hand(), &[(prowlers(), ORCS_2)], &[ORCS_2]),
        skills,
    )
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn foresters() -> UnitCard {
    UnitCard::new(Unit::Foresters, 1)
}

fn crossbowmen() -> UnitCard {
    UnitCard::new(Unit::UtemCrossbowmen, 1)
}

fn herbalists() -> UnitCard {
    UnitCard::new(Unit::Herbalists, 1)
}

// --- C. Points ---

// C18 — texte ; Compendium, *Glittering Fortune* S1 : Influence 1 par couleur de cristal, pendant
// l'interaction seulement.
#[test]
fn glittering_fortune_counts_crystal_colors() {
    let base = crystals(
        owning(
            arena(VILLAGE, &move_hand(), &[]),
            &[Skill::GlitteringFortune],
        ),
        &[
            BasicColor::Red,
            BasicColor::Blue,
            BasicColor::Blue,
            BasicColor::White,
        ],
    );
    assert!(skill_uses(&base, Skill::GlitteringFortune).is_empty());
    let interacting = done(&base, Action::Interact);
    assert_eq!(
        skill_uses(&interacting, Skill::GlitteringFortune),
        [use_skill(Skill::GlitteringFortune, 0)]
    );
    let before = interacting.influence_points(SOLO);
    let after = done(&interacting, use_skill(Skill::GlitteringFortune, 0));
    assert_eq!(after.interaction_influence(SOLO), 3);
    assert_eq!(after.influence_points(SOLO), before);
    // Les cristaux comptés restent (S1).
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::Blue), 2);
    // L4 : sans cristal, pas proposée.
    let poor = done(
        &owning(
            arena(VILLAGE, &move_hand(), &[]),
            &[Skill::GlitteringFortune],
        ),
        Action::Interact,
    );
    assert!(skill_uses(&poor, Skill::GlitteringFortune).is_empty());
}

// C19 — texte (Q3) : l'Influence d'interaction paie un recrutement, pas Apprentissage.
#[test]
fn glittering_fortune_influence_only_for_interaction() {
    let learning = CardId::new(DeedCard::Advanced(AdvancedAction::Learning), 1);
    let units = Units {
        hand: vec![learning, card(BasicAction::March, 1)],
        offer: vec![peasants()],
        ..Units::default()
    };
    let mut events = units.events();
    events.extend(advanced_offer(&[advanced(AdvancedAction::BloodRage)]));
    let state = crystals(
        owning(
            events.iter().fold(GameState::default(), apply),
            &[Skill::GlitteringFortune],
        ),
        &BasicColor::ALL,
    );
    let state = done(&state, Action::Interact);
    let recruit = Action::RecruitUnit {
        unit: peasants(),
        disband: None,
        bonds: false,
    };
    assert!(!legal_actions(&state, SOLO).contains(&recruit));
    let rich = done(&state, use_skill(Skill::GlitteringFortune, 0));
    assert_eq!(rich.interaction_influence(SOLO), 4);
    // Peasants : 4 Influence, payés par l'Influence d'interaction d'abord.
    let recruited = done(&rich, recruit);
    assert_eq!(recruited.interaction_influence(SOLO), 0);
    assert_eq!(
        recruited.influence_points(SOLO),
        rich.influence_points(SOLO)
    );
    // Apprentissage : Influence 2 et 6 pour une action avancée ; les 4 d'interaction ne servent pas.
    let learned = done(&rich, basic(learning));
    assert!(!state.offer(Offer::AdvancedActions).is_empty());
    assert!(
        !legal_actions(&learned, SOLO)
            .iter()
            .any(|action| matches!(action, Action::LearnAdvancedAction { .. }))
    );
}

// C20 — texte : Mouvement 1 par unité disponible et non blessée, 3 au plus.
#[test]
fn forward_march_counts_ready_unwounded_units() {
    let mut events = Units {
        owned: vec![peasants(), foresters(), crossbowmen()],
        ..Units::default()
    }
    .events();
    events.push(Event::UnitWounded {
        player: SOLO,
        unit: foresters(),
        wounds: 1,
    });
    events.push(Event::UnitActivated {
        player: SOLO,
        unit: crossbowmen(),
        ability: 1,
        option: 0,
        effect: Effect::Nothing,
        enemy: None,
    });
    let state = owning(
        events.iter().fold(GameState::default(), apply),
        &[Skill::ForwardMarch],
    );
    assert_eq!(
        used_effect(&state, &use_skill(Skill::ForwardMarch, 0)),
        Some(Effect::Move(1))
    );
    let before = state.move_points(SOLO);
    let after = done(&state, use_skill(Skill::ForwardMarch, 0));
    assert_eq!(after.move_points(SOLO), before + 1);

    let four = owning(
        Units {
            owned: vec![peasants(), foresters(), crossbowmen(), herbalists()],
            ..Units::default()
        }
        .state(),
        &[Skill::ForwardMarch],
    );
    assert_eq!(
        used_effect(&four, &use_skill(Skill::ForwardMarch, 0)),
        Some(Effect::Move(3))
    );
    // Aucune unité : pas proposée.
    let alone = owning(Units::default().state(), &[Skill::ForwardMarch]);
    assert!(skill_uses(&alone, Skill::ForwardMarch).is_empty());
}

// C21 — texte ; spec 034 : une Blessure dans la défausse, une carte piochée.
#[test]
fn i_feel_no_pain_discards_wound_and_draws() {
    let hand = [
        wound(1),
        wound(2),
        card(BasicAction::March, 1),
        card(BasicAction::Rage, 1),
    ];
    let state = owning(arena(VILLAGE, &hand, &[]), &[Skill::IFeelNoPain]);
    assert_eq!(
        skill_uses(&state, Skill::IFeelNoPain),
        [use_skill(Skill::IFeelNoPain, 0)]
    );
    let top = state.deck(SOLO)[0];
    let events = events_of(&state, &use_skill(Skill::IFeelNoPain, 0));
    assert_events_in_order(
        &events,
        &[
            Event::CardDiscarded {
                player: SOLO,
                card: wound(1),
            },
            Event::CardDrawn {
                player: SOLO,
                card: top,
            },
        ],
    );
    let after = events.iter().fold(state, apply);
    let wounds = |cards: &[CardId]| cards.iter().filter(|card| card.card().is_wound()).count();
    assert_eq!(wounds(after.hand(SOLO)), 1);
    assert_eq!(wounds(after.discard_pile(SOLO)), 1);
    assert!(after.hand(SOLO).contains(&top));
    // Ni Blessure en main, ni combat.
    let healthy = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::IFeelNoPain]);
    assert!(skill_uses(&healthy, Skill::IFeelNoPain).is_empty());
    assert!(skill_uses(&fighting(&[Skill::IFeelNoPain]), Skill::IFeelNoPain).is_empty());
    // Deck vide : la Blessure part, rien n'est pioché.
    let empty = owning(
        state_with_deck(vec![wound(1), card(BasicAction::March, 1)], 2),
        &[Skill::IFeelNoPain],
    );
    let events = events_of(&empty, &use_skill(Skill::IFeelNoPain, 0));
    assert!(events.contains(&Event::CardDiscarded {
        player: SOLO,
        card: wound(1),
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
}

// --- A. Cartes jouées de côté ---

fn skill_sideways(
    card: CardId,
    sideways: SidewaysEffect,
    skill: Skill,
    option: u8,
    mana: Option<SkillMana>,
) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::SidewaysSkill {
            sideways,
            skill,
            option,
            mana,
        },
    }
}

/// Les jeux de côté légaux avec cette compétence : la carte, l'option, le mana et l'effet.
fn skill_sideways_plays(
    state: &GameState,
    skill: Skill,
) -> Vec<(CardId, u8, Option<SkillMana>, Effect)> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard {
                card,
                play:
                    play @ CardPlay::SidewaysSkill {
                        skill: used,
                        option,
                        mana,
                        ..
                    },
            } if used == skill => card.effect(play).map(|effect| (card, option, mana, effect)),
            _ => None,
        })
        .collect()
}

fn token_mana(color: ManaColor) -> Option<SkillMana> {
    Some(SkillMana {
        color,
        payment: ManaPayment::Token,
    })
}

fn tokens(state: GameState, colors: &[ManaColor]) -> GameState {
    colors.iter().fold(state, |state, &color| {
        apply(
            state,
            &Event::ManaTokenGained {
                player: SOLO,
                color,
            },
        )
    })
}

fn red() -> ManaColor {
    ManaColor::Basic(BasicColor::Red)
}

fn green() -> ManaColor {
    ManaColor::Basic(BasicColor::Green)
}

fn attack(amount: u8) -> Effect {
    Effect::Attack {
        amount,
        element: Element::Physical,
        kind: AttackKind::Melee,
    }
}

fn end_turn_without_discard() -> Action {
    Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

/// En phase d'attaque contre les Rôdeurs, la main `hand`, l'attaque déclarée.
fn attacking(hand: &[CardId], skills: &[Skill]) -> GameState {
    let state = skip_to_attack_phase(&owning(
        fight(VILLAGE, hand, &[(prowlers(), ORCS_2)], &[ORCS_2]),
        skills,
    ));
    done(&state, declare(&[prowlers()]))
}

// C2 — texte ; Compendium, *Ritual of Pain* S2 : une Blessure de côté, Mouvement 2, dans la zone
// de jeu puis la défausse.
#[test]
fn power_of_pain_plays_wound_sideways() {
    let hand = [
        wound(1),
        card(BasicAction::March, 1),
        card(BasicAction::Rage, 1),
    ];
    let state = owning(arena(VILLAGE, &hand, &[]), &[Skill::PowerOfPain]);
    let play = skill_sideways(wound(1), SidewaysEffect::Move, Skill::PowerOfPain, 0, None);
    let events = events_of(&state, &play);
    assert!(events.contains(&Event::SkillUsed {
        player: SOLO,
        skill: Skill::PowerOfPain,
        option: 0,
        effect: Effect::Nothing,
    }));
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(after.move_points(SOLO), state.move_points(SOLO) + 2);
    assert!(
        after
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == wound(1))
    );
    assert!(!after.hand(SOLO).contains(&wound(1)));
    let ended = done(&after, end_turn_without_discard());
    assert!(ended.discard_pile(SOLO).contains(&wound(1)));
}

// C3 — spec 034 : sans la compétence, pas de Blessure de côté ; avec, seulement une Blessure.
#[test]
fn power_of_pain_only_for_wounds() {
    let hand = [wound(1), card(BasicAction::March, 1)];
    let state = owning(arena(VILLAGE, &hand, &[]), &[Skill::PowerOfPain]);
    let plays = skill_sideways_plays(&state, Skill::PowerOfPain);
    assert!(!plays.is_empty());
    assert!(plays.iter().all(|(card, ..)| *card == wound(1)));
    let without = arena(VILLAGE, &hand, &[]);
    refused(
        &without,
        skill_sideways(wound(1), SidewaysEffect::Move, Skill::PowerOfPain, 0, None),
    );
}

// C4 — Compendium, *Minimum Turn* S2 : la Blessure de côté est une carte jouée.
#[test]
fn wound_played_sideways_counts_as_played() {
    let state = owning(
        arena(VILLAGE, &[wound(1), wound(2)], &[]),
        &[Skill::PowerOfPain],
    );
    refused(&state, end_turn_without_discard());
    let after = done(
        &state,
        skill_sideways(wound(1), SidewaysEffect::Move, Skill::PowerOfPain, 0, None),
    );
    assert!(legal_actions(&after, SOLO).contains(&end_turn_without_discard()));
}

// C5 — texte ; Compendium, *I Don't Give a Damn* S2, S3 : +2, +3 pour une action avancée, un
// sort ou un artefact ; jamais une Blessure.
#[test]
fn i_dont_give_a_damn_values() {
    let rage = card(BasicAction::Rage, 1);
    let blood_rage = advanced(AdvancedAction::BloodRage);
    let banner = artifact(Artifact::BannerOfGlory);
    let state = attacking(
        &[rage, blood_rage, banner, wound(1)],
        &[Skill::IDontGiveADamn],
    );
    let plays = skill_sideways_plays(&state, Skill::IDontGiveADamn);
    let effect_of = |wanted: CardId| {
        plays
            .iter()
            .find(|(card, ..)| *card == wanted)
            .map(|(.., effect)| *effect)
    };
    assert_eq!(effect_of(rage), Some(attack(2)));
    assert_eq!(effect_of(blood_rage), Some(attack(3)));
    assert_eq!(effect_of(banner), Some(attack(3)));
    assert_eq!(effect_of(wound(1)), None);
}

// C6 — Compendium, *Who Needs Magic* Q1 : +3 sans dé pris ; après, plus de dé ce tour.
#[test]
fn who_needs_magic_three_forbids_dice() {
    let march = card(BasicAction::March, 1);
    let stamina = card(BasicAction::Stamina, 1);
    let mana_draw = card(BasicAction::ManaDraw, 1);
    let state = owning(
        arena(VILLAGE, &[march, stamina, mana_draw], &[]),
        &[Skill::WhoNeedsMagic],
    );
    let options: Vec<u8> = skill_sideways_plays(&state, Skill::WhoNeedsMagic)
        .into_iter()
        .filter(|(card, ..)| *card == march)
        .map(|(_, option, ..)| option)
        .collect();
    assert!(options.contains(&0) && options.contains(&1));
    assert!(legal_actions(&state, SOLO).contains(&basic(mana_draw)));
    let three = done(
        &state,
        skill_sideways(march, SidewaysEffect::Move, Skill::WhoNeedsMagic, 1, None),
    );
    assert_eq!(three.move_points(SOLO), state.move_points(SOLO) + 3);
    refused(&three, strong(stamina, BLUE_DIE));
    assert!(!legal_actions(&three, SOLO).contains(&basic(mana_draw)));
    // Le dé du tour déjà pris : seulement +2.
    let paid = done(&state, strong(march, GREEN_DIE));
    let options: Vec<u8> = skill_sideways_plays(&paid, Skill::WhoNeedsMagic)
        .into_iter()
        .map(|(_, option, ..)| option)
        .collect();
    assert!(!options.is_empty());
    assert!(options.iter().all(|&option| option == 0));
}

// C7 — texte ; Compendium, *Universal Power* S4 à S6 : +3, ou +4 de la couleur de la carte.
#[test]
fn universal_power_values() {
    let march = card(BasicAction::March, 1);
    let state = tokens(
        owning(arena(VILLAGE, &[march], &[]), &[Skill::UniversalPower]),
        &[green(), red()],
    );
    let effect = |state: &GameState, card: CardId, sideways, mana| {
        let play = CardPlay::SidewaysSkill {
            sideways,
            skill: Skill::UniversalPower,
            option: 0,
            mana,
        };
        assert!(
            legal_actions(state, SOLO).contains(&Action::PlayCard { card, play }),
            "{card:?} {mana:?}"
        );
        card.effect(play)
    };
    assert_eq!(
        effect(&state, march, SidewaysEffect::Move, token_mana(green())),
        Some(Effect::Move(4))
    );
    assert_eq!(
        effect(&state, march, SidewaysEffect::Move, token_mana(red())),
        Some(Effect::Move(3))
    );
    let after = done(
        &state,
        skill_sideways(
            march,
            SidewaysEffect::Move,
            Skill::UniversalPower,
            0,
            token_mana(green()),
        ),
    );
    assert_eq!(after.move_points(SOLO), 4);
    assert_eq!(after.mana_tokens(SOLO), [red()]);
    // Éclair de feu (sort rouge) en phase d'attaque, un cristal rouge : Attaque 4.
    let fireball = spell(Spell::Fireball);
    let banner = artifact(Artifact::BannerOfGlory);
    let fighting = crystals(
        attacking(&[fireball, banner], &[Skill::UniversalPower]),
        &[BasicColor::Red],
    );
    let crystal = Some(SkillMana {
        color: red(),
        payment: ManaPayment::Crystal,
    });
    assert_eq!(
        effect(&fighting, fireball, SidewaysEffect::Attack, crystal),
        Some(attack(4))
    );
    assert_eq!(
        effect(&fighting, banner, SidewaysEffect::Attack, crystal),
        Some(attack(3))
    );
    // De nuit, un mana noir : 3, même sur un sort.
    let night = Sandbox {
        figure: Some(VILLAGE),
        hand: vec![fireball],
        placed: vec![(prowlers(), ORCS_2)],
        time: TimeOfDay::Night,
        ..Sandbox::default()
    }
    .state();
    let night = done(&night, challenge(ORCS_2));
    let night = done(&night, Action::StartCombat);
    let night = skip_to_attack_phase(&tokens(
        owning(night, &[Skill::UniversalPower]),
        &[ManaColor::Black],
    ));
    let night = done(&night, declare(&[prowlers()]));
    assert_eq!(
        effect(
            &night,
            fireball,
            SidewaysEffect::Attack,
            token_mana(ManaColor::Black)
        ),
        Some(attack(3))
    );
    // Un dé noir de jour (épuisé) : refusé.
    let black_die = Sandbox {
        figure: Some(VILLAGE),
        hand: vec![march],
        dice: vec![ManaColor::Black, red()],
        ..Sandbox::default()
    }
    .state();
    let black_die = owning(black_die, &[Skill::UniversalPower]);
    refused(
        &black_die,
        skill_sideways(
            march,
            SidewaysEffect::Move,
            Skill::UniversalPower,
            0,
            Some(SkillMana {
                color: ManaColor::Black,
                payment: ManaPayment::Die(0),
            }),
        ),
    );
}

// C8 — Compendium, *Who Needs Magic* S2, *Power of Pain* S2 : une seule compétence par carte ;
// Pouvoir universel jamais sur une Blessure.
#[test]
fn sideways_skills_do_not_stack() {
    let state = tokens(
        owning(
            arena(VILLAGE, &[wound(1), card(BasicAction::March, 1)], &[]),
            &[Skill::PowerOfPain, Skill::UniversalPower],
        ),
        &[red()],
    );
    assert!(
        skill_sideways_plays(&state, Skill::UniversalPower)
            .iter()
            .all(|(card, ..)| !card.card().is_wound())
    );
    let after = done(
        &state,
        skill_sideways(wound(1), SidewaysEffect::Move, Skill::PowerOfPain, 0, None),
    );
    assert_eq!(after.move_points(SOLO), 2);
    assert!(skill_sideways_plays(&after, Skill::PowerOfPain).is_empty());
}

// C9 — Compendium, *Universal Power* S1 : pas d'Attaque à distance de côté.
#[test]
fn sideways_skill_keeps_sideways_kind() {
    let state = tokens(
        owning(
            fight(
                VILLAGE,
                &[card(BasicAction::Rage, 1)],
                &[(prowlers(), ORCS_2)],
                &[ORCS_2],
            ),
            &[Skill::UniversalPower],
        ),
        &[red()],
    );
    assert_eq!(current_phase(&state), Some(CombatPhase::RangedAndSiege));
    assert!(skill_sideways_plays(&state, Skill::UniversalPower).is_empty());
}

// --- B. Mana payé autrement ---

fn strong_paid(card: CardId, mana: ManaPayment) -> Action {
    Action::PlayCard {
        card,
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

fn polarized(from: ManaColor, source: PolarSource) -> ManaPayment {
    ManaPayment::Polarized { from, source }
}

/// Tous les paiements des cartes jouables parmi les actions légales.
fn payments(state: &GameState) -> Vec<ManaPayment> {
    legal_actions(state, SOLO)
        .into_iter()
        .flat_map(|action| match action {
            Action::PlayCard { play, .. } => match play {
                CardPlay::Basic { mana, .. } => mana.into_iter().collect::<Vec<_>>(),
                CardPlay::Strong {
                    mana, extra_mana, ..
                } => [Some(mana), extra_mana].into_iter().flatten().collect(),
                CardPlay::Cast {
                    mana,
                    black,
                    extra_mana,
                    ..
                } => [Some(mana), black, extra_mana]
                    .into_iter()
                    .flatten()
                    .collect(),
                CardPlay::SidewaysSkill { mana, .. } => {
                    mana.map(|mana| mana.payment).into_iter().collect()
                }
                _ => Vec::new(),
            },
            Action::ActivateUnit { mana, .. } => mana,
            _ => Vec::new(),
        })
        .collect()
}

/// Les mana noirs des sorts renforcés parmi les actions légales.
fn black_of_casts(state: &GameState) -> Vec<ManaPayment> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard {
                play: CardPlay::Cast {
                    black: Some(black), ..
                },
                ..
            } => Some(black),
            _ => None,
        })
        .collect()
}

/// De nuit, en phase d'attaque contre les Rôdeurs, la main `hand`, les dés `dice`.
fn attacking_at_night(hand: &[CardId], dice: Vec<ManaColor>, skills: &[Skill]) -> GameState {
    let state = Sandbox {
        figure: Some(VILLAGE),
        hand: hand.to_vec(),
        placed: vec![(prowlers(), ORCS_2)],
        time: TimeOfDay::Night,
        dice,
        ..Sandbox::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = skip_to_attack_phase(&owning(state, skills));
    done(&state, declare(&[prowlers()]))
}

// C10 — texte : une Blessure paie du rouge, une autre carte du blanc ou du vert ; la carte va
// dans la défausse.
#[test]
fn invocation_pays_by_discarding() {
    let rage = card(BasicAction::Rage, 1);
    let march = card(BasicAction::March, 1);
    let state = attacking(&[rage, wound(1), march], &[Skill::Invocation]);
    let events = events_of(
        &state,
        &strong_paid(rage, ManaPayment::Invocation(wound(1))),
    );
    assert_events_in_order(
        &events,
        &[
            Event::SkillUsed {
                player: SOLO,
                skill: Skill::Invocation,
                option: 0,
                effect: Effect::Nothing,
            },
            Event::CardDiscarded {
                player: SOLO,
                card: wound(1),
            },
        ],
    );
    let after = events.iter().fold(state.clone(), apply);
    assert!(after.discard_pile(SOLO).contains(&wound(1)));
    assert!(
        !after
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == wound(1))
    );
    refused(&state, strong_paid(rage, ManaPayment::Invocation(march)));
    // Une carte non-Blessure paie le vert de Marche renforcée.
    let free = owning(arena(VILLAGE, &[march, rage], &[]), &[Skill::Invocation]);
    let moved = done(&free, strong_paid(march, ManaPayment::Invocation(rage)));
    assert_eq!(moved.move_points(SOLO), 4);
    // L2 : jamais la carte qu'on joue.
    refused(&free, strong_paid(march, ManaPayment::Invocation(march)));
}

// C11 — Compendium, *Invocation* S1, S2 : le noir, seulement là où il est permis ; en combat.
#[test]
fn invocation_black_only_where_allowed() {
    let fireball = spell(Spell::Fireball);
    let night = crystals(
        attacking_at_night(
            &[fireball, wound(1)],
            Sandbox::default().dice,
            &[Skill::Invocation],
        ),
        &[BasicColor::Red],
    );
    assert!(black_of_casts(&night).contains(&ManaPayment::Invocation(wound(1))));
    let day = crystals(
        attacking(&[fireball, wound(1)], &[Skill::Invocation]),
        &[BasicColor::Red],
    );
    assert!(black_of_casts(&day).is_empty());
}

// C12 — Compendium, *Minimum Turn* S3 : une Invocation n'est pas une carte jouée.
#[test]
fn invocation_is_not_a_played_card() {
    let march = card(BasicAction::March, 1);
    let rage = card(BasicAction::Rage, 1);
    let state = owning(
        Units {
            hand: vec![march, rage],
            owned: vec![herbalists()],
            ..Units::default()
        }
        .state(),
        &[Skill::Invocation],
    );
    let activate = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| {
            matches!(action, Action::ActivateUnit { mana, .. }
                if mana == &[ManaPayment::Invocation(march)])
        })
        .expect("Herboristes payés par Invocation");
    let after = done(&state, activate);
    assert!(after.discard_pile(SOLO).contains(&march));
    assert!(after.play_area(SOLO).is_empty());
    refused(&after, end_turn_without_discard());
}

// C13 — texte : la couleur opposée, sous toute forme.
#[test]
fn polarization_pays_opposite_color() {
    let rage = card(BasicAction::Rage, 1);
    let state = crystals(
        attacking(&[rage], &[Skill::Polarization]),
        &[BasicColor::Blue],
    );
    let blue = polarized(ManaColor::Basic(BasicColor::Blue), PolarSource::Crystal);
    let after = done(&state, strong_paid(rage, blue));
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::Blue), 0);
    let march = card(BasicAction::March, 1);
    let swiftness = card(BasicAction::Swiftness, 1);
    let free = tokens(
        owning(
            arena(VILLAGE, &[march, swiftness], &[]),
            &[Skill::Polarization],
        ),
        &[ManaColor::Basic(BasicColor::White), red()],
    );
    let white = polarized(ManaColor::Basic(BasicColor::White), PolarSource::Token);
    let moved = done(&free, strong_paid(march, white));
    assert_eq!(moved.move_points(SOLO), 4);
    refused(
        &free,
        strong_paid(swiftness, polarized(red(), PolarSource::Token)),
    );
}

// C14 — Compendium, *Polarization* S1 : le dé noir épuisé de jour paie du rouge ; c'est le dé du
// tour, relancé à la fin du tour.
#[test]
fn polarization_black_by_day() {
    let rage = card(BasicAction::Rage, 1);
    let state = Sandbox {
        figure: Some(VILLAGE),
        hand: vec![rage, card(BasicAction::March, 1)],
        placed: vec![(prowlers(), ORCS_2)],
        dice: vec![ManaColor::Black, green()],
        ..Sandbox::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = done(
        &skip_to_attack_phase(&owning(state, &[Skill::Polarization])),
        declare(&[prowlers()]),
    );
    let black_die = polarized(ManaColor::Black, PolarSource::Die(0));
    let after = done(&state, strong_paid(rage, black_die));
    assert_eq!(after.dice_taken(SOLO), [0]);
    // Le dé du tour est pris : plus d'autre dé.
    assert!(
        !payments(&after)
            .iter()
            .any(|payment| matches!(payment, ManaPayment::Die(_)))
    );
    // Attaque 4 contre Armure 3 : les Rôdeurs vaincus, le combat se termine.
    let won = done(&after, Action::ResolveAttack);
    let ended = done(&won, Action::EndCombat);
    let events = events_of(&ended, &end_turn_without_discard());
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::DieReturned { die: 0, .. }))
    );
}

// C15 — texte : de nuit, le dé or épuisé paie le noir d'un sort renforcé, et rien d'autre.
#[test]
fn polarization_gold_by_night() {
    let fireball = spell(Spell::Fireball);
    let rage = card(BasicAction::Rage, 1);
    let state = crystals(
        attacking_at_night(
            &[fireball, rage],
            vec![ManaColor::Gold, green()],
            &[Skill::Polarization],
        ),
        &[BasicColor::Red],
    );
    let gold_die = polarized(ManaColor::Gold, PolarSource::Die(0));
    assert!(black_of_casts(&state).contains(&gold_die));
    refused(&state, strong_paid(rage, gold_die));
}

// C16 — Compendium, *Invocation* S3 : Invocation puis Polarisation sur le même mana.
#[test]
fn invocation_then_polarization() {
    let determination = card(BasicAction::Determination, 1);
    let state = fight(
        VILLAGE,
        &[determination, wound(1)],
        &[(prowlers(), ORCS_2)],
        &[ORCS_2],
    );
    let state = owning(
        to_block_phase(&state),
        &[Skill::Invocation, Skill::Polarization],
    );
    let state = done(&state, declare_block(prowlers()));
    let payment = polarized(red(), PolarSource::Invocation(wound(1)));
    let after = done(&state, strong_paid(determination, payment));
    assert!(after.discard_pile(SOLO).contains(&wound(1)));
    let used = |skill| {
        after
            .skills(SOLO)
            .iter()
            .find(|owned| owned.skill == skill)
            .map(|owned| owned.state)
    };
    assert_eq!(used(Skill::Invocation), Some(SkillState::UsedThisTurn));
    assert_eq!(used(Skill::Polarization), Some(SkillState::UsedThisTurn));
}

// C17 — MKUE p. 5, 4.c : une fois par tour.
#[test]
fn payment_skill_once_a_turn() {
    let march = card(BasicAction::March, 1);
    let stamina = card(BasicAction::Stamina, 1);
    let state = tokens(
        owning(
            arena(VILLAGE, &[march, stamina], &[]),
            &[Skill::Polarization],
        ),
        &[ManaColor::Basic(BasicColor::White), red(), red()],
    );
    let polarizations = |state: &GameState| {
        payments(state)
            .into_iter()
            .filter(|payment| matches!(payment, ManaPayment::Polarized { .. }))
            .count()
    };
    assert!(polarizations(&state) > 0);
    let white = polarized(ManaColor::Basic(BasicColor::White), PolarSource::Token);
    let after = done(&state, strong_paid(march, white));
    assert_eq!(polarizations(&after), 0);
    let next = [
        Event::TurnEnded { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
        Event::ManaTokenGained {
            player: SOLO,
            color: red(),
        },
    ]
    .iter()
    .fold(after, apply);
    assert!(polarizations(&next) > 0);
}

// --- G. Compétences posées au centre : Rituel de guérison ---

fn state_of(state: &GameState, skill: Skill) -> Option<SkillState> {
    state
        .skills(SOLO)
        .iter()
        .find(|owned| owned.skill == skill)
        .map(|owned| owned.state)
}

/// Le tour suivant, par ses événements.
fn next_turn(state: GameState) -> GameState {
    [
        Event::TurnEnded { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    ]
    .iter()
    .fold(state, apply)
}

// C36 — texte ; Compendium, *Ritual of Pain* S1 : jeter 0, 1 ou 2 Blessures hors combat ; la
// compétence est au centre.
#[test]
fn ritual_of_pain_throws_wounds() {
    let hand = [wound(1), wound(2), card(BasicAction::March, 1)];
    let state = owning(arena(VILLAGE, &hand, &[]), &[Skill::RitualOfPain]);
    assert_eq!(
        skill_uses(&state, Skill::RitualOfPain),
        [0, 1, 2].map(|option| use_skill(Skill::RitualOfPain, option))
    );
    let events = events_of(&state, &use_skill(Skill::RitualOfPain, 2));
    let thrown = events
        .iter()
        .filter(|event| {
            **event
                == Event::WoundThrownAway {
                    player: SOLO,
                    from: arcanum_core::WoundPile::Hand,
                }
        })
        .count();
    assert_eq!(thrown, 2);
    assert!(events.contains(&Event::SkillPlacedInCenter {
        player: SOLO,
        skill: Skill::RitualOfPain,
        color: None,
    }));
    let after = events.iter().fold(state.clone(), apply);
    assert!(after.hand(SOLO).iter().all(|card| !card.card().is_wound()));
    assert_eq!(
        state_of(&after, Skill::RitualOfPain),
        Some(SkillState::InCenter)
    );
    // Jeter 0 : permis ; une seule Blessure en main : pas d'option 2.
    let zero = done(&state, use_skill(Skill::RitualOfPain, 0));
    assert_eq!(zero.hand(SOLO), state.hand(SOLO));
    let one = owning(
        arena(VILLAGE, &[wound(1), card(BasicAction::March, 1)], &[]),
        &[Skill::RitualOfPain],
    );
    assert_eq!(skill_uses(&one, Skill::RitualOfPain).len(), 2);
    // En combat, pas proposée.
    assert!(skill_uses(&fighting(&[Skill::RitualOfPain]), Skill::RitualOfPain).is_empty());
}

// C37 — texte ; S1 : au tour suivant, une Blessure de côté +3, en combat aussi ; une seule ; la
// compétence retournée à la fin de ce tour.
#[test]
fn ritual_of_pain_advantage_next_turn() {
    let hand = [wound(1), wound(2), card(BasicAction::Rage, 1)];
    let state = owning(
        arena(VILLAGE, &hand, &[(prowlers(), ORCS_2)]),
        &[Skill::RitualOfPain],
    );
    let placed = done(&state, use_skill(Skill::RitualOfPain, 0));
    // Pas d'avantage le tour même (Q1).
    assert!(skill_sideways_plays(&placed, Skill::RitualOfPain).is_empty());
    let next = next_turn(placed);
    let advantage = skill_sideways(wound(1), SidewaysEffect::Move, Skill::RitualOfPain, 0, None);
    let events = events_of(&next, &advantage);
    assert!(events.contains(&Event::CenterAdvantageUsed {
        player: SOLO,
        skill: Skill::RitualOfPain,
    }));
    let moved = events.iter().fold(next.clone(), apply);
    assert_eq!(moved.move_points(SOLO), 3);
    // Une seule Blessure.
    assert!(skill_sideways_plays(&moved, Skill::RitualOfPain).is_empty());
    // La fin de ce tour retourne la compétence.
    let events = events_of(&moved, &end_turn_without_discard());
    assert!(events.contains(&Event::CenterSkillReturned {
        player: SOLO,
        skill: Skill::RitualOfPain,
    }));
    let ended = events.iter().fold(moved, apply);
    assert_eq!(
        state_of(&ended, Skill::RitualOfPain),
        Some(SkillState::Flipped)
    );
    // En combat : Parade 3.
    let fighting = done(&done(&next, challenge(ORCS_2)), Action::StartCombat);
    let blocking = done(&to_block_phase(&fighting), declare_block(prowlers()));
    let plays = skill_sideways_plays(&blocking, Skill::RitualOfPain);
    assert!(plays.iter().any(|(card, .., effect)| *card == wound(1)
        && *effect
            == Effect::Block {
                amount: 3,
                element: Element::Physical,
            }));
}

// --- D. Vol ---

/// La forêt à l'ouest de la clairière magique, voisine des orques ① (`docs/reference/bac-a-sable.md`).
const WEST_FOREST: Hex = Hex::new(0, -3);
const LAKE: Hex = Hex::new(2, -4);
/// Une case hors des tuiles posées.
const UNREVEALED: Hex = Hex::new(0, -4);

fn fly(option: u8, to: Hex) -> Action {
    Action::UseSkill {
        skill: Skill::Flight,
        option,
        unit: None,
        target: Some(SkillTarget::Space(to)),
    }
}

fn flying_from(figure: Hex, hand: &[CardId]) -> GameState {
    owning(arena(figure, hand, &sandbox_tokens()), &[Skill::Flight])
}

fn legal(state: &GameState, action: &Action) -> bool {
    legal_actions(state, SOLO).contains(action)
}

// C22 — texte ; `docs/reference/bac-a-sable.md` : une case voisine, gratuite, sans provoquer.
#[test]
fn flight_moves_without_provoking() {
    let state = flying_from(WEST_FOREST, &move_hand());
    assert!(legal(&state, &fly(0, GLADE)));
    let events = events_of(&state, &fly(0, GLADE));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. })),
        "{events:?}"
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.figure(SOLO), Some(GLADE));
    assert_eq!(after.move_points(SOLO), 0);
    assert!(after.enemies_to_fight(SOLO).is_empty());
    assert_eq!(state_of(&after, Skill::Flight), Some(SkillState::Flipped));
}

// C23 — Compendium, *Flight* S3, S4 : deux cases pour 2 Mouvement, par toute case révélée, vers
// une case sûre.
#[test]
fn flight_two_spaces() {
    let state = flying_from(GLADE, &move_hand());
    // Sans Mouvement, l'option à deux cases n'est pas proposée.
    assert!(!legal(&state, &fly(1, VILLAGE)));
    let moving = done(&state, basic(card(BasicAction::March, 1)));
    assert!(legal(&moving, &fly(1, VILLAGE)));
    let after = done(&moving, fly(1, VILLAGE));
    assert_eq!(after.figure(SOLO), Some(VILLAGE));
    assert_eq!(after.move_points(SOLO), 0);
    // Ni le lac, ni les orques, ni une case non révélée.
    for to in [LAKE, ORCS_1, UNREVEALED] {
        refused(&moving, fly(1, to));
        refused(&moving, fly(0, to));
    }
    // Avec 1 Mouvement : refusée.
    let one = done(&state, sideways(card(BasicAction::March, 1)));
    assert_eq!(one.move_points(SOLO), 1);
    refused(&one, fly(1, VILLAGE));
}

// C24 — Compendium, *Flight* Q1 : avec Distorsion spatiale, « voisine » va jusqu'à 2 cases.
#[test]
fn flight_with_space_bending() {
    let bending = spell(Spell::SpaceBending);
    let state = flying_from(WEST_FOREST, &[bending, card(BasicAction::March, 1)]);
    assert!(!legal(&state, &fly(0, VILLAGE)));
    let bent = done(
        &state,
        Action::PlayCard {
            card: bending,
            play: CardPlay::Cast {
                option: 0,
                mana: ManaPayment::Die(BLUE_DIE),
                black: None,
                extra_mana: None,
                enemy: None,
                from_offer: false,
                targets: Default::default(),
            },
        },
    );
    assert!(legal(&bent, &fly(0, VILLAGE)));
    let after = done(&bent, fly(0, VILLAGE));
    assert_eq!(after.figure(SOLO), Some(VILLAGE));
}

// C25 — MKUE p. 6, 7 : le vol est un déplacement ; pas en combat, ni après l'action du tour.
#[test]
fn flight_is_a_move() {
    let state = flying_from(WEST_FOREST, &move_hand());
    assert!(state.can_rest(SOLO));
    let after = done(&state, fly(0, GLADE));
    assert!(!after.can_rest(SOLO));
    // En combat : pas proposé.
    let fighting = owning(
        fight(VILLAGE, &move_hand(), &[(prowlers(), ORCS_2)], &[ORCS_2]),
        &[Skill::Flight],
    );
    assert!(skill_uses(&fighting, Skill::Flight).is_empty());
    // Après l'interaction (l'action du tour) : pas proposé.
    let interacting = done(&flying_from(VILLAGE, &move_hand()), Action::Interact);
    assert!(skill_uses(&interacting, Skill::Flight).is_empty());
}

// --- E. Unités ---

fn activate(unit: UnitCard, ability: u8, option: u8, leadership: bool) -> Action {
    Action::ActivateUnit {
        unit,
        ability,
        option,
        mana: Vec::new(),
        target: None,
        enemy: None,
        leadership,
    }
}

fn in_progress_total(state: &GameState) -> Option<u32> {
    state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress_total())
}

/// Combat engagé contre `enemy` sur les orques ②, depuis le village, avec ces unités et Charisme.
fn leading(enemy: arcanum_core::EnemyToken, owned: &[UnitCard]) -> GameState {
    let state = Units {
        owned: owned.to_vec(),
        placed: vec![(enemy, ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    owning(done(&state, Action::StartCombat), &[Skill::Leadership])
}

fn leading_block(enemy: arcanum_core::EnemyToken, owned: &[UnitCard]) -> GameState {
    let state = to_block_phase(&leading(enemy, owned));
    done(&state, declare_block(enemy))
}

fn leading_attack(enemy: arcanum_core::EnemyToken, owned: &[UnitCard]) -> GameState {
    let state = skip_to_attack_phase(&leading(enemy, owned));
    done(&state, declare(&[enemy]))
}

fn guardsmen() -> UnitCard {
    UnitCard::new(Unit::UtemGuardsmen, 1)
}

fn golems() -> UnitCard {
    UnitCard::new(Unit::GuardianGolems, 1)
}

// C26 — texte ; Compendium, *Leadership* S1, S2 : +1 à distance, +3 en parade, +2 en attaque.
#[test]
fn leadership_unit_bonus() {
    let ranged = done(
        &leading(prowlers(), &[crossbowmen()]),
        declare(&[prowlers()]),
    );
    assert!(legal(&ranged, &activate(crossbowmen(), 1, 0, true)));
    let events = events_of(&ranged, &activate(crossbowmen(), 1, 0, true));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::SkillUsed {
            skill: Skill::Leadership,
            ..
        }
    )));
    let after = events.iter().fold(ranged, apply);
    assert_eq!(in_progress_total(&after), Some(3));
    let block = leading_block(prowlers(), &[peasants()]);
    let after = done(&block, activate(peasants(), 0, 1, true));
    assert_eq!(in_progress_total(&after), Some(5));
    // En phase d'attaque, l'Attaque à distance est une Attaque : +2.
    let attack = leading_attack(prowlers(), &[crossbowmen()]);
    let after = done(&attack, activate(crossbowmen(), 1, 0, true));
    assert_eq!(in_progress_total(&after), Some(4));
    // Une capacité sans Attaque ni Parade : pas de Charisme (Paysans, Influence, hors combat).
    let calm = owning(
        Units {
            owned: vec![peasants()],
            ..Units::default()
        }
        .state(),
        &[Skill::Leadership],
    );
    assert!(legal(&calm, &activate(peasants(), 2, 0, false)));
    assert!(!legal(&calm, &activate(peasants(), 2, 0, true)));
}

// C27 — Compendium, *Leadership* S4 : l'élément reste ; la Parade des Gardes d'Utem compte double
// contre Vif, bonus compris.
#[test]
fn leadership_keeps_element_and_swift() {
    let block = leading_block(prowlers(), &[golems()]);
    let ice = Action::ActivateUnit {
        unit: golems(),
        ability: 2,
        option: 0,
        mana: vec![ManaPayment::Die(BLUE_DIE)],
        target: None,
        enemy: None,
        leadership: true,
    };
    assert!(legal(&block, &ice));
    let events = events_of(&block, &ice);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::UnitActivated {
            effect: Effect::Block {
                amount: 7,
                element: Element::Ice,
            },
            ..
        }
    )));
    let swift = leading_block(wolf_riders(), &[guardsmen()]);
    let after = done(&swift, activate(guardsmen(), 1, 0, true));
    assert_eq!(in_progress_total(&after), Some(14));
}

// C28 — Compendium, *Skill Tokens* S2 : une seule unité par tour.
#[test]
fn leadership_once_a_turn() {
    let attack = leading_attack(prowlers(), &[peasants(), crossbowmen()]);
    let after = done(&attack, activate(peasants(), 0, 0, true));
    assert_eq!(in_progress_total(&after), Some(4));
    assert!(legal(&after, &activate(crossbowmen(), 0, 0, false)));
    assert!(!legal(&after, &activate(crossbowmen(), 0, 0, true)));
}

fn recruit(unit: UnitCard, bonds: bool) -> Action {
    Action::RecruitUnit {
        unit,
        disband: None,
        bonds,
    }
}

/// À l'interaction du village, avec Loyauté sans faille et `influence` points.
fn bonded(owned: &[UnitCard], offer: &[UnitCard], influence: u8) -> GameState {
    let state = owning(
        Units {
            owned: owned.to_vec(),
            offer: offer.to_vec(),
            ..Units::default()
        }
        .state(),
        &[Skill::BondsOfLoyalty],
    );
    let state = done(&state, Action::Interact);
    if influence == 0 {
        return state;
    }
    let played = state.hand(SOLO)[0];
    apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: played,
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(influence),
        },
    )
}

fn unit_offer(state: &GameState) -> Vec<UnitCard> {
    state
        .offer(Offer::Units)
        .iter()
        .filter_map(|card| card.unit())
        .collect()
}

// C29 — texte ; Compendium, *Bonds of Loyalty* Q2, S8 : deux unités régulières de plus dans
// l'offre, qui en partent à la nouvelle manche sans être remplacées.
#[test]
fn bonds_of_loyalty_adds_two_units() {
    let mut events = Units {
        offer: vec![peasants()],
        ..Units::default()
    }
    .events();
    events.push(Event::CommonDeckShuffled {
        deck: arcanum_core::CommonDeck::RegularUnits,
        order: [crossbowmen(), herbalists(), foresters()]
            .map(arcanum_core::CommonCard::Unit)
            .to_vec(),
    });
    events.push(Event::LevelGained {
        player: SOLO,
        level: 2,
    });
    events.push(Event::SkillsRevealed {
        player: SOLO,
        skills: vec![Skill::BondsOfLoyalty, Skill::Leadership],
    });
    let choosing = events.iter().fold(GameState::default(), apply);
    let action = Action::ChooseSkill {
        skill: Skill::BondsOfLoyalty,
    };
    assert!(legal(&choosing, &action));
    let after = done(&choosing, action.clone());
    assert_eq!(
        unit_offer(&after),
        [peasants(), crossbowmen(), herbalists()]
    );
    // L6 : pioche régulière vide, la compétence se gagne sans unité ajoutée.
    let empty = apply(
        choosing,
        &Event::CommonDeckShuffled {
            deck: arcanum_core::CommonDeck::RegularUnits,
            order: Vec::new(),
        },
    );
    let after = done(&empty, action);
    assert_eq!(unit_offer(&after), [peasants()]);
    assert!(
        after
            .skills(SOLO)
            .iter()
            .any(|owned| owned.skill == Skill::BondsOfLoyalty)
    );
}

// C30 — texte ; Compendium, *Bonds of Loyalty* S5 : recruter sous le jeton, 5 de moins.
#[test]
fn bonds_of_loyalty_discounted_recruit() {
    let state = bonded(&[], &[crossbowmen(), herbalists()], 1);
    // Arbalétriers d'Utem : 6 − 5 = 1 ; sans Loyauté, pas assez d'Influence.
    assert!(!legal(&state, &recruit(crossbowmen(), false)));
    assert!(legal(&state, &recruit(crossbowmen(), true)));
    let events = events_of(&state, &recruit(crossbowmen(), true));
    assert!(events.contains(&Event::UnitRecruited {
        player: SOLO,
        unit: crossbowmen(),
        cost: 1,
    }));
    let after = events.iter().fold(state, apply);
    assert_eq!(after.bonds_unit(SOLO), Some(crossbowmen()));
    assert_eq!(after.influence_points(SOLO), 0);
    // Une seconde unité sous ce jeton : refusée.
    refused(&after, recruit(herbalists(), true));
    // Herboristes : 3 − 5, 0.
    let free = bonded(&[], &[herbalists()], 0);
    assert!(legal(&free, &recruit(herbalists(), true)));
    // Tous les jetons ordinaires pris (niveau 1, un jeton) : le jeton de Loyauté sans renvoi.
    let full = bonded(&[peasants()], &[crossbowmen()], 1);
    assert!(legal(&full, &recruit(crossbowmen(), true)));
}

// C31 — texte ; Compendium, *Bonds of Loyalty* Q1, S6 : l'unité sous Loyauté ne se renvoie pas ;
// elle sert dans un donjon, pas blessée.
#[test]
fn bonds_unit_fights_where_units_sit_out() {
    let state = bonded(&[peasants()], &[crossbowmen(), foresters()], 9);
    let state = done(&state, recruit(crossbowmen(), true));
    // Le jeton ordinaire est pris par les Paysans : renvoyer les Paysans, jamais les Arbalétriers.
    let disbands: Vec<Option<UnitCard>> = legal_actions(&state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::RecruitUnit {
                unit,
                disband,
                bonds: false,
            } if unit == foresters() => Some(disband),
            _ => None,
        })
        .collect();
    assert_eq!(disbands, [Some(peasants())]);
    // Au donjon : seule l'unité sous Loyauté s'active.
    let mut events = Adventure {
        owned: vec![peasants(), golems()],
        ..Adventure::default()
    }
    .events();
    events.push(Event::SkillGained {
        player: SOLO,
        skill: Skill::BondsOfLoyalty,
    });
    events.push(Event::UnitUnderBonds {
        player: SOLO,
        unit: golems(),
    });
    let dungeon = events.iter().fold(GameState::default(), apply);
    let entered = done(&dungeon, Action::EnterAdventureSite);
    let blocking = done(
        &done(&entered, Action::NextCombatPhase),
        declare_block(minotaur()),
    );
    assert!(legal(&blocking, &activate(golems(), 0, 1, false)));
    assert!(!legal(&blocking, &activate(peasants(), 0, 1, false)));
    // Blessée : elle ne sert pas.
    let wounded = apply(
        blocking,
        &Event::UnitWounded {
            player: SOLO,
            unit: golems(),
            wounds: 1,
        },
    );
    assert!(!legal(&wounded, &activate(golems(), 0, 1, false)));
}

// C32 — Compendium, *Bonds of Loyalty* Q3 : l'unité détruite libère le jeton.
#[test]
fn bonds_token_freed_when_unit_dies() {
    let state = bonded(&[], &[crossbowmen(), herbalists()], 1);
    let state = done(&state, recruit(crossbowmen(), true));
    let destroyed = apply(
        state,
        &Event::UnitDestroyed {
            player: SOLO,
            unit: crossbowmen(),
        },
    );
    assert_eq!(destroyed.bonds_unit(SOLO), None);
    assert!(legal(&destroyed, &recruit(herbalists(), true)));
}

// --- F. Botte secrète ---

fn break_resistances(enemy: arcanum_core::EnemyToken) -> Action {
    Action::UseSkill {
        skill: Skill::ResistanceBreak,
        option: 0,
        unit: None,
        target: Some(SkillTarget::Enemy(enemy)),
    }
}

/// Combat engagé contre `enemy` sur les orques ②, depuis le village, avec Botte secrète.
fn breaking(enemy: arcanum_core::EnemyToken, hand: &[CardId]) -> GameState {
    owning(
        fight(VILLAGE, hand, &[(enemy, ORCS_2)], &[ORCS_2]),
        &[Skill::ResistanceBreak],
    )
}

fn armor_of(state: &GameState, enemy: arcanum_core::EnemyToken) -> u8 {
    state
        .combat(SOLO)
        .and_then(|combat| combat.enemy(enemy))
        .map(|enemy| enemy.effective_armor())
        .expect("ennemi du combat")
}

fn altem_guardsmen() -> arcanum_core::EnemyToken {
    token(arcanum_core::Enemy::AltemGuardsmen, 1)
}

fn ice_golems() -> arcanum_core::EnemyToken {
    token(arcanum_core::Enemy::IceGolems, 1)
}

// C33 — texte ; Compendium, *Resistance Break* S3 : Armure −1 par résistance.
#[test]
fn resistance_break_lowers_armor() {
    let guardsmen = breaking(altem_guardsmen(), &move_hand());
    assert_eq!(
        skill_uses(&guardsmen, Skill::ResistanceBreak),
        [break_resistances(altem_guardsmen())]
    );
    let after = done(&guardsmen, break_resistances(altem_guardsmen()));
    assert_eq!(armor_of(&after, altem_guardsmen()), 4);
    assert_eq!(
        state_of(&after, Skill::ResistanceBreak),
        Some(SkillState::UsedThisTurn)
    );
    let golems = done(
        &breaking(ice_golems(), &move_hand()),
        break_resistances(ice_golems()),
    );
    assert_eq!(armor_of(&golems, ice_golems()), 2);
    // Sans résistance : pas proposée ; hors combat non plus.
    assert!(skill_uses(&breaking(prowlers(), &move_hand()), Skill::ResistanceBreak).is_empty());
    let calm = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::ResistanceBreak]);
    assert!(skill_uses(&calm, Skill::ResistanceBreak).is_empty());
}

fn expose_at(enemy: arcanum_core::EnemyToken) -> Action {
    Action::PlayCard {
        card: spell(Spell::Expose),
        play: CardPlay::Cast {
            option: 0,
            mana: ManaPayment::Crystal,
            black: None,
            extra_mana: None,
            enemy: Some(enemy),
            from_offer: false,
            targets: Default::default(),
        },
    }
}

// C34 — Compendium, *Resistance Break* S2 : les résistances comptées au moment de l'utiliser.
#[test]
fn resistance_break_counts_resistances_when_used() {
    let hand = [spell(Spell::Expose), card(BasicAction::March, 1)];
    let state = crystals(breaking(ice_golems(), &hand), &[BasicColor::White]);
    let exposed = done(&state, expose_at(ice_golems()));
    assert!(skill_uses(&exposed, Skill::ResistanceBreak).is_empty());
    assert_eq!(armor_of(&exposed, ice_golems()), 4);
    let broken = done(&state, break_resistances(ice_golems()));
    let then_exposed = done(&broken, expose_at(ice_golems()));
    assert_eq!(armor_of(&then_exposed, ice_golems()), 2);
}

// C35 — texte : jamais sous 1 (avec une autre baisse : la même, posée deux fois).
#[test]
fn resistance_break_minimum_one() {
    let state = breaking(ice_golems(), &move_hand());
    let events = events_of(&state, &break_resistances(ice_golems()));
    let reduced = events
        .iter()
        .find(|event| matches!(event, Event::ResistanceBroken { .. }))
        .cloned()
        .expect("l'Armure baisse");
    let after = events.iter().fold(state, apply);
    assert_eq!(armor_of(&after, ice_golems()), 2);
    let twice = apply(after, &reduced);
    assert_eq!(armor_of(&twice, ice_golems()), 1);
}

// --- G. Compétences posées au centre : durée, Embrasure, Temps calme, Surcharge ---

fn target_skill(skill: Skill, target: Option<SkillTarget>) -> Action {
    Action::UseSkill {
        skill,
        option: 0,
        unit: None,
        target,
    }
}

// C38 — Compendium, *Skill Tokens* S4, S5 : l'avantage ne dure que le tour suivant ; posée au
// dernier tour de la manche, pas d'avantage.
#[test]
fn center_advantage_lasts_one_turn() {
    let hand = [
        wound(1),
        card(BasicAction::March, 1),
        card(BasicAction::Rage, 1),
    ];
    let state = owning(arena(VILLAGE, &hand, &[]), &[Skill::RitualOfPain]);
    let placed = done(&state, use_skill(Skill::RitualOfPain, 0));
    let next = next_turn(placed.clone());
    assert!(!skill_sideways_plays(&next, Skill::RitualOfPain).is_empty());
    // Pas utilisé : la fin de ce tour le retire, la compétence est retournée.
    let played = done(&next, sideways(card(BasicAction::March, 1)));
    let events = events_of(&played, &end_turn_without_discard());
    assert!(events.contains(&Event::CenterSkillReturned {
        player: SOLO,
        skill: Skill::RitualOfPain,
    }));
    let ended = events.iter().fold(played, apply);
    assert_eq!(
        state_of(&ended, Skill::RitualOfPain),
        Some(SkillState::Flipped)
    );
    // Posée au dernier tour de la manche : face visible à la manche suivante, sans avantage.
    let new_round = [
        Event::TurnEnded { player: SOLO },
        Event::RoundEnded { round: 1 },
        Event::RoundStarted {
            round: 2,
            time_of_day: TimeOfDay::Night,
        },
        Event::TurnStarted {
            player: SOLO,
            turn: 1,
        },
    ]
    .iter()
    .fold(placed, apply);
    assert_eq!(
        state_of(&new_round, Skill::RitualOfPain),
        Some(SkillState::Ready)
    );
    assert!(skill_sideways_plays(&new_round, Skill::RitualOfPain).is_empty());
}

fn gold() -> ManaColor {
    ManaColor::Gold
}

/// Le bac à sable de jour, les dés or, vert, rouge et bleu, Embrasure de la Source.
fn opening(hand: &[CardId]) -> GameState {
    owning(
        Sandbox {
            figure: Some(VILLAGE),
            hand: hand.to_vec(),
            placed: vec![(prowlers(), ORCS_2)],
            dice: vec![gold(), green(), red(), ManaColor::Basic(BasicColor::Blue)],
            ..Sandbox::default()
        }
        .state(),
        &[Skill::SourceOpening],
    )
}

// C39 — Compendium, *SOURCE Opening* S1, S3 à S5 : relancer un dé, ou aucun ; au tour suivant, un
// dé de base en plus et son cristal ; ce dé gardé tel quel ou relancé à la fin du tour.
#[test]
fn source_opening_extra_die_and_crystal() {
    let hand = [card(BasicAction::Stamina, 1), card(BasicAction::March, 1)];
    let state = opening(&hand);
    let uses = skill_uses(&state, Skill::SourceOpening);
    assert!(uses.contains(&target_skill(Skill::SourceOpening, None)));
    assert!(uses.contains(&target_skill(
        Skill::SourceOpening,
        Some(SkillTarget::Die(0))
    )));
    let events = events_of(
        &state,
        &target_skill(Skill::SourceOpening, Some(SkillTarget::Die(0))),
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::DieRerolled { die: 0, .. })),
        "{events:?}"
    );
    assert!(events.iter().any(|event| event.blocks_undo()));
    // Ne rien relancer : permis, la Source ne change pas.
    let placed = done(&state, target_skill(Skill::SourceOpening, None));
    assert_eq!(placed.source(), state.source());
    assert_eq!(
        state_of(&placed, Skill::SourceOpening),
        Some(SkillState::InCenter)
    );
    // Pas d'avantage le tour même.
    assert!(skill_uses(&placed, Skill::SourceOpening).is_empty());
    let next = next_turn(placed);
    let extra = skill_uses(&next, Skill::SourceOpening);
    // Le dé or n'est pas proposé en plus.
    assert!(!extra.contains(&target_skill(
        Skill::SourceOpening,
        Some(SkillTarget::Die(0))
    )));
    let take_green = target_skill(Skill::SourceOpening, Some(SkillTarget::Die(1)));
    assert!(extra.contains(&take_green));
    let events = events_of(&next, &take_green);
    assert!(events.contains(&Event::CenterAdvantageUsed {
        player: SOLO,
        skill: Skill::SourceOpening,
    }));
    let taken = events.iter().fold(next.clone(), apply);
    assert_eq!(taken.inventory(SOLO).crystals(BasicColor::Green), 1);
    assert!(taken.mana_tokens(SOLO).contains(&green()));
    // Le dé du tour reste : Endurance renforcée payée par le dé bleu.
    assert!(legal(&taken, &strong(card(BasicAction::Stamina, 1), 3)));
    // À la fin du tour : le garder tel quel.
    assert!(legal(&taken, &Action::KeepSourceDie));
    let kept = done(&taken, Action::KeepSourceDie);
    let played = done(&kept, sideways(card(BasicAction::March, 1)));
    let events = events_of(&played, &end_turn_without_discard());
    assert!(
        events.contains(&Event::DieReturned {
            player: SOLO,
            die: 1,
            color: green(),
        }),
        "{events:?}"
    );
}

// C40 — texte : terrains −2 ce tour, −1 au tour suivant, jamais sous 1.
#[test]
fn calming_the_weather_reduces_terrain() {
    let state = owning(
        arena(VILLAGE, &move_hand(), &[]),
        &[Skill::CalmingTheWeather],
    );
    let calm = done(&state, use_skill(Skill::CalmingTheWeather, 0));
    assert_eq!(
        state_of(&calm, Skill::CalmingTheWeather),
        Some(SkillState::InCenter)
    );
    let rules = calm.move_rules(SOLO);
    assert_eq!(rules.terrain_cost(arcanum_core::Terrain::Hills), Some(1));
    assert_eq!(rules.terrain_cost(arcanum_core::Terrain::Plains), Some(1));
    let next = next_turn(calm);
    let rules = next.move_rules(SOLO);
    assert_eq!(rules.terrain_cost(arcanum_core::Terrain::Hills), Some(2));
    assert_eq!(rules.terrain_cost(arcanum_core::Terrain::Plains), Some(1));
    // Le tour d'après : plus rien.
    let played = done(&next, sideways(card(BasicAction::March, 1)));
    let after = done(&played, end_turn_without_discard());
    let rules = after.move_rules(SOLO);
    assert_eq!(rules.terrain_cost(arcanum_core::Terrain::Hills), Some(3));
}

fn overload(color: ManaColor) -> Action {
    target_skill(Skill::ManaOverload, Some(SkillTarget::Color(color)))
}

fn rage(copy: u16) -> CardId {
    card(BasicAction::Rage, copy)
}

/// Surcharge de mana posée en rouge, puis le tour suivant en phase d'attaque contre les Rôdeurs.
fn overloaded(hand: &[CardId]) -> GameState {
    let state = owning(
        arena(VILLAGE, hand, &[(prowlers(), ORCS_2)]),
        &[Skill::ManaOverload],
    );
    let placed = done(&state, overload(red()));
    let next = next_turn(placed);
    let fighting = done(&done(&next, challenge(ORCS_2)), Action::StartCombat);
    done(&skip_to_attack_phase(&fighting), declare(&[prowlers()]))
}

// C41 — texte : un jeton de la couleur ; au tour suivant, la première carte alimentée de cette
// couleur qui donne des points : +4.
#[test]
fn mana_overload_first_powered_card() {
    let state = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::ManaOverload]);
    let uses = skill_uses(&state, Skill::ManaOverload);
    assert_eq!(uses.len(), 5);
    assert!(!uses.contains(&overload(gold())));
    let events = events_of(&state, &overload(red()));
    assert!(events.contains(&Event::ManaTokenGained {
        player: SOLO,
        color: red(),
    }));
    assert!(events.contains(&Event::SkillPlacedInCenter {
        player: SOLO,
        skill: Skill::ManaOverload,
        color: Some(red()),
    }));
    let attacking = crystals(
        overloaded(&[rage(1), rage(2), card(BasicAction::March, 1)]),
        &[BasicColor::Red],
    );
    let events = events_of(&attacking, &strong(rage(1), RED_DIE));
    assert!(events.contains(&Event::CenterAdvantageUsed {
        player: SOLO,
        skill: Skill::ManaOverload,
    }));
    let first = events.iter().fold(attacking, apply);
    assert_eq!(in_progress_total(&first), Some(8));
    let second = done(&first, strong_crystal(rage(2), 0));
    assert_eq!(in_progress_total(&second), Some(12));
}

// C42 — Compendium, *Mana Overload* S1, S3, S6 : Pouvoir universel ne déclenche rien ; l'avantage
// attend.
#[test]
fn mana_overload_needs_a_powered_points_card() {
    let hand = [rage(1), card(BasicAction::March, 1)];
    let attacking = owning(
        tokens(overloaded(&hand), &[red()]),
        &[Skill::UniversalPower],
    );
    let universal = skill_sideways(
        card(BasicAction::March, 1),
        SidewaysEffect::Attack,
        Skill::UniversalPower,
        0,
        token_mana(red()),
    );
    let events = events_of(&attacking, &universal);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CenterAdvantageUsed { .. }))
    );
    let after = events.iter().fold(attacking, apply);
    assert_eq!(in_progress_total(&after), Some(3));
    let powered = done(&after, strong(rage(1), RED_DIE));
    assert_eq!(in_progress_total(&powered), Some(3 + 8));
}

// C43 — Compendium, *Skill Tokens* S5 (Q1) : pas de bonus le tour même.
#[test]
fn mana_overload_not_on_same_turn() {
    let state = owning(
        fight(VILLAGE, &[rage(1)], &[(prowlers(), ORCS_2)], &[ORCS_2]),
        &[Skill::ManaOverload],
    );
    let placed = done(&state, overload(red()));
    let attacking = done(&skip_to_attack_phase(&placed), declare(&[prowlers()]));
    let after = done(&attacking, strong(rage(1), RED_DIE));
    assert_eq!(in_progress_total(&after), Some(4));
}

// --- Vue ---

fn seen(state: &GameState, skill: Skill) -> arcanum_core::SkillView {
    arcanum_core::view(state, SOLO)
        .skills
        .into_iter()
        .find(|seen| seen.skill == skill)
        .expect("compétence gagnée")
}

// C44 — la vue : options utilisables avec leurs cibles (cartes, paiements, unités, cases), état au
// centre et avantage, Influence d'interaction, plus de dé, unité sous Loyauté.
#[test]
fn view_shows_second_batch_skills() {
    let march = card(BasicAction::March, 1);
    let state = owning(
        arena(VILLAGE, &[march, wound(1)], &[]),
        &[Skill::IDontGiveADamn, Skill::Flight, Skill::Polarization],
    );
    let flexibility = seen(&state, Skill::IDontGiveADamn);
    assert_eq!(flexibility.usable, [0]);
    assert_eq!(flexibility.cards, [march]);
    assert!(!flexibility.unsupported);
    let flight = seen(&state, Skill::Flight);
    assert_eq!(flight.usable, [0]);
    assert!(flight.targets.contains(&SkillTarget::Space(PLAIN_W)));
    // Polarisation : le jeton blanc paie le vert de Marche renforcée.
    let polarizing = tokens(state, &[ManaColor::Basic(BasicColor::White)]);
    assert_eq!(seen(&polarizing, Skill::Polarization).usable, [0]);
    // Charisme : les unités qui peuvent le recevoir.
    let leadership = seen(
        &leading_attack(prowlers(), &[peasants()]),
        Skill::Leadership,
    );
    assert_eq!(leadership.usable, [0]);
    assert_eq!(leadership.units, [peasants()]);
    // Au centre : l'état et l'avantage attendu.
    let ritual = done(
        &owning(arena(VILLAGE, &[wound(1)], &[]), &[Skill::RitualOfPain]),
        use_skill(Skill::RitualOfPain, 0),
    );
    let view = arcanum_core::view(&ritual, SOLO);
    assert_eq!(
        seen(&ritual, Skill::RitualOfPain).state,
        SkillState::InCenter
    );
    assert_eq!(
        view.center_advantages,
        [arcanum_core::CenterAdvantage {
            skill: Skill::RitualOfPain,
            color: None,
            active: false,
            used: false,
        }]
    );
    // Influence d'interaction.
    let fortune = crystals(
        owning(
            arena(VILLAGE, &move_hand(), &[]),
            &[Skill::GlitteringFortune],
        ),
        &[BasicColor::Red, BasicColor::Blue],
    );
    let fortune = done(
        &done(&fortune, Action::Interact),
        use_skill(Skill::GlitteringFortune, 0),
    );
    assert_eq!(arcanum_core::view(&fortune, SOLO).interaction_influence, 2);
    // Magie inutile à +3 : plus de dé.
    let magic = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::WhoNeedsMagic]);
    let magic = done(
        &magic,
        skill_sideways(march, SidewaysEffect::Move, Skill::WhoNeedsMagic, 1, None),
    );
    assert!(arcanum_core::view(&magic, SOLO).source_dice_forbidden);
    // L'unité sous Loyauté sans faille.
    let bonds = done(
        &bonded(&[], &[crossbowmen()], 1),
        recruit(crossbowmen(), true),
    );
    assert_eq!(
        arcanum_core::view(&bonds, SOLO).bonds_unit,
        Some(crossbowmen())
    );
}

// --- Retour en arrière ---

/// Un joueur au village avec `hand`, les dés `dice` et les compétences `skills`.
fn undo_runner(hand: &[CardId], dice: &[ManaColor], skills: &[Skill]) -> (GameRunner, Puppet) {
    let mut events = Sandbox {
        figure: Some(VILLAGE),
        hand: hand.to_vec(),
        dice: dice.to_vec(),
        ..Sandbox::default()
    }
    .events();
    events.extend(skills.iter().map(|&skill| Event::SkillGained {
        player: SOLO,
        skill,
    }));
    runner(events)
}

/// Joue `action`, puis tente de l'annuler : `Ok` si l'état revient à l'identique.
fn undone(runner: &mut GameRunner, puppet: &Puppet, action: Action) -> bool {
    let before = runner.state().clone();
    play(runner, puppet, action);
    let undone = runner.undo().is_ok();
    if undone {
        assert_eq!(runner.state(), &before);
    }
    undone
}

// C45 — spec 024 : Vol, une carte de côté avec compétence et Polarisation s'annulent ; Même pas mal
// (pioche) et Embrasure de la Source (relance) non.
#[test]
fn second_batch_skill_undo() {
    let march = card(BasicAction::March, 1);
    let skills = [
        Skill::Flight,
        Skill::IDontGiveADamn,
        Skill::Polarization,
        Skill::IFeelNoPain,
        Skill::SourceOpening,
    ];
    let white = ManaColor::Basic(BasicColor::White);
    let (mut runner, puppet) = undo_runner(&[march, wound(1)], &[white, red(), gold()], &skills);
    assert!(undone(&mut runner, &puppet, fly(0, PLAIN_W)));
    assert!(undone(
        &mut runner,
        &puppet,
        skill_sideways(march, SidewaysEffect::Move, Skill::IDontGiveADamn, 0, None),
    ));
    assert!(undone(
        &mut runner,
        &puppet,
        strong_paid(march, polarized(white, PolarSource::Die(0))),
    ));
    assert!(!undone(
        &mut runner,
        &puppet,
        use_skill(Skill::IFeelNoPain, 0)
    ));
    let (mut runner, puppet) = undo_runner(&[march], &[white, red(), gold()], &skills);
    assert!(!undone(
        &mut runner,
        &puppet,
        target_skill(Skill::SourceOpening, Some(SkillTarget::Die(0))),
    ));
}

// Issue privée 302 : les valeurs possibles d'une compétence de côté, quelle que soit la carte, pour le
// bouton grisé (texte des compétences ; Compendium, *I Don't Give a Damn* S2, *Universal Power*
// S5, S6).
#[test]
fn sideways_values_list_every_possible_value() {
    assert_eq!(Skill::IDontGiveADamn.sideways_values(0), [2, 3]);
    assert_eq!(Skill::WhoNeedsMagic.sideways_values(0), [2]);
    assert_eq!(Skill::WhoNeedsMagic.sideways_values(1), [3]);
    assert_eq!(Skill::UniversalPower.sideways_values(0), [3, 4]);
    assert_eq!(Skill::PowerOfPain.sideways_values(0), [2]);
    assert!(Skill::Flight.sideways_values(0).is_empty());
}
