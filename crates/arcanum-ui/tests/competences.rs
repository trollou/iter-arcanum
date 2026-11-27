//! Scénarios de la spec 051 côté affichage (compétences) : libellés, section Compétences du
//! panneau du bas, fenêtre du choix de compétence et bandeau du niveau pair.

use arcanum_core::{
    Action, BasicAction, CardId, Effect, Event, GameConfig, GameState, Hero, Offer, Scenario,
    Skill, Unit, UnitCard, apply, decide, legal_actions, new_game, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, GameScreen, LevelChoiceWindow, ScreenMode, ScreenPhase, Shortcut, SkillLine,
    SkillOffer, SkillsSection, game_screen, repository_assets_dir,
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
    // Issue #105 : le texte dit déjà la fréquence et « hors combat » (spec #68), l'infobulle ne
    // les répète plus au-dessus de lui.
    assert_eq!(
        section.skills[0].tooltip,
        "Marche forcée\nUne fois par tour : [[move]]Mouvement 2 de jour, ou \
         [[move]]Mouvement 1 de nuit."
    );
    assert_eq!(
        section.skills[1].tooltip,
        "Inspiration\n\
         Une fois par manche, hors combat, en la retournant : rendez disponible une unité, ou \
         soignez-en une."
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

// --- Fenêtre du niveau pair (spec #20) ---

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
    level_up(common, 3, Vec::new())
}

/// Comme `level_two`, avec `fame` gloire et les événements `extra` avant la fin du tour.
fn level_up(common: &[Skill], fame: u32, extra: Vec<Event>) -> (Vec<Event>, GameState) {
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
        fame,
    });
    events.extend(extra);
    let mut state = events.iter().fold(GameState::default(), apply);
    let card = state.hand(SOLO)[0];
    play_in(&mut events, &mut state, &sideways(card));
    play_in(&mut events, &mut state, &end_turn());
    (events, state)
}

/// Nom, encadrée ou non, commande qui l'encadre, note : une compétence de la fenêtre.
fn offered(offers: &[SkillOffer]) -> Vec<(&str, bool, Option<Command>, Option<&str>)> {
    offers
        .iter()
        .map(|offer| {
            (
                offer.name.as_str(),
                offer.selected,
                offer.select,
                offer.note.as_deref(),
            )
        })
        .collect()
}

fn level_window(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> LevelChoiceWindow {
    screen_of(state, mode, catalog)
        .level_choice
        .expect("la fenêtre du niveau pair")
}

/// L'offre d'actions avancées, dans l'ordre : Boule écrasante, Récit héroïque, Régénération.
fn advanced_offer(state: &GameState) -> Vec<CardId> {
    state
        .offer(Offer::AdvancedActions)
        .iter()
        .filter_map(|card| card.deed())
        .collect()
}

/// La commande de « Prendre » de chaque carte, `None` quand il est grisé.
fn takes(window: &LevelChoiceWindow) -> Vec<Option<Command>> {
    window
        .advanced_actions
        .iter()
        .map(|shown| shown.take.command)
        .collect()
}

fn pair(state: &GameState) -> Vec<Skill> {
    view(state, SOLO).skill_choice
}

// Spec #20, C1 — MKUE p. 11, 6.b : au niveau 2, une seule fenêtre, les deux compétences non
// encadrées, puis les actions avancées, « Prendre » grisé, et la ligne d'aide ; la fenêtre Offres
// ne s'ouvre pas (test modifié : remplace `skill_choice_window_at_even_level`).
#[test]
fn even_level_opens_one_level_window() {
    let catalog = french();
    let (events, state) = level_two(&[]);
    let pair = pair(&state);
    assert_eq!(pair, [Skill::ColdSwordsmanship, Skill::MotivationTovak]);
    let window = level_window(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        window.title,
        "Niveau 2 : choisir une compétence et une action avancée"
    );
    assert_eq!(window.revealed_title, "Compétences révélées");
    let motivation = catalog.text(&pair[1].label_key());
    assert_eq!(
        offered(&window.revealed),
        [
            (
                "Bretteur de glace",
                false,
                Some(Command::SelectSkill(pair[0])),
                None
            ),
            (
                motivation.as_str(),
                false,
                Some(Command::SelectSkill(pair[1])),
                None
            ),
        ]
    );
    let first = &window.revealed[0];
    assert_eq!(first.text, catalog.text(&pair[0].text_key()));
    assert!(!first.usage.is_empty());
    assert_eq!(first.take, None);
    assert!(window.common.is_empty());
    assert_eq!(window.advanced_actions_title, "Actions avancées");
    let names: Vec<&str> = window
        .advanced_actions
        .iter()
        .map(|shown| shown.card.name.as_str())
        .collect();
    assert_eq!(names, ["Boule écrasante", "Récit héroïque", "Régénération"]);
    assert!(window.advanced_actions.iter().all(|shown| {
        shown.take.label == "Prendre" && shown.take.command.is_none() && shown.card.take.is_none()
    }));
    assert_eq!(
        window.hint.as_deref(),
        Some("Choisissez d'abord une compétence.")
    );

    let mut session = support::session_of(events);
    assert!(session.offers_window(&catalog).is_none());
    session.command(Command::SelectSkill(pair[0]));
    session.command(Command::TakeLevelReward(advanced_offer(&state)[1]));
    session.run_frame();
    assert!(session.screen(&catalog).level_choice.is_none());
    assert!(session.offers_window(&catalog).is_none());
}

// Spec #20, C2 — MKUE p. 11, 6.b : une compétence révélée encadrée laisse prendre toute l'offre.
#[test]
fn revealed_skill_lets_any_advanced_action_be_taken() {
    let catalog = french();
    let (_, state) = level_two(&[]);
    let cold = Skill::ColdSwordsmanship;
    let window = level_window(&state, &ScreenMode::ChoosingLevelSkill(cold), &catalog);
    let selected: Vec<bool> = window.revealed.iter().map(|offer| offer.selected).collect();
    assert_eq!(selected, [true, false]);
    assert_eq!(window.hint, None);
    let offer = advanced_offer(&state);
    assert_eq!(
        takes(&window),
        offer
            .iter()
            .map(|&card| Some(Command::TakeLevelReward(card)))
            .collect::<Vec<_>>()
    );
}

// Spec #20, C3 — MKUE p. 11, 6.b ; FAQ p3 : une compétence de l'offre commune encadrée ne laisse
// prendre que la dernière place ; les autres disent pourquoi.
#[test]
fn common_skill_leaves_only_lowest_advanced_action() {
    let catalog = french();
    let (_, state) = level_two(&[Skill::HotSwordsmanship]);
    let hot = Skill::HotSwordsmanship;
    let window = level_window(&state, &ScreenMode::ChoosingLevelSkill(hot), &catalog);
    assert_eq!(
        offered(&window.common),
        [(
            "Bretteur de feu",
            true,
            Some(Command::SelectSkill(hot)),
            None
        )]
    );
    let offer = advanced_offer(&state);
    assert_eq!(
        takes(&window),
        [None, None, Some(Command::TakeLevelReward(offer[2]))]
    );
    let note =
        "Prendre une compétence de l'offre commune impose l'action avancée en dernière place.";
    let tooltips: Vec<Option<&str>> = window
        .advanced_actions
        .iter()
        .map(|shown| shown.take.tooltip.as_deref())
        .collect();
    assert_eq!(tooltips, [Some(note), Some(note), None]);
}

// Spec #20, C4 — Q1 : encadrer une autre compétence change l'encadré, sans rien jouer.
#[test]
fn changing_skill_plays_nothing() {
    let catalog = french();
    let (events, state) = level_two(&[Skill::HotSwordsmanship]);
    let pair = pair(&state);
    let mut session = support::session_of(events);
    let before = session.runner().log().len();
    session.command(Command::SelectSkill(Skill::HotSwordsmanship));
    session.command(Command::SelectSkill(pair[1]));
    session.run_frame();
    assert_eq!(session.runner().log().len(), before);
    let window = session
        .screen(&catalog)
        .level_choice
        .expect("la fenêtre du niveau pair");
    let selected: Vec<bool> = window
        .revealed
        .iter()
        .chain(&window.common)
        .map(|offer| offer.selected)
        .collect();
    assert_eq!(selected, [false, true, false]);
    assert!(takes(&window).iter().all(Option::is_some));
}

// Spec #20, C5 — MKUE p. 11, 6.b : « Prendre » joue la compétence encadrée puis la carte.
#[test]
fn take_level_reward_plays_skill_then_card() {
    let catalog = french();
    let (events, state) = level_two(&[]);
    let pair = pair(&state);
    let heroic_tale = advanced_offer(&state)[1];
    let mut session = support::session_of(events);
    session.command(Command::SelectSkill(pair[0]));
    session.command(Command::TakeLevelReward(heroic_tale));
    session.run_frame();
    let seen = session.view();
    assert_eq!(
        seen.skills
            .iter()
            .map(|owned| owned.skill)
            .collect::<Vec<_>>(),
        [pair[0]]
    );
    assert!(
        seen.common_skills
            .iter()
            .any(|common| common.skill == pair[1])
    );
    assert!(seen.hand.contains(&heroic_tale));
    assert!(session.screen(&catalog).level_choice.is_none());
}

// Spec #20, C6 — spec 051 C8 ; Compendium, *Skill Tokens* S3 : la compétence du héros dans
// l'offre commune, grisée avec sa note, ne s'encadre pas (test modifié : remplace
// `common_offer_in_skill_choice_window`).
#[test]
fn own_skill_cannot_be_selected() {
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
    let (_, state) = level_two(&[own, Skill::HotSwordsmanship]);
    let window = level_window(&state, &ScreenMode::Normal, &catalog);
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
                false,
                None,
                Some("Compétence de votre héros : pas prenable ici")
            ),
            (
                "Bretteur de feu",
                false,
                Some(Command::SelectSkill(Skill::HotSwordsmanship)),
                None
            ),
        ]
    );
    // L2 : un encadré qui n'est pas prenable est ignoré.
    let window = level_window(&state, &ScreenMode::ChoosingLevelSkill(own), &catalog);
    assert!(
        window
            .revealed
            .iter()
            .chain(&window.common)
            .all(|offer| !offer.selected)
    );
    assert!(window.hint.is_some());
    assert!(takes(&window).iter().all(Option::is_none));
}

// Spec #20, C7 — spec 051 C9 : offre d'actions avancées vide, la section est absente et chaque
// compétence prenable porte « Prendre », qui joue la compétence seule.
#[test]
fn empty_offer_takes_skill_alone() {
    let catalog = french();
    let config = tovak();
    let offer: Vec<arcanum_core::CommonCard> = new_game(&config, &mut config.rngs())
        .iter()
        .fold(GameState::default(), apply)
        .offer(Offer::AdvancedActions)
        .to_vec();
    let emptied = offer
        .into_iter()
        .map(|card| Event::OfferCardReturned {
            offer: Offer::AdvancedActions,
            card,
        })
        .collect();
    let (_, state) = level_up(&[], 3, emptied);
    let pair = pair(&state);
    let window = level_window(&state, &ScreenMode::Normal, &catalog);
    assert!(window.advanced_actions.is_empty());
    assert_eq!(window.hint, None);
    for (offer, skill) in window.revealed.iter().zip(&pair) {
        assert_eq!(offer.select, None);
        assert_eq!(
            offer
                .take
                .as_ref()
                .map(|button| (button.label.as_str(), button.command)),
            Some((
                "Prendre",
                Command::Play(index_of(&state, &Action::ChooseSkill { skill: *skill }))
            ))
        );
    }
}

// Spec #20, C8 — spec 051 C10 : deux niveaux pairs d'un coup ; la fenêtre du niveau 4 suit celle
// du niveau 2, sans compétence encadrée d'avance.
#[test]
fn second_even_level_starts_unselected() {
    let catalog = french();
    let (events, state) = level_up(&[], 15, Vec::new());
    let first = pair(&state);
    let mut session = support::session_of(events);
    session.command(Command::SelectSkill(first[0]));
    session.command(Command::TakeLevelReward(advanced_offer(&state)[0]));
    session.run_frame();
    let window = session
        .screen(&catalog)
        .level_choice
        .expect("la fenêtre du niveau 4");
    assert_eq!(
        window.title,
        "Niveau 4 : choisir une compétence et une action avancée"
    );
    let second: Vec<Skill> = window.revealed.iter().map(|offer| offer.skill).collect();
    assert_eq!(second.len(), 2);
    assert!(second.iter().all(|skill| !first.contains(skill)));
    assert!(
        window
            .revealed
            .iter()
            .chain(&window.common)
            .all(|offer| !offer.selected)
    );
    assert!(window.hint.is_some());
}

// Spec #20, C10, C19 — spec 071, C22 : 1 à 3 prennent la carte de cette place une fois une
// compétence encadrée ; sans encadré, ou sur une carte grisée, rien ; O n'ouvre pas les offres.
#[test]
fn level_window_shortcuts() {
    let catalog = french();
    let (events, state) = level_two(&[Skill::HotSwordsmanship]);
    let offer = advanced_offer(&state);
    let mut session = support::session_of(events);
    let before = session.runner().log().len();
    session.shortcut(Shortcut::Take(2));
    session.shortcut(Shortcut::Offers);
    session.run_frame();
    assert_eq!(session.runner().log().len(), before);
    assert!(session.offers_window(&catalog).is_none());
    session.command(Command::SelectSkill(Skill::HotSwordsmanship));
    let window = session
        .screen(&catalog)
        .level_choice
        .expect("la fenêtre du niveau pair");
    let keys: Vec<Option<&str>> = window
        .advanced_actions
        .iter()
        .map(|shown| shown.take.shortcut.as_deref())
        .collect();
    assert_eq!(keys, [None, None, Some("3")]);
    session.shortcut(Shortcut::Take(1));
    session.run_frame();
    assert_eq!(session.runner().log().len(), before);
    session.shortcut(Shortcut::Take(3));
    session.run_frame();
    let seen = session.view();
    assert!(seen.hand.contains(&offer[2]));
    assert_eq!(
        seen.skills
            .iter()
            .map(|owned| owned.skill)
            .collect::<Vec<_>>(),
        [Skill::HotSwordsmanship]
    );
}
