//! Scénarios de la spec 029 (contenu du jeu en données).

use std::collections::BTreeSet;

use arcanum_core::{
    BasicAction, CardId, ContentError, ContentSources, DeedCard, GameContent, Hero,
};

/// Les fichiers embarqués, sauf celui qu'un test remplace.
fn with_basic_actions(text: &str) -> Result<GameContent, ContentError> {
    GameContent::parse(ContentSources {
        basic_actions: text,
        ..ContentSources::EMBEDDED
    })
}

fn with_heroes(text: &str) -> Result<GameContent, ContentError> {
    GameContent::parse(ContentSources {
        heroes: text,
        ..ContentSources::EMBEDDED
    })
}

fn with_tiles(text: &str) -> Result<GameContent, ContentError> {
    GameContent::parse(ContentSources {
        tiles: text,
        ..ContentSources::EMBEDDED
    })
}

fn with_levels(text: &str) -> Result<GameContent, ContentError> {
    GameContent::parse(ContentSources {
        levels: text,
        ..ContentSources::EMBEDDED
    })
}

// --- Fichiers embarqués ---

// C1
#[test]
fn embedded_content_is_valid() {
    let parsed = GameContent::parse(ContentSources::EMBEDDED);
    assert!(parsed.is_ok(), "{:?}", parsed.err());
}

// C2
#[test]
fn every_basic_action_has_one_definition() {
    let content = GameContent::embedded();
    for action in BasicAction::ALL {
        assert_eq!(content.card(action), action.definition(), "{action:?}");
    }
}

// C2
#[test]
fn every_hero_has_one_entry() {
    let content = GameContent::embedded();
    let heroes: BTreeSet<Hero> = Hero::ALL.into_iter().collect();
    assert_eq!(heroes.len(), 4);
    for hero in Hero::ALL {
        assert_eq!(
            content.starting_deck(hero),
            hero.starting_deck(),
            "{hero:?}"
        );
    }
}

// C3 — MKUE p. 3.
#[test]
fn base_game_has_twenty_one_tiles() {
    let tiles = GameContent::embedded().tiles();
    assert_eq!(tiles.len(), 21);
    let ids: BTreeSet<_> = tiles.iter().map(|t| t.id).collect();
    assert_eq!(ids.len(), 21);
}

// C4 — MKUE p. 3 : 16 cartes, dont les deux cartes propres au héros.
#[test]
fn starting_decks_have_sixteen_cards() {
    use BasicAction::*;
    for (hero, own) in [
        (Hero::Arythea, [BattleVersatility, ManaPull]),
        (Hero::Goldyx, [WillFocus, CrystalJoy]),
        (Hero::Norowas, [NobleManners, Rejuvenate]),
        (Hero::Tovak, [ColdToughness, Instinct]),
    ] {
        let deck = GameContent::embedded().starting_deck(hero);
        assert_eq!(deck.len(), 16, "{hero:?}");
        for card in own {
            assert!(
                deck.contains(&CardId::new(DeedCard::Basic(card), 1)),
                "{hero:?} {card:?}"
            );
        }
    }
}

// --- Refus ---

// L1
#[test]
fn unknown_card_name_is_a_syntax_error_with_its_position() {
    let text = "{\n    Marhc: (color: Green, basic: Unsupported, strong: Unsupported),\n}\n";
    match with_basic_actions(text) {
        Err(ContentError::Syntax {
            file, line, column, ..
        }) => {
            assert_eq!(file, "basic_actions.ron");
            assert_eq!(line, 2);
            assert!(column >= 5, "colonne {column}");
        }
        other => panic!("{other:?}"),
    }
}

// L1
#[test]
fn broken_syntax_is_a_syntax_error() {
    let result = with_tiles("[ (id: StartingA, ");
    assert!(
        matches!(
            result,
            Err(ContentError::Syntax {
                file: "tiles.ron",
                ..
            })
        ),
        "{result:?}"
    );
}

const MARCH: &str = "March: (color: Green, basic: Options([Move(2)]), strong: Options([Move(4)])),";

// L2
#[test]
fn card_without_definition_is_missing() {
    let result = with_basic_actions(&format!("{{ {MARCH} }}"));
    match result {
        Err(ContentError::Missing { file, id }) => {
            assert_eq!(file, "basic_actions.ron");
            assert_eq!(id, "Rage");
        }
        other => panic!("{other:?}"),
    }
}

const STANDARD_DECK: &str = "standard_deck: {
    Rage: 2, Determination: 1, Swiftness: 2, March: 2, Stamina: 2, Tranquility: 1,
    Promise: 1, Threaten: 1, Crystallize: 1, ManaDraw: 1, Concentration: 1, Improvisation: 1,
}";

fn heroes_text(standard_deck: &str, arythea: &str) -> String {
    format!(
        "({standard_deck}, heroes: {{
            Arythea: {arythea},
            Goldyx: [(replaced: Concentration, by: WillFocus), (replaced: Crystallize, by: CrystalJoy)],
            Norowas: [(replaced: Promise, by: NobleManners), (replaced: Tranquility, by: Rejuvenate)],
            Tovak: [(replaced: Determination, by: ColdToughness), (replaced: Improvisation, by: Instinct)],
        }}, {DUMMY_CRYSTALS})"
    )
}

/// Spec 061 : les cercles colorés des héros, que `heroes.ron` doit donner.
const DUMMY_CRYSTALS: &str = "dummy_crystals: {
    Arythea: [Red, Red, White], Goldyx: [Green, Green, Blue],
    Norowas: [White, White, Green], Tovak: [Blue, Blue, Red],
}";

const ARYTHEA: &str =
    "[(replaced: Rage, by: BattleVersatility), (replaced: ManaDraw, by: ManaPull)]";

#[test]
fn heroes_text_used_by_the_tests_is_valid() {
    let result = with_heroes(&heroes_text(STANDARD_DECK, ARYTHEA));
    assert!(result.is_ok(), "{:?}", result.err());
}

// L2
#[test]
fn hero_without_entry_is_missing() {
    let text = format!(
        "({STANDARD_DECK}, heroes: {{ Arythea: {ARYTHEA}, Goldyx: {ARYTHEA}, Norowas: {ARYTHEA} }}, {DUMMY_CRYSTALS})"
    );
    match with_heroes(&text) {
        Err(ContentError::Missing { file, id }) => {
            assert_eq!(file, "heroes.ron");
            assert_eq!(id, "Tovak");
        }
        other => panic!("{other:?}"),
    }
}

// L2 — MKUE p. 3 : 2 tuiles de départ, 11 de campagne, 8 royaumes.
#[test]
fn tile_without_definition_is_missing() {
    let text = "[(id: StartingA, center: (Plains, Some(Site(Portal))), sides: [
        (Plains, None), (Forest, None), (Plains, None), (Sea, None), (Sea, None), (Sea, None),
    ])]";
    match with_tiles(text) {
        Err(ContentError::Missing { file, id }) => {
            assert_eq!(file, "tiles.ron");
            assert_eq!(id, "StartingB");
        }
        other => panic!("{other:?}"),
    }
}

// L3
#[test]
fn card_defined_twice_is_a_duplicate() {
    match with_basic_actions(&format!("{{ {MARCH} {MARCH} }}")) {
        Err(ContentError::Duplicate { file, id }) => {
            assert_eq!(file, "basic_actions.ron");
            assert_eq!(id, "March");
        }
        other => panic!("{other:?}"),
    }
}

// L3
#[test]
fn tile_defined_twice_is_a_duplicate() {
    let tile = "(id: Countryside(3), center: (Forest, None), sides: [
        (Hills, None), (Hills, None), (Plains, None), (Plains, None), (Plains, None), (Hills, None),
    ]),";
    match with_tiles(&format!("[{tile} {tile}]")) {
        Err(ContentError::Duplicate { file, id }) => {
            assert_eq!(file, "tiles.ron");
            assert_eq!(id, "Countryside(3)");
        }
        other => panic!("{other:?}"),
    }
}

// L4 — MKUE p. 3 : le deck standard a 16 cartes.
#[test]
fn standard_deck_without_sixteen_cards_is_invalid() {
    let fifteen = STANDARD_DECK.replace("Rage: 2", "Rage: 1");
    let result = with_heroes(&heroes_text(&fifteen, ARYTHEA));
    assert!(
        matches!(result, Err(ContentError::InvalidDeck { .. })),
        "{result:?}"
    );
}

// L4
#[test]
fn replacing_a_card_absent_from_the_standard_deck_is_invalid() {
    let arythea =
        "[(replaced: Instinct, by: BattleVersatility), (replaced: ManaDraw, by: ManaPull)]";
    let result = with_heroes(&heroes_text(STANDARD_DECK, arythea));
    assert!(
        matches!(result, Err(ContentError::InvalidDeck { .. })),
        "{result:?}"
    );
}

// L5
#[test]
fn empty_options_are_an_invalid_effect() {
    let embedded = ContentSources::EMBEDDED.basic_actions;
    let text = embedded.replacen("Options([Move(2)])", "Options([])", 1);
    assert_ne!(
        text, embedded,
        "le fichier embarqué contient Options([Move(2)])"
    );
    let result = with_basic_actions(&text);
    assert!(
        matches!(result, Err(ContentError::InvalidEffect { .. })),
        "{result:?}"
    );
}

// L5
#[test]
fn zero_points_are_an_invalid_effect() {
    let embedded = ContentSources::EMBEDDED.basic_actions;
    let text = embedded.replacen("Move(2)", "Move(0)", 1);
    assert_ne!(text, embedded, "le fichier embarqué contient Move(2)");
    let result = with_basic_actions(&text);
    assert!(
        matches!(result, Err(ContentError::InvalidEffect { .. })),
        "{result:?}"
    );
}

// L2, réciproque : une tuile hors du jeu de base serait une tuile en trop (MKUE p. 3).
#[test]
fn tile_outside_the_base_game_is_unknown() {
    let embedded = ContentSources::EMBEDDED.tiles;
    let text = embedded.replacen("id: Countryside(11)", "id: Countryside(12)", 1);
    assert_ne!(
        text, embedded,
        "le fichier embarqué contient Countryside(11)"
    );
    match with_tiles(&text) {
        Err(ContentError::Unknown { file, id }) => {
            assert_eq!(file, "tiles.ron");
            assert_eq!(id, "Countryside(12)");
        }
        other => panic!("{other:?}"),
    }
}

// L1 : une tuile a exactement six côtés.
#[test]
fn tile_without_six_sides_is_a_syntax_error() {
    let embedded = ContentSources::EMBEDDED.tiles;
    let text = embedded.replacen("            (Sea, None), // S-E\n", "", 1);
    assert_ne!(text, embedded);
    let result = with_tiles(&text);
    assert!(
        matches!(
            result,
            Err(ContentError::Syntax {
                file: "tiles.ron",
                ..
            })
        ),
        "{result:?}"
    );
}

// Spec 025 : le prix d'une option payante se règle avec l'effet de base ; l'effet renforcé est
// déjà payé par la couleur de la carte.
#[test]
fn pay_for_crystal_in_a_strong_effect_is_invalid() {
    let embedded = ContentSources::EMBEDDED.basic_actions;
    let text = embedded.replacen(
        "strong: Options([GainCrystal(Red)",
        "strong: Options([PayForCrystal(Red)",
        1,
    );
    assert_ne!(
        text, embedded,
        "le fichier embarqué contient strong: Options([GainCrystal(Red)"
    );
    let result = with_basic_actions(&text);
    assert!(
        matches!(result, Err(ContentError::InvalidEffect { .. })),
        "{result:?}"
    );
}

// Spec 025 : un dé mis sur une couleur donne au moins un jeton, et on en met un ou deux (la Source
// d'un joueur solo a trois dés ; aucune carte n'en demande plus).
#[test]
fn set_dice_needs_one_or_two_dice_and_tokens() {
    let embedded = ContentSources::EMBEDDED.basic_actions;
    for invalid in [
        "SetDice(dice: 0, tokens: 2)",
        "SetDice(dice: 3, tokens: 2)",
        "SetDice(dice: 1, tokens: 0)",
    ] {
        let text = embedded.replacen("SetDice(dice: 1, tokens: 2)", invalid, 1);
        assert_ne!(
            text, embedded,
            "le fichier embarqué contient SetDice(dice: 1, tokens: 2)"
        );
        let result = with_basic_actions(&text);
        assert!(
            matches!(result, Err(ContentError::InvalidEffect { .. })),
            "{invalid} : {result:?}"
        );
    }
}

// Spec 025 : Concentration renforcée ajoute au moins 1.
#[test]
fn boost_needs_a_bonus() {
    let embedded = ContentSources::EMBEDDED.basic_actions;
    let text = embedded.replacen("Boost(2)", "Boost(0)", 1);
    assert_ne!(text, embedded, "le fichier embarqué contient Boost(2)");
    let result = with_basic_actions(&text);
    assert!(
        matches!(result, Err(ContentError::InvalidEffect { .. })),
        "{result:?}"
    );
}

// Spec 025, Q4 : jamais de jeton or ni noir ; le type n'accepte qu'une couleur de base.
#[test]
fn gold_or_black_token_is_a_syntax_error() {
    let embedded = ContentSources::EMBEDDED.basic_actions;
    for color in ["Gold", "Black"] {
        let text = embedded.replacen("GainToken(Blue)", &format!("GainToken({color})"), 1);
        assert_ne!(
            text, embedded,
            "le fichier embarqué contient GainToken(Blue)"
        );
        let result = with_basic_actions(&text);
        assert!(
            matches!(
                result,
                Err(ContentError::Syntax {
                    file: "basic_actions.ron",
                    ..
                })
            ),
            "{result:?}"
        );
    }
}

// --- Niveaux (spec 035) ---

/// Les niveaux embarqués, une entrée remplacée.
fn levels_replacing(from: &str, to: &str) -> String {
    let embedded = ContentSources::EMBEDDED.levels;
    let text = embedded.replacen(from, to, 1);
    assert_ne!(text, embedded, "le fichier embarqué contient {from}");
    text
}

fn assert_invalid_levels(text: &str) {
    let result = with_levels(text);
    assert!(
        matches!(result, Err(ContentError::InvalidLevels { .. })),
        "{result:?}"
    );
}

// Spec 035, Modèle : dix niveaux.
#[test]
fn levels_need_ten_entries() {
    assert_invalid_levels(&levels_replacing(
        "(fame: 99, armor: 4, hand_limit: 7, command_tokens: 5, reward: SkillAndAdvancedAction),",
        "",
    ));
}

// Spec 035, Modèle : seuils croissants à partir de 0.
#[test]
fn level_thresholds_start_at_zero_and_increase() {
    assert_invalid_levels(&levels_replacing("(fame: 0,", "(fame: 1,"));
    assert_invalid_levels(&levels_replacing("(fame: 15,", "(fame: 8,"));
}

// Spec 035, Modèle : armure, limite de main et jetons de commandement ne baissent pas.
#[test]
fn level_token_values_never_decrease() {
    assert_invalid_levels(&levels_replacing(
        "(fame: 48, armor: 4,",
        "(fame: 48, armor: 2,",
    ));
    assert_invalid_levels(&levels_replacing(
        "hand_limit: 7, command_tokens: 5, reward: LevelToken",
        "hand_limit: 5, command_tokens: 5, reward: LevelToken",
    ));
    assert_invalid_levels(&levels_replacing(
        "command_tokens: 4, reward: SkillAndAdvancedAction",
        "command_tokens: 1, reward: SkillAndAdvancedAction",
    ));
}

// --- Capacités des unités (spec 042) ---

mod unit_abilities {
    use arcanum_core::{
        AbilityEffect, AbilityOption, AttackKind, BasicColor, Effect, Element, EnemyModifierKind,
        ManaColor, ModifierKind, Terrain, TerrainChange, Terrains, Unit, UnitAbility,
    };

    fn attack(amount: u8, element: Element, kind: AttackKind) -> Effect {
        Effect::Attack {
            amount,
            element,
            kind,
        }
    }

    fn melee(amount: u8) -> Effect {
        attack(amount, Element::Physical, AttackKind::Melee)
    }

    fn block(amount: u8, element: Element) -> Effect {
        Effect::Block { amount, element }
    }

    fn ability(mana: Option<ManaColor>, effects: &[Effect]) -> UnitAbility {
        UnitAbility {
            mana,
            effect: AbilityEffect::Options {
                options: effects.iter().copied().map(AbilityOption::new).collect(),
                wounds_unit: false,
            },
        }
    }

    fn free(effects: &[Effect]) -> UnitAbility {
        ability(None, effects)
    }

    fn paid(color: BasicColor, effects: &[Effect]) -> UnitAbility {
        ability(Some(ManaColor::Basic(color)), effects)
    }

    /// Une capacité qui ne fait que poser ses modificateurs, après un mana (spec 053).
    fn modifiers_only(mana: ManaColor, options: &[&[ModifierKind]]) -> UnitAbility {
        UnitAbility {
            mana: Some(mana),
            effect: AbilityEffect::Options {
                options: options
                    .iter()
                    .map(|kinds| AbilityOption {
                        modifiers: kinds.to_vec(),
                        ..AbilityOption::new(Effect::Nothing)
                    })
                    .collect(),
                wounds_unit: false,
            },
        }
    }

    fn attack_or_block(amount: u8) -> UnitAbility {
        free(&[melee(amount), block(amount, Element::Physical)])
    }

    /// Les capacités de `../mage-knight/docs/regles/cartes/unites.md`, version *Lost Legion* des Gardes d'Utem et
    /// des Gardiens d'Altem, dans l'ordre des cartes.
    fn expected(unit: Unit) -> Vec<UnitAbility> {
        use BasicColor::{Blue, Green, Red, White};
        use Element::{ColdFire, Fire, Ice, Physical};
        match unit {
            Unit::Peasants => vec![
                attack_or_block(2),
                free(&[Effect::Influence(2)]),
                free(&[Effect::Move(2)]),
            ],
            Unit::Herbalists => vec![
                paid(Green, &[Effect::Heal(2)]),
                free(&[Effect::ReadyUnit(2)]),
                free(&[Effect::GainToken(Green)]),
            ],
            // Spec 052 : Mouvement 2 et forêts, collines, marais −1 (min. 0).
            Unit::Foresters => vec![
                UnitAbility {
                    mana: None,
                    effect: AbilityEffect::Options {
                        options: vec![AbilityOption {
                            modifiers: vec![ModifierKind::TerrainCost {
                                terrains: Terrains::of(&[
                                    Terrain::Forest,
                                    Terrain::Hills,
                                    Terrain::Swamp,
                                ]),
                                change: TerrainChange::Reduce { by: 1, min: 0 },
                            }],
                            ..AbilityOption::new(Effect::Move(2))
                        }],
                        wounds_unit: false,
                    },
                },
                free(&[block(3, Physical)]),
            ],
            Unit::UtemCrossbowmen => vec![
                attack_or_block(3),
                free(&[attack(2, Physical, AttackKind::Ranged)]),
            ],
            Unit::UtemGuardsmen => vec![free(&[melee(2)]), free(&[Effect::SwiftBlock(4)])],
            Unit::UtemSwordsmen => vec![
                attack_or_block(3),
                UnitAbility {
                    mana: None,
                    effect: AbilityEffect::Options {
                        options: vec![
                            AbilityOption::new(melee(6)),
                            AbilityOption::new(block(6, Physical)),
                        ],
                        wounds_unit: true,
                    },
                },
            ],
            Unit::GuardianGolems => vec![
                attack_or_block(2),
                paid(Red, &[block(4, Fire)]),
                paid(Blue, &[block(4, Ice)]),
            ],
            Unit::Illusionists => vec![
                free(&[Effect::Influence(4)]),
                // Spec 053.
                modifiers_only(
                    ManaColor::Basic(White),
                    &[&[ModifierKind::OnUnfortifiedEnemy(
                        EnemyModifierKind::DoesNotAttack,
                    )]],
                ),
                free(&[Effect::GainCrystal(White)]),
            ],
            Unit::RedCapeMonks => vec![
                attack_or_block(3),
                paid(Red, &[attack(4, Fire, AttackKind::Melee), block(4, Fire)]),
            ],
            Unit::NorthernMonks => vec![
                attack_or_block(3),
                paid(Blue, &[attack(4, Ice, AttackKind::Melee), block(4, Ice)]),
            ],
            Unit::SavageMonks => vec![
                attack_or_block(3),
                paid(Green, &[attack(4, Physical, AttackKind::Siege)]),
            ],
            Unit::FireGolems => vec![
                attack_or_block(3),
                paid(Red, &[attack(4, Fire, AttackKind::Ranged)]),
            ],
            Unit::IceGolems => vec![
                attack_or_block(3),
                paid(Blue, &[attack(6, Ice, AttackKind::Melee)]),
            ],
            Unit::FireMages => vec![
                free(&[attack(3, Fire, AttackKind::Ranged)]),
                paid(Red, &[attack(6, Fire, AttackKind::Melee), block(6, Fire)]),
                free(&[Effect::TokenAndCrystal(Red)]),
            ],
            Unit::IceMages => vec![
                free(&[attack(4, Ice, AttackKind::Melee), block(4, Ice)]),
                paid(Blue, &[attack(4, Ice, AttackKind::Siege)]),
                free(&[Effect::TokenAndCrystal(Blue)]),
            ],
            Unit::AmotepGunners => vec![
                attack_or_block(5),
                paid(Red, &[attack(6, Fire, AttackKind::Ranged)]),
            ],
            Unit::AmotepFreezers => vec![
                attack_or_block(5),
                // Spec 053.
                modifiers_only(
                    ManaColor::Basic(Blue),
                    &[&[
                        ModifierKind::OnEnemy(EnemyModifierKind::DoesNotAttack),
                        ModifierKind::OnEnemy(EnemyModifierKind::ArmorReduction { by: 3, min: 1 }),
                    ]],
                ),
            ],
            Unit::Catapults => vec![
                free(&[attack(3, Physical, AttackKind::Siege)]),
                paid(Red, &[attack(5, Fire, AttackKind::Siege)]),
                paid(Blue, &[attack(5, Ice, AttackKind::Siege)]),
            ],
            Unit::AltemMages => {
                // Compendium, *ALTEM MAGES* Q3 : les six couleurs, une paire par option.
                let mut pairs = Vec::new();
                for (index, &first) in ManaColor::ALL.iter().enumerate() {
                    for &second in &ManaColor::ALL[index..] {
                        pairs.push(Effect::GainTokens(first, second));
                    }
                }
                let cold_fire = |amount, extra: &[BasicColor]| {
                    [
                        attack(amount, ColdFire, AttackKind::Melee),
                        block(amount, ColdFire),
                    ]
                    .map(|effect| AbilityOption {
                        effect,
                        extra_mana: extra.to_vec(),
                        modifiers: Vec::new(),
                    })
                };
                let mut options = Vec::new();
                options.extend(cold_fire(5, &[]));
                options.extend(cold_fire(7, &[Blue]));
                options.extend(cold_fire(7, &[Red]));
                options.extend(cold_fire(9, &[Blue, Red]));
                vec![
                    free(&pairs),
                    UnitAbility {
                        mana: None,
                        effect: AbilityEffect::Options {
                            options,
                            wounds_unit: false,
                        },
                    },
                    // Spec 053.
                    modifiers_only(
                        ManaColor::Black,
                        &[
                            &[ModifierKind::AttacksBecome(ColdFire)],
                            &[ModifierKind::AttacksGainSiege],
                        ],
                    ),
                ]
            }
            Unit::AltemGuardians => vec![
                free(&[melee(5)]),
                free(&[Effect::SwiftBlock(8)]),
                // Spec 053.
                modifiers_only(
                    ManaColor::Basic(Green),
                    &[&[ModifierKind::UnitsAllResistances]],
                ),
            ],
        }
    }

    // Spec 042, Modèle : chaque unité a les capacités de cartes/unites.md, dans l'ordre.
    #[test]
    fn every_unit_has_the_abilities_of_its_card() {
        for unit in Unit::ALL {
            assert_eq!(unit.definition().abilities, expected(unit), "{unit:?}");
        }
    }

    // Spec 042, Q2 : cinq capacités restaient `Unsupported` ; la spec 052 code celle des
    // Bûcherons, la spec 053 les quatre autres (C36).
    #[test]
    fn no_ability_is_unsupported() {
        let unsupported: Vec<(Unit, usize)> = Unit::ALL
            .into_iter()
            .flat_map(|unit| {
                unit.definition()
                    .abilities
                    .iter()
                    .enumerate()
                    .filter(|(_, ability)| !ability.is_supported())
                    .map(move |(index, _)| (unit, index))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(unsupported, vec![]);
    }
}
