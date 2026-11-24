//! Événements pour `decide` : paiements de mana, activations d'unités et de compétences.

use super::*;

/// Les événements d'une Invocation (spec 058) : la compétence, puis la carte défaussée (le jeton
/// gagné est dépensé aussitôt).
fn invocation_events(player: PlayerId, card: CardId) -> [Event; 2] {
    [
        Event::SkillUsed {
            player,
            skill: Skill::Invocation,
            option: 0,
            effect: Effect::Nothing,
        },
        Event::CardDiscarded { player, card },
    ]
}

/// Les événements d'un paiement par Invocation ou Polarisation (spec 058), ou `None` pour un
/// paiement ordinaire.
fn skill_payment_events(
    player: PlayerId,
    mana: ManaPayment,
    inventory: &mut Inventory,
) -> Option<Result<Vec<Event>, ActionError>> {
    match mana {
        ManaPayment::Invocation(card) => Some(Ok(invocation_events(player, card).to_vec())),
        ManaPayment::Polarized { from, source } => {
            let mut events = vec![Event::SkillUsed {
                player,
                skill: Skill::Polarization,
                option: 0,
                effect: Effect::Nothing,
            }];
            match source {
                PolarSource::Die(die) => events.push(Event::DieTaken {
                    player,
                    die,
                    color: from,
                }),
                PolarSource::Crystal => {
                    let ManaColor::Basic(color) = from else {
                        return Some(Err(ActionError::NotLegal));
                    };
                    inventory.remove(color);
                    events.push(Event::CrystalUsed { player, color });
                }
                PolarSource::Token => events.push(Event::ManaTokenUsed {
                    player,
                    color: from,
                }),
                PolarSource::Invocation(card) => events.extend(invocation_events(player, card)),
                PolarSource::StolenDie => events.push(Event::StolenDieUsed { player }),
            }
            Some(Ok(events))
        }
        _ => None,
    }
}

/// Les événements qui paient un mana de `color` par `mana`, Invocation et Polarisation comprises
/// (spec 058).
pub(in crate::engine) fn payment_events(
    state: &GameState,
    p: &PlayerState,
    player: PlayerId,
    mana: ManaPayment,
    color: BasicColor,
    inventory: &mut Inventory,
) -> Result<Vec<Event>, ActionError> {
    match skill_payment_events(player, mana, inventory) {
        Some(events) => events,
        None => Ok(vec![payment_event(
            state, p, player, mana, color, inventory,
        )?]),
    }
}

/// Les événements qui paient un mana noir par `payment`, Invocation et or polarisé compris
/// (spec 058).
pub(super) fn black_payment_events(
    state: &GameState,
    player: PlayerId,
    payment: ManaPayment,
) -> Result<Vec<Event>, ActionError> {
    let mut inventory = Inventory::default();
    match skill_payment_events(player, payment, &mut inventory) {
        Some(events) => events,
        None => Ok(vec![black_payment_event(state, player, payment)?]),
    }
}

/// L'événement qui paie un mana de `color` par `mana` (MKUE p. 5, « Using mana » ; spec 025) ;
/// `inventory` perd le cristal dépensé.
fn payment_event(
    state: &GameState,
    p: &PlayerState,
    player: PlayerId,
    mana: ManaPayment,
    color: BasicColor,
    inventory: &mut Inventory,
) -> Result<Event, ActionError> {
    Ok(match mana {
        ManaPayment::Die(die) => Event::DieTaken {
            player,
            die,
            color: *state
                .source
                .get(usize::from(die))
                .ok_or(ActionError::NotLegal)?,
        },
        ManaPayment::Crystal => {
            inventory.remove(color);
            Event::CrystalUsed { player, color }
        }
        ManaPayment::Token => Event::ManaTokenUsed {
            player,
            color: token_paying(p, color, p.combat_time(state.time_of_day))
                .ok_or(ActionError::NotLegal)?,
        },
        ManaPayment::Endless(color) => Event::EndlessManaUsed { player, color },
        ManaPayment::StolenDie => Event::StolenDieUsed { player },
        ManaPayment::Invocation(_) | ManaPayment::Polarized { .. } => {
            return Err(ActionError::NotLegal);
        }
    })
}

/// Le paiement d'un mana noir (spec 053) : un dé de la Source ou un jeton ; il n'existe pas de
/// cristal noir.
fn black_payment_event(
    state: &GameState,
    player: PlayerId,
    payment: ManaPayment,
) -> Result<Event, ActionError> {
    Ok(match payment {
        ManaPayment::Die(die) => Event::DieTaken {
            player,
            die,
            color: *state
                .source
                .get(usize::from(die))
                .ok_or(ActionError::NotLegal)?,
        },
        ManaPayment::Token => Event::ManaTokenUsed {
            player,
            color: ManaColor::Black,
        },
        ManaPayment::Endless(color) => Event::EndlessManaUsed { player, color },
        // Spec 060 : le dé volé est d'une couleur de base, jamais noir.
        ManaPayment::Crystal
        | ManaPayment::Invocation(_)
        | ManaPayment::Polarized { .. }
        | ManaPayment::StolenDie => {
            return Err(ActionError::NotLegal);
        }
    })
}

/// Les événements d'une unité activée (spec 042) : `UnitActivated`, le mana payé (la couleur de
/// la capacité, puis le supplément), puis l'effet, comme pour une carte ; la cible rendue
/// disponible ; la Blessure des Épéistes.
pub(in crate::engine) fn activation_events(
    state: &GameState,
    player: PlayerId,
    action: &Action,
) -> Result<Vec<Event>, ActionError> {
    let Action::ActivateUnit {
        unit,
        ability,
        option,
        mana,
        target,
        enemy,
        leadership,
    } = action
    else {
        return Err(ActionError::NotLegal);
    };
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let definition = unit
        .unit
        .definition()
        .abilities
        .get(usize::from(*ability))
        .ok_or(ActionError::NotLegal)?;
    let choice = definition
        .options()
        .get(usize::from(*option))
        .ok_or(ActionError::NotLegal)?;
    // Spec 058 : Charisme, utilisé avec l'activation ; le bonus va à la capacité.
    let mut events = Vec::new();
    let effect = if *leadership {
        let phase = p.combat.as_ref().map(|combat| combat.phase);
        let boosted = leadership_effect(choice.effect, phase).ok_or(ActionError::NotLegal)?;
        events.push(Event::SkillUsed {
            player,
            skill: Skill::Leadership,
            option: 0,
            effect: Effect::Nothing,
        });
        boosted
    } else {
        choice.effect
    };
    events.push(Event::UnitActivated {
        enemy: *enemy,
        player,
        unit: *unit,
        ability: *ability,
        option: *option,
        effect,
    });
    let mut colors = Vec::new();
    if let Some(ManaColor::Basic(color)) = definition.mana {
        colors.push(color);
    }
    colors.extend(&choice.extra_mana);
    let mut inventory = p.inventory;
    let mut paying = p.clone();
    // Spec 053 : le mana noir, un dé ou un jeton.
    let mut mana = mana.as_slice();
    if definition.mana == Some(ManaColor::Black) {
        let (&payment, rest) = mana.split_first().ok_or(ActionError::NotLegal)?;
        events.extend(black_payment_events(state, player, payment)?);
        paying = after_payment(state, &paying, payment, ManaColor::Black);
        mana = rest;
    }
    for (&payment, &color) in mana.iter().zip(&colors) {
        events.extend(payment_events(
            state,
            &paying,
            player,
            payment,
            color,
            &mut inventory,
        )?);
        paying = after_payment(state, &paying, payment, ManaColor::Basic(color));
    }
    effect_events(&mut events, &mut inventory, p, player, effect);
    if let Some(target) = target {
        events.push(Event::UnitReadied {
            player,
            unit: *target,
        });
    }
    // Compendium, *UTEM SWORDSMEN* S2 : une Blessure aussitôt, pas un dégât assigné.
    if definition.wounds_unit() {
        events.push(Event::UnitWounded {
            player,
            unit: *unit,
            wounds: 1,
        });
    }
    Ok(events)
}

/// Spec 058 : ce que fait la cible d'une compétence (validée par les actions légales).
pub(in crate::engine) fn skill_target_events(
    state: &GameState,
    player: PlayerId,
    effect: Effect,
    target: Option<SkillTarget>,
) -> Vec<Event> {
    let mut events = Vec::new();
    let Some(p) = state.player(player) else {
        return events;
    };
    // Texte du Vol : un déplacement d'un coup, sans provoquer ; l'arrivée révèle ce qu'une arrivée
    // ordinaire révèle (comme Ailes du vent, spec 057).
    if let (Effect::Flight { cost, .. }, Some(SkillTarget::Space(to)), Some(from)) =
        (effect, target, p.figure)
    {
        events.push(Event::HeroMoved {
            player,
            from,
            to,
            cost,
        });
        events.extend(ruin_reveal(state, to));
        events.extend(reveals_next_to(state, reveal_time(state, player), to, &[]));
    }
    // Texte de Surcharge de mana : un jeton de la couleur, pas forcément dépensé (S5).
    if let (Effect::ManaOverload, Some(SkillTarget::Color(color))) = (effect, target) {
        events.push(Event::ManaTokenGained { player, color });
    }
    // Texte de Botte secrète : Armure −1 par résistance, comptées maintenant.
    if let (Effect::ResistanceBreak, Some(SkillTarget::Enemy(enemy))) = (effect, target)
        && let Some(fought) = p.combat.as_ref().and_then(|combat| combat.enemy(enemy))
    {
        events.push(Event::ResistanceBroken {
            player,
            enemy,
            by: u8::try_from(fought.effective_resistances().len()).unwrap_or(u8::MAX),
        });
    }
    events
}
