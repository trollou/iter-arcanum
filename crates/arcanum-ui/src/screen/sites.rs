//! Villages et monastères du panneau latéral (spec 041) : pillage, achats, incendie, récompense
//! artefact.

use arcanum_core::{
    BasicColor, CITY_ADVANCED_ACTION_COST, CITY_ARTIFACT_COST, CITY_ELITE_UNIT_COST, CommonCard,
    Interaction, MONASTERY_ADVANCED_ACTION_COST, MONASTERY_HEALING_COST, Offer, SPELL_COST, Site,
    VILLAGE_HEALING_COST,
};

use super::*;

/// Le bouton d'une action légale, s'il y en a une.
fn legal_button(legal_actions: &[Action], wanted: &Action, label: String) -> Option<Button> {
    legal_actions
        .iter()
        .position(|action| action == wanted)
        .map(|index| Button {
            label,
            command: Command::Play(index),
            shortcut: None,
        })
}

/// « Prendre » un artefact ou un sort tiré, avec sa touche 1 à 3 (spec 071, C22).
fn take_button(
    legal_actions: &[Action],
    wanted: &Action,
    same_kind: fn(&Action) -> bool,
    catalog: &Catalog,
) -> Option<Button> {
    legal_button(legal_actions, wanted, catalog.text("reward.take")).map(|button| Button {
        shortcut: match button.command {
            Command::Play(index) => crate::take_label_of(legal_actions, index, same_kind, catalog),
            _ => None,
        },
        ..button
    })
}

/// « Piller le village (2 cartes, Réputation −1) », au début du tour (Compendium, *Turn Start* S1).
pub(super) fn plunder_button(legal_actions: &[Action], catalog: &Catalog) -> Option<Button> {
    legal_button(
        legal_actions,
        &Action::PlunderVillage,
        catalog.text("plunder.button"),
    )
}

/// En interaction au village ou au monastère : au monastère, une ligne par action avancée de
/// l'offre d'unités (MKUE p. 8, 4.c), puis le point de soin (4.b). Un achat trop cher reste
/// visible, grisé.
pub(super) fn purchase_buttons(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<PurchaseButton> {
    let healing_cost = match view.interaction {
        Some(Interaction {
            site: Site::Village,
            ..
        }) => VILLAGE_HEALING_COST,
        Some(Interaction {
            site: Site::Monastery,
            ..
        }) => MONASTERY_HEALING_COST,
        Some(Interaction {
            site: Site::MageTower | Site::City(BasicColor::Blue),
            ..
        }) => return spell_purchases(view, legal_actions, catalog),
        Some(Interaction {
            site: Site::City(color),
            ..
        }) => return city_purchases(view, color, legal_actions, catalog),
        _ => return Vec::new(),
    };
    let command = |wanted: Action| {
        legal_actions
            .iter()
            .position(|action| *action == wanted)
            .map(Command::Play)
    };
    let mut buttons = Vec::new();
    if view
        .interaction
        .is_some_and(|interaction| interaction.site == Site::Monastery)
    {
        let deeds = view
            .offers
            .get(&Offer::Units)
            .into_iter()
            .flatten()
            .filter_map(|card| match card {
                CommonCard::Deed(deed) => Some(*deed),
                CommonCard::Unit(_) => None,
            });
        buttons.extend(deeds.map(|card| {
            let (basic, strong) = card_effect_texts(card, catalog);
            PurchaseButton {
                label: catalog.format(
                    "purchase.advanced_action",
                    &[
                        ("name", &card_name(card, catalog)),
                        ("cost", &MONASTERY_ADVANCED_ACTION_COST.to_string()),
                    ],
                ),
                command: command(Action::BuyAdvancedAction { card }),
                tooltip: Some(format!("{basic}\n{strong}")),
                shortcut: None,
            }
        }));
    }
    buttons.push(PurchaseButton {
        label: catalog.format("purchase.healing", &[("cost", &healing_cost.to_string())]),
        command: command(Action::BuyHealing),
        tooltip: None,
        shortcut: None,
    });
    buttons
}

/// Dans une cité conquise, les achats de sa couleur, sans soin (MKUE p. 8, 4.e ; Compendium,
/// *Cities* S21 ; spec 062, C40) : la verte, une ligne par action avancée de l'offre puis la carte
/// du dessus de leur pioche ; la rouge, un artefact ; la blanche, une unité d'élite ajoutée à
/// l'offre. Un achat trop cher reste visible, grisé.
fn city_purchases(
    view: &PlayerView,
    color: BasicColor,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<PurchaseButton> {
    let button = |label: String, wanted: Action, tooltip: Option<String>| PurchaseButton {
        label,
        command: legal_actions
            .iter()
            .position(|action| *action == wanted)
            .map(Command::Play),
        tooltip,
        shortcut: None,
    };
    let cost = |key: &str, cost: u8| catalog.format(key, &[("cost", &cost.to_string())]);
    match color {
        BasicColor::Green => {
            let offered = view
                .offers
                .get(&Offer::AdvancedActions)
                .into_iter()
                .flatten()
                .filter_map(|card| card.deed());
            let mut buttons: Vec<PurchaseButton> = offered
                .map(|card| {
                    let (basic, strong) = card_effect_texts(card, catalog);
                    button(
                        catalog.format(
                            "purchase.advanced_action",
                            &[
                                ("name", &card_name(card, catalog)),
                                ("cost", &CITY_ADVANCED_ACTION_COST.to_string()),
                            ],
                        ),
                        Action::BuyAdvancedAction { card },
                        Some(format!("{basic}\n{strong}")),
                    )
                })
                .collect();
            buttons.push(button(
                cost("purchase.advanced_action_deck", CITY_ADVANCED_ACTION_COST),
                Action::BuyAdvancedActionFromDeck,
                None,
            ));
            buttons
        }
        BasicColor::Red => vec![button(
            cost("purchase.artifact", CITY_ARTIFACT_COST),
            Action::BuyArtifact,
            None,
        )],
        BasicColor::White => vec![button(
            cost("purchase.elite_unit", CITY_ELITE_UNIT_COST),
            Action::AddEliteUnit,
            None,
        )],
        BasicColor::Blue => spell_purchases(view, legal_actions, catalog),
    }
}

/// « Boucliers dans la cité : Influence +3 », pendant l'interaction dans une cité (spec 062, C40).
pub(super) fn city_shields_line(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.interaction
        .filter(|interaction| interaction.city_shields > 0)
        .map(|interaction| {
            catalog.format(
                "interaction.city_shields",
                &[("bonus", &interaction.city_shields.to_string())],
            )
        })
}

/// L'incendie : son bouton et les points qu'il ferait perdre (MKUE p. 6, 6.e), ou sa
/// confirmation, qui dit son prix.
pub(super) struct BurnLines {
    pub button: Option<Button>,
    pub warning: Option<String>,
    pub confirm: Option<AnnouncePrompt>,
}

pub(super) fn burn_lines(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> BurnLines {
    let none = BurnLines {
        button: None,
        warning: None,
        confirm: None,
    };
    let Some(index) = legal_actions
        .iter()
        .position(|action| *action == Action::BurnMonastery)
    else {
        return none;
    };
    match mode {
        ScreenMode::ConfirmingBurn => BurnLines {
            confirm: Some(AnnouncePrompt {
                text: catalog.text("burn.confirm"),
                buttons: vec![
                    Button {
                        label: catalog.text("burn.confirm_yes"),
                        command: Command::Play(index),
                        shortcut: None,
                    },
                    Button {
                        label: catalog.text("burn.cancel"),
                        command: Command::CancelChoice,
                        shortcut: shortcut_label(Shortcut::Cancel, catalog),
                    },
                ],
            }),
            ..none
        },
        ScreenMode::Normal => BurnLines {
            button: Some(Button {
                label: catalog.text("burn.button"),
                command: Command::OpenBurn,
                shortcut: None,
            }),
            warning: (view.move_points > 0 || view.influence_points > 0).then(|| {
                catalog.format(
                    "burn.lost_points",
                    &[
                        ("move", &view.move_points.to_string()),
                        ("influence", &with_minus(view.influence_points)),
                    ],
                )
            }),
            confirm: None,
        },
        _ => none,
    }
}

/// « Récompense en fin de tour : 1 artefact(s) », tant que la récompense attend.
pub(super) fn reward_pending(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    (view.artifact_rewards > 0).then(|| {
        catalog.format(
            "reward.pending",
            &[("count", &view.artifact_rewards.to_string())],
        )
    })
}

/// La fenêtre du choix de l'artefact (MKUE p. 11, 5.b) : les artefacts tirés dans l'ordre du
/// tirage, nom et texte, « Prendre » sous chacun.
pub(super) fn artifact_choice(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<ArtifactChoice> {
    (!view.drawn_artifacts.is_empty()).then(|| ArtifactChoice {
        title: catalog.text("reward.choose_artifact"),
        cards: view
            .drawn_artifacts
            .iter()
            .map(|&card| {
                let (basic, strong) = card_effect_texts(card, catalog);
                OfferCardView {
                    name: card_name(card, catalog),
                    color: None,
                    lines: vec![basic, strong],
                    take: take_button(
                        legal_actions,
                        &Action::TakeArtifact { card },
                        |action| matches!(action, Action::TakeArtifact { .. }),
                        catalog,
                    ),
                    mark: None,
                    illustration: crate::card_illustration(card.card()),
                    uses: Vec::new(),
                }
            })
            .collect(),
    })
}

/// Les sorts de l'offre, dans l'ordre : leur couleur, et leur nom (spec 043).
fn offered_spells(view: &PlayerView) -> Vec<CardId> {
    view.offers
        .get(&Offer::Spells)
        .into_iter()
        .flatten()
        .filter_map(|card| match card {
            CommonCard::Deed(deed) => Some(*deed),
            CommonCard::Unit(_) => None,
        })
        .collect()
}

fn spell_color(card: CardId) -> Option<ManaColor> {
    card.card()
        .definition()
        .map(|definition| ManaColor::Basic(definition.color))
}

/// « Acheter : Éclair de feu (7 + mana rouge) ».
fn spell_label(card: CardId, catalog: &Catalog) -> String {
    catalog.format(
        "purchase.spell",
        &[
            ("name", &card_name(card, catalog)),
            ("cost", &SPELL_COST.to_string()),
            (
                "color",
                &spell_color(card)
                    .map(|color| mana_name(color, catalog))
                    .unwrap_or_default(),
            ),
        ],
    )
}

/// À une tour de mage conquise : un achat par sort de l'offre (MKUE p. 8, 4.d), grisé faute
/// d'influence ou de mana ; un clic ouvre le choix du mana (spec 043, C41).
fn spell_purchases(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<PurchaseButton> {
    offered_spells(view)
        .into_iter()
        .map(|card| {
            let (basic, strong) = card_effect_texts(card, catalog);
            let buyable = legal_actions
                .iter()
                .any(|action| matches!(action, Action::BuySpell { card: c, .. } if *c == card));
            PurchaseButton {
                label: spell_label(card, catalog),
                command: buyable.then_some(Command::ChooseSpellMana(card)),
                tooltip: Some(format!("{basic}\n{strong}")),
                shortcut: None,
            }
        })
        .collect()
}

/// Le choix du mana qui paie l'achat d'un sort : un bouton par paiement légal, comme pour un
/// effet renforcé (spec 043, C41) ; ignoré si l'achat n'est plus légal.
pub(super) fn spell_mana_prompt(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<ManaPrompt> {
    let ScreenMode::ChoosingSpellMana(card) = *mode else {
        return None;
    };
    let color = spell_color(card)?;
    let mut payments: Vec<Button> = Vec::new();
    for (index, action) in legal_actions.iter().enumerate() {
        let Action::BuySpell { card: bought, mana } = action else {
            continue;
        };
        if *bought != card {
            continue;
        }
        let label = payment_label(view, *mana, color, catalog);
        // Deux dés de la même couleur paient pareil : un bouton suffit.
        if !payments.iter().any(|button| button.label == label) {
            payments.push(Button {
                label,
                command: Command::Play(index),
                shortcut: None,
            });
        }
    }
    (!payments.is_empty()).then(|| ManaPrompt {
        text: spell_label(card, catalog),
        payments,
        cancel: Button {
            label: catalog.text("mana_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    })
}

/// « Récompense en fin de tour : 1 sort(s) », tant que la récompense attend (spec 043, C45).
pub(super) fn spell_reward_pending(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    (view.spell_rewards > 0).then(|| {
        catalog.format(
            "reward.pending_spell",
            &[("count", &view.spell_rewards.to_string())],
        )
    })
}

/// La fenêtre du choix du sort (MKUE p. 11, 5.c) : les sorts de l'offre, nom et texte,
/// « Prendre » sous chacun (spec 043, C43).
pub(super) fn spell_choice(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<ArtifactChoice> {
    (view.spell_choices > 0).then(|| ArtifactChoice {
        title: catalog.text("reward.choose_spell"),
        cards: offered_spells(view)
            .into_iter()
            .map(|card| {
                let (basic, strong) = card_effect_texts(card, catalog);
                OfferCardView {
                    name: card_name(card, catalog),
                    color: None,
                    lines: vec![basic, strong],
                    take: take_button(
                        legal_actions,
                        &Action::TakeSpell { card },
                        |action| matches!(action, Action::TakeSpell { .. }),
                        catalog,
                    ),
                    mark: None,
                    illustration: crate::card_illustration(card.card()),
                    uses: Vec::new(),
                }
            })
            .collect(),
    })
}

/// « Assaut repoussé : retour sur la case de départ », après une retraite (MKUE p. 10, 3.d) :
/// l'assaut du tour reste, la figurine est revenue sur sa case de départ, le site n'est pas
/// conquis (spec 043, C40).
pub(super) fn withdrawal(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.assault
        .filter(|assault| {
            assault.from != assault.site
                && view.figure == Some(assault.from)
                && !view.shields.contains_key(&assault.site)
        })
        .map(|_| catalog.text("assault.withdrew"))
}

/// « Limite de main : 6 (+1 forteresse) », près d'une forteresse possédée (spec 043, C44) ;
/// « Limite de main : 7 (+2 cité) » quand le bonus d'une cité conquise est le plus grand (spec 062,
/// C41 : seul le plus grand compte).
pub(super) fn keep_hand_limit(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    let (key, bonus) = if view.city_hand_bonus > view.keep_hand_bonus {
        ("level.hand_limit_city", view.city_hand_bonus)
    } else {
        ("level.hand_limit_keep", view.keep_hand_bonus)
    };
    (bonus > 0).then(|| {
        catalog.format(
            key,
            &[
                ("limit", &(view.hand_limit + bonus).to_string()),
                ("bonus", &bonus.to_string()),
            ],
        )
    })
}
