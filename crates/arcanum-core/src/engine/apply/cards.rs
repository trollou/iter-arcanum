//! `apply` : deck, main, cartes jouées, défausse et Blessures.

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn card_event(state: &mut GameState, event: &Event) {
    match event {
        Event::DeckShuffled { player, order } => {
            if let Some(p) = state.player_mut(*player) {
                // Spec 023 : la main et la défausse sont dans `order` ; vides à la mise en place.
                p.deck = DeedDeck::new(order.clone());
                p.hand.clear();
                p.discard_pile.clear();
                // Spec 057 : la carte écartée (Distorsion temporelle) est dans `order`.
                p.set_aside.clear();
            }
        }
        Event::CardDrawn { player, card } => {
            if let Some(p) = state.player_mut(*player) {
                p.deck.remove_top();
                p.hand.push(*card);
            }
        }
        Event::CardPlayed {
            player,
            card,
            play,
            effect,
        } => apply_card_played(state, player, card, play, effect),
        Event::CardDiscarded { player, card } => {
            if let Some(p) = state.player_mut(*player)
                && take_from_hand(&mut p.hand, *card)
            {
                p.discard_pile.push(*card);
            }
        }
        Event::CardReturnedToHand { player, card } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(index) = p.play_area.iter().position(|played| played.card == *card)
            {
                p.play_area.remove(index);
                p.hand.push(*card);
            }
        }
        Event::DeckReshuffled { player, order } => {
            if let Some(p) = state.player_mut(*player) {
                p.discard_pile.retain(|card| !order.contains(card));
                p.deck = DeedDeck::new(order.clone());
            }
        }
        // MKUE p. 11, 3.c : les Blessures jetées quittent la partie (pile illimitée).
        Event::WoundsHealed { player, wounds } => {
            if let Some(p) = state.player_mut(*player) {
                // Spec 042 (Q1) : 1 point de soin par Blessure.
                p.healing = p.healing.saturating_sub(wounds.len() as u32);
                spend_grail_fame(p, wounds.len() as u32);
                // Spec 055 (C20) : une Blessure soignée n'est plus à jeter.
                p.wounds_received_hand = p.wounds_received_hand.saturating_sub(wounds.len() as u32);
                for wound in wounds {
                    take_from_hand(&mut p.hand, *wound);
                }
            }
        }
        Event::SteadyTempoKept { player, card, top } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(index) = p.play_area.iter().position(|played| played.card == *card)
            {
                p.play_area.remove(index);
                if *top {
                    p.deck.put_on_top(*card);
                } else {
                    p.deck.put_under(*card);
                }
            }
        }
        Event::WoundThrownAway { player, from } => {
            if let Some(p) = state.player_mut(*player) {
                let pile = match from {
                    WoundPile::Hand => &mut p.hand,
                    WoundPile::DiscardPile => &mut p.discard_pile,
                };
                if let Some(index) = pile.iter().position(|card| card.card().is_wound()) {
                    pile.remove(index);
                }
            }
        }
        // Spec 056 : écrits avec chaque carte.
        Event::CardDestroyed { player, card, from } => {
            if let Some(p) = state.player_mut(*player) {
                match from {
                    DestroyedFrom::Hand => {
                        take_from_hand(&mut p.hand, *card);
                    }
                    DestroyedFrom::PlayArea => {
                        if let Some(index) =
                            p.play_area.iter().position(|played| played.card == *card)
                        {
                            p.play_area.remove(index);
                            p.played_elsewhere = true;
                        }
                    }
                }
            }
        }
        // Spec 057 : Distorsion temporelle.
        Event::CardsReturnedToHand { player, cards } => {
            if let Some(p) = state.player_mut(*player) {
                p.play_area.retain(|played| !cards.contains(&played.card));
                p.hand.extend(cards.iter().copied());
                p.extra_turn_pending = true;
            }
        }
        Event::CardSetAside { player, card } => {
            if let Some(p) = state.player_mut(*player) {
                p.play_area.retain(|played| played.card != *card);
                p.set_aside.push(*card);
                p.extra_turn_pending = true;
            }
        }
        Event::DiscardPicked { player, cards } => {
            if let Some(p) = state.player_mut(*player) {
                p.meditation = Some(Meditation {
                    cards: cards.clone(),
                    pick: cards.len(),
                });
            }
        }
        Event::CardsPlacedOnDeck { player, top, order } => {
            if let Some(p) = state.player_mut(*player) {
                p.discard_pile.retain(|card| !order.contains(card));
                // La première de `order` est la plus haute.
                if *top {
                    for &card in order.iter().rev() {
                        p.deck.put_on_top(card);
                    }
                } else {
                    for &card in order {
                        p.deck.put_under(card);
                    }
                }
                p.meditation = None;
            }
        }
        Event::DrawBonusUsed { player } => {
            if let Some(p) = state.player_mut(*player) {
                p.modifiers
                    .retain(|modifier| !matches!(modifier.kind, ModifierKind::DrawBonus(_)));
            }
        }
        _ => {}
    }
}

/// `Event::CardPlayed` : la carte jouée et son effet.
fn apply_card_played(
    state: &mut GameState,
    player: &PlayerId,
    card: &CardId,
    play: &CardPlay,
    effect: &Effect,
) {
    let claimed_color = claimed_die(play)
        .and_then(|die| state.source.get(usize::from(die)))
        .copied();
    let time_of_day = state.time_of_day;
    // Spec 054 : un sort lancé depuis l'offre y reste, ni en main ni dans la zone de jeu ;
    // il retire le modificateur de Don pour la magie.
    let from_offer = matches!(
        play,
        CardPlay::Cast {
            from_offer: true,
            ..
        }
    );
    if let Some(p) = state.player_mut(*player)
        && (from_offer || take_from_hand(&mut p.hand, *card))
    {
        // Spec 055 (Q3) : les sorts lancés ce tour, pour les anneaux.
        if matches!(play, CardPlay::Cast { .. })
            && let Some(definition) = card.card().definition()
        {
            p.spells_cast.push(definition.color);
        }
        if from_offer {
            let color = card.card().definition().map(|definition| definition.color);
            let used = p
                .modifiers
                .iter()
                .position(|modifier| Some(modifier.kind) == color.map(ModifierKind::CastFromOffer));
            if let Some(index) = used {
                p.modifiers.remove(index);
            }
        } else {
            p.play_area.push(PlayedCard {
                card: *card,
                play: *play,
                effect: *effect,
            });
        }
        // Spec 054 : Don pour la magie, un sort de la couleur de la carte défaussée.
        if let (
            Effect::CastFromOffer,
            CardPlay::Basic {
                discard: Some(discarded),
                ..
            },
        ) = (effect, play)
            && let Some(definition) = discarded.card().definition()
        {
            p.modifiers.push(Modifier {
                kind: ModifierKind::CastFromOffer(definition.color),
                source: ModifierSource::Card(*card),
                duration: ModifierDuration::Turn,
            });
        }
        // Spec 057 : Puits de mana, un jeton de la couleur du dé à chaque tour de la manche.
        if *effect == (Effect::ClaimDie { each_turn: true })
            && let Some(ManaColor::Basic(color)) = claimed_color
        {
            p.modifiers.push(Modifier {
                kind: ModifierKind::ManaClaimPerTurn(color),
                source: ModifierSource::Card(*card),
                duration: ModifierDuration::Round,
            });
        }
        // Spec 057 : Transe, deux cartes choisies dans toute la défausse (moins s'il y en
        // a moins).
        if *effect == (Effect::DiscardToDeck { random: false }) && !p.discard_pile.is_empty() {
            p.meditation = Some(Meditation {
                cards: p.discard_pile.clone(),
                pick: p.discard_pile.len().min(MEDITATION_CARDS),
            });
        }
        // MKUE p. 4, 4 : les effets semblables s'additionnent. Les points d'Attaque vont
        // à l'attaque en cours, ceux de Parade à la parade en cours, sans réserve (MKUE
        // p. 9, *Ranged and Siege Attack Phase* 8 et *Block Phase* 10 ; spec 031).
        // Le mana gagné arrive par les événements qui suivent (spec 025).
        // Spec 052 : le bonus d'Embuscade s'ajoute à la première carte d'Attaque ou de
        // Parade, de côté comprise (Compendium, *Ambush* S2, S4).
        let effect = with_ambush_bonus(p, *effect);
        let effect = with_attack_changes(&p.modifiers, p.combat.as_ref(), effect);
        if let Some(combat) = p.combat.as_mut() {
            combat.add_points(*card, effect);
        }
        add_effect_points(p, effect);
        // Spec 052 (ADR 0008) : les modificateurs de l'option jouée ; spec 055 (L6) : les
        // règles de jour seulement de nuit, le mana noir seulement de jour.
        let source = ModifierSource::Card(*card);
        let kinds: Vec<ModifierKind> = card
            .modifiers(*play)
            .iter()
            .copied()
            .filter(|kind| match kind {
                ModifierKind::DayRules => time_of_day == TimeOfDay::Night,
                ModifierKind::NightMana => time_of_day == TimeOfDay::Day,
                _ => true,
            })
            .collect();
        add_modifiers(p, &kinds, source);
        // Spec 054 : ce que posera la parade en cours si elle réussit (Bouclier ardent et
        // explosif).
        if let Some(combat) = p.combat.as_mut()
            && matches!(combat.in_progress, Some(crate::InProgress::Block { .. }))
        {
            combat.on_block_success.extend(
                card.modifiers(*play)
                    .iter()
                    .filter(|kind| {
                        matches!(
                            kind,
                            ModifierKind::DeferredAttack { .. } | ModifierKind::DestroyBlockedEnemy
                        )
                    })
                    .map(|&kind| (kind, source)),
            );
        }
        // Spec 054 : Démolition, les défenseurs ne sont plus fortifiés par leur site ; le
        // combat finit avec le tour, l'assaut suivant les retrouve fortifiés.
        if let Some(combat) = p.combat.as_mut()
            && card
                .modifiers(*play)
                .contains(&ModifierKind::IgnoreSiteFortifications)
        {
            for enemy in &mut combat.enemies {
                enemy.site_fortified = false;
            }
        }
        // Spec 054 : un sort pose ses modificateurs d'ennemi sur l'ennemi visé ou sur tous.
        if let (Some(combat), Some(definition)) = (p.combat.as_mut(), card.card().definition()) {
            let target = match play {
                CardPlay::Cast { enemy, .. } => *enemy,
                _ => None,
            };
            combat.pose_spell(target, card.modifiers(*play), source, definition.color);
        }
        // Spec 055 : l'Épée de justice renforcée, le doublement et la résistance perdue.
        if let (DeedCard::Artifact(_), Some(combat)) = (card.card(), p.combat.as_mut()) {
            let modifiers = card.modifiers(*play);
            if modifiers.contains(&ModifierKind::DoublePhysicalAttacks) {
                combat.double_physical = true;
            }
            combat.pose_spell(
                None,
                modifiers,
                ModifierSource::Card(*card),
                BasicColor::White,
            );
        }
        // Spec 055 : Bannière de peur renforcée, chaque ennemi choisi n'attaque pas.
        if let (Effect::CancelAttacks(_), CardPlay::ThrowAway { choices, .. }) = (effect, play)
            && let Some(combat) = p.combat.as_mut()
        {
            for enemy in choices.enemies.iter() {
                combat.pose_on_enemy(
                    enemy,
                    &[ModifierKind::OnEnemy(EnemyModifierKind::DoesNotAttack)],
                    ModifierSource::Card(*card),
                    None,
                );
            }
        }
        // Spec 057 : Ailes de la nuit, chaque ennemi visé n'attaque pas ; le Mouvement payé.
        if effect == Effect::DoesNotAttackPerMove
            && let CardPlay::Cast { targets, .. } = play
        {
            p.move_points = p
                .move_points
                .saturating_sub(wings_of_night_cost(targets.enemies.len()));
            if let Some(combat) = p.combat.as_mut() {
                for enemy in targets.enemies.iter() {
                    combat.pose_spell(
                        Some(enemy),
                        &[ModifierKind::OnTargetEnemy(
                            EnemyModifierKind::DoesNotAttack,
                        )],
                        source,
                        BasicColor::White,
                    );
                }
            }
        }
        // Spec 053 : sur l'ennemi de la parade en cours, de la couleur de la carte, même
        // jouée par Concentration (FAQ p2).
        if let Some(combat) = p.combat.as_mut()
            && let Some(crate::InProgress::Block { target, .. }) = combat.in_progress
        {
            let color = card.card().definition().map(|definition| definition.color);
            combat.pose_on_enemy(target, card.modifiers(*play), source, color);
        }
    }
}

/// Pose les modificateurs `kinds` venus de `source`, pour le tour (spec 052, ADR 0008 : toutes les
/// cartes de la spec 052 durent le tour).
/// L'effet d'une carte avec le bonus d'Embuscade en attente, consommé s'il s'applique (une Attaque
/// ou une Parade ; spec 052).
fn with_ambush_bonus(p: &mut PlayerState, effect: Effect) -> Effect {
    let Some(index) = p
        .modifiers
        .iter()
        .position(|modifier| matches!(modifier.kind, ModifierKind::NextCardBonus { .. }))
    else {
        return effect;
    };
    let ModifierKind::NextCardBonus { attack, block } = p.modifiers[index].kind else {
        return effect;
    };
    let boosted = match effect {
        Effect::Attack {
            amount,
            element,
            kind,
        } => Effect::Attack {
            amount: amount.saturating_add(attack),
            element,
            kind,
        },
        Effect::Block { amount, element } => Effect::Block {
            amount: amount.saturating_add(block),
            element,
        },
        _ => return effect,
    };
    p.modifiers.remove(index);
    boosted
}
