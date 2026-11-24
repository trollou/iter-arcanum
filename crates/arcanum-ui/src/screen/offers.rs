//! Fenêtre Offres (spec 050) : les trois offres et le choix de l'action avancée d'un niveau pair.

use arcanum_core::{CommonCard, DeedCard, Offer, UnitCard, UnitRank};

use super::*;
use crate::enemy_text::resistance_key;

/// La fenêtre Offres, déjà traduite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OffersWindow {
    /// « Offres », ou pendant un choix « Niveau 2 : choisissez une action avancée » ou le titre
    /// d'une récompense (spec 045).
    pub title: String,
    /// Faux pendant le choix d'une action avancée : la fenêtre reste ouverte.
    pub closable: bool,
    pub close: Button,
    /// « Renoncer », pendant le choix de l'unité de récompense sans jeton de commandement libre
    /// (spec 045, C39).
    pub decline: Option<Button>,
    /// Actions avancées, sorts, unités ; chacune dans l'ordre de l'offre.
    pub columns: Vec<OfferColumn>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferColumn {
    pub title: String,
    pub cards: Vec<OfferCardView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferCardView {
    pub name: String,
    /// Couleur d'une action avancée ou d'un sort ; aucune pour une unité.
    pub color: Option<BasicColor>,
    /// Effets de base et renforcé, ou valeurs et capacités d'une unité.
    pub lines: Vec<String>,
    /// « Prendre », pendant le choix d'une action avancée.
    pub take: Option<Button>,
    /// « monastère » : une action avancée d'un monastère dans l'offre d'unités (spec 041).
    pub mark: Option<String>,
    /// L'illustration de la carte ou de l'unité (spec 072, C6).
    pub illustration: crate::Illustration,
    /// « Utiliser : … », une par capacité d'une unité de l'offre avec Appel aux armes (spec 057).
    pub uses: Vec<Button>,
}

/// Calculée depuis la vue et les actions légales (les « Prendre » pendant le choix).
pub fn offers_window(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> OffersWindow {
    offers_window_in(view, legal_actions, &ScreenMode::Normal, catalog)
}

/// La fenêtre Offres dans ce mode de l'écran : « Prendre » la carte d'Entraînement ou de Don pour
/// la magie, « Apprendre » une action avancée avec Apprentissage (spec 056).
pub fn offers_window_in(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> OffersWindow {
    // Spec 056 : la carte d'Entraînement ou de Don pour la magie ; la fenêtre se ferme en annulant.
    let gaining = super::gain::choosing_offer_card(mode, legal_actions);
    let choosing = view.advanced_action_choices > 0;
    // Spec 045 : le choix de l'unité de récompense vient après celui des actions avancées.
    let choosing_unit = !choosing && view.unit_choices > 0;
    let title = match choosing_level(view) {
        Some(level) if choosing => catalog.format(
            "offers.choose_advanced_action",
            &[("level", &level.to_string())],
        ),
        // Spec 045, C41 : sans niveau pair à servir, le choix vient d'une récompense, prise
        // avant les niveaux (C24).
        None if choosing => catalog.text("reward.choose_advanced_action"),
        _ if choosing_unit => catalog.text("reward.choose_unit"),
        _ => catalog.text("offers.title"),
    };
    let view_modifiers: Vec<ModifierKind> = view
        .modifiers
        .iter()
        .map(|modifier| modifier.kind)
        .collect();
    // Spec 070, C17 : sans la taille des pioches.
    let column = |offer: Offer, title: &str| OfferColumn {
        title: catalog.text(title),
        cards: view
            .offers
            .get(&offer)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .map(|&card| {
                let mut shown = offer_card(card, legal_actions, catalog);
                if gaining {
                    shown.take = super::gain::gain_take(card, view, legal_actions, mode, catalog);
                } else if let Some(learn) = learn_button(card, view, legal_actions, catalog) {
                    shown.take = Some(learn);
                } else if let Some(cast) =
                    super::spells::cast_offer_button(card, view, legal_actions, catalog)
                {
                    // Spec 054 : un sort de l'offre lancé avec Don pour la magie.
                    shown.take = Some(cast);
                }
                let mut view = shown;
                if let CommonCard::Unit(unit) = card
                    && choosing_unit
                {
                    view.take = unit_reward_take(unit, legal_actions, catalog);
                }
                // Spec 057 : Appel aux armes et Appel à la gloire.
                if let CommonCard::Unit(unit) = card
                    && !choosing_unit
                {
                    view.uses = offer_unit_uses(unit, legal_actions, catalog);
                    if let Some(recruit) =
                        free_recruit_take(unit, &view_modifiers, legal_actions, catalog)
                    {
                        view.take = Some(recruit);
                    }
                }
                // Spec 041 : une action avancée dans l'offre d'unités vient d'un monastère.
                if offer == Offer::Units && matches!(card, CommonCard::Deed(_)) {
                    view.mark = Some(catalog.text("offers.monastery_card"));
                }
                view
            })
            .collect(),
    };
    OffersWindow {
        title,
        closable: !choosing && !choosing_unit,
        close: Button {
            label: catalog.text("offers.close"),
            command: if gaining {
                Command::CancelChoice
            } else {
                Command::CloseOffers
            },
            // Spec 071, C23 : Échap, seulement quand la fenêtre se ferme.
            shortcut: (!choosing && !choosing_unit)
                .then(|| shortcut_label(Shortcut::Cancel, catalog))
                .flatten(),
        },
        decline: legal_actions
            .iter()
            .position(|action| *action == Action::DeclineUnitReward)
            .filter(|_| choosing_unit)
            .map(|index| Button {
                label: catalog.text("reward.decline_unit"),
                command: Command::Play(index),
                shortcut: None,
            }),
        columns: vec![
            column(Offer::AdvancedActions, "offers.advanced_actions"),
            column(Offer::Spells, "offers.spells"),
            column(Offer::Units, "offers.units"),
        ],
    }
}

/// « Apprendre (6 Influence) » une action avancée de l'offre avec Apprentissage (spec 056, C26,
/// C28), tant que l'achat est légal.
fn learn_button(
    card: CommonCard,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<Button> {
    let CommonCard::Deed(deed) = card else {
        return None;
    };
    let index = legal_actions
        .iter()
        .position(|action| *action == Action::LearnAdvancedAction { card: deed })?;
    let cost = view
        .modifiers
        .iter()
        .find_map(|modifier| match modifier.kind {
            ModifierKind::LearningDiscount { cost, .. } => Some(cost),
            _ => None,
        })?;
    Some(Button {
        label: catalog.format("offers.learn", &[("cost", &cost.to_string())]),
        command: Command::Play(index),
        shortcut: None,
    })
}

/// « Prendre » une unité de récompense (spec 045, C39) : aussitôt avec un jeton de commandement
/// libre, sinon le choix de l'unité à renvoyer.
fn unit_reward_take(unit: UnitCard, legal_actions: &[Action], catalog: &Catalog) -> Option<Button> {
    let free = Action::TakeUnitReward {
        unit,
        disband: None,
    };
    let command = match legal_actions.iter().position(|action| *action == free) {
        Some(index) => Command::Play(index),
        None => legal_actions
            .iter()
            .any(|action| {
                matches!(action, Action::TakeUnitReward { unit: u, disband: Some(_) } if *u == unit)
            })
            .then_some(Command::ChooseDisband(unit))?,
    };
    // Spec 071, C22 : la place de l'unité parmi celles proposées, comme la touche
    // (`GameSession::take_unit_reward`).
    let mut offered: Vec<UnitCard> = Vec::new();
    for action in legal_actions {
        if let Action::TakeUnitReward { unit, .. } = action
            && !offered.contains(unit)
        {
            offered.push(*unit);
        }
    }
    Some(Button {
        label: catalog.text("reward.take"),
        command,
        shortcut: offered
            .iter()
            .position(|other| *other == unit)
            .and_then(|place| crate::take_label(place + 1, catalog)),
    })
}

/// « Utiliser : Attaque à distance 2 », une par capacité d'une unité de l'offre activable avec
/// Appel aux armes (spec 057, C17) : une seule activation légale la joue, plusieurs ouvrent le
/// choix comme pour une unité possédée.
fn offer_unit_uses(unit: UnitCard, legal_actions: &[Action], catalog: &Catalog) -> Vec<Button> {
    let abilities = unit.unit.definition().abilities.len();
    (0..abilities)
        .filter_map(|index| {
            let ability = u8::try_from(index).ok()?;
            let found: Vec<usize> = legal_actions
                .iter()
                .enumerate()
                .filter(|(_, action)| {
                    matches!(action, Action::ActivateUnit { unit: activated, ability: chosen, .. }
                        if *activated == unit && *chosen == ability)
                })
                .map(|(index, _)| index)
                .collect();
            let command = match found.as_slice() {
                [] => return None,
                [only] => Command::Play(*only),
                _ => Command::ChooseUnitAbility {
                    unit,
                    ability,
                    option: None,
                },
            };
            Some(Button {
                label: catalog.format(
                    "offers.use",
                    &[("ability", &catalog.text(&unit.unit.ability_key(index)))],
                ),
                command,
                shortcut: None,
            })
        })
        .collect()
}

/// « Recruter (gratuit) » une unité de l'offre avec Appel à la gloire (spec 057, C18) : aussitôt
/// avec un jeton de commandement libre, sinon le choix de l'unité à renvoyer.
fn free_recruit_take(
    unit: UnitCard,
    modifiers: &[ModifierKind],
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<Button> {
    if !modifiers.contains(&ModifierKind::RecruitFromOffer) {
        return None;
    }
    let free = Action::RecruitUnit {
        unit,
        disband: None,
        bonds: false,
    };
    let command = match legal_actions.iter().position(|action| *action == free) {
        Some(index) => Command::Play(index),
        None => legal_actions
            .iter()
            .any(|action| {
                matches!(action, Action::RecruitUnit { unit: u, disband: Some(_), bonds: false } if *u == unit)
            })
            .then_some(Command::ChooseDisband(unit))?,
    };
    Some(Button {
        label: catalog.text("offers.recruit_free"),
        command,
        shortcut: None,
    })
}

/// Le niveau pair dont le choix est en cours : le premier des niveaux pairs gagnés à cette fin de
/// tour dont la carte n'est pas encore prise.
pub(super) fn choosing_level(view: &PlayerView) -> Option<u8> {
    let even: Vec<u8> = view
        .levels_gained
        .iter()
        .copied()
        .filter(|level| level % 2 == 0)
        .collect();
    let taken = even
        .len()
        .checked_sub(usize::from(view.advanced_action_choices))?;
    even.get(taken).copied()
}

fn offer_card(card: CommonCard, legal_actions: &[Action], catalog: &Catalog) -> OfferCardView {
    match card {
        CommonCard::Deed(deed) => {
            let (basic, strong) = card_effect_texts(deed, catalog);
            let name = match deed.card() {
                DeedCard::Spell(spell) => catalog.format(
                    "offers.spell_name",
                    &[
                        ("basic", &catalog.text(spell.label_key())),
                        ("strong", &catalog.text(spell.strong_name_key())),
                    ],
                ),
                _ => card_name(deed, catalog),
            };
            let take = legal_actions
                .iter()
                .position(|action| *action == Action::GainAdvancedAction { card: deed })
                .map(|index| Button {
                    label: catalog.text("offers.take"),
                    command: Command::Play(index),
                    // Spec 071, C22 : 1 à 3, comme la touche.
                    shortcut: crate::take_label_of(
                        legal_actions,
                        index,
                        |action| matches!(action, Action::GainAdvancedAction { .. }),
                        catalog,
                    ),
                });
            OfferCardView {
                name,
                color: deed.card().definition().map(|definition| definition.color),
                lines: vec![basic, strong],
                take,
                mark: None,
                illustration: crate::card_illustration(deed.card()),
                uses: Vec::new(),
            }
        }
        CommonCard::Unit(unit) => unit_card(unit, catalog),
    }
}

/// Une unité : régulière ou élite, niveau, coût, armure, résistances, lieux de recrutement, puis
/// ses capacités, un texte écrit par nous (spec 050, Q1).
pub(super) fn unit_card(card: UnitCard, catalog: &Catalog) -> OfferCardView {
    let definition = card.unit.definition();
    let join = |items: Vec<String>| items.join(", ");
    let rank = match definition.rank {
        UnitRank::Regular => catalog.text("unit.regular"),
        UnitRank::Elite => catalog.text("unit.elite"),
    };
    let level = catalog.format("unit.level", &[("level", roman(definition.level))]);
    let mut lines = vec![
        format!("{rank} · {level}"),
        catalog.format(
            "unit.cost",
            &[("influence", &definition.influence.to_string())],
        ),
        catalog.format("unit.armor", &[("armor", &definition.armor.to_string())]),
    ];
    if !definition.resistances.is_empty() {
        let list = join(
            definition
                .resistances
                .iter()
                .map(|&resistance| catalog.text(resistance_key(resistance)))
                .collect(),
        );
        lines.push(catalog.format("enemy.resistances", &[("list", &list)]));
    }
    let sites = join(
        definition
            .recruit
            .iter()
            .map(|site| catalog.text(site.label_key()))
            .collect(),
    );
    lines.push(catalog.format("unit.recruit_sites", &[("list", &sites)]));
    lines.extend(
        (0..definition.abilities.len())
            .map(|ability| catalog.text(&card.unit.ability_key(ability))),
    );
    OfferCardView {
        name: catalog.text(card.unit.label_key()),
        color: None,
        lines,
        take: None,
        mark: None,
        illustration: crate::unit_illustration(card.unit),
        uses: Vec::new(),
    }
}

/// Niveau d'unité, en chiffres romains comme sur la carte.
pub(super) fn roman(level: u8) -> &'static str {
    match level {
        1 => "I",
        2 => "II",
        3 => "III",
        _ => "IV",
    }
}
