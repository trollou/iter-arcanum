//! Panneau de droite et boutons du bas (spec 073) : en-tête du joueur, défausse, soins (la
//! section Ici est devenue les boutons du plateau, spec 075), faits du tour, boutons « Offres… »,
//! « Finir la manche… » et « Finir le tour… ».

use arcanum_core::{Feature, Site, TurnFact};

use super::*;

/// « Niveau 3 » (C23).
pub(super) fn player_level(view: &PlayerView, catalog: &Catalog) -> String {
    catalog.format("panel.level", &[("level", &view.level.to_string())])
}

/// L'icône Gloire et « 2 / 15 », ou « 2 » seul au dernier niveau (C23).
pub(super) fn player_fame(view: &PlayerView, catalog: &Catalog) -> crate::IconValue {
    let value = match view.next_level_fame {
        Some(next) => catalog.format(
            "panel.fame",
            &[
                ("fame", &view.fame.to_string()),
                ("next", &next.to_string()),
            ],
        ),
        None => view.fame.to_string(),
    };
    icon_value(Icon::Fame, value, catalog)
}

/// « Défausse : 2 carte(s) », qui ouvre la fenêtre de la défausse ; grisé sans carte (C24, C36).
pub(super) fn discard_button(view: &PlayerView, catalog: &Catalog) -> PurchaseButton {
    PurchaseButton {
        label: catalog.format(
            "discard.count",
            &[("count", &view.discard.len().to_string())],
        ),
        command: (!view.discard.is_empty()).then_some(Command::OpenDiscard),
        tooltip: None,
        shortcut: shortcut_label(Shortcut::Discard, catalog),
    }
}

/// Une ligne par fait du tour, dans l'ordre (C28), sauf les sorts lancés depuis l'offre, que la
/// zone de jeu montre (spec 054, issue privée 246).
pub(super) fn turn_fact_lines(view: &PlayerView, catalog: &Catalog) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    // Spec 055 : les dés d'un artefact en une ligne, à la place du premier, même quand ce qu'ils
    // donnent (gloire, cristal) s'intercale.
    let mut dice: Vec<(CardId, usize, Vec<String>)> = Vec::new();
    for fact in &view.turn_facts {
        match fact {
            TurnFact::SpellCastFromOffer { .. } => {}
            TurnFact::ArtifactDieRolled { card, color } => {
                let face = mana_name(*color, catalog);
                match dice.iter_mut().find(|(rolled, _, _)| rolled == card) {
                    Some((_, _, faces)) => faces.push(face),
                    None => {
                        dice.push((*card, lines.len(), vec![face]));
                        lines.push(String::new());
                    }
                }
            }
            _ => lines.push(turn_fact_line(fact, view, catalog)),
        }
    }
    for (_, at, faces) in dice {
        lines[at] = catalog.format("artifact_dice.rolled", &[("faces", &faces.join(", "))]);
    }
    lines
}

/// Les sorts lancés depuis l'offre ce tour, pour la zone de jeu (spec 054, issue privée 246).
pub(super) fn offer_casts(view: &PlayerView) -> impl Iterator<Item = &PlayedCard> {
    view.turn_facts.iter().filter_map(|fact| match fact {
        TurnFact::SpellCastFromOffer { played } => Some(played),
        _ => None,
    })
}

fn turn_fact_line(fact: &TurnFact, view: &PlayerView, catalog: &Catalog) -> String {
    let unit = |key: &str, unit: &UnitCard| {
        catalog.format(key, &[("unit", &catalog.text(unit.unit.label_key()))])
    };
    match fact {
        TurnFact::TileRevealed { .. } => catalog.text("turn_fact.explored"),
        TurnFact::EnemiesDefeated { enemies, fame } => {
            let names: Vec<String> = enemies
                .iter()
                .map(|token| catalog.text(token.enemy().label_key()))
                .collect();
            catalog.format(
                "turn_fact.defeated",
                &[("enemies", &names.join(", ")), ("fame", &fame.to_string())],
            )
        }
        TurnFact::FameGained { fame } => {
            catalog.format("turn_fact.fame", &[("fame", &fame.to_string())])
        }
        TurnFact::ReputationChanged { change } => catalog.format(
            "turn_fact.reputation",
            &[("change", &signed(i32::from(*change)))],
        ),
        TurnFact::WoundsTaken { count } => {
            catalog.format("turn_fact.wounds", &[("count", &count.to_string())])
        }
        TurnFact::WoundsHealed { count } => {
            catalog.format("turn_fact.healed", &[("count", &count.to_string())])
        }
        TurnFact::CrystalGained { color } => catalog.format(
            "turn_fact.crystal",
            &[("color", &color_name(*color, catalog))],
        ),
        TurnFact::CardGained { card } => {
            catalog.format("turn_fact.card", &[("card", &card_name(*card, catalog))])
        }
        TurnFact::UnitGained { unit: gained } => unit("turn_fact.unit_gained", gained),
        TurnFact::UnitLost { unit: lost } => unit("turn_fact.unit_lost", lost),
        TurnFact::UnitWounded { unit: wounded } => unit("turn_fact.unit_wounded", wounded),
        TurnFact::UnitHealed { unit: healed } => unit("turn_fact.unit_healed", healed),
        TurnFact::SiteConquered { hex } => {
            match view.map.space(*hex).and_then(|space| space.feature) {
                Some(Feature::Site(Site::Ruins)) => catalog.text("site.conquered.ruins"),
                Some(Feature::Site(Site::Keep)) => catalog.text("site.conquered.keep"),
                Some(Feature::Site(Site::MageTower)) => catalog.text("site.conquered.mage_tower"),
                // Spec 062 : « Cité bleue conquise ».
                Some(Feature::Site(site @ Site::City(_))) => catalog.format(
                    "site.conquered.city",
                    &[(
                        "site",
                        &catalog.text(crate::board::feature_key(Feature::Site(site))),
                    )],
                ),
                Some(feature) => catalog.format(
                    "site.conquered.adventure",
                    &[("site", &catalog.text(crate::board::feature_key(feature)))],
                ),
                None => String::new(),
            }
        }
        TurnFact::VillagePlundered => catalog.text("turn_fact.plundered"),
        TurnFact::MonasteryBurned { .. } => catalog.text("turn_fact.burned"),
        // Spec 056 : libellés ajoutés par le fil écran (étape 10).
        TurnFact::CardDestroyed { card } => catalog.format(
            "turn_fact.card_destroyed",
            &[("card", &card_name(*card, catalog))],
        ),
        TurnFact::CrystalsReturned { count } => catalog.format(
            "turn_fact.crystals_returned",
            &[("count", &count.to_string())],
        ),
        // Spec 054 : un ennemi détruit par un sort.
        TurnFact::EnemyDestroyed { enemy } => catalog.format(
            "turn_fact.enemy_destroyed",
            &[("enemy", &catalog.text(enemy.enemy().label_key()))],
        ),
        // Montré par la zone de jeu, filtré par `turn_fact_lines`.
        TurnFact::SpellCastFromOffer { .. } => String::new(),
        // Regroupés en une ligne par `turn_fact_lines`.
        TurnFact::ArtifactDieRolled { .. } => String::new(),
    }
}

/// Ce qu'il faut pour les boutons du bas, calculé par `game_screen`.
pub(super) struct BottomState<'a> {
    pub mode: &'a ScreenMode,
    /// Le bouton « Finir le tour… » quand finir le tour est possible hors des modes.
    pub end_turn: Option<&'a Button>,
    pub end_turn_prompt: Option<&'a EndTurnPrompt>,
    /// Le bandeau d'annonce de la fin de la manche, hors confirmation.
    pub announce: Option<&'a AnnouncePrompt>,
    pub game_over: bool,
}

/// « Offres… », « Finir la manche… » s'il y a lieu, « Finir le tour… » ; « Annuler » et « Finir le
/// tour » en mode fin du tour (C30 à C34).
pub(super) fn bottom_buttons(
    view: &PlayerView,
    state: BottomState<'_>,
    catalog: &Catalog,
) -> Vec<PurchaseButton> {
    if let Some(prompt) = state.end_turn_prompt {
        return vec![
            PurchaseButton {
                label: prompt.cancel.label.clone(),
                command: Some(prompt.cancel.command),
                tooltip: None,
                shortcut: shortcut_label(Shortcut::Cancel, catalog),
            },
            PurchaseButton {
                label: prompt.confirm.label.clone(),
                command: prompt.confirm_enabled.then_some(prompt.confirm.command),
                tooltip: None,
                shortcut: shortcut_label(Shortcut::Confirm, catalog),
            },
        ];
    }
    let fighting = view.combat.as_ref().is_some_and(|combat| !combat.ended);
    let choice = catalog.text("end_turn.disabled.choice");
    let offers_reason = if fighting {
        Some(catalog.text("offers.disabled.combat"))
    } else if !matches!(state.mode, ScreenMode::Normal) {
        Some(choice.clone())
    } else {
        None
    };
    let mut buttons = vec![PurchaseButton {
        label: catalog.text("offers.button"),
        command: offers_reason.is_none().then_some(Command::OpenOffers),
        tooltip: offers_reason,
        shortcut: shortcut_label(Shortcut::Offers, catalog),
    }];
    // Spec 023 : le texte du bandeau d'aujourd'hui passe dans l'infobulle.
    if let Some(prompt) = state.announce
        && let Some(button) = prompt.buttons.first()
    {
        buttons.push(PurchaseButton {
            label: catalog.text("round.announce.button_short"),
            command: Some(button.command),
            tooltip: Some(prompt.text.clone()),
            shortcut: None,
        });
    }
    buttons.push(match state.end_turn {
        Some(button) => PurchaseButton {
            label: button.label.clone(),
            command: Some(button.command),
            tooltip: None,
            shortcut: button.shortcut.clone(),
        },
        None => PurchaseButton {
            label: catalog.text("end_turn.button"),
            command: None,
            tooltip: Some(if state.game_over {
                catalog.text("end_turn.disabled.game_over")
            } else if fighting {
                catalog.text("end_turn.disabled.combat")
            } else {
                choice
            }),
            shortcut: shortcut_label(Shortcut::EndTurn, catalog),
        },
    });
    buttons
}

impl GameScreen {
    /// Les boutons du joueur, sous la ligne des points : les soins hors combat (spec 075, C32),
    /// par leur libellé et leur commande.
    pub fn player_buttons(&self) -> Vec<(&str, &Command)> {
        self.healing
            .iter()
            .map(|button| (button.label.as_str(), &button.command))
            .collect()
    }

    /// Vrai si une récompense ou un événement du tour s'affiche sous « Ce tour » (C29).
    pub(super) fn has_pending_lines(&self) -> bool {
        [
            &self.reward_pending,
            &self.spell_reward_pending,
            &self.ruin_reward_pending,
            &self.revealed_ruins,
            &self.reward_die,
            &self.crystal_reward_pending,
            &self.glade_token,
            &self.withdrawal,
        ]
        .iter()
        .any(|line| line.is_some())
    }
}

/// La fenêtre de la défausse (C35) : ses cartes, la dernière défaussée en tête, avec leurs effets
/// mais sans bouton ; aucune fenêtre quand la défausse est vide (C36).
pub fn discard_window(view: &PlayerView, catalog: &Catalog) -> Option<DiscardWindow> {
    pile_window(catalog.text("discard.title"), &view.discard, catalog)
}

/// Une fenêtre de défausse : celle du joueur, ou celle du fantôme (spec 061, C38).
pub(super) fn pile_window(
    title: String,
    pile: &[CardId],
    catalog: &Catalog,
) -> Option<DiscardWindow> {
    (!pile.is_empty()).then(|| DiscardWindow {
        title,
        cards: pile
            .iter()
            .rev()
            .map(|&card| HandCard {
                basic_effects: Vec::new(),
                strong_effects: Vec::new(),
                ..idle_card(card, catalog)
            })
            .collect(),
        close: Button {
            label: catalog.text("offers.close"),
            command: Command::CloseDiscard,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    })
}
