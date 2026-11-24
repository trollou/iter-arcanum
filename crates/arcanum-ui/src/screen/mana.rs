//! Paiement du mana d'un effet renforcé : boutons et bandeau du choix (spec 022).

use arcanum_core::PolarSource;

use super::*;

/// Raison d'un bouton payant grisé : plus de dé ce tour, ou aucun mana de cette couleur.
pub(super) fn missing_mana(view: &PlayerView, color: BasicColor, catalog: &Catalog) -> String {
    let name = color_name(color, catalog);
    if view.dice_taken.len() >= view.dice_limit {
        catalog.format("play.strong_die_taken", &[("color", &name)])
    } else {
        catalog.format("play.strong_no_mana", &[("color", &name)])
    }
}

/// Les boutons des options de l'effet de base qui coûtent du mana (spec 025).
pub(super) fn paid_buttons(
    card: CardId,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<StrongButton> {
    let Some(basic) = card.card().definition().map(|definition| definition.basic) else {
        return Vec::new();
    };
    let BasicEffect::Options { options, .. } = basic else {
        return Vec::new();
    };
    let mut buttons = Vec::new();
    for (option, effect) in (0u8..).zip(options) {
        // Spec 053 : le mana de l'option (Magie pure) ou celui de l'effet (Cristallisation).
        let Some(color) = basic.option_mana(option) else {
            continue;
        };
        let choice = ManaChoice {
            card,
            option,
            discard: None,
        };
        let found = payments(legal_actions, &view.source, &choice, false, NO_SET_DICE);
        let command = match found.as_slice() {
            [(_, _, only)] => Command::Play(*only),
            _ => Command::ChooseBasicMana { card, option },
        };
        buttons.push(StrongButton {
            label: effect_text(*effect, catalog),
            color,
            command,
            enabled: !found.is_empty(),
            tooltip: found.is_empty().then(|| missing_mana(view, color, catalog)),
        });
    }
    buttons
}

/// Les boutons « Renforcer » d'une carte : un par option jouable hors combat (MKUE p. 6, 5.c).
pub(super) fn strong_buttons(
    card: CardId,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<StrongButton> {
    let Some(definition) = card.card().definition() else {
        return Vec::new();
    };
    let BasicEffect::Options { options, cost, .. } = definition.strong else {
        return Vec::new();
    };
    let color = definition.color;
    let mut buttons = Vec::new();
    for (option, effect) in (0u8..).zip(options) {
        // Spec 053 : une option sans points se joue pour ses modificateurs (Au cœur du combat).
        if !view.is_option_playable_now(*effect, definition.strong.option_modifiers(option)) {
            continue;
        }
        let discarding = cost != Cost::None;
        // Sans autre carte à défausser, l'option n'existe pas plus que son effet de base.
        if discarding && view.hand.len() < 2 {
            continue;
        }
        let text = card_option_label(card, *effect, true, Some(view), catalog);
        let label = if discarding {
            catalog.format("play.strong_discarding", &[("effect", &text)])
        } else {
            catalog.format("play.strong", &[("effect", &text)])
        };
        let offered = strong_offered(card, option, legal_actions);
        let command = strong_command(card, option, *effect, cost, view, legal_actions);
        let tooltip = (!offered).then(|| strong_reason(card, *effect, color, view, catalog));
        buttons.push(StrongButton {
            label,
            color,
            command,
            enabled: offered,
            tooltip,
        });
    }
    buttons
}

/// Vrai si une action légale joue cette option renforcée.
pub(super) fn strong_offered(card: CardId, option: u8, legal_actions: &[Action]) -> bool {
    legal_actions.iter().any(|action| {
        matches!(action, Action::PlayCard { card: played, play: CardPlay::Strong { option: o, .. } }
            if *played == card && *o == option)
    })
}

/// Ce que fait le bouton d'une option renforcée : la carte à défausser, les dés, la carte à
/// renforcer, le mana, ou l'action seule s'il n'y a qu'un paiement.
pub(super) fn strong_command(
    card: CardId,
    option: u8,
    effect: Effect,
    cost: Cost,
    view: &PlayerView,
    legal_actions: &[Action],
) -> Command {
    let mana = ManaChoice {
        card,
        option,
        discard: None,
    };
    if matches!(effect, Effect::TrainFromOffer(_) | Effect::GainSpell(_)) {
        // Spec 056 : la carte à détruire, la carte de l'offre, puis le mana.
        Command::ChooseGain(GainChoice {
            card,
            option,
            strong: true,
            discard: None,
            gain: None,
        })
    } else if matches!(effect, Effect::Repeat(_)) {
        // Spec 056 : la cible d'Effet maximum, comme la carte renforcée par Concentration.
        Command::ChooseBoost(BoostChoice {
            card,
            option,
            basic: false,
            target: None,
            target_option: 0,
            discard: None,
            settings: NO_SET_DICE,
            die: None,
        })
    } else if cost != Cost::None {
        Command::ChooseStrongDiscard { card, option }
    } else if matches!(effect, Effect::SetDice { .. }) {
        // Spec 025 : d'abord les dés et leurs couleurs.
        Command::ChooseSetDice { card, option }
    } else if matches!(effect, Effect::Boost(_)) {
        // Spec 025 : d'abord la carte à renforcer.
        Command::ChooseBoost(BoostChoice {
            card,
            basic: false,
            option,
            target: None,
            target_option: 0,
            discard: None,
            settings: NO_SET_DICE,
            die: None,
        })
    } else if has_extra_mana(card, option) {
        // Spec 053 : deux manas, choisis ensemble dans la fenêtre du choix du mana.
        match extra_payments(legal_actions, &view.source, &mana).as_slice() {
            [(_, _, only)] => Command::Play(*only),
            _ => Command::ChooseMana {
                card,
                option,
                discard: None,
            },
        }
    } else {
        match payments(legal_actions, &view.source, &mana, true, NO_SET_DICE).as_slice() {
            [(_, _, only)] => Command::Play(*only),
            _ => Command::ChooseMana {
                card,
                option,
                discard: None,
            },
        }
    }
}

/// Pourquoi une option renforcée jouable maintenant n'est pas offerte : pas de carte à renforcer
/// pour Concentration, sinon le mana manquant.
pub(super) fn strong_reason(
    card: CardId,
    effect: Effect,
    color: BasicColor,
    view: &PlayerView,
    catalog: &Catalog,
) -> String {
    if matches!(effect, Effect::Boost(_)) && !has_boost_target(card, view) {
        catalog.text("concentration.no_target")
    } else {
        missing_mana(view, color, catalog)
    }
}

/// Le paiement tel que le joueur le choisit : un dé d'une couleur donnée (deux dés verts se
/// valent), un cristal ou un jeton (spec 025).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PaymentKind {
    Die(ManaColor),
    Crystal,
    Token,
    /// Mana illimité d'un anneau (spec 055, C25).
    Endless(ManaColor),
    /// Invocation : cette carte défaussée (spec 058, C10).
    Invocation(CardId),
    /// Polarisation : un mana de la couleur `from`, venu de `source` (spec 058, C13).
    Polarized {
        from: ManaColor,
        source: PolarKind,
    },
    /// Vol de mana : le dé posé sur la tactique (spec 060, C12).
    StolenDie,
}

/// D'où vient le mana polarisé, tel que le joueur le choisit : deux dés de la même face se valent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PolarKind {
    Die(ManaColor),
    Crystal,
    Token,
    Invocation(CardId),
    /// Le dé de Vol de mana (spec 060).
    StolenDie,
}

/// « Invocation : défausser Rage (vert) », « Polarisation : Cristal blanc comme vert » (spec 058,
/// Libellés) ; `None` pour un autre paiement. `color` : la couleur payée.
pub(super) fn skill_payment_text(
    view: &PlayerView,
    kind: PaymentKind,
    color: ManaColor,
    catalog: &Catalog,
) -> Option<String> {
    // Spec 060, C12 : « Dé volé : bleu ».
    let stolen = || {
        let face = view
            .tactic
            .as_ref()
            .and_then(|tactic| tactic.stolen_die)
            .map_or(color, |(_, face)| face);
        catalog.format("tactic.stolen_die", &[("color", &mana_name(face, catalog))])
    };
    let invocation = |card: CardId, color: ManaColor| {
        catalog.format(
            "mana_choice.invocation",
            &[
                ("card", &card_name(card, catalog)),
                ("color", &mana_name(color, catalog)),
            ],
        )
    };
    match kind {
        PaymentKind::Invocation(card) => Some(invocation(card, color)),
        PaymentKind::StolenDie => Some(stolen()),
        PaymentKind::Polarized { from, source } => {
            let from_color = mana_name(from, catalog);
            let origin = match source {
                PolarKind::Die(face) => {
                    catalog.format("mana_choice.die", &[("color", &mana_name(face, catalog))])
                }
                PolarKind::Crystal => {
                    catalog.format("effect.gain_crystal", &[("color", &from_color)])
                }
                PolarKind::Token => catalog.format("effect.gain_token", &[("color", &from_color)]),
                PolarKind::Invocation(card) => invocation(card, from),
                PolarKind::StolenDie => stolen(),
            };
            Some(catalog.format(
                "mana_choice.polarization",
                &[("from", &origin), ("color", &mana_name(color, catalog))],
            ))
        }
        _ => None,
    }
}

/// Les paiements distincts de `choice`, chacun avec l'index de sa première action légale, dans
/// l'ordre des actions (dés dans l'ordre de la Source, puis cristal, puis jeton) ; effet
/// renforcé si `strong`, sinon option payante de l'effet de base.
pub(super) fn payments(
    legal_actions: &[Action],
    source: &[ManaColor],
    choice: &ManaChoice,
    strong: bool,
    set_dice: [Option<DieSetting>; 2],
) -> Vec<(PaymentKind, ManaPayment, usize)> {
    let mut found: Vec<(PaymentKind, ManaPayment, usize)> = Vec::new();
    for (index, action) in legal_actions.iter().enumerate() {
        let Action::PlayCard { card, play } = action else {
            continue;
        };
        let (option, discard, mana) = match (play, strong) {
            (
                CardPlay::Strong {
                    option,
                    discard,
                    mana,
                    set_dice: chosen,
                    with: None,
                    extra_mana: None,
                    gain: None,
                },
                true,
            ) if *chosen == set_dice => (option, discard, mana),
            (
                CardPlay::Basic {
                    option,
                    discard,
                    mana: Some(mana),
                    with: None,
                    gain: None,
                    die: None,
                },
                false,
            ) => (option, discard, mana),
            _ => continue,
        };
        if *card == choice.card && *option == choice.option && *discard == choice.discard {
            add_payment(&mut found, source, *mana, index);
        }
    }
    found
}

/// Ajoute le paiement `mana` de l'action `index` s'il est d'une sorte pas encore vue.
pub(super) fn add_payment(
    found: &mut Vec<(PaymentKind, ManaPayment, usize)>,
    source: &[ManaColor],
    mana: ManaPayment,
    index: usize,
) {
    let kind = match mana {
        ManaPayment::Die(die) => match source.get(usize::from(die)) {
            Some(face) => PaymentKind::Die(*face),
            None => return,
        },
        ManaPayment::Crystal => PaymentKind::Crystal,
        ManaPayment::Token => PaymentKind::Token,
        ManaPayment::Endless(_)
        | ManaPayment::Invocation(_)
        | ManaPayment::Polarized { .. }
        | ManaPayment::StolenDie => match kind_of(source, mana) {
            Some(kind) => kind,
            None => return,
        },
    };
    if !found.iter().any(|(seen, _, _)| *seen == kind) {
        found.push((kind, mana, index));
    }
}

/// Bandeau du choix du mana : un bouton par paiement distinct trouvé.
pub(super) fn mana_prompt(
    view: &PlayerView,
    choice: &ManaChoice,
    strong: bool,
    found: Vec<(PaymentKind, ManaPayment, usize)>,
    catalog: &Catalog,
) -> ManaPrompt {
    let effect = option_effect_of(choice.card, choice.option, strong);
    // Effet renforcé : la couleur de la carte ; option payante : la sienne (spec 025).
    let card_color = choice
        .card
        .card()
        .definition()
        .map(|definition| definition.color);
    let color = if strong {
        card_color
    } else {
        choice
            .card
            .card()
            .definition()
            .and_then(|definition| definition.basic.option_mana(choice.option))
            .or(card_color)
    };
    // Une Blessure ne se joue jamais : la carte en cours de jeu a toujours une couleur, et sans
    // action légale `found` est vide.
    let color_text = color
        .map(|color| color_name(color, catalog))
        .unwrap_or_default();
    let buttons = found
        .into_iter()
        .map(|(kind, _, index)| {
            let label = match kind {
                PaymentKind::Die(ManaColor::Gold) => {
                    catalog.format("mana_choice.gold_die", &[("color", &color_text)])
                }
                PaymentKind::Die(face) => {
                    catalog.format("mana_choice.die", &[("color", &mana_name(face, catalog))])
                }
                PaymentKind::Crystal => catalog.format(
                    "mana_choice.crystal",
                    &[
                        ("color", &color_text),
                        (
                            "count",
                            &color
                                .map_or(0, |color| view.inventory.crystals(color))
                                .to_string(),
                        ),
                        ("limit", &CRYSTAL_LIMIT.to_string()),
                    ],
                ),
                PaymentKind::Token => {
                    catalog.format("mana_choice.token", &[("color", &color_text)])
                }
                PaymentKind::Endless(face) => catalog.format(
                    "mana_choice.endless",
                    &[("color", &mana_name(face, catalog))],
                ),
                PaymentKind::Invocation(_)
                | PaymentKind::Polarized { .. }
                | PaymentKind::StolenDie => color
                    .and_then(|color| {
                        skill_payment_text(view, kind, ManaColor::Basic(color), catalog)
                    })
                    .unwrap_or_default(),
            };
            Button {
                label,
                command: Command::Play(index),
                shortcut: None,
            }
        })
        .collect();
    ManaPrompt {
        text: catalog.format(
            if strong {
                "mana_choice.prompt"
            } else {
                "mana_choice.prompt_basic"
            },
            &[
                ("card", &card_name(choice.card, catalog)),
                (
                    "effect",
                    &effect
                        .map(|effect| effect_text(effect, catalog))
                        .unwrap_or_default(),
                ),
            ],
        ),
        payments: buttons,
        cancel: Button {
            label: catalog.text("mana_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    }
}

/// Vrai si l'option renforcée `option` coûte un mana en plus (Magie pure, Mélodie du vent ;
/// spec 053).
pub(super) fn has_extra_mana(card: CardId, option: u8) -> bool {
    card.card()
        .definition()
        .is_some_and(|definition| definition.strong.option_mana(option).is_some())
}

/// Les paiements des deux manas d'une option renforcée qui en coûte un en plus (spec 053) : chaque
/// paire distincte (mana de la carte, puis mana en plus), avec l'index de sa première action
/// légale.
pub(super) fn extra_payments(
    legal_actions: &[Action],
    source: &[ManaColor],
    choice: &ManaChoice,
) -> Vec<(ManaPayment, ManaPayment, usize)> {
    let mut seen: Vec<(PaymentKind, PaymentKind)> = Vec::new();
    let mut found = Vec::new();
    for (index, action) in legal_actions.iter().enumerate() {
        let Action::PlayCard {
            card,
            play:
                CardPlay::Strong {
                    option,
                    discard,
                    mana,
                    extra_mana: Some(extra),
                    ..
                },
        } = action
        else {
            continue;
        };
        if *card != choice.card || *option != choice.option || *discard != choice.discard {
            continue;
        }
        let (Some(first), Some(second)) = (kind_of(source, *mana), kind_of(source, *extra)) else {
            continue;
        };
        if !seen.contains(&(first, second)) {
            seen.push((first, second));
            found.push((*mana, *extra, index));
        }
    }
    found
}

/// La sorte d'un paiement : un dé par sa face, un cristal, un jeton.
pub(super) fn kind_of(source: &[ManaColor], mana: ManaPayment) -> Option<PaymentKind> {
    match mana {
        ManaPayment::Die(die) => source
            .get(usize::from(die))
            .map(|face| PaymentKind::Die(*face)),
        ManaPayment::Crystal => Some(PaymentKind::Crystal),
        ManaPayment::Token => Some(PaymentKind::Token),
        ManaPayment::Endless(color) => Some(PaymentKind::Endless(color)),
        ManaPayment::Invocation(card) => Some(PaymentKind::Invocation(card)),
        ManaPayment::StolenDie => Some(PaymentKind::StolenDie),
        ManaPayment::Polarized {
            from,
            source: origin,
        } => Some(PaymentKind::Polarized {
            from,
            source: match origin {
                PolarSource::Die(die) => PolarKind::Die(*source.get(usize::from(die))?),
                PolarSource::Crystal => PolarKind::Crystal,
                PolarSource::Token => PolarKind::Token,
                PolarSource::Invocation(card) => PolarKind::Invocation(card),
                PolarSource::StolenDie => PolarKind::StolenDie,
            },
        }),
    }
}

/// Le bandeau du choix des deux manas (spec 053) : « Dé bleu · Cristal bleu (1 sur 3) », le mana
/// de la carte puis le mana en plus.
pub(super) fn extra_mana_prompt(
    view: &PlayerView,
    choice: &ManaChoice,
    found: Vec<(ManaPayment, ManaPayment, usize)>,
    catalog: &Catalog,
) -> ManaPrompt {
    let definition = choice.card.card().definition();
    let card_color = definition.map(|definition| ManaColor::Basic(definition.color));
    let extra_color = definition
        .and_then(|definition| definition.strong.option_mana(choice.option))
        .map(ManaColor::Basic);
    let mut prompt = mana_prompt(view, choice, true, Vec::new(), catalog);
    if let (Some(card_color), Some(extra_color)) = (card_color, extra_color) {
        prompt.payments = found
            .into_iter()
            .map(|(mana, extra, index)| Button {
                label: format!(
                    "{} · {}",
                    payment_label(view, mana, card_color, catalog),
                    payment_label(view, extra, extra_color, catalog)
                ),
                command: Command::Play(index),
                shortcut: None,
            })
            .collect();
    }
    prompt
}
