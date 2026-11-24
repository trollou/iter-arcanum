//! Parcours de bout en bout du jalon 3 (spec 039, ADR 0006) : deux combats contre les orques du
//! bac à sable, un soin entre les deux, un niveau gagné, jusqu'au début de la manche 2.

mod support;

use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, CardId, CardPlay, Combat, CombatPhase, DeedCard,
    Effect, Enemy, EnemyColor, EnemyToken, Event, EventLog, GameConfig, HealingTarget, Hero, Hex,
    ManaColor, Offer, PlayerView, RngPositions, STARTING_HAND_LIMIT, Scenario, SidewaysEffect,
    Terrain, TimeOfDay, routes_avoiding,
};
use support::arrangement::Setup;
use support::{Parcours, has};

/// Une graine quelconque : la mise en place arrangée ne dépend plus d'elle (spec 039, « Départ et
/// préconditions ») ; `balayer_les_graines` mesure combien la suite en supporte.
const SEED: u64 = 0;
/// Spec 039 (issue privée 322) : le parcours passe au moins 90 graines sur 100, seuil de la 059.
const SWEEP_THRESHOLD: usize = 90;

const DECK_SIZE: usize = 16;

const VILLAGE: Hex = Hex::new(2, -3);
/// Plaine voisine du village et des orques de la tuile 2.
const PLAIN: Hex = Hex::new(2, -2);
/// Les orques de la tuile de campagne 2 : combat 1, provoqué.
const ORCS_2: Hex = Hex::new(3, -3);
/// La clairière magique, voisine des orques de la tuile 1.
const GLADE: Hex = Hex::new(1, -3);
/// Les orques de la tuile de campagne 1 : combat 2, défié.
const ORCS_1: Hex = Hex::new(1, -4);

/// Un tour ne suffit pas toujours pour marcher : borne des tours de marche.
const MAX_WALKING_TURNS: u32 = 4;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        // Le héros de la partie lancée par l'interface, comme les 019 et 028.
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Spec 039, L1 : un départ qui ne remplit plus une précondition se revoit, pas la graine.
fn precondition(holds: bool, what: &str) {
    assert!(holds, "départ à revoir : {what}");
}

/// Spec 039, « Départ et préconditions » : Rôdeurs puis Creuseurs sur le dessus de la pile verte,
/// les errants posés sur `(1, -4)` puis `(3, -3)` à la mise en place ; le deck et la Source qui
/// font tenir les deux combats dans la première manche, de jour.
fn arranged(seed: u64) -> Setup {
    Setup::new(config(seed))
        .enemies(
            EnemyColor::Green,
            &[
                EnemyToken::new(Enemy::Prowlers, 1),
                EnemyToken::new(Enemy::Diggers, 1),
            ],
        )
        .deck_cards(&DECK)
        .source(&SOURCE)
}

/// Le deck d'Arythea, tour par tour (main de 5, les cartes gardées restent en main) :
/// 1. Marche, Endurance et Célérité : 6 points jusqu'au village ; Rage et Tranquillité gardées ;
/// 2. Endurance : le pas qui provoque ; Rage renforcée (dé rouge) : attaque 4 contre les Creuseurs ;
/// 3. Tranquillité soigne une Blessure ; Promesse et Menace de côté : retour au village ;
/// 4. Marche et Célérité : la clairière ; Détermination et Polyvalence gardées ;
/// 5. Détermination, Polyvalence et Improvisation (en défaussant Concentration) : parade 4, puis
///    attaque 3 contre les Rôdeurs ; le jeton d'or de la clairière renforce Détermination.
const DECK: [DeedCard; 16] = [
    DeedCard::Basic(BasicAction::March),
    DeedCard::Basic(BasicAction::Stamina),
    DeedCard::Basic(BasicAction::Swiftness),
    DeedCard::Basic(BasicAction::Rage),
    DeedCard::Basic(BasicAction::Tranquility),
    DeedCard::Basic(BasicAction::Stamina),
    DeedCard::Basic(BasicAction::Promise),
    DeedCard::Basic(BasicAction::Threaten),
    DeedCard::Basic(BasicAction::March),
    DeedCard::Basic(BasicAction::Swiftness),
    DeedCard::Basic(BasicAction::Determination),
    DeedCard::Basic(BasicAction::BattleVersatility),
    DeedCard::Basic(BasicAction::Improvisation),
    DeedCard::Basic(BasicAction::Concentration),
    DeedCard::Basic(BasicAction::Crystallize),
    DeedCard::Basic(BasicAction::ManaPull),
];

/// Rouge pour Rage renforcée au combat 1 ; les deux autres ne servent à aucune carte des tours 1 à 3.
const SOURCE: [ManaColor; 3] = [
    ManaColor::Basic(BasicColor::Red),
    ManaColor::Basic(BasicColor::White),
    ManaColor::Basic(BasicColor::White),
];

/// Le plateau de jeu traversé ne dépend pas de la graine.
fn expect_terrain(parcours: &Parcours, hex: Hex, terrain: Terrain) {
    let space = parcours.view().map.space(hex);
    assert_eq!(
        space.map(|space| space.terrain),
        Some(terrain),
        "case {hex:?} : la mise en place a changé"
    );
}

fn wounds_in(cards: &[CardId]) -> usize {
    cards.iter().filter(|card| card.card().is_wound()).count()
}

fn combat(view: &PlayerView) -> Combat {
    view.combat.clone().expect("un combat en cours")
}

/// Les cartes gardées pour les combats et le soin : jamais jouées pour se déplacer.
fn kept_for_later(card: CardId) -> bool {
    matches!(
        card.card(),
        DeedCard::Basic(
            BasicAction::Rage
                | BasicAction::Determination
                | BasicAction::BattleVersatility
                | BasicAction::Improvisation
                | BasicAction::Tranquility
        )
    )
}

/// Une carte jouée légale et l'effet qu'elle donne.
fn card_plays(parcours: &Parcours) -> Vec<(Action, CardId, CardPlay, Effect)> {
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

/// Joue des cartes jusqu'à avoir `points` points de mouvement : effet de base de Mouvement (sans
/// mana), sinon de côté, en gardant les cartes de combat et Tranquillité. Rend faux s'il en manque.
fn gather_move(parcours: &mut Parcours, points: u32) -> bool {
    while parcours.view().move_points < points {
        let best = card_plays(parcours)
            .into_iter()
            .filter(|(_, card, play, _)| {
                !kept_for_later(*card)
                    && matches!(
                        play,
                        CardPlay::Basic {
                            mana: None,
                            discard: None,
                            ..
                        } | CardPlay::Sideways(SidewaysEffect::Move)
                    )
            })
            .filter_map(|(action, _, _, effect)| match effect {
                Effect::Move(amount) => Some((amount, action)),
                _ => None,
            })
            .max_by_key(|(amount, _)| *amount);
        let Some((_, action)) = best else {
            return false;
        };
        parcours.play(action);
    }
    true
}

/// Joue des cartes jusqu'à ce que l'attaque ou la parade en cours atteigne `needed` : à chaque
/// fois la carte qui rapporte le plus de points par carte quittant la main (de base, renforcée
/// ou de côté).
fn gather_combat(parcours: &mut Parcours, needed: u32, what: &str) {
    loop {
        let current = combat(&parcours.view());
        if current.in_progress_total().unwrap_or(0) >= needed {
            return;
        }
        let best = card_plays(parcours)
            .into_iter()
            .filter(|(_, _, _, effect)| current.efficiency(*effect).is_some())
            .filter_map(|(action, _, play, effect)| {
                let points = current.counted_points(effect)?;
                let cards = match play {
                    CardPlay::Basic {
                        discard: Some(_), ..
                    }
                    | CardPlay::Strong {
                        discard: Some(_), ..
                    } => 2,
                    _ => 1,
                };
                Some((points * 10 / cards, action))
            })
            .max_by_key(|(score, _)| *score);
        precondition(best.is_some(), &format!("assez de cartes pour {what}"));
        let Some((_, action)) = best else { return };
        parcours.play(action);
    }
}

/// Finit le tour ; sans carte jouée, en défaussant une carte qui n'est pas une Blessure (MKUE p. 6,
/// 5.a).
fn finish_turn(parcours: &mut Parcours) -> Vec<Event> {
    let view = parcours.view();
    let discard = if view.play_area.is_empty() {
        let card = view
            .hand
            .iter()
            .copied()
            .find(|card| !card.card().is_wound() && !kept_for_later(*card))
            .or_else(|| view.hand.iter().copied().find(|c| !c.card().is_wound()));
        card.into_iter().collect()
    } else {
        Vec::new()
    };
    parcours.play(Action::EndTurn {
        discard,
        reclaim: None,
        glade: None,
        steady_tempo: false,
    })
}

/// Marche case par case, sur un ou plusieurs tours, jusqu'à la dernière case de `steps` (chacune
/// avec son coût) ; le tour n'est pas fini à l'arrivée. Rend les événements des pas.
fn walk(parcours: &mut Parcours, steps: &[(Hex, u32)]) -> Vec<Event> {
    let mut events = Vec::new();
    let mut next = 0;
    for _ in 0..MAX_WALKING_TURNS {
        let remaining: u32 = steps[next..].iter().map(|(_, cost)| cost).sum();
        gather_move(parcours, remaining);
        while let Some(&(hex, _)) = steps.get(next) {
            if !parcours.legal().contains(&Action::MoveHero { to: hex }) {
                break;
            }
            events.extend(parcours.play(Action::MoveHero { to: hex }));
            next += 1;
        }
        if next == steps.len() {
            return events;
        }
        finish_turn(parcours);
    }
    precondition(false, "assez de cartes de Mouvement pour marcher");
    events
}

/// Le trajet le moins cher vers `to`, qui évite les ennemis errants (spec 030, C19), avec le coût
/// de chaque pas.
fn route_to(parcours: &Parcours, to: Hex) -> Vec<(Hex, u32)> {
    let view = parcours.view();
    let from = view.figure.expect("la figurine est posée");
    let enemies: Vec<Hex> = view.enemy_tokens.keys().copied().collect();
    let route = routes_avoiding(&view.map, from, view.time_of_day, &enemies)
        .into_iter()
        .find(|route| route.to == to)
        .expect("la case est atteignable");
    route
        .steps
        .iter()
        .map(|&hex| {
            let space = view.map.space(hex).expect("case du trajet");
            let cost = match arcanum_core::move_cost(space, view.time_of_day) {
                arcanum_core::MoveCost::Points(points) => u32::from(points),
                other => panic!("pas infranchissable sur le trajet : {other:?}"),
            };
            (hex, cost)
        })
        .collect()
}

/// Les Blessures gardées : reçues (en main ou toxiques en défausse) moins celles soignées, d'après
/// le journal.
fn wounds_kept(log: &EventLog) -> usize {
    let count = |wanted: fn(&Event) -> Option<usize>| -> usize {
        log.entries()
            .iter()
            .filter_map(|entry| wanted(&entry.event))
            .sum()
    };
    let taken = count(|event| match event {
        Event::WoundsTaken { wounds, .. } | Event::PoisonWoundsDiscarded { wounds, .. } => {
            Some(wounds.len())
        }
        _ => None,
    });
    let healed = count(|event| match event {
        Event::WoundsHealed { wounds, .. } => Some(wounds.len()),
        _ => None,
    });
    taken - healed
}

fn tranquility(view: &PlayerView) -> Option<CardId> {
    view.hand
        .iter()
        .copied()
        .find(|card| card.card() == DeedCard::Basic(BasicAction::Tranquility))
}

/// C7 — joue Tranquillité pour « Soin 1 », si elle est en main avec une Blessure ; rend vrai si
/// c'est fait.
fn heal(parcours: &mut Parcours) -> bool {
    let view = parcours.view();
    let Some(card) = tranquility(&view) else {
        return false;
    };
    if wounds_in(&view.hand) == 0 {
        return false;
    }
    // MKUE p. 11, *Wounds and Healing* 3 ; spec 034 (C1) : hors combat, une Blessure de moins,
    // par le point de soin dépensé ensuite (spec 042, Q1).
    parcours.play(Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        },
    });
    let events = parcours.play(Action::SpendHealing {
        on: HealingTarget::Wound,
    });
    assert!(has(&events, |e| matches!(
        e,
        Event::WoundsHealed { wounds, .. } if wounds.len() == 1
    )));
    assert_eq!(wounds_in(&parcours.view().hand), wounds_in(&view.hand) - 1);
    true
}

/// C1 — MKUE p. 3 ; specs 030, 035.
fn setup(parcours: &Parcours) {
    let view = parcours.view();
    assert!(view.enemy_tokens.contains_key(&ORCS_1));
    assert!(view.enemy_tokens.contains_key(&ORCS_2));
    assert_eq!((view.level, view.fame, view.armor), (1, 0, 2));
    for (hex, terrain) in [
        (VILLAGE, Terrain::Plains),
        (PLAIN, Terrain::Plains),
        (ORCS_2, Terrain::Hills),
        (GLADE, Terrain::Forest),
        (ORCS_1, Terrain::Forest),
    ] {
        expect_terrain(parcours, hex, terrain);
    }
    // Les ennemis arrangés, nommés pour écrire en chiffres les Blessures et la gloire attendues
    // (`enemies.ron`).
    precondition(
        view.enemy_tokens[&ORCS_2][0].enemy() == Enemy::Diggers,
        "les orques de la tuile 2 sont des Creuseurs (armure 3, attaque 3, gloire 2)",
    );
    precondition(
        view.enemy_tokens[&ORCS_1][0].enemy() == Enemy::Prowlers,
        "les orques de la tuile 1 sont des Rôdeurs (armure 3, attaque 4, gloire 2)",
    );
}

/// Combat 1 : C2 à C6.
fn first_combat(parcours: &mut Parcours) {
    walk(parcours, &route_to(parcours, VILLAGE));
    finish_turn(parcours);
    precondition(
        gather_move(parcours, 2),
        "de quoi payer le pas qui provoque",
    );

    // C2 — MKUE p. 7, 4.c ; p. 8, 1.e ; specs 030, 031 (C1, C23).
    let events = parcours.play(Action::MoveHero { to: PLAIN });
    assert!(events.contains(&Event::EnemiesProvoked {
        player: support::SOLO,
        enemies: vec![ORCS_2],
    }));
    let legal = parcours.legal();
    assert!(legal.contains(&Action::StartCombat));
    assert!(
        !legal
            .iter()
            .any(|a| matches!(a, Action::EndTurn { .. } | Action::MoveHero { .. }))
    );
    assert!(
        card_plays(parcours)
            .iter()
            .all(|(_, _, _, e)| !matches!(e, Effect::Move(_) | Effect::Influence(_))),
        "ni Mouvement ni Influence"
    );

    // C3 — MKUE p. 9, *Ranged and Siege Attack Phase* ; spec 031 (C8).
    parcours.play(Action::StartCombat);
    assert_eq!(combat(&parcours.view()).phase, CombatPhase::RangedAndSiege);
    assert!(card_plays(parcours).iter().all(|(_, _, _, e)| !matches!(
        e,
        Effect::Attack {
            kind: AttackKind::Melee,
            ..
        }
    )));
    parcours.play(Action::NextCombatPhase);

    // C4 — MKUE p. 9, *Assign Damage Phase* 5.a à 5.c ; spec 031 (C16, C17) : Creuseurs non parés
    // (attaque 3), armure 2 : deux Blessures, une par tranche entamée.
    let events = parcours.play(Action::NextCombatPhase);
    let wounds: Vec<usize> = events
        .iter()
        .filter_map(|e| match e {
            Event::WoundsTaken { wounds, .. } => Some(wounds.len()),
            _ => None,
        })
        .collect();
    assert_eq!(wounds, [2]);
    assert_eq!(wounds_in(&parcours.view().hand), 2);
    assert_eq!(combat(&parcours.view()).phase, CombatPhase::Attack);

    // C5 — MKUE p. 9, 5.b ; p. 10, *Combat Outcomes* 1 ; specs 031 (C19, C22), 035 : Creuseurs,
    // armure 3 (fortifiés : seulement en phase d'attaque à distance), gloire 2.
    parcours.play(Action::DeclareAttack {
        targets: vec![foe_at(&parcours.view(), ORCS_2)],
    });
    gather_combat(parcours, 3, "vaincre les orques de la tuile 2");
    let events = parcours.play(Action::ResolveAttack);
    assert!(events.contains(&Event::EnemiesDefeated {
        player: support::SOLO,
        enemies: vec![foe_at(&parcours.view(), ORCS_2)],
        fame: 2,
    }));
    parcours.play(Action::EndCombat);
    let view = parcours.view();
    assert!(!view.enemy_tokens.contains_key(&ORCS_2));
    assert_eq!((view.fame, view.level, view.pending_level), (2, 1, None));

    // C6 — MKUE p. 11, 6 et 7.b ; *Wounds and Healing* 1.a ; specs 031 (C28), 035 (C8).
    let events = finish_turn(parcours);
    assert!(!has(&events, |e| matches!(e, Event::LevelGained { .. })));
    let view = parcours.view();
    assert_eq!(wounds_in(&view.hand), 2);
    assert_eq!(view.hand.len(), STARTING_HAND_LIMIT);
}

/// Du combat 1 à la clairière : C7 (si Tranquillité arrive en main) et C8.
fn to_the_glade(parcours: &mut Parcours) -> bool {
    let healed = heal(parcours);
    // C8 — MKUE p. 7, 4.c ; spec 031 (C22) : (2, -2) → (2, -3), entre deux cases voisines de
    // l'ennemi vaincu, ne provoque rien ; le village n'est pas voisin des orques de la tuile 1.
    let events = walk(parcours, &[(VILLAGE, 2), (GLADE, 3)]);
    assert!(!has(&events, |e| matches!(
        e,
        Event::EnemiesProvoked { .. }
    )));
    assert_eq!(parcours.view().figure, Some(GLADE));
    finish_turn(parcours);
    healed
}

/// Combat 2, jusqu'à la parade réussie : C9, C10 (C7 si pas encore fait).
fn second_combat_block(parcours: &mut Parcours, healed: bool) {
    let healed = healed || heal(parcours);
    precondition(healed, "Tranquillité en main avant le combat 2");

    // C9 — MKUE p. 8, 1.d ; spec 031 (C2).
    let events = parcours.play(Action::Challenge { enemy: ORCS_1 });
    assert!(events.contains(&Event::EnemyChallenged {
        player: support::SOLO,
        enemy: ORCS_1,
    }));
    parcours.play(Action::StartCombat);
    parcours.play(Action::NextCombatPhase);

    // C10 — MKUE p. 9, *Block Phase* ; specs 031 (C12, C18), 033 (C1).
    parcours.play(Action::DeclareBlock {
        target: foe_at(&parcours.view(), ORCS_1),
    });
    // Rôdeurs : attaque 4, pas vifs ; la parade à atteindre est celle que montre la vue.
    let enemy = combat(&parcours.view()).enemies[0].clone();
    assert_eq!(enemy.block_needed(), Some(4));
    gather_combat(parcours, 4, "parer les orques de la tuile 1");
    let events = parcours.play(Action::ResolveBlock);
    assert!(events.contains(&Event::EnemyBlocked {
        player: support::SOLO,
        enemy: foe_at(&parcours.view(), ORCS_1),
    }));
}

/// Du combat 2 paré au début de la manche 2 : C10 (fin), C11 à C14.
fn second_combat_and_new_round(parcours: &mut Parcours) {
    let events = parcours.play(Action::NextCombatPhase);
    assert!(!has(&events, |e| matches!(e, Event::WoundsTaken { .. })));
    assert!(!events.contains(&Event::CombatPhaseStarted {
        player: support::SOLO,
        phase: CombatPhase::AssignDamage,
    }));
    assert_eq!(combat(&parcours.view()).phase, CombatPhase::Attack);

    // C11 — MKUE p. 9, 5.b ; spec 035 (C3).
    parcours.play(Action::DeclareAttack {
        targets: vec![foe_at(&parcours.view(), ORCS_1)],
    });
    // Rôdeurs : armure 3, gloire 2 ; 2 + 2 = 4 franchit la ligne du niveau 2 (3, `levels.ron`).
    gather_combat(parcours, 3, "vaincre les orques de la tuile 1");
    let before = parcours.view();
    let events = parcours.play(Action::ResolveAttack);
    assert!(events.contains(&Event::EnemiesDefeated {
        player: support::SOLO,
        enemies: vec![foe_at(&parcours.view(), ORCS_1)],
        fame: 2,
    }));
    let after = parcours.view();
    assert_eq!(after.fame, 4);
    assert_eq!((after.level, after.armor), (1, 2));
    assert_eq!(after.pending_level, Some(2));

    // C12 — MKUE p. 5, *Reverting* 1 ; specs 024, 035 (C12).
    assert_eq!(parcours.undo(), Ok(()));
    assert_eq!(parcours.view(), before);
    assert_eq!(parcours.view().pending_level, None);
    assert_eq!(parcours.play(Action::ResolveAttack), events);
    assert_eq!(parcours.view(), after);
    parcours.play(Action::EndCombat);
    assert_eq!(parcours.view().level, 1);

    // C13 — MKUE p. 11, 6 ; spec 035 (C4).
    let events = finish_turn(parcours);
    let position = |wanted: fn(&Event) -> bool| events.iter().position(wanted);
    let ended = position(|e| matches!(e, Event::TurnEnded { .. }));
    let level = position(|e| matches!(e, Event::LevelGained { level: 2, .. }));
    let drawn = position(|e| matches!(e, Event::CardDrawn { .. }));
    assert!(ended.is_some() && level.is_some());
    assert!(ended < level);
    assert!(drawn.is_none_or(|drawn| Some(drawn) > level));
    // Spec 050 (C10, C11) : le niveau 2 donne une action avancée de l'offre, sur le deck, avant
    // la pioche ; le parcours prend la première.
    // Spec 051 : la compétence du niveau 2 d'abord ; le parcours garde la première.
    support::choose_first_skill(parcours);
    let view = parcours.view();
    assert_eq!(view.advanced_action_choices, 1);
    let card = view.offers[&Offer::AdvancedActions][0]
        .deed()
        .expect("une action avancée");
    let gained = parcours.play(Action::GainAdvancedAction { card });
    // Spec 044 : sur une clairière, le jeton de mana suit `TurnStarted`.
    let started = gained
        .iter()
        .rev()
        .find(|event| !matches!(event, Event::ManaTokenGained { .. }));
    assert!(matches!(started, Some(Event::TurnStarted { .. })));
    let view = parcours.view();
    assert_eq!((view.level, view.armor, view.hand_limit), (2, 2, 5));

    // C14 — MKUE p. 11, *Wounds and Healing* 1.c ; p. 4, 2.g ; specs 023, 031 (C29), 035 (C11).
    let fame = view.fame;
    parcours.finish_round();
    let view = parcours.view();
    assert_eq!((view.round, view.time_of_day), (2, TimeOfDay::Night));
    let wounds = wounds_kept(parcours.runner.log());
    assert!(wounds > 0, "des Blessures gardées");
    // Spec 050 : l'action avancée gagnée au niveau 2 en plus.
    assert_eq!(view.deck_size + view.hand.len(), DECK_SIZE + wounds + 1);
    assert!(view.discard.is_empty() && view.play_area.is_empty());
    assert_eq!(view.hand.len(), STARTING_HAND_LIMIT);
    assert_eq!((view.level, view.fame), (2, fame));

    // L2 — spec 034 : jamais assommé.
    assert!(
        !parcours
            .runner
            .log()
            .entries()
            .iter()
            .any(|entry| matches!(entry.event, Event::KnockedOut { .. }))
    );
}

/// Du début de la partie à la sauvegarde, après la parade du combat 2.
fn up_to_the_save_point(seed: u64) -> Parcours {
    let mut parcours = Parcours::arranged(&arranged(seed));
    setup(&parcours);
    first_combat(&mut parcours);
    let healed = to_the_glade(&mut parcours);
    second_combat_block(&mut parcours, healed);
    parcours
}

// C1 à C14 : un seul parcours, de la mise en place au début de la manche 2.
#[test]
fn parcours_deux_combats_et_un_niveau() {
    let mut parcours = up_to_the_save_point(SEED);
    second_combat_and_new_round(&mut parcours);
}

// C15 — ADR 0001 ; spec 024 : reprise en plein combat.
#[test]
fn parcours_reprise_en_plein_combat() {
    let mut played = up_to_the_save_point(SEED);
    let saved: (EventLog, RngPositions) = played.save();
    assert!(played.view().combat.is_some(), "sauvegarde en plein combat");
    second_combat_and_new_round(&mut played);

    let mut resumed = Parcours::resume(&saved);
    second_combat_and_new_round(&mut resumed);
    assert_eq!(resumed.runner.log(), played.runner.log());
    assert_eq!(resumed.runner.state(), played.runner.state());
}

/// Spec 039 (issue privée 322) : `cargo test -p arcanum-core --test parcours_j3 -- --ignored --nocapture` joue
/// le parcours sur les graines 0 à 99 et dit pourquoi les autres échouent.
#[test]
#[ignore]
fn balayer_les_graines() {
    let passed = support::sweep_seeds(0..100, |seed| {
        let mut parcours = up_to_the_save_point(seed);
        second_combat_and_new_round(&mut parcours);
    });
    assert!(
        passed.len() >= SWEEP_THRESHOLD,
        "{} graines sur 100, moins que {SWEEP_THRESHOLD}",
        passed.len()
    );
}

/// Le jeton de l'ennemi de cette case, dans le combat ou sur le plateau de jeu (spec 044 : les
/// actions de combat désignent l'ennemi par son jeton).
fn foe_at(view: &PlayerView, hex: Hex) -> EnemyToken {
    view.combat
        .iter()
        .flat_map(|combat| combat.enemies.iter())
        .find(|enemy| enemy.hex == hex)
        .map(|enemy| enemy.token)
        .or_else(|| {
            view.enemy_tokens
                .get(&hex)
                .and_then(|tokens| tokens.first().copied())
        })
        .unwrap_or_else(|| panic!("aucun ennemi en {hex:?}"))
}
