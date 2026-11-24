//! Ruines du panneau latéral (spec 045) : l'offrande à l'autel et sa fenêtre, la récompense du
//! jeton, le trajet arrêté par une ruine révélée. Les textes de la récompense et des couleurs à
//! tirer servent aussi à l'infobulle du plateau et à la confirmation de l'entrée.

use arcanum_core::{EnemyColor, RuinReward, RuinToken, Site};

use super::*;
use crate::board::pile_color_key;

/// Le jeton de ruines face visible de cette case, s'il y en a un.
pub(crate) fn ruin_token_at(view: &PlayerView, hex: Hex) -> Option<RuinToken> {
    view.ruins.get(&hex).copied().flatten()
}

/// « gris, violet » : les couleurs des piles d'ennemis à tirer.
pub(crate) fn enemy_colors_text(colors: &[EnemyColor], catalog: &Catalog) -> String {
    colors
        .iter()
        .map(|&color| catalog.text(pile_color_key(color)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// « un sort, un cristal de chaque couleur » : la récompense d'un jeton d'ennemis.
pub(crate) fn token_reward_text(token: RuinToken, catalog: &Catalog) -> String {
    let rewards = token.definition().rewards();
    let count = |wanted: RuinReward| {
        u8::try_from(rewards.iter().filter(|&&reward| reward == wanted).count()).unwrap_or(u8::MAX)
    };
    reward_list(
        RewardCounts {
            artifacts: count(RuinReward::Artifact),
            spells: count(RuinReward::Spell),
            advanced_actions: count(RuinReward::AdvancedAction),
            crystal_sets: count(RuinReward::CrystalSet),
            units: count(RuinReward::Unit),
        },
        catalog,
    )
    .join(", ")
}

/// Récompenses d'un jeton, ou en attente, par sorte.
struct RewardCounts {
    artifacts: u8,
    spells: u8,
    advanced_actions: u8,
    crystal_sets: u8,
    units: u8,
}

/// Une entrée par récompense, dans l'ordre de la fin du tour (MKUE p. 11, 5) ; plusieurs artefacts
/// se comptent : « 2 artefact(s) ».
fn reward_list(counts: RewardCounts, catalog: &Catalog) -> Vec<String> {
    let mut list = Vec::new();
    match counts.artifacts {
        0 => {}
        1 => list.push(catalog.text("ruins.reward.artifact")),
        count => {
            list.push(catalog.format("reward.artifacts_count", &[("count", &count.to_string())]))
        }
    }
    let mut each = |count: u8, key: &str| {
        list.extend((0..count).map(|_| catalog.text(key)));
    };
    each(counts.spells, "ruins.reward.spell");
    each(counts.advanced_actions, "ruins.reward.advanced_action");
    each(counts.crystal_sets, "ruins.reward.crystal_set");
    each(counts.units, "ruins.reward.unit");
    list
}

/// La figurine est sur une ruine conquise : ses récompenses en attente sont celles du jeton
/// (une seule ruine conquise par tour).
fn on_conquered_ruins(view: &PlayerView) -> bool {
    view.figure.is_some_and(|hex| {
        view.shields.contains_key(&hex) && adventure::site_at(view, hex) == Some(Site::Ruins)
    })
}

/// « Récompense en fin de tour : un artefact, une action avancée », après la conquête d'une ruine,
/// à la place des lignes des artefacts et des sorts (C37).
pub(super) fn ruin_reward_pending(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    if !on_conquered_ruins(view) {
        return None;
    }
    let list = reward_list(
        RewardCounts {
            artifacts: view.artifact_rewards,
            spells: view.spell_rewards,
            advanced_actions: view.advanced_action_rewards,
            crystal_sets: view.crystal_set_rewards,
            units: view.unit_rewards,
        },
        catalog,
    );
    (!list.is_empty())
        .then(|| catalog.format("reward.pending_ruins", &[("list", &list.join(", "))]))
}

/// Les lignes séparées des artefacts et des sorts cèdent la place à celle des ruines.
pub(super) fn replaced_by_ruin_rewards(view: &PlayerView) -> bool {
    on_conquered_ruins(view)
}

/// L'offrande : son bouton, actif ou grisé, et sa ligne (points perdus ou mana qui manque), ou sa
/// fenêtre (C33, C34 ; Q1).
pub(super) struct AltarLines {
    pub button: Option<PurchaseButton>,
    pub warning: Option<String>,
    pub window: Option<AltarWindow>,
}

pub(super) fn altar_lines(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> AltarLines {
    let none = AltarLines {
        button: None,
        warning: None,
        window: None,
    };
    let Some(color) = view
        .figure
        .and_then(|hex| ruin_token_at(view, hex))
        .and_then(|token| token.definition().altar())
    else {
        return none;
    };
    let tributes: Vec<(usize, &[ManaPayment])> = legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::PayAltarTribute { mana } => Some((index, mana.as_slice())),
            _ => None,
        })
        .collect();
    let color_text = color_name(color, catalog);
    match mode {
        ScreenMode::PayingTribute if !tributes.is_empty() => AltarLines {
            window: Some(AltarWindow {
                title: catalog.format("ruins.altar.window", &[("color", &color_text)]),
                payments: tributes
                    .iter()
                    .map(|&(index, mana)| Button {
                        label: payment_text(view, color, mana, catalog),
                        command: Command::Play(index),
                        // Spec 071, C22 : 1 à 9, comme la touche.
                        shortcut: crate::take_label_of(
                            legal_actions,
                            index,
                            |action| matches!(action, Action::PayAltarTribute { .. }),
                            catalog,
                        ),
                    })
                    .collect(),
                note: catalog.text("ruins.altar.no_undo"),
                cancel: Button {
                    label: catalog.text("adventure.cancel"),
                    command: Command::CancelChoice,
                    shortcut: shortcut_label(Shortcut::Cancel, catalog),
                },
            }),
            ..none
        },
        // Hors d'un tour qui permet encore l'offrande (action du tour prise ou impossible,
        // déplacement terminé, partie finie), pas de bouton : le grisé ne dit que le mana qui
        // manque (issue privée 141).
        ScreenMode::Normal
            if !legal_actions.is_empty() && !view.turn_action_taken && !view.movement_ended =>
        {
            let enabled = !tributes.is_empty();
            AltarLines {
                button: Some(PurchaseButton {
                    label: catalog.text("ruins.altar.tribute"),
                    command: enabled.then_some(Command::OpenAltar),
                    tooltip: None,
                    // Spec 071, C21 : E, comme « Entrer dans ».
                    shortcut: shortcut_label(Shortcut::EnterSite, catalog),
                }),
                warning: if enabled {
                    (view.move_points > 0 || view.influence_points > 0).then(|| {
                        catalog.format(
                            "adventure.lost_points",
                            &[
                                ("move", &view.move_points.to_string()),
                                ("influence", &with_minus(view.influence_points)),
                            ],
                        )
                    })
                } else {
                    Some(catalog.format("ruins.altar.needs", &[("color", &color_text)]))
                },
                window: None,
            }
        }
        _ => none,
    }
}

/// « dé bleu, cristal bleu, jeton bleu » : un dé de la couleur de sa face, un cristal de la couleur
/// de l'autel, un jeton de cette couleur d'abord, puis or (comme le moteur, `token_paying`).
fn payment_text(
    view: &PlayerView,
    color: BasicColor,
    mana: &[ManaPayment],
    catalog: &Catalog,
) -> String {
    let own_tokens = view
        .mana_tokens
        .iter()
        .filter(|&&token| token == ManaColor::Basic(color))
        .count();
    let mut tokens = 0;
    mana.iter()
        .map(|payment| match *payment {
            ManaPayment::Die(die) => {
                let face = view
                    .source
                    .get(usize::from(die))
                    .copied()
                    .unwrap_or(ManaColor::Basic(color));
                catalog.format("ruins.payment.die", &[("color", &mana_name(face, catalog))])
            }
            ManaPayment::Crystal => catalog.format(
                "ruins.payment.crystal",
                &[("color", &color_name(color, catalog))],
            ),
            ManaPayment::Token => {
                let token = if tokens < own_tokens {
                    ManaColor::Basic(color)
                } else {
                    ManaColor::Gold
                };
                tokens += 1;
                catalog.format(
                    "ruins.payment.token",
                    &[("color", &mana_name(token, catalog))],
                )
            }
            // Spec 055 : l'offrande ne se paie pas en mana illimité.
            ManaPayment::Endless(face) => mana_name(face, catalog),
            // Spec 058 : « Invocation : défausser … », « Polarisation : … comme … ».
            ManaPayment::Invocation(_) | ManaPayment::Polarized { .. } | ManaPayment::StolenDie => {
                super::mana::kind_of(&view.source, *payment)
                    .and_then(|kind| {
                        super::mana::skill_payment_text(
                            view,
                            kind,
                            ManaColor::Basic(color),
                            catalog,
                        )
                    })
                    .unwrap_or_else(|| mana_name(ManaColor::Basic(color), catalog))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// « Ruines révélées : le trajet s'arrête », jusqu'à l'action suivante (C40).
pub(super) fn revealed_ruins_line(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    (!view.revealed_ruins.is_empty()).then(|| catalog.text("ruins.revealed"))
}
