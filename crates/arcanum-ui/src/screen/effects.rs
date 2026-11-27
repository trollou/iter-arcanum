//! Boutons d'effet des cartes de la main et des unités (spec 073) : une icône et sa valeur, un
//! cristal ou le texte de l'effet ; toutes les options montrées, grisées quand elles ne sont pas
//! légales.

use arcanum_core::{EnemyAbility, OwnedUnit, UnitAbility, leadership_effect};

use super::*;

/// Le visage d'un effet : Mouvement, Influence, Attaque et Parade en icône suivie de leurs points,
/// jeton ou cristal gagné en cristal, le reste en texte (spec 073, tableau « Bouton d'effet »).
pub(super) fn effect_face(effect: Effect, catalog: &Catalog) -> EffectFace {
    let icon =
        |icon: Icon, amount: u8| EffectFace::Icon(icon_value(icon, amount.to_string(), catalog));
    match effect {
        Effect::Move(amount) => icon(Icon::Move, amount),
        Effect::Influence(amount)
        | Effect::InfluenceReputation { amount, .. }
        | Effect::InfluenceInInteraction { amount, .. } => icon(Icon::Influence, amount),
        Effect::Attack {
            amount,
            element,
            kind,
        } => icon(crate::attack_icon(kind, element), amount),
        Effect::Block { amount, element } => icon(crate::block_icon(element), amount),
        Effect::SwiftBlock(amount) => icon(crate::block_icon(Element::Physical), amount),
        Effect::IceBlockPerAbility(amount) => icon(crate::block_icon(Element::Ice), amount),
        Effect::GainToken(color) | Effect::GainCrystal(color) => EffectFace::Crystal {
            color: ManaColor::Basic(color),
            label: effect_text(effect, catalog),
        },
        _ => EffectFace::Text(effect_text(effect, catalog)),
    }
}

/// « ×2 si » et l'icône Vif après la valeur d'une Parade vive (spec #27, C13) ; rien pour les
/// autres effets.
fn effect_condition(effect: Effect, catalog: &Catalog) -> Option<EffectCondition> {
    matches!(effect, Effect::SwiftBlock(_)).then(|| EffectCondition {
        text: catalog.text("effect.double_if"),
        icon: crate::enemy_icon_value(Icon::Ability(EnemyAbility::Swift), "", catalog),
    })
}

/// « ÷2 » quand l'effet compte à moitié contre la cible du combat (spec 032).
fn halved_mark(effect: Effect, view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.combat
        .as_ref()
        .filter(|combat| !combat.ended)
        .and_then(|combat| combat.efficiency(effect))
        .filter(|efficiency| *efficiency == Efficiency::Halved)
        .map(|_| catalog.text("combat.mark.halved"))
}

/// L'infobulle, puis la raison du grisé à la ligne.
fn with_reason(label: String, reason: Option<String>) -> String {
    match reason {
        Some(reason) => format!("{label}\n{reason}"),
        None => label,
    }
}

/// Les options de l'effet de base, hors options payantes (elles vont dans le bloc renforcé) et
/// les options de l'effet renforcé, grisées : la carte pendant un choix, ou la forme d'une carte
/// qui ne peut rien jouer.
pub(super) fn idle_effects(
    card: CardId,
    view: Option<&PlayerView>,
    catalog: &Catalog,
) -> (Vec<EffectButton>, Vec<EffectButton>) {
    // Spec 054 : un sort a ses propres boutons (couleur, noir, mana en plus).
    if card.card().is_spell() {
        return spell_effects(card, view, None, catalog);
    }
    // Spec 055 : un artefact se joue sans mana, renforcé en le détruisant.
    if card.card().is_artifact() {
        return artifact_effects(card, view, None, catalog);
    }
    let Some(definition) = card.card().definition() else {
        return (Vec::new(), Vec::new());
    };
    // Spec 053 : une option sans points se nomme par le texte écrit de la carte.
    let face = |effect: Effect, strong: bool| match card_option_text(card, effect, strong, catalog)
    {
        Some(written) => EffectFace::Text(written),
        None => paid_face(effect, catalog),
    };
    let text =
        |effect: Effect, strong: bool| card_option_label(card, effect, strong, view, catalog);
    let mut basic = Vec::new();
    let mut paid = Vec::new();
    if let BasicEffect::Options { options, cost, .. } = definition.basic {
        for (option, effect) in (0u8..).zip(options) {
            // Spec 053 : le mana de l'option (Magie pure) ou celui de l'effet (Cristallisation).
            let mana = definition.basic.option_mana(option);
            let tooltip = match (mana, cost) {
                (Some(_), _) => effect_text(*effect, catalog),
                // Spec 056 : la carte à détruire se choisit comme celle à défausser (fil écran).
                (None, Cost::DiscardAnotherCard | Cost::DestroyActionCard) => catalog.format(
                    "play.basic_discarding",
                    &[("effect", &text(*effect, false))],
                ),
                (None, Cost::None) => {
                    catalog.format("play.basic", &[("effect", &text(*effect, false))])
                }
            };
            let button = EffectButton {
                condition: None,
                face: face(*effect, false),
                mark: view.and_then(|view| halved_mark(*effect, view, catalog)),
                color: mana.map(ManaColor::Basic),
                command: None,
                tooltip,
            };
            match mana {
                Some(_) => paid.push(button),
                None => basic.push(button),
            }
        }
    }
    // Spec #11 C11 (Q2) : les options payantes à la suite des options gratuites de base.
    basic.extend(paid);
    let mut strong: Vec<EffectButton> = Vec::new();
    if let BasicEffect::Options { options, cost, .. } = definition.strong {
        let key = match cost {
            Cost::None => "play.strong",
            Cost::DiscardAnotherCard | Cost::DestroyActionCard => "play.strong_discarding",
        };
        strong.extend((0u8..).zip(options).map(|(option, effect)| {
            EffectButton {
                condition: None,
                face: face(*effect, true),
                mark: view.and_then(|view| halved_mark(*effect, view, catalog)),
                // Spec #11 C8, C10 : seul le mana en plus de l'option (Magie pure) ; le titre du bloc
                // porte la couleur de la carte.
                color: definition.strong.option_mana(option).map(ManaColor::Basic),
                command: None,
                tooltip: catalog.format(key, &[("effect", &text(*effect, true))]),
            }
        }));
    }
    (basic, strong)
}

/// Le visage d'une option payante sans son paiement, que dit la pastille (spec #11, C11, C12) :
/// le cristal gagné, ou « un sort rouge de l'offre ».
fn paid_face(effect: Effect, catalog: &Catalog) -> EffectFace {
    match effect {
        Effect::PayForCrystal(color) => effect_face(Effect::GainCrystal(color), catalog),
        Effect::GainSpell(color) => EffectFace::Text(catalog.format(
            "effect.spell_from_offer",
            &[("color", &super::source::tagged_color_name(color, catalog))],
        )),
        _ => effect_face(effect, catalog),
    }
}

/// Les boutons d'effet d'une carte hors des modes : chaque option grisée, sauf celles qu'une action
/// légale joue (spec 073, C13 à C17).
pub(super) fn live_effects(
    card: CardId,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> (Vec<EffectButton>, Vec<EffectButton>) {
    if card.card().is_spell() {
        return spell_effects(card, Some(view), Some(legal_actions), catalog);
    }
    if card.card().is_artifact() {
        return artifact_effects(card, Some(view), Some(legal_actions), catalog);
    }
    let (mut basic, mut strong) = idle_effects(card, Some(view), catalog);
    let Some(definition) = card.card().definition() else {
        return (basic, strong);
    };
    // Effet de base : l'action qui joue l'option, ou le choix de la carte à défausser.
    if let BasicEffect::Options { options, .. } = definition.basic {
        let free_options = (0u8..)
            .zip(options)
            .filter(|(option, _)| definition.basic.option_mana(*option).is_none());
        for ((option, _), button) in free_options.zip(basic.iter_mut()) {
            button.command = basic_command(card, option, legal_actions);
        }
    }
    // Effet renforcé : comme les boutons « Renforcer » (spec 022), la raison du grisé en plus.
    if let BasicEffect::Options { options, cost, .. } = definition.strong {
        for ((option, effect), button) in (0u8..).zip(options).zip(strong.iter_mut()) {
            let playable = view
                .is_option_playable_now(*effect, definition.strong.option_modifiers(option))
                && !(cost != Cost::None && view.hand.len() < 2);
            if strong_offered(card, option, legal_actions) {
                button.command = Some(strong_command(
                    card,
                    option,
                    *effect,
                    cost,
                    view,
                    legal_actions,
                ));
            } else if playable {
                let reason = strong_reason(card, *effect, definition.color, view, catalog);
                button.tooltip = with_reason(button.tooltip.clone(), Some(reason));
            }
        }
    }
    // Options payantes (spec 025), à la suite des options gratuites de base (spec #11, C11).
    let free = basic.len() - paid_count(card);
    for (paid, button) in paid_buttons(card, view, legal_actions, catalog)
        .into_iter()
        .zip(basic.iter_mut().skip(free))
    {
        button.command = paid.enabled.then_some(paid.command);
        button.tooltip = with_reason(button.tooltip.clone(), paid.tooltip);
    }
    (basic, strong)
}

/// Le nombre d'options payantes de l'effet de base.
fn paid_count(card: CardId) -> usize {
    match card.card().definition().map(|definition| definition.basic) {
        Some(basic @ BasicEffect::Options { options, .. }) => (0u8..)
            .zip(options)
            .filter(|(option, _)| basic.option_mana(*option).is_some())
            .count(),
        _ => 0,
    }
}

/// L'action légale qui joue cette option de l'effet de base sans rien défausser, sinon le choix de
/// la carte à défausser ; spec 056 : sinon le choix de la cible d'Effet maximum, de la carte d'une
/// offre ou du dé de la Source ; `None` si aucune ne la joue.
fn basic_command(card: CardId, option: u8, legal_actions: &[Action]) -> Option<Command> {
    let mut later = None;
    for (index, action) in legal_actions.iter().enumerate() {
        let Action::PlayCard {
            card: played,
            play:
                CardPlay::Basic {
                    option: chosen,
                    discard,
                    mana: None,
                    with,
                    gain,
                    die,
                },
        } = action
        else {
            continue;
        };
        if *played != card || *chosen != option {
            continue;
        }
        let next = if with.is_some() {
            Command::ChooseBoost(BoostChoice {
                card,
                option,
                basic: true,
                target: None,
                target_option: 0,
                discard: None,
                settings: NO_SET_DICE,
                die: None,
            })
        } else if gain.is_some() {
            Command::ChooseGain(GainChoice {
                card,
                option,
                strong: false,
                discard: None,
                gain: None,
            })
        } else if die.is_some() {
            Command::ChooseDie { card, option }
        } else if discard.is_some() {
            Command::ChooseDiscard { card, option }
        } else {
            return Some(Command::Play(index));
        };
        later.get_or_insert(next);
    }
    later
}

/// Un bouton d'effet par option de chaque capacité de l'unité, avec la pastille du mana de la
/// capacité (spec 073, C20 à C22). Une option qui se paie ou se vise de plusieurs façons ouvre le
/// choix de la capacité réduit à elle.
pub(super) fn unit_effects(
    owned: &OwnedUnit,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<EffectButton> {
    let unit = owned.card;
    let mut buttons = Vec::new();
    for (index, printed) in unit.unit.definition().abilities.iter().enumerate() {
        let ability = u8::try_from(index).unwrap_or(u8::MAX);
        if !printed.is_supported() {
            buttons.push(EffectButton {
                condition: None,
                face: EffectFace::Text(without_mana(
                    catalog.text(&unit.unit.ability_key(index)),
                    printed,
                )),
                mark: None,
                color: printed.mana,
                command: None,
                tooltip: catalog.text("units.unsupported"),
            });
            continue;
        }
        buttons.extend(option_buttons(
            unit,
            ability,
            printed,
            view,
            legal_actions,
            catalog,
        ));
    }
    buttons
}

/// Le texte d'une capacité à mana sans « (mana blanc) » devant : la pastille le dit (spec #11,
/// C13, C14).
fn without_mana(text: String, printed: &UnitAbility) -> String {
    match (printed.mana, text.strip_prefix('(')) {
        (Some(_), Some(rest)) => rest
            .split_once(") ")
            .map_or(text.clone(), |(_, effect)| effect.to_owned()),
        _ => text,
    }
}

fn option_buttons(
    unit: UnitCard,
    ability: u8,
    printed: &UnitAbility,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<EffectButton> {
    let mut buttons = Vec::new();
    for (option, chosen) in (0u8..).zip(printed.options()) {
        // Spec 058 (Q6) : avec Charisme, un second bouton à la valeur augmentée.
        let found = |led: bool| -> Vec<usize> {
            legal_actions
                .iter()
                .enumerate()
                .filter(|(_, action)| {
                    matches!(action, Action::ActivateUnit { unit: activated, ability: a, option: o, leadership, .. }
                        if *activated == unit && *a == ability && *o == option && *leadership == led)
                })
                .map(|(index, _)| index)
                .collect()
        };
        let command = |found: &[usize]| match found {
            [] => None,
            [only] => Some(Command::Play(*only)),
            _ => Some(Command::ChooseUnitAbility {
                unit,
                ability,
                option: Some(option),
            }),
        };
        // Spec 053 : une option sans points se nomme par le texte de la capacité.
        let written = unit_option_text(unit, ability, printed, chosen, catalog);
        buttons.push(EffectButton {
            condition: written
                .is_none()
                .then(|| effect_condition(chosen.effect, catalog))
                .flatten(),
            face: written.clone().map_or_else(
                || effect_face(chosen.effect, catalog),
                |text| EffectFace::Text(without_mana(text, printed)),
            ),
            mark: halved_mark(chosen.effect, view, catalog),
            color: printed.mana,
            command: command(&found(false)),
            tooltip: written.unwrap_or_else(|| marked_effect_text(chosen.effect, view, catalog)),
        });
        let led = found(true);
        let phase = view.combat.as_ref().map(|combat| combat.phase);
        if let (false, Some(effect)) = (led.is_empty(), leadership_effect(chosen.effect, phase)) {
            buttons.push(EffectButton {
                condition: effect_condition(effect, catalog),
                face: effect_face(effect, catalog),
                mark: halved_mark(effect, view, catalog),
                color: printed.mana,
                command: command(&led),
                tooltip: catalog.format(
                    "skill.with_leadership",
                    &[("effect", &marked_effect_text(effect, view, catalog))],
                ),
            });
        }
    }
    buttons
}
