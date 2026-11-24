//! Scénarios de la spec 051 côté affichage (compétences) : libellés, section Compétences du
//! panneau du bas, fenêtre du choix de compétence et bandeau du niveau pair.

use arcanum_core::{
    Action, BasicAction, Effect, Event, GameConfig, GameState, Hero, Scenario, Skill, Unit,
    UnitCard, apply, decide, legal_actions, new_game, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, GameScreen, ScreenMode, ScreenPhase, SkillLine, SkillOffer, SkillsSection,
    game_screen, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;

use BasicAction::*;

// --- Outils ---

fn catalog(language: &str) -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language(language.into()),
    )
    .expect("fichier de langue du dépôt")
}

fn french() -> Catalog {
    catalog("fr")
}

fn screen_of(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
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

fn end_turn() -> Action {
    Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

// --- Libellés ---

// Spec 051, Libellés : un nom et un texte par compétence, en fr et en en.
#[test]
fn every_skill_has_a_name_and_a_text() {
    for language in ["fr", "en"] {
        let catalog = catalog(language);
        for skill in Skill::ALL {
            for key in [skill.label_key(), skill.text_key()] {
                let text = catalog.text(&key);
                assert!(!text.starts_with('⟦'), "{language} : {key} manquante");
            }
        }
    }
    let french = french();
    assert_eq!(
        french.text(&Skill::ColdSwordsmanship.label_key()),
        "Bretteur de glace"
    );
    // Q3 : la coopérative d'Arythea porte le nom de la carte française.
    assert_eq!(
        french.text(&Skill::RitualOfPain.label_key()),
        "Rituel de guérison"
    );
    assert_eq!(
        catalog("en").text(&Skill::ColdSwordsmanship.label_key()),
        "Cold Swordsmanship"
    );
}

// C11 — MKUE p. 11, 6.b : le bandeau du niveau pair dit ce qui a été gagné.
#[test]
fn even_level_banner_mentions_skill() {
    let catalog = french();
    assert_eq!(
        catalog.format("level.gained_even", &[("level", "2")]),
        "Niveau 2 : une compétence et une action avancée gagnées"
    );
    let (rage, march) = (card(Rage, 1), card(March, 1));
    let state = arena_at(VILLAGE, &[rage, march, card(March, 2)], &[], 3, 1);
    let state = done(&state, sideways(rage));
    let state = done(&state, end_turn());
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        screen.level_gained,
        ["Niveau 2 : une compétence et une action avancée gagnées"]
    );
}

// --- Section Compétences ---

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

/// L'index de l'action légale `wanted`.
#[track_caller]
fn index_of(state: &GameState, wanted: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"))
}

fn section(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> SkillsSection {
    screen_of(state, mode, catalog)
        .skills
        .expect("la section Compétences")
}

fn line(section: &SkillsSection, skill: Skill) -> &SkillLine {
    section
        .skills
        .iter()
        .find(|line| line.skill == skill)
        .unwrap_or_else(|| panic!("{skill:?} dans la section"))
}

/// Le texte de repli et la commande de chaque bouton d'option.
fn options(line: &SkillLine) -> Vec<(String, Option<Command>)> {
    line.options
        .iter()
        .map(|button| (button.fallback_text(), button.command))
        .collect()
}

// C26 — MKUE p. 5, *Using Skills* 2.a : une ligne par compétence gagnée, nom, fréquence et
// texte en infobulle ; pas de section sans compétence.
#[test]
fn skills_section_lists_owned_skills() {
    let catalog = french();
    let state = arena(VILLAGE, &move_hand(), &[]);
    assert_eq!(
        screen_of(&state, &ScreenMode::Normal, &catalog).skills,
        None
    );
    let state = owning(state, &[Skill::DoubleTime, Skill::Inspiration]);
    let section = section(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(section.title, "Compétences");
    assert_eq!(section.pile, "Pile : 0");
    let lines: Vec<(&str, &str, Option<&str>)> = section
        .skills
        .iter()
        .map(|line| {
            (
                line.name.as_str(),
                line.usage.as_str(),
                line.state.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            ("Marche forcée", "Une fois par tour", None),
            ("Inspiration", "Une fois par manche", None),
        ]
    );
    assert_eq!(
        section.skills[0].tooltip,
        "Marche forcée\nUne fois par tour\nMouvement 2 le jour, 1 la nuit."
    );
    assert_eq!(
        section.skills[1].tooltip,
        "Inspiration\nUne fois par manche\nHors combat\n\
         Retournée, hors combat : rendre disponible une unité, ou la soigner."
    );
}

// C12 — MKUE p. 5, *Using Skills* 4.c : un bouton par option, actif quand l'option est légale ;
// utilisée, la compétence est grisée jusqu'au tour suivant.
#[test]
fn skill_option_buttons_follow_legal_actions() {
    let catalog = french();
    let state = owning(
        arena(VILLAGE, &move_hand(), &[]),
        &[Skill::DoubleTime, Skill::ColdSwordsmanship],
    );
    let double = use_skill(Skill::DoubleTime, 0);
    let section = section(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        options(line(&section, Skill::DoubleTime)),
        [(
            "Mouvement 2".to_string(),
            Some(Command::Play(index_of(&state, &double)))
        )]
    );
    // Hors combat, Bretteur de glace ne sert pas : ses deux options sont grisées.
    assert_eq!(
        options(line(&section, Skill::ColdSwordsmanship)),
        [
            ("Attaque 2".to_string(), None),
            ("Attaque de glace 2".to_string(), None)
        ]
    );
    let used = done(&state, double);
    let section = self::section(&used, &ScreenMode::Normal, &catalog);
    let double_line = line(&section, Skill::DoubleTime);
    assert_eq!(double_line.state.as_deref(), Some("Utilisée ce tour"));
    assert_eq!(options(double_line), [("Mouvement 2".to_string(), None)]);

    // En phase d'attaque, les deux options de Bretteur de glace.
    let attacking = done(
        &skip_to_attack_phase(&owning(
            fight(VILLAGE, &move_hand(), &[(prowlers(), ORCS_2)], &[ORCS_2]),
            &[Skill::ColdSwordsmanship],
        )),
        declare(&[prowlers()]),
    );
    let section = self::section(&attacking, &ScreenMode::Normal, &catalog);
    assert_eq!(
        options(line(&section, Skill::ColdSwordsmanship)),
        [
            (
                "Attaque 2".to_string(),
                Some(Command::Play(index_of(
                    &attacking,
                    &use_skill(Skill::ColdSwordsmanship, 0)
                )))
            ),
            (
                "Attaque de glace 2".to_string(),
                Some(Command::Play(index_of(
                    &attacking,
                    &use_skill(Skill::ColdSwordsmanship, 1)
                )))
            ),
        ]
    );
}

// C16 — MKUE p. 5, 4.a : une compétence « par manche » utilisée est retournée ; texte : une seule
// Motivation jusqu'à la fin du tour suivant. (C24, une compétence du second lot sans bouton, n'a plus
// lieu d'être : spec 058, toutes servent.)
#[test]
fn flipped_and_blocked_skills_are_greyed() {
    let catalog = french();
    let state = owning(
        arena(VILLAGE, &move_hand(), &[]),
        &[
            Skill::DarkFireMagic,
            Skill::MotivationTovak,
            Skill::MotivationNorowas,
        ],
    );
    let section = section(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(line(&section, Skill::DarkFireMagic).options.len(), 2);

    let state = done(&state, use_skill(Skill::DarkFireMagic, 1));
    let state = done(&state, use_skill(Skill::MotivationTovak, 0));
    let section = self::section(&state, &ScreenMode::Normal, &catalog);
    let flipped = line(&section, Skill::DarkFireMagic);
    assert_eq!(
        flipped.state.as_deref(),
        Some("Retournée jusqu'à la prochaine manche")
    );
    assert!(
        flipped
            .options
            .iter()
            .all(|button| button.command.is_none())
    );
    let blocked = line(&section, Skill::MotivationNorowas);
    assert_eq!(
        blocked.state.as_deref(),
        Some("Une autre Motivation a servi : pas avant la fin du prochain tour")
    );
    assert!(
        blocked
            .options
            .iter()
            .all(|button| button.command.is_none())
    );
}

fn fire_golems() -> UnitCard {
    UnitCard::new(Unit::FireGolems, 1)
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

/// Le village, `owned` recrutées et épuisées, Inspiration gagnée.
fn inspired(owned: &[UnitCard]) -> GameState {
    let mut events = arena_events(VILLAGE, &move_hand(), &[], &[]);
    events.extend(unit_events(owned, &[]));
    events.extend(owned.iter().map(|&unit| Event::UnitActivated {
        player: SOLO,
        unit,
        ability: 0,
        option: 0,
        effect: Effect::Nothing,
        enemy: None,
    }));
    owning(
        events.iter().fold(GameState::default(), apply),
        &[Skill::Inspiration],
    )
}

fn inspiration(option: u8, unit: UnitCard) -> Action {
    Action::UseSkill {
        skill: Skill::Inspiration,
        option,
        unit: Some(unit),
        target: None,
    }
}

// C22 — texte ; Compendium, *Inspiration* Q1 : une seule unité possible, le bouton l'utilise ;
// plusieurs, l'unité se choisit dans la section Unités, comme la cible d'une capacité.
#[test]
fn inspiration_picks_the_unit() {
    let catalog = french();
    let state = inspired(&[fire_golems()]);
    let section = section(&state, &ScreenMode::Normal, &catalog);
    let ready = &line(&section, Skill::Inspiration).options[0];
    assert_eq!(
        ready.command,
        Some(Command::Play(index_of(
            &state,
            &inspiration(0, fire_golems())
        )))
    );

    let state = inspired(&[fire_golems(), peasants()]);
    let section = self::section(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        line(&section, Skill::Inspiration).options[0].command,
        Some(Command::ChooseSkillUnit {
            skill: Skill::Inspiration,
            option: 0,
        })
    );
    let mode = ScreenMode::ChoosingSkillUnit {
        skill: Skill::Inspiration,
        option: 0,
    };
    let screen = screen_of(&state, &mode, &catalog);
    let prompt = screen
        .disband_prompt
        .expect("le bandeau du choix de l'unité");
    assert_eq!(prompt.text, "Inspiration : choisir l'unité");
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    let picks: Vec<(&str, Command)> = screen
        .units
        .units
        .iter()
        .map(|unit| {
            let button = unit.disband.as_ref().expect("un bouton par unité épuisée");
            (button.label.as_str(), button.command)
        })
        .collect();
    assert_eq!(
        picks,
        [
            (
                "Choisir",
                Command::Play(index_of(&state, &inspiration(0, fire_golems())))
            ),
            (
                "Choisir",
                Command::Play(index_of(&state, &inspiration(0, peasants())))
            ),
        ]
    );
}

// C22 : la session passe au choix de l'unité, et « Annuler » le quitte.
#[test]
fn inspiration_unit_choice_in_session() {
    let catalog = french();
    let mut events = arena_events(VILLAGE, &move_hand(), &[], &[]);
    events.extend(unit_events(&[fire_golems(), peasants()], &[]));
    events.extend(
        [fire_golems(), peasants()].map(|unit| Event::UnitActivated {
            player: SOLO,
            unit,
            ability: 0,
            option: 0,
            effect: Effect::Nothing,
            enemy: None,
        }),
    );
    events.push(Event::SkillGained {
        player: SOLO,
        skill: Skill::Inspiration,
    });
    let mut session = support::session_of(events);
    session.command(Command::ChooseSkillUnit {
        skill: Skill::Inspiration,
        option: 0,
    });
    assert!(session.screen(&catalog).disband_prompt.is_some());
    session.command(Command::CancelChoice);
    assert!(session.screen(&catalog).disband_prompt.is_none());
}

// --- Fenêtre du choix de compétence ---

fn tovak() -> GameConfig {
    GameConfig {
        seed: 4,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

#[track_caller]
fn play_in(events: &mut Vec<Event>, state: &mut GameState, action: &Action) {
    let produced = decide(state, SOLO, action, &mut tovak().rngs())
        .unwrap_or_else(|error| panic!("{action:?} : {error:?}"));
    *state = produced.iter().fold(state.clone(), apply);
    events.extend(produced);
}

/// Tovak, la vraie mise en place du bac à sable, l'offre commune préparée avec `common`, 3 de
/// gloire et le tour fini : niveau 2, deux compétences révélées.
fn level_two(common: &[Skill]) -> (Vec<Event>, GameState) {
    let config = tovak();
    let mut events = new_game(&config, &mut config.rngs());
    events.extend(
        common
            .iter()
            .map(|&skill| Event::SkillToCommonOffer { skill }),
    );
    events.push(Event::EnemiesDefeated {
        player: SOLO,
        enemies: Vec::new(),
        fame: 3,
    });
    let mut state = events.iter().fold(GameState::default(), apply);
    let card = state.hand(SOLO)[0];
    play_in(&mut events, &mut state, &sideways(card));
    play_in(&mut events, &mut state, &end_turn());
    (events, state)
}

fn offered(offers: &[SkillOffer]) -> Vec<(&str, Option<Command>, Option<&str>)> {
    offers
        .iter()
        .map(|offer| {
            (
                offer.name.as_str(),
                offer.take.as_ref().map(|button| button.command),
                offer.note.as_deref(),
            )
        })
        .collect()
}

// C4 à C6 — MKUE p. 11, 6.b : la fenêtre s'ouvre d'office avec les deux compétences révélées ;
// l'action avancée attend la compétence.
#[test]
fn skill_choice_window_at_even_level() {
    let catalog = french();
    let (events, state) = level_two(&[]);
    let pair = view(&state, SOLO).skill_choice;
    assert_eq!(pair.len(), 2);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let window = screen
        .skill_choice
        .expect("la fenêtre du choix de compétence");
    assert_eq!(window.title, "Niveau 2 : choisir une compétence");
    assert_eq!(window.revealed_title, "Compétences révélées");
    let names: Vec<String> = pair
        .iter()
        .map(|skill| catalog.text(&skill.label_key()))
        .collect();
    assert_eq!(
        offered(&window.revealed),
        [
            (
                names[0].as_str(),
                Some(Command::Play(index_of(
                    &state,
                    &Action::ChooseSkill { skill: pair[0] }
                ))),
                None
            ),
            (
                names[1].as_str(),
                Some(Command::Play(index_of(
                    &state,
                    &Action::ChooseSkill { skill: pair[1] }
                ))),
                None
            ),
        ]
    );
    let first = &window.revealed[0];
    assert_eq!(first.text, catalog.text(&pair[0].text_key()));
    assert!(!first.usage.is_empty());
    assert_eq!(
        first.take.as_ref().map(|button| button.label.as_str()),
        Some("Prendre")
    );
    assert!(window.common.is_empty());

    // C6 : la fenêtre Offres attend la compétence, puis s'ouvre d'office.
    let mut session = support::session_of(events);
    assert!(session.screen(&catalog).skill_choice.is_some());
    assert_eq!(session.offers_window(&catalog), None);
    support::play(&mut session, &Action::ChooseSkill { skill: pair[0] });
    assert_eq!(session.screen(&catalog).skill_choice, None);
    let offers = session.offers_window(&catalog).expect("ouverte d'office");
    assert_eq!(offers.title, "Niveau 2 : choisissez une action avancée");
}

// C7, C8 — MKUE p. 11, 6.b ; FAQ p3 ; Compendium, *Skill Tokens* S3 : l'offre commune sous les
// compétences révélées ; celle d'un autre héros se prend, et impose la dernière carte ; celle du
// héros, jamais.
#[test]
fn common_offer_in_skill_choice_window() {
    let catalog = french();
    let pile = {
        let config = tovak();
        new_game(&config, &mut config.rngs())
            .iter()
            .fold(GameState::default(), apply)
            .skill_pile(SOLO)
            .to_vec()
    };
    let own = pile[5];
    let (events, state) = level_two(&[own, Skill::HotSwordsmanship]);
    let window = screen_of(&state, &ScreenMode::Normal, &catalog)
        .skill_choice
        .expect("la fenêtre du choix de compétence");
    assert_eq!(window.common_title, "Offre de compétences communes");
    assert_eq!(
        window.common_note,
        "Prendre une compétence de l'offre commune impose l'action avancée en dernière place."
    );
    let own_name = catalog.text(&own.label_key());
    assert_eq!(
        offered(&window.common),
        [
            (
                own_name.as_str(),
                None,
                Some("Compétence de votre héros : pas prenable ici")
            ),
            (
                "Bretteur de feu",
                Some(Command::Play(index_of(
                    &state,
                    &Action::ChooseSkill {
                        skill: Skill::HotSwordsmanship
                    }
                ))),
                None
            ),
        ]
    );

    // Après Bretteur de feu, seule la dernière carte de l'offre d'actions avancées se prend.
    let mut session = support::session_of(events);
    support::play(
        &mut session,
        &Action::ChooseSkill {
            skill: Skill::HotSwordsmanship,
        },
    );
    let offers = session.offers_window(&catalog).expect("ouverte d'office");
    let takes: Vec<bool> = offers.columns[0]
        .cards
        .iter()
        .map(|card| card.take.is_some())
        .collect();
    assert_eq!(takes, [false, false, true]);
}
