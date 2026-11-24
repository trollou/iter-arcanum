//! Zone de jeu : cartes jouées et jetons de mana (specs 013, 025).

use super::*;

/// La ligne « Jetons : », s'il y en a (spec 025).
pub(super) fn token_line(view: &PlayerView, catalog: &Catalog) -> Option<TokenLine> {
    let token = |color: ManaColor, key: &str| TokenView {
        color,
        tooltip: catalog.format(key, &[("color", &super::source::mana_name(color, catalog))]),
    };
    // Spec 073, C26 : les jetons dépensés ce tour suivent les jetons disponibles.
    (!view.mana_tokens.is_empty() || !view.spent_mana_tokens.is_empty()).then(|| TokenLine {
        title: catalog.text("play_area.tokens"),
        tokens: view
            .mana_tokens
            .iter()
            .map(|&color| token(color, "play_area.token_tooltip"))
            .collect(),
        spent: view
            .spent_mana_tokens
            .iter()
            .map(|&color| token(color, "play_area.token_spent"))
            .collect(),
    })
}

/// « Effets en cours » : une ligne par modificateur (spec 052, C4), puis les avantages du centre
/// de ce tour et « plus de dé de la Source » (spec 058, C6, C37).
pub(super) fn modifier_list(view: &PlayerView, catalog: &Catalog) -> Option<ModifierList> {
    let mut extra: Vec<String> = view
        .center_advantages
        .iter()
        .filter(|advantage| advantage.active && !advantage.used)
        .map(|advantage| {
            let mut name = catalog.text(&advantage.skill.label_key());
            // Surcharge de mana : la couleur choisie.
            if let Some(color) = advantage.color {
                name = format!("{name} ({})", super::source::mana_name(color, catalog));
            }
            catalog.format("skill.advantage", &[("skill", &name)])
        })
        .collect();
    if view.source_dice_forbidden {
        extra.push(catalog.text("skill.no_dice"));
    }
    match modifier_lines(view, catalog) {
        Some(mut list) => {
            list.lines.extend(extra);
            Some(list)
        }
        None => (!extra.is_empty()).then(|| ModifierList {
            title: catalog.text("play_area.modifiers"),
            lines: extra,
        }),
    }
}

/// Une ligne par modificateur, s'il y en a (spec 052, C4).
fn modifier_lines(view: &PlayerView, catalog: &Catalog) -> Option<ModifierList> {
    (!view.modifiers.is_empty()).then(|| ModifierList {
        title: catalog.text("play_area.modifiers"),
        lines: view
            .modifiers
            .iter()
            .map(|modifier| {
                let source = match modifier.source {
                    // Issue privée 260 : un sort renforcé se nomme par l'effet joué, comme dans la zone
                    // de jeu (Distorsion temporelle).
                    ModifierSource::Card(card) => {
                        let strong = view
                            .play_area
                            .iter()
                            .any(|played| played.card == card && played.play.is_strong());
                        super::spells::spell_name(card, strong, catalog)
                    }
                    ModifierSource::Unit(unit) => catalog.text(unit.unit.label_key()),
                    ModifierSource::Skill(skill) => catalog.text(&skill.label_key()),
                };
                // Spec 056, C33 : les cristaux qui reviendront avec Maîtrise du cristal.
                let effect = match modifier.kind {
                    ModifierKind::ReturnSpentCrystals if !view.crystals_to_return.is_empty() => {
                        let colors: Vec<String> = view
                            .crystals_to_return
                            .iter()
                            .map(|&color| color_name(color, catalog))
                            .collect();
                        catalog.format(
                            "modifier.return_spent_crystals_list",
                            &[("colors", &colors.join(", "))],
                        )
                    }
                    kind => modifier_text(kind, catalog),
                };
                catalog.format(
                    "play_area.modifier_line",
                    &[("source", &source), ("effect", &effect)],
                )
            })
            .collect(),
    })
}

/// Le texte d'un effet en cours selon sa sorte et ses paramètres (spec 052, libellés).
pub(super) fn modifier_text(kind: ModifierKind, catalog: &Catalog) -> String {
    let terrain = |terrain| catalog.text(crate::board::terrain_key(terrain));
    match kind {
        ModifierKind::TerrainCost { terrains, change } => {
            let names = if terrains.is_all() {
                catalog.text("modifier.terrains_all")
            } else {
                terrains
                    .list()
                    .into_iter()
                    .map(terrain)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            match change {
                TerrainChange::Reduce { by, min } => catalog.format(
                    "modifier.terrain_reduce",
                    &[
                        ("terrains", &names),
                        ("by", &by.to_string()),
                        ("min", &min.to_string()),
                    ],
                ),
                TerrainChange::SetTo(cost) => catalog.format(
                    "modifier.terrain_set",
                    &[("terrains", &names), ("cost", &cost.to_string())],
                ),
            }
        }
        ModifierKind::Passable(passable) => {
            catalog.format("modifier.passable", &[("terrain", &terrain(passable))])
        }
        ModifierKind::Convert { to, rate, .. } => match to {
            ConvertedPoints::Attack { kind, .. } => catalog.format(
                if kind == AttackKind::Melee {
                    "modifier.convert_attack"
                } else {
                    "modifier.convert_ranged"
                },
                &[("amount", &rate.to_string()), ("points", "1")],
            ),
            ConvertedPoints::Block {
                element: Element::Physical,
            } => catalog.text("modifier.convert_block"),
            ConvertedPoints::Block { element } => catalog.format(
                "modifier.convert_block_element",
                &[("element", &catalog.text(element_key(element)))],
            ),
        },
        ModifierKind::NextCardBonus { attack, block } => catalog.format(
            "modifier.next_card_bonus",
            &[
                ("attack", &attack.to_string()),
                ("block", &block.to_string()),
            ],
        ),
        ModifierKind::EndOfTurnReputation(change) => catalog.format(
            "modifier.end_of_turn_reputation",
            &[("change", &signed(i32::from(change)))],
        ),
        ModifierKind::ReturnToDeck { top: false } => catalog.text("modifier.return_under_deck"),
        ModifierKind::ReturnToDeck { top: true } => catalog.text("modifier.return_on_deck"),
        // Spec 053 : sortes nouvelles. Celles qui visent un ennemi ne restent pas au joueur : elles
        // ne nomment que l'option d'une capacité qui en a plusieurs.
        ModifierKind::OnEnemy(kind)
        | ModifierKind::OnUnfortifiedEnemy(kind)
        | ModifierKind::OnTargetEnemy(kind) => enemy_kind_text(kind, catalog),
        // Spec 054 : les modificateurs d'ennemi des sorts.
        ModifierKind::OnTargetEnemyByFortification { plain, fortified } => {
            by_fortification(plain, fortified, catalog)
        }
        ModifierKind::OnAllEnemies(kind) => catalog.format(
            "effect.on_all_enemies",
            &[("effect", &enemy_kind_text(kind, catalog))],
        ),
        ModifierKind::OnAllEnemiesByFortification { plain, fortified } => catalog.format(
            "effect.on_all_enemies",
            &[("effect", &by_fortification(plain, fortified, catalog))],
        ),
        ModifierKind::IgnoreSiteFortifications => {
            catalog.text("modifier.ignore_site_fortifications")
        }
        ModifierKind::DeferredAttack { amount, element } => catalog.format(
            "modifier.deferred_attack",
            &[(
                "attack",
                &effect_text(
                    Effect::Attack {
                        amount,
                        element,
                        kind: AttackKind::Melee,
                    },
                    catalog,
                ),
            )],
        ),
        ModifierKind::DestroyBlockedEnemy => catalog.text("effect.destroy_enemy"),
        ModifierKind::CastFromOffer(color) => catalog.format(
            "modifier.cast_from_offer",
            &[("color", &color_name(color, catalog))],
        ),
        // Spec 057 : libellés ajoutés par le fil écran (étape 10).
        ModifierKind::SpaceBending => catalog.text("modifier.space_bending"),
        ModifierKind::TimeBending => catalog.text("modifier.time_bending"),
        ModifierKind::UnitFromOffer => catalog.text("modifier.unit_from_offer"),
        ModifierKind::RecruitFromOffer => catalog.text("modifier.recruit_from_offer"),
        // Spec 055 : libellés de la section Libellés, ajoutés par le fil écran.
        ModifierKind::DayRules => catalog.text("modifier.day_rules"),
        ModifierKind::EndlessMana(color) => catalog.format(
            "modifier.endless_mana",
            &[("color", &source::color_name(color, catalog))],
        ),
        ModifierKind::NightMana => catalog.text("modifier.night_mana"),
        ModifierKind::UnitArmorBonus(bonus) => catalog.format(
            "modifier.unit_armor_bonus",
            &[("amount", &bonus.to_string())],
        ),
        ModifierKind::ThrowAwayReceivedWounds => {
            catalog.text("modifier.throw_away_received_wounds")
        }
        ModifierKind::FamePerEnemyDefeated => catalog.text("modifier.fame_per_enemy_defeated"),
        ModifierKind::DoublePhysicalAttacks => catalog.text("modifier.double_physical_attacks"),
        ModifierKind::GrailFame { left } => {
            catalog.format("modifier.grail_fame", &[("amount", &left.to_string())])
        }
        ModifierKind::DrawPerHandWoundHealed => catalog.text("modifier.draw_per_wound_healed"),
        ModifierKind::FamePerUnitAttackOrBlock => {
            catalog.text("modifier.fame_per_unit_attack_or_block")
        }
        ModifierKind::ManaClaimPerTurn(color) => catalog.format(
            "modifier.mana_claim_per_turn",
            &[("color", &source::color_name(color, catalog))],
        ),
        ModifierKind::DrawBonus(bonus) => {
            catalog.format("modifier.draw_bonus", &[("amount", &bonus.to_string())])
        }
        // Au cœur du combat : le même bonus en Attaque et en Parade.
        ModifierKind::UnitBonus { attack, .. } => {
            catalog.format("modifier.unit_bonus", &[("bonus", &attack.to_string())])
        }
        ModifierKind::NoDamageToUnits => catalog.text("modifier.no_damage_to_units"),
        ModifierKind::EndOfTurnPerRecruit {
            fame: 0,
            reputation,
        } => catalog.format(
            "modifier.end_of_turn_per_recruit",
            &[("reputation", &reputation.to_string())],
        ),
        ModifierKind::EndOfTurnPerRecruit { fame, reputation } => catalog.format(
            "modifier.end_of_turn_per_recruit_fame",
            &[
                ("fame", &fame.to_string()),
                ("reputation", &reputation.to_string()),
            ],
        ),
        // Seuls les Mages d'Altem changent l'élément des attaques, en feu glacé.
        ModifierKind::AttacksBecome(_) => catalog.text("modifier.attacks_cold_fire"),
        ModifierKind::AttacksGainSiege => catalog.text("modifier.attacks_siege"),
        ModifierKind::UnitsAllResistances => catalog.text("modifier.units_all_resistances"),
        // Spec 056 : libellés ajoutés par le fil écran (étape 10).
        ModifierKind::ReturnSpentCrystals => catalog.text("modifier.return_spent_crystals"),
        ModifierKind::LearningDiscount { cost, to } => catalog.format(
            "modifier.learning",
            &[
                ("cost", &cost.to_string()),
                (
                    "to",
                    &catalog.text(match to {
                        arcanum_core::CardDestination::Hand => "destination.hand",
                        _ => "destination.discard",
                    }),
                ),
            ],
        ),
    }
}

/// Ce qu'un modificateur fait à un ennemi (specs 053, 054).
fn enemy_kind_text(kind: arcanum_core::EnemyModifierKind, catalog: &Catalog) -> String {
    use arcanum_core::EnemyModifierKind as Kind;
    match kind {
        Kind::DoesNotAttack => catalog.text("combat.enemy.does_not_attack"),
        Kind::ArmorReduction { by, min } => catalog.format(
            "modifier.armor_reduction",
            &[("by", &by.to_string()), ("min", &min.to_string())],
        ),
        Kind::LosesFortifications => catalog.text("combat.enemy.no_fortification"),
        Kind::LosesResistances => catalog.text("combat.enemy.no_resistances"),
        Kind::LosesResistance(Resistance::Fire) => catalog.text("combat.enemy.no_fire_resistance"),
        // Seul Frisson retire une résistance, celle au Feu.
        Kind::LosesResistance(_) => catalog.text("combat.enemy.no_resistances"),
    }
}

/// « Armure −3 (min. 1) (Armure −6 (min. 1) si fortifié) » : Tremblement de terre (spec 054).
fn by_fortification(
    plain: arcanum_core::EnemyModifierKind,
    fortified: arcanum_core::EnemyModifierKind,
    catalog: &Catalog,
) -> String {
    catalog.format(
        "modifier.by_fortification",
        &[
            ("plain", &enemy_kind_text(plain, catalog)),
            ("fortified", &enemy_kind_text(fortified, catalog)),
        ],
    )
}

fn element_key(element: Element) -> &'static str {
    match element {
        Element::Physical => "enemy.element.physical",
        Element::Fire => "enemy.element.fire",
        Element::Ice => "enemy.element.ice",
        Element::ColdFire => "enemy.element.cold_fire",
    }
}

/// `None` pour une carte sans définition : une Blessure ne se joue jamais (spec 031).
pub(super) fn played_line(
    played: &PlayedCard,
    view: &PlayerView,
    catalog: &Catalog,
) -> Option<PlayedLine> {
    let card_color = played.card.card().definition()?.color;
    let card = card_name(played.card, catalog);
    // Spec 053 : une option sans points se nomme par le texte écrit de la carte.
    let strong = played.play.is_strong();
    let effect = repeated_text(played, strong, catalog)
        .unwrap_or_else(|| card_option_label(played.card, played.effect, strong, None, catalog));
    let text = match played.play {
        CardPlay::Sideways(_) | CardPlay::SidewaysSkill { .. } => catalog.format(
            "play_area.line_sideways",
            &[("card", &card), ("effect", &effect)],
        ),
        CardPlay::Basic {
            discard: Some(discarded),
            ..
        } => catalog.format(
            // Spec 056 : Décomposition et Entraînement détruisent la carte.
            if destroys(played.card, false) {
                "play_area.line_destroying"
            } else {
                "play_area.line_discarding"
            },
            &[
                ("card", &card),
                ("effect", &effect),
                ("discarded", &card_name(discarded, catalog)),
            ],
        ),
        // Une option payante dit déjà ce qu'elle a coûté : « Payer vert : cristal vert ».
        CardPlay::Basic { discard: None, .. } => {
            catalog.format("play_area.line", &[("card", &card), ("effect", &effect)])
        }
        // Spec 054 : le sort par son nom renforcé s'il l'est, et le texte de son option ; Déluge de
        // flammes par l'effet obtenu, compté avec les ennemis au lancement (issue privée 246).
        CardPlay::Cast {
            option, from_offer, ..
        } => {
            let effect = if matches!(
                option_effect_of(played.card, option, strong),
                Some(Effect::AttackPerEnemy { .. } | Effect::BlockPerEnemy { .. })
            ) {
                effect_text(played.effect, catalog)
            } else {
                super::spells::spell_option_label(played.card, strong, option, None, catalog)
            };
            catalog.format(
                if from_offer {
                    "play_area.line_from_offer"
                } else {
                    "play_area.line"
                },
                &[
                    (
                        "card",
                        &super::spells::spell_name(played.card, strong, catalog),
                    ),
                    ("effect", &effect),
                ],
            )
        }
        // Spec 055 : libellés des artefacts ajoutés par le fil écran.
        CardPlay::Artifact { .. } | CardPlay::ThrowAway { .. } => {
            catalog.format("play_area.line", &[("card", &card), ("effect", &effect)])
        }
        CardPlay::Boosted { .. } => catalog.format(
            "play_area.line_boosted",
            &[("card", &card), ("effect", &effect)],
        ),
        CardPlay::Strong { mana, .. } => {
            match mana {
                // Le dé pris garde sa face jusqu'à la fin du tour, qui vide la zone de jeu.
                ManaPayment::Die(die) => {
                    let face = view
                        .source
                        .get(usize::from(die))
                        .copied()
                        .unwrap_or(ManaColor::Basic(card_color));
                    catalog.format(
                        "play_area.line_strong_die",
                        &[
                            ("card", &card),
                            ("color", &mana_name(face, catalog)),
                            ("effect", &effect),
                        ],
                    )
                }
                ManaPayment::Crystal => catalog.format(
                    "play_area.line_strong_crystal",
                    &[
                        ("card", &card),
                        ("color", &color_name(card_color, catalog)),
                        ("effect", &effect),
                    ],
                ),
                ManaPayment::Token => catalog.format(
                    "play_area.line_strong_token",
                    &[
                        ("card", &card),
                        ("color", &color_name(card_color, catalog)),
                        ("effect", &effect),
                    ],
                ),
                // Spec 055 : le mana illimité ; spec 058 : Invocation et Polarisation, dont la
                // compétence est dans « Ce tour » ; spec 060 : le dé volé, montré sur la tactique :
                // la ligne sans le mana.
                ManaPayment::Endless(_)
                | ManaPayment::Invocation(_)
                | ManaPayment::Polarized { .. }
                | ManaPayment::StolenDie => {
                    catalog.format("play_area.line", &[("card", &card), ("effect", &effect)])
                }
            }
        }
    };
    Some(PlayedLine {
        color: card_color,
        text,
        tooltip: played_tooltip(played.card, catalog),
    })
}

/// Effet maximum : la cible détruite et l'effet obtenu, « Marche 3 fois (Mouvement 6), détruite »
/// (issue privée 235) ; `None` pour toute autre carte.
fn repeated_text(played: &PlayedCard, strong: bool, catalog: &Catalog) -> Option<String> {
    let (CardPlay::Basic {
        with: Some(target), ..
    }
    | CardPlay::Strong {
        with: Some(target), ..
    }) = played.play
    else {
        return None;
    };
    let Effect::Repeat(times) = played.effect else {
        return None;
    };
    let effect = option_effect_of(target.card, target.option, strong)?.repeated(times);
    Some(catalog.format(
        "play_area.repeated",
        &[
            ("target", &card_name(target.card, catalog)),
            ("times", &times.to_string()),
            ("effect", &effect_text(effect, catalog)),
        ],
    ))
}

/// Le nom de la carte jouée, son effet de base et son effet renforcé (spec 073, C27).
fn played_tooltip(card: CardId, catalog: &Catalog) -> String {
    let (basic, strong) = card_effect_texts(card, catalog);
    [card_name(card, catalog), basic, strong].join("\n")
}
