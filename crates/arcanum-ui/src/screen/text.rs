//! Noms de carte et textes d'effet traduits.

use super::interaction::signed;
use super::*;

pub(super) fn card_name(card: CardId, catalog: &Catalog) -> String {
    catalog.text(card.card().label_key())
}

pub(super) fn option_effect(card: CardId, option: u8) -> Option<Effect> {
    option_effect_of(card, option, false)
}

/// L'effet d'une option de l'effet de base, ou de l'effet renforcé si `strong`.
pub(super) fn option_effect_of(card: CardId, option: u8, strong: bool) -> Option<Effect> {
    // Spec 055 : les effets d'un artefact sont lus à part.
    if card.card().is_artifact() {
        let choices = arcanum_core::ArtifactChoices::default();
        return card.effect(if strong {
            CardPlay::ThrowAway { option, choices }
        } else {
            CardPlay::Artifact { option, choices }
        });
    }
    let definition = card.card().definition()?;
    if strong {
        definition.strong.option(option)
    } else {
        definition.basic.option(option)
    }
}

/// « Renforcé : Mouvement 4 », ou « Renforcé : pas encore disponible ».
pub(super) fn strong_effect_text(strong: BasicEffect, catalog: &Catalog) -> String {
    match strong {
        BasicEffect::Options { .. } => catalog.format(
            "card.strong",
            &[("effect", &basic_effect_text(strong, catalog))],
        ),
        BasicEffect::Unsupported => catalog.text("card.strong_unsupported"),
    }
}

pub(super) fn basic_effect_text(basic: BasicEffect, catalog: &Catalog) -> String {
    match basic {
        BasicEffect::Options { options, cost, .. } => {
            let effects = options
                .iter()
                .map(|effect| effect_text(*effect, catalog))
                .reduce(|first, second| {
                    catalog.format("effect.or", &[("first", &first), ("second", &second)])
                })
                .unwrap_or_default();
            match cost {
                Cost::None => effects,
                Cost::DiscardAnotherCard => {
                    catalog.format("effect.discard_another", &[("effects", &effects)])
                }
                // Spec 056 : le texte de l'effet dit déjà la carte détruite.
                Cost::DestroyActionCard => effects,
            }
        }
        BasicEffect::Unsupported => catalog.text("effect.unsupported"),
    }
}

/// « Mouvement 2 », « Attaque de glace 2 », « Jeton bleu »…
pub fn effect_text(effect: Effect, catalog: &Catalog) -> String {
    let colored = |key, color| catalog.format(key, &[("color", &color_name(color, catalog))]);
    let (key, amount) = match effect {
        // Spec 025 : le mana gagné.
        Effect::GainToken(color) => return colored("effect.gain_token", color),
        Effect::GainCrystal(color) => return colored("effect.gain_crystal", color),
        Effect::PayForCrystal(color) => return colored("effect.pay_for_crystal", color),
        Effect::ExtraDie => return catalog.text("effect.extra_die"),
        Effect::ExtraDieBlackAnyColor => return catalog.text("effect.extra_die_black"),
        // Spec 034 : soin et pioche.
        Effect::Heal(amount) => ("effect.heal", amount),
        Effect::DrawCards(count) => {
            return catalog.format("effect.draw_cards", &[("count", &count.to_string())]);
        }
        Effect::SetDice { dice, .. } => ("effect.set_dice", dice),
        Effect::Boost(bonus) => ("effect.boost", bonus),
        Effect::Move(amount) => ("effect.move", amount),
        Effect::Influence(amount) => ("effect.influence", amount),
        // Spec 058 : compétences du second lot.
        Effect::InteractionInfluence(amount) => {
            return catalog.format(
                "skill.interaction_influence",
                &[("amount", &amount.to_string())],
            );
        }
        Effect::MovePerUnit { max } => {
            return catalog.format("effect.move_per_unit", &[("max", &max.to_string())]);
        }
        Effect::ThrowWounds(count) => {
            return catalog.format("skill.throw_wounds", &[("count", &count.to_string())]);
        }
        Effect::Flight { spaces, cost } => {
            let spaces = spaces.to_string();
            return if cost == 0 {
                catalog.format("effect.flight", &[("spaces", &spaces)])
            } else {
                catalog.format(
                    "effect.flight_cost",
                    &[("spaces", &spaces), ("cost", &cost.to_string())],
                )
            };
        }
        Effect::CrystalColorInfluence => return catalog.text("effect.crystal_color_influence"),
        Effect::DiscardWoundToDraw => return catalog.text("effect.discard_wound_to_draw"),
        Effect::ResistanceBreak => return catalog.text("effect.resistance_break"),
        Effect::SourceOpening => return catalog.text("effect.source_opening"),
        Effect::CalmWeather => return catalog.text("effect.calm_weather"),
        Effect::ManaOverload => return catalog.text("effect.mana_overload"),
        // Spec 040 : Menace renforcée et Nobles manières.
        Effect::InfluenceReputation { amount, reputation } => {
            return catalog.format(
                "effect.influence_reputation",
                &[
                    ("amount", &amount.to_string()),
                    ("reputation", &signed(i32::from(reputation))),
                ],
            );
        }
        Effect::InfluenceInInteraction {
            amount,
            fame,
            reputation,
        } => {
            let mut bonus = Vec::new();
            if fame > 0 {
                bonus.push(catalog.format("effect.fame", &[("amount", &fame.to_string())]));
            }
            if reputation != 0 {
                bonus.push(catalog.format(
                    "effect.reputation",
                    &[("amount", &signed(i32::from(reputation)))],
                ));
            }
            return catalog.format(
                "effect.influence_in_interaction",
                &[
                    ("amount", &amount.to_string()),
                    ("bonus", &bonus.join(", ")),
                ],
            );
        }
        Effect::Attack {
            amount,
            element,
            kind,
        } => match (kind, element) {
            (AttackKind::Melee, Element::Physical) => ("effect.attack", amount),
            (AttackKind::Melee, Element::Fire) => ("effect.fire_attack", amount),
            (AttackKind::Melee, Element::Ice) => ("effect.ice_attack", amount),
            (AttackKind::Melee, Element::ColdFire) => ("effect.cold_fire_attack", amount),
            (AttackKind::Ranged, Element::Physical) => ("effect.ranged_attack", amount),
            (AttackKind::Ranged, Element::Fire) => ("effect.ranged_fire_attack", amount),
            (AttackKind::Ranged, Element::Ice) => ("effect.ranged_ice_attack", amount),
            (AttackKind::Ranged, Element::ColdFire) => ("effect.ranged_cold_fire_attack", amount),
            (AttackKind::Siege, Element::Physical) => ("effect.siege_attack", amount),
            (AttackKind::Siege, Element::Fire) => ("effect.siege_fire_attack", amount),
            (AttackKind::Siege, Element::Ice) => ("effect.siege_ice_attack", amount),
            (AttackKind::Siege, Element::ColdFire) => ("effect.siege_cold_fire_attack", amount),
        },
        Effect::Block { amount, element } => match element {
            Element::Physical => ("effect.block", amount),
            Element::Fire => ("effect.fire_block", amount),
            Element::Ice => ("effect.ice_block", amount),
            Element::ColdFire => ("effect.cold_fire_block", amount),
        },
        // Spec 033 : Endurance givrée renforcée, hors parade en cours.
        Effect::IceBlockPerAbility(amount) => ("effect.ice_block_per_ability", amount),
        // Spec 042 : capacités d'unité.
        Effect::SwiftBlock(amount) => ("effect.swift_block", amount),
        Effect::ReadyUnit(level) => {
            return catalog.format("effect.ready_unit", &[("level", &level.to_string())]);
        }
        Effect::TokenAndCrystal(color) | Effect::TokenCrystalAndFame(color) => {
            return colored("effect.token_and_crystal", color);
        }
        // Spec 051 : compétences.
        Effect::CrystalAndToken { crystal, token } => {
            return catalog.format(
                "effect.crystal_and_token",
                &[
                    ("crystal", &color_name(crystal, catalog)),
                    ("token", &super::source::mana_name(token, catalog)),
                ],
            );
        }
        Effect::DrawAndToken { cards, token } => {
            return catalog.format(
                "effect.draw_and_token",
                &[
                    ("cards", &cards.to_string()),
                    ("token", &color_name(token, catalog)),
                ],
            );
        }
        Effect::HealUnit => return catalog.text("effect.heal_unit"),
        Effect::GainTokens(first, second) => {
            return catalog.format(
                "effect.gain_tokens",
                &[
                    ("first", &super::source::mana_name(first, catalog)),
                    ("second", &super::source::mana_name(second, catalog)),
                ],
            );
        }
        // Spec 053 : une option qui ne fait que poser ses modificateurs n'a pas de points à dire.
        Effect::Nothing => return String::new(),
        // Coup de sang (spec 053), Tempête de feu et Blizzard (spec 054).
        Effect::AttackWithWound { .. } => {
            return catalog.format(
                "effect.attack_with_wound",
                &[("attack", &effect_text(effect.resolved(), catalog))],
            );
        }
        // Spec 054 : Déluge de flammes, par ses points de base (le texte écrit dit le reste).
        Effect::AttackPerEnemy { .. } | Effect::BlockPerEnemy { .. } => {
            return effect_text(effect.resolved(), catalog);
        }
        Effect::DestroyEnemy => return catalog.text("effect.destroy_enemy"),
        Effect::CastFromOffer => return catalog.text("card.magic_talent.basic_text"),
        Effect::MoveAndHeal { movement, heal } => {
            return catalog.format(
                "effect.move_and_heal",
                &[
                    ("movement", &movement.to_string()),
                    ("heal", &heal.to_string()),
                ],
            );
        }
        Effect::HealAndReady { heal, level } => {
            return catalog.format(
                "effect.heal_and_ready",
                &[("heal", &heal.to_string()), ("level", &level.to_string())],
            );
        }
        Effect::InfluencePerWound { base, per } => {
            return catalog.format(
                "effect.influence_per_wound",
                &[("base", &base.to_string()), ("per", &per.to_string())],
            );
        }
        // Spec 056 : libellés de la section Libellés, ajoutés par le fil écran (étape 10).
        Effect::WoundCrystalAndToken { crystal, token } => {
            return catalog.format(
                "effect.wound_crystal_and_token",
                &[
                    ("crystal", &color_name(crystal, catalog)),
                    ("token", &super::source::mana_name(token, catalog)),
                ],
            );
        }
        Effect::WoundAndTokens(count) => {
            return catalog.format("effect.wound_and_tokens", &[("count", &count.to_string())]);
        }
        Effect::CrystalsFromDestroyed(count) => {
            return catalog.format(
                "effect.crystals_from_destroyed",
                &[("count", &count.to_string())],
            );
        }
        Effect::CrystalsOtherColors => return catalog.text("effect.crystals_other_colors"),
        Effect::GainOwnedCrystal(color) => return colored("effect.gain_owned_crystal", color),
        Effect::CrystalFromDie => return catalog.text("effect.crystal_from_die"),
        Effect::RerollSource => return catalog.text("effect.reroll_source"),
        Effect::Repeat(times) => {
            return catalog.format("effect.repeat", &[("times", &times.to_string())]);
        }
        Effect::GainSpell(color) => return colored("effect.gain_spell", color),
        Effect::TrainFromOffer(_) => return catalog.text("effect.train"),
        // Spec 057 : libellés ajoutés par le fil écran (étape 10).
        Effect::WoundsPerCrystal(color) => return colored("effect.wounds_per_crystal", color),
        Effect::OthersOnly => return catalog.text("effect.others_only"),
        Effect::HealByTerrain { heal, forest } => {
            return catalog.format(
                "effect.heal_by_terrain",
                &[("heal", &heal.to_string()), ("forest", &forest.to_string())],
            );
        }
        Effect::HealAndReadyLevels {
            heal,
            forest_heal,
            levels,
            forest_levels,
        } => {
            return catalog.format(
                "effect.heal_and_ready_levels",
                &[
                    ("heal", &heal.to_string()),
                    ("forest_heal", &forest_heal.to_string()),
                    ("levels", &levels.to_string()),
                    ("forest_levels", &forest_levels.to_string()),
                ],
            );
        }
        Effect::ReadyAnyUnit { heal: false } => return catalog.text("effect.ready_any_unit"),
        Effect::ReadyAnyUnit { heal: true } => {
            return catalog.text("effect.ready_and_heal_any_unit");
        }
        Effect::UndergroundMove { assault: false } => return catalog.text("effect.underground"),
        Effect::UndergroundMove { assault: true } => {
            return catalog.text("effect.underground_attack");
        }
        Effect::FlyMove { max } => {
            return catalog.format("effect.fly", &[("max", &max.to_string())]);
        }
        Effect::DoesNotAttackPerMove => return catalog.text("effect.does_not_attack_per_move"),
        Effect::DiscardToDeck { random: true } => return catalog.text("effect.meditation"),
        Effect::DiscardToDeck { random: false } => return catalog.text("effect.trance"),
        Effect::ClaimDie { each_turn: false } => return catalog.text("effect.claim_die"),
        Effect::ClaimDie { each_turn: true } => return catalog.text("effect.claim_die_each_turn"),
        // Spec 055 : libellés de la section Libellés, ajoutés par le fil écran.
        Effect::BannerAssigned => return catalog.text("play.assign_banner"),
        Effect::InfluenceAndFame { amount, .. } => ("effect.influence", amount),
        Effect::AmuletOfSun { tokens } | Effect::AmuletOfDarkness { tokens } => {
            ("effect.amulet", tokens)
        }
        Effect::CancelAttacks(count) => ("effect.cancel_attacks", count),
        Effect::ReadyAllUnits => return catalog.text("effect.ready_all_units"),
        Effect::SwordDiscard => return catalog.text("effect.sword_discard"),
        Effect::HornOfWrath { .. } => ("effect.horn_of_wrath", 5),
        Effect::GemPouchDice => return catalog.text("effect.gem_pouch_dice"),
        Effect::TokensEachColorAndTime => return catalog.text("effect.tokens_each_color"),
        Effect::SpellAndCrystalFromOffer => return catalog.text("effect.spell_and_crystal"),
    };
    catalog.format(key, &[("amount", &amount.to_string())])
}

/// Le texte de l'effet, avec « ÷2 » s'il compterait à moitié dans l'attaque ou la parade en cours
/// (spec 032, Q3 : pas de repère quand il compte en plein).
pub(super) fn marked_effect_text(effect: Effect, view: &PlayerView, catalog: &Catalog) -> String {
    let combat = view.combat.as_ref().filter(|combat| !combat.ended);
    // Spec 033 : dans une parade en cours, Endurance givrée renforcée dit son bonus.
    let text = match (effect, combat.and_then(|combat| combat.block_bonus())) {
        (Effect::IceBlockPerAbility(base), Some(bonus)) => catalog.format(
            "effect.cold_toughness_strong",
            &[("base", &base.to_string()), ("bonus", &bonus.to_string())],
        ),
        _ => effect_text(effect, catalog),
    };
    match combat.and_then(|combat| combat.efficiency(effect)) {
        Some(Efficiency::Halved) => format!("{text} {}", catalog.text("combat.mark.halved")),
        _ => text,
    }
}

/// Le texte de l'effet renforcé sans « Renforcé : », sous le titre du bloc (spec 073).
pub(crate) fn card_strong_text(card: CardId, catalog: &Catalog) -> String {
    // Spec 055 : un artefact se dit par son texte écrit.
    if let (true, Some((_, key))) = (
        card.card().is_artifact(),
        card.card().unsupported_text_keys(),
    ) {
        return catalog.text(key);
    }
    let strong = card
        .card()
        .definition()
        .map_or(BasicEffect::Unsupported, |definition| definition.strong);
    let keys = card.card().unsupported_text_keys();
    if let Some(text) = written_text(strong, keys.map(|(_, key)| key), catalog) {
        return text;
    }
    match (strong, keys) {
        (BasicEffect::Unsupported, Some((_, key))) => {
            catalog.format("offers.unsupported", &[("text", &catalog.text(key))])
        }
        (BasicEffect::Unsupported, None) => catalog.text("effect.unsupported"),
        (strong, _) => basic_effect_text(strong, catalog),
    }
}

/// Textes des effets de base et renforcé d'une carte (spec 050) : le texte calculé d'un effet codé,
/// sinon notre texte écrit suivi de « pas encore codé : se joue de côté » ; une carte d'action de
/// base sans texte écrit garde « pas encore disponible ».
pub(crate) fn card_effect_texts(card: CardId, catalog: &Catalog) -> (String, String) {
    let keys = card.card().unsupported_text_keys();
    // Spec 055 : un artefact se dit par son texte écrit, ses effets sont tous codés.
    if let (true, Some((basic, strong))) = (card.card().is_artifact(), keys) {
        return (
            catalog.text(basic),
            catalog.format("card.strong", &[("effect", &catalog.text(strong))]),
        );
    }
    let unsupported =
        |key: &str| catalog.format("offers.unsupported", &[("text", &catalog.text(key))]);
    let definition = card.card().definition();
    let (basic, strong) = match definition {
        Some(definition) => (definition.basic, definition.strong),
        None => (BasicEffect::Unsupported, BasicEffect::Unsupported),
    };
    let basic_key = keys.map(|(basic, _)| basic);
    let strong_key = keys.map(|(_, strong)| strong);
    // Le texte écrit ne se lit que pour un effet non codé : une carte aux effets codés n'en a pas
    // (Boule de glace, issue privée 276).
    let basic = match (basic, basic_key) {
        (BasicEffect::Unsupported, Some(key)) => unsupported(key),
        (basic, _) => written_text(basic, basic_key, catalog)
            .unwrap_or_else(|| basic_effect_text(basic, catalog)),
    };
    let strong = match (strong, strong_key) {
        (BasicEffect::Unsupported, Some(key)) => {
            catalog.format("card.strong", &[("effect", &unsupported(key))])
        }
        (strong, _) => match written_text(strong, strong_key, catalog) {
            Some(text) => catalog.format("card.strong", &[("effect", &text)]),
            None => strong_effect_text(strong, catalog),
        },
    };
    (basic, strong)
}

/// Le texte écrit d'un effet dont une option pose un effet en cours : le texte calculé ne dirait
/// que ses points (spec 052, Interface) ; de même pour une option à mana en plus ou un effet qui
/// change en combat ou selon les Blessures (spec 053) ; et une option par couleur de jeton, que le
/// texte calculé énumérerait (Rituel de sang, issue privée 234).
fn written_text(effect: BasicEffect, key: Option<&str>, catalog: &Catalog) -> Option<String> {
    let BasicEffect::Options {
        options,
        modifiers,
        mana,
        ..
    } = effect
    else {
        return None;
    };
    let partial = options.iter().any(|option| {
        matches!(
            option,
            Effect::Nothing
                | Effect::DestroyEnemy
                | Effect::CastFromOffer
                | Effect::AttackPerEnemy { .. }
                | Effect::BlockPerEnemy { .. }
                | Effect::MoveAndHeal { .. }
                | Effect::HealAndReady { .. }
                | Effect::InfluencePerWound { .. }
                | Effect::WoundCrystalAndToken { .. }
        )
    });
    (partial || mana.iter().any(Option::is_some) || modifiers.iter().any(|kinds| !kinds.is_empty()))
        .then(|| key.map(|key| catalog.text(key)))
        .flatten()
}

/// Le texte d'une option de capacité d'unité : celui de son effet, ou, pour une option qui ne fait
/// que poser des modificateurs, le texte écrit de la capacité (spec 053).
pub(super) fn unit_option_text(
    unit: UnitCard,
    ability: u8,
    printed: &arcanum_core::UnitAbility,
    chosen: &arcanum_core::AbilityOption,
    catalog: &Catalog,
) -> Option<String> {
    if chosen.effect != Effect::Nothing {
        return None;
    }
    // Plusieurs options sans points (Mages d'Altem) : chacune par ses modificateurs.
    Some(if printed.options().len() > 1 {
        chosen
            .modifiers
            .iter()
            .map(|kind| super::play_area::modifier_text(*kind, catalog))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        catalog.text(&unit.unit.ability_key(usize::from(ability)))
    })
}

/// Le texte d'une option de carte, le seul à appeler pour la nommer : celui de son effet (avec le
/// repère « ÷2 » si une vue est donnée), ou le texte écrit de la carte pour une option qui ne fait
/// que poser des modificateurs (Au cœur du combat, spec 053, issue privée 223).
pub(super) fn card_option_label(
    card: CardId,
    effect: Effect,
    strong: bool,
    view: Option<&PlayerView>,
    catalog: &Catalog,
) -> String {
    card_option_text(card, effect, strong, catalog).unwrap_or_else(|| match view {
        Some(view) => marked_effect_text(effect, view, catalog),
        None => effect_text(effect, catalog),
    })
}

/// Le texte écrit d'une option de carte sans effet propre ; `None` pour toute autre option.
pub(super) fn card_option_text(
    card: CardId,
    effect: Effect,
    strong: bool,
    catalog: &Catalog,
) -> Option<String> {
    let (basic, strong_key) = card.card().unsupported_text_keys()?;
    (effect == Effect::Nothing).then(|| catalog.text(if strong { strong_key } else { basic }))
}

/// Vrai si l'effet (de base ou renforcé) de `card` détruit la carte choisie au lieu de la
/// défausser (Décomposition, Entraînement ; spec 056).
pub(super) fn destroys(card: CardId, strong: bool) -> bool {
    card.card().definition().is_some_and(|definition| {
        let effect = if strong {
            definition.strong
        } else {
            definition.basic
        };
        matches!(
            effect,
            BasicEffect::Options {
                cost: Cost::DestroyActionCard,
                ..
            }
        )
    })
}
