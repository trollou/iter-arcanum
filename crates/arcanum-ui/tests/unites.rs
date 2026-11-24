//! Scénarios de la spec 042 (Unités) côté affichage.

use arcanum_core::UndoStatus;
use arcanum_core::{
    Action, BasicAction, BasicColor, CardPlay, DamageTarget, Effect, Enemy, Event, GameEndReason,
    GameState, HealingTarget, ManaPayment, SidewaysEffect, Site, Unit, UnitCard, apply,
    final_score, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    AbilityButton, CombatChoice, Command, GameScreen, ScreenMode, ScreenPhase, combat_screen,
    game_screen, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::labels_and_commands;

use BasicAction::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn screen_in(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        mode,
        catalog,
    )
}

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    screen_in(state, &ScreenMode::Normal, catalog)
}

/// L'index de cette action parmi les actions légales.
fn index_of(state: &GameState, wanted: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"))
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn guardsmen() -> UnitCard {
    UnitCard::new(Unit::UtemGuardsmen, 1)
}

fn herbalists() -> UnitCard {
    UnitCard::new(Unit::Herbalists, 1)
}

fn foresters() -> UnitCard {
    UnitCard::new(Unit::Foresters, 1)
}

fn fire_golems() -> UnitCard {
    UnitCard::new(Unit::FireGolems, 1)
}

fn spent(state: GameState, unit: UnitCard) -> GameState {
    apply(
        state,
        &Event::UnitActivated {
            enemy: None,
            player: SOLO,
            unit,
            ability: 0,
            option: 0,
            effect: Effect::Move(0),
        },
    )
}

fn wounded(state: GameState, unit: UnitCard, wounds: u8) -> GameState {
    apply(
        state,
        &Event::UnitWounded {
            player: SOLO,
            unit,
            wounds,
        },
    )
}

fn activate(unit: UnitCard, ability: u8, option: u8, mana: &[ManaPayment]) -> Action {
    Action::ActivateUnit {
        enemy: None,
        unit,
        ability,
        option,
        mana: mana.to_vec(),
        target: None,
        leadership: false,
    }
}

// C39 — spec 042, *Interface*.
#[test]
fn panel_lists_units() {
    let catalog = french();
    let state = Units {
        level: 3,
        owned: vec![peasants(), fire_golems()],
        ..Units::default()
    }
    .state();
    let state = wounded(spent(state, peasants()), fire_golems(), 2);
    let section = screen_of(&state, &catalog).units;
    assert_eq!(section.header, "Unités : 2/2");
    // Spec 071, C13 : l'armure et les résistances en icônes, lues par leur texte de repli.
    let lines: Vec<(&str, String, Vec<String>, &str)> = section
        .units
        .iter()
        .map(|line| {
            (
                line.text.as_str(),
                line.armor.fallback_text(),
                line.resistances
                    .iter()
                    .map(|resistance| resistance.fallback_text())
                    .collect(),
                line.state.as_str(),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            (
                "Paysans · niveau I",
                "Armure 3".to_string(),
                vec![],
                "Épuisée"
            ),
            (
                "Golem de feu · niveau III",
                "Armure 4".to_string(),
                vec![
                    "Résistance au feu".to_string(),
                    "Résistance physique".to_string()
                ],
                "Disponible · Blessée (2)"
            ),
        ]
    );
}

// C40 — spec 042, *Interface* ; spec 022 pour le choix du mana.
#[test]
fn unit_ability_buttons() {
    let catalog = french();
    let state = Units {
        level: 3,
        owned: vec![peasants(), foresters()],
        ..Units::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    assert_eq!(screen.units.units.len(), 2);
    // Hors combat : pas d'attaque ni de parade.
    assert_eq!(
        screen.units.units[0].abilities,
        [
            AbilityButton {
                label: "Attaque ou Parade 2".into(),
                command: None,
                tooltip: None,
            },
            AbilityButton {
                label: "Influence 2".into(),
                command: Some(Command::Play(index_of(
                    &state,
                    &activate(peasants(), 1, 0, &[])
                ))),
                tooltip: None,
            },
            AbilityButton {
                label: "Mouvement 2".into(),
                command: Some(Command::Play(index_of(
                    &state,
                    &activate(peasants(), 2, 0, &[])
                ))),
                tooltip: None,
            },
        ]
    );
    // Spec 052 : la capacité des Bûcherons est codée, jouable comme les autres.
    assert_eq!(
        screen.units.units[1].abilities[0],
        AbilityButton {
            label: "Mouvement 2 ; forêts, collines et marais coûtent 1 de moins (min. 0)".into(),
            command: Some(Command::Play(index_of(
                &state,
                &activate(foresters(), 0, 0, &[])
            ))),
            tooltip: None,
        }
    );

    // Deux paiements possibles du mana vert : le choix du mana.
    let state = Units {
        owned: vec![herbalists()],
        ..Units::default()
    }
    .state();
    let state = apply(
        state,
        &Event::CrystalGained {
            player: SOLO,
            color: BasicColor::Green,
        },
    );
    let heal = &screen_of(&state, &catalog).units.units[0].abilities[0];
    assert_eq!(heal.label, "(mana vert) Soin 2");
    let choosing = ScreenMode::ChoosingUnitAbility {
        unit: herbalists(),
        ability: 0,
        option: None,
    };
    assert_eq!(
        heal.command,
        Some(Command::ChooseUnitAbility {
            unit: herbalists(),
            ability: 0,
            option: None,
        })
    );
    let prompt = screen_in(&state, &choosing, &catalog)
        .unit_prompt
        .expect("choix du mana");
    assert_eq!(prompt.text, "Herboristes : (mana vert) Soin 2");
    let payments: Vec<(&str, Command)> = prompt
        .payments
        .iter()
        .map(|button| (button.label.as_str(), button.command))
        .collect();
    assert_eq!(
        payments,
        [
            (
                "Soin 2 · Dé vert",
                Command::Play(index_of(
                    &state,
                    &activate(herbalists(), 0, 0, &[ManaPayment::Die(GREEN_DIE)])
                ))
            ),
            (
                "Soin 2 · Cristal vert (1 sur 3)",
                Command::Play(index_of(
                    &state,
                    &activate(herbalists(), 0, 0, &[ManaPayment::Crystal])
                ))
            ),
        ]
    );
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
}

/// L'interaction commencée au village, avec ce bonus : l'influence de départ (spec 040).
fn interacting(state: GameState, influence: i8) -> GameState {
    apply(
        state,
        &Event::InteractionStarted {
            player: SOLO,
            site: Site::Village,
            bonus: influence,
            city_shields: 0,
        },
    )
}

// C41 — spec 042, *Interface*.
#[test]
fn panel_offers_recruiting() {
    let catalog = french();
    let state = Units {
        offer: vec![peasants(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = interacting(state, 4);
    let screen = screen_of(&state, &catalog);
    // Les Gardes d'Utem coûtent 5 : pas assez d'influence.
    let recruit = Action::RecruitUnit {
        unit: peasants(),
        disband: None,
        bonds: false,
    };
    // Issue privée 341 : les Gardes restent visibles, grisés (C46).
    assert_eq!(
        screen
            .recruit
            .iter()
            .map(|button| (button.label.as_str(), button.command))
            .collect::<Vec<_>>(),
        [
            (
                "Recruter : Paysans (4)",
                Some(Command::Play(index_of(&state, &recruit)))
            ),
            ("Recruter : Gardes d'Utem (5)", None)
        ]
    );

    // Le jeton de commandement est pris : choisir l'unité à renvoyer.
    let state = Units {
        owned: vec![herbalists()],
        offer: vec![peasants()],
        ..Units::default()
    }
    .state();
    let state = interacting(state, 4);
    let screen = screen_of(&state, &catalog);
    assert_eq!(screen.recruit[0].label, "Recruter : Paysans (4)");
    assert_eq!(
        screen.recruit[0].command,
        Some(Command::ChooseDisband(peasants()))
    );
    let screen = screen_in(&state, &ScreenMode::Disbanding(peasants()), &catalog);
    let prompt = screen.disband_prompt.expect("choix de l'unité à renvoyer");
    assert_eq!(prompt.text, "Choisir l'unité à renvoyer");
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    let disband = screen.units.units[0].disband.clone().expect("Renvoyer");
    assert_eq!(disband.label, "Renvoyer");
    assert_eq!(
        disband.command,
        Command::Play(index_of(
            &state,
            &Action::RecruitUnit {
                unit: peasants(),
                disband: Some(herbalists()),
                bonds: false,
            }
        ))
    );
    // Hors de ce mode, pas de bouton « Renvoyer ».
    assert_eq!(screen_of(&state, &catalog).units.units[0].disband, None);
}

fn combat_of(state: &GameState, catalog: &Catalog) -> arcanum_ui::CombatScreen {
    combat_screen(
        &view(state, SOLO),
        &legal_actions(state, SOLO),
        &CombatChoice::default(),
        UndoStatus::Possible,
        catalog,
    )
    .expect("combat en cours")
}

fn assign(unit: UnitCard) -> Action {
    Action::AssignDamage {
        enemy: token(Enemy::Prowlers, 1),
        to: DamageTarget::Unit(unit),
    }
}

// C42 — spec 042, *Interface*.
#[test]
fn combat_screen_assigns_damage() {
    let catalog = french();
    let state = Units {
        owned: vec![peasants(), guardsmen()],
        placed: vec![(token(Enemy::Prowlers, 1), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = done(&to_block_phase(&state), Action::NextCombatPhase);
    let hero = Action::AssignDamage {
        enemy: foe(&state, ORCS_2),
        to: DamageTarget::Hero,
    };
    let screen = combat_of(&state, &catalog);
    assert_eq!(screen.phase_button, None);
    assert_eq!(screen.damage.len(), 1);
    assert_eq!(screen.damage[0].text, "Rôdeurs : 4 dégâts à assigner");
    assert_eq!(
        labels_and_commands(&screen.damage[0].buttons),
        [
            (
                "Sur Paysans (armure 3)",
                &Command::Play(index_of(&state, &assign(peasants())))
            ),
            (
                "Sur Gardes d'Utem (armure 5)",
                &Command::Play(index_of(&state, &assign(guardsmen())))
            ),
            ("Sur le héros", &Command::Play(index_of(&state, &hero))),
        ]
    );
    let state = done(&state, assign(peasants()));
    let screen = combat_of(&state, &catalog);
    assert_eq!(screen.damage.len(), 1);
    assert_eq!(screen.damage[0].text, "Rôdeurs : 1 dégâts à assigner");
    assert_eq!(
        screen.damage[0]
            .buttons
            .iter()
            .map(|button| button.label.as_str())
            .collect::<Vec<_>>(),
        ["Sur Gardes d'Utem (armure 5)", "Sur le héros"]
    );
    // L'historique de l'ennemi nomme l'unité blessée.
    assert!(
        screen.enemies[0]
            .history
            .contains(&"Paysans : 1 Blessure(s)".to_string()),
        "{:?}",
        screen.enemies[0].history
    );
}

// C43 — spec 042, *Interface*.
#[test]
fn panel_offers_healing() {
    let catalog = french();
    let state = Units {
        hand: vec![wound(1), card(March, 1)],
        owned: vec![guardsmen()],
        ..Units::default()
    }
    .state();
    let state = wounded(state, guardsmen(), 1);
    let state = apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: card(March, 1),
            play: CardPlay::Sideways(SidewaysEffect::Move),
            effect: Effect::Heal(2),
        },
    );
    let screen = screen_of(&state, &catalog);
    // Spec 071, C3 : icônes, lues par leur texte de repli ; le Soin reste en texte.
    assert_eq!(
        screen.points.fallback_text(),
        "Mouvement 0 · Influence 0 · Soin : 2"
    );
    let heal = |on| Action::SpendHealing { on };
    assert_eq!(
        screen
            .healing
            .iter()
            .map(|button| (button.label.as_str(), button.command))
            .collect::<Vec<_>>(),
        [
            (
                "Jeter une Blessure (1)",
                Command::Play(index_of(&state, &heal(HealingTarget::Wound)))
            ),
            (
                "Soigner Gardes d'Utem (2)",
                Command::Play(index_of(&state, &heal(HealingTarget::Unit(guardsmen()))))
            ),
        ]
    );
}

// C44 — MKUE p. 15.
#[test]
fn score_shows_leader() {
    let catalog = french();
    let state = Units {
        owned: vec![peasants(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = wounded(state, guardsmen(), 1);
    let finished = arcanum_core::PlayerView {
        final_score: Some(final_score(&state, SOLO)),
        game_end: Some(GameEndReason::LastRound),
        ..view(&state, SOLO)
    };
    let screen = game_screen(
        &finished,
        ScreenPhase::Finished,
        &ScreenMode::Normal,
        &catalog,
    );
    let window = screen.score_window.expect("fenêtre de fin");
    assert!(
        window.lines.contains(&"Le Commandant : 2".to_string()),
        "{:?}",
        window.lines
    );
}

// Spec 042 : chaque libellé des unités existe en français.
#[test]
fn every_unit_key_exists_in_french() {
    let keys = french().keys();
    for key in [
        "units.header",
        "units.line",
        "units.ready",
        "units.spent",
        "units.wounded",
        "units.unsupported",
        "units.recruit",
        "units.disband_prompt",
        "units.disband",
        "units.choice_prompt",
        "damage.remaining",
        "damage.to_unit",
        "damage.to_hero",
        "healing.points",
        "healing.wound",
        "healing.unit",
        "combat.history.unit_damage",
        "combat.history.unit_destroyed",
    ] {
        assert!(keys.contains(key), "{key}");
    }
    for unit in Unit::ALL {
        let abilities = unit.definition().abilities.len();
        for ability in 0..abilities {
            assert!(
                keys.contains(&unit.ability_key(ability)),
                "{unit:?} {ability}"
            );
        }
        assert!(!keys.contains(&unit.ability_key(abilities)), "{unit:?}");
    }
}

// Spec 042, *Effets de carte* : Rajeunissement rend disponible l'unité choisie, un bouton par
// unité épuisée.
#[test]
fn rejuvenate_offers_one_button_per_unit() {
    let catalog = french();
    let rejuvenate = card(Rejuvenate, 1);
    let state = Units {
        hand: vec![rejuvenate, card(March, 1)],
        owned: vec![peasants(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = spent(spent(state, peasants()), guardsmen());
    let screen = screen_of(&state, &catalog);
    let hand = screen
        .hand
        .iter()
        .find(|hand| hand.card == rejuvenate)
        .expect("Rajeunissement en main");
    let ready = |target| Action::PlayCardOnUnit {
        card: rejuvenate,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 3,
            discard: None,
            mana: None,
        },
        target,
    };
    let strong = |target| Action::PlayCardOnUnit {
        card: rejuvenate,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 3,
            discard: None,
            mana: ManaPayment::Die(GREEN_DIE),
            set_dice: [None, None],
            with: None,
        },
        target,
    };
    let labels: Vec<(&str, Command)> = hand
        .plays
        .iter()
        .filter(|button| button.label.contains("Rendre disponible"))
        .map(|button| (button.label.as_str(), button.command))
        .collect();
    assert_eq!(
        labels,
        [
            (
                "Jouer : Rendre disponible une unité de niveau 2 au plus · Paysans",
                Command::Play(index_of(&state, &ready(peasants())))
            ),
            (
                "Jouer : Rendre disponible une unité de niveau 2 au plus · Gardes d'Utem",
                Command::Play(index_of(&state, &ready(guardsmen())))
            ),
            (
                "Renforcer : Rendre disponible une unité de niveau 3 au plus · Paysans · Dé vert",
                Command::Play(index_of(&state, &strong(peasants())))
            ),
            (
                "Renforcer : Rendre disponible une unité de niveau 3 au plus · Gardes d'Utem · Dé \
                 vert",
                Command::Play(index_of(&state, &strong(guardsmen())))
            ),
        ]
    );
}

// Spec 042, *Interface* : les capacités ont les repères d'efficacité des cartes (spec 032).
#[test]
fn unit_buttons_mark_halved_attacks() {
    let catalog = french();
    let state = Units {
        owned: vec![peasants()],
        placed: vec![(ironclads(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = skip_to_attack_phase(&done(&state, Action::StartCombat));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let button = &screen_of(&state, &catalog).units.units[0].abilities[0];
    assert_eq!(button.label, "Attaque ou Parade 2 ÷2");
    assert!(button.command.is_some());
}

// C46 — Issue privée 341 ; MKUE p. 8, 4.a : chaque unité de l'offre recrutable dans ce lieu a sa ligne,
// grisée quand l'Influence manque, avec ses capacités en infobulle ; une unité d'un autre lieu
// n'apparaît pas.
#[test]
fn panel_greys_unaffordable_recruits() {
    let catalog = french();
    let state = Units {
        offer: vec![guardsmen(), fire_golems(), herbalists()],
        ..Units::default()
    }
    .state();
    let state = interacting(state, 0);
    let screen = screen_of(&state, &catalog);
    let lines: Vec<(&str, Option<Command>)> = screen
        .recruit
        .iter()
        .map(|button| (button.label.as_str(), button.command))
        .collect();
    assert_eq!(
        lines,
        [
            ("Recruter : Gardes d'Utem (5)", None),
            ("Recruter : Herboristes (3)", None),
        ]
    );
    let tooltip = screen.recruit[1].tooltip.clone().unwrap_or_default();
    assert!(
        tooltip.contains(&catalog.text(&Unit::Herbalists.ability_key(0))),
        "{tooltip}"
    );

    // Jetons tous pris mais Influence suffisante : actif, il mène au renvoi (spec 042, C41).
    let state = Units {
        owned: vec![peasants()],
        offer: vec![herbalists(), guardsmen()],
        ..Units::default()
    }
    .state();
    let state = interacting(state, 3);
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        screen
            .recruit
            .iter()
            .map(|button| button.command)
            .collect::<Vec<_>>(),
        [Some(Command::ChooseDisband(herbalists())), None]
    );
}
