use std::collections::BTreeMap;

use crate::city::CityState;
use crate::combat::{PlayMoment, is_playable_now};
use crate::engine::{after_forced_withdrawal, forced_withdrawal_path, wings_of_night_castable};
use crate::exploration::removed_counts;
use crate::{
    Action, CardPlay, CommonSkillView, ManaPayment, PolarSource, Skill, SkillEffect, SkillView,
};
use crate::{
    Assault, BasicColor, CardId, Combat, CommonCard, CommonDeck, Effect, EnemyColor, EnemyToken,
    FinalScore, FinalScoring, GameContent, GameEndReason, GameState, Hero, Hex, Interaction,
    Inventory, LastTurn, ManaChoice, ManaColor, Map, Meditation, Modifier, ModifierKind, MoveRules,
    NextTile, Offer, OwnedUnit, PlayedCard, PlayerId, Reputation, RuinToken, Scenario, Tactic,
    TacticSlot, TacticView, TileDeckCounts, TimeOfDay, TurnFact, final_score, next_tile,
    pending_decision,
};

/// Ce qu'un joueur a le droit de connaître de l'état.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub player: PlayerId,
    pub hand: Vec<CardId>,
    pub deck_size: usize,
    pub has_pending_decision: bool,
    /// Public, forme du plateau de jeu comprise.
    pub scenario: Option<Scenario>,
    pub hero: Option<Hero>,
    pub map: Map,
    pub figure: Option<Hex>,
    /// Seul le dos des tuiles de la pioche se voit (MKUE p. 3).
    pub tile_deck: TileDeckCounts,
    /// Publics : zone de jeu, points du tour et défausse (spec 013).
    pub play_area: Vec<PlayedCard>,
    pub move_points: u32,
    pub influence_points: i32,
    pub discard: Vec<CardId>,
    /// Public (spec 014).
    pub time_of_day: TimeOfDay,
    /// Dos de la prochaine tuile à révéler, ou tuile écartée, ou rien (spec 015).
    pub next_tile: NextTile,
    /// Tuiles écartées encore révélables, par dos : leur nombre est public, pas leur identité.
    pub removed_tiles: TileDeckCounts,
    /// Numéro du tour, public (spec 016).
    pub turn: u32,
    /// Numéro de la manche, public (spec 023) ; le nombre de manches est dans le scénario.
    pub round: u32,
    /// Source, publique (spec 020).
    pub source: Vec<ManaColor>,
    /// Inventaire, public (spec 021).
    pub inventory: Inventory,
    /// Dés pris ce tour, publics (spec 022).
    pub dice_taken: Vec<u8>,
    /// Jetons de mana de la zone de jeu, publics (spec 025).
    pub mana_tokens: Vec<ManaColor>,
    /// Dés que le joueur peut prendre ce tour, dés de plus compris (spec 025).
    pub dice_limit: usize,
    /// Dés mis sur une couleur ce tour, publics (spec 025).
    pub dice_set: Vec<u8>,
    /// Jetons d'ennemis face visible, par case (spec 030) ; sans les défenseurs face cachée
    /// (spec 043).
    /// Une liste par case (spec 044, Q1).
    pub enemy_tokens: BTreeMap<Hex, Vec<EnemyToken>>,
    /// Défenseurs face cachée : leur case et leur couleur seulement (spec 043).
    pub face_down_defenders: BTreeMap<Hex, Vec<EnemyColor>>,
    /// Spec 062, public : chaque cité révélée, sa couleur, son niveau, ses Boucliers et son Seigneur.
    pub cities: BTreeMap<Hex, CityState>,
    /// Nombre de jetons de chaque pile d'ennemis : l'ordre reste caché (spec 030).
    pub enemy_piles: BTreeMap<EnemyColor, usize>,
    /// Phase de mouvement finie (provocation, spec 030 ; début de l'action, issue privée 164), et cases
    /// des ennemis à combattre.
    pub movement_ended: bool,
    pub enemies_to_fight: Vec<Hex>,
    /// Le combat entier est public : ennemis, phase, cibles, totaux (spec 031).
    pub combat: Option<Combat>,
    /// Total de gloire, public (spec 031).
    pub fame: u32,
    /// Défausses d'ennemis, publiques ; l'ordre d'une pile ne l'est jamais (spec 031).
    pub enemy_discards: BTreeMap<EnemyColor, Vec<EnemyToken>>,
    /// Le héros peut encore se reposer ce tour (spec 034).
    pub can_rest: bool,
    /// Spec 055 : une carte jouée ce tour a quitté la zone de jeu (artefact détruit, bannière
    /// assignée) ; elle compte pour le tour minimal.
    pub played_elsewhere: bool,
    /// Spec 055 (Q2) : la manche finie, le choix des bannières à reprendre attend.
    pub banner_reclaim: bool,
    /// L'action du tour n'est plus possible, ou pas encore (interagir, entrer, faire une offrande) ;
    /// public. L'écran s'en sert pour ses boutons grisés au lieu de recopier la règle (issue privée 141).
    pub turn_action_taken: bool,
    /// Spec 035, tout public : niveau, valeurs du jeton de niveau, gloire du niveau suivant
    /// (aucune au niveau 10), niveau atteint à la fin du tour s'il change, niveaux gagnés à la
    /// dernière fin de tour.
    pub level: u8,
    pub armor: u8,
    pub hand_limit: usize,
    pub command_tokens: u8,
    pub next_level_fame: Option<u32>,
    pub pending_level: Option<u8>,
    pub levels_gained: Vec<u8>,
    /// Spec 063, tout public : dernier tour, raison de la fin de partie, et le décompte final,
    /// donné seulement une fois la partie terminée et si le scénario en a un (C19).
    pub last_turn: LastTurn,
    pub game_end: Option<GameEndReason>,
    pub final_score: Option<FinalScore>,
    /// Spec 064 : cités conquises et cités du scénario, seulement pour Conquête en solitaire.
    pub conquest: Option<crate::ConquestProgress>,
    /// Spec 050 : les trois offres (première place en tête), le nombre de cartes de chaque pioche
    /// commune (jamais leur ordre), les choix d'action avancée en attente.
    pub offers: BTreeMap<Offer, Vec<CommonCard>>,
    pub common_decks: BTreeMap<CommonDeck, usize>,
    pub advanced_action_choices: u8,
    /// Spec 040, public : réputation et son bonus d'interaction (`None` sur X).
    pub reputation: Reputation,
    pub reputation_bonus: Option<i8>,
    /// L'interaction en cours ce tour, publique (spec 040).
    pub interaction: Option<Interaction>,
    /// Spec 042, public : la zone des unités, dans l'ordre du recrutement, et les points de soin.
    /// Les dégâts en cours d'assignation sont dans `combat`.
    pub units: Vec<OwnedUnit>,
    pub healing_points: u32,
    /// Issue privée 341, public : les unités de l'offre recrutables dans le lieu de l'interaction en
    /// cours, dans l'ordre de l'offre, quelle que soit l'Influence (l'écran grise les trop chères).
    pub recruitable_units: Vec<crate::UnitCard>,
    /// Spec 041, public : les jetons Bouclier par case, les monastères incendiés, le monastère
    /// incendié ce tour (son combat exclut les unités), les récompenses artefact du tour et les
    /// artefacts tirés en attente du choix.
    pub shields: BTreeMap<Hex, PlayerId>,
    pub burned_monasteries: Vec<Hex>,
    pub burning: Option<Hex>,
    pub artifact_rewards: u8,
    pub drawn_artifacts: Vec<CardId>,
    /// Spec 043, public : l'assaut du tour (site assailli, case de départ), les récompenses sort
    /// du tour et les choix de sort en attente.
    pub assault: Option<Assault>,
    pub spell_rewards: u8,
    pub spell_choices: u8,
    /// Spec 044, public : le site d'aventure où le joueur est entré ce tour, et le moment de la
    /// journée du combat (la Nuit dans un donjon ou un tombeau).
    pub adventure: Option<Hex>,
    pub combat_time_of_day: TimeOfDay,
    /// Spec 044, public : les cristaux aléatoires gagnés ce tour et les choix de couleur en
    /// attente.
    pub crystal_rewards: u8,
    pub crystal_choices: u8,
    /// Spec 044, public : le dé de récompense du donjon conquis ce tour (C45).
    pub reward_die: Option<ManaColor>,
    /// Bonus de limite de main de la pioche près d'une forteresse possédée, public (spec 043).
    pub keep_hand_bonus: usize,
    /// Bonus de limite de main d'une cité conquise à portée (spec 062) : +1 avec un Bouclier,
    /// +2 pour le Seigneur.
    pub city_hand_bonus: usize,
    /// Spec 045 : les ruines non conquises, par case, avec leur jeton face visible, ou `None` face
    /// cachée ; le nombre de jetons de la pile (son ordre reste caché).
    pub ruins: BTreeMap<Hex, Option<RuinToken>>,
    pub ruin_pile: usize,
    /// Le nombre de compétences encore dans la pile ; jamais leur ordre (spec 051).
    pub skill_pile_count: usize,
    /// Spec 045, public : les récompenses de ruines du tour (lot de cristaux, action avancée,
    /// unité), les choix d'unité et d'artefacts à garder en attente, et les ruines révélées par le
    /// dernier pas ou trajet (C40).
    pub crystal_set_rewards: u8,
    pub advanced_action_rewards: u8,
    pub unit_rewards: u8,
    pub unit_choices: u8,
    pub artifacts_to_keep: u8,
    pub revealed_ruins: Vec<Hex>,
    /// Spec 073, public : les faits du tour, dans l'ordre, et les jetons de mana dépensés ce tour.
    pub turn_facts: Vec<TurnFact>,
    pub spent_mana_tokens: Vec<ManaColor>,
    /// Spec 052, public : les effets en cours, avec leur source, et les règles de déplacement
    /// qu'ils font (coûts réduits, lacs franchissables).
    pub modifiers: Vec<Modifier>,
    pub move_rules: MoveRules,
    /// Spec 052, public : la retraite forcée qui suivrait la fin du tour maintenant, hors assaut en
    /// cours.
    pub forced_withdrawal: Option<ForcedWithdrawalPreview>,
    /// Spec 052, public : la vue telle que la laisserait cette retraite forcée (figurine sur la case
    /// d'arrivée, Blessures en main, limite de main près d'une forteresse), pour l'écran de fin du
    /// tour (issue privée 217).
    pub after_withdrawal: Option<Box<PlayerView>>,
    /// Spec 056, public : le choix en attente de Rituel de sang renforcé (jetons restant à choisir,
    /// cristal proposé).
    pub mana_choice: Option<ManaChoice>,
    /// Spec 058, public : les avantages du centre de ce tour, l'Influence d'interaction (Fortune
    /// clinquante), plus de dé de la Source ce tour (Magie inutile à +3), l'unité sous le jeton
    /// de Loyauté sans faille.
    pub center_advantages: Vec<crate::CenterAdvantage>,
    pub interaction_influence: u32,
    pub source_dice_forbidden: bool,
    pub bonds_unit: Option<crate::UnitCard>,
    /// Spec 056, public : les cristaux qui reviendront à la fin du tour avec Maîtrise du cristal
    /// renforcée, au plus 3 par couleur.
    pub crystals_to_return: Vec<BasicColor>,
    /// Spec 057, public : le dé de la Source gardé par Puits de mana (par sa position), les cartes
    /// écartées par Distorsion temporelle, le tour supplémentaire en cours, les cartes de
    /// Méditation qui attendent leur place, et le Mouvement en combat d'Ailes de la nuit.
    pub kept_die: Option<u8>,
    pub set_aside: Vec<CardId>,
    pub extra_turn: bool,
    pub meditation: Option<Meditation>,
    pub fly_in_combat: bool,
    /// Spec 051, public : les compétences gagnées (état, options utilisables), les deux révélées
    /// en attente du choix, l'offre commune (prenable ou non), et la contrainte de la dernière
    /// place après une compétence de l'offre commune.
    pub skills: Vec<SkillView>,
    pub skill_choice: Vec<Skill>,
    pub common_skills: Vec<CommonSkillView>,
    pub lowest_advanced_action_only: bool,
    /// Spec #20, public : pendant le choix d'un niveau pair, pour chaque compétence prenable, les
    /// actions avancées de l'offre qu'elle laisse prendre, dans l'ordre de l'offre ; vide sinon.
    pub skill_choice_cards: Vec<(Skill, Vec<CardId>)>,
    /// Spec 060, public : pendant le choix, une case par tactique du moment (disponible, retirée,
    /// prise par un joueur) ; les tactiques retirées de la partie ; la tactique du joueur et son
    /// état ; les cartes du deck pendant le choix de Préparation, triées (pas leur ordre).
    pub tactic_choice: Vec<(Tactic, TacticSlot)>,
    pub removed_tactics: Vec<Tactic>,
    pub tactic: Option<TacticView>,
    pub preparation_choice: Vec<CardId>,
    /// Spec 061, public (MKUE p. 15, *Note 1* ; Compendium, *Miscellaneous* S2) : le joueur
    /// fantôme sans l'ordre de son deck ni de sa pile de compétences ; qui joue en premier.
    pub dummy: Option<DummyView>,
    pub round_order: Option<crate::RoundOrder>,
}

/// Le joueur fantôme tel que le joueur le voit (spec 061, C31).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummyView {
    pub hero: Hero,
    pub tactic: Option<Tactic>,
    pub deck_size: usize,
    pub discard: Vec<CardId>,
    /// Cristaux par couleur, dans l'ordre de `BasicColor::ALL`.
    pub crystals: Vec<(BasicColor, u8)>,
    pub last_turn: Vec<CardId>,
    pub announced: bool,
    pub skill_count: usize,
}

/// Spec 061 : le fantôme sans l'ordre de son deck ni de sa pile.
fn dummy_view(dummy: &crate::DummyState) -> DummyView {
    DummyView {
        hero: dummy.hero(),
        tactic: dummy.tactic(),
        deck_size: dummy.deck().len(),
        discard: dummy.discard().to_vec(),
        crystals: BasicColor::ALL
            .into_iter()
            .map(|color| (color, dummy.crystals(color)))
            .collect(),
        last_turn: dummy.last_turn().to_vec(),
        announced: dummy.announced(),
        skill_count: dummy.skills().len(),
    }
}

impl DummyView {
    pub fn crystals(&self, color: BasicColor) -> u8 {
        self.crystals
            .iter()
            .find(|(held, _)| *held == color)
            .map_or(0, |(_, count)| *count)
    }
}

/// La retraite forcée annoncée, hors assaut en cours : la figurine est alors sur le site, qu'elle
/// quittera par la retraite d'assaut (spec 052, C26).
fn forced_withdrawal_preview(
    state: &GameState,
    player: PlayerId,
) -> Option<ForcedWithdrawalPreview> {
    let p = state.player(player)?;
    if p.assault.is_some() {
        return None;
    }
    let path = forced_withdrawal_path(state, p)?;
    Some(ForcedWithdrawalPreview {
        to: *path.last()?,
        wounds: path.len(),
    })
}

/// La retraite forcée annoncée (spec 052) : la case d'arrivée et le nombre de Blessures, une par
/// case parcourue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForcedWithdrawalPreview {
    pub to: Hex,
    pub wounds: usize,
}

impl PlayerView {
    /// Vrai si l'effet se joue maintenant, mana mis à part : la même règle que les actions légales
    /// (issue privée 111), pour que l'écran montre aussi les effets renforcés faute de mana.
    pub fn is_playable_now(&self, effect: Effect) -> bool {
        self.is_option_playable_now(effect, &[])
    }

    /// Vrai si l'offre d'unités a une unité (spec 057).
    fn unit_offer(&self) -> bool {
        self.offers
            .get(&Offer::Units)
            .is_some_and(|cards| cards.iter().any(|card| card.unit().is_some()))
    }

    /// Vrai si une option (effet de points et modificateurs posés) se joue maintenant, mana mis à
    /// part (spec 052) : ses points servent, ou l'un de ses modificateurs.
    pub fn is_option_playable_now(&self, effect: Effect, modifiers: &[ModifierKind]) -> bool {
        // Un combat fini laisse le tour se dérouler comme avant (`Moment::Free`).
        let combat = self.combat.as_ref().filter(|combat| !combat.ended);
        is_playable_now(
            effect,
            modifiers,
            PlayMoment {
                combat,
                movement_ended: self.movement_ended,
                interacting: self.interaction.is_some(),
                modifiers: &self.modifiers,
                // L'aventure de nuit n'est pas dans la vue : les actions légales le savent.
                units_fight: !self.units.is_empty() && self.burning.is_none(),
                // Spec 057 : le Mouvement d'Ailes de la nuit, le tour supplémentaire.
                fly_in_combat: self.fly_in_combat,
                extra_turn: self.extra_turn,
                unit_offer: self.unit_offer(),
                offer_units_fight: self.unit_offer()
                    && (combat.is_none() || self.burning.is_none()),
            },
        )
    }
}

pub fn view(state: &GameState, player: PlayerId) -> PlayerView {
    let mut view = base_view(state, player);
    if view.forced_withdrawal.is_some() {
        // Calculé par le moteur, comme les fins de tour légales ; la vue d'après n'a plus de retraite.
        let after = after_forced_withdrawal(state, player);
        view.after_withdrawal = Some(Box::new(base_view(&after, player)));
    }
    view
}

fn base_view(state: &GameState, player: PlayerId) -> PlayerView {
    PlayerView {
        player,
        hand: state.hand(player).to_vec(),
        deck_size: state.deck(player).len(),
        has_pending_decision: pending_decision(state).is_some_and(|d| d.player == player),
        scenario: state.scenario().copied(),
        hero: state.hero(player),
        map: state.map().clone(),
        figure: state.figure(player),
        tile_deck: state.tile_deck().counts(),
        play_area: state.play_area(player).to_vec(),
        move_points: state.move_points(player),
        influence_points: state.influence_points(player),
        discard: state.discard_pile(player).to_vec(),
        time_of_day: state.time_of_day(),
        next_tile: next_tile(state.map(), state.tile_deck()),
        removed_tiles: removed_counts(state.map(), state.tile_deck()),
        turn: state.turn(),
        round: state.round(),
        source: state.source().to_vec(),
        inventory: state.inventory(player),
        dice_taken: state.dice_taken(player).to_vec(),
        mana_tokens: state.mana_tokens(player).to_vec(),
        dice_limit: state.dice_limit(player),
        dice_set: state.dice_set(player).to_vec(),
        enemy_tokens: state
            .enemy_tokens()
            .iter()
            .filter(|(hex, _)| !state.is_face_down(**hex))
            .map(|(hex, tokens)| (*hex, tokens.clone()))
            .collect(),
        face_down_defenders: state.face_down_defenders(),
        cities: state.cities().clone(),
        enemy_piles: EnemyColor::ALL
            .into_iter()
            .map(|color| (color, state.enemy_pile(color).len()))
            .collect(),
        movement_ended: state.movement_ended(player),
        enemies_to_fight: state.enemies_to_fight(player).to_vec(),
        combat: state.combat(player).cloned(),
        fame: state.fame(player),
        enemy_discards: EnemyColor::ALL
            .into_iter()
            .map(|color| (color, state.enemy_discard(color).to_vec()))
            .collect(),
        can_rest: state.can_rest(player),
        played_elsewhere: state.player(player).is_some_and(|p| p.played_elsewhere),
        banner_reclaim: state.player(player).is_some_and(|p| p.banner_reclaim),
        turn_action_taken: state.turn_action_taken(player),
        level: state.level(player),
        armor: state.armor(player),
        hand_limit: state.hand_limit(player),
        command_tokens: state.command_tokens(player),
        next_level_fame: GameContent::embedded()
            .fame_track()
            .next_threshold(state.level(player)),
        pending_level: state.pending_level(player),
        levels_gained: state.levels_gained(player).to_vec(),
        last_turn: state.last_turn(),
        game_end: state.game_end(),
        // Le décompte compte les Blessures du deck, caché : seulement quand plus rien ne se pioche.
        final_score: state
            .game_end()
            .filter(|_| {
                state
                    .scenario()
                    .is_some_and(|scenario| scenario.scoring() != FinalScoring::None)
            })
            .map(|_| final_score(state, player)),
        conquest: crate::conquest_progress(state),
        offers: Offer::ALL
            .into_iter()
            .map(|offer| (offer, state.offer(offer).to_vec()))
            .collect(),
        common_decks: CommonDeck::ALL
            .into_iter()
            .map(|deck| (deck, state.common_deck(deck).len()))
            .collect(),
        advanced_action_choices: state.advanced_action_choices(player),
        reputation: state.reputation(player),
        reputation_bonus: state.reputation_bonus(player),
        interaction: state.interaction(player),
        units: state.units(player).to_vec(),
        healing_points: state.healing_points(player),
        recruitable_units: crate::engine::recruitable_units(state, player),
        shields: state.shields().clone(),
        burned_monasteries: state.burned_monasteries(),
        burning: state.burning(player),
        artifact_rewards: state.artifact_rewards(player),
        drawn_artifacts: state.drawn_artifacts(player).to_vec(),
        assault: state.assault(player),
        spell_rewards: state.spell_rewards(player),
        spell_choices: state.spell_choices(player),
        adventure: state.adventure(player),
        combat_time_of_day: state.combat_time_of_day(player),
        crystal_rewards: state.crystal_rewards(player),
        crystal_choices: state.crystal_choices(player),
        reward_die: state.reward_die(player),
        keep_hand_bonus: state.keep_hand_bonus(player),
        city_hand_bonus: state.city_hand_bonus(player),
        ruins: state
            .ruins()
            .iter()
            .map(|(&hex, placed)| (hex, placed.face_up.then_some(placed.token)))
            .collect(),
        ruin_pile: state.ruin_pile().len(),
        skill_pile_count: state.skill_pile(player).len(),
        crystal_set_rewards: state.crystal_set_rewards(player),
        advanced_action_rewards: state.advanced_action_rewards(player),
        unit_rewards: state.unit_rewards(player),
        unit_choices: state.unit_choices(player),
        artifacts_to_keep: state.artifacts_to_keep(player),
        revealed_ruins: state.revealed_ruins(player).to_vec(),
        turn_facts: state.turn_facts(player).to_vec(),
        spent_mana_tokens: state.spent_mana_tokens(player).to_vec(),
        modifiers: state.modifiers(player).to_vec(),
        move_rules: state.move_rules(player),
        forced_withdrawal: forced_withdrawal_preview(state, player),
        after_withdrawal: None,
        mana_choice: state.mana_choice(player),
        crystals_to_return: state.crystals_to_return(player),
        kept_die: state.kept_die(player),
        skills: skill_views(state, player),
        center_advantages: state.center_advantages(player).to_vec(),
        interaction_influence: state.interaction_influence(player),
        source_dice_forbidden: state.source_dice_forbidden(player),
        bonds_unit: state.bonds_unit(player),
        skill_choice: state.skill_choice(player).to_vec(),
        common_skills: state
            .common_skills()
            .iter()
            .map(|&skill| CommonSkillView {
                skill,
                takeable: !state.skill_choice(player).is_empty()
                    && crate::legal_actions(state, player).contains(&Action::ChooseSkill { skill })
                    && !state.skill_choice(player).contains(&skill),
            })
            .collect(),
        lowest_advanced_action_only: state.lowest_advanced_action_only(player),
        skill_choice_cards: crate::engine::skill_choice_cards(state, player),
        set_aside: state.set_aside(player).to_vec(),
        extra_turn: state.extra_turn(player),
        meditation: state.player(player).and_then(|p| p.meditation.clone()),
        fly_in_combat: state
            .player(player)
            .is_some_and(|p| wings_of_night_castable(state, p)),
        tactic_choice: tactic_choice(state, player),
        removed_tactics: state.removed_tactics().to_vec(),
        tactic: state.tactic(player).map(|tactic| TacticView {
            tactic: tactic.tactic,
            flipped: tactic.flipped,
            stolen_die: tactic
                .stolen_die
                .and_then(|die| Some((die, *state.source().get(usize::from(die))?))),
            stored: tactic.stored_count(),
            mana_search_used: tactic.mana_search_used,
            right_moment: tactic.right_moment,
        }),
        preparation_choice: preparation_choice(state, player),
        dummy: state.dummy().map(dummy_view),
        round_order: state.round_order(),
    }
}

/// Spec 060 (remarque de Mathieu, grille 2 × 3) : pendant le choix du joueur, les six tactiques
/// du moment, chacune disponible, retirée de la partie, ou prise cette manche par un joueur.
fn tactic_choice(state: &GameState, player: PlayerId) -> Vec<(Tactic, TacticSlot)> {
    let choosing = state.tactic_choice && state.player(player).is_some_and(|p| p.tactic.is_none());
    if !choosing
        || state
            .scenario()
            .is_none_or(|s| s.tactics() == crate::TacticRule::None)
    {
        return Vec::new();
    }
    Tactic::of(state.time_of_day())
        .map(|tactic| {
            let taken = state
                .players
                .iter()
                .position(|p| p.tactic.as_ref().is_some_and(|held| held.tactic == tactic));
            let slot = if state.removed_tactics().contains(&tactic) {
                TacticSlot::Removed
            } else if let Some(index) = taken.and_then(|index| u8::try_from(index).ok()) {
                TacticSlot::Taken(PlayerId(index))
            } else {
                TacticSlot::Available
            };
            (tactic, slot)
        })
        .collect()
}

/// Spec 060 (C30) : les cartes du deck pendant le choix de Préparation, triées.
fn preparation_choice(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let preparing = state.player(player).is_some_and(|p| {
        p.tactic_follow_up
            && p.tactic
                .as_ref()
                .is_some_and(|tactic| tactic.tactic == Tactic::Preparation)
    });
    if !preparing {
        return Vec::new();
    }
    let mut cards = state.deck(player).to_vec();
    cards.sort();
    cards
}

/// Spec 051 (C26) et 058 (C44) : les compétences gagnées, avec ce que les actions légales en font
/// maintenant : options, cartes, unités et cibles.
fn skill_views(state: &GameState, player: PlayerId) -> Vec<SkillView> {
    let legal = crate::legal_actions(state, player);
    let blocked = state.motivation_blocked(player);
    state
        .skills(player)
        .iter()
        .map(|owned| {
            let mut seen = SkillView {
                skill: owned.skill,
                state: owned.state,
                usable: Vec::new(),
                unsupported: owned.skill.definition().effect == SkillEffect::Unsupported,
                motivation_blocked: blocked && owned.skill.is_motivation(),
                cards: Vec::new(),
                units: Vec::new(),
                targets: Vec::new(),
            };
            for action in &legal {
                note_skill_use(&mut seen, action);
            }
            seen
        })
        .collect()
}

fn push_new<T: PartialEq>(list: &mut Vec<T>, item: T) {
    if !list.contains(&item) {
        list.push(item);
    }
}

/// Ce que cette action légale fait de la compétence vue (spec 058) : une option utilisée seule,
/// un jeu de côté, un paiement (Invocation, Polarisation), une unité sous Charisme.
fn note_skill_use(seen: &mut SkillView, action: &Action) {
    match action {
        Action::UseSkill {
            skill,
            option,
            target,
            ..
        } if *skill == seen.skill => {
            push_new(&mut seen.usable, *option);
            if let Some(target) = target {
                push_new(&mut seen.targets, *target);
            }
        }
        Action::PlayCard { card, play } | Action::PlayCardOnUnit { card, play, .. } => {
            if let CardPlay::SidewaysSkill { skill, option, .. } = play
                && *skill == seen.skill
            {
                push_new(&mut seen.usable, *option);
                push_new(&mut seen.cards, *card);
            }
            for payment in play_payments(play) {
                note_payment(seen, payment);
            }
        }
        Action::ActivateUnit {
            unit,
            mana,
            leadership,
            ..
        } => {
            if *leadership && seen.skill == Skill::Leadership {
                push_new(&mut seen.usable, 0);
                push_new(&mut seen.units, *unit);
            }
            for &payment in mana {
                note_payment(seen, payment);
            }
        }
        _ => {}
    }
}

/// Les paiements d'un jeu de carte.
fn play_payments(play: &CardPlay) -> Vec<ManaPayment> {
    match *play {
        CardPlay::Basic { mana, .. } => mana.into_iter().collect(),
        CardPlay::SidewaysSkill { mana, .. } => mana.map(|mana| mana.payment).into_iter().collect(),
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
        _ => Vec::new(),
    }
}

fn note_payment(seen: &mut SkillView, payment: ManaPayment) {
    let invoked = match payment {
        ManaPayment::Invocation(card) => Some(card),
        ManaPayment::Polarized { source, .. } => {
            if seen.skill == Skill::Polarization {
                push_new(&mut seen.usable, 0);
            }
            match source {
                PolarSource::Invocation(card) => Some(card),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(card) = invoked
        && seen.skill == Skill::Invocation
    {
        push_new(&mut seen.usable, 0);
        push_new(&mut seen.cards, card);
    }
}
