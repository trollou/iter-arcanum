//! Scénarios de la spec 051 (compétences : gain aux niveaux pairs et premier lot).

mod arene;

use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, CardId, CommonCard, CommonSkillView, ContentError,
    ContentSources, Effect, Element, Event, GameConfig, GameContent, GameRngs, GameState, Hero,
    InProgress, ManaColor, Offer, OwnedSkill, PlayerEvent, RngPositions, RngUsage, Scenario, Skill,
    SkillEffect, SkillState, SkillView, TimeOfDay, Unit, UnitCard, apply, filter_event,
    legal_actions, new_game, view,
};
use arene::*;

// --- Contenu et mise en place ---

// C1 — MKUE p. 3 ; Compendium, *Skill Tokens* S5 : dix compétences par héros, dont la coopérative.
#[test]
fn skills_content_has_ten_skills_per_hero() {
    for hero in Hero::ALL {
        let skills: Vec<Skill> = Skill::ALL
            .into_iter()
            .filter(|skill| skill.hero() == hero)
            .collect();
        assert_eq!(skills.len(), 10, "{hero:?}");
        let cooperative = skills
            .iter()
            .filter(|skill| skill.definition().cooperative)
            .count();
        assert_eq!(cooperative, 1, "{hero:?}");
    }
}

// C1 — spec 058 (spec 051 C1 mis à jour) : les 40 compétences sont jouables, plus aucune
// `Unsupported` ; celles qui ne s'utilisent pas seules ont des options.
#[test]
fn all_skills_are_supported() {
    assert_eq!(Skill::ALL.len(), 40);
    for skill in Skill::ALL {
        assert_ne!(
            skill.definition().effect,
            SkillEffect::Unsupported,
            "{skill:?}"
        );
        if !matches!(
            skill.definition().effect,
            SkillEffect::Sideways
                | SkillEffect::Payment
                | SkillEffect::Leadership
                | SkillEffect::Permanent
        ) {
            assert!(!skill.definition().options().is_empty(), "{skill:?}");
        }
    }
}

// C1 — spec 029 : une compétence sans définition est refusée.
#[test]
fn content_refuses_missing_skill() {
    let text = ContentSources::EMBEDDED
        .skills
        .replace("    DarkPaths:", "    // DarkPaths:");
    assert_eq!(
        GameContent::parse(ContentSources {
            skills: &text,
            ..ContentSources::EMBEDDED
        })
        .err(),
        Some(ContentError::Missing {
            file: "skills.ron",
            id: "DarkPaths".to_string(),
        })
    );
}

fn tovak(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

fn setup(config: &GameConfig) -> Vec<Event> {
    new_game(config, &mut config.rngs())
}

fn started(config: &GameConfig) -> GameState {
    past_tactic_choice(setup(config).iter().fold(GameState::default(), apply))
}

fn pile_of(events: &[Event]) -> Vec<Skill> {
    events
        .iter()
        .find_map(|event| match event {
            Event::SkillPileShuffled { order, .. } => Some(order.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

// C2 — MKUE p. 3 ; ADR 0007 : les dix compétences du héros, mélangées par leur usage.
#[test]
fn setup_shuffles_the_hero_skill_pile() {
    let config = tovak(7);
    let pile = pile_of(&setup(&config));
    let mut sorted = pile.clone();
    sorted.sort();
    let mut own: Vec<Skill> = Skill::ALL
        .into_iter()
        .filter(|skill| skill.hero() == Hero::Tovak)
        .collect();
    own.sort();
    assert_eq!(sorted, own);
    assert_eq!(pile, pile_of(&setup(&config)));
    let state = started(&config);
    assert_eq!(state.skill_pile(SOLO), pile.as_slice());
    assert_eq!(view(&state, SOLO).skill_pile_count, 10);
    // La pile est face cachée : seul le nombre se voit.
    let shuffled = setup(&config)
        .into_iter()
        .find(|event| matches!(event, Event::SkillPileShuffled { .. }))
        .expect("pile mélangée");
    assert_eq!(
        filter_event(&shuffled, SOLO),
        PlayerEvent::SkillPileShuffled {
            player: SOLO,
            count: 10
        }
    );
}

// C2 — ADR 0007 : la pile avancée loin change, rien d'autre.
#[test]
fn skill_pile_does_not_shift_other_draws() {
    let config = tovak(11);
    let shifted: RngPositions = [(RngUsage::SkillPile(SOLO), 4096)].into_iter().collect();
    let other = new_game(&config, &mut GameRngs::resume(config.seed, &shifted));
    let without_pile = |events: &[Event]| -> Vec<Event> {
        events
            .iter()
            .filter(|event| !matches!(event, Event::SkillPileShuffled { .. }))
            .cloned()
            .collect()
    };
    assert_eq!(without_pile(&setup(&config)), without_pile(&other));
    assert_ne!(pile_of(&setup(&config)), pile_of(&other));
}

// C3 — MKUE p. 5, *Using Skills* 2 : aucune compétence ni offre commune au départ.
#[test]
fn no_skill_at_start() {
    let state = started(&tovak(3));
    assert!(state.skills(SOLO).is_empty());
    assert!(state.common_skills().is_empty());
}

// --- Gagner une compétence ---

fn fame(state: GameState, fame: u32) -> GameState {
    apply(
        state,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame,
        },
    )
}

fn end_turn_action(state: &GameState) -> Action {
    legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, .. } if discard.is_empty()))
        .expect("finir le tour sans défausse est légal")
}

/// Une carte jouée de côté, puis la fin du tour sans défausse.
fn end_turn(state: &GameState) -> (Vec<Event>, GameState) {
    let card = state.hand(SOLO)[0];
    let state = &done(state, sideways(card));
    let events = events_of(state, &end_turn_action(state));
    let after = events.iter().fold(state.clone(), apply);
    (events, after)
}

fn gain(card: CommonCard) -> Action {
    Action::GainAdvancedAction {
        card: card.deed().expect("une action avancée est une carte Acte"),
    }
}

fn choose(skill: Skill) -> Action {
    Action::ChooseSkill { skill }
}

fn revealed(events: &[Event]) -> Vec<Skill> {
    events
        .iter()
        .find_map(|event| match event {
            Event::SkillsRevealed { skills, .. } => Some(skills.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Tovak au niveau 1 avec 3 gloire, à la fin du tour : niveau 2.
fn level_two() -> (Vec<Event>, GameState) {
    end_turn(&fame(started(&tovak(4)), 3))
}

// C4 — MKUE p. 11, 6.b : niveau 2, deux compétences du dessus révélées ; la pioche attend.
#[test]
fn even_level_reveals_two_skills() {
    let state = fame(started(&tovak(4)), 3);
    let pile = state.skill_pile(SOLO).to_vec();
    let (events, after) = end_turn(&state);
    assert_events_in_order(
        &events,
        &[
            Event::LevelGained {
                player: SOLO,
                level: 2,
            },
            Event::SkillsRevealed {
                player: SOLO,
                skills: pile[..2].to_vec(),
            },
        ],
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. } | Event::TurnStarted { .. }))
    );
    assert_eq!(after.skill_pile(SOLO), &pile[2..]);
    assert_eq!(
        legal_actions(&after, SOLO),
        vec![choose(pile[0]), choose(pile[1])]
    );
}

// C5 — MKUE p. 11, 6.b : une gardée, l'autre dans l'offre commune ; puis n'importe quelle carte,
// puis la pioche.
#[test]
fn keep_one_skill_and_any_advanced_action() {
    let (events, choosing) = level_two();
    let pair = revealed(&events);
    let events = events_of(&choosing, &choose(pair[1]));
    assert_eq!(
        events,
        [
            Event::SkillGained {
                player: SOLO,
                skill: pair[1],
            },
            Event::SkillToCommonOffer { skill: pair[0] },
        ]
    );
    let after = events.iter().fold(choosing, apply);
    assert_eq!(
        after.skills(SOLO),
        [OwnedSkill {
            skill: pair[1],
            state: SkillState::Ready,
        }]
    );
    assert_eq!(after.common_skills(), [pair[0]]);
    let offer = after.offer(Offer::AdvancedActions).to_vec();
    assert_eq!(
        legal_actions(&after, SOLO),
        offer.iter().map(|&card| gain(card)).collect::<Vec<_>>()
    );
    let events = events_of(&after, &gain(offer[2]));
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
}

// C6 : tant que la compétence n'est pas choisie, aucune action avancée.
#[test]
fn skill_is_chosen_before_advanced_action() {
    let (_, choosing) = level_two();
    let card = choosing.offer(Offer::AdvancedActions)[0];
    assert!(
        !legal_actions(&choosing, SOLO)
            .iter()
            .any(|action| matches!(action, Action::GainAdvancedAction { .. }))
    );
    refused(&choosing, gain(card));
}

// C9 — spec 050, L1 : offre d'actions avancées vide, la compétence puis la pioche.
#[test]
fn skill_gained_with_empty_advanced_action_offer() {
    let mut state = fame(started(&tovak(4)), 3);
    for card in state.offer(Offer::AdvancedActions).to_vec() {
        state = apply(
            state,
            &Event::OfferCardReturned {
                offer: Offer::AdvancedActions,
                card,
            },
        );
    }
    let (events, choosing) = end_turn(&state);
    let pair = revealed(&events);
    assert_eq!(pair.len(), 2);
    let events = events_of(&choosing, &choose(pair[0]));
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
}

// C10 — spec 035, L4 : niveaux 2, 3 et 4 d'un coup ; la seconde paire après la première carte.
#[test]
fn two_even_levels_reveal_pairs_one_after_the_other() {
    let state = fame(started(&tovak(4)), 15);
    let pile = state.skill_pile(SOLO).to_vec();
    let (events, choosing) = end_turn(&state);
    assert_eq!(revealed(&events), pile[..2]);
    let after_skill = done(&choosing, choose(pile[0]));
    let card = after_skill.offer(Offer::AdvancedActions)[0];
    let events = events_of(&after_skill, &gain(card));
    assert_eq!(revealed(&events), pile[2..4]);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. }))
    );
    let second = events.iter().fold(after_skill, apply);
    let after_second = done(&second, choose(pile[3]));
    let card = after_second.offer(Offer::AdvancedActions)[1];
    let events = events_of(&after_second, &gain(card));
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
    let after = events.iter().fold(after_second, apply);
    assert_eq!(
        after
            .skills(SOLO)
            .iter()
            .map(|owned| owned.skill)
            .collect::<Vec<_>>(),
        [pile[0], pile[3]]
    );
    assert_eq!(after.common_skills(), [pile[1], pile[2]]);
}

// L1 : sans choix en attente, avec une compétence non révélée, ou deux fois : refusée.
#[test]
fn choose_skill_outside_the_choice_is_refused() {
    let state = fame(started(&tovak(4)), 3);
    let pile = state.skill_pile(SOLO).to_vec();
    refused(&state, choose(pile[0]));
    let (_, choosing) = end_turn(&state);
    refused(&choosing, choose(pile[2]));
    let after = done(&choosing, choose(pile[0]));
    refused(&after, choose(pile[1]));
}

// L5 — spec 063 : au dernier tour, la compétence se gagne quand même, puis la fin de partie.
#[test]
fn last_turn_even_level_still_gives_a_skill() {
    let mut state = fame(
        started(&GameConfig {
            scenario: Scenario::first_reconnaissance().without_dummy(),
            ..tovak(4)
        }),
        3,
    );
    for event in [
        Event::LastTurnAnnounced { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    ] {
        state = apply(state, &event);
    }
    let (events, choosing) = end_turn(&state);
    let pair = revealed(&events);
    let after = done(&choosing, choose(pair[0]));
    assert_eq!(after.skills(SOLO).len(), 1);
    let card = after.offer(Offer::AdvancedActions)[0];
    let events = events_of(&after, &gain(card));
    assert!(matches!(events.last(), Some(Event::GameEnded { .. })));
}

/// Tovak au niveau 2, l'offre commune préparée avec `common`.
fn level_two_with_common(common: &[Skill]) -> (Vec<Event>, GameState) {
    let state = common
        .iter()
        .fold(fame(started(&tovak(4)), 3), |state, &skill| {
            apply(state, &Event::SkillToCommonOffer { skill })
        });
    end_turn(&state)
}

// C7 — MKUE p. 11, 6.b ; FAQ p3 : une compétence d'un autre héros prise dans l'offre commune ;
// les deux révélées y vont, et seule la dernière carte de l'offre d'actions avancées est légale.
#[test]
fn common_offer_skill_forces_lowest_advanced_action() {
    let (events, choosing) = level_two_with_common(&[Skill::HotSwordsmanship]);
    let pair = revealed(&events);
    assert_eq!(
        legal_actions(&choosing, SOLO),
        vec![
            choose(pair[0]),
            choose(pair[1]),
            choose(Skill::HotSwordsmanship)
        ]
    );
    let events = events_of(&choosing, &choose(Skill::HotSwordsmanship));
    assert_eq!(
        events,
        [
            Event::SkillTakenFromCommonOffer {
                player: SOLO,
                skill: Skill::HotSwordsmanship,
            },
            Event::SkillGained {
                player: SOLO,
                skill: Skill::HotSwordsmanship,
            },
            Event::SkillToCommonOffer { skill: pair[0] },
            Event::SkillToCommonOffer { skill: pair[1] },
        ]
    );
    let after = events.iter().fold(choosing, apply);
    assert_eq!(after.common_skills(), [pair[0], pair[1]]);
    let offer = after.offer(Offer::AdvancedActions).to_vec();
    let lowest = *offer.last().expect("une offre de trois cartes");
    assert_eq!(legal_actions(&after, SOLO), vec![gain(lowest)]);
    refused(&after, gain(offer[0]));
}

// C8 — Compendium, *Skill Tokens* S3 ; FAQ p3 : jamais une compétence de son propre héros.
#[test]
fn own_skill_in_common_offer_is_not_offered() {
    let pile = started(&tovak(4)).skill_pile(SOLO).to_vec();
    let own = pile[5];
    let (_, choosing) = level_two_with_common(&[own, Skill::FreezingPower]);
    let legal = legal_actions(&choosing, SOLO);
    assert!(legal.contains(&choose(Skill::FreezingPower)));
    assert!(!legal.contains(&choose(own)));
    refused(&choosing, choose(own));
}

// Spec #20, C9 — MKUE p. 11, 6.b ; FAQ p3 : pendant le choix d'un niveau pair, la vue dit pour
// chaque compétence prenable les actions avancées qu'elle laisse prendre : toutes pour une
// révélée, la dernière place pour une de l'offre commune ; jamais la compétence de son héros.
#[test]
fn view_lists_cards_allowed_by_each_skill() {
    let pile = started(&tovak(4)).skill_pile(SOLO).to_vec();
    let own = pile[5];
    let (events, choosing) = level_two_with_common(&[Skill::HotSwordsmanship, own]);
    let pair = revealed(&events);
    let offer: Vec<CardId> = choosing
        .offer(Offer::AdvancedActions)
        .iter()
        .filter_map(|card| card.deed())
        .collect();
    assert_eq!(offer.len(), 3);
    assert_eq!(
        view(&choosing, SOLO).skill_choice_cards,
        vec![
            (pair[0], offer.clone()),
            (pair[1], offer.clone()),
            (Skill::HotSwordsmanship, vec![offer[2]]),
        ]
    );
    // Le niveau réglé, la liste est vide.
    let after = done(&done(&choosing, choose(pair[0])), gain_deed(offer[1]));
    assert!(view(&after, SOLO).skill_choice_cards.is_empty());
}

fn gain_deed(card: CardId) -> Action {
    Action::GainAdvancedAction { card }
}

// --- Utiliser une compétence ---

/// Les compétences `skills` gagnées, prêtes.
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

fn skill_uses(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::UseSkill { .. }))
        .collect()
}

fn state_of(state: &GameState, skill: Skill) -> Option<SkillState> {
    state
        .skills(SOLO)
        .iter()
        .find(|owned| owned.skill == skill)
        .map(|owned| owned.state)
}

/// La nouvelle journée d'un tour suivant, par ses événements.
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

/// Combat engagé contre les Rôdeurs depuis le village, compétences de Tovak gagnées.
fn fighting(skills: &[Skill]) -> GameState {
    owning(
        fight(VILLAGE, &move_hand(), &[(prowlers(), ORCS_2)], &[ORCS_2]),
        skills,
    )
}

fn attack_points(state: &GameState, element: Element) -> u32 {
    match state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress.clone())
    {
        Some(InProgress::Attack { points, .. } | InProgress::Block { points, .. }) => {
            points.get(element)
        }
        None => 0,
    }
}

// C12 — MKUE p. 5, *Using Skills* 4.c : une fois par tour, de nouveau au tour suivant.
#[test]
fn once_a_turn_skill_resets_next_turn() {
    let attacking = done(
        &skip_to_attack_phase(&fighting(&[Skill::ColdSwordsmanship])),
        declare(&[prowlers()]),
    );
    let ice = use_skill(Skill::ColdSwordsmanship, 1);
    let events = events_of(&attacking, &ice);
    assert_eq!(
        events[0],
        Event::SkillUsed {
            player: SOLO,
            skill: Skill::ColdSwordsmanship,
            option: 1,
            effect: Effect::Attack {
                amount: 2,
                element: Element::Ice,
                kind: AttackKind::Melee,
            },
        }
    );
    let after = events.iter().fold(attacking, apply);
    assert_eq!(attack_points(&after, Element::Ice), 2);
    assert_eq!(
        state_of(&after, Skill::ColdSwordsmanship),
        Some(SkillState::UsedThisTurn)
    );
    refused(&after, ice);
    assert_eq!(
        state_of(&next_turn(after), Skill::ColdSwordsmanship),
        Some(SkillState::Ready)
    );
    // Hors combat, Marche forcée : utilisée, refusée, de nouveau légale au tour suivant.
    let state = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::DoubleTime]);
    let double = use_skill(Skill::DoubleTime, 0);
    let used = done(&state, double.clone());
    refused(&used, double.clone());
    assert!(skill_uses(&next_turn(used)).contains(&double));
}

// C13, L3 — spec 052 : une option se propose quand ses points servent.
#[test]
fn skill_options_follow_play_moment() {
    let skills = [
        Skill::ColdSwordsmanship,
        Skill::ShieldMastery,
        Skill::BurningPower,
        Skill::HotSwordsmanship,
    ];
    let free = owning(arena(VILLAGE, &move_hand(), &[]), &skills);
    assert!(skill_uses(&free).is_empty());
    let ranged = done(&fighting(&skills), declare(&[prowlers()]));
    assert_eq!(
        skill_uses(&ranged),
        [
            use_skill(Skill::BurningPower, 0),
            use_skill(Skill::BurningPower, 1),
        ]
    );
    let blocking = done(
        &to_block_phase(&fighting(&skills)),
        declare_block(prowlers()),
    );
    assert_eq!(
        skill_uses(&blocking),
        [
            use_skill(Skill::ShieldMastery, 0),
            use_skill(Skill::ShieldMastery, 1),
            use_skill(Skill::ShieldMastery, 2),
        ]
    );
    refused(&blocking, use_skill(Skill::HotSwordsmanship, 0));
}

fn used_effect(state: &GameState, action: &Action) -> Option<Effect> {
    events_of(state, action)
        .iter()
        .find_map(|event| match event {
            Event::SkillUsed { effect, .. } => Some(*effect),
            _ => None,
        })
}

// C14 — texte ; Compendium, *Day Sharpshooting* S1 : la valeur du moment où elle est utilisée,
// la nuit dans un donjon.
#[test]
fn day_and_night_skill_values() {
    let day = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::DarkPaths]);
    assert_eq!(
        used_effect(&day, &use_skill(Skill::DarkPaths, 0)),
        Some(Effect::Move(1))
    );
    let night = owning(
        Sandbox {
            figure: Some(VILLAGE),
            hand: move_hand().to_vec(),
            time: TimeOfDay::Night,
            ..Sandbox::default()
        }
        .state(),
        &[Skill::DarkPaths],
    );
    assert_eq!(
        used_effect(&night, &use_skill(Skill::DarkPaths, 0)),
        Some(Effect::Move(2))
    );
    let dungeon = owning(
        done(&Adventure::default().state(), Action::EnterAdventureSite),
        &[Skill::DaySharpshooting],
    );
    let aiming = done(&dungeon, declare(&[minotaur()]));
    assert_eq!(
        used_effect(&aiming, &use_skill(Skill::DaySharpshooting, 0)),
        Some(Effect::Attack {
            amount: 1,
            element: Element::Physical,
            kind: AttackKind::Ranged,
        })
    );
}

// C15 — spec 040 : l'Influence de la compétence s'ajoute à celle des cartes.
#[test]
fn skill_influence_adds_to_card_influence() {
    let state = done(
        &owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::DarkNegotiation]),
        Action::Interact,
    );
    let before = state.influence_points(SOLO);
    let after = done(&state, use_skill(Skill::DarkNegotiation, 0));
    assert_eq!(after.influence_points(SOLO), before + 2);
}

fn end_turns(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::EndTurn { .. }))
        .collect()
}

// C23 — Compendium, *Skill Tokens* S1 : pas l'action du tour, rien dans la zone de jeu ; le repos
// reste possible.
#[test]
fn skill_use_is_not_an_action() {
    let state = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::DarkNegotiation]);
    let after = done(&state, use_skill(Skill::DarkNegotiation, 0));
    assert_eq!(after.influence_points(SOLO), 2);
    assert!(after.play_area(SOLO).is_empty());
    assert_eq!(end_turns(&after), end_turns(&state));
    assert!(legal_actions(&after, SOLO).contains(&Action::Interact));
}

// L2 : pendant un choix en attente (fin de tour), pas de compétence.
#[test]
fn no_skill_during_a_pending_choice() {
    let (_, choosing) = level_two();
    let choosing = owning(choosing, &[Skill::DoubleTime]);
    assert!(skill_uses(&choosing).is_empty());
}

// --- Compétences « par manche » ---

fn next_round(state: GameState) -> GameState {
    apply(
        state,
        &Event::RoundStarted {
            round: 2,
            time_of_day: TimeOfDay::Night,
        },
    )
}

// C16 — MKUE p. 5, *Using Skills* 4.a ; p. 4, 2.g : Magie noire, cristal rouge et jeton rouge ou
// noir ; retournée jusqu'à la manche suivante.
#[test]
fn once_a_round_skill_flips_until_next_round() {
    let state = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::DarkFireMagic]);
    assert_eq!(
        skill_uses(&state),
        [
            use_skill(Skill::DarkFireMagic, 0),
            use_skill(Skill::DarkFireMagic, 1),
        ]
    );
    let events = events_of(&state, &use_skill(Skill::DarkFireMagic, 1));
    assert_eq!(
        events[1..],
        [
            Event::CrystalGained {
                player: SOLO,
                color: BasicColor::Red,
            },
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Black,
            },
        ]
    );
    let red = events_of(&state, &use_skill(Skill::DarkFireMagic, 0));
    assert_eq!(
        red[2],
        Event::ManaTokenGained {
            player: SOLO,
            color: ManaColor::Basic(BasicColor::Red),
        }
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(
        state_of(&after, Skill::DarkFireMagic),
        Some(SkillState::Flipped)
    );
    let later = next_turn(after);
    assert!(skill_uses(&later).is_empty());
    let next = next_round(later);
    assert_eq!(
        state_of(&next, Skill::DarkFireMagic),
        Some(SkillState::Ready)
    );
}

// C17 — spec 025 : le 4e cristal devient un jeton, en plus du jeton de la compétence.
#[test]
fn crystal_skill_fourth_crystal_becomes_token() {
    let state = (0..3).fold(
        owning(
            arena(VILLAGE, &move_hand(), &[]),
            &[Skill::WhiteCrystalCraft],
        ),
        |state, _| {
            apply(
                state,
                &Event::CrystalGained {
                    player: SOLO,
                    color: BasicColor::Blue,
                },
            )
        },
    );
    let events = events_of(&state, &use_skill(Skill::WhiteCrystalCraft, 0));
    assert_eq!(
        events[1..],
        [
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Basic(BasicColor::Blue),
            },
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Basic(BasicColor::White),
            },
        ]
    );
}

fn drawn(events: &[Event]) -> Vec<CardId> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::CardDrawn { card, .. } => Some(*card),
            _ => None,
        })
        .collect()
}

// C18 — FAQ p2 ; Compendium, *Motivation* S1 : deux cartes et toujours le jeton, en combat aussi.
#[test]
fn motivation_draws_two_and_gains_token() {
    let state = owning(
        arena(VILLAGE, &move_hand(), &[]),
        &[Skill::MotivationNorowas],
    );
    let top = state.deck(SOLO)[..2].to_vec();
    let events = events_of(&state, &use_skill(Skill::MotivationNorowas, 0));
    assert_eq!(drawn(&events), top);
    assert!(events.contains(&Event::ManaTokenGained {
        player: SOLO,
        color: ManaColor::Basic(BasicColor::White),
    }));
    let after = events.iter().fold(state, apply);
    assert!(top.iter().all(|card| after.hand(SOLO).contains(card)));
    let fighting = fighting(&[Skill::MotivationNorowas]);
    assert!(skill_uses(&fighting).contains(&use_skill(Skill::MotivationNorowas, 0)));
}

// C18 — Compendium, *Motivation* S2 : une Blessure piochée ne compte pas pour Assommé.
#[test]
fn motivation_wound_does_not_knock_out() {
    let state = fighting(&[Skill::MotivationTovak]);
    let hand = state.hand(SOLO).to_vec();
    let order: Vec<CardId> = hand.iter().copied().chain([wound(1), wound(2)]).collect();
    let mut events = vec![Event::DeckShuffled {
        player: SOLO,
        order,
    }];
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    let state = events.iter().fold(state, apply);
    let events = events_of(&state, &use_skill(Skill::MotivationTovak, 0));
    assert_eq!(drawn(&events), [wound(1), wound(2)]);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::KnockedOut { .. }))
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(
        after.combat(SOLO).map(|combat| combat.wounds_to_hand),
        Some(0)
    );
}

// C19 — Compendium, *Motivation* S3 : un deck d'une carte, une carte piochée, le jeton quand même.
#[test]
fn motivation_with_short_deck() {
    let only = card(BasicAction::March, 1);
    let state = owning(state_with_deck(vec![only], 0), &[Skill::MotivationTovak]);
    let events = events_of(&state, &use_skill(Skill::MotivationTovak, 0));
    assert_eq!(drawn(&events), [only]);
    assert!(events.contains(&Event::ManaTokenGained {
        player: SOLO,
        color: ManaColor::Basic(BasicColor::Blue),
    }));
}

// C20 — texte ; Compendium, *Motivation* S5 : une seule Motivation jusqu'à la fin du tour
// suivant ; de nouveau à la manche suivante.
#[test]
fn second_motivation_blocked_until_end_of_next_turn() {
    let state = owning(
        arena(VILLAGE, &move_hand(), &[]),
        &[Skill::MotivationTovak, Skill::MotivationArythea],
    );
    let after = done(&state, use_skill(Skill::MotivationTovak, 0));
    let second = use_skill(Skill::MotivationArythea, 0);
    refused(&after, second.clone());
    let next = next_turn(after);
    assert!(!skill_uses(&next).contains(&second));
    assert!(skill_uses(&next_round(next)).contains(&second));
}

// C21 — texte ; spec 034 : Soin 2 hors combat, jamais en combat.
#[test]
fn potion_making_heals_two_out_of_combat() {
    let state = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::PotionMaking]);
    let after = done(&state, use_skill(Skill::PotionMaking, 0));
    assert_eq!(after.healing_points(SOLO), 2);
    assert!(skill_uses(&fighting(&[Skill::PotionMaking])).is_empty());
}

fn fire_golems() -> UnitCard {
    UnitCard::new(Unit::FireGolems, 1)
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn inspiration(option: u8, unit: UnitCard) -> Action {
    Action::UseSkill {
        skill: Skill::Inspiration,
        option,
        unit: Some(unit),
        target: None,
    }
}

// C22 — texte ; Compendium, *Inspiration* Q1, S2 : rendre disponible une unité épuisée de tout
// niveau, ou en soigner une blessée sans payer ; jamais en combat.
#[test]
fn inspiration_readies_or_heals_a_unit() {
    let mut events = arena_events(VILLAGE, &move_hand(), &[], &[]);
    events.extend(unit_events(&[fire_golems(), peasants()], &[]));
    events.push(Event::UnitActivated {
        player: SOLO,
        unit: fire_golems(),
        ability: 0,
        option: 0,
        effect: Effect::Nothing,
        enemy: None,
    });
    events.push(Event::UnitWounded {
        player: SOLO,
        unit: peasants(),
        wounds: 1,
    });
    let state = owning(
        events.iter().fold(GameState::default(), apply),
        &[Skill::Inspiration],
    );
    assert_eq!(
        skill_uses(&state),
        [inspiration(0, fire_golems()), inspiration(1, peasants())]
    );
    assert!(
        events_of(&state, &inspiration(0, fire_golems())).contains(&Event::UnitReadied {
            player: SOLO,
            unit: fire_golems(),
        })
    );
    let healed = done(&state, inspiration(1, peasants()));
    assert!(healed.units(SOLO).iter().all(|owned| !owned.is_wounded()));
    assert_eq!(healed.healing_points(SOLO), 0);
    refused(&state, inspiration(0, peasants()));
    refused(&state, inspiration(1, fire_golems()));
    assert!(skill_uses(&fighting(&[Skill::Inspiration])).is_empty());
}

// --- Retour en arrière, vue ---

// C25 — spec 024 : une compétence utilisée sans pioche s'annule ; Motivation pioche, non.
#[test]
fn skill_use_undo() {
    let mut events = arena_events(VILLAGE, &move_hand(), &[], &[]);
    events.extend(
        [Skill::DoubleTime, Skill::MotivationTovak].map(|skill| Event::SkillGained {
            player: SOLO,
            skill,
        }),
    );
    let (mut runner, puppet) = runner(events);
    let before = runner.state().clone();
    play(&mut runner, &puppet, use_skill(Skill::DoubleTime, 0));
    assert_eq!(
        state_of(runner.state(), Skill::DoubleTime),
        Some(SkillState::UsedThisTurn)
    );
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(runner.state(), &before);
    play(&mut runner, &puppet, use_skill(Skill::MotivationTovak, 0));
    assert!(runner.undo().is_err());
}

// C26 : la vue montre les compétences, les révélées, l'offre commune et la dernière place ;
// jamais l'ordre de la pile.
#[test]
fn view_shows_skills() {
    let pile = started(&tovak(4)).skill_pile(SOLO).to_vec();
    let own = pile[5];
    let (events, choosing) = level_two_with_common(&[Skill::HotSwordsmanship, own]);
    let pair = revealed(&events);
    let seen = view(&choosing, SOLO);
    assert_eq!(seen.skill_choice, pair);
    assert_eq!(
        seen.common_skills,
        [
            CommonSkillView {
                skill: Skill::HotSwordsmanship,
                takeable: true,
            },
            CommonSkillView {
                skill: own,
                takeable: false,
            },
        ]
    );
    assert_eq!(seen.skill_pile_count, 8);
    let after = done(&choosing, choose(Skill::HotSwordsmanship));
    assert!(view(&after, SOLO).lowest_advanced_action_only);
    assert!(
        view(&after, SOLO)
            .common_skills
            .iter()
            .all(|common| !common.takeable)
    );

    let state = owning(
        arena(VILLAGE, &move_hand(), &[]),
        &[
            Skill::DoubleTime,
            Skill::ColdSwordsmanship,
            Skill::MotivationTovak,
            Skill::MotivationArythea,
        ],
    );
    let state = done(&state, use_skill(Skill::MotivationTovak, 0));
    let skills = view(&state, SOLO).skills;
    let view_of = |skill: Skill| {
        skills
            .iter()
            .find(|seen| seen.skill == skill)
            .cloned()
            .expect("compétence gagnée")
    };
    assert_eq!(
        view_of(Skill::DoubleTime),
        SkillView {
            skill: Skill::DoubleTime,
            state: SkillState::Ready,
            usable: vec![0],
            unsupported: false,
            motivation_blocked: false,
            cards: Vec::new(),
            units: Vec::new(),
            targets: Vec::new(),
        }
    );
    assert!(view_of(Skill::ColdSwordsmanship).usable.is_empty());
    assert_eq!(view_of(Skill::MotivationTovak).state, SkillState::Flipped);
    assert!(view_of(Skill::MotivationArythea).motivation_blocked);
    assert_eq!(skills.len(), 4);
}
