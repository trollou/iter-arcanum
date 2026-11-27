//! Scénarios de la spec 053 côté affichage (actions avancées : combat, unités et soins) : ennemis à
//! l'armure réduite ou qui n'attaquent pas, choix de l'ennemi d'une capacité d'unité, options à
//! second mana, Régénération, effets en cours nouveaux.

use arcanum_core::{
    Action, AdvancedAction, BasicColor, CardId, DeedCard, Effect, EnemyToken, Event, GameState,
    ManaColor, ManaPayment, Spell, UndoStatus, Unit, UnitCard, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CombatChoice, CombatScreen, Command, EffectFace, GameScreen, ManaChoice, ScreenMode,
    ScreenPhase, ValuePiece, combat_screen, game_screen, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use AdvancedAction::*;
use arcanum_core::BasicAction::Rage;

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// Un décor de la spec 053 : le bac à sable (Arythea) de jour, sur la plaine à l'O du village, la
/// main, les ennemis posés aux orques ②, les unités recrutées, puis des événements de plus.
#[derive(Clone, Default)]
struct Decor {
    hand: Vec<CardId>,
    placed: Vec<EnemyToken>,
    owned: Vec<UnitCard>,
    extra: Vec<Event>,
}

impl Decor {
    fn state(&self) -> GameState {
        let mut events = Sandbox {
            figure: Some(PLAIN_W),
            hand: self.hand.clone(),
            placed: self.placed.iter().map(|&enemy| (enemy, ORCS_2)).collect(),
            ..Sandbox::default()
        }
        .events();
        events.extend(unit_events(&self.owned, &[]));
        events.extend(self.extra.iter().cloned());
        events.iter().fold(GameState::default(), apply)
    }

    /// Le décor, puis le combat engagé contre les ennemis des orques ② (phase à distance).
    fn fighting(&self) -> GameState {
        let state = done(&self.state(), challenge(ORCS_2));
        done(&state, Action::StartCombat)
    }
}

fn illusionists() -> UnitCard {
    UnitCard::new(Unit::Illusionists, 1)
}

fn freezers() -> UnitCard {
    UnitCard::new(Unit::AmotepFreezers, 1)
}

fn activate(of: UnitCard, ability: u8, mana: ManaPayment, enemy: EnemyToken) -> Action {
    Action::ActivateUnit {
        unit: of,
        ability,
        option: 0,
        mana: vec![mana],
        target: None,
        enemy: Some(enemy),
        leadership: false,
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

fn combat_of(state: &GameState, catalog: &Catalog) -> CombatScreen {
    let legal = legal_actions(state, SOLO);
    combat_screen(
        &view(state, SOLO),
        &legal,
        &CombatChoice::default(),
        UndoStatus::Possible,
        catalog,
    )
    .expect("combat engagé")
}

fn enemy_name(token: EnemyToken, catalog: &Catalog) -> String {
    catalog.text(token.enemy().label_key())
}

/// La valeur de l'icône Armure d'une carte d'ennemi (la première valeur).
fn shown_armor(combat: &CombatScreen, token: EnemyToken) -> String {
    let enemy = combat
        .enemies
        .iter()
        .find(|enemy| enemy.token == token)
        .expect("ennemi du combat");
    match &enemy.values[0] {
        ValuePiece::Icon(icon) => icon.value.clone(),
        ValuePiece::Text(text) | ValuePiece::Colored { text, .. } => text.clone(),
    }
}

fn status_of(combat: &CombatScreen, token: EnemyToken) -> String {
    combat
        .enemies
        .iter()
        .find(|enemy| enemy.token == token)
        .and_then(|enemy| enemy.status.clone())
        .unwrap_or_default()
}

// --- Écran de combat (étape 11) ---

// Spec 053 C37, C12
#[test]
fn reduced_armor_shown_on_enemy_and_needed() {
    // Carte Bouclier de glace ; MKUE p. 9, *Attack Phase* 3.
    let catalog = french();
    let ice_shield = advanced(IceShield);
    let state = Decor {
        hand: vec![ice_shield, card(Rage, 1)],
        placed: vec![prowlers()],
        ..Decor::default()
    }
    .fighting();
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let state = done(&state, strong(ice_shield, BLUE_DIE));
    let combat = combat_of(&state, &catalog);
    assert_eq!(shown_armor(&combat, prowlers()), "1");
    assert!(
        status_of(&combat, prowlers()).contains("Armure 1 (au lieu de 3)"),
        "{:?}",
        status_of(&combat, prowlers())
    );
    // En phase d'attaque, ce qu'il faut atteindre est l'armure effective.
    let state = done(&state, Action::ResolveBlock);
    let state = skip_to_attack_phase(&state);
    let state = done(&state, declare(&[prowlers()]));
    let combat = combat_of(&state, &catalog);
    let needed = combat
        .in_progress
        .and_then(|panel| panel.needed)
        .unwrap_or_default();
    assert!(needed.ends_with(": 1"), "{needed:?}");
}

// Spec 053 C7
#[test]
fn enemy_that_does_not_attack_has_no_block_nor_stakes() {
    // Compendium, *Illusionists* Q1.
    let catalog = french();
    let state = Decor {
        placed: vec![prowlers()],
        owned: vec![illusionists()],
        ..Decor::default()
    }
    .fighting();
    let state = done(
        &state,
        activate(illusionists(), 1, ManaPayment::Die(WHITE_DIE), prowlers()),
    );
    let combat = combat_of(&to_block_phase(&state), &catalog);
    let enemy = &combat.enemies[0];
    assert!(enemy.block.is_none());
    assert!(enemy.stakes.is_empty());
    assert!(
        status_of(&combat, prowlers()).contains("n'attaque pas"),
        "{:?}",
        enemy.status
    );
}

// Spec 053 C30
#[test]
fn unit_ability_offers_each_enemy_target() {
    // Carte Illusionnistes ; Compendium, *Illusionists* S2.
    let catalog = french();
    let state = Decor {
        placed: vec![prowlers(), orc_summoners(), diggers()],
        owned: vec![illusionists()],
        ..Decor::default()
    }
    .fighting();
    let screen = screen_of(&state, &catalog);
    let ability = &screen.units.units[0].abilities[1];
    assert_eq!(
        ability.command,
        Some(Command::ChooseUnitAbility {
            unit: illusionists(),
            ability: 1,
            option: None,
        })
    );
    let mode = ScreenMode::ChoosingUnitAbility {
        unit: illusionists(),
        ability: 1,
        option: None,
    };
    let prompt = screen_in(&state, &mode, &catalog)
        .unit_prompt
        .expect("choix de la cible");
    let labels: Vec<&str> = prompt
        .payments
        .iter()
        .map(|button| button.label.as_str())
        .collect();
    // Les Excavateurs, fortifiés, ne sont pas des cibles.
    assert_eq!(labels.len(), 2, "{labels:?}");
    assert!(
        labels[0].contains(&enemy_name(prowlers(), &catalog)),
        "{labels:?}"
    );
    assert!(
        labels[1].contains(&enemy_name(orc_summoners(), &catalog)),
        "{labels:?}"
    );
    assert!(
        !labels
            .iter()
            .any(|label| label.contains(&enemy_name(diggers(), &catalog)))
    );
    // Les Invocateurs qui n'attaquent plus n'invoquent rien : plus d'avertissement (C30).
    assert!(combat_of(&state, &catalog).summon_warning.is_some());
    let state = done(
        &state,
        activate(
            illusionists(),
            1,
            ManaPayment::Die(WHITE_DIE),
            orc_summoners(),
        ),
    );
    let warning = combat_of(&state, &catalog).summon_warning;
    assert!(warning.is_none(), "{warning:?}");
}

// Spec 053 C31
#[test]
fn freezers_ability_named_and_armor_shown() {
    // Carte Glaceurs d'Amotep ; Q7.
    let catalog = french();
    let state = Decor {
        placed: vec![prowlers()],
        owned: vec![freezers()],
        ..Decor::default()
    }
    .fighting();
    let screen = screen_of(&state, &catalog);
    let effect = &screen.units.units[0].effects[2];
    // Spec #11 C13 : sans « (mana bleu) » devant, que dit la pastille.
    let written = catalog.text(&Unit::AmotepFreezers.ability_key(1));
    let (_, without_mana) = written.split_once(") ").expect("« (mana bleu) » en tête");
    assert_eq!(effect.face, EffectFace::Text(without_mana.into()));
    let state = skip_to_attack_phase(&state);
    let state = done(
        &state,
        activate(freezers(), 1, ManaPayment::Die(BLUE_DIE), prowlers()),
    );
    let combat = combat_of(&state, &catalog);
    assert_eq!(shown_armor(&combat, prowlers()), "1");
    assert!(!status_of(&combat, prowlers()).contains("n'attaque pas"));
}

// --- Options, Régénération, effets en cours (étape 12) ---

fn crystal(color: BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

/// Les Paysans recrutés et épuisés (activés ce tour).
fn exhausted_peasants() -> Vec<Event> {
    vec![Event::UnitActivated {
        player: SOLO,
        unit: peasants(),
        ability: 0,
        option: 0,
        effect: Effect::Nothing,
        enemy: None,
    }]
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

// Spec 053 C22
#[test]
fn pure_magic_offers_one_paid_option_per_color() {
    // Compendium, *Pure Magic* S2.
    let catalog = french();
    let pure_magic = advanced(PureMagic);
    let state = Decor {
        hand: vec![pure_magic],
        ..Decor::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    let paid = &screen.hand[0].paid_plays;
    let colors: Vec<BasicColor> = paid.iter().map(|button| button.color).collect();
    assert_eq!(
        colors,
        vec![
            BasicColor::Green,
            BasicColor::White,
            BasicColor::Blue,
            BasicColor::Red
        ]
    );
    assert!(paid[0].enabled, "Mouvement 4 contre un mana vert");
    assert!(!paid[2].enabled, "Parade 4 hors combat");
}

// Spec 073 (pastille d'un bouton renforcé) ; issue privée 312
#[test]
fn strong_pure_magic_buttons_show_the_option_mana() {
    // Carte : renforcé (mana bleu), puis un mana de la couleur de l'option ; le bleu est sur le
    // titre « Renforcé », chaque bouton porte la couleur de son option.
    let catalog = french();
    let pure_magic = advanced(PureMagic);
    let state = Decor {
        hand: vec![pure_magic],
        ..Decor::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    let colors: Vec<Option<ManaColor>> = screen.hand[0]
        .strong_effects
        .iter()
        .map(|button| button.color)
        .collect();
    let basic = |color| Some(ManaColor::Basic(color));
    assert_eq!(
        colors,
        vec![
            basic(BasicColor::Green),
            basic(BasicColor::White),
            basic(BasicColor::Blue),
            basic(BasicColor::Red),
        ]
    );
    // Spec #11 C11 (Q2) : les options payantes de l'effet de base sont dans le bloc de base.
    let basic_colors: Vec<Option<ManaColor>> = screen.hand[0]
        .basic_effects
        .iter()
        .map(|button| button.color)
        .collect();
    assert_eq!(basic_colors, colors);
}

// Spec 053 C23
#[test]
fn strong_pure_magic_chooses_both_mana() {
    // Compendium, *Pure Magic* S2 ; spec 022 (fenêtre du choix du mana).
    let catalog = french();
    let pure_magic = advanced(PureMagic);
    let state = Decor {
        hand: vec![pure_magic],
        placed: vec![prowlers()],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let screen = screen_of(&state, &catalog);
    let block = screen.hand[0]
        .strong_plays
        .iter()
        .find(|button| button.label.contains("Parade 7"))
        .expect("Renforcer : Parade 7");
    assert!(block.enabled, "{block:?}");
    assert_eq!(
        block.command,
        Command::ChooseMana {
            card: pure_magic,
            option: 2,
            discard: None,
        }
    );
    let mode = ScreenMode::ChoosingMana(ManaChoice {
        card: pure_magic,
        option: 2,
        discard: None,
    });
    let prompt = screen_in(&state, &mode, &catalog)
        .mana_prompt
        .expect("choix du mana");
    let labels: Vec<&str> = prompt
        .payments
        .iter()
        .map(|button| button.label.as_str())
        .collect();
    assert!(
        labels
            .iter()
            .any(|label| label.starts_with("Dé bleu · Cristal bleu")),
        "{labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label.starts_with("Cristal bleu") && label.ends_with("Dé bleu")),
        "{labels:?}"
    );
}

// Spec 053 C17
#[test]
fn regeneration_chooses_the_unit() {
    // Carte Régénération ; spec 042 (choix de l'unité comme Rajeunissement).
    let catalog = french();
    let regeneration = advanced(Regeneration);
    let state = Decor {
        hand: vec![regeneration],
        owned: vec![peasants()],
        extra: exhausted_peasants(),
        ..Decor::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    let card = &screen.hand[0];
    assert_eq!(card.effect, catalog.text("card.regeneration.basic_text"));
    let labels: Vec<&str> = card
        .unit_plays
        .iter()
        .map(|button| button.label.as_str())
        .collect();
    assert!(
        labels.contains(
            &"Jouer : Soin 1 ; rendre disponible une unité de niveau 2 au plus · Paysans"
        ),
        "{labels:?}"
    );
}

// Spec 053 C9, C27
#[test]
fn option_buttons_name_the_new_effects() {
    // Cartes Coup de sang et Au cœur du combat.
    let catalog = french();
    let blood_rage = advanced(BloodRage);
    let into_the_heat = advanced(IntoTheHeat);
    let state = Decor {
        hand: vec![blood_rage, into_the_heat],
        placed: vec![prowlers()],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .fighting();
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        screen.hand[0].basic_effects[1].face,
        EffectFace::Text("Attaque 5 et une Blessure".into())
    );
    assert_eq!(
        screen.hand[1].basic_effects[0].face,
        EffectFace::Text(catalog.text("card.into_the_heat.basic_label"))
    );
    assert_eq!(
        screen.hand[1].effect,
        catalog.text("card.into_the_heat.basic_text")
    );
    let written = format!("Jouer : {}", catalog.text("card.into_the_heat.basic_label"));
    assert!(
        screen.hand[1]
            .plays
            .iter()
            .any(|button| button.label == written),
        "{:?}",
        screen.hand[1].plays
    );
    let strong = format!(
        "Renforcer : {}",
        catalog.text("card.into_the_heat.strong_label")
    );
    assert!(
        screen.hand[1]
            .strong_plays
            .iter()
            .any(|button| button.label == strong),
        "{:?}",
        screen.hand[1].strong_plays
    );
}

// Spec 053 C27, C29
#[test]
fn into_the_heat_modifiers_and_damage_to_hero_only() {
    // Carte ; MKUE p. 9, *Assign Damage Phase* 3 et 5.
    let catalog = french();
    let into_the_heat = advanced(IntoTheHeat);
    let state = Decor {
        hand: vec![into_the_heat],
        placed: vec![prowlers()],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, basic(into_the_heat));
    // La zone de jeu nomme la carte jouée par son texte écrit (vérification à l'œil).
    let played = &screen_of(&state, &catalog).play_area[0].text;
    assert!(
        played.ends_with(&catalog.text("card.into_the_heat.basic_label")),
        "{played:?}"
    );
    let lines = screen_of(&state, &catalog)
        .modifiers
        .expect("effets en cours")
        .lines;
    assert_eq!(
        lines,
        vec![
            "Au cœur du combat : unités : Attaque et Parade +2".to_string(),
            "Au cœur du combat : pas de dégâts sur les unités".to_string(),
        ]
    );
    let state = done(&to_block_phase(&state), Action::NextCombatPhase);
    // Les dégâts vont au héros d'office (arcanum-core, C29) : aucun bouton vers une unité.
    let damage = combat_of(&state, &catalog).damage;
    assert!(damage.is_empty(), "{damage:?}");
}

// Spec 053 C32, C33
#[test]
fn altem_mages_black_options_named_by_their_modifiers() {
    // Carte Mages d'Altem ; Compendium, *Altem Mages* Q2, S7.
    let catalog = french();
    let state = Decor {
        owned: vec![UnitCard::new(Unit::AltemMages, 1)],
        ..Decor::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    // Les deux options de la capacité noire, dernières de la carte.
    let effects = &screen.units.units[0].effects;
    let effects = &effects[effects.len() - 2..];
    assert_eq!(
        effects[0].face,
        EffectFace::Text("attaques de feu glacé".into())
    );
    assert_eq!(
        effects[1].face,
        EffectFace::Text("attaques de siège en plus".into())
    );
}

// Issue privée 276 : une carte dont tous les effets sont codés n'a pas de texte écrit, et l'afficher en
// main ne doit pas réclamer de clé manquante (Boule de glace, Boule vivifiante…), sort compris.
#[test]
fn hand_of_coded_cards_asks_for_no_missing_label() {
    for language in ["fr", "en"] {
        let catalog = Catalog::load(
            &repository_assets_dir().join("i18n"),
            &Language(language.into()),
        )
        .expect("catalogue du dépôt");
        // Par mains de quatre : une grosse main multiplie les fins de tour légales (issue #3).
        let cards: Vec<CardId> = AdvancedAction::ALL
            .into_iter()
            .map(advanced)
            .chain(
                Spell::ALL
                    .into_iter()
                    .map(|spell| CardId::new(DeedCard::Spell(spell), 1)),
            )
            .collect();
        for hand in cards.chunks(4) {
            let decor = Decor {
                hand: hand.to_vec(),
                placed: vec![prowlers()],
                ..Decor::default()
            };
            screen_of(&decor.state(), &catalog);
            let fighting = decor.fighting();
            screen_of(&fighting, &catalog);
            combat_of(&fighting, &catalog);
        }
        assert_eq!(catalog.gaps(), vec![], "langue {language}");
    }
}

// Spec #68 C3 : une option sans effet propre a pour bouton son libellé court, et pour « i » le
// texte complet (Au cœur du combat, jouée au début d'un combat).
#[test]
fn option_button_reads_label_and_info_reads_full_text() {
    let catalog = french();
    let into_the_heat = advanced(IntoTheHeat);
    let state = Decor {
        hand: vec![into_the_heat, card(Rage, 1)],
        placed: vec![prowlers()],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .fighting();
    let screen = screen_of(&state, &catalog);
    let held = &screen.hand[0];
    assert_eq!(
        held.basic_effects[0].face,
        EffectFace::Text(catalog.text("card.into_the_heat.basic_label"))
    );
    assert!(
        !catalog
            .text("card.into_the_heat.basic_label")
            .starts_with('⟦')
    );
    let info = held.info.as_ref().expect("texte complet");
    assert_eq!(info.basic, catalog.text("card.into_the_heat.basic_text"));
    assert_eq!(info.strong, catalog.text("card.into_the_heat.strong_text"));
}
