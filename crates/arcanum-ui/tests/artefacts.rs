//! Scénarios de la spec 055 côté affichage (artefacts) : boutons Jouer et Détruire, bannières
//! assignées, portées et utilisées, effets en cours, dés lancés, choix des artefacts, mana illimité
//! et fenêtre de fin de manche.

use arcanum_core::{
    Action, Artifact, ArtifactChoices, CardId, CardPlay, Effect, EnemyToken, Event, GameState, Hex,
    ManaColor, TimeOfDay, Unit, UnitCard, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    ArtifactPlay, Command, EffectButton, GameScreen, HandCard, ScreenMode, ScreenPhase, UnitLine,
    game_screen, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;

use Artifact::*;
use arcanum_core::BasicAction::{March, Promise, Rage};

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// Le décor des tests : le bac à sable (Arythea), la figurine, la main, les ennemis posés, les
/// unités recrutées, puis des événements de plus.
struct Decor {
    time: TimeOfDay,
    hand: Vec<CardId>,
    placed: Vec<(EnemyToken, Hex)>,
    owned: Vec<UnitCard>,
    extra: Vec<Event>,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            time: TimeOfDay::Day,
            hand: Vec::new(),
            placed: vec![(prowlers(), ORCS_2)],
            owned: Vec::new(),
            extra: Vec::new(),
        }
    }
}

impl Decor {
    fn events(&self) -> Vec<Event> {
        let mut events = Sandbox {
            time: self.time,
            figure: Some(PLAIN_W),
            hand: self.hand.clone(),
            placed: self.placed.clone(),
            ..Sandbox::default()
        }
        .events();
        events.extend(unit_events(&self.owned, &[]));
        events.extend(self.extra.iter().cloned());
        events
    }

    fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }

    /// Le combat engagé contre les ennemis des orques ② (phase à distance).
    fn fighting(&self) -> GameState {
        let state = done(&self.state(), challenge(ORCS_2));
        done(&state, Action::StartCombat)
    }
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

fn swordsmen() -> EnemyToken {
    EnemyToken::new(arcanum_core::Enemy::Swordsmen, 1)
}

fn exhaust(unit: UnitCard) -> Event {
    Event::UnitActivated {
        player: SOLO,
        unit,
        ability: 0,
        option: 0,
        effect: Effect::Nothing,
        enemy: None,
    }
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

fn held(screen: &GameScreen, card: CardId) -> HandCard {
    screen
        .hand
        .iter()
        .find(|shown| shown.card == card)
        .cloned()
        .expect("carte en main")
}

fn unit_line(screen: &GameScreen, unit: UnitCard) -> UnitLine {
    screen
        .units
        .units
        .iter()
        .find(|line| line.card == unit)
        .cloned()
        .expect("unité affichée")
}

/// L'action légale que joue `command`.
fn action_of(state: &GameState, command: Option<Command>) -> Option<Action> {
    match command {
        Some(Command::Play(index)) => legal_actions(state, SOLO).get(index).cloned(),
        _ => None,
    }
}

fn played(card: CardId, strong: bool, choices: ArtifactChoices) -> Action {
    Action::PlayCard {
        card,
        play: if strong {
            CardPlay::ThrowAway { option: 0, choices }
        } else {
            CardPlay::Artifact { option: 0, choices }
        },
    }
}

fn commands(buttons: &[EffectButton]) -> Vec<Option<Command>> {
    buttons.iter().map(|button| button.command).collect()
}

// --- Étape 14 : boutons, bannières, effets en cours ---

// Spec 055 C2 — MKUE p. 4, 3.c, 3.e : l'effet de base se joue sans mana, l'effet renforcé en
// détruisant la carte, ou de côté ; le texte de l'artefact est le sien, pas « pas encore codé ».
#[test]
fn artifact_in_hand_plays_basic_or_throw_away() {
    let catalog = french();
    let bag = artifact(EndlessBagOfGold);
    let state = Decor {
        hand: vec![bag, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let shown = held(&screen_of(&state, &catalog), bag);
    assert_eq!(
        action_of(
            &state,
            shown
                .basic_effects
                .first()
                .and_then(|button| button.command)
        ),
        Some(played(bag, false, ArtifactChoices::default()))
    );
    let strong = shown.strong_effects.first().expect("bouton renforcé");
    assert_eq!(
        action_of(&state, strong.command),
        Some(played(bag, true, ArtifactChoices::default()))
    );
    assert!(
        strong.tooltip.starts_with("Détruire : "),
        "{}",
        strong.tooltip
    );
    assert!(!shown.sideways.is_empty());
    assert!(
        !shown.effect.contains("pas encore codé"),
        "{}",
        shown.effect
    );
    assert!(shown.effect.contains("Influence 4"), "{}", shown.effect);
}

// Spec 055 C6 — MKUE p. 5, 8 : « Assigner à une unité », puis l'unité choisie d'un clic ; Échap
// annule.
#[test]
fn banner_assigned_by_clicking_a_unit() {
    let catalog = french();
    let glory = artifact(BannerOfGlory);
    let state = Decor {
        hand: vec![glory, card(March, 1)],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .state();
    let shown = held(&screen_of(&state, &catalog), glory);
    assert_eq!(
        commands(&shown.basic_effects),
        vec![Some(Command::AssignBanner(glory))]
    );
    let choosing = screen_in(&state, &ScreenMode::AssigningBanner(glory), &catalog);
    assert!(choosing.hand.iter().all(|shown| shown.card != glory));
    let prompt = choosing
        .disband_prompt
        .clone()
        .expect("bandeau du choix de l'unité");
    assert_eq!(
        prompt.text,
        "Bannière de gloire : à quelle unité l'assigner ?"
    );
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    let assign = unit_line(&choosing, peasants())
        .disband
        .expect("bouton Assigner");
    assert_eq!(assign.label, "Assigner");
    assert_eq!(
        action_of(&state, Some(assign.command)),
        Some(Action::AssignBanner {
            card: glory,
            unit: peasants()
        })
    );
}

// Spec 055 C22, C42 — la bannière portée par l'unité, son nom en infobulle, retournée après
// usage ; la Bannière de courage rend l'unité disponible hors combat.
#[test]
fn unit_shows_its_banner_and_courage() {
    let catalog = french();
    let courage = artifact(BannerOfCourage);
    let state = Decor {
        hand: vec![courage, card(March, 1)],
        owned: vec![peasants()],
        extra: vec![exhaust(peasants())],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        Action::AssignBanner {
            card: courage,
            unit: peasants(),
        },
    );
    let line = unit_line(&screen_of(&state, &catalog), peasants());
    let banner = line.banner.expect("bannière sous l'unité");
    assert_eq!(banner.name, "Bannière de courage");
    assert!(banner.tooltip.contains("Bannière : Bannière de courage"));
    assert!(!banner.flipped);
    let [ready] = line.banner_uses.as_slice() else {
        panic!("un bouton : {:?}", line.banner_uses);
    };
    assert_eq!(ready.label, "Rendre disponible (une fois par manche)");
    let action = action_of(&state, Some(ready.command)).expect("action légale");
    assert_eq!(
        action,
        Action::UseBanner {
            unit: peasants(),
            enemy: None
        }
    );
    let line = unit_line(&screen_of(&done(&state, action), &catalog), peasants());
    let banner = line.banner.expect("bannière sous l'unité");
    assert!(banner.flipped);
    assert!(
        banner
            .tooltip
            .contains("retournée jusqu'à la prochaine manche")
    );
    assert!(line.banner_uses.is_empty());
}

// Spec 055 C15 — Bannière de peur, phase de parade : un bouton par attaque à annuler, sur
// l'unité.
#[test]
fn banner_of_fear_button_on_unit() {
    let catalog = french();
    let fear = artifact(BannerOfFear);
    let decor = Decor {
        hand: vec![fear, card(March, 1)],
        owned: vec![peasants()],
        placed: vec![(swordsmen(), ORCS_2)],
        ..Decor::default()
    };
    let state = done(
        &decor.fighting(),
        Action::AssignBanner {
            card: fear,
            unit: peasants(),
        },
    );
    let state = to_block_phase(&state);
    let line = unit_line(&screen_of(&state, &catalog), peasants());
    let [cancel] = line.banner_uses.as_slice() else {
        panic!("un bouton : {:?}", line.banner_uses);
    };
    assert_eq!(cancel.label, "Annuler l'attaque : Soldats (Gloire +1)");
    assert_eq!(
        action_of(&state, Some(cancel.command)),
        Some(Action::UseBanner {
            unit: peasants(),
            enemy: Some(swordsmen())
        })
    );
}

// Spec 055 C42 — les effets en cours des artefacts : l'Anneau de rubis renforcé.
#[test]
fn artifact_modifiers_in_play_area() {
    let catalog = french();
    let ruby = artifact(RubyRing);
    let state = Decor {
        hand: vec![ruby, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, played(ruby, true, ArtifactChoices::default()));
    let lines = screen_of(&state, &catalog)
        .modifiers
        .expect("effets en cours")
        .lines;
    assert!(
        lines
            .iter()
            .any(|line| line.contains("mana rouge et noir illimité ; Gloire +1 par sort rouge")),
        "{lines:?}"
    );
}

// Spec 055, Interface : les dés lancés par un artefact dans les faits du tour, en une ligne.
#[test]
fn rolled_dice_shown_this_turn() {
    let catalog = french();
    let horn = artifact(HornOfWrath);
    let state = Decor {
        hand: vec![horn, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let rolled = |state: GameState, color: ManaColor| {
        apply(
            state,
            &Event::ArtifactDieRolled {
                player: SOLO,
                card: horn,
                color,
            },
        )
    };
    let state = rolled(state, ManaColor::Basic(arcanum_core::BasicColor::Blue));
    // Ce que donne un dé (gloire du noir de la Poche de mana) s'intercale sans couper la ligne.
    let state = apply(
        state,
        &Event::FameGained {
            player: SOLO,
            fame: 1,
        },
    );
    let state = rolled(state, ManaColor::Black);
    let facts = screen_of(&state, &catalog).turn_facts;
    assert!(
        facts.contains(&"Dés lancés : bleu, noir".to_string()),
        "{facts:?}"
    );
}

// --- Étape 15 : choix des artefacts, mana illimité, fin de manche ---

fn artifact_mode(card: CardId, strong: bool, picks: ArtifactChoices) -> ScreenMode {
    ScreenMode::PlayingArtifact(ArtifactPlay {
        card,
        strong,
        option: 0,
        picks,
    })
}

/// Les boutons du bandeau du choix en cours (artefact, mana).
fn prompt_buttons(screen: &GameScreen) -> Vec<(String, Command)> {
    screen
        .mana_prompt
        .as_ref()
        .expect("bandeau du choix")
        .payments
        .iter()
        .map(|button| (button.label.clone(), button.command))
        .collect()
}

// Spec 055 C27 — Épée de justice de base : les cartes à défausser se cochent dans la main, pas une
// Blessure ; le bandeau joue l'Attaque des cartes cochées.
#[test]
fn sword_of_justice_ticks_cards_to_discard() {
    let catalog = french();
    let sword = artifact(SwordOfJustice);
    let (march, promise) = (card(March, 1), card(Promise, 1));
    let state = Decor {
        hand: vec![sword, march, promise, wound(101)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    let shown = held(&screen_of(&state, &catalog), sword);
    let none = ArtifactChoices::default();
    assert_eq!(
        commands(&shown.basic_effects),
        vec![Some(Command::ChooseArtifact(ArtifactPlay {
            card: sword,
            strong: false,
            option: 0,
            picks: none,
        }))]
    );
    let screen = screen_in(&state, &artifact_mode(sword, false, none), &catalog);
    assert!(screen.hand.iter().all(|shown| shown.card != sword));
    assert!(held(&screen, wound(101)).discard_toggle.is_none());
    let toggle = held(&screen, march).discard_toggle.expect("case à cocher");
    assert!(!toggle.checked);
    let one = ArtifactChoices {
        discard: arcanum_core::Picks::of(&[march]),
        ..none
    };
    assert_eq!(
        toggle.command,
        Command::ChooseArtifact(ArtifactPlay {
            card: sword,
            strong: false,
            option: 0,
            picks: one,
        })
    );
    let two = ArtifactChoices {
        discard: arcanum_core::Picks::of(&[march, promise]),
        ..none
    };
    let screen = screen_in(&state, &artifact_mode(sword, false, two), &catalog);
    assert_eq!(
        screen
            .mana_prompt
            .as_ref()
            .map(|prompt| prompt.text.as_str()),
        Some("Épée de justice : cartes à défausser (Attaque 3 chacune)")
    );
    assert!(held(&screen, march).discard_toggle.expect("cochée").checked);
    let [(label, command)] = prompt_buttons(&screen).try_into().expect("un bouton");
    assert_eq!(label, "Jouer : Attaque 6");
    assert_eq!(
        action_of(&state, Some(command)),
        Some(played(sword, false, two))
    );
}

// Spec 055 C32 — Corne de la colère renforcée : un bouton par bonus, de +0 à +5.
#[test]
fn horn_of_wrath_strong_bonus_buttons() {
    let catalog = french();
    let horn = artifact(HornOfWrath);
    let state = Decor {
        hand: vec![horn, card(March, 1)],
        placed: vec![(diggers(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, declare(&[diggers()]));
    let none = ArtifactChoices::default();
    let shown = held(&screen_of(&state, &catalog), horn);
    assert_eq!(
        commands(&shown.strong_effects),
        vec![Some(Command::ChooseArtifact(ArtifactPlay {
            card: horn,
            strong: true,
            option: 0,
            picks: none,
        }))]
    );
    let screen = screen_in(&state, &artifact_mode(horn, true, none), &catalog);
    assert_eq!(
        screen
            .mana_prompt
            .as_ref()
            .map(|prompt| prompt.text.as_str()),
        Some("Corne de la colère : ajouter combien ? (un dé par point)")
    );
    let buttons = prompt_buttons(&screen);
    let labels: Vec<&str> = buttons.iter().map(|(label, _)| label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "+0 : Attaque de siège 5",
            "+1 : Attaque de siège 6",
            "+2 : Attaque de siège 7",
            "+3 : Attaque de siège 8",
            "+4 : Attaque de siège 9",
            "+5 : Attaque de siège 10"
        ]
    );
    let three = ArtifactChoices { bonus: 3, ..none };
    assert_eq!(
        action_of(&state, Some(buttons[3].1)),
        Some(played(horn, true, three))
    );
}

// Spec 055 C18 — Bannière de peur renforcée : les ennemis se cochent sur l'écran de combat,
// trois au plus.
#[test]
fn banner_of_fear_strong_ticks_enemies() {
    let catalog = french();
    let fear = artifact(BannerOfFear);
    let decor = Decor {
        hand: vec![fear, card(March, 1)],
        placed: vec![
            (prowlers(), ORCS_2),
            (swordsmen(), ORCS_2),
            (wolf_riders(), ORCS_2),
            (diggers(), ORCS_2),
        ],
        ..Decor::default()
    };
    let mut session = support::session_of(decor.events());
    support::play(&mut session, &challenge(ORCS_2));
    support::play(&mut session, &Action::StartCombat);
    let none = ArtifactChoices::default();
    session.command(Command::ChooseArtifact(ArtifactPlay {
        card: fear,
        strong: true,
        option: 0,
        picks: none,
    }));
    let combat = session.combat_screen(&catalog).expect("combat");
    let toggles: Vec<_> = combat
        .enemies
        .iter()
        .filter_map(|enemy| enemy.target.clone())
        .collect();
    assert_eq!(toggles.len(), 4);
    for enemy in [prowlers(), swordsmen(), diggers()] {
        let toggle = session
            .combat_screen(&catalog)
            .expect("combat")
            .enemies
            .iter()
            .find(|shown| shown.token == enemy)
            .and_then(|shown| shown.target.clone())
            .expect("case cible");
        session.command(toggle.command);
    }
    let three = ArtifactChoices {
        enemies: arcanum_core::Picks::of(&[prowlers(), swordsmen(), diggers()]),
        ..none
    };
    let screen = session.screen(&catalog);
    let [(_, command)] = prompt_buttons(&screen).try_into().expect("un bouton");
    let legal = session.legal_actions();
    let Command::Play(index) = command else {
        panic!("{command:?}");
    };
    let Some(Action::PlayCard {
        play: CardPlay::ThrowAway { choices, .. },
        ..
    }) = legal.get(index)
    else {
        panic!("{:?}", legal.get(index));
    };
    assert!(
        choices.enemies.len() == 3 && three.enemies.iter().all(|e| choices.enemies.contains(e))
    );
}

// Spec 055 C40, C41 — Amulette des ténèbres : la couleur de chaque jeton, un bouton par couleur ;
// renforcée, trois couleurs l'une après l'autre.
#[test]
fn amulet_of_darkness_colors() {
    let catalog = french();
    let darkness = artifact(AmuletOfDarkness);
    let state = Decor {
        hand: vec![darkness, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let none = ArtifactChoices::default();
    let screen = screen_in(&state, &artifact_mode(darkness, false, none), &catalog);
    let buttons = prompt_buttons(&screen);
    assert_eq!(buttons.len(), 6);
    let red = buttons
        .iter()
        .find(|(label, _)| label == "Jeton rouge")
        .expect("jeton rouge");
    let reds = |count: usize| ArtifactChoices {
        colors: arcanum_core::Picks::of(&vec![
            ManaColor::Basic(arcanum_core::BasicColor::Red);
            count
        ]),
        ..none
    };
    assert_eq!(
        action_of(&state, Some(red.1)),
        Some(played(darkness, false, reds(1)))
    );
    let screen = screen_in(&state, &artifact_mode(darkness, true, none), &catalog);
    let red = prompt_buttons(&screen)
        .into_iter()
        .find(|(label, _)| label == "Jeton rouge")
        .expect("jeton rouge");
    assert_eq!(
        red.1,
        Command::ChooseArtifact(ArtifactPlay {
            card: darkness,
            strong: true,
            option: 0,
            picks: reds(1),
        })
    );
    let screen = screen_in(&state, &artifact_mode(darkness, true, reds(2)), &catalog);
    let red = prompt_buttons(&screen)
        .into_iter()
        .find(|(label, _)| label == "Jeton rouge")
        .expect("jeton rouge");
    assert_eq!(
        action_of(&state, Some(red.1)),
        Some(played(darkness, true, reds(3)))
    );
}

// Spec 055 C33 — Poche de mana : le dé or attend la couleur du cristal, dans la fenêtre du dé or de
// récompense (spec 044).
#[test]
fn gem_pouch_gold_die_chooses_crystal() {
    let catalog = french();
    let pouch = artifact(EndlessGemPouch);
    let state = Decor {
        hand: vec![pouch, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let state = apply(
        state,
        &Event::ArtifactDieRolled {
            player: SOLO,
            card: pouch,
            color: ManaColor::Gold,
        },
    );
    let window = screen_of(&state, &catalog)
        .crystal_choice
        .expect("choix du cristal");
    assert!(window.colors.iter().all(|button| button.command.is_some()));
}

// Spec 055 C37 — Livre de sagesse : la carte à détruire comme pour Entraînement, puis la carte de
// l'offre.
#[test]
fn book_of_wisdom_destroys_then_takes() {
    let catalog = french();
    let book = artifact(BookOfWisdom);
    let blood_rage = advanced(arcanum_core::AdvancedAction::BloodRage);
    let rage = card(Rage, 1);
    let state = Decor {
        hand: vec![book, rage, card(March, 1)],
        extra: advanced_offer(&[blood_rage]),
        ..Decor::default()
    }
    .state();
    let shown = held(&screen_of(&state, &catalog), book);
    let choice = arcanum_ui::GainChoice {
        card: book,
        option: 0,
        strong: false,
        discard: None,
        gain: None,
    };
    assert_eq!(
        commands(&shown.basic_effects),
        vec![Some(Command::ChooseGain(choice))]
    );
    let screen = screen_in(&state, &ScreenMode::ChoosingGain(choice), &catalog);
    let destroy = held(&screen, rage);
    let [button] = destroy.plays.as_slice() else {
        panic!("{:?}", destroy.plays);
    };
    assert_eq!(button.label, "Détruire");
    assert!(held(&screen, card(March, 1)).plays.is_empty());
}

// Spec #20, C16 — spec 055 : la carte écartée par le Livre de sagesse choisie, la fenêtre de choix
// ne montre que les actions avancées de sa couleur.
#[test]
fn book_of_wisdom_gain_in_choice_window() {
    let catalog = french();
    let book = artifact(BookOfWisdom);
    let blood_rage = advanced(arcanum_core::AdvancedAction::BloodRage);
    let ice_bolt = advanced(arcanum_core::AdvancedAction::IceBolt);
    let rage = card(Rage, 1);
    let state = Decor {
        hand: vec![book, rage, card(March, 1)],
        extra: advanced_offer(&[ice_bolt, blood_rage]),
        ..Decor::default()
    }
    .state();
    let choice = arcanum_ui::GainChoice {
        card: book,
        option: 0,
        strong: false,
        discard: Some(rage),
        gain: None,
    };
    let window = screen_in(&state, &ScreenMode::ChoosingGain(choice), &catalog)
        .gain_choice
        .expect("fenêtre de choix de la carte");
    let names: Vec<&str> = window.cards.iter().map(|card| card.name.as_str()).collect();
    assert_eq!(names, [catalog.text(blood_rage.card().label_key())]);
    assert!(window.cards[0].take.is_some());
}

// Spec 055 C25 — Anneau de rubis renforcé : « illimité (rouge) » dans le choix du mana.
#[test]
fn endless_mana_in_mana_choice() {
    let catalog = french();
    let ruby = artifact(RubyRing);
    let rage = card(Rage, 1);
    let state = Decor {
        hand: vec![ruby, rage, card(March, 1)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    let state = done(&state, played(ruby, true, ArtifactChoices::default()));
    let mode = ScreenMode::ChoosingMana(arcanum_ui::ManaChoice {
        card: rage,
        option: 0,
        discard: None,
    });
    let buttons = prompt_buttons(&screen_in(&state, &mode, &catalog));
    let endless = buttons
        .iter()
        .find(|(label, _)| label == "illimité (rouge)")
        .unwrap_or_else(|| panic!("{buttons:?}"));
    assert!(matches!(
        action_of(&state, Some(endless.1)),
        Some(Action::PlayCard {
            play: CardPlay::Strong {
                mana: arcanum_core::ManaPayment::Endless(_),
                ..
            },
            ..
        })
    ));
}

// Spec 055 C9 — fin de manche : une ligne par bannière assignée, « Garder » ou « Reprendre », puis
// « Valider ».
#[test]
fn round_end_window_reclaims_banners() {
    let catalog = french();
    let courage = artifact(BannerOfCourage);
    let glory = artifact(BannerOfGlory);
    let guardsmen = UnitCard::new(Unit::UtemGuardsmen, 1);
    let hand = vec![card(March, 1), courage, glory];
    let mut extra = vec![Event::DeckShuffled {
        player: SOLO,
        order: hand.clone(),
    }];
    extra.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    let state = Decor {
        hand: hand.clone(),
        owned: vec![peasants(), guardsmen],
        extra,
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        Action::AssignBanner {
            card: courage,
            unit: peasants(),
        },
    );
    let state = done(
        &state,
        Action::AssignBanner {
            card: glory,
            unit: guardsmen,
        },
    );
    let state = done(&state, Action::AnnounceEndOfRound);
    let window = screen_of(&state, &catalog)
        .banner_reclaim
        .expect("fenêtre de fin de manche");
    assert_eq!(
        window.title,
        "Fin de manche : bannières à reprendre dans le deck"
    );
    let texts: Vec<&str> = window
        .banners
        .iter()
        .map(|line| line.text.as_str())
        .collect();
    assert_eq!(
        texts,
        [
            "Bannière de courage (Paysans)",
            "Bannière de gloire (Gardes d'Utem)"
        ]
    );
    assert!(window.banners.iter().all(|line| !line.taken));
    let take = &window.banners[0].take;
    assert_eq!(take.label, "Reprendre");
    let one = arcanum_core::Picks::of(&[courage]);
    assert_eq!(take.command, Command::ChooseReclaimed(one));
    assert_eq!(
        action_of(&state, window.confirm.map(|button| button.command)),
        Some(Action::ReclaimBanners {
            banners: Vec::new()
        })
    );
    let window = screen_in(&state, &ScreenMode::ReclaimingBanners(one), &catalog)
        .banner_reclaim
        .expect("fenêtre de fin de manche");
    assert!(window.banners[0].taken && !window.banners[1].taken);
    assert_eq!(
        window.banners[0].keep.command,
        Command::ChooseReclaimed(arcanum_core::Picks::default())
    );
    assert_eq!(
        action_of(&state, window.confirm.map(|button| button.command)),
        Some(Action::ReclaimBanners {
            banners: vec![courage]
        })
    );
}
