//! Fenêtre Offres (spec 050) : les trois offres et le choix de l'action avancée d'un niveau pair.

use arcanum_core::{CommonCard, Offer, UnitCard, UnitRank};

use super::*;
use crate::enemy_text::resistance_key;

/// La fenêtre Offres, déjà traduite : consultation seulement, toujours fermable (spec #20, C18).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OffersWindow {
    /// « Offres ».
    pub title: String,
    pub close: Button,
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
    /// Le titre de la séparation, comme en main (spec #32) : « Renforcé », le nom renforcé d'un
    /// sort, « si détruit » pour un artefact ; vide pour une unité.
    pub strong_heading: String,
    /// Les pastilles de la séparation (spec #32) ; vides pour un artefact et une unité.
    pub strong_colors: Vec<ManaColor>,
}

/// Calculée depuis la vue et les actions légales (les effets du tour qui agissent sur l'offre).
pub fn offers_window(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> OffersWindow {
    offers_window_in(view, legal_actions, &ScreenMode::Normal, catalog)
}

/// La fenêtre Offres dans ce mode de l'écran. Ses seuls boutons sont ceux des effets du tour qui
/// agissent sur l'offre (spec #20, Q2) : « Apprendre » (Apprentissage, spec 056), « Lancer » (Don
/// pour la magie, spec 054), « Utiliser : … » et « Recruter (gratuit) » (spec 057) ; les choix de
/// carte ont leur fenêtre.
pub fn offers_window_in(
    view: &PlayerView,
    legal_actions: &[Action],
    _mode: &ScreenMode,
    catalog: &Catalog,
) -> OffersWindow {
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
                let mut shown = match card {
                    CommonCard::Deed(deed) => super::hand::window_card(deed, catalog),
                    CommonCard::Unit(unit) => unit_card(unit, catalog),
                };
                if let Some(learn) = learn_button(card, view, legal_actions, catalog) {
                    shown.take = Some(learn);
                } else if let Some(cast) =
                    super::spells::cast_offer_button(card, view, legal_actions, catalog)
                {
                    // Spec 054 : un sort de l'offre lancé avec Don pour la magie.
                    shown.take = Some(cast);
                }
                // Spec 057 : Appel aux armes et Appel à la gloire.
                if let CommonCard::Unit(unit) = card {
                    shown.uses = offer_unit_uses(unit, legal_actions, catalog);
                    if let Some(recruit) =
                        free_recruit_take(unit, &view_modifiers, legal_actions, catalog)
                    {
                        shown.take = Some(recruit);
                    }
                }
                // Spec 041 : une action avancée dans l'offre d'unités vient d'un monastère.
                if offer == Offer::Units && matches!(card, CommonCard::Deed(_)) {
                    shown.mark = Some(catalog.text("offers.monastery_card"));
                }
                shown
            })
            .collect(),
    };
    OffersWindow {
        title: catalog.text("offers.title"),
        close: Button {
            label: catalog.text("offers.close"),
            command: Command::CloseOffers,
            // Spec 071, C23 : Échap.
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
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
pub(super) fn unit_reward_take(
    unit: UnitCard,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<Button> {
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

pub(super) fn offer_card(
    card: CommonCard,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> OfferCardView {
    match card {
        CommonCard::Deed(deed) => {
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
                take,
                ..super::hand::window_card(deed, catalog)
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
        strong_heading: String::new(),
        strong_colors: Vec::new(),
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
