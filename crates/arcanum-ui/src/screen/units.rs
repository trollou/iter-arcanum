//! Unités du panneau latéral : la zone, leurs capacités, le recrutement et les soins (spec 042).

use arcanum_core::{HealingTarget, Offer, OwnedUnit, UnitAbility};

use super::offers::roman;
use super::*;

/// Les actions légales qui activent la capacité `ability` de `unit`, avec leur index.
fn activations(legal_actions: &[Action], unit: UnitCard, ability: u8) -> Vec<(usize, &Action)> {
    legal_actions
        .iter()
        .enumerate()
        .filter(|(_, action)| {
            matches!(action, Action::ActivateUnit { unit: activated, ability: chosen, .. }
                if *activated == unit && *chosen == ability)
        })
        .collect()
}

/// « Unités : 1/2 », puis une ligne par unité de la zone avec ses boutons de capacité.
pub(super) fn units_section(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> UnitsSection {
    let disbanding = match mode {
        ScreenMode::Disbanding(recruited) => Some(*recruited),
        _ => None,
    };
    // Spec 055 : le choix de l'unité d'une bannière.
    let banner = super::artifacts::assigning(mode, legal_actions);
    // Issue privée 258 : les unités que vise un lancer légal du sort en cours se cochent.
    let tickable = super::spells::tickable_targets(legal_actions, mode);
    let spell_target = |unit: UnitCard| {
        tickable
            .as_ref()
            .filter(|(_, units, _)| units.contains(&unit))
            .map(|(picks, _, _)| DiscardToggle {
                label: catalog.text("combat.target"),
                checked: picks.units.contains(unit),
                command: Command::ToggleCastUnit(unit),
            })
    };
    UnitsSection {
        header: catalog.format(
            "units.header",
            &[
                // Issue #26 : l'unité sous Loyauté ne prend pas de jeton ordinaire.
                (
                    "count",
                    &view
                        .units
                        .iter()
                        .filter(|owned| Some(owned.card) != view.bonds_unit)
                        .count()
                        .to_string(),
                ),
                ("command", &view.command_tokens.to_string()),
            ],
        ),
        units: view
            .units
            .iter()
            .map(|owned| {
                let line = unit_line(owned, view, legal_actions, disbanding, catalog);
                UnitLine {
                    spell_target: spell_target(owned.card),
                    disband: line
                        .disband
                        .clone()
                        .or_else(|| {
                            banner.and_then(|card| {
                                assign_button(card, owned.card, legal_actions, catalog)
                            })
                        })
                        // Spec 051, C22 : l'unité visée par Inspiration.
                        .or_else(|| {
                            super::skills::skill_unit_button(
                                mode,
                                owned.card,
                                legal_actions,
                                catalog,
                            )
                        }),
                    banner: banner_tag(owned, catalog),
                    banner_uses: if matches!(mode, ScreenMode::Normal) {
                        banner_uses(owned.card, legal_actions, catalog)
                    } else {
                        Vec::new()
                    },
                    ..line
                }
            })
            .collect(),
        empty: view.units.is_empty().then(|| catalog.text("units.none")),
        // Spec 058, C30 : le jeton de Loyauté sans faille, libre ou occupé (issue #26).
        bonds_token: view
            .skills
            .iter()
            .any(|owned| owned.skill == arcanum_core::Skill::BondsOfLoyalty)
            .then(|| match view.bonds_unit {
                None => catalog.text("skill.bonds_token"),
                Some(unit) => catalog.format(
                    "skill.bonds_token_used",
                    &[("name", &catalog.text(unit.unit.label_key()))],
                ),
            }),
    }
}

fn unit_line(
    owned: &OwnedUnit,
    view: &PlayerView,
    legal_actions: &[Action],
    disbanding: Option<UnitCard>,
    catalog: &Catalog,
) -> UnitLine {
    let unit = owned.card.unit;
    let definition = unit.definition();
    let mut state = vec![catalog.text(if owned.ready {
        "units.ready"
    } else {
        "units.spent"
    })];
    if owned.is_wounded() {
        state.push(catalog.format("units.wounded", &[("wounds", &owned.wounds.to_string())]));
    }
    let abilities = definition
        .abilities
        .iter()
        .enumerate()
        .map(|(index, ability)| {
            ability_button(owned.card, index, ability, view, legal_actions, catalog)
        })
        .collect();
    UnitLine {
        card: owned.card,
        text: catalog.format(
            "units.line",
            &[
                ("name", &catalog.text(unit.label_key())),
                ("level", roman(definition.level)),
            ],
        ),
        armor: icon_value(Icon::Armor, definition.armor.to_string(), catalog),
        resistances: definition
            .resistances
            .iter()
            .map(|&resistance| icon_value(Icon::Resistance(resistance), "", catalog))
            .collect(),
        state: state.join(" · "),
        abilities,
        spell_target: None,
        banner: None,
        banner_uses: Vec::new(),
        disband: disbanding.and_then(|recruited| {
            // Spec 045 : le renvoi sert aussi à la récompense unité des ruines.
            let wanted = [
                Action::RecruitUnit {
                    unit: recruited,
                    disband: Some(owned.card),
                    bonds: false,
                },
                Action::TakeUnitReward {
                    unit: recruited,
                    disband: Some(owned.card),
                },
            ];
            legal_actions
                .iter()
                .position(|action| wanted.contains(action))
                .map(|index| Button {
                    label: catalog.text("units.disband"),
                    command: Command::Play(index),
                    shortcut: None,
                })
        }),
        illustration: crate::unit_illustration(unit),
        effects: unit_effects(owned, view, legal_actions, catalog),
    }
}

/// Le bouton d'une capacité : une seule action légale la joue aussitôt, plusieurs ouvrent le
/// choix ; grisé sans action légale, avec « pas encore disponible » si elle n'est pas codée.
fn ability_button(
    unit: UnitCard,
    index: usize,
    printed: &UnitAbility,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> AbilityButton {
    let label = catalog.text(&unit.unit.ability_key(index));
    if !printed.is_supported() {
        return AbilityButton {
            label,
            command: None,
            tooltip: Some(catalog.text("units.unsupported")),
        };
    }
    let ability = u8::try_from(index).unwrap_or(u8::MAX);
    let found = activations(legal_actions, unit, ability);
    let command = match found.as_slice() {
        [] => None,
        [(index, _)] => Some(Command::Play(*index)),
        _ => Some(Command::ChooseUnitAbility {
            unit,
            ability,
            option: None,
        }),
    };
    // Spec 032 : le même repère que les cartes quand l'effet compte à moitié contre la cible.
    let combat = view.combat.as_ref().filter(|combat| !combat.ended);
    let halved = combat.is_some_and(|combat| {
        !found.is_empty()
            && found.iter().all(|(_, action)| match action {
                Action::ActivateUnit { option, .. } => {
                    printed
                        .options()
                        .get(usize::from(*option))
                        .and_then(|chosen| combat.efficiency(chosen.effect))
                        == Some(Efficiency::Halved)
                }
                _ => false,
            })
    });
    let label = if halved {
        format!("{label} {}", catalog.text("combat.mark.halved"))
    } else {
        label
    };
    AbilityButton {
        label,
        command,
        tooltip: None,
    }
}

/// Le bandeau du choix d'une capacité : un bouton par activation légale, nommée par son effet,
/// l'unité visée et le mana payé (« Soin 2 · Dé vert »).
pub(super) fn unit_prompt(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<ManaPrompt> {
    let ScreenMode::ChoosingUnitAbility {
        unit,
        ability,
        option: only,
    } = *mode
    else {
        return None;
    };
    let definition = unit.unit.definition();
    let printed = definition.abilities.get(usize::from(ability))?;
    let mut buttons: Vec<Button> = Vec::new();
    for (index, action) in activations(legal_actions, unit, ability) {
        let Action::ActivateUnit {
            option,
            mana,
            target,
            enemy,
            leadership,
            ..
        } = action
        else {
            continue;
        };
        // Spec 073, C22 : depuis le bouton d'une option, le choix réduit à elle.
        if only.is_some_and(|only| only != *option) {
            continue;
        }
        let Some(chosen) = printed.options().get(usize::from(*option)) else {
            continue;
        };
        // Spec 053 : une option sans points est déjà nommée par le bandeau (la capacité).
        let mut parts = Vec::new();
        let phase = view.combat.as_ref().map(|combat| combat.phase);
        match arcanum_core::leadership_effect(chosen.effect, phase).filter(|_| *leadership) {
            // Spec 058 : « Avec Charisme : Parade 5 », distinct de l'activation sans Charisme.
            Some(effect) => parts.push(catalog.format(
                "skill.with_leadership",
                &[("effect", &effect_text(effect, catalog))],
            )),
            None if chosen.effect != Effect::Nothing => {
                parts.push(effect_text(chosen.effect, catalog));
            }
            None => {}
        }
        if let Some(target) = target {
            parts.push(catalog.text(target.unit.label_key()));
        }
        // Spec 053, C30 : l'ennemi visé, parmi les cibles légales.
        if let Some(enemy) = enemy {
            parts.push(catalog.text(enemy.enemy().label_key()));
        }
        // Les couleurs à payer, dans l'ordre des paiements : la capacité, puis le supplément.
        let colors = printed.mana.into_iter().chain(
            chosen
                .extra_mana
                .iter()
                .map(|&color| ManaColor::Basic(color)),
        );
        parts.extend(
            mana.iter()
                .zip(colors)
                .map(|(payment, color)| payment_label(view, *payment, color, catalog)),
        );
        let label = parts.join(" · ");
        // Deux dés de la même couleur paient pareil : un bouton suffit.
        if !buttons.iter().any(|button| button.label == label) {
            buttons.push(Button {
                label,
                command: Command::Play(index),
                shortcut: None,
            });
        }
    }
    if buttons.is_empty() {
        return None;
    }
    Some(ManaPrompt {
        text: catalog.format(
            "units.choice_prompt",
            &[
                ("name", &catalog.text(unit.unit.label_key())),
                (
                    "ability",
                    &catalog.text(&unit.unit.ability_key(usize::from(ability))),
                ),
            ],
        ),
        payments: buttons,
        cancel: Button {
            label: catalog.text("mana_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    })
}

/// « Dé vert », « Cristal vert (1 sur 3) », « Jeton or », comme le choix du mana (spec 022).
pub(super) fn payment_label(
    view: &PlayerView,
    payment: ManaPayment,
    color: ManaColor,
    catalog: &Catalog,
) -> String {
    let color_text = mana_name(color, catalog);
    match payment {
        // Spec 055 : « illimité (rouge) » (`mana_choice.endless`, section Libellés).
        ManaPayment::Endless(face) => catalog.format(
            "mana_choice.endless",
            &[("color", &mana_name(face, catalog))],
        ),
        ManaPayment::Die(die) => match view.source.get(usize::from(die)) {
            Some(ManaColor::Gold) if color != ManaColor::Gold => {
                catalog.format("mana_choice.gold_die", &[("color", &color_text)])
            }
            Some(face) => {
                catalog.format("mana_choice.die", &[("color", &mana_name(*face, catalog))])
            }
            None => color_text,
        },
        ManaPayment::Crystal => {
            let count = match color {
                ManaColor::Basic(basic) => view.inventory.crystals(basic),
                _ => 0,
            };
            catalog.format(
                "mana_choice.crystal",
                &[
                    ("color", &color_text),
                    ("count", &count.to_string()),
                    ("limit", &CRYSTAL_LIMIT.to_string()),
                ],
            )
        }
        ManaPayment::Token => catalog.format("mana_choice.token", &[("color", &color_text)]),
        // Spec 058 : « Invocation : défausser … », « Polarisation : … comme … ».
        ManaPayment::Invocation(_) | ManaPayment::Polarized { .. } | ManaPayment::StolenDie => {
            super::mana::kind_of(&view.source, payment)
                .and_then(|kind| super::mana::skill_payment_text(view, kind, color, catalog))
                .unwrap_or(color_text)
        }
    }
}

/// En interaction : « Recruter : Paysans (4) », une ligne par unité de l'offre recrutable dans ce
/// lieu, grisée quand l'Influence manque, avec ses caractéristiques en infobulle (issue privée 341).
/// Jetons tous pris, le bouton ouvre le choix de l'unité à renvoyer. Loyauté sans faille ajoute
/// devant « Recruter sous Loyauté », grisé de même (spec 058).
pub(super) fn recruit_buttons(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<PurchaseButton> {
    let recruit = |unit: UnitCard, bonds: bool| {
        legal_actions
            .iter()
            .enumerate()
            .find_map(|(index, action)| match action {
                Action::RecruitUnit {
                    unit: recruited,
                    disband,
                    bonds: under_bonds,
                } if *recruited == unit && *under_bonds == bonds => Some(match disband {
                    None => Command::Play(index),
                    Some(_) => Command::ChooseDisband(unit),
                }),
                _ => None,
            })
    };
    // Appel à la gloire recrute sans payer : pas de Loyauté (spec 057).
    let free = view
        .modifiers
        .iter()
        .any(|modifier| modifier.kind == arcanum_core::ModifierKind::RecruitFromOffer);
    let bonds_free = !free
        && view.bonds_unit.is_none()
        && view
            .skills
            .iter()
            .any(|owned| owned.skill == arcanum_core::Skill::BondsOfLoyalty);
    let offered = view
        .offers
        .get(&Offer::Units)
        .into_iter()
        .flatten()
        .filter_map(|card| card.unit());
    let mut buttons = Vec::new();
    for unit in offered {
        let here = view.recruitable_units.contains(&unit);
        let ordinary = recruit(unit, false);
        let bonded = recruit(unit, true);
        if !here && ordinary.is_none() && bonded.is_none() {
            continue;
        }
        let name = catalog.text(unit.unit.label_key());
        let influence = unit.unit.definition().influence;
        let tooltip = Some(super::offers::unit_card(unit, catalog).lines.join("\n"));
        // Spec 058, C30 : « Recruter sous Loyauté : Arbalétriers d'Utem (1) », sans renvoi.
        if bonded.is_some() || (bonds_free && here) {
            buttons.push(PurchaseButton {
                label: catalog.format(
                    "skill.bonds_recruit",
                    &[
                        ("name", &name),
                        ("cost", &arcanum_core::bonds_cost(influence).to_string()),
                    ],
                ),
                command: bonded,
                tooltip: tooltip.clone(),
                shortcut: None,
            });
        }
        buttons.push(PurchaseButton {
            label: catalog.format(
                "units.recruit",
                &[("name", &name), ("cost", &influence.to_string())],
            ),
            command: ordinary,
            tooltip,
            shortcut: None,
        });
    }
    buttons
}

/// « Choisir l'unité à renvoyer », pendant le choix ; ignoré si le recrutement n'est plus légal.
pub(super) fn disband_prompt(
    mode: &ScreenMode,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<DiscardPrompt> {
    let ScreenMode::Disbanding(recruited) = *mode else {
        return None;
    };
    legal_actions
        .iter()
        .any(|action| {
            matches!(
                action,
                Action::RecruitUnit { unit, disband: Some(_), bonds: false }
                    | Action::TakeUnitReward { unit, disband: Some(_) }
                    if *unit == recruited
            )
        })
        .then(|| DiscardPrompt {
            text: catalog.text("units.disband_prompt"),
            cancel: Button {
                label: catalog.text("discard_choice.cancel"),
                command: Command::CancelChoice,
                shortcut: shortcut_label(Shortcut::Cancel, catalog),
            },
        })
}

/// « Jeter une Blessure (1) », « Soigner Gardes d'Utem (2) » : les dépenses de soin légales.
pub(super) fn healing_buttons(legal_actions: &[Action], catalog: &Catalog) -> Vec<Button> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            let Action::SpendHealing { on } = action else {
                return None;
            };
            let label = match on {
                HealingTarget::Wound => catalog.text("healing.wound"),
                HealingTarget::Unit(unit) => catalog.format(
                    "healing.unit",
                    &[
                        ("name", &catalog.text(unit.unit.label_key())),
                        ("cost", &unit.unit.definition().level.to_string()),
                    ],
                ),
            };
            Some(Button {
                label,
                command: Command::Play(index),
                shortcut: None,
            })
        })
        .collect()
}

/// « Soin : 2 », à la suite des points du tour quand il y en a (spec 042, Q1).
pub(super) fn healing_points(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    (view.healing_points > 0).then(|| {
        catalog.format(
            "healing.points",
            &[("points", &view.healing_points.to_string())],
        )
    })
}
