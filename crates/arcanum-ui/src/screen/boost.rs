//! Mode Concentration : choix de la carte à renforcer et de son paiement (spec 025) ; Effet maximum
//! y choisit sa cible de la même façon (spec 056).

use super::*;

/// Étape du mode Concentration renforcée (spec 025).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BoostStage {
    Target,
    Discard,
    Dice,
    Mana,
}

/// Une action qui prolonge le mode : son index, la carte visée et ses choix, le paiement de la
/// carte (aucun pour Effet maximum de base).
pub(super) type BoostCandidate = (usize, BoostTarget, Option<ManaPayment>);

/// Mode Concentration en cours : le choix, les actions qui le prolongent et l'étape.
pub(super) type Boost = (BoostChoice, Vec<BoostCandidate>, BoostStage);

/// Les actions qui prolongent `choice` : Concentration renforcée ou Effet maximum (spec 056) avec la
/// carte, l'option, la défausse et les dés déjà choisis (dans n'importe quel ordre), avec leur index
/// et leur paiement.
pub(super) fn boost_candidates(
    legal_actions: &[Action],
    choice: &BoostChoice,
) -> Vec<BoostCandidate> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            let Action::PlayCard { card, play } = action else {
                return None;
            };
            let (option, target, mana) = match (play, choice.basic) {
                (
                    CardPlay::Strong {
                        option,
                        mana,
                        with: Some(target),
                        ..
                    },
                    false,
                ) => (option, target, Some(*mana)),
                (
                    CardPlay::Basic {
                        option,
                        with: Some(target),
                        ..
                    },
                    true,
                ) => (option, target, None),
                _ => return None,
            };
            (*card == choice.card
                && *option == choice.option
                && choice.target.is_none_or(|chosen| {
                    chosen == target.card && choice.target_option == target.option
                })
                && choice
                    .discard
                    .is_none_or(|chosen| target.discard == Some(chosen))
                && choice
                    .settings
                    .iter()
                    .flatten()
                    .all(|setting| target.set_dice.contains(&Some(*setting))))
            .then_some((index, *target, mana))
        })
        .collect()
}

/// Le prochain choix à faire : la carte, sa défausse, ses dés, puis le mana.
pub(super) fn boost_stage(choice: &BoostChoice, candidates: &[BoostCandidate]) -> BoostStage {
    let chosen = choice.settings.iter().flatten().count();
    if choice.target.is_none() {
        BoostStage::Target
    } else if choice.discard.is_none()
        && candidates
            .iter()
            .any(|(_, target, _)| target.discard.is_some())
    {
        BoostStage::Discard
    } else if candidates
        .iter()
        .any(|(_, target, _)| target.set_dice.iter().flatten().count() > chosen)
    {
        BoostStage::Dice
    } else {
        BoostStage::Mana
    }
}

/// Les paiements distincts de Concentration pour les actions qui prolongent `choice`.
pub(super) fn boost_payments(
    legal_actions: &[Action],
    source: &[ManaColor],
    choice: &BoostChoice,
) -> Vec<(PaymentKind, ManaPayment, usize)> {
    let mut found = Vec::new();
    for (index, _, mana) in boost_candidates(legal_actions, choice) {
        if let Some(mana) = mana {
            add_payment(&mut found, source, mana, index);
        }
    }
    found
}

/// Le bouton qui mène à `next` : il joue l'action quand il ne reste qu'un paiement à choisir.
pub(super) fn boost_command(
    legal_actions: &[Action],
    source: &[ManaColor],
    next: BoostChoice,
) -> Command {
    let candidates = boost_candidates(legal_actions, &next);
    if boost_stage(&next, &candidates) == BoostStage::Mana {
        // Spec 056 : Effet maximum de base ne se paie pas, sa cible choisie le joue.
        if let [(only, _, None)] = candidates.as_slice() {
            return Command::Play(*only);
        }
        if let [(_, _, only)] = boost_payments(legal_actions, source, &next).as_slice() {
            return Command::Play(*only);
        }
    }
    Command::ChooseBoost(next)
}

/// L'effet de la carte qui mène le mode : `Boost` (Concentration) ou `Repeat` (Effet maximum).
fn leading_effect(choice: &BoostChoice) -> Option<Effect> {
    option_effect_of(choice.card, choice.option, !choice.basic)
}

/// Le bonus de l'option de Concentration renforcée.
pub(super) fn boost_bonus(choice: &BoostChoice) -> u8 {
    match leading_effect(choice) {
        Some(Effect::Boost(bonus)) => bonus,
        _ => 0,
    }
}

/// L'effet de l'option `option` de `card` tel que le mode le joue : renforcé et augmenté par
/// Concentration, ou répété par Effet maximum (de base ou renforcé, spec 056).
pub(super) fn boosted_effect(choice: &BoostChoice, card: CardId, option: u8) -> Option<Effect> {
    let effect = option_effect_of(card, option, !choice.basic)?;
    Some(match leading_effect(choice) {
        Some(Effect::Repeat(times)) => effect.repeated(times),
        _ => effect.with_bonus(boost_bonus(choice)),
    })
}

/// Le texte de l'effet de `card` tel que le mode le joue.
pub(super) fn boosted_effect_text(
    choice: &BoostChoice,
    card: CardId,
    option: u8,
    catalog: &Catalog,
) -> String {
    boosted_effect(choice, card, option)
        .map(|effect| effect_text(effect, catalog))
        .unwrap_or_default()
}

/// Carte de la main en mode Concentration : ses options renforcées quand on choisit la carte,
/// « Défausser » quand on choisit la défausse ; rien sinon.
pub(super) fn boost_hand_card(
    card: CardId,
    view: &PlayerView,
    legal_actions: &[Action],
    boost: Option<&Boost>,
    catalog: &Catalog,
) -> HandCard {
    let mut held = idle_card(card, catalog);
    let Some((choice, candidates, stage)) = boost else {
        return held;
    };
    match stage {
        BoostStage::Target => {
            let mut options: Vec<u8> = Vec::new();
            for (_, target, _) in candidates {
                if target.card == card && !options.contains(&target.option) {
                    options.push(target.option);
                }
            }
            // Spec 073 (C19 bis) : dans le bloc renforcé, gratuites, bonus compris ; Effet maximum
            // de base, dans le bloc de base (spec 056).
            if !options.is_empty() {
                let buttons = options
                    .into_iter()
                    .map(|option| {
                        let boosted = boosted_effect(choice, card, option);
                        EffectButton {
                            condition: None,
                            face: boosted.map_or_else(
                                || EffectFace::Text(String::new()),
                                |effect| effect_face(effect, catalog),
                            ),
                            mark: None,
                            color: None,
                            command: Some(boost_command(
                                legal_actions,
                                &view.source,
                                BoostChoice {
                                    target: Some(card),
                                    target_option: option,
                                    ..*choice
                                },
                            )),
                            tooltip: catalog.format(
                                "concentration.button",
                                &[
                                    ("card", &card_name(choice.card, catalog)),
                                    (
                                        "effect",
                                        &boosted_effect_text(choice, card, option, catalog),
                                    ),
                                ],
                            ),
                        }
                    })
                    .collect();
                if choice.basic {
                    held.basic_effects = buttons;
                } else {
                    held.strong_effects = buttons;
                }
            }
        }
        BoostStage::Discard => {
            if candidates
                .iter()
                .any(|(_, target, _)| target.discard == Some(card))
            {
                held.plays.push(Button {
                    label: catalog.text("discard_choice.pick"),
                    command: boost_command(
                        legal_actions,
                        &view.source,
                        BoostChoice {
                            discard: Some(card),
                            ..*choice
                        },
                    ),
                    shortcut: None,
                });
            }
        }
        BoostStage::Dice | BoostStage::Mana => {}
    }
    // Spec 073 : le bouton du choix remplace les boutons d'effet grisés de la carte.
    if !held.plays.is_empty() {
        held.basic_effects.clear();
        held.strong_effects.clear();
    }
    held
}

/// Les bandeaux du mode Concentration, selon l'étape, et les dés cliquables de la Source.
pub(super) struct BoostPrompts {
    pub(super) target: Option<DiscardPrompt>,
    pub(super) discard: Option<DiscardPrompt>,
    pub(super) set_dice: Option<SetDicePrompt>,
    pub(super) mana: Option<ManaPrompt>,
    pub(super) picks: Vec<(u8, Command)>,
}

pub(super) fn boost_prompts(
    view: &PlayerView,
    legal_actions: &[Action],
    (choice, candidates, stage): &Boost,
    catalog: &Catalog,
) -> BoostPrompts {
    let cancel = || Button {
        label: catalog.text("mana_choice.cancel"),
        command: Command::CancelChoice,
        shortcut: shortcut_label(Shortcut::Cancel, catalog),
    };
    let mut prompts = BoostPrompts {
        target: None,
        discard: None,
        set_dice: None,
        mana: None,
        picks: Vec::new(),
    };
    match stage {
        BoostStage::Target => {
            let text = match leading_effect(choice) {
                Some(Effect::Repeat(times)) => catalog.format(
                    "concentration.maximal_prompt",
                    &[
                        ("card", &card_name(choice.card, catalog)),
                        ("times", &times.to_string()),
                    ],
                ),
                _ => catalog.format(
                    "concentration.prompt",
                    &[
                        ("card", &card_name(choice.card, catalog)),
                        ("amount", &boost_bonus(choice).to_string()),
                    ],
                ),
            };
            prompts.target = Some(DiscardPrompt {
                text,
                cancel: cancel(),
            });
        }
        BoostStage::Discard => {
            if let Some(target) = choice.target {
                prompts.discard = Some(DiscardPrompt {
                    text: catalog.format(
                        "discard_choice.prompt",
                        &[
                            ("card", &card_name(target, catalog)),
                            (
                                "effect",
                                &boosted_effect_text(choice, target, choice.target_option, catalog),
                            ),
                        ],
                    ),
                    cancel: Button {
                        label: catalog.text("discard_choice.cancel"),
                        command: Command::CancelChoice,
                        shortcut: shortcut_label(Shortcut::Cancel, catalog),
                    },
                });
            }
        }
        BoostStage::Dice => {
            let colors = match choice.die {
                None => {
                    let mut dice: Vec<u8> = candidates
                        .iter()
                        .flat_map(|(_, target, _)| {
                            target.set_dice.iter().flatten().map(|setting| setting.die)
                        })
                        .filter(|die| {
                            !choice
                                .settings
                                .iter()
                                .flatten()
                                .any(|setting| setting.die == *die)
                        })
                        .collect();
                    dice.sort_unstable();
                    dice.dedup();
                    prompts.picks = dice
                        .into_iter()
                        .map(|die| {
                            (
                                die,
                                Command::ChooseBoost(BoostChoice {
                                    die: Some(die),
                                    ..*choice
                                }),
                            )
                        })
                        .collect();
                    Vec::new()
                }
                Some(die) => BasicColor::ALL
                    .into_iter()
                    .filter(|&color| {
                        let setting = Some(DieSetting { die, color });
                        candidates
                            .iter()
                            .any(|(_, target, _)| target.set_dice.contains(&setting))
                    })
                    .map(|color| {
                        let mut settings = choice.settings;
                        if let Some(slot) = settings.iter_mut().find(|slot| slot.is_none()) {
                            *slot = Some(DieSetting { die, color });
                        }
                        StrongButton {
                            label: color_name(color, catalog),
                            color,
                            command: boost_command(
                                legal_actions,
                                &view.source,
                                BoostChoice {
                                    settings,
                                    die: None,
                                    ..*choice
                                },
                            ),
                            enabled: true,
                            tooltip: None,
                        }
                    })
                    .collect(),
            };
            prompts.set_dice = Some(SetDicePrompt {
                text: catalog.text("set_die.prompt"),
                colors,
                cancel: cancel(),
            });
        }
        BoostStage::Mana => {
            let mana = ManaChoice {
                card: choice.card,
                option: choice.option,
                discard: None,
            };
            let found = boost_payments(legal_actions, &view.source, choice);
            prompts.mana = Some(mana_prompt(view, &mana, true, found, catalog));
        }
    }
    prompts
}
