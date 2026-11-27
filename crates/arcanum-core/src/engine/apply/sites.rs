//! `apply` : sites, interaction, offres, achats et artefacts.

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn site_event(state: &mut GameState, event: &Event) {
    match event {
        Event::CommonDeckShuffled { deck, order } => {
            state.common_decks.insert(*deck, order.clone());
        }
        Event::OfferCardDealt { offer, card } => {
            // Seul le dessus de sa pioche entre dans l'offre.
            if let Some(pile) = state.common_decks.get_mut(&card.deck())
                && pile.first() == Some(card)
            {
                pile.remove(0);
                let cards = state.offers.entry(*offer).or_default();
                match offer {
                    Offer::AdvancedActions | Offer::Spells => cards.insert(0, *card),
                    Offer::Units => cards.push(*card),
                }
            }
        }
        Event::OfferCardReturned { offer, card } => {
            if take_from_offer(state, *offer, *card) {
                state
                    .common_decks
                    .entry(card.deck())
                    .or_default()
                    .push(*card);
            }
        }
        Event::OfferCardGained {
            player,
            offer,
            card,
            to,
        } => {
            if take_from_offer(state, *offer, *card)
                && let Some(p) = state.player_mut(*player)
            {
                // Spec 056 : là où la carte qui la fait gagner le dit.
                if let Some(deed) = card.deed() {
                    match to {
                        CardDestination::DeckTop => p.deck.put_on_top(deed),
                        CardDestination::DiscardPile => p.discard_pile.push(deed),
                        CardDestination::Hand => p.hand.push(deed),
                    }
                }
                // Les choix de fin de tour ne prennent que sur le deck.
                if *to == CardDestination::DeckTop {
                    match offer {
                        Offer::AdvancedActions => {
                            p.advanced_action_choices = p.advanced_action_choices.saturating_sub(1);
                            p.lowest_advanced_action_only = false;
                        }
                        Offer::Spells => p.spell_choices = p.spell_choices.saturating_sub(1),
                        Offer::Units => {}
                    }
                }
            }
        }
        Event::InteractionStarted {
            player,
            site,
            bonus,
            city_shields,
        } => {
            if let Some(p) = state.player_mut(*player) {
                // MKUE p. 6, 6.e : l'action commence, le mouvement restant est perdu ; p. 8,
                // 2.a : le bonus de réputation, une fois. Plus de repos (p. 6, 7). Variante
                // maison (issue #95) : l'influence déjà jouée reste, pour que l'ordre des clics
                // ne coûte rien ; les autres actions (combat, incendie, aventure) la perdent.
                p.move_points = 0;
                p.influence_points += i32::from(*bonus) + i32::from(*city_shields);
                p.interaction = Some(Interaction {
                    site: *site,
                    bonus: *bonus,
                    city_shields: *city_shields,
                });
                p.moved_or_fought = true;
            }
        }
        // Spec 055 : l'or de la Poche de mana attend le choix de la couleur, s'il en reste une non
        // pleine.
        Event::ArtifactDieRolled {
            player,
            card,
            color,
        } => {
            if let Some(p) = state.player_mut(*player)
                && card.card() == DeedCard::Artifact(Artifact::EndlessGemPouch)
                && *color == ManaColor::Gold
                && BasicColor::ALL
                    .into_iter()
                    .any(|color| !p.inventory.is_full(color))
            {
                p.pouch_choices += 1;
            }
        }
        Event::HealingBought { player, cost } => {
            if let Some(p) = state.player_mut(*player) {
                p.spend_purchase_influence(*cost);
                p.healing += 1;
            }
        }
        Event::VillagePlundered { player } => {
            if let Some(p) = state.player_mut(*player) {
                p.plundered = true;
            }
        }
        Event::ArtifactsDrawn { player, cards } => {
            let pile = state.common_decks.entry(CommonDeck::Artifacts).or_default();
            let mut drawn = Vec::new();
            for &card in cards {
                if pile.first() == Some(&CommonCard::Deed(card)) {
                    pile.remove(0);
                    drawn.push(card);
                }
            }
            if let Some(p) = state.player_mut(*player) {
                // Spec 045 : autant d'artefacts à garder que de récompenses.
                p.artifacts_to_keep = p
                    .artifact_rewards
                    .min(u8::try_from(drawn.len()).unwrap_or(u8::MAX));
                p.drawn_artifacts.extend(drawn);
                p.artifact_rewards = 0;
            }
        }
        Event::ArtifactTaken { player, card } => {
            if let Some(p) = state.player_mut(*player)
                && take_from_hand(&mut p.drawn_artifacts, *card)
            {
                p.deck.put_on_top(*card);
                p.artifacts_to_keep = p.artifacts_to_keep.saturating_sub(1);
            }
        }
        Event::ArtifactReturned { card } => {
            let returned = state
                .players
                .iter_mut()
                .any(|p| take_from_hand(&mut p.drawn_artifacts, *card));
            if returned {
                state
                    .common_decks
                    .entry(CommonDeck::Artifacts)
                    .or_default()
                    .push(CommonCard::Deed(*card));
            }
        }
        Event::MonasteryBurnStarted { player, hex } => {
            if let Some(p) = state.player_mut(*player) {
                // MKUE p. 6, 6.e : l'action commence, comme une interaction ou un défi.
                p.move_points = 0;
                p.influence_points = 0;
                p.movement_ended = true;
                p.moved_or_fought = true;
                p.burning = Some(*hex);
            }
        }
        Event::MonasteryBurned { player, hex } => {
            state.shields.insert(*hex, *player);
            if let Some(p) = state.player_mut(*player) {
                p.artifact_rewards += 1;
            }
        }
        Event::SiteConquered { player, hex } => apply_site_conquered(state, player, hex),
        // Spec 062 : les Boucliers et la conquête d'une cité.
        Event::CityShieldPlaced { player, hex, .. } => {
            if let Some(city) = state.cities.get_mut(hex) {
                city.shields.push(*player);
            }
        }
        // Spec 062 : les achats des cités.
        Event::AdvancedActionDrawnBlind { player, card, cost } => {
            if let Some(pile) = state.common_decks.get_mut(&CommonDeck::AdvancedActions)
                && pile.first() == Some(&CommonCard::Deed(*card))
            {
                pile.remove(0);
                if let Some(p) = state.player_mut(*player) {
                    p.deck.put_on_top(*card);
                    p.spend_purchase_influence(*cost);
                }
            }
        }
        Event::ArtifactBought { player, cost } => {
            if let Some(p) = state.player_mut(*player) {
                p.artifact_rewards += 1;
                p.spend_purchase_influence(*cost);
            }
        }
        Event::EliteUnitAdded { player, card, cost } => {
            if let Some(pile) = state.common_decks.get_mut(&CommonDeck::EliteUnits)
                && pile.first() == Some(card)
            {
                pile.remove(0);
                state.offers.entry(Offer::Units).or_default().push(*card);
                if let Some(p) = state.player_mut(*player) {
                    p.spend_purchase_influence(*cost);
                }
            }
        }
        Event::CityConquered { hex, leader } => {
            if let Some(city) = state.cities.get_mut(hex) {
                city.leader = Some(*leader);
            }
        }
        Event::AdventureSiteEntered { player, hex } => {
            if let Some(p) = state.player_mut(*player) {
                // MKUE p. 6, 6.e : l'action commence, comme un incendie.
                p.move_points = 0;
                p.influence_points = 0;
                p.movement_ended = true;
                p.moved_or_fought = true;
                p.adventure = Some(*hex);
            }
            let night = matches!(
                state.map.space(*hex).and_then(|space| space.feature),
                Some(Feature::Site(Site::Dungeon | Site::Tomb))
            );
            if let Some(p) = state.player_mut(*player) {
                p.night_adventure = night;
            }
        }
        // MKUE p. 8, 4.c : l'offre d'unités n'est pas complétée.
        Event::SpellBought { player, card, cost } => {
            if take_from_offer(state, Offer::Spells, CommonCard::Deed(*card))
                && let Some(p) = state.player_mut(*player)
            {
                p.deck.put_on_top(*card);
                p.spend_purchase_influence(*cost);
            }
        }
        // Spec 062 : depuis l'offre d'actions avancées dans la cité verte.
        Event::AdvancedActionBought { player, card, cost } => {
            if (take_from_offer(state, Offer::Units, CommonCard::Deed(*card))
                || take_from_offer(state, Offer::AdvancedActions, CommonCard::Deed(*card)))
                && let Some(p) = state.player_mut(*player)
            {
                p.deck.put_on_top(*card);
                p.spend_purchase_influence(*cost);
            }
        }
        Event::AltarTributePaid { player, hex } => {
            if let Some(p) = state.player_mut(*player) {
                // MKUE p. 6, 6.e : l'action du tour, comme l'entrée dans un site d'aventure.
                p.move_points = 0;
                p.influence_points = 0;
                p.movement_ended = true;
                p.moved_or_fought = true;
                p.adventure = Some(*hex);
            }
        }
        Event::AdvancedActionLearned { player, cost, .. } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(index) = p.modifiers.iter().position(|modifier| {
                    matches!(modifier.kind, ModifierKind::LearningDiscount { .. })
                })
            {
                p.modifiers.remove(index);
                p.influence_points -= i32::from(*cost);
            }
        }
        _ => {}
    }
}

/// `Event::SiteConquered` : le site conquis et ses récompenses.
fn apply_site_conquered(state: &mut GameState, player: &PlayerId, hex: &Hex) {
    state.shields.insert(*hex, *player);
    // Spec 045 : le jeton de ruines est défaussé (Compendium, *Miscellaneous* S6), sa
    // récompense attend la fin du tour.
    let ruin_rewards = state.ruins.remove(hex).map(|placed| {
        state.ruin_discard.push(placed.token);
        placed.token.definition().rewards().to_vec()
    });
    if let Some(p) = state.player_mut(*player) {
        for reward in ruin_rewards.unwrap_or_default() {
            match reward {
                RuinReward::Artifact => p.artifact_rewards += 1,
                RuinReward::Spell => p.spell_rewards += 1,
                RuinReward::AdvancedAction => p.advanced_action_rewards += 1,
                RuinReward::CrystalSet => p.crystal_set_rewards += 1,
                RuinReward::Unit => p.unit_rewards += 1,
            }
        }
    }
    // Récompenses prises à la fin du tour (MKUE p. 11, 5 ; cartes de site) : (sorts,
    // artefacts, cristaux aléatoires). Le donjon attend son dé de récompense.
    let rewards = match state.map.space(*hex).and_then(|space| space.feature) {
        Some(Feature::Site(Site::MageTower)) => (1, 0, 0),
        Some(Feature::Site(Site::Tomb)) => (1, 1, 0),
        Some(Feature::Site(Site::MonsterDen)) => (0, 0, 2),
        Some(Feature::Site(Site::SpawningGrounds)) => (0, 1, 3),
        _ => (0, 0, 0),
    };
    if let Some(p) = state.player_mut(*player) {
        p.spell_rewards += rewards.0;
        p.artifact_rewards += rewards.1;
        p.crystal_rewards += rewards.2;
    }
}
