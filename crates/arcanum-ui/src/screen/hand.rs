//! Cartes de la main : boutons de jeu selon le mode en cours (specs 013, 016, 022).

use super::*;

pub(super) fn hand_card(
    card: CardId,
    view: &PlayerView,
    legal_actions: &[Action],
    choice: Option<(DiscardChoice, bool)>,
    catalog: &Catalog,
) -> HandCard {
    let mut plays = Vec::new();
    let mut sideways = Vec::new();
    for (index, action) in legal_actions.iter().enumerate() {
        // Les déplacements se font sur le plateau de jeu (spec 014).
        let Action::PlayCard { card: played, play } = action else {
            continue;
        };
        match (choice, play) {
            // Mode choix : seules comptent les actions qui jouent la carte choisie en défaussant celle-ci.
            (
                Some((choice, false)),
                CardPlay::Basic {
                    option, discard, ..
                },
            ) => {
                if *played == choice.card && *option == choice.option && *discard == Some(card) {
                    plays.push(Button {
                        label: pick_label(choice.card, false, catalog),
                        command: Command::Play(index),
                        shortcut: None,
                    });
                }
            }
            (
                Some((choice, true)),
                CardPlay::Strong {
                    option, discard, ..
                },
            ) => {
                let mana = ManaChoice {
                    card: choice.card,
                    option: choice.option,
                    discard: Some(card),
                };
                if *played == choice.card && *option == choice.option && *discard == Some(card) {
                    // Un seul bouton par carte à défausser ; le mana se choisit ensuite s'il y a
                    // plusieurs paiements.
                    let command =
                        match payments(legal_actions, &view.source, &mana, true, NO_SET_DICE)
                            .as_slice()
                        {
                            [(_, _, only)] => Command::Play(*only),
                            _ => Command::ChooseMana {
                                card: mana.card,
                                option: mana.option,
                                discard: mana.discard,
                            },
                        };
                    if !plays.iter().any(|button| button.command == command) {
                        plays.push(Button {
                            label: pick_label(choice.card, true, catalog),
                            command,
                            shortcut: None,
                        });
                    }
                }
            }
            (Some(_), _) => {}
            (None, _) if *played != card => {}
            // Spec 025 : les options payantes ont leurs propres boutons.
            (None, CardPlay::Basic { mana: Some(_), .. }) => {}
            (
                None,
                CardPlay::Basic {
                    option, discard, ..
                },
            ) => {
                let effect = option_effect(card, *option)
                    .map(|effect| card_option_label(card, effect, false, Some(view), catalog))
                    .unwrap_or_default();
                match discard {
                    None => plays.push(Button {
                        label: catalog.format("play.basic", &[("effect", &effect)]),
                        command: Command::Play(index),
                        shortcut: None,
                    }),
                    // Un seul bouton par option : la carte à défausser se choisit ensuite.
                    Some(_) => {
                        let command = Command::ChooseDiscard {
                            card,
                            option: *option,
                        };
                        if !plays.iter().any(|button| button.command == command) {
                            plays.push(Button {
                                label: catalog
                                    .format("play.basic_discarding", &[("effect", &effect)]),
                                command,
                                shortcut: None,
                            });
                        }
                    }
                }
            }
            // Une carte renforcée avec Concentration n'est jamais une action légale à part.
            (None, CardPlay::Strong { .. } | CardPlay::Boosted { .. }) => {}
            // Spec 054 : le sort lancé a ses boutons (fil écran).
            (None, CardPlay::Cast { .. }) => {}
            // Spec 055 : les artefacts ont leurs boutons (fil écran).
            (None, CardPlay::Artifact { .. } | CardPlay::ThrowAway { .. }) => {}
            // Spec 058 : le jeu de côté avec compétence part du bouton de la compétence (fil écran).
            (None, CardPlay::SidewaysSkill { .. }) => {}
            (None, CardPlay::Sideways(sideways_effect)) => {
                sideways.push(sideways_button(*sideways_effect, index, view, catalog))
            }
        }
    }
    let unit_plays = if choice.is_none() {
        ready_unit_buttons(card, view, legal_actions, catalog)
    } else {
        Vec::new()
    };
    plays.extend(unit_plays.iter().cloned());
    // Spec 073 : en mode choix, « Défausser cette carte » remplace les boutons d'effet.
    let (basic_effects, strong_effects) = match choice {
        None => live_effects(card, view, legal_actions, catalog),
        Some(_) => (Vec::new(), Vec::new()),
    };
    let (strong_plays, paid_plays) = if choice.is_none() {
        (
            strong_buttons(card, view, legal_actions, catalog),
            paid_buttons(card, view, legal_actions, catalog),
        )
    } else {
        (Vec::new(), Vec::new())
    };
    HandCard {
        plays,
        sideways,
        strong_plays,
        paid_plays,
        combat_note: siege_only_note(card, view, catalog)
            .or_else(|| healing_note(card, view, catalog)),
        basic_effects,
        strong_effects,
        unit_plays,
        ..idle_card(card, catalog)
    }
}

/// Le bouton-icône d'un effet de côté (spec 071, C14 à C16) : son icône, « ÷2 » quand il compte à
/// moitié (spec 032), l'effet en infobulle.
fn sideways_button(
    effect: SidewaysEffect,
    index: usize,
    view: &PlayerView,
    catalog: &Catalog,
) -> crate::IconButton {
    let icon = match effect {
        SidewaysEffect::Move => Icon::Move,
        SidewaysEffect::Influence => Icon::Influence,
        SidewaysEffect::Attack => crate::attack_icon(AttackKind::Melee, Element::Physical),
        SidewaysEffect::Block => crate::block_icon(Element::Physical),
    };
    let label = marked_effect_text(effect.effect(), view, catalog);
    let halved = view
        .combat
        .as_ref()
        .filter(|combat| !combat.ended)
        .and_then(|combat| combat.efficiency(effect.effect()))
        == Some(Efficiency::Halved);
    crate::IconButton {
        icon,
        tooltip: catalog.format("play.sideways_effect", &[("effect", &label)]),
        label,
        mark: halved.then(|| catalog.text("combat.mark.halved")),
        command: Command::Play(index),
    }
}

/// « siège seulement » sur une carte qui a une Attaque à distance, quand l'attaque en cours vise un
/// fortifié en phase d'attaque à distance (MKUE p. 9, 3.a ; spec 032, C21).
fn siege_only_note(card: CardId, view: &PlayerView, catalog: &Catalog) -> Option<String> {
    let combat = view.combat.as_ref().filter(|combat| !combat.ended)?;
    let ranged = Effect::Attack {
        amount: 1,
        element: Element::Physical,
        kind: AttackKind::Ranged,
    };
    if combat.phase != CombatPhase::RangedAndSiege
        || !matches!(combat.in_progress, Some(InProgress::Attack { .. }))
        || combat.accepts(ranged)
    {
        return None;
    }
    let definition = card.card().definition()?;
    let options =
        [definition.basic, definition.strong]
            .into_iter()
            .flat_map(|effect| match effect {
                BasicEffect::Options { options, .. } => options.to_vec(),
                BasicEffect::Unsupported => Vec::new(),
            });
    options
        .into_iter()
        .any(|effect| {
            matches!(
                effect,
                Effect::Attack {
                    kind: AttackKind::Ranged,
                    ..
                }
            )
        })
        .then(|| catalog.text("combat.mark.siege_only"))
}

/// « Soin : pas en combat » sur une carte de soin pendant un combat (spec 034, Q3).
fn healing_note(card: CardId, view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.combat.as_ref().filter(|combat| !combat.ended)?;
    card.card()
        .definition()
        .is_some_and(|definition| definition.healing)
        .then(|| catalog.text("effect.not_in_combat"))
}

/// Une autre carte de la main que Concentration peut renforcer maintenant (spec 025, Q5 ; en
/// combat, spec 031, L6).
pub(super) fn has_boost_target(concentration: CardId, view: &PlayerView) -> bool {
    view.hand.iter().any(|&other| {
        let Some(BasicEffect::Options { options, cost, .. }) = other
            .card()
            .definition()
            .map(|definition| definition.strong)
        else {
            return false;
        };
        other != concentration
            && !options
                .iter()
                .any(|effect| matches!(effect, Effect::Boost(_)))
            && options.iter().any(|effect| view.is_playable_now(*effect))
            // Une carte à défausser, autre que Concentration et la carte renforcée.
            && (cost == Cost::None || view.hand.len() >= 3)
    })
}

/// Carte de la main pendant le choix du mana : rien à cliquer. Une Blessure n'a ni couleur ni
/// effet (spec 031).
pub(super) fn idle_card(card: CardId, catalog: &Catalog) -> HandCard {
    let definition = card.card().definition();
    // Une Blessure n'a aucun texte d'effet (spec 031).
    let (effect, strong_effect, strong_text) = if card.card().is_wound() {
        (String::new(), String::new(), String::new())
    } else {
        let (effect, strong_effect) = card_effect_texts(card, catalog);
        (effect, strong_effect, card_strong_text(card, catalog))
    };
    let (basic_effects, strong_effects) = idle_effects(card, None, catalog);
    let info = card_info(card, &effect, &strong_text, catalog);
    HandCard {
        card,
        name: card_name(card, catalog),
        color: definition.map(|definition| definition.color),
        effect,
        plays: Vec::new(),
        sideways: Vec::new(),
        strong_effect,
        strong_plays: Vec::new(),
        paid_plays: Vec::new(),
        discard_toggle: None,
        combat_note: None,
        illustration: crate::card_illustration(card.card()),
        art_fills_card: card.card().is_wound(),
        basic_effects,
        strong_text,
        strong_effects,
        unit_plays: Vec::new(),
        info,
        show_text: card.card().is_artifact(),
        strong_heading: String::new(),
        strong_colors: Vec::new(),
    }
}

/// Le texte complet derrière « i » (spec #11, C1 à C5) : le texte écrit de la carte quand elle en
/// a un (actions avancées, sorts), sinon les textes calculés ; rien pour une Blessure ni un
/// artefact, qui écrit le sien sur lui (Q1).
fn card_info(card: CardId, effect: &str, strong_text: &str, catalog: &Catalog) -> Option<CardInfo> {
    if card.card().is_wound() || card.card().is_artifact() {
        return None;
    }
    let (basic, strong) = match card.card().unsupported_text_keys() {
        Some((basic, strong)) => (catalog.text(basic), catalog.text(strong)),
        None => (effect.to_owned(), strong_text.to_owned()),
    };
    Some(CardInfo {
        name: card_name(card, catalog),
        basic,
        strong_title: strong_title(card, catalog),
        strong,
    })
}

/// « Renforcé », ou le nom renforcé d'un sort (« Vol d'esprit »).
fn strong_title(card: CardId, catalog: &Catalog) -> String {
    if card.card().is_spell() {
        spell_name(card, true, catalog)
    } else {
        catalog.text("panel.strong")
    }
}

/// Carte de la main en mode fin du tour : seulement sa case « Défausser », sauf si elle est
/// choisie pour la reprise (spec 025), ou si aucune fin de tour légale ne la défausse : une
/// Blessure, hors repos (spec 034). Les Blessures étant identiques, une Blessure se coche dès
/// qu'une fin de tour légale en défausse une.
pub(super) fn ending_turn_card(
    card: CardId,
    ending: &EndTurnChoice,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> HandCard {
    let wound = card.card().is_wound();
    let discardable = legal_actions.iter().any(|action| {
        matches!(action, Action::EndTurn { discard, .. }
            if discard.contains(&card)
                || (wound && discard.iter().any(|other| other.card().is_wound())))
    });
    HandCard {
        discard_toggle: (discardable && ending.reclaim != Some(card)).then(|| DiscardToggle {
            label: catalog.text("end_turn.discard"),
            checked: ending.discard.contains(&card),
            command: Command::ToggleEndTurnDiscard(card),
        }),
        ..idle_card(card, catalog)
    }
}

/// Rajeunissement : un bouton par unité à rendre disponible, et par mana pour l'effet renforcé
/// (« Renforcer : Rendre disponible … · Paysans · Dé vert ») (spec 042).
fn ready_unit_buttons(
    card: CardId,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<Button> {
    let color = card.card().definition().map(|definition| definition.color);
    let mut buttons: Vec<Button> = Vec::new();
    for (index, action) in legal_actions.iter().enumerate() {
        let Action::PlayCardOnUnit {
            card: played,
            play,
            target,
        } = action
        else {
            continue;
        };
        if *played != card {
            continue;
        }
        let (key, effect, payment) = match play {
            CardPlay::Basic { option, .. } => ("play.basic", option_effect(card, *option), None),
            CardPlay::Strong { option, mana, .. } => (
                "play.strong",
                option_effect_of(card, *option, true),
                Some(*mana),
            ),
            _ => continue,
        };
        let mut parts = vec![
            effect
                .map(|effect| effect_text(effect, catalog))
                .unwrap_or_default(),
            catalog.text(target.unit.label_key()),
        ];
        if let (Some(payment), Some(color)) = (payment, color) {
            parts.push(payment_label(
                view,
                payment,
                ManaColor::Basic(color),
                catalog,
            ));
        }
        let label = catalog.format(key, &[("effect", &parts.join(" · "))]);
        if !buttons.iter().any(|button| button.label == label) {
            buttons.push(Button {
                label,
                command: Command::Play(index),
                shortcut: None,
            });
        }
    }
    buttons
}

/// « Défausser cette carte », ou « Détruire cette carte » pour Décomposition (spec 056).
fn pick_label(card: CardId, strong: bool, catalog: &Catalog) -> String {
    catalog.text(if destroys(card, strong) {
        "discard_choice.destroy"
    } else {
        "discard_choice.pick"
    })
}
