//! Compétences (spec 051) : la section Compétences du panneau du bas et la fenêtre du choix de
//! compétence d'un niveau pair.

use arcanum_core::{
    SidewaysEffect, Skill, SkillEffect, SkillMana, SkillState, SkillTarget, SkillUsage, SkillView,
    UnitCard,
};

use super::*;

/// « Une fois par tour » ou « Une fois par manche » (MKUE p. 5, *Using Skills* 4) ; rien pour
/// une compétence permanente (second lot, spec 058).
fn usage_text(skill: Skill, catalog: &Catalog) -> String {
    match skill.definition().usage {
        SkillUsage::Turn => catalog.text("skill.once_a_turn"),
        SkillUsage::Round => catalog.text("skill.once_a_round"),
        SkillUsage::Permanent => String::new(),
    }
}

/// Nom, fréquence, « Hors combat » s'il y a lieu, puis le texte : l'infobulle de la ligne.
fn skill_tooltip(skill: Skill, catalog: &Catalog) -> String {
    let mut lines = vec![catalog.text(&skill.label_key()), usage_text(skill, catalog)];
    if skill.definition().out_of_combat {
        lines.push(catalog.text("skill.out_of_combat"));
    }
    lines.push(catalog.text(&skill.text_key()));
    lines.retain(|line| !line.is_empty());
    lines.join("\n")
}

/// L'état qui grise la compétence, `None` quand elle est prête, ou au centre avec son avantage à
/// prendre ce tour (spec 058, C37).
fn state_text(owned: &SkillView, view: &PlayerView, catalog: &Catalog) -> Option<String> {
    if owned.unsupported {
        return Some(catalog.text("effect.unsupported"));
    }
    match owned.state {
        SkillState::Flipped => Some(catalog.text("skill.flipped")),
        // Spec 058, C37 : posée au centre, son avantage au tour suivant.
        SkillState::InCenter
            if view.center_advantages.iter().any(|advantage| {
                advantage.skill == owned.skill && advantage.active && !advantage.used
            }) =>
        {
            None
        }
        SkillState::InCenter => Some(catalog.text("skill.in_center")),
        SkillState::UsedThisTurn => Some(catalog.text("skill.used_this_turn")),
        SkillState::Ready if owned.motivation_blocked => {
            Some(catalog.text("skill.motivation_blocked"))
        }
        SkillState::Ready => None,
    }
}

/// Les actions légales qui utilisent l'option `option` de `skill`, avec leur index.
fn uses(legal_actions: &[Action], skill: Skill, option: u8) -> Vec<(usize, Option<UnitCard>)> {
    targeted_uses(legal_actions, skill, option)
        .into_iter()
        .map(|(index, unit, _)| (index, unit))
        .collect()
}

/// Les actions légales `UseSkill` de cette option, avec l'unité et la cible (spec 058).
fn targeted_uses(
    legal_actions: &[Action],
    skill: Skill,
    option: u8,
) -> Vec<(usize, Option<UnitCard>, Option<SkillTarget>)> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::UseSkill {
                skill: used,
                option: chosen,
                unit,
                target,
            } if *used == skill && *chosen == option => Some((index, *unit, *target)),
            _ => None,
        })
        .collect()
}

/// Une carte jouée de côté avec une compétence (spec 058, groupe A ; avantage du Rituel).
#[derive(Clone, Copy)]
struct SidewaysUse {
    index: usize,
    card: CardId,
    sideways: SidewaysEffect,
    mana: Option<SkillMana>,
    effect: Effect,
}

/// Les jeux de côté légaux avec l'option `option` de `skill`.
fn sideways_uses(legal_actions: &[Action], skill: Skill, option: u8) -> Vec<SidewaysUse> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::PlayCard {
                card,
                play:
                    play @ CardPlay::SidewaysSkill {
                        sideways,
                        skill: used,
                        option: chosen,
                        mana,
                    },
            } if *used == skill && *chosen == option => {
                card.effect(*play).map(|effect| SidewaysUse {
                    index,
                    card: *card,
                    sideways: *sideways,
                    mana: *mana,
                    effect,
                })
            }
            _ => None,
        })
        .collect()
}

/// Les points d'un effet de côté : Mouvement, Influence, Attaque ou Parade.
fn sideways_amount(effect: Effect) -> Option<u8> {
    match effect {
        Effect::Move(amount) | Effect::Influence(amount) => Some(amount),
        Effect::Attack { amount, .. } | Effect::Block { amount, .. } => Some(amount),
        _ => None,
    }
}

/// « De côté : 2 », « De côté : 3 / 4 » : les valeurs des jeux de côté légaux ; sans jeu légal,
/// celles que peut donner l'option (issue privée 302) ; « Jouer de côté » s'il n'y en a aucune.
fn sideways_face(found: &[SidewaysUse], skill: Skill, option: u8, catalog: &Catalog) -> String {
    let mut values: Vec<u8> = found
        .iter()
        .filter_map(|played| sideways_amount(played.effect))
        .collect();
    values.sort_unstable();
    values.dedup();
    if values.is_empty() {
        values = skill.sideways_values(option);
    }
    if values.is_empty() {
        return catalog.text("play.sideways");
    }
    let values: Vec<String> = values.iter().map(u8::to_string).collect();
    catalog.format("skill.sideways", &[("value", &values.join(" / "))])
}

/// La commande d'une compétence qui accompagne une carte jouée de côté : la seule action légale se
/// joue, plusieurs font choisir la carte.
fn sideways_command(found: &[SidewaysUse], skill: Skill, option: u8) -> Option<Command> {
    match found {
        [] => None,
        [only] => Some(Command::Play(only.index)),
        _ => Some(Command::ChooseSkillTarget(SkillChoice {
            skill,
            option,
            card: None,
        })),
    }
}

/// « Avantage de Rituel de guérison » (spec 058, C37, C39).
fn advantage_text(skill: Skill, catalog: &Catalog) -> String {
    catalog.format(
        "skill.advantage",
        &[("skill", &catalog.text(&skill.label_key()))],
    )
}

/// Un bouton par option, avec l'effet du moment de la journée (spec 044, `combat_time`) : la
/// seule action légale se joue, plusieurs font choisir l'unité (Inspiration, C22) ou la cible
/// (spec 058). Une compétence de côté : un bouton par option, qui fait choisir la carte ; au
/// centre, l'avantage du tour suivant.
fn option_buttons(
    owned: &SkillView,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<EffectButton> {
    let skill = owned.skill;
    let in_center = owned.state == SkillState::InCenter;
    let text_button = |face: String, tooltip: String, command| EffectButton {
        condition: None,
        face: EffectFace::Text(face),
        mark: None,
        color: None,
        command,
        tooltip,
    };
    if skill.definition().effect == SkillEffect::Sideways {
        return (0..skill.sideways_options())
            .map(|option| {
                let found = sideways_uses(legal_actions, skill, option);
                text_button(
                    sideways_face(&found, skill, option, catalog),
                    catalog.text(&skill.text_key()),
                    sideways_command(&found, skill, option),
                )
            })
            .collect();
    }
    let mut buttons: Vec<EffectButton> = (0u8..)
        .zip(skill.definition().options())
        .filter_map(|(option, choice)| {
            let effect = choice.effect(view.combat_time_of_day);
            let found = targeted_uses(legal_actions, skill, option);
            let command = match found.as_slice() {
                [] => None,
                [(index, _, _)] => Some(Command::Play(*index)),
                _ if found.iter().any(|(_, unit, _)| unit.is_some()) => {
                    Some(Command::ChooseSkillUnit { skill, option })
                }
                _ => Some(Command::ChooseSkillTarget(SkillChoice {
                    skill,
                    option,
                    card: None,
                })),
            };
            // Spec 058, C39 : au centre, l'option ne sert plus que l'avantage du tour suivant.
            if in_center {
                let text = advantage_text(skill, catalog);
                return command.map(|command| text_button(text.clone(), text, Some(command)));
            }
            Some(EffectButton {
                condition: None,
                face: effect_face(effect, catalog),
                mark: None,
                color: None,
                command,
                tooltip: effect_text(effect, catalog),
            })
        })
        .collect();
    // Spec 058, C37 : l'avantage du Rituel de guérison, une Blessure jouée de côté.
    let found = sideways_uses(legal_actions, skill, 0);
    if !found.is_empty() {
        let text = advantage_text(skill, catalog);
        buttons.push(text_button(
            text.clone(),
            text,
            sideways_command(&found, skill, 0),
        ));
    }
    buttons
}

/// Le choix de cible en cours, tant qu'il mène à une action légale (spec 058).
pub(super) enum SkillTargeting {
    /// La carte à jouer de côté (groupe A, avantage du Rituel).
    Card(SkillChoice),
    /// La carte choisie : le mana de Pouvoir universel.
    Mana(SkillChoice, CardId, SidewaysEffect),
    /// Une case, un ennemi, une couleur ou un dé.
    Target(SkillChoice),
}

pub(super) fn skill_targeting(
    mode: &ScreenMode,
    legal_actions: &[Action],
) -> Option<SkillTargeting> {
    let ScreenMode::ChoosingSkillTarget(choice) = *mode else {
        return None;
    };
    let sideways = sideways_uses(legal_actions, choice.skill, choice.option);
    match choice.card {
        Some((card, effect)) => sideways
            .iter()
            .any(|played| played.card == card && played.sideways == effect)
            .then_some(SkillTargeting::Mana(choice, card, effect)),
        None if !sideways.is_empty() => Some(SkillTargeting::Card(choice)),
        None => (!targeted_uses(legal_actions, choice.skill, choice.option).is_empty())
            .then_some(SkillTargeting::Target(choice)),
    }
}

/// Le bandeau du choix de la carte : « Flexibilité : choisir la carte à jouer de côté ».
pub(super) fn skill_card_prompt(
    targeting: Option<&SkillTargeting>,
    catalog: &Catalog,
) -> Option<DiscardPrompt> {
    let Some(SkillTargeting::Card(choice)) = targeting else {
        return None;
    };
    Some(DiscardPrompt {
        text: catalog.format(
            "skill.choose_card",
            &[("skill", &catalog.text(&choice.skill.label_key()))],
        ),
        cancel: cancel_button(catalog),
    })
}

fn cancel_button(catalog: &Catalog) -> Button {
    Button {
        label: catalog.text("mana_choice.cancel"),
        command: Command::CancelChoice,
        shortcut: shortcut_label(Shortcut::Cancel, catalog),
    }
}

/// Une carte de la main pendant le choix de la carte : « Jouer de côté : Mouvement 2 », un bouton
/// par effet de côté ; plusieurs manas possibles (Pouvoir universel) les font choisir ensuite.
pub(super) fn skill_hand_card(
    card: CardId,
    targeting: &SkillTargeting,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> HandCard {
    let mut plays: Vec<Button> = Vec::new();
    if let SkillTargeting::Card(choice) = targeting {
        let found: Vec<SidewaysUse> = sideways_uses(legal_actions, choice.skill, choice.option)
            .into_iter()
            .filter(|played| played.card == card)
            .collect();
        let mut kinds: Vec<SidewaysEffect> = Vec::new();
        for played in &found {
            if !kinds.contains(&played.sideways) {
                kinds.push(played.sideways);
            }
        }
        for sideways in kinds {
            let same: Vec<&SidewaysUse> = found
                .iter()
                .filter(|played| played.sideways == sideways)
                .collect();
            let mut effects: Vec<Effect> = same.iter().map(|played| played.effect).collect();
            effects.sort_by_key(|effect| sideways_amount(*effect));
            effects.dedup();
            let texts: Vec<String> = effects
                .iter()
                .map(|&effect| effect_text(effect, catalog))
                .collect();
            let effect = match texts.as_slice() {
                [first, .., last] => {
                    catalog.format("effect.or", &[("first", first), ("second", last)])
                }
                [only] => only.clone(),
                [] => String::new(),
            };
            plays.push(Button {
                label: catalog.format("play.sideways_effect", &[("effect", &effect)]),
                command: match same.as_slice() {
                    [only] => Command::Play(only.index),
                    _ => Command::ChooseSkillTarget(SkillChoice {
                        card: Some((card, sideways)),
                        ..*choice
                    }),
                },
                shortcut: None,
            });
        }
    }
    let idle = idle_card(card, catalog);
    // Spec 073 : en mode choix, les boutons de jeu remplacent les boutons d'effet.
    if plays.is_empty() {
        return idle;
    }
    HandCard {
        plays,
        basic_effects: Vec::new(),
        strong_effects: Vec::new(),
        ..idle
    }
}

/// Le bandeau du choix en cours : le mana de Pouvoir universel (« Jeton vert · Mouvement 4 »), ou
/// la cible (case, ennemi, couleur, dé) ; `None` pendant le choix de la carte.
pub(super) fn skill_target_prompt(
    view: &PlayerView,
    targeting: Option<&SkillTargeting>,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<ManaPrompt> {
    match targeting? {
        SkillTargeting::Card(_) => None,
        SkillTargeting::Mana(choice, card, sideways) => {
            let payments = sideways_uses(legal_actions, choice.skill, choice.option)
                .into_iter()
                .filter(|played| played.card == *card && played.sideways == *sideways)
                .map(|played| {
                    let effect = effect_text(played.effect, catalog);
                    let label = match played.mana {
                        Some(mana) => format!(
                            "{} · {effect}",
                            payment_label(view, mana.payment, mana.color, catalog)
                        ),
                        None => effect,
                    };
                    Button {
                        label,
                        command: Command::Play(played.index),
                        shortcut: None,
                    }
                })
                .fold(Vec::<Button>::new(), |mut buttons, button| {
                    // Deux dés de la même couleur paient pareil : un bouton suffit.
                    if !buttons.iter().any(|seen| seen.label == button.label) {
                        buttons.push(button);
                    }
                    buttons
                });
            Some(ManaPrompt {
                text: catalog.format(
                    "mana_choice.prompt_basic",
                    &[
                        ("card", &card_name(*card, catalog)),
                        ("effect", &catalog.text("play.sideways")),
                    ],
                ),
                payments,
                cancel: cancel_button(catalog),
            })
        }
        SkillTargeting::Target(choice) => {
            let found = targeted_uses(legal_actions, choice.skill, choice.option);
            let name = catalog.text(&choice.skill.label_key());
            let in_center = view
                .skills
                .iter()
                .any(|owned| owned.skill == choice.skill && owned.state == SkillState::InCenter);
            let mut key = "skill.choose_space";
            let mut payments = Vec::new();
            for (index, _, target) in &found {
                let label = match target {
                    Some(SkillTarget::Space(_)) => continue,
                    Some(SkillTarget::Enemy(enemy)) => {
                        key = "skill.choose_enemy";
                        catalog.text(enemy.enemy().label_key())
                    }
                    Some(SkillTarget::Color(color)) => {
                        key = "skill.choose_color";
                        mana_name(*color, catalog)
                    }
                    // Les dés se choisissent dans la Source.
                    Some(SkillTarget::Die(_)) => {
                        key = "skill.choose_die";
                        continue;
                    }
                    None => {
                        key = "skill.choose_die";
                        catalog.text("skill.no_reroll")
                    }
                };
                payments.push(Button {
                    label,
                    command: Command::Play(*index),
                    shortcut: None,
                });
            }
            // Au tour suivant : « Avantage de Embrasure de la Source : choisir le dé ».
            let name = if in_center {
                advantage_text(choice.skill, catalog)
            } else {
                name
            };
            let text = catalog.format(key, &[("skill", &name)]);
            Some(ManaPrompt {
                text,
                payments,
                cancel: cancel_button(catalog),
            })
        }
    }
}

/// Les cases d'arrivée du Vol, en évidence sur le plateau (spec 058, C22).
pub(super) fn skill_destinations(
    targeting: Option<&SkillTargeting>,
    legal_actions: &[Action],
) -> Vec<CastDestination> {
    let Some(SkillTargeting::Target(choice)) = targeting else {
        return Vec::new();
    };
    targeted_uses(legal_actions, choice.skill, choice.option)
        .into_iter()
        .filter_map(|(index, _, target)| match target {
            Some(SkillTarget::Space(hex)) => Some(CastDestination {
                hex,
                command: Command::Play(index),
            }),
            _ => None,
        })
        .collect()
}

/// Les dés de la Source cliquables : relancé par Embrasure de la Source, ou pris en plus avec son
/// avantage (spec 058, C39).
pub(super) fn skill_die_picks(
    targeting: Option<&SkillTargeting>,
    legal_actions: &[Action],
) -> Vec<(u8, Command)> {
    let Some(SkillTargeting::Target(choice)) = targeting else {
        return Vec::new();
    };
    targeted_uses(legal_actions, choice.skill, choice.option)
        .into_iter()
        .filter_map(|(index, _, target)| match target {
            Some(SkillTarget::Die(die)) => Some((die, Command::Play(index))),
            _ => None,
        })
        .collect()
}

/// « Garder ce dé tel quel », quand le dé pris avec l'avantage d'Embrasure de la Source peut ne
/// pas être relancé (spec 058, C39).
pub(super) fn keep_die_button(legal_actions: &[Action], catalog: &Catalog) -> Option<Button> {
    legal_actions
        .iter()
        .position(|action| *action == Action::KeepSourceDie)
        .map(|index| Button {
            label: catalog.text("skill.keep_die"),
            command: Command::Play(index),
            shortcut: None,
        })
}

/// La section Compétences, avec au moins une compétence gagnée (C26).
pub(super) fn skills_section(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<SkillsSection> {
    if view.skills.is_empty() {
        return None;
    }
    Some(SkillsSection {
        title: catalog.text("skill.title"),
        pile: catalog.format(
            "skill.pile",
            &[("count", &view.skill_pile_count.to_string())],
        ),
        skills: view
            .skills
            .iter()
            .map(|owned| SkillLine {
                skill: owned.skill,
                name: catalog.text(&owned.skill.label_key()),
                usage: usage_text(owned.skill, catalog),
                state: state_text(owned, view, catalog),
                options: option_buttons(owned, view, legal_actions, catalog),
                tooltip: skill_tooltip(owned.skill, catalog),
            })
            .collect(),
    })
}

/// L'option de compétence dont l'unité se choisit, tant qu'elle reste légale (C22).
pub(super) fn choosing_skill_unit(
    mode: &ScreenMode,
    legal_actions: &[Action],
) -> Option<(Skill, u8)> {
    match *mode {
        ScreenMode::ChoosingSkillUnit { skill, option }
            if !uses(legal_actions, skill, option).is_empty() =>
        {
            Some((skill, option))
        }
        _ => None,
    }
}

/// « Choisir » sur une unité que vise l'option de compétence en cours.
pub(super) fn skill_unit_button(
    mode: &ScreenMode,
    unit: UnitCard,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<Button> {
    let (skill, option) = choosing_skill_unit(mode, legal_actions)?;
    uses(legal_actions, skill, option)
        .into_iter()
        .find(|(_, target)| *target == Some(unit))
        .map(|(index, _)| Button {
            label: catalog.text("skill.pick_unit"),
            command: Command::Play(index),
            shortcut: None,
        })
}

/// Le bandeau du choix de l'unité : « Inspiration : choisir l'unité » et « Annuler » (Échap).
pub(super) fn skill_unit_prompt(
    mode: &ScreenMode,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<DiscardPrompt> {
    let (skill, _) = choosing_skill_unit(mode, legal_actions)?;
    Some(DiscardPrompt {
        text: catalog.format(
            "skill.choose_unit",
            &[("skill", &catalog.text(&skill.label_key()))],
        ),
        cancel: Button {
            label: catalog.text("discard_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    })
}

/// La fenêtre du niveau pair (spec #20, C1 à C8) : après les choix de l'artefact et du sort, tant
/// que la compétence ou l'action avancée d'un niveau pair attend.
pub(super) fn level_choice_window(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<LevelChoiceWindow> {
    let level = super::offers::choosing_level(view);
    let pending =
        !view.skill_choice.is_empty() || (view.advanced_action_choices > 0 && level.is_some());
    if !pending || !view.drawn_artifacts.is_empty() || view.spell_choices > 0 {
        return None;
    }
    let offer: Vec<CardId> = view
        .offers
        .get(&arcanum_core::Offer::AdvancedActions)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter_map(|card| card.deed())
        .collect();
    // L2 : l'encadré ne vaut que pour une compétence prenable.
    let selected = match mode {
        ScreenMode::ChoosingLevelSkill(skill) => view
            .skill_choice_cards
            .iter()
            .find(|(takeable, _)| takeable == skill),
        _ => None,
    };
    let skill_offer = |skill: Skill, own_note: Option<String>| {
        let takeable = view
            .skill_choice_cards
            .iter()
            .any(|(takeable, _)| *takeable == skill);
        // C7 : offre vide, la compétence se prend seule.
        let take = legal_actions
            .iter()
            .position(|action| *action == Action::ChooseSkill { skill })
            .filter(|_| offer.is_empty())
            .map(|index| Button {
                label: catalog.text("offers.take"),
                command: Command::Play(index),
                shortcut: None,
            });
        SkillOffer {
            skill,
            name: catalog.text(&skill.label_key()),
            usage: usage_text(skill, catalog),
            text: catalog.text(&skill.text_key()),
            selected: selected.is_some_and(|(chosen, _)| *chosen == skill),
            select: (takeable && !offer.is_empty()).then_some(Command::SelectSkill(skill)),
            take,
            note: (!takeable).then_some(own_note).flatten(),
        }
    };
    let choosing_skill = !view.skill_choice.is_empty();
    let common_note = catalog.text("skill.choice.common_note");
    let advanced_actions = offer
        .iter()
        .enumerate()
        .map(|(place, &card)| {
            let command = if choosing_skill {
                selected
                    .filter(|(_, cards)| cards.contains(&card))
                    .map(|_| Command::TakeLevelReward(card))
            } else {
                // La compétence du niveau déjà prise : la carte seule.
                legal_actions
                    .iter()
                    .position(|action| *action == Action::GainAdvancedAction { card })
                    .map(Command::Play)
            };
            // C3 : une compétence de l'offre commune impose la dernière place.
            let tooltip = (command.is_none()
                && selected.is_some_and(|(skill, _)| !view.skill_choice.contains(skill)))
            .then(|| common_note.clone());
            LevelCard {
                card: super::hand::window_card(card, catalog),
                take: PurchaseButton {
                    card: None,
                    label: catalog.text("offers.take"),
                    shortcut: command.and_then(|_| crate::take_label(place + 1, catalog)),
                    command,
                    tooltip,
                },
            }
        })
        .collect();
    Some(LevelChoiceWindow {
        title: catalog.format(
            "level_choice.title",
            &[("level", &level.unwrap_or(view.level).to_string())],
        ),
        revealed_title: catalog.text("skill.choice.revealed"),
        revealed: view
            .skill_choice
            .iter()
            .map(|&skill| skill_offer(skill, None))
            .collect(),
        common_title: catalog.text("skill.choice.common"),
        common_note,
        common: if choosing_skill {
            view.common_skills
                .iter()
                .map(|common| skill_offer(common.skill, Some(catalog.text("skill.choice.own"))))
                .collect()
        } else {
            Vec::new()
        },
        advanced_actions_title: catalog.text("offers.advanced_actions"),
        advanced_actions,
        hint: (choosing_skill && selected.is_none() && !offer.is_empty())
            .then(|| catalog.text("level_choice.skill_first")),
    })
}
