//! Spec 071 : icônes et raccourcis visibles, côté écran.

use std::fs;

use arcanum_core::{AttackKind, Element, EnemyAbility, Hero, Resistance};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Icon, IconValue, attack_icon, block_icon, hero_emblem, repository_assets_dir, ui_images,
};

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

// --- Jeu d'icônes ---

// Spec 071, C1 : un SVG par image de `ui/`, comme `every_board_image_exists` (spec 011).
#[test]
fn every_icon_image_exists() {
    let images_dir = repository_assets_dir().join("images");
    let images = ui_images();
    // 26 icônes nommées (spec 074, C8 : les deux dos de tuile), 6 cristaux pleins et
    // l'emplacement vide, 4 emblèmes de héros ; le « i » des cartes de la main (spec #11).
    // Spec #27, C1 : les cinq capacités des ennemis et l'invocation.
    assert_eq!(images.len(), 44);
    assert!(images.contains(&Icon::Info.image()));
    for icon in Icon::all() {
        assert!(
            images.contains(&icon.image()),
            "{icon:?} absente de ui_images"
        );
    }
    for image in &images {
        let path = image.path(&images_dir);
        let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?} manque"));
        assert!(text.starts_with("<svg"), "{path:?} n'est pas un SVG");
    }
}

// Spec 071, C1 : chaque icône a son nom en français, sans clé visible.
#[test]
fn every_icon_has_a_name() {
    let catalog = french();
    let keys = catalog.keys();
    // Spec 074, C8 : la planche passe de 24 à 26 icônes.
    // Spec #27, C2 : puis à 32, avec les capacités des ennemis et l'invocation.
    assert_eq!(Icon::all().len(), 32);
    for icon in Icon::all() {
        assert!(
            keys.contains(icon.name_key()),
            "{icon:?} : clé {} absente",
            icon.name_key()
        );
    }
    assert_eq!(catalog.text(Icon::Move.name_key()), "Mouvement");
    assert_eq!(
        catalog.text(Icon::Resistance(Resistance::Ice).name_key()),
        "Résistance à la glace"
    );
    assert_eq!(
        catalog.text(Icon::TileCountryside.name_key()),
        "Tuile de campagne"
    );
    assert_eq!(catalog.text(Icon::TileCore.name_key()), "Tuile royaume");
    assert_eq!(
        catalog.text(Icon::Ability(EnemyAbility::Swift).name_key()),
        "Vif"
    );
    assert_eq!(catalog.text(Icon::Summon.name_key()), "Invocation");
}

// Spec 071, C2 : l'icône d'une attaque suit sa sorte et son élément.
#[test]
fn attack_icon_follows_kind_and_element() {
    let catalog = french();
    let fire = attack_icon(AttackKind::Melee, Element::Fire);
    assert_eq!(
        fire.image().path("".as_ref()),
        "ui/attack_fire.svg".as_ref() as &std::path::Path
    );
    assert_eq!(catalog.text(fire.name_key()), "Attaque de feu");
    let ice = attack_icon(AttackKind::Ranged, Element::Ice);
    assert_eq!(
        ice.image().path("".as_ref()),
        std::path::Path::new("ui").join("ranged_attack_ice.svg")
    );
    assert_eq!(catalog.text(ice.name_key()), "Attaque à distance de glace");
    let siege = attack_icon(AttackKind::Siege, Element::ColdFire);
    assert_eq!(
        siege.image().path("".as_ref()),
        std::path::Path::new("ui").join("siege_attack_cold_fire.svg")
    );
    assert_eq!(
        catalog.text(siege.name_key()),
        "Attaque de siège de feu glacé"
    );
    assert_eq!(
        attack_icon(AttackKind::Melee, Element::Physical)
            .image()
            .path("".as_ref()),
        std::path::Path::new("ui").join("attack.svg")
    );
}

// Spec 071, C2 : une Parade de feu, la rondache de feu.
#[test]
fn block_icon_follows_element() {
    let catalog = french();
    let fire = block_icon(Element::Fire);
    assert_eq!(
        fire.image().path("".as_ref()),
        std::path::Path::new("ui").join("block_fire.svg")
    );
    assert_eq!(catalog.text(fire.name_key()), "Parade de feu");
    assert_eq!(
        catalog.text(block_icon(Element::Physical).name_key()),
        "Parade"
    );
}

// --- Héros ---

// Spec 071, C26 : un emblème par héros, dans `ui/`.
#[test]
fn every_hero_has_an_emblem() {
    let images_dir = repository_assets_dir().join("images");
    for hero in Hero::ALL {
        let emblem = hero_emblem(hero);
        assert!(ui_images().contains(&emblem), "{hero:?}");
        let path = emblem.path(&images_dir);
        let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?} manque"));
        assert!(text.starts_with("<svg"));
    }
    assert_eq!(
        hero_emblem(Hero::Tovak).path("".as_ref()),
        std::path::Path::new("ui").join("hero_tovak.svg")
    );
}

// --- Panneau latéral ---

mod support;
use support::ctrl;

#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;

fn panel(view: &arcanum_core::PlayerView, catalog: &Catalog) -> arcanum_ui::GameScreen {
    arcanum_ui::game_screen(
        view,
        arcanum_ui::ScreenPhase::Playing { legal_actions: &[] },
        &arcanum_ui::ScreenMode::Normal,
        catalog,
    )
}

fn value(icon: Icon, value: &str, tooltip: &str) -> IconValue {
    IconValue {
        icon,
        value: value.into(),
        tooltip: tooltip.into(),
        rule: None,
    }
}

// Spec 071, C3 : Mouvement et Influence en icônes, à la place de « Mouvement : 3 · Influence : 2 ».
#[test]
fn points_line_uses_icons() {
    let catalog = french();
    let state = arene::arena_at(arene::VILLAGE, &[], &[], 0, 1);
    let mut view = arcanum_core::view(&state, arene::SOLO);
    view.move_points = 3;
    view.influence_points = 2;
    let points = panel(&view, &catalog).points;
    assert_eq!(
        points.icons,
        [
            value(Icon::Move, "3", "Mouvement"),
            value(Icon::Influence, "2", "Influence"),
        ]
    );
    assert_eq!(points.text, "");
}

// Spec 071, C4, repris par la spec 073, C23 : l'icône Gloire et sa valeur sous le niveau
// (issue privée 259 : la ligne dessinée).
#[test]
fn fame_line_starts_with_icon() {
    let catalog = french();
    let state = arene::arena_at(arene::VILLAGE, &[], &[], 0, 1);
    let screen = panel(&arcanum_core::view(&state, arene::SOLO), &catalog);
    assert_eq!(screen.player_fame, value(Icon::Fame, "0 / 3", "Gloire"));
    assert_eq!(screen.player_level, "Niveau 1");

    let state = arene::arena_at(arene::VILLAGE, &[], &[], 104, 10);
    let screen = panel(&arcanum_core::view(&state, arene::SOLO), &catalog);
    assert_eq!(screen.player_fame, value(Icon::Fame, "104", "Gloire"));
    assert_eq!(screen.player_level, "Niveau 10");
}

// Spec 071, C5 : l'icône Armure et l'armure du héros, puis la limite de main et les jetons.
#[test]
fn level_stats_start_with_armor_icon() {
    let catalog = french();
    let state = arene::arena_at(arene::VILLAGE, &[], &[], 0, 1);
    let view = arcanum_core::view(&state, arene::SOLO);
    let stats = panel(&view, &catalog).level_stats;
    assert_eq!(
        stats.icons,
        [value(Icon::Armor, &view.armor.to_string(), "Armure")]
    );
    assert_eq!(stats.text, "Limite de main 5 · Jetons de commandement 1");
}

// --- Écran de combat ---

use arcanum_core::{Action, Enemy, EnemyToken, GameState, Hex, legal_actions, view};
use arcanum_ui::{CombatChoice, CombatScreen, ValuePiece, combat_screen};
use arene::{SOLO, done, fight, foe};

fn combat_of(state: &GameState, catalog: &Catalog) -> CombatScreen {
    let legal = legal_actions(state, SOLO);
    combat_screen(
        &view(state, SOLO),
        &legal,
        &CombatChoice::default(),
        arcanum_core::UndoStatus::Possible,
        catalog,
    )
    .expect("écran de combat")
}

fn icon(icon: Icon, value: &str, tooltip: &str) -> ValuePiece {
    ValuePiece::Icon(IconValue {
        icon,
        value: value.into(),
        tooltip: tooltip.into(),
        rule: None,
    })
}

/// Une valeur à icône qui porte sa règle (spec #27).
fn ruled(icon: Icon, value: &str, tooltip: &str, rule: &str) -> IconValue {
    IconValue {
        icon,
        value: value.into(),
        tooltip: tooltip.into(),
        rule: Some(rule.into()),
    }
}

/// Combat engagé contre ce seul ennemi, posé à côté du village.
fn fighting(enemy: Enemy) -> GameState {
    let hand = [arene::card(arcanum_core::BasicAction::Rage, 1)];
    fight(
        arene::VILLAGE,
        &hand,
        &[(arene::token(enemy, 1), arene::ORCS_2)],
        &[arene::ORCS_2],
    )
}

// Spec 074, C10 : en combat, la barre de menu est la même : nom du scénario, manche et tour,
// pioches et Menu.
#[test]
fn bar_stays_during_combat() {
    let catalog = french();
    let state = fighting(Enemy::FireMages);
    let view = arcanum_core::view(&state, arene::SOLO);
    assert!(view.combat.is_some());
    let bar = support::bar_of(&view, &catalog);
    assert_eq!(bar.title, "Menu");
    // Spec 080 C13 : Préférences… et son séparateur ajoutés.
    // Spec #33 : « À propos… » ajouté au Menu.
    assert_eq!(bar.items.len(), 13);
    // Spec 080 C14 : Préférences… reste actif en combat.
    assert!(bar.items.iter().any(|item| matches!(
        item,
        arcanum_ui::MenuItem::Entry(entry)
            if entry.command == arcanum_ui::Command::OpenPreferences && entry.enabled
    )));
    assert_eq!(bar.scenario.as_deref(), Some("Bac à sable"));
    assert!(bar.round.starts_with("Manche 1/4 (tour "), "{}", bar.round);
    assert_eq!(bar.tile_decks.len(), 2);
}

// Spec 071, C8 : Armure, épée de feu, Gloire (Mages de feu : armure 5, attaque de feu 6,
// gloire 5). Spec #27, C11 : l'attaque de feu de l'ennemi porte sa règle.
#[test]
fn enemy_card_values_use_icons() {
    let catalog = french();
    let screen = combat_of(&fighting(Enemy::FireMages), &catalog);
    assert_eq!(
        screen.enemies[0].values,
        [
            icon(Icon::Armor, "5", "Armure"),
            ValuePiece::Icon(ruled(
                attack_icon(AttackKind::Melee, Element::Fire),
                "6",
                "Attaque de feu",
                "Seules les Parades de glace et de feu glacé comptent en entier ; les autres, à moitié."
            )),
            icon(Icon::Fame, "5", "Gloire"),
        ]
    );
    // Une attaque physique n'a pas de règle.
    let screen = combat_of(&fighting(Enemy::Prowlers), &catalog);
    assert!(
        matches!(&screen.enemies[0].values[1], ValuePiece::Icon(value) if value.rule.is_none()),
        "{:?}",
        screen.enemies[0].values
    );
}

// Spec 071, C9 ; spec #27, C6 : l'attaque d'un Invocateur s'affiche avec l'icône Invocation,
// sans valeur, avec sa règle.
#[test]
fn summoner_attack_uses_icon() {
    let catalog = french();
    let screen = combat_of(&fighting(Enemy::OrcSummoners), &catalog);
    let definition = Enemy::OrcSummoners.definition();
    assert_eq!(
        screen.enemies[0].values,
        [
            icon(Icon::Armor, &definition.armor.to_string(), "Armure"),
            ValuePiece::Icon(ruled(
                Icon::Summon,
                "",
                "Invocation",
                &catalog.text("icon.rule.summon")
            )),
            icon(Icon::Fame, &definition.fame.to_string(), "Gloire"),
        ]
    );
}

// Spec #27, C3 : Glaceurs (Fortifié, Vif, Paralysant) : trois icônes dans l'ordre de
// `enemies.ron`, sans valeur, nommées et avec leur règle.
#[test]
fn abilities_use_icons() {
    let catalog = french();
    let screen = combat_of(&fighting(Enemy::Freezers), &catalog);
    let abilities: Vec<(Icon, &str, &str, bool)> = screen.enemies[0]
        .abilities
        .iter()
        .map(|value| {
            (
                value.icon,
                value.value.as_str(),
                value.tooltip.as_str(),
                value.rule.is_some(),
            )
        })
        .collect();
    assert_eq!(
        abilities,
        [
            (Icon::Ability(EnemyAbility::Fortified), "", "Fortifié", true),
            (Icon::Ability(EnemyAbility::Swift), "", "Vif", true),
            (
                Icon::Ability(EnemyAbility::Paralyze),
                "",
                "Paralysant",
                true
            ),
        ]
    );
}

// Spec #27, C4 : la règle de Vif (Chevaucheurs de loups : armure 4, attaque 3, Vif).
#[test]
fn ability_tooltip_gives_rule() {
    let catalog = french();
    let screen = combat_of(&fighting(Enemy::WolfRiders), &catalog);
    assert_eq!(
        screen.enemies[0].abilities,
        [ruled(
            Icon::Ability(EnemyAbility::Swift),
            "",
            "Vif",
            "Pour le parer, il faut une Parade du double de son attaque."
        )]
    );
}

// Spec #27, C5 : Rôdeurs, sans capacité, sans icône de capacité.
#[test]
fn no_ability_no_icon() {
    let catalog = french();
    let screen = combat_of(&fighting(Enemy::Prowlers), &catalog);
    assert!(screen.enemies[0].abilities.is_empty());
}

// Spec 071, C10 : une icône par résistance, sans valeur, à la place des badges (spec 032).
#[test]
fn resistances_use_icons() {
    let catalog = french();
    // Gardiens d'Altem : résistances au feu, à la glace et physique.
    let screen = combat_of(&fighting(Enemy::AltemGuardsmen), &catalog);
    // Dans l'ordre de la définition de l'ennemi. Spec #27, C10 : chacune a sa règle.
    assert_eq!(
        screen.enemies[0].resistances,
        [
            ruled(
                Icon::Resistance(Resistance::Fire),
                "",
                "Résistance au feu",
                &catalog.text("icon.rule.resistance_fire")
            ),
            ruled(
                Icon::Resistance(Resistance::Ice),
                "",
                "Résistance à la glace",
                &catalog.text("icon.rule.resistance_ice")
            ),
            ruled(
                Icon::Resistance(Resistance::Physical),
                "",
                "Résistance physique",
                &catalog.text("icon.rule.resistance_physical")
            ),
        ]
    );
}

// Spec 071, C11 : la créature invoquée (spec 033) a ses valeurs en icônes.
#[test]
fn summoned_values_use_icons() {
    let catalog = french();
    let state = fighting(Enemy::OrcSummoners);
    let mut view = view(&state, SOLO);
    let minotaur = EnemyToken::new(Enemy::Minotaur, 1);
    let combat = view.combat.as_mut().expect("combat");
    combat.enemies[0].summoned = Some(minotaur);
    let legal = legal_actions(&state, SOLO);
    let screen = combat_screen(
        &view,
        &legal,
        &CombatChoice::default(),
        arcanum_core::UndoStatus::Possible,
        &catalog,
    )
    .expect("écran de combat");
    let definition = Enemy::Minotaur.definition();
    let summoned = screen.enemies[0].summoned.as_ref().expect("créature");
    assert_eq!(
        summoned.values,
        [
            icon(Icon::Armor, &definition.armor.to_string(), "Armure"),
            icon(
                attack_icon(AttackKind::Melee, Element::Physical),
                "5",
                "Attaque"
            ),
            icon(Icon::Fame, &definition.fame.to_string(), "Gloire"),
        ]
    );
}

// Spec 071, C12 : le total d'une attaque en cours porte l'icône de la phase, celui d'une parade
// la rondache physique.
#[test]
fn in_progress_total_uses_phase_icon() {
    let catalog = french();
    let prowlers: Hex = arene::ORCS_2;
    let state = fighting(Enemy::Prowlers);
    let target = foe(&state, prowlers);

    // Phase d'attaque à distance : l'arc, « Attaque à distance ou de siège ».
    let ranged = done(&state, arene::declare(&[target]));
    let total = combat_of(&ranged, &catalog)
        .in_progress
        .expect("attaque")
        .total;
    assert_eq!(
        (total.icon, total.value.as_str(), total.tooltip.as_str()),
        (
            attack_icon(AttackKind::Ranged, Element::Physical),
            "0 / 3",
            "Attaque à distance ou de siège"
        )
    );

    // Phase de parade : la rondache physique.
    let block = done(&state, Action::NextCombatPhase);
    let block = done(&block, arene::declare_block(target));
    let total = combat_of(&block, &catalog)
        .in_progress
        .expect("parade")
        .total;
    assert_eq!(
        (total.icon, total.value.as_str(), total.tooltip.as_str()),
        (block_icon(Element::Physical), "0 / 4", "Parade")
    );

    // Phase d'attaque : l'épée.
    let attack = arene::skip_to_attack_phase(&state);
    let attack = done(&attack, arene::declare(&[target]));
    let total = combat_of(&attack, &catalog)
        .in_progress
        .expect("attaque")
        .total;
    assert_eq!(
        (total.icon, total.value.as_str(), total.tooltip.as_str()),
        (
            attack_icon(AttackKind::Melee, Element::Physical),
            "0 / 3",
            "Attaque"
        )
    );
}

// --- Unités ---

// Spec 071, C13 : nom et niveau en texte, puis l'Armure et les résistances en icônes.
#[test]
fn unit_line_uses_icons() {
    let catalog = french();
    let golems = arcanum_core::UnitCard::new(arcanum_core::Unit::FireGolems, 1);
    let state = arene::Units {
        level: 3,
        owned: vec![golems],
        ..arene::Units::default()
    }
    .state();
    let units = panel(&view(&state, SOLO), &catalog).units;
    let line = &units.units[0];
    assert_eq!(line.text, "Golem de feu · niveau III");
    assert_eq!(line.armor, value(Icon::Armor, "4", "Armure"));
    assert_eq!(
        line.resistances,
        [
            value(Icon::Resistance(Resistance::Fire), "", "Résistance au feu"),
            value(
                Icon::Resistance(Resistance::Physical),
                "",
                "Résistance physique"
            ),
        ]
    );
}

// --- Jouer de côté ---

use arcanum_core::{BasicAction, CardPlay, SidewaysEffect};

/// L'écran de partie avec les actions légales de l'état.
fn screen_with_actions(state: &GameState, catalog: &Catalog) -> arcanum_ui::GameScreen {
    let legal = legal_actions(state, SOLO);
    arcanum_ui::game_screen(
        &view(state, SOLO),
        arcanum_ui::ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &arcanum_ui::ScreenMode::Normal,
        catalog,
    )
}

/// Les boutons-icônes de côté de cette carte : icône, repère, infobulle.
fn sideways_of(
    screen: &arcanum_ui::GameScreen,
    card: arcanum_core::CardId,
) -> Vec<(Icon, Option<String>, String)> {
    screen
        .hand
        .iter()
        .find(|held| held.card == card)
        .expect("carte en main")
        .sideways
        .iter()
        .map(|button| (button.icon, button.mark.clone(), button.tooltip.clone()))
        .collect()
}

// Spec 071, C14 : un bouton-icône par effet légal, un clic joue la carte de côté.
#[test]
fn sideways_buttons_one_per_effect() {
    let catalog = french();
    let march = arene::card(BasicAction::March, 1);
    let state = arene::arena(arene::VILLAGE, &[march], &[]);
    let screen = screen_with_actions(&state, &catalog);
    assert_eq!(
        sideways_of(&screen, march),
        [
            (Icon::Move, None, "Jouer de côté : Mouvement 1".to_string()),
            (
                Icon::Influence,
                None,
                "Jouer de côté : Influence 1".to_string()
            ),
        ]
    );
    let legal = legal_actions(&state, SOLO);
    let commands: Vec<Action> = screen.hand[0]
        .sideways
        .iter()
        .map(|button| match button.command {
            arcanum_ui::Command::Play(index) => legal[index].clone(),
            other => panic!("un clic joue la carte : {other:?}"),
        })
        .collect();
    assert_eq!(
        commands,
        [
            Action::PlayCard {
                card: march,
                play: CardPlay::Sideways(SidewaysEffect::Move),
            },
            Action::PlayCard {
                card: march,
                play: CardPlay::Sideways(SidewaysEffect::Influence),
            },
        ]
    );
}

// Spec 071, C15 : en combat, l'Attaque seule pendant une attaque en cours de la phase d'attaque,
// la Parade seule pendant une parade en cours, aucun sinon.
#[test]
fn sideways_buttons_follow_legal_actions() {
    let catalog = french();
    let march = arene::card(BasicAction::March, 1);
    let rage = arene::card(BasicAction::Rage, 1);
    let state = fight(
        arene::VILLAGE,
        &[march, rage],
        &[(arene::prowlers(), arene::ORCS_2)],
        &[arene::ORCS_2],
    );
    let icons = |state: &GameState| -> Vec<Icon> {
        sideways_of(&screen_with_actions(state, &catalog), march)
            .into_iter()
            .map(|(icon, _, _)| icon)
            .collect()
    };
    // Phase d'attaque à distance, rien en cours.
    assert_eq!(icons(&state), []);
    let block = done(&state, Action::NextCombatPhase);
    let block = done(&block, arene::declare_block(arene::prowlers()));
    assert_eq!(icons(&block), [block_icon(Element::Physical)]);
    let attack = arene::skip_to_attack_phase(&state);
    let attack = done(&attack, arene::declare(&[arene::prowlers()]));
    assert_eq!(
        icons(&attack),
        [attack_icon(AttackKind::Melee, Element::Physical)]
    );
}

// Spec 071, C16 : contre une résistance physique, le bouton d'Attaque porte « ÷2 » (spec 032).
#[test]
fn sideways_halved_mark() {
    let catalog = french();
    let march = arene::card(BasicAction::March, 1);
    let state = fight(
        arene::VILLAGE,
        &[march],
        &[(arene::ironclads(), arene::ORCS_2)],
        &[arene::ORCS_2],
    );
    let state = arene::skip_to_attack_phase(&state);
    let state = done(&state, arene::declare(&[arene::ironclads()]));
    assert_eq!(
        sideways_of(&screen_with_actions(&state, &catalog), march),
        [(
            attack_icon(AttackKind::Melee, Element::Physical),
            Some("÷2".to_string()),
            "Jouer de côté : Attaque 1 ÷2".to_string()
        )]
    );
}

// Spec 071, C17 : une Blessure ne se joue pas de côté (spec 031).
#[test]
fn wound_has_no_sideways_buttons() {
    let catalog = french();
    let wound = arene::wound(1);
    let state = arene::arena(
        arene::VILLAGE,
        &[wound, arene::card(BasicAction::March, 1)],
        &[],
    );
    assert_eq!(
        sideways_of(&screen_with_actions(&state, &catalog), wound),
        []
    );
}

// --- Raccourcis affichés ---

use arcanum_ui::{KEY_BINDINGS, Shortcut, hover_text, key_label, shortcut_label};

// Spec 071, C24 : un bouton sans touche n'a pas de ligne « Raccourci ».
#[test]
fn buttons_without_key_show_no_shortcut() {
    let catalog = french();
    let march = arene::card(BasicAction::March, 1);
    let state = fight(
        arene::VILLAGE,
        &[march],
        &[(arene::prowlers(), arene::ORCS_2)],
        &[arene::ORCS_2],
    );
    let combat = combat_of(&state, &catalog);
    let phase = combat.phase_button.expect("Phase suivante");
    assert_eq!(phase.shortcut, None);
    assert_eq!(hover_text(None, phase.shortcut.as_deref(), &catalog), None);
    let screen = screen_with_actions(&arene::arena(arene::VILLAGE, &[march], &[]), &catalog);
    assert!(
        screen.hand[0]
            .plays
            .iter()
            .all(|button| button.shortcut.is_none())
    );
    assert_eq!(
        hover_text(Some("Il faut un mana vert."), None, &catalog).as_deref(),
        Some("Il faut un mana vert.")
    );
}

// Spec 071, C25 : le libellé affiché vient de la table des touches que lit le clavier.
#[test]
fn shown_shortcut_matches_bound_key() {
    let catalog = french();
    for binding in &KEY_BINDINGS {
        assert_eq!(
            shortcut_label(binding.shortcut, &catalog),
            Some(key_label(binding, &catalog)),
            "{binding:?}"
        );
        // Un raccourci, une seule touche : le texte ne peut pas en dire une autre.
        assert_eq!(
            KEY_BINDINGS
                .iter()
                .filter(|other| other.shortcut == binding.shortcut)
                .count(),
            1,
            "{binding:?}"
        );
    }
    let labels: Vec<Option<String>> = [
        Shortcut::Undo,
        Shortcut::Save,
        Shortcut::OpenLoad,
        Shortcut::NewGame,
        Shortcut::Cancel,
        Shortcut::EndTurn,
        Shortcut::Confirm,
        Shortcut::Offers,
        Shortcut::Interact,
        Shortcut::EnterSite,
        Shortcut::Take(2),
        Shortcut::Take(9),
    ]
    .into_iter()
    .map(|shortcut| shortcut_label(shortcut, &catalog))
    .collect();
    let expected = [
        ctrl("Z"),
        ctrl("S"),
        ctrl("O"),
        ctrl("N"),
        "Échap".into(),
        "Fin".into(),
        "Entrée".into(),
        "O".into(),
        "I".into(),
        "E".into(),
        "2".into(),
        "9".into(),
    ];
    assert_eq!(labels, expected.map(Some).to_vec());
    // Sans touche dans la table, rien à afficher.
    assert_eq!(shortcut_label(Shortcut::Take(10), &catalog), None);
}

// Spec 071, C21 : « Offres… » dit O, « Interagir avec les habitants » I, « Entrer dans : … » E
// (l'offrande à l'autel : `ruines.rs`).
#[test]
fn site_buttons_show_shortcut() {
    let catalog = french();
    let hand = [
        arene::card(BasicAction::NobleManners, 1),
        arene::card(BasicAction::Promise, 1),
        arene::card(BasicAction::March, 1),
    ];
    let screen = screen_with_actions(&arene::arena(arene::VILLAGE, &hand, &[]), &catalog);
    let offers = &screen.bottom_buttons[0];
    assert_eq!(offers.label, "Offres…");
    assert_eq!(offers.shortcut.as_deref(), Some("O"));
    let interact = screen.interact.expect("Interagir avec les habitants");
    assert_eq!(
        hover_text(None, interact.shortcut.as_deref(), &catalog).as_deref(),
        Some("Raccourci : I")
    );
    let screen = screen_with_actions(&arene::Adventure::default().state(), &catalog);
    let enter = screen.adventure.expect("Entrer dans");
    assert_eq!(enter.shortcut.as_deref(), Some("E"));
}

// Spec 071, C23 : les « Annuler » et « Fermer » qu'Échap actionne disent Échap ; « Commencer »
// dit Entrée.
#[test]
fn escape_buttons_show_shortcut() {
    let catalog = french();
    let escape = Some("Échap".to_string());
    // Fenêtre de chargement.
    assert_eq!(
        arcanum_ui::load_window_screen(&[], &catalog).close.shortcut,
        escape
    );
    // Fenêtre de nouvelle partie, en cours de partie.
    let form = arcanum_ui::NewGameForm::default();
    let screen = arcanum_ui::new_game_screen(
        &form,
        &arcanum_ui::NewGameContext::InGame { finished: false },
        &catalog,
    );
    assert_eq!(screen.cancel.expect("Annuler").shortcut, escape);
    assert_eq!(screen.start.shortcut.as_deref(), Some("Entrée"));
    // Fenêtre Offres, hors choix.
    let state = arene::arena(arene::VILLAGE, &[], &[]);
    let window = arcanum_ui::offers_window(&view(&state, SOLO), &[], &catalog);
    assert_eq!(window.close.shortcut, escape);
    // Confirmation de l'entrée dans un site d'aventure.
    let state = arene::Adventure::default().state();
    let legal = legal_actions(&state, SOLO);
    let confirm = arcanum_ui::game_screen(
        &view(&state, SOLO),
        arcanum_ui::ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &arcanum_ui::ScreenMode::ConfirmingAdventure,
        &catalog,
    )
    .adventure_confirm
    .expect("confirmation");
    let cancel = confirm.buttons.last().expect("Annuler");
    assert_eq!(cancel.label, "Annuler");
    assert_eq!(cancel.shortcut, escape);
    // L'entrée elle-même n'a pas de touche.
    assert_eq!(confirm.buttons[0].shortcut, None);
    // Choix de la carte à défausser : son « Annuler ».
    let improvisation = arene::card(BasicAction::Improvisation, 1);
    let state = arene::arena(
        arene::VILLAGE,
        &[improvisation, arene::card(BasicAction::March, 1)],
        &[],
    );
    let legal = legal_actions(&state, SOLO);
    let discard = arcanum_ui::game_screen(
        &view(&state, SOLO),
        arcanum_ui::ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &arcanum_ui::ScreenMode::ChoosingDiscard(arcanum_ui::DiscardChoice {
            card: improvisation,
            option: 0,
        }),
        &catalog,
    )
    .discard_prompt
    .expect("choix de la carte à défausser");
    assert_eq!(discard.cancel.shortcut, escape);
}

// --- Texte à icônes (spec #27) ---

use arcanum_ui::{ICON_MARKERS, icon_pieces, plain_text};

fn english() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt")
}

// Spec #27, C15 : un repère devant le mot, un repère seul à la place du mot, un texte sans
// repère.
#[test]
fn icon_pieces_split_markers() {
    let catalog = french();
    let brutal = ruled(
        Icon::Ability(EnemyAbility::Brutal),
        "",
        "Brutal",
        "S'il n'est pas paré, il inflige le double de son attaque.",
    );
    assert_eq!(
        icon_pieces("Capacités : [[brutal]]brutal", &catalog),
        [
            ValuePiece::Text("Capacités : ".into()),
            ValuePiece::Icon(brutal),
            ValuePiece::Text("brutal".into()),
        ]
    );
    let swift = ruled(
        Icon::Ability(EnemyAbility::Swift),
        "",
        "Vif",
        "Pour le parer, il faut une Parade du double de son attaque.",
    );
    assert_eq!(
        icon_pieces("À parer : 6 ([[swift]], 2 × 3)", &catalog),
        [
            ValuePiece::Text("À parer : 6 (".into()),
            ValuePiece::Icon(swift),
            ValuePiece::Text(", 2 × 3)".into()),
        ]
    );
    assert_eq!(
        icon_pieces("Armure 3", &catalog),
        [ValuePiece::Text("Armure 3".into())]
    );
    assert_eq!(
        plain_text("Capacités : [[brutal]]brutal", &catalog),
        "Capacités : brutal"
    );
    assert_eq!(
        plain_text("À parer : 6 ([[swift]], 2 × 3)", &catalog),
        "À parer : 6 (Vif, 2 × 3)"
    );
}

/// Les repères `[[nom]]` d'un texte, dans l'ordre.
fn markers(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let end = after.find("]]").expect("repère fermé");
        found.push(after[..end].to_string());
        rest = &after[end + 2..];
    }
    found
}

// Spec #27, C16 : chaque repère nomme une des six icônes, et chaque clé a les mêmes repères dans
// le même ordre en français et en anglais.
#[test]
fn icon_markers_are_known_and_shared() {
    let (fr, en) = (french(), english());
    let known: Vec<&str> = ICON_MARKERS.iter().map(|(name, _)| *name).collect();
    let mut marked = 0;
    for key in fr.keys() {
        let found = markers(&fr.text(&key));
        for name in &found {
            assert!(known.contains(&name.as_str()), "{key} : [[{name}]] inconnu");
        }
        assert_eq!(found, markers(&en.text(&key)), "{key}");
        marked += usize::from(!found.is_empty());
    }
    // Les clés marquées de la spec (section Libellés).
    assert!(marked >= 30, "{marked} clés marquées");
}

// Spec #27, C19 et C20 : hors du combat, l'icône devant le mot, qui reste.
#[test]
fn marked_texts_keep_the_word() {
    let catalog = french();
    for (key, expected) in [
        (
            "board.hover.city.bonus_red",
            "Bonus des défenseurs : [[brutal]]brutal si leur attaque est physique",
        ),
        (
            "board.hover.city.bonus_green",
            "Bonus des défenseurs : [[poison]]toxique si leur attaque est physique",
        ),
        (
            "unit.altem_guardians.ability_2",
            "Parade 8 ; compte double contre un ennemi [[swift]]Vif",
        ),
        (
            "unit.illusionists.ability_2",
            "(mana blanc) Un ennemi [[fortified]]non fortifié n'attaque pas",
        ),
        (
            "card.tremor.strong_text",
            "Un ennemi Armure -3 (-6 si [[fortified]]fortifié), ou tous -2 (-4 si \
             [[fortified]]fortifiés) (min. 1)",
        ),
        (
            "modifier.by_fortification",
            "{plain} ({fortified} si [[fortified]]fortifié)",
        ),
    ] {
        assert_eq!(catalog.text(key), expected, "{key}");
    }
    assert!(
        catalog
            .text("card.expose.basic_text")
            .contains("perd [[fortified]]fortifications")
    );
}
