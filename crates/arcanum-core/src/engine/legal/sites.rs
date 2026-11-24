//! Actions légales des sites : interaction, pillage, incendie, sites d'aventure, autel, apprentissage,
//! soins, recrutement et achats.

use super::*;

pub(in crate::engine) fn interaction_site(state: &GameState, player: PlayerId) -> Option<Site> {
    let p = state.player(player)?;
    if turn_action_taken(p) || p.reputation.is_x() {
        return None;
    }
    let figure = p.figure?;
    // Spec 041 : un monastère incendié est une case vide de son terrain.
    match state.map.space(figure)?.feature {
        Some(Feature::Site(Site::Monastery)) if state.is_burned_monastery(figure) => None,
        Some(Feature::Site(site @ (Site::Village | Site::Monastery))) => Some(site),
        // Spec 043 : sa forteresse, une tour de mage conquise (cartes de site).
        Some(Feature::Site(Site::Keep)) if state.shields.get(&figure) == Some(&player) => {
            Some(Site::Keep)
        }
        Some(Feature::Site(Site::MageTower)) if state.shields.contains_key(&figure) => {
            Some(Site::MageTower)
        }
        // Spec 062 : une cité conquise, par n'importe qui (Compendium, *Cities* S19).
        Some(Feature::Site(site @ Site::City(_)))
            if state.city(figure).is_some_and(|city| city.is_conquered()) =>
        {
            Some(site)
        }
        _ => None,
    }
}

/// Vrai si l'action du tour n'est plus possible, ou ne l'est pas encore (interagir, incendier) :
/// interaction, combat, ennemi provoqué ou défié ce tour (MKUE p. 6, 6.b à 6.d) ; une main de
/// Blessures sans carte jouée (Compendium, *Resting* S4) ; l'annonce de fin de manche obligatoire
/// (MKUE p. 6, 3.b).
pub(crate) fn turn_action_taken(p: &PlayerState) -> bool {
    p.interaction.is_some()
        || p.combat.is_some()
        || p.adventure.is_some()
        || !p.enemies_to_fight.is_empty()
        || must_announce_end_of_round(p)
        || (p.play_area.is_empty()
            && !p.hand.is_empty()
            && p.hand.iter().all(|card| card.card().is_wound()))
}

/// Vrai si le héros peut piller le village de sa figurine (carte du village ; Compendium, *Turn
/// Start* S1 et *Miscellaneous* S5 ; spec 041, Q1) : au début du tour, avant toute carte jouée,
/// unité activée, pas ou action, une fois ; réputation X comprise (*Influence* S5). Pas quand
/// l'annonce de fin de manche est obligatoire (MKUE p. 6, 3.b) : il n'y a rien à piocher.
pub(super) fn can_plunder(state: &GameState, player: PlayerId) -> bool {
    let Some(p) = state.player(player) else {
        return false;
    };
    let on_village = p
        .figure
        .and_then(|hex| state.map.space(hex))
        .map(|space| space.feature)
        == Some(Some(Feature::Site(Site::Village)));
    on_village
        && !p.plundered
        && p.play_area.is_empty()
        && !p.unit_activated
        && !p.moved_or_fought
        && p.interaction.is_none()
        && p.combat.is_none()
        && p.enemies_to_fight.is_empty()
        && !must_announce_end_of_round(p)
}

/// Le monastère que le héros peut incendier maintenant (spec 041) : sous la figurine, non incendié,
/// aux mêmes conditions que l'interaction, réputation comprise sur X (MKUE p. 8, *Combat with
/// Enemies* 1.c ; Compendium, *Influence* S5 et S7, *Combat* S17).
pub(in crate::engine) fn burnable_monastery(state: &GameState, player: PlayerId) -> Option<Hex> {
    let p = state.player(player)?;
    let figure = p.figure?;
    let monastery = state.map.space(figure)?.feature == Some(Feature::Site(Site::Monastery));
    (monastery && !turn_action_taken(p) && !state.is_burned_monastery(figure)).then_some(figure)
}

/// Le site d'aventure où le héros peut entrer maintenant (spec 044) : sous la figurine, un donjon
/// ou un tombeau, conquis ou non, un antre ou un Sombre Terrier non conquis (MKUE p. 8, *Combat
/// with Enemies* 1.b ; cartes de site) ; aux mêmes conditions que l'incendie, réputation X comprise
/// (Compendium, *Combat* S40) ; une ruine non conquise à jeton d'ennemis (spec 045).
pub(in crate::engine) fn enterable_adventure_site(
    state: &GameState,
    player: PlayerId,
) -> Option<Hex> {
    let p = state.player(player)?;
    let figure = p.figure?;
    let enterable = match state.map.space(figure)?.feature {
        Some(Feature::Site(Site::Dungeon | Site::Tomb)) => true,
        Some(Feature::Site(Site::MonsterDen | Site::SpawningGrounds)) => {
            !state.shields.contains_key(&figure)
        }
        // Spec 045 : une ruine non conquise à jeton d'ennemis, face visible.
        Some(Feature::Site(Site::Ruins)) => state
            .ruins
            .get(&figure)
            .is_some_and(|placed| placed.face_up && !placed.token.is_altar()),
        _ => false,
    };
    (enterable && !turn_action_taken(p)).then_some(figure)
}

/// La gloire d'une offrande à l'autel (carte *Ancient Ruins*, *Altar*).
pub const ALTAR_FAME: u32 = 7;

/// Le mana d'une offrande, d'une seule couleur (carte *Ancient Ruins*, *Altar*).
const ALTAR_MANA: usize = 3;

/// L'autel où le héros peut faire une offrande maintenant, et sa couleur (spec 045) : la ruine non
/// conquise de la figurine, jeton face visible, aux mêmes conditions que l'entrée dans un site
/// d'aventure (Compendium, *Movement* S11 (d)).
pub(in crate::engine) fn tribute_altar(
    state: &GameState,
    player: PlayerId,
) -> Option<(Hex, BasicColor)> {
    let p = state.player(player)?;
    let figure = p.figure?;
    let placed = state.ruins.get(&figure)?;
    if !placed.face_up || turn_action_taken(p) {
        return None;
    }
    Some((figure, placed.token.definition().altar()?))
}

/// Les paiements de l'offrande : 3 mana de `color` par les moyens ordinaires (Compendium, *Mana*
/// S4 et S12), dés dans la limite du tour (spec 025), cristaux et jetons de l'inventaire. Ordre
/// canonique, sans doublon : dés, puis cristaux, puis jetons ; d'abord le plus de dés, puis le plus
/// de cristaux.
pub(super) fn altar_payments(
    state: &GameState,
    p: &PlayerState,
    color: BasicColor,
) -> Vec<Vec<ManaPayment>> {
    let dice: Vec<ManaPayment> = mana_payments(state, p, color)
        .into_iter()
        .filter(|payment| matches!(payment, ManaPayment::Die(_)))
        .collect();
    let free_dice = (DICE_PER_TURN + usize::from(p.extra_dice)).saturating_sub(p.dice_taken.len());
    let crystals = usize::from(p.inventory.crystals(color));
    let time = p.gold_time(p.combat_time(state.time_of_day));
    let tokens = p
        .mana_tokens
        .iter()
        .filter(|&&token| {
            token == ManaColor::Basic(color) || (time == TimeOfDay::Day && token == ManaColor::Gold)
        })
        .count();
    let mut payments = Vec::new();
    for count in (0..=ALTAR_MANA.min(free_dice).min(dice.len())).rev() {
        for chosen in combinations(&dice, count) {
            for crystal_count in (0..=(ALTAR_MANA - count).min(crystals)).rev() {
                let token_count = ALTAR_MANA - count - crystal_count;
                if token_count > tokens {
                    continue;
                }
                let mut payment = chosen.clone();
                payment.extend(std::iter::repeat_n(ManaPayment::Crystal, crystal_count));
                payment.extend(std::iter::repeat_n(ManaPayment::Token, token_count));
                payments.push(payment);
            }
        }
    }
    payments
}

/// L'entrée dans le site d'aventure `hex` (spec 044) : les jetons combattus (cartes de site), tirés
/// aussitôt, puis le combat sans moment « à engager » (Compendium, *Combat* Q1.d et Q2). Donjon :
/// un jeton marron ; tombeau : un rouge ; antre : le jeton resté, sinon un marron ; Sombre Terrier :
/// les jetons restés, complétés à deux par des marron (Compendium, *Miscellaneous* S15). Sans
/// aucun jeton, le site est conquis sans combat (spec 043, L4).
pub(in crate::engine) fn adventure_events(
    state: &GameState,
    player: PlayerId,
    hex: Hex,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    // Donjon et tombeau : un jeton neuf à chaque entrée ; antre, Sombre Terrier et ruines gardent
    // les leurs.
    let staying = state.enemy_tokens_at(hex);
    let draws: Vec<EnemyColor> = match state.map.space(hex).and_then(|space| space.feature) {
        Some(Feature::Site(Site::Tomb)) => vec![EnemyColor::Red],
        Some(Feature::Site(Site::MonsterDen)) => {
            vec![EnemyColor::Brown; 1usize.saturating_sub(staying.len())]
        }
        Some(Feature::Site(Site::SpawningGrounds)) => {
            vec![EnemyColor::Brown; 2usize.saturating_sub(staying.len())]
        }
        // Spec 045 : à la première entrée, un jeton par couleur du jeton de ruines ; ensuite, les
        // jetons restés seuls, sans remplacement (MKUE p. 10, 3.c).
        Some(Feature::Site(Site::Ruins)) if staying.is_empty() => state
            .ruins
            .get(&hex)
            .map(|placed| placed.token.definition().enemies().to_vec())
            .unwrap_or_default(),
        Some(Feature::Site(Site::Ruins)) => Vec::new(),
        _ => vec![EnemyColor::Brown],
    };
    let keeps = matches!(
        state.map.space(hex).and_then(|space| space.feature),
        Some(Feature::Site(
            Site::MonsterDen | Site::SpawningGrounds | Site::Ruins
        ))
    );
    let staying: &[EnemyToken] = if keeps { staying } else { &[] };
    let mut events = vec![Event::AdventureSiteEntered { player, hex }];
    let mut piles = state.enemy_piles.clone();
    let mut discards = state.enemy_discards.clone();
    let mut enemies: Vec<(Hex, EnemyToken)> = staying.iter().map(|&token| (hex, token)).collect();
    for color in draws {
        let (drawn, token) = draw_enemy_token(color, &mut piles, &mut discards, rngs);
        events.extend(drawn);
        let Some(token) = token else {
            break;
        };
        events.push(Event::EnemyTokenPlaced { token, hex });
        enemies.push((hex, token));
    }
    if enemies.is_empty() {
        events.push(Event::SiteConquered { player, hex });
    } else {
        events.push(Event::CombatStarted { player, enemies });
    }
    events
}

/// L'issue du combat d'un site d'aventure, après `CombatEnded` (spec 044 ; MKUE p. 10 ; cartes de
/// site). Tous ses jetons vaincus : un site non conquis l'est, et le donjon lance son dé de
/// récompense ; un donjon ou un tombeau déjà conquis ne rapporte que la gloire. Sinon, au donjon et
/// au tombeau les jetons non vaincus sont défaussés ; à l'antre et au Sombre Terrier, ils restent.
pub(in crate::engine) fn adventure_outcome_events(
    state: &GameState,
    player: PlayerId,
    hex: Hex,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let Some(combat) = state.combat(player) else {
        return Vec::new();
    };
    let site = state.map.space(hex).and_then(|space| space.feature);
    let night = matches!(site, Some(Feature::Site(Site::Dungeon | Site::Tomb)));
    let standing: Vec<EnemyToken> = combat
        .enemies
        .iter()
        .filter(|enemy| enemy.hex == hex && !enemy.defeated)
        .map(|enemy| enemy.token)
        .collect();
    if !standing.is_empty() {
        if !night {
            return Vec::new();
        }
        return standing
            .into_iter()
            .map(|token| Event::EnemyTokenDiscarded { hex, token })
            .collect();
    }
    if state.shields.contains_key(&hex) {
        return Vec::new();
    }
    let mut events = vec![Event::SiteConquered { player, hex }];
    if site == Some(Feature::Site(Site::Dungeon)) {
        events.push(Event::RewardDieRolled {
            player,
            color: roll_die(rngs.rng(RngUsage::RewardDice)),
        });
    }
    events
}

/// Les recrutements légaux pendant une interaction (spec 042), dans l'ordre de l'offre : une unité
/// dont un lieu de recrutement est le site (MKUE p. 8, 4.a), pour une influence qui atteint son
/// coût ; jetons de commandement tous pris, une action par unité à renvoyer, dans l'ordre de la
/// zone (MKUE p. 5, 4.a). Les jetons sont ceux du niveau, pas du niveau atteint à la fin du tour
/// (FAQ p3).
/// Le modificateur d'Apprentissage en cours, s'il y en a un (spec 056) : son coût et la
/// destination de la carte.
pub(in crate::engine) fn learning_discount(p: &PlayerState) -> Option<(u8, CardDestination)> {
    p.modifiers.iter().find_map(|modifier| match modifier.kind {
        ModifierKind::LearningDiscount { cost, to } => Some((cost, to)),
        _ => None,
    })
}

/// Les achats d'Apprentissage (spec 056, Q7), dans l'ordre de l'offre d'actions avancées : hors
/// combat, tant que l'Influence sert (phase de mouvement ou interaction), avec assez d'Influence.
pub(super) fn learn_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    let Some((cost, _)) = learning_discount(p) else {
        return Vec::new();
    };
    let interacting = p.interaction.is_some();
    let influence_serves = interacting || !movement_over(p.movement_ended, interacting);
    if !matches!(moment(p), Moment::Free)
        || !influence_serves
        || p.influence_points < i32::from(cost)
    {
        return Vec::new();
    }
    state
        .offer(Offer::AdvancedActions)
        .iter()
        .filter_map(|card| card.deed())
        .map(|card| Action::LearnAdvancedAction { card })
        .collect()
}

/// Les dépenses de points de soin possibles, hors combat : une Blessure de la main, puis chaque
/// unité blessée dans l'ordre de la zone (MKUE p. 11, 3.c ; spec 042, Q1).
pub(in crate::engine) fn healing_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    if matches!(moment(p), Moment::Fighting(_)) || p.healing == 0 {
        return Vec::new();
    }
    let mut actions = Vec::new();
    if p.hand.iter().any(|card| card.card().is_wound()) {
        actions.push(Action::SpendHealing {
            on: HealingTarget::Wound,
        });
    }
    actions.extend(
        p.units
            .iter()
            .filter(|owned| {
                owned.is_wounded() && p.healing >= u32::from(owned.card.unit.definition().level)
            })
            .map(|owned| Action::SpendHealing {
                on: HealingTarget::Unit(owned.card),
            }),
    );
    actions
}

/// Les unités sous un jeton de commandement ordinaire : toutes, sauf celle sous le jeton de Loyauté
/// sans faille, qui ne se renvoie pas (spec 058).
pub(super) fn ordinary_units(p: &PlayerState) -> impl Iterator<Item = &OwnedUnit> {
    p.units
        .iter()
        .filter(move |owned| Some(owned.card) != p.bonds_unit)
}

/// Le coût d'une unité recrutée sous le jeton de Loyauté sans faille (texte : 5 de moins) ; l'écran
/// l'affiche sur le bouton (spec 058, Interface).
pub fn bonds_cost(influence: u8) -> u8 {
    influence.saturating_sub(BONDS_DISCOUNT)
}

const BONDS_DISCOUNT: u8 = 5;

/// Les unités régulières ajoutées à l'offre quand Loyauté sans faille est gagnée (texte).
pub(in crate::engine) const BONDS_UNITS: usize = 2;

/// Vrai si Appel à la gloire attend son recrutement gratuit (spec 057).
pub(in crate::engine) fn free_recruit(p: &PlayerState) -> bool {
    p.modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::RecruitFromOffer)
}

/// Le lieu de recrutement d'un site d'interaction : `None` hors lieu de recrutement, `Some(None)`
/// pour toutes les unités (cité blanche, MKUE p. 8, 4.e ; spec 062).
fn recruit_site(site: Site) -> Option<Option<RecruitSite>> {
    match site {
        Site::Village => Some(Some(RecruitSite::Village)),
        Site::Monastery => Some(Some(RecruitSite::Monastery)),
        Site::Keep => Some(Some(RecruitSite::Keep)),
        Site::MageTower => Some(Some(RecruitSite::MageTower)),
        Site::City(BasicColor::White) => Some(None),
        Site::City(_) => Some(Some(RecruitSite::City)),
        _ => None,
    }
}

/// Les unités de l'offre recrutables dans le lieu de l'interaction en cours, dans l'ordre de
/// l'offre, quelle que soit l'Influence (MKUE p. 8, 4.a ; issue privée 341) : l'écran grise les trop
/// chères.
pub(crate) fn recruitable_units(state: &GameState, player: PlayerId) -> Vec<UnitCard> {
    let Some(site) = state
        .player(player)
        .and_then(|p| p.interaction)
        .and_then(|interaction| recruit_site(interaction.site))
    else {
        return Vec::new();
    };
    offered_units(state, site)
}

/// Les unités de l'offre dont un lieu de recrutement est `site` (toutes pour `None`).
fn offered_units(state: &GameState, site: Option<RecruitSite>) -> Vec<UnitCard> {
    state
        .offer(Offer::Units)
        .iter()
        .filter_map(|card| card.unit())
        .filter(|unit| site.is_none_or(|site| unit.unit.definition().recruit.contains(&site)))
        .collect()
}

pub(super) fn recruit_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    // Spec 057 : Appel à la gloire, toute unité de l'offre, partout, sans Influence.
    let free = free_recruit(p);
    let units = if free {
        offered_units(state, None)
    } else {
        recruitable_units(state, player)
    };
    let pays = !free;
    let full = ordinary_units(p).count() >= usize::from(state.command_tokens(player));
    // Spec 058 : le jeton de Loyauté sans faille, libre.
    let bonds_free = p.bonds_unit.is_none()
        && p.skills
            .iter()
            .any(|owned| owned.skill == Skill::BondsOfLoyalty);
    let mut actions = Vec::new();
    for unit in units {
        let definition = unit.unit.definition();
        // Texte de Loyauté sans faille ; Compendium, *Bonds of Loyalty* S5 : 5 de moins, régulière
        // ou d'élite, sans renvoi.
        if pays
            && bonds_free
            && p.purchase_influence() >= i32::from(bonds_cost(definition.influence))
        {
            actions.push(Action::RecruitUnit {
                unit,
                disband: None,
                bonds: true,
            });
        }
        if pays && p.purchase_influence() < i32::from(definition.influence) {
            continue;
        }
        if full {
            actions.extend(ordinary_units(p).map(|owned| Action::RecruitUnit {
                unit,
                disband: Some(owned.card),
                bonds: false,
            }));
        } else {
            actions.push(Action::RecruitUnit {
                unit,
                disband: None,
                bonds: false,
            });
        }
    }
    actions
}

/// Coût d'un artefact dans la cité rouge (MKUE p. 8, 4.e ; carte de la cité rouge ; spec 062).
pub const CITY_ARTIFACT_COST: u8 = 12;

/// Coût d'une action avancée dans la cité verte (MKUE p. 8, 4.e ; spec 062).
pub const CITY_ADVANCED_ACTION_COST: u8 = 6;

/// Coût d'une unité d'élite ajoutée à l'offre dans la cité blanche (MKUE p. 8, 4.e ; spec 062).
pub const CITY_ELITE_UNIT_COST: u8 = 2;

/// Les achats de la cité verte, rouge ou blanche (MKUE p. 8, 4.e ; spec 062) : dans la verte,
/// chaque action avancée de l'offre d'actions avancées puis la carte du dessus de leur pioche ;
/// dans la rouge, un artefact si la pioche en garde assez pour le tirage de fin de tour (L7) ;
/// dans la blanche, la carte du dessus des unités d'élite.
fn city_purchase_actions(state: &GameState, p: &PlayerState, color: BasicColor) -> Vec<Action> {
    let affordable = |cost: u8| p.purchase_influence() >= i32::from(cost);
    let mut actions = Vec::new();
    match color {
        BasicColor::Green if affordable(CITY_ADVANCED_ACTION_COST) => {
            actions.extend(
                state
                    .offer(Offer::AdvancedActions)
                    .iter()
                    .filter_map(|card| card.deed())
                    .map(|card| Action::BuyAdvancedAction { card }),
            );
            if !state.common_deck(CommonDeck::AdvancedActions).is_empty() {
                actions.push(Action::BuyAdvancedActionFromDeck);
            }
        }
        // MKUE p. 11, 5.b : un de plus que les artefacts dus, achat compris.
        BasicColor::Red
            if affordable(CITY_ARTIFACT_COST)
                && state.common_deck(CommonDeck::Artifacts).len()
                    >= usize::from(p.artifact_rewards) + 2 =>
        {
            actions.push(Action::BuyArtifact);
        }
        BasicColor::White
            if affordable(CITY_ELITE_UNIT_COST)
                && !state.common_deck(CommonDeck::EliteUnits).is_empty() =>
        {
            actions.push(Action::AddEliteUnit);
        }
        _ => {}
    }
    actions
}

/// Coût d'un point de soin au village et au monastère, imprimé sur leurs cartes de site (MKUE
/// p. 8, 4.b ; spec 041).
pub const VILLAGE_HEALING_COST: u8 = 3;

pub const MONASTERY_HEALING_COST: u8 = 2;

/// Coût d'une action avancée au monastère (MKUE p. 8, 4.c ; carte du monastère).
pub const MONASTERY_ADVANCED_ACTION_COST: u8 = 6;

/// Coût d'un sort à la tour de mage, plus un mana de sa couleur (MKUE p. 8, 4.d ; spec 043).
pub const SPELL_COST: u8 = 7;

/// Les achats légaux pendant une interaction (spec 041) : au monastère, chaque action avancée de
/// l'offre d'unités dans l'ordre de l'offre (MKUE p. 8, 4.c) ; à la tour de mage, chaque sort de
/// l'offre de sorts, par moyen de paiement de sa couleur (4.d ; spec 043) ; puis le point de soin
/// au village et au monastère (4.b), pour une influence qui atteint le coût (spec 042).
pub(super) fn purchase_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    let site = p.interaction.map(|interaction| interaction.site);
    let affordable = |cost: u8| p.purchase_influence() >= i32::from(cost);
    let mut actions = Vec::new();
    // Spec 062 : les achats des cités, selon leur couleur ; jamais de soin.
    if let Some(Site::City(color)) = site
        && color != BasicColor::Blue
    {
        return city_purchase_actions(state, p, color);
    }
    if matches!(site, Some(Site::MageTower | Site::City(BasicColor::Blue)))
        && affordable(SPELL_COST)
    {
        for card in state
            .offer(Offer::Spells)
            .iter()
            .filter_map(|card| card.deed())
        {
            let Some(color) = card.card().definition().map(|definition| definition.color) else {
                continue;
            };
            actions.extend(
                mana_payments(state, p, color)
                    .into_iter()
                    .map(|mana| Action::BuySpell { card, mana }),
            );
        }
        return actions;
    }
    let healing_cost = match site {
        Some(Site::Village) => VILLAGE_HEALING_COST,
        Some(Site::Monastery) => MONASTERY_HEALING_COST,
        _ => return Vec::new(),
    };
    if site == Some(Site::Monastery) && affordable(MONASTERY_ADVANCED_ACTION_COST) {
        actions.extend(
            state
                .offer(Offer::Units)
                .iter()
                .filter_map(|card| card.deed())
                .map(|card| Action::BuyAdvancedAction { card }),
        );
    }
    if affordable(healing_cost) {
        actions.push(Action::BuyHealing);
    }
    actions
}
