//! Paiements de mana : dés, jetons, cristaux, et ceux que des compétences permettent.

use super::*;

/// Le joueur une fois `payment` fait pour un mana de `color` : le dé pris, le cristal ou le jeton
/// dépensé (spec 042).
pub(in crate::engine) fn after_payment(
    state: &GameState,
    p: &PlayerState,
    payment: ManaPayment,
    color: ManaColor,
) -> PlayerState {
    let mut after = p.clone();
    match (payment, color) {
        (ManaPayment::Die(die), _) => after.dice_taken.push(die),
        (ManaPayment::Crystal, ManaColor::Basic(color)) => after.inventory.remove(color),
        (ManaPayment::Token, ManaColor::Basic(color)) => {
            if let Some(token) = token_paying(p, color, p.combat_time(state.time_of_day))
                && let Some(index) = after.mana_tokens.iter().position(|t| *t == token)
            {
                after.mana_tokens.remove(index);
            }
        }
        (ManaPayment::Token, color) => {
            if let Some(index) = after.mana_tokens.iter().position(|t| *t == color) {
                after.mana_tokens.remove(index);
            }
        }
        (ManaPayment::Crystal, _) | (ManaPayment::Endless(_), _) => {}
        // Spec 060 : la tactique retournée, le dé dépensé.
        (ManaPayment::StolenDie, _) => stolen_die_spent(&mut after),
        // Spec 058 : la carte défaussée, la compétence utilisée.
        (ManaPayment::Invocation(card), _) => invoked(&mut after, card),
        (ManaPayment::Polarized { from, source }, _) => {
            use_skill_now(&mut after, Skill::Polarization);
            match source {
                PolarSource::Die(die) => after.dice_taken.push(die),
                PolarSource::Crystal => {
                    if let ManaColor::Basic(basic) = from {
                        after.inventory.remove(basic);
                    }
                }
                PolarSource::Token => {
                    if let Some(index) = after.mana_tokens.iter().position(|t| *t == from) {
                        after.mana_tokens.remove(index);
                    }
                }
                PolarSource::Invocation(card) => invoked(&mut after, card),
                PolarSource::StolenDie => stolen_die_spent(&mut after),
            }
        }
    }
    after
}

/// Le dé de Vol de mana dépensé, dans un état provisoire de paiement (spec 060).
fn stolen_die_spent(p: &mut PlayerState) {
    if let Some(tactic) = p.tactic.as_mut() {
        tactic.flipped = true;
    }
}

/// Spec 060 : le dé posé sur Vol de mana, s'il peut encore payer (tactique face visible), et sa
/// face.
pub(in crate::engine) fn stolen_die(state: &GameState, p: &PlayerState) -> Option<(u8, ManaColor)> {
    let tactic = p.tactic.as_ref().filter(|tactic| !tactic.flipped)?;
    let die = tactic.stolen_die?;
    Some((die, *state.source.get(usize::from(die))?))
}

/// Spec 060 (carte) : les dés de couleur de base de la Source que Vol de mana peut prendre.
pub(in crate::engine) fn stealable_dice(state: &GameState, p: &PlayerState) -> Vec<u8> {
    (0u8..)
        .zip(&state.source)
        .filter(|&(die, face)| face.is_basic() && die_free(p, die))
        .map(|(die, _)| die)
        .collect()
}

/// La compétence marquée utilisée, dans un état provisoire de paiement (spec 058).
fn use_skill_now(p: &mut PlayerState, skill: Skill) {
    if let Some(owned) = p.skills.iter_mut().find(|owned| owned.skill == skill) {
        owned.state = SkillState::UsedThisTurn;
    }
}

/// La carte d'Invocation sortie de la main, dans un état provisoire de paiement (spec 058).
fn invoked(p: &mut PlayerState, card: CardId) {
    use_skill_now(p, Skill::Invocation);
    if let Some(index) = p.hand.iter().position(|held| *held == card) {
        p.hand.remove(index);
    }
}

/// Vrai si le dé `die` de la Source est libre pour ce joueur : ni pris, ni mis sur une couleur ce
/// tour (spec 025), ni gardé par Puits de mana (spec 057).
pub(in crate::engine) fn die_free(p: &PlayerState, die: u8) -> bool {
    !p.dice_taken.contains(&die)
        && !p.dice_set.contains(&die)
        && p.kept_die != Some(die)
        // Spec 060 : le dé posé sur Vol de mana n'est plus dans la Source.
        && p.tactic.as_ref().is_none_or(|tactic| tactic.stolen_die != Some(die))
}

/// Les façons de payer un mana noir (spec 053) : de nuit (ou dans un donjon, spec 044), un dé noir
/// de la Source si le dé du tour n'est pas pris, puis un jeton noir ; jamais de jour (MKUE p. 5,
/// 2.b), et il n'existe pas de cristal noir.
pub(super) fn black_payments(state: &GameState, p: &PlayerState) -> Vec<ManaPayment> {
    // Spec 055 : le noir sert le jour avec l'Amulette des ténèbres.
    if p.combat_time(state.time_of_day) == TimeOfDay::Day
        && !p
            .modifiers
            .iter()
            .any(|modifier| modifier.kind == ModifierKind::NightMana)
    {
        return Vec::new();
    }
    let mut payments = Vec::new();
    if p.dice_taken.len() < DICE_PER_TURN + usize::from(p.extra_dice) && !p.source_dice_forbidden {
        payments.extend(
            (0u8..)
                .zip(&state.source)
                .filter(|&(die, face)| *face == ManaColor::Black && die_free(p, die))
                .map(|(die, _)| ManaPayment::Die(die)),
        );
    }
    if p.mana_tokens.contains(&ManaColor::Black) {
        payments.push(ManaPayment::Token);
    }
    // Spec 055 (Compendium, *Rings* S1) : le noir illimité d'un anneau, là où le noir sert.
    if p.modifiers
        .iter()
        .any(|modifier| matches!(modifier.kind, ModifierKind::EndlessMana(_)))
    {
        payments.push(ManaPayment::Endless(ManaColor::Black));
    }
    // Spec 058 : une Blessure par Invocation, là où le noir sert (Compendium, *Invocation* S1).
    payments.extend(invocation_payments(p, ManaColor::Black));
    payments
}

/// Les façons de payer ces couleurs l'une après l'autre (spec 042) : pour chacune, les paiements
/// de `mana_payments` qui restent une fois les précédents faits.
pub(super) fn payment_lists(
    state: &GameState,
    p: &PlayerState,
    colors: &[BasicColor],
) -> Vec<Vec<ManaPayment>> {
    let Some((&color, rest)) = colors.split_first() else {
        return vec![Vec::new()];
    };
    let mut lists = Vec::new();
    for payment in mana_payments(state, p, color) {
        let after = after_payment(state, p, payment, ManaColor::Basic(color));
        lists.extend(
            payment_lists(state, &after, rest)
                .into_iter()
                .map(|mut list| {
                    list.insert(0, payment);
                    list
                }),
        );
    }
    lists
}

/// Les façons de payer un mana de `color` : chaque dé de la Source qui le peut, dans l'ordre,
/// si le dé du tour n'est pas encore pris (MKUE p. 5, 3), puis un cristal (p. 5, 1.b), puis un
/// jeton (spec 025).
pub(super) fn mana_payments(
    state: &GameState,
    p: &PlayerState,
    color: BasicColor,
) -> Vec<ManaPayment> {
    // Spec 044 : règles de Nuit pendant le combat d'un donjon ou d'un tombeau ; spec 055 : l'or
    // sert avec l'Amulette du soleil.
    let time = p.gold_time(p.combat_time(state.time_of_day));
    let mut payments = Vec::new();
    // Spec 025 : les dés de plus relèvent la limite ; un dé mis sur une couleur ne paie rien ;
    // spec 058 : plus aucun après Magie inutile à +3.
    if p.dice_taken.len() < DICE_PER_TURN + usize::from(p.extra_dice) && !p.source_dice_forbidden {
        // Attraction de mana : un dé noir pris vaut toute couleur, même de jour (Q3, L4).
        let black_taken = p
            .dice_taken
            .iter()
            .filter(|&&die| state.source.get(usize::from(die)) == Some(&ManaColor::Black))
            .count();
        let black_any = black_taken < usize::from(p.black_any_dice);
        payments.extend(
            (0u8..)
                .zip(&state.source)
                .filter(|&(die, face)| {
                    die_free(p, die)
                        && ((!face.is_depleted(time) && face.can_pay(color, time))
                            || (*face == ManaColor::Black && black_any)
                            // Spec 056, Q9 : Tempête de mana renforcée, noir et or, même épuisés.
                            || (p.any_color_dice && !face.is_basic()))
                })
                .map(|(die, _)| ManaPayment::Die(die)),
        );
    }
    // Spec 060 (FAQ p1 ; Compendium, *Who Needs Magic* S3) : le dé volé, en plus du dé du tour,
    // même quand la Source est interdite.
    if let Some((_, face)) = stolen_die(state, p)
        && !face.is_depleted(time)
        && face.can_pay(color, time)
    {
        payments.push(ManaPayment::StolenDie);
    }
    if p.inventory.crystals(color) > 0 {
        payments.push(ManaPayment::Crystal);
    }
    if token_paying(p, color, time).is_some() {
        payments.push(ManaPayment::Token);
    }
    // Spec 055 : le mana illimité d'un anneau renforcé.
    if p.modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::EndlessMana(color))
    {
        payments.push(ManaPayment::Endless(ManaColor::Basic(color)));
    }
    // Spec 058 : Invocation, puis Polarisation.
    payments.extend(invocation_payments(p, ManaColor::Basic(color)));
    payments.extend(polarized_payments(state, p, ManaColor::Basic(color)));
    payments
}

/// Vrai si le joueur a cette compétence, prête (spec 058).
fn skill_ready(p: &PlayerState, skill: Skill) -> bool {
    p.skills
        .iter()
        .any(|owned| owned.skill == skill && owned.state == SkillState::Ready)
}

/// Les cartes de la main qu'Invocation défausse pour un mana de `color` (texte ; spec 058) : une
/// Blessure (la première, elles sont identiques) pour le rouge ou le noir, toute autre carte pour
/// le blanc ou le vert.
fn invocation_cards(p: &PlayerState, color: ManaColor) -> Vec<CardId> {
    match color {
        ManaColor::Basic(BasicColor::Red) | ManaColor::Black => p
            .hand
            .iter()
            .copied()
            .find(|card| card.card().is_wound())
            .into_iter()
            .collect(),
        ManaColor::Basic(BasicColor::White | BasicColor::Green) => p
            .hand
            .iter()
            .copied()
            .filter(|card| !card.card().is_wound())
            .collect(),
        _ => Vec::new(),
    }
}

/// Les paiements d'un mana de `color` par Invocation (spec 058), la compétence prête.
fn invocation_payments(p: &PlayerState, color: ManaColor) -> Vec<ManaPayment> {
    if !skill_ready(p, Skill::Invocation) {
        return Vec::new();
    }
    invocation_cards(p, color)
        .into_iter()
        .map(ManaPayment::Invocation)
        .collect()
}

/// Les paiements d'un mana de `color` par Polarisation (texte ; Compendium, *Polarization* S1 à
/// S4), la compétence prête : un mana de la couleur opposée sous toute forme (dé, épuisé compris,
/// cristal, jeton, Invocation) ; de jour, un mana noir pour toute couleur de base ; de nuit, un
/// mana or pour le noir (d'un sort renforcé, seul appel avec le noir).
pub(super) fn polarized_payments(
    state: &GameState,
    p: &PlayerState,
    color: ManaColor,
) -> Vec<ManaPayment> {
    if !skill_ready(p, Skill::Polarization) {
        return Vec::new();
    }
    let day = p.combat_time(state.time_of_day) == TimeOfDay::Day;
    let froms: Vec<ManaColor> = match color {
        ManaColor::Basic(_) if day => vec![color.opposite(), ManaColor::Black],
        ManaColor::Basic(_) => vec![color.opposite()],
        ManaColor::Black if !day => vec![ManaColor::Gold],
        _ => Vec::new(),
    };
    let dice_open =
        p.dice_taken.len() < DICE_PER_TURN + usize::from(p.extra_dice) && !p.source_dice_forbidden;
    let mut payments = Vec::new();
    for from in froms {
        let mut sources = Vec::new();
        if dice_open {
            sources.extend(
                (0u8..)
                    .zip(&state.source)
                    .filter(|&(die, face)| *face == from && die_free(p, die))
                    .map(|(die, _)| PolarSource::Die(die)),
            );
        }
        // Spec 060 : le dé volé, comme un dé.
        if stolen_die(state, p).is_some_and(|(_, face)| face == from) {
            sources.push(PolarSource::StolenDie);
        }
        if let ManaColor::Basic(basic) = from
            && p.inventory.crystals(basic) > 0
        {
            sources.push(PolarSource::Crystal);
        }
        if p.mana_tokens.contains(&from) {
            sources.push(PolarSource::Token);
        }
        if skill_ready(p, Skill::Invocation) {
            sources.extend(
                invocation_cards(p, from)
                    .into_iter()
                    .map(PolarSource::Invocation),
            );
        }
        payments.extend(
            sources
                .into_iter()
                .map(|source| ManaPayment::Polarized { from, source }),
        );
    }
    payments
}

/// Le jeton qui paie un mana de `color` : de cette couleur d'abord, sinon un jeton or de jour
/// (MKUE p. 5, 3.c ; jetons or des Mages d'Altem, spec 042). Un jeton noir ne paie rien encore.
pub(crate) fn token_paying(
    p: &PlayerState,
    color: BasicColor,
    time: TimeOfDay,
) -> Option<ManaColor> {
    let exact = ManaColor::Basic(color);
    if p.mana_tokens.contains(&exact) {
        Some(exact)
    } else if p.gold_time(time) == TimeOfDay::Day && p.mana_tokens.contains(&ManaColor::Gold) {
        Some(ManaColor::Gold)
    } else {
        None
    }
}
