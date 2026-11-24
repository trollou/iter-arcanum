//! Décision des actions des sites : interaction, recrutement, achats, pillage, incendie,
//! aventure, autel.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn site_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    match action {
        Action::Interact => {
            let site = interaction_site(state, player).ok_or(ActionError::NotLegal)?;
            let bonus = state
                .reputation_bonus(player)
                .ok_or(ActionError::NotLegal)?;
            // Spec 062 (MKUE p. 8, 3.a) : +1 par Bouclier du joueur sur la cité.
            let city_shields = state
                .figure(player)
                .and_then(|figure| state.city(figure))
                .map_or(0, |city| city.shields_of(player));
            Ok(vec![Event::InteractionStarted {
                player,
                site,
                bonus,
                city_shields: u8::try_from(city_shields).unwrap_or(u8::MAX),
            }])
        }
        Action::RecruitUnit {
            unit,
            disband,
            bonds,
        } => recruit_events(state, player, unit, disband, bonds),
        Action::BuyHealing => {
            let cost = match state
                .interaction(player)
                .map(|interaction| interaction.site)
            {
                Some(Site::Monastery) => MONASTERY_HEALING_COST,
                _ => VILLAGE_HEALING_COST,
            };
            Ok(vec![Event::HealingBought { player, cost }])
        }
        // Spec 062 (MKUE p. 8, 4.e) : les achats des cités.
        Action::BuyAdvancedActionFromDeck => {
            let card = state
                .common_deck(CommonDeck::AdvancedActions)
                .first()
                .and_then(|card| card.deed())
                .ok_or(ActionError::NotLegal)?;
            Ok(vec![Event::AdvancedActionDrawnBlind {
                player,
                card,
                cost: CITY_ADVANCED_ACTION_COST,
            }])
        }
        Action::BuyArtifact => Ok(vec![Event::ArtifactBought {
            player,
            cost: CITY_ARTIFACT_COST,
        }]),
        Action::AddEliteUnit => {
            let card = *state
                .common_deck(CommonDeck::EliteUnits)
                .first()
                .ok_or(ActionError::NotLegal)?;
            Ok(vec![Event::EliteUnitAdded {
                player,
                card,
                cost: CITY_ELITE_UNIT_COST,
            }])
        }
        Action::BuyAdvancedAction { card } => {
            let mut events = vec![Event::AdvancedActionBought {
                player,
                card: *card,
                cost: MONASTERY_ADVANCED_ACTION_COST,
            }];
            // Spec 062 : dans la cité verte, l'offre d'actions avancées est complétée.
            if matches!(
                state.interaction(player).map(|i| i.site),
                Some(Site::City(_))
            ) {
                let offer = Offer::AdvancedActions;
                let top = state.common_deck(CommonDeck::AdvancedActions).first();
                events.extend(top.map(|&card| Event::OfferCardDealt { offer, card }));
            }
            Ok(events)
        }
        Action::BuySpell { card, mana } => buy_spell_events(state, player, card, mana),
        Action::PlunderVillage => {
            // Carte du village : deux cartes, sans remélange si le deck s'épuise (MKUE p. 11,
            // 7.d).
            let mut events = vec![
                Event::VillagePlundered { player },
                Event::ReputationChanged { player, change: -1 },
            ];
            events.extend(
                state
                    .deck(player)
                    .iter()
                    .take(2)
                    .map(|&card| Event::CardDrawn { player, card }),
            );
            Ok(events)
        }
        Action::EnterAdventureSite => {
            let hex = enterable_adventure_site(state, player).ok_or(ActionError::NotLegal)?;
            Ok(adventure_events(state, player, hex, rngs))
        }
        Action::BurnMonastery => burn_monastery_events(state, player, rngs),
        Action::PayAltarTribute { mana } => altar_tribute_events(state, player, mana),
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}

/// `Action::RecruitUnit` : les unités renvoyées, puis la recrue et son prix.
fn recruit_events(
    state: &GameState,
    player: PlayerId,
    unit: &UnitCard,
    disband: &Option<UnitCard>,
    bonds: &bool,
) -> Result<Vec<Event>, ActionError> {
    let mut events: Vec<Event> = disband
        .iter()
        .map(|&unit| Event::UnitDisbanded { player, unit })
        .collect();
    // Spec 057 : Appel à la gloire, sans Influence.
    let free = state.player(player).is_some_and(free_recruit);
    let influence = unit.unit.definition().influence;
    events.push(Event::UnitRecruited {
        player,
        unit: *unit,
        cost: match (free, bonds) {
            (true, _) => 0,
            (false, true) => bonds_cost(influence),
            (false, false) => influence,
        },
    });
    if *bonds {
        events.push(Event::UnitUnderBonds {
            player,
            unit: *unit,
        });
    }
    Ok(events)
}

/// `Action::BuySpell` : le sort acheté, payé en mana de sa couleur.
fn buy_spell_events(
    state: &GameState,
    player: PlayerId,
    card: &CardId,
    mana: &ManaPayment,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let color = card.card().definition().ok_or(ActionError::NotLegal)?.color;
    let mut inventory = p.inventory;
    let mut events = vec![Event::SpellBought {
        player,
        card: *card,
        cost: SPELL_COST,
    }];
    events.extend(payment_events(
        state,
        p,
        player,
        *mana,
        color,
        &mut inventory,
    )?);
    // Compendium, *Miscellaneous* S7 : l'offre de sorts est complétée aussitôt.
    let offer = Offer::Spells;
    let top = state.common_deck(CommonDeck::Spells).first();
    events.extend(top.map(|&card| Event::OfferCardDealt { offer, card }));
    Ok(events)
}

/// `Action::BurnMonastery` : la réputation perdue et le combat contre un jeton violet.
fn burn_monastery_events(
    state: &GameState,
    player: PlayerId,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let hex = burnable_monastery(state, player).ok_or(ActionError::NotLegal)?;
    // Carte du monastère : Réputation −3 aussitôt, puis un jeton violet, combattu seul
    // (Compendium, *Combat* S17) ; pile de secours au besoin (spec 033).
    let mut events = vec![
        Event::MonasteryBurnStarted { player, hex },
        Event::ReputationChanged { player, change: -3 },
    ];
    let mut piles = state.enemy_piles.clone();
    let mut discards = state.enemy_discards.clone();
    let (drawn, token) = draw_enemy_token(EnemyColor::Violet, &mut piles, &mut discards, rngs);
    events.extend(drawn);
    let enemies = match token {
        Some(token) => {
            events.push(Event::EnemyTokenPlaced { token, hex });
            vec![(hex, token)]
        }
        None => Vec::new(),
    };
    events.push(Event::CombatStarted { player, enemies });
    Ok(events)
}

/// `Action::PayAltarTribute` : le tribut payé, l'autel conquis et sa gloire.
fn altar_tribute_events(
    state: &GameState,
    player: PlayerId,
    mana: &[ManaPayment],
) -> Result<Vec<Event>, ActionError> {
    let (hex, color) = tribute_altar(state, player).ok_or(ActionError::NotLegal)?;
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let mut events = vec![Event::AltarTributePaid { player, hex }];
    let mut inventory = p.inventory;
    let mut paying = p.clone();
    for &payment in mana {
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
    // Q3 : la gloire aussitôt ; le niveau attend la fin du tour (MKUE p. 11, 6).
    events.push(Event::SiteConquered { player, hex });
    events.push(Event::FameGained {
        player,
        fame: ALTAR_FAME,
    });
    Ok(events)
}
