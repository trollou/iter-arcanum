//! Choix d'une carte d'offre (spec 056) : Entraînement détruit une carte de la main puis prend une
//! action avancée de sa couleur, Don pour la magie renforcé prend un sort de la couleur payée ; le
//! mana de l'effet renforcé vient en dernier.

use arcanum_core::CommonCard;

use super::*;

/// Étape du choix d'une carte d'offre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GainStage {
    Destroy,
    Offer,
    Mana,
}

/// Une action qui prolonge le choix : son index, la carte détruite, la carte prise, le mana de la
/// carte et le mana en plus.
pub(super) struct GainCandidate {
    pub(super) index: usize,
    pub(super) discard: Option<CardId>,
    pub(super) gain: CardId,
    pub(super) mana: Option<ManaPayment>,
    pub(super) extra: Option<ManaPayment>,
}

/// Les actions légales qui prolongent `choice` : la carte jouée avec cette option, la carte
/// détruite et la carte prise déjà choisies.
pub(super) fn gain_candidates(legal_actions: &[Action], choice: &GainChoice) -> Vec<GainCandidate> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            let Action::PlayCard { card, play } = action else {
                return None;
            };
            // Spec 055 : le Livre de sagesse, la carte détruite dans ses choix.
            let artifact = match (play, choice.strong) {
                (CardPlay::Artifact { option, choices }, false)
                | (CardPlay::ThrowAway { option, choices }, true) => choices
                    .gain
                    .map(|gain| (*option, choices.discard.iter().next(), gain)),
                _ => None,
            };
            if let Some((option, discard, gain)) = artifact {
                return (*card == choice.card
                    && option == choice.option
                    && choice.discard.is_none_or(|chosen| discard == Some(chosen))
                    && choice.gain.is_none_or(|chosen| gain == chosen))
                .then_some(GainCandidate {
                    index,
                    discard,
                    gain,
                    mana: None,
                    extra: None,
                });
            }
            let (option, discard, gain, mana, extra) = match (play, choice.strong) {
                (
                    CardPlay::Basic {
                        option,
                        discard,
                        gain: Some(gain),
                        ..
                    },
                    false,
                ) => (option, discard, gain, None, None),
                (
                    CardPlay::Strong {
                        option,
                        discard,
                        gain: Some(gain),
                        mana,
                        extra_mana,
                        ..
                    },
                    true,
                ) => (option, discard, gain, Some(*mana), *extra_mana),
                _ => return None,
            };
            (*card == choice.card
                && *option == choice.option
                && choice.discard.is_none_or(|chosen| *discard == Some(chosen))
                && choice.gain.is_none_or(|chosen| *gain == chosen))
            .then_some(GainCandidate {
                index,
                discard: *discard,
                gain: *gain,
                mana,
                extra,
            })
        })
        .collect()
}

/// Le prochain choix : la carte à détruire, la carte de l'offre, puis le mana.
pub(super) fn gain_stage(choice: &GainChoice, candidates: &[GainCandidate]) -> GainStage {
    if choice.discard.is_none() && candidates.iter().any(|found| found.discard.is_some()) {
        GainStage::Destroy
    } else if choice.gain.is_none() {
        GainStage::Offer
    } else {
        GainStage::Mana
    }
}

/// Les paiements distincts des candidats (mana de la carte, puis mana en plus), chacun avec l'index
/// de sa première action.
fn gain_payments(
    source: &[ManaColor],
    candidates: &[GainCandidate],
) -> Vec<(ManaPayment, Option<ManaPayment>, usize)> {
    let mut seen: Vec<(PaymentKind, Option<PaymentKind>)> = Vec::new();
    let mut found = Vec::new();
    for candidate in candidates {
        let Some(mana) = candidate.mana else {
            continue;
        };
        let Some(first) = kind_of(source, mana) else {
            continue;
        };
        let second = candidate.extra.and_then(|extra| kind_of(source, extra));
        if !seen.contains(&(first, second)) {
            seen.push((first, second));
            found.push((mana, candidate.extra, candidate.index));
        }
    }
    found
}

/// Le bouton qui mène à `next` : il joue l'action quand il ne reste qu'une façon de la jouer.
pub(super) fn gain_command(
    legal_actions: &[Action],
    source: &[ManaColor],
    next: GainChoice,
) -> Command {
    let candidates = gain_candidates(legal_actions, &next);
    if gain_stage(&next, &candidates) == GainStage::Mana {
        if let [only] = candidates.as_slice() {
            return Command::Play(only.index);
        }
        if let [(_, _, only)] = gain_payments(source, &candidates).as_slice() {
            return Command::Play(*only);
        }
    }
    Command::ChooseGain(next)
}

/// Le choix en cours s'il mène encore à une action légale, ses candidats et son étape.
pub(super) fn active_gain(
    mode: &ScreenMode,
    legal_actions: &[Action],
) -> Option<(GainChoice, Vec<GainCandidate>, GainStage)> {
    let ScreenMode::ChoosingGain(choice) = mode else {
        return None;
    };
    let candidates = gain_candidates(legal_actions, choice);
    (!candidates.is_empty()).then(|| {
        let stage = gain_stage(choice, &candidates);
        (*choice, candidates, stage)
    })
}

/// Vrai pendant le choix de la carte dans la fenêtre Offres, qui s'ouvre alors d'office.
pub(super) fn choosing_offer_card(mode: &ScreenMode, legal_actions: &[Action]) -> bool {
    matches!(
        active_gain(mode, legal_actions),
        Some((_, _, GainStage::Offer))
    )
}

/// Carte de la main pendant le choix : « Détruire » sur chaque carte permise, à
/// l'étape de la carte à détruire.
pub(super) fn gain_hand_card(
    card: CardId,
    view: &PlayerView,
    legal_actions: &[Action],
    (choice, candidates, stage): &(GainChoice, Vec<GainCandidate>, GainStage),
    catalog: &Catalog,
) -> HandCard {
    let mut held = idle_card(card, catalog);
    if *stage == GainStage::Destroy && candidates.iter().any(|found| found.discard == Some(card)) {
        held.plays.push(Button {
            label: catalog.text("discard_choice.destroy"),
            command: gain_command(
                legal_actions,
                &view.source,
                GainChoice {
                    discard: Some(card),
                    ..*choice
                },
            ),
            shortcut: None,
        });
        held.basic_effects.clear();
        held.strong_effects.clear();
    }
    held
}

/// Le bandeau de l'étape : la carte à détruire, la carte de l'offre ; `None` à l'étape du mana.
pub(super) fn gain_prompt(
    (choice, _, stage): &(GainChoice, Vec<GainCandidate>, GainStage),
    catalog: &Catalog,
) -> Option<DiscardPrompt> {
    let key = match stage {
        GainStage::Destroy => "discard_choice.destroy_prompt",
        GainStage::Offer => "gain_choice.prompt",
        GainStage::Mana => return None,
    };
    Some(DiscardPrompt {
        text: catalog.format(
            key,
            &[
                ("card", &card_name(choice.card, catalog)),
                (
                    "effect",
                    &option_effect_of(choice.card, choice.option, choice.strong)
                        .map(|effect| effect_text(effect, catalog))
                        .unwrap_or_default(),
                ),
            ],
        ),
        cancel: Button {
            label: catalog.text("discard_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    })
}

/// Le bandeau du mana à l'étape du mana : « Dé vert », ou « Dé bleu · Cristal rouge » avec un mana
/// en plus (Don pour la magie).
pub(super) fn gain_mana_prompt(
    view: &PlayerView,
    (choice, candidates, stage): &(GainChoice, Vec<GainCandidate>, GainStage),
    catalog: &Catalog,
) -> Option<ManaPrompt> {
    if *stage != GainStage::Mana || !choice.strong {
        return None;
    }
    let definition = choice.card.card().definition()?;
    let card_color = ManaColor::Basic(definition.color);
    let extra_color = definition
        .strong
        .option_mana(choice.option)
        .map(ManaColor::Basic);
    let mana = ManaChoice {
        card: choice.card,
        option: choice.option,
        discard: choice.discard,
    };
    let mut prompt = mana_prompt(view, &mana, true, Vec::new(), catalog);
    prompt.payments = gain_payments(&view.source, candidates)
        .into_iter()
        .map(|(mana, extra, index)| {
            let first = payment_label(view, mana, card_color, catalog);
            let label = match (extra, extra_color) {
                (Some(extra), Some(color)) => {
                    format!("{first} · {}", payment_label(view, extra, color, catalog))
                }
                _ => first,
            };
            Button {
                label,
                command: Command::Play(index),
                shortcut: None,
            }
        })
        .collect();
    Some(prompt)
}

/// « Prendre » une carte de l'offre pendant le choix (C24, C29) : seules les cartes permises.
pub(super) fn gain_take(
    card: CommonCard,
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<Button> {
    let CommonCard::Deed(deed) = card else {
        return None;
    };
    let (choice, candidates, stage) = active_gain(mode, legal_actions)?;
    if stage != GainStage::Offer || !candidates.iter().any(|found| found.gain == deed) {
        return None;
    }
    Some(Button {
        label: catalog.text("offers.take"),
        command: gain_command(
            legal_actions,
            &view.source,
            GainChoice {
                gain: Some(deed),
                ..choice
            },
        ),
        shortcut: None,
    })
}
