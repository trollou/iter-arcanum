//! Actions légales : ce que le joueur peut faire dans l'état courant (`legal_actions`).
//! Le détail est rangé par thème dans les sous-modules ; ici, l'aiguillage et le déplacement.

use super::*;

mod artifacts;
mod cards;
mod mana;
mod sites;
mod skills;
mod spells;
mod turn_end;
mod units;

// Chaque thème partage ses fonctions avec le reste du moteur à la visibilité qu'elles ont :
// un glob ne réexporte un élément qu'à sa propre visibilité, au plus celle du `use`.
use artifacts::*;
pub(super) use cards::*;
pub(super) use mana::*;
pub use sites::*;
pub(super) use skills::*;
pub use spells::*;
pub use turn_end::*;
pub use units::*;

pub fn legal_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    match pending_decision(state) {
        Some(decision) if decision.player == player => available_actions(state, player)
            .into_iter()
            .filter(|action| !invokes_played_card(action))
            .collect(),
        _ => Vec::new(),
    }
}

/// Spec 058 (L2 ; Compendium, *Invocation* S2) : Invocation ne défausse pas une carte que l'action
/// joue (la carte jouée, celle qu'elle défausse ou joue avec elle) : elle n'est plus en main.
fn invokes_played_card(action: &Action) -> bool {
    let (Action::PlayCard { card, play } | Action::PlayCardOnUnit { card, play, .. }) = action
    else {
        return false;
    };
    let payments: Vec<ManaPayment> = match *play {
        CardPlay::Basic { mana, .. } => mana.into_iter().collect(),
        CardPlay::Strong {
            mana, extra_mana, ..
        } => [Some(mana), extra_mana].into_iter().flatten().collect(),
        CardPlay::Cast {
            mana,
            black,
            extra_mana,
            ..
        } => [Some(mana), black, extra_mana]
            .into_iter()
            .flatten()
            .collect(),
        CardPlay::SidewaysSkill { mana, .. } => mana.map(|mana| mana.payment).into_iter().collect(),
        _ => Vec::new(),
    };
    let with = match *play {
        CardPlay::Basic { with, .. } | CardPlay::Strong { with, .. } => with,
        _ => None,
    };
    let played: Vec<CardId> = [Some(*card), play_discard(*play)]
        .into_iter()
        .chain(with.map(|target| Some(target.card)))
        .chain(with.map(|target| target.discard))
        .flatten()
        .collect();
    payments.into_iter().any(|payment| match payment {
        ManaPayment::Invocation(invoked)
        | ManaPayment::Polarized {
            source: PolarSource::Invocation(invoked),
            ..
        } => played.contains(&invoked),
        _ => false,
    })
}

/// Les suites de `count` cartes distinctes de `cards`, dans l'ordre de `cards` (spec 057).
fn arrangements(cards: &[CardId], count: usize) -> Vec<Vec<CardId>> {
    if count == 0 {
        return vec![Vec::new()];
    }
    let mut orders = Vec::new();
    for (index, &card) in cards.iter().enumerate() {
        let mut rest = cards.to_vec();
        rest.remove(index);
        for mut order in arrangements(&rest, count - 1) {
            order.insert(0, card);
            orders.push(order);
        }
    }
    orders
}

/// Énumération exhaustive : les cartes dans l'ordre de la spec 013, les déplacements par
/// direction (spec 014), les trajets suivis dans l'ordre des trajets (spec 036), les explorations par emplacement (spec 015), les défis (spec 031), puis
/// les fins de tour (spec 016). Autour du combat, seulement ce que permet son moment (spec 031).
pub(super) fn available_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    // Spec 055 (Q2) : à la fin de la manche, chaque ensemble de bannières assignées à reprendre,
    // dans l'ordre des unités.
    if p.banner_reclaim {
        let banners: Vec<CardId> = p.units.iter().filter_map(|owned| owned.banner).collect();
        return (0..=banners.len())
            .flat_map(|size| combinations(&banners, size))
            .map(|banners| Action::ReclaimBanners { banners })
            .collect();
    }
    // Spec 060 : au début de la manche, rien que le choix de la tactique (Compendium, *Turn
    // Start* S1, note †), parmi celles qui restent, par numéro.
    if state.tactic_choice && p.tactic.is_none() {
        return offered_tactics(state)
            .into_iter()
            .map(|tactic| Action::ChooseTactic { tactic })
            .collect();
    }
    // Spec 060 (Q3) : puis le choix qui suit la prise.
    if p.tactic_follow_up {
        return tactic_follow_up_actions(state, p);
    }
    // Spec 060 (Q5) : la pioche de fin de tour arrêtée attend Longue Nuit.
    if p.draw_stopped.is_some() {
        return vec![Action::UseLongNight, Action::DeclineLongNight];
    }
    // Spec 060 (C31 à C33) : Provisions avant le tour ; deck vide, seul le retournement.
    if p.sparing_pending {
        let store = (!p.deck.is_empty()).then_some(Action::SparingPower { take: false });
        return store
            .into_iter()
            .chain([Action::SparingPower { take: true }])
            .collect();
    }
    // Spec 056 : le choix de Rituel de sang renforcé, aussitôt : les couleurs des jetons, puis le
    // cristal payé d'un mana de base ou refusé (Compendium, *Blood Ritual* S1).
    if let Some(choice) = p.mana_choice {
        if choice.tokens_left > 0 {
            return ManaColor::ALL
                .into_iter()
                .map(|color| Action::ChooseTokenColor { color })
                .collect();
        }
        let mut actions: Vec<Action> = BasicColor::ALL
            .into_iter()
            .flat_map(|color| {
                mana_payments(state, p, color)
                    .into_iter()
                    .map(move |mana| Action::PayForCrystal { color, mana })
            })
            .collect();
        actions.push(Action::DeclineCrystal);
        return actions;
    }
    // Spec 057 (L4) : les cartes de Méditation attendent leur place, aussitôt.
    if let Some(meditation) = &p.meditation {
        let orders = arrangements(&meditation.cards, meditation.pick);
        return [true, false]
            .into_iter()
            .flat_map(|top| {
                orders.iter().map(move |order| Action::PlaceOnDeck {
                    top,
                    order: order.clone(),
                })
            })
            .collect();
    }
    // Spec 044 : à la fin du tour, d'abord la couleur du cristal d'un dé or (Compendium, *End of
    // Turn* Q4), une couleur non pleine.
    if p.crystal_choices > 0 || p.pouch_choices > 0 {
        return BasicColor::ALL
            .into_iter()
            .filter(|&color| !p.inventory.is_full(color))
            .map(|color| Action::ChooseCrystalColor { color })
            .collect();
    }
    // Spec 041 : à la fin du tour, d'abord le choix de l'artefact de la récompense, dans l'ordre
    // du tirage (MKUE p. 11, 5.b, avant 6.b).
    if !p.drawn_artifacts.is_empty() {
        return p
            .drawn_artifacts
            .iter()
            .map(|&card| Action::TakeArtifact { card })
            .collect();
    }
    // Spec 043 : puis le sort de la récompense d'une tour de mage, dans l'ordre de l'offre (MKUE
    // p. 11, 5.c).
    if p.spell_choices > 0 {
        return state
            .offer(Offer::Spells)
            .iter()
            .filter_map(|card| card.deed())
            .map(|card| Action::TakeSpell { card })
            .collect();
    }
    // Spec 051 : à la fin d'un tour à niveau pair, la compétence d'abord, parmi les deux
    // révélées (MKUE p. 11, 6.b).
    // Ou une compétence d'un autre héros de l'offre commune, jamais du sien (Compendium, *Skill
    // Tokens* S3 ; FAQ p3).
    if !p.skill_choice.is_empty() {
        return p
            .skill_choice
            .iter()
            .chain(common_skills_takeable(state, p))
            .map(|&skill| Action::ChooseSkill { skill })
            .collect();
    }
    // Spec 050 : à la fin d'un tour à niveau pair, seulement le choix de l'action avancée, dans
    // l'ordre de l'offre ; spec 051 : une fois la compétence de ce niveau choisie.
    if p.advanced_action_choices > p.skill_choices {
        let offer = state.offer(Offer::AdvancedActions);
        // Spec 051 : après une compétence de l'offre commune, la carte en dernière place.
        let from = if p.lowest_advanced_action_only {
            offer.len().saturating_sub(1)
        } else {
            0
        };
        return offer[from..]
            .iter()
            .filter_map(|card| card.deed())
            .map(|card| Action::GainAdvancedAction { card })
            .collect();
    }
    // Spec 045 : puis l'unité de la récompense d'une ruine (MKUE p. 11, 5.d).
    if p.unit_choices > 0 {
        return unit_reward_actions(state, p);
    }
    match moment(p) {
        Moment::Engaging => return engaging_actions(state, player),
        Moment::Fighting(combat) => {
            let mut actions = card_actions(state, player);
            actions.extend(unit_actions(state, player));
            // Spec 051 : les compétences, après les unités.
            actions.extend(skill_actions(state, player));
            // Spec 057 : Appel à la gloire, en combat aussi.
            actions.extend(recruit_actions(state, player));
            // Spec 052 : les conversions, après les cartes et les unités.
            actions.extend(conversion_actions(p, combat));
            // Spec 054 : l'attaque différée, dans une attaque en cours de la phase d'attaque.
            if combat.phase == crate::CombatPhase::Attack
                && matches!(combat.in_progress, Some(crate::InProgress::Attack { .. }))
                && deferred_attack(p).is_some()
            {
                actions.push(Action::UseDeferredAttack);
            }
            actions.extend(target_actions(combat));
            actions.extend(assign_actions(p, combat));
            actions.extend(phase_actions(combat));
            return actions;
        }
        Moment::Free => {}
    }
    // Spec 040 : l'interaction est l'action du tour ; restent les cartes et la fin du tour (MKUE
    // p. 6, 6.d).
    if p.interaction.is_some() {
        let mut actions = card_actions(state, player);
        actions.extend(unit_actions(state, player));
        actions.extend(skill_actions(state, player));
        actions.extend(healing_actions(state, player));
        actions.extend(recruit_actions(state, player));
        actions.extend(purchase_actions(state, player));
        actions.extend(learn_actions(state, player));
        actions.extend(keep_die_action(p));
        actions.extend(right_moment_action(p));
        actions.extend(long_night_action(p));
        actions.extend(mana_search_actions(state, p));
        actions.extend(meditation_actions(p));
        actions.extend(end_turn_actions(state, player));
        return actions;
    }
    // Spec 041 (Q1) : le pillage, en tête, avant toute autre chose du tour.
    let mut actions = Vec::new();
    if can_plunder(state, player) {
        actions.push(Action::PlunderVillage);
    }
    actions.extend(card_actions(state, player));
    actions.extend(unit_actions(state, player));
    actions.extend(skill_actions(state, player));
    actions.extend(healing_actions(state, player));
    // Spec 057 : Appel à la gloire, hors de toute interaction.
    actions.extend(recruit_actions(state, player));
    actions.extend(learn_actions(state, player));
    actions.extend(
        affordable_moves(state, player)
            .into_iter()
            .map(|(to, _)| Action::MoveHero { to }),
    );
    actions.extend(
        affordable_routes(state, player)
            .into_iter()
            .map(|route| Action::FollowRoute { to: route.to }),
    );
    actions.extend(
        affordable_explorations(state, player)
            .into_iter()
            .map(|position| Action::Explore { position }),
    );
    if interaction_site(state, player).is_some() {
        actions.push(Action::Interact);
    }
    if burnable_monastery(state, player).is_some() {
        actions.push(Action::BurnMonastery);
    }
    if enterable_adventure_site(state, player).is_some() {
        actions.push(Action::EnterAdventureSite);
    }
    // Spec 045 : l'offrande à l'autel, une action par paiement ; une ruine a un autel ou des
    // ennemis, jamais les deux.
    if let Some((_, color)) = tribute_altar(state, player)
        && let Some(p) = state.player(player)
    {
        actions.extend(
            altar_payments(state, p, color)
                .into_iter()
                .map(|mana| Action::PayAltarTribute { mana }),
        );
    }
    actions.extend(engaging_actions(state, player));
    actions.extend(keep_die_action(p));
    actions.extend(right_moment_action(p));
    actions.extend(long_night_action(p));
    actions.extend(mana_search_actions(state, p));
    actions.extend(meditation_actions(p));
    actions.extend(end_turn_actions(state, player));
    // Spec 023 : en dernier, pour que le joueur « premier choix » joue ses cartes avant.
    if can_announce_end_of_round(state, player) {
        actions.push(Action::AnnounceEndOfRound);
    }
    actions
}

/// Le site où le héros peut interagir avec les habitants maintenant (spec 040) : un village ou un
/// monastère sous la figurine (MKUE p. 8, 1), réputation hors de X (2.a), ni interaction, ni
/// combat, ni ennemi provoqué ou défié ce tour (MKUE p. 6, 6.b à 6.d). Comme pour un défi : pas
/// avec une main de Blessures sans carte jouée (Compendium, *Resting* S4), ni quand l'annonce de
/// fin de manche est obligatoire (MKUE p. 6, 3.b).
/// L'attaque différée du Bouclier ardent en attente (spec 054), la première posée.
pub(super) fn deferred_attack(p: &PlayerState) -> Option<(u8, Element)> {
    p.modifiers.iter().find_map(|modifier| match modifier.kind {
        ModifierKind::DeferredAttack { amount, element } => Some((amount, element)),
        _ => None,
    })
}

/// Spec 060 : le choix qui suit la prise de la tactique (Q3).
fn tactic_follow_up_actions(state: &GameState, p: &PlayerState) -> Vec<Action> {
    match p.tactic.as_ref().map(|taken| taken.tactic) {
        // Carte : 0 à 3 cartes de la main, Blessures comprises ; les Blessures étant identiques,
        // les premières de la main, une combinaison par nombre (comme la fin du tour).
        Some(Tactic::Rethink) => hand_choices(&p.hand, 0, 3)
            .into_iter()
            .map(|cards| Action::RethinkDiscard { cards })
            .collect(),
        // Carte ; Compendium, *Preparation* S1 : une action par carte distincte du deck, dans
        // l'ordre des cartes et non du deck, qui reste caché.
        Some(Tactic::Preparation) => {
            let mut cards: Vec<CardId> = p.deck.cards().to_vec();
            cards.sort();
            cards.dedup_by_key(|card| card.card());
            cards
                .into_iter()
                .map(|card| Action::PrepareCard { card })
                .collect()
        }
        // Carte : un dé de couleur de base de la Source, dans l'ordre de la Source.
        Some(Tactic::ManaSteal) => stealable_dice(state, p)
            .into_iter()
            .map(|die| Action::StealDie { die })
            .collect(),
        _ => Vec::new(),
    }
}

/// Spec 060 : les ensembles de `min` à `max` cartes de `hand`, par taille croissante ; les cartes
/// autres que les Blessures dans l'ordre de la main, suivies d'autant de Blessures qu'il faut,
/// les premières de la main.
pub(super) fn hand_choices(hand: &[CardId], min: usize, max: usize) -> Vec<Vec<CardId>> {
    let (wounds, others): (Vec<CardId>, Vec<CardId>) =
        hand.iter().partition(|card| card.card().is_wound());
    let mut choices = Vec::new();
    for size in min..=max.min(hand.len()) {
        for taken in 0..=size.min(wounds.len()) {
            if size - taken > others.len() {
                continue;
            }
            for mut cards in combinations(&others, size - taken) {
                cards.extend_from_slice(&wounds[..taken]);
                choices.push(cards);
            }
        }
    }
    choices
}

/// Les sous-ensembles de `cards` (cartes, cibles…) de taille `size`, dans l'ordre lexicographique des positions.
pub(crate) fn combinations<T: Copy>(cards: &[T], size: usize) -> Vec<Vec<T>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    (0..cards.len())
        .flat_map(|first| {
            combinations(&cards[first + 1..], size - 1)
                .into_iter()
                .map(move |mut rest| {
                    rest.insert(0, cards[first]);
                    rest
                })
        })
        .collect()
}

/// Emplacements où la figurine peut révéler la prochaine tuile, si elle a les 2 points
/// (MKUE p. 7, 5.a à 5.e).
fn affordable_explorations(state: &GameState, player: PlayerId) -> Vec<TilePosition> {
    let (Some(figure), Some(scenario)) = (state.figure(player), state.scenario()) else {
        return Vec::new();
    };
    // Spec 030 : une provocation termine le déplacement, exploration comprise.
    if state.move_points(player) < u32::from(EXPLORE_COST) || state.movement_ended(player) {
        return Vec::new();
    }
    let next = next_tile(&state.map, &state.tile_deck);
    // Spec 057 : avec Distorsion spatiale, depuis une case à 2 de l'emplacement (Compendium,
    // *Space Bending* S2).
    let mut positions = free_positions_touching(&state.map, figure);
    if state.move_rules(player).space_bending {
        for neighbor in figure.neighbors() {
            positions.extend(free_positions_touching(&state.map, neighbor));
        }
        positions.sort();
        positions.dedup();
    }
    positions
        .into_iter()
        .filter(|position| {
            placement(&state.map, scenario.shape(), *position, next) == Placement::Allowed
        })
        .collect()
}

/// Cases adjacentes à la figurine où elle peut entrer avec ses points, et leur coût, dans
/// l'ordre des directions (MKUE p. 7, 3).
pub(super) fn affordable_moves(state: &GameState, player: PlayerId) -> Vec<(Hex, u8)> {
    let Some((figure, points)) = state
        .player(player)
        .filter(|p| !p.movement_ended)
        .and_then(|p| p.figure.map(|figure| (figure, p.move_points)))
    else {
        return Vec::new();
    };
    let rampaging = rampaging_enemies(state);
    // Spec 052 : les coûts réduits et les terrains franchissables.
    let rules = state.move_rules(player);
    // Spec 057 : avec Distorsion spatiale, les cases à 2 aussi.
    crate::steps(figure, rules.space_bending)
        .into_iter()
        // MKUE p. 7, 4.b : pas d'entrée sur la case d'un ennemi errant (spec 030).
        .filter(|to| !rampaging.contains(to))
        .filter_map(|to| {
            let space = state.map.space(to)?;
            match move_cost(space, &rules) {
                MoveCost::Points(cost) if u32::from(cost) <= points => Some((to, cost)),
                // Spec 043 : l'assaut d'une forteresse ou d'une tour de mage coûte son terrain ;
                // conquise, c'est une case ordinaire.
                // Spec 062 : une cité en jeu, assaillie ou conquise ; jamais une cité fermée.
                MoveCost::Fortified(cost)
                    if (is_assault_target(state, to) || state.is_conquered(to))
                        && u32::from(cost) <= points =>
                {
                    Some((to, cost))
                }
                // Infranchissable, ou cité fermée (spec 062).
                MoveCost::Points(_) | MoveCost::Inaccessible | MoveCost::Fortified(_) => None,
            }
        })
        .collect()
}

/// Trajets retenus vers les cases lointaines qu'on peut payer avec les points déjà disponibles,
/// dans l'ordre de `routes_avoiding` : ni case d'ennemi errant, ni pas qui provoque (spec 036 ;
/// spec 030, C19). Les voisines restent aux `MoveHero` (une action légale par déplacement).
pub(super) fn affordable_routes(state: &GameState, player: PlayerId) -> Vec<Route> {
    let Some((figure, points)) = state
        .player(player)
        .filter(|p| !p.movement_ended)
        .and_then(|p| p.figure.map(|figure| (figure, p.move_points)))
    else {
        return Vec::new();
    };
    let enemies = rampaging_enemies(state);
    let assaults = assault_targets(state);
    let conquered = state.conquered_sites();
    routes_ending_in(
        &state.map,
        figure,
        state.move_rules(player),
        &enemies,
        &assaults,
        &conquered,
    )
    .into_iter()
    .filter(|route| route.steps.len() >= 2 && route.cost <= points)
    .collect()
}

/// Au bon moment (spec 060, C20, C22) : pendant le tour, avant la fin engagée, une fois.
fn right_moment_action(p: &PlayerState) -> Option<Action> {
    p.tactic
        .as_ref()
        .filter(|tactic| tactic.tactic == crate::Tactic::TheRightMoment && !tactic.flipped)
        .map(|_| Action::AnnounceRightMoment)
}

/// Longue Nuit (spec 060, C23) : pendant le tour, deck vide.
fn long_night_action(p: &PlayerState) -> Option<Action> {
    (p.deck.is_empty() && long_night_usable(p)).then_some(Action::UseLongNight)
}

/// Appel du mana (spec 060, C26, C27, L6) : une fois par tour, avant tout dé de la Source pris,
/// 1 ou 2 dés de la Source, les dés or d'abord.
fn mana_search_actions(state: &GameState, p: &PlayerState) -> Vec<Action> {
    let usable = p.tactic.as_ref().is_some_and(|tactic| {
        tactic.tactic == crate::Tactic::ManaSearch && !tactic.mana_search_used
    }) && p.dice_taken.is_empty();
    if !usable {
        return Vec::new();
    }
    let dice: Vec<u8> = (0..state.source.len())
        .filter_map(|die| u8::try_from(die).ok())
        .filter(|&die| die_free(p, die))
        .collect();
    let gold = |die: &u8| state.source.get(usize::from(*die)) == Some(&ManaColor::Gold);
    let golds = dice.iter().filter(|die| gold(die)).count();
    (1..=2)
        .flat_map(|size| combinations(&dice, size))
        .filter(|chosen| chosen.iter().filter(|die| gold(die)).count() == golds.min(chosen.len()))
        .map(|dice| Action::ManaSearch { dice })
        .collect()
}

/// Méditation nocturne (spec 060, C28, C29) : avant le tour, rien de fait encore (Compendium,
/// *Turn Start* S1) ; 1 à 5 cartes de la main.
fn meditation_actions(p: &PlayerState) -> Vec<Action> {
    let usable = p.tactic.as_ref().is_some_and(|tactic| {
        tactic.tactic == crate::Tactic::MidnightMeditation && !tactic.flipped
    });
    if !usable || !turn_untouched(p) {
        return Vec::new();
    }
    hand_choices(&p.hand, 1, 5)
        .into_iter()
        .map(|cards| Action::MidnightMeditation { cards })
        .collect()
}

/// Rien de fait depuis le début du tour : ni carte, ni déplacement, ni dé, ni jeton, ni cristal,
/// ni unité, ni interaction, ni combat, ni fait du tour (spec 060, « avant un tour »).
fn turn_untouched(p: &PlayerState) -> bool {
    p.play_area.is_empty()
        && !p.played_elsewhere
        && !p.moved_or_fought
        && p.turn_path.len() <= 1
        && p.dice_taken.is_empty()
        && p.spent_mana_tokens.is_empty()
        && p.crystals_spent.is_empty()
        && !p.unit_activated
        && p.interaction.is_none()
        && p.combat.is_none()
        && p.turn_facts.is_empty()
}
