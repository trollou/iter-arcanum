//! Scénarios de la spec 058 côté affichage (compétences : second lot) : textes des effets, état
//! « au centre » et avantages, paiements d'Invocation et de Polarisation, choix des cibles, unités.

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, Event, GameState, Hex, ManaColor,
    ManaPayment, PolarSource, SidewaysEffect, Skill, SkillMana, SkillTarget, Unit, UnitCard, apply,
    legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, GameScreen, HandCard, ManaChoice, ScreenMode, ScreenPhase, SkillChoice, SkillLine,
    game_screen, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fichier de langue du dépôt")
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

#[track_caller]
fn line(screen: &GameScreen, skill: Skill) -> SkillLine {
    screen
        .skills
        .as_ref()
        .expect("la section Compétences")
        .skills
        .iter()
        .find(|line| line.skill == skill)
        .cloned()
        .unwrap_or_else(|| panic!("{skill:?} dans la section"))
}

fn tooltips(line: &SkillLine) -> Vec<&str> {
    line.options
        .iter()
        .map(|button| button.tooltip.as_str())
        .collect()
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

fn march() -> CardId {
    card(BasicAction::March, 1)
}

fn rage() -> CardId {
    card(BasicAction::Rage, 1)
}

// --- Textes des effets et état ---

// Spec 058, Libellés et Interface : chaque option des compétences du second lot a son texte
// (plus de « Pas encore disponible ») ; Invocation, Polarisation, Charisme et Loyauté sans faille
// n'ont pas de bouton dans la section (leurs effets passent par la fenêtre du mana et les unités).
#[test]
fn second_batch_options_have_texts() {
    let catalog = french();
    let skills = [
        Skill::GlitteringFortune,
        Skill::ForwardMarch,
        Skill::IFeelNoPain,
        Skill::RitualOfPain,
        Skill::Flight,
        Skill::ResistanceBreak,
        Skill::SourceOpening,
        Skill::CalmingTheWeather,
        Skill::ManaOverload,
        Skill::Invocation,
        Skill::Polarization,
        Skill::Leadership,
        Skill::BondsOfLoyalty,
    ];
    let state = owning(arena(VILLAGE, &move_hand(), &[]), &skills);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let expected: [(Skill, &[&str]); 13] = [
        (
            Skill::GlitteringFortune,
            &["Influence d'interaction 1 par couleur de cristal"],
        ),
        (
            Skill::ForwardMarch,
            &["Mouvement 1 par unité prête et non blessée (3 au plus)"],
        ),
        (
            Skill::IFeelNoPain,
            &["Défausser une Blessure, piocher une carte"],
        ),
        (
            Skill::RitualOfPain,
            &[
                "Jeter 0 Blessure(s)",
                "Jeter 1 Blessure(s)",
                "Jeter 2 Blessure(s)",
            ],
        ),
        (
            Skill::Flight,
            &["Vol : 1 case(s)", "Vol : 2 case(s) pour 2 Mouvement"],
        ),
        (
            Skill::ResistanceBreak,
            &["Un ennemi : Armure −1 par résistance"],
        ),
        (Skill::SourceOpening, &["Relancer un dé de la Source"]),
        (
            Skill::CalmingTheWeather,
            &["Terrains −2 ce tour (au moins 1)"],
        ),
        (Skill::ManaOverload, &["Un jeton de la couleur choisie"]),
        (Skill::Invocation, &[]),
        (Skill::Polarization, &[]),
        (Skill::Leadership, &[]),
        (Skill::BondsOfLoyalty, &[]),
    ];
    for (skill, texts) in expected {
        assert_eq!(tooltips(&line(&screen, skill)), texts, "{skill:?}");
    }
}

// C36, C37 — MKUE p. 5, 4.b : une compétence posée au centre le dit ; au tour suivant, son
// avantage est rappelé dans les effets en cours.
#[test]
fn skill_in_center_and_its_advantage() {
    let catalog = french();
    let state = owning(
        arena(VILLAGE, &[wound(1), wound(2), rage()], &[]),
        &[Skill::RitualOfPain],
    );
    let placed = done(&state, use_skill(Skill::RitualOfPain, 0));
    let screen = screen_of(&placed, &ScreenMode::Normal, &catalog);
    assert_eq!(
        line(&screen, Skill::RitualOfPain).state.as_deref(),
        Some("Au centre : avantage au prochain tour")
    );
    // Pas d'avantage le tour même (Q1).
    let advantage = "Avantage de Rituel de guérison";
    assert!(
        !screen
            .modifiers
            .iter()
            .flat_map(|list| &list.lines)
            .any(|line| line == advantage)
    );
    let next = screen_of(&next_turn(placed), &ScreenMode::Normal, &catalog);
    // Au tour de l'avantage, le bouton le dit : plus d'état grisé.
    assert_eq!(line(&next, Skill::RitualOfPain).state, None);
    let lines = next.modifiers.expect("les effets en cours").lines;
    assert!(lines.iter().any(|line| line == advantage), "{lines:?}");
}

// C6 — Compendium, *Who Needs Magic* Q1 : après Magie inutile à +3, le rappel dans les effets en
// cours et les dés de la Source grisés.
#[test]
fn who_needs_magic_forbids_dice_on_screen() {
    let catalog = french();
    let state = owning(
        arena(VILLAGE, &[march(), card(BasicAction::Stamina, 1)], &[]),
        &[Skill::WhoNeedsMagic],
    );
    let before = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert!(before.source.dice.iter().all(|die| !die.forbidden));
    let three = done(
        &state,
        Action::PlayCard {
            card: march(),
            play: arcanum_core::CardPlay::SidewaysSkill {
                sideways: arcanum_core::SidewaysEffect::Move,
                skill: Skill::WhoNeedsMagic,
                option: 1,
                mana: None,
            },
        },
    );
    let screen = screen_of(&three, &ScreenMode::Normal, &catalog);
    let lines = screen.modifiers.expect("les effets en cours").lines;
    assert!(
        lines
            .iter()
            .any(|line| line == "Magie inutile : plus de dé de la Source ce tour"),
        "{lines:?}"
    );
    assert!(!screen.source.dice.is_empty());
    assert!(screen.source.dice.iter().all(|die| die.forbidden));
}

// C18 — l'Influence d'interaction s'affiche à côté de l'Influence.
#[test]
fn glittering_fortune_shows_interaction_influence() {
    let catalog = french();
    let state = done(
        &crystals(
            owning(
                arena(VILLAGE, &move_hand(), &[]),
                &[Skill::GlitteringFortune],
            ),
            &[BasicColor::Red, BasicColor::Blue],
        ),
        Action::Interact,
    );
    assert_eq!(
        screen_of(&state, &ScreenMode::Normal, &catalog).interaction_influence,
        None
    );
    let after = done(&state, use_skill(Skill::GlitteringFortune, 0));
    assert_eq!(
        screen_of(&after, &ScreenMode::Normal, &catalog)
            .interaction_influence
            .as_deref(),
        Some("Influence d'interaction : 2")
    );
}

// --- B. Fenêtre du mana ---

/// Les libellés des paiements du choix du mana de `card` renforcée.
fn mana_lines(state: &GameState, card: CardId, catalog: &Catalog) -> Vec<String> {
    let mode = ScreenMode::ChoosingMana(ManaChoice {
        card,
        option: 0,
        discard: None,
    });
    screen_of(state, &mode, catalog)
        .mana_prompt
        .expect("le choix du mana")
        .payments
        .into_iter()
        .map(|button| button.label)
        .collect()
}

// C10, C13 — Interface : Invocation, une ligne par carte défaussable et couleur ; Polarisation,
// une ligne par mana d'origine ; chacune joue son paiement.
#[test]
fn invocation_and_polarization_lines_in_mana_window() {
    let catalog = french();
    let state = crystals(
        owning(
            arena(VILLAGE, &[march(), rage()], &[]),
            &[Skill::Invocation, Skill::Polarization],
        ),
        &[BasicColor::White],
    );
    let lines = mana_lines(&state, march(), &catalog);
    assert!(
        lines
            .iter()
            .any(|line| line == "Invocation : défausser Rage (vert)"),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Polarisation : Cristal blanc comme vert"),
        "{lines:?}"
    );
    let mode = ScreenMode::ChoosingMana(ManaChoice {
        card: march(),
        option: 0,
        discard: None,
    });
    let prompt = screen_of(&state, &mode, &catalog)
        .mana_prompt
        .expect("le choix du mana");
    let invoke = prompt
        .payments
        .iter()
        .find(|button| button.label.starts_with("Invocation"))
        .expect("la ligne d'Invocation");
    let wanted = Action::PlayCard {
        card: march(),
        play: arcanum_core::CardPlay::Strong {
            option: 0,
            discard: None,
            mana: arcanum_core::ManaPayment::Invocation(rage()),
            set_dice: [None, None],
            with: None,
            extra_mana: None,
            gain: None,
        },
    };
    assert_eq!(invoke.command, Command::Play(index_of(&state, &wanted)));
    let polarized = arcanum_core::ManaPayment::Polarized {
        from: ManaColor::Basic(BasicColor::White),
        source: PolarSource::Crystal,
    };
    let polarize = prompt
        .payments
        .iter()
        .find(|button| button.label == "Polarisation : Cristal blanc comme vert")
        .expect("la ligne de Polarisation");
    let wanted = Action::PlayCard {
        card: march(),
        play: arcanum_core::CardPlay::Strong {
            option: 0,
            discard: None,
            mana: polarized,
            set_dice: [None, None],
            with: None,
            extra_mana: None,
            gain: None,
        },
    };
    assert_eq!(polarize.command, Command::Play(index_of(&state, &wanted)));
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

fn choosing(skill: Skill, option: u8) -> ScreenMode {
    ScreenMode::ChoosingSkillTarget(SkillChoice {
        skill,
        option,
        card: None,
    })
}

#[track_caller]
fn hand_card(screen: &GameScreen, card: CardId) -> HandCard {
    screen
        .hand
        .iter()
        .find(|shown| shown.card == card)
        .cloned()
        .unwrap_or_else(|| panic!("{card:?} en main"))
}

fn plays(card: &HandCard) -> Vec<(&str, Command)> {
    card.plays
        .iter()
        .map(|button| (button.label.as_str(), button.command))
        .collect()
}

// C5, Interface — Q6 : le bouton de la compétence ouvre le choix de la carte ; sur chaque carte,
// ses jeux de côté avec la valeur remplacée.
#[test]
fn sideways_skill_button_opens_card_choice() {
    let catalog = french();
    let state = owning(
        arena(VILLAGE, &[march(), wound(1)], &[]),
        &[Skill::IDontGiveADamn],
    );
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let flexibility = line(&screen, Skill::IDontGiveADamn);
    assert_eq!(flexibility.options.len(), 1);
    assert_eq!(flexibility.options[0].fallback_text(), "De côté : 2");
    let mode = choosing(Skill::IDontGiveADamn, 0);
    assert_eq!(
        flexibility.options[0].command,
        Some(Command::ChooseSkillTarget(SkillChoice {
            skill: Skill::IDontGiveADamn,
            option: 0,
            card: None,
        }))
    );
    let screen = screen_of(&state, &mode, &catalog);
    let prompt = screen.discard_prompt.as_ref().expect("le bandeau du choix");
    assert_eq!(
        prompt.text,
        "Flexibilité : choisir la carte à jouer de côté"
    );
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    let index = |sideways| {
        index_of(
            &state,
            &skill_sideways(march(), sideways, Skill::IDontGiveADamn, 0, None),
        )
    };
    assert_eq!(
        plays(&hand_card(&screen, march())),
        [
            (
                "Jouer de côté : Mouvement 2",
                Command::Play(index(SidewaysEffect::Move))
            ),
            (
                "Jouer de côté : Influence 2",
                Command::Play(index(SidewaysEffect::Influence))
            ),
        ]
    );
    // Une Blessure ne se joue pas avec Flexibilité.
    assert!(hand_card(&screen, wound(1)).plays.is_empty());
}

// C7 — Pouvoir universel : la carte, puis le mana dans le bandeau (fenêtre du mana).
#[test]
fn universal_power_chooses_mana_after_card() {
    let catalog = french();
    let green = ManaColor::Basic(BasicColor::Green);
    let red = ManaColor::Basic(BasicColor::Red);
    let state = [green, red].iter().fold(
        owning(arena(VILLAGE, &[march()], &[]), &[Skill::UniversalPower]),
        |state, &color| {
            apply(
                state,
                &Event::ManaTokenGained {
                    player: SOLO,
                    color,
                },
            )
        },
    );
    let screen = screen_of(&state, &choosing(Skill::UniversalPower, 0), &catalog);
    let card = hand_card(&screen, march());
    let chosen = SkillChoice {
        skill: Skill::UniversalPower,
        option: 0,
        card: Some((march(), SidewaysEffect::Move)),
    };
    let move_play = card
        .plays
        .iter()
        .find(|button| button.command == Command::ChooseSkillTarget(chosen))
        .expect("Marche jouée de côté en Mouvement, mana à choisir");
    assert_eq!(
        move_play.label,
        "Jouer de côté : Mouvement 3 ou Mouvement 4"
    );
    let screen = screen_of(&state, &ScreenMode::ChoosingSkillTarget(chosen), &catalog);
    let prompt = screen.mana_prompt.expect("le choix du mana");
    let labels: Vec<&str> = prompt
        .payments
        .iter()
        .map(|button| button.label.as_str())
        .collect();
    assert!(labels.contains(&"Jeton vert · Mouvement 4"), "{labels:?}");
    assert!(labels.contains(&"Jeton rouge · Mouvement 3"), "{labels:?}");
    let green_token = prompt
        .payments
        .iter()
        .find(|button| button.label == "Jeton vert · Mouvement 4")
        .expect("le jeton vert");
    let wanted = skill_sideways(
        march(),
        SidewaysEffect::Move,
        Skill::UniversalPower,
        0,
        Some(SkillMana {
            color: green,
            payment: ManaPayment::Token,
        }),
    );
    assert_eq!(
        green_token.command,
        Command::Play(index_of(&state, &wanted))
    );
}

// C37 — l'avantage du Rituel de guérison : au tour suivant, un bouton « Avantage de … » qui fait
// choisir la Blessure à jouer de côté.
#[test]
fn ritual_advantage_plays_wound_sideways() {
    let catalog = french();
    let state = owning(
        arena(VILLAGE, &[wound(1), wound(2), rage()], &[]),
        &[Skill::RitualOfPain],
    );
    let next = next_turn(done(&state, use_skill(Skill::RitualOfPain, 0)));
    let screen = screen_of(&next, &ScreenMode::Normal, &catalog);
    let ritual = line(&screen, Skill::RitualOfPain);
    let advantage = ritual
        .options
        .iter()
        .find(|button| button.fallback_text() == "Avantage de Rituel de guérison")
        .expect("le bouton de l'avantage");
    assert_eq!(
        advantage.command,
        Some(Command::ChooseSkillTarget(SkillChoice {
            skill: Skill::RitualOfPain,
            option: 0,
            card: None,
        }))
    );
    let screen = screen_of(&next, &choosing(Skill::RitualOfPain, 0), &catalog);
    let wound_card = hand_card(&screen, wound(1));
    let wound_plays = plays(&wound_card);
    assert!(
        wound_plays
            .iter()
            .any(|(label, command)| *label == "Jouer de côté : Mouvement 3"
                && *command
                    == Command::Play(index_of(
                        &next,
                        &skill_sideways(
                            wound(1),
                            SidewaysEffect::Move,
                            Skill::RitualOfPain,
                            0,
                            None
                        )
                    ))),
        "{wound_plays:?}"
    );
    assert!(hand_card(&screen, rage()).plays.is_empty());
}

// --- D, F, G. Cibles ---

fn target_skill(skill: Skill, option: u8, target: Option<SkillTarget>) -> Action {
    Action::UseSkill {
        skill,
        option,
        unit: None,
        target,
    }
}

/// La forêt à l'ouest de la clairière magique (`docs/reference/bac-a-sable.md`).
const WEST_FOREST: Hex = Hex::new(0, -3);

// C22 — Interface : le bouton de l'option met le plateau en choix de case, cases possibles en
// évidence et seules cliquables ; bandeau et « Annuler ».
#[test]
fn flight_chooses_space_on_board() {
    let catalog = french();
    let state = owning(
        arena(WEST_FOREST, &move_hand(), &sandbox_tokens()),
        &[Skill::Flight],
    );
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        line(&screen, Skill::Flight).options[0].command,
        Some(Command::ChooseSkillTarget(SkillChoice {
            skill: Skill::Flight,
            option: 0,
            card: None,
        }))
    );
    let screen = screen_of(&state, &choosing(Skill::Flight, 0), &catalog);
    let prompt = screen.mana_prompt.as_ref().expect("le bandeau");
    assert_eq!(prompt.text, "Vol : choisir la case d'arrivée");
    assert!(prompt.payments.is_empty());
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    let glade = screen
        .cast_destinations
        .iter()
        .find(|destination| destination.hex == GLADE)
        .expect("la clairière en évidence");
    assert_eq!(
        glade.command,
        Command::Play(index_of(
            &state,
            &target_skill(Skill::Flight, 0, Some(SkillTarget::Space(GLADE)))
        ))
    );
}

// C33 — Interface : Botte secrète, l'ennemi choisi dans le bandeau, comme la cible d'un sort.
#[test]
fn resistance_break_chooses_enemy() {
    let catalog = french();
    let guardsmen = token(arcanum_core::Enemy::AltemGuardsmen, 1);
    let state = owning(
        fight(VILLAGE, &move_hand(), &[(guardsmen, ORCS_2)], &[ORCS_2]),
        &[Skill::ResistanceBreak],
    );
    let screen = screen_of(&state, &choosing(Skill::ResistanceBreak, 0), &catalog);
    let prompt = screen.mana_prompt.expect("le bandeau");
    assert_eq!(prompt.text, "Botte secrète : choisir l'ennemi");
    let name = catalog.text(guardsmen.enemy().label_key());
    assert_eq!(
        prompt
            .payments
            .iter()
            .map(|button| (button.label.as_str(), button.command))
            .collect::<Vec<_>>(),
        [(
            name.as_str(),
            Command::Play(index_of(
                &state,
                &target_skill(
                    Skill::ResistanceBreak,
                    0,
                    Some(SkillTarget::Enemy(guardsmen))
                )
            ))
        )]
    );
}

// C41 — Interface : Surcharge de mana, un bouton par couleur.
#[test]
fn mana_overload_chooses_color() {
    let catalog = french();
    let state = owning(arena(VILLAGE, &move_hand(), &[]), &[Skill::ManaOverload]);
    let screen = screen_of(&state, &choosing(Skill::ManaOverload, 0), &catalog);
    let prompt = screen.mana_prompt.expect("le bandeau");
    assert_eq!(prompt.text, "Surcharge de mana : choisir la couleur");
    let labels: Vec<&str> = prompt
        .payments
        .iter()
        .map(|button| button.label.as_str())
        .collect();
    assert_eq!(labels[..4], ["rouge", "bleu", "blanc", "vert"]);
    let red = ManaColor::Basic(BasicColor::Red);
    assert_eq!(
        prompt.payments[0].command,
        Command::Play(index_of(
            &state,
            &target_skill(Skill::ManaOverload, 0, Some(SkillTarget::Color(red)))
        ))
    );
}

/// Le bac à sable de jour, les dés or, vert, rouge et bleu, Embrasure de la Source.
fn opening(hand: &[CardId]) -> GameState {
    owning(
        Sandbox {
            figure: Some(VILLAGE),
            hand: hand.to_vec(),
            dice: vec![
                ManaColor::Gold,
                ManaColor::Basic(BasicColor::Green),
                ManaColor::Basic(BasicColor::Red),
                ManaColor::Basic(BasicColor::Blue),
            ],
            ..Sandbox::default()
        }
        .state(),
        &[Skill::SourceOpening],
    )
}

// C39 — Interface : le dé à relancer se choisit dans la Source, ou « Ne rien relancer » ; au tour
// suivant, le dé en plus se prend dans la Source ; puis « Garder ce dé tel quel ».
#[test]
fn source_opening_chooses_die() {
    let catalog = french();
    let state = opening(&[march(), card(BasicAction::Stamina, 1)]);
    let screen = screen_of(&state, &choosing(Skill::SourceOpening, 0), &catalog);
    let prompt = screen.mana_prompt.as_ref().expect("le bandeau");
    assert_eq!(prompt.text, "Embrasure de la Source : choisir le dé");
    let none = target_skill(Skill::SourceOpening, 0, None);
    assert_eq!(
        prompt
            .payments
            .iter()
            .map(|button| (button.label.as_str(), button.command))
            .collect::<Vec<_>>(),
        [("Ne rien relancer", Command::Play(index_of(&state, &none)))]
    );
    let reroll_green = target_skill(Skill::SourceOpening, 0, Some(SkillTarget::Die(1)));
    let green = screen
        .source
        .dice
        .iter()
        .find(|die| die.color == ManaColor::Basic(BasicColor::Green))
        .expect("le dé vert");
    assert_eq!(
        green.pick,
        Some(Command::Play(index_of(&state, &reroll_green)))
    );

    let next = next_turn(done(&state, none));
    let screen = screen_of(&next, &ScreenMode::Normal, &catalog);
    let opening_line = line(&screen, Skill::SourceOpening);
    assert_eq!(
        opening_line.options[0].fallback_text(),
        "Avantage de Embrasure de la Source"
    );
    let screen = screen_of(&next, &choosing(Skill::SourceOpening, 0), &catalog);
    assert_eq!(
        screen
            .mana_prompt
            .as_ref()
            .map(|prompt| prompt.text.as_str()),
        Some("Avantage de Embrasure de la Source : choisir le dé")
    );
    let take_green = target_skill(Skill::SourceOpening, 0, Some(SkillTarget::Die(1)));
    let green = screen
        .source
        .dice
        .iter()
        .find(|die| die.color == ManaColor::Basic(BasicColor::Green))
        .expect("le dé vert");
    assert_eq!(
        green.pick,
        Some(Command::Play(index_of(&next, &take_green)))
    );
    let taken = done(&next, take_green);
    let keep = screen_of(&taken, &ScreenMode::Normal, &catalog)
        .source
        .keep
        .expect("« Garder ce dé tel quel »");
    assert_eq!(keep.label, "Garder ce dé tel quel");
    assert_eq!(
        keep.command,
        Command::Play(index_of(&taken, &Action::KeepSourceDie))
    );
}

// Interface : la session passe au choix de la cible, et « Annuler » le quitte.
#[test]
fn skill_target_choice_in_session() {
    let catalog = french();
    let mut events = arena_events(WEST_FOREST, &move_hand(), &sandbox_tokens(), &[]);
    events.push(Event::SkillGained {
        player: SOLO,
        skill: Skill::Flight,
    });
    let mut session = support::session_of(events);
    session.command(Command::ChooseSkillTarget(SkillChoice {
        skill: Skill::Flight,
        option: 0,
        card: None,
    }));
    assert!(!session.screen(&catalog).cast_destinations.is_empty());
    session.command(Command::CancelChoice);
    assert!(session.screen(&catalog).cast_destinations.is_empty());
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

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn crossbowmen() -> UnitCard {
    UnitCard::new(Unit::UtemCrossbowmen, 1)
}

fn herbalists() -> UnitCard {
    UnitCard::new(Unit::Herbalists, 1)
}

// C26 — Interface (Q6) : sur la capacité qui peut recevoir Charisme, un second bouton avec la
// valeur augmentée ; le premier garde la valeur imprimée.
#[test]
fn leadership_second_button_on_unit() {
    let catalog = french();
    let state = Units {
        owned: vec![peasants()],
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = owning(done(&state, Action::StartCombat), &[Skill::Leadership]);
    let state = to_block_phase(&state);
    let state = done(&state, declare_block(prowlers()));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let unit = screen
        .units
        .units
        .iter()
        .find(|line| line.card == peasants())
        .expect("les Paysans");
    let plain = Command::Play(index_of(&state, &activate(peasants(), 0, 1, false)));
    let led = Command::Play(index_of(&state, &activate(peasants(), 0, 1, true)));
    let buttons: Vec<(&str, Option<Command>)> = unit
        .effects
        .iter()
        .map(|button| (button.tooltip.as_str(), button.command))
        .collect();
    assert!(buttons.contains(&("Parade 2", Some(plain))), "{buttons:?}");
    assert!(
        buttons.contains(&("Avec Charisme : Parade 5", Some(led))),
        "{buttons:?}"
    );
}

/// À l'interaction du village, avec Loyauté sans faille et 1 Influence.
fn bonded(owned: &[UnitCard], offer: &[UnitCard]) -> GameState {
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
    let played = state.hand(SOLO)[0];
    apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: played,
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: arcanum_core::Effect::Influence(1),
        },
    )
}

// C29, C30 — Interface : l'emplacement de Loyauté dans la section Unités ; « Recruter sous
// Loyauté (coût) » quand le jeton est libre.
#[test]
fn bonds_of_loyalty_token_and_recruit() {
    let catalog = french();
    let state = bonded(&[], &[crossbowmen(), herbalists()]);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        screen.units.bonds_token.as_deref(),
        Some("Jeton de Loyauté : recrutement −5")
    );
    let recruit = |unit, bonds| Action::RecruitUnit {
        unit,
        disband: None,
        bonds,
    };
    let labels: Vec<(&str, Option<Command>)> = screen
        .recruit
        .iter()
        .map(|button| (button.label.as_str(), button.command))
        .collect();
    assert!(
        labels.contains(&(
            "Recruter sous Loyauté : Arbalétriers d'Utem (1)",
            Some(Command::Play(index_of(
                &state,
                &recruit(crossbowmen(), true)
            )))
        )),
        "{labels:?}"
    );
    assert!(
        labels.contains(&(
            "Recruter sous Loyauté : Herboristes (0)",
            Some(Command::Play(index_of(
                &state,
                &recruit(herbalists(), true)
            )))
        )),
        "{labels:?}"
    );
    let after = done(&state, recruit(crossbowmen(), true));
    let screen = screen_of(&after, &ScreenMode::Normal, &catalog);
    // Issue #26 : le jeton occupé reste affiché et nomme son unité.
    assert_eq!(
        screen.units.bonds_token.as_deref(),
        Some("Jeton de Loyauté : Arbalétriers d'Utem")
    );
    assert!(
        !screen
            .recruit
            .iter()
            .any(|button| button.label.starts_with("Recruter sous Loyauté"))
    );
}

// Issue #26 (texte de Loyauté sans faille) : l'unité sous le jeton de Loyauté ne prend pas de
// jeton de commandement ordinaire ; l'en-tête ne la compte pas, à cette manche comme aux suivantes.
#[test]
fn bonds_unit_not_counted_in_units_header() {
    let catalog = french();
    let state = bonded(&[], &[crossbowmen(), herbalists()]);
    let after = done(
        &state,
        Action::RecruitUnit {
            unit: crossbowmen(),
            disband: None,
            bonds: true,
        },
    );
    let screen = screen_of(&after, &ScreenMode::Normal, &catalog);
    assert_eq!(screen.units.header, "Unités : 0/1");
    assert_eq!(screen.units.units.len(), 1);
}

// Issue privée 341 : sans assez d'Influence pour le coût réduit, la ligne « Recruter sous Loyauté »
// reste visible, grisée, comme le recrutement ordinaire.
#[test]
fn bonds_recruit_greyed_without_influence() {
    let catalog = french();
    let state = owning(
        Units {
            offer: vec![crossbowmen(), herbalists()],
            ..Units::default()
        }
        .state(),
        &[Skill::BondsOfLoyalty],
    );
    let state = done(&state, Action::Interact);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let labels: Vec<(&str, Option<Command>)> = screen
        .recruit
        .iter()
        .map(|button| (button.label.as_str(), button.command))
        .collect();
    assert_eq!(
        labels,
        [
            ("Recruter sous Loyauté : Arbalétriers d'Utem (1)", None),
            ("Recruter : Arbalétriers d'Utem (6)", None),
            (
                "Recruter sous Loyauté : Herboristes (0)",
                Some(Command::Play(index_of(
                    &state,
                    &Action::RecruitUnit {
                        unit: herbalists(),
                        disband: None,
                        bonds: true,
                    }
                )))
            ),
            ("Recruter : Herboristes (3)", None),
        ]
    );
}

// Issue privée 302 : sans jeu de côté légal, le bouton grisé garde les valeurs possibles de la
// compétence, et deux options grisées ne se confondent pas.
#[test]
fn greyed_sideways_skill_keeps_its_values() {
    let catalog = french();
    let state = owning(
        arena(VILLAGE, &[wound(1)], &[]),
        &[Skill::IDontGiveADamn, Skill::WhoNeedsMagic],
    );
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let flexibility = line(&screen, Skill::IDontGiveADamn);
    assert_eq!(flexibility.options[0].fallback_text(), "De côté : 2 / 3");
    assert_eq!(flexibility.options[0].command, None);
    let who_needs_magic = line(&screen, Skill::WhoNeedsMagic);
    let faces: Vec<String> = who_needs_magic
        .options
        .iter()
        .map(|button| button.fallback_text().to_string())
        .collect();
    assert_eq!(faces, ["De côté : 2", "De côté : 3"]);
    assert!(who_needs_magic.options.iter().all(|b| b.command.is_none()));
}
