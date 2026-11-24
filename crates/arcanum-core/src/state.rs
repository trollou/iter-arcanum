use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Assault, BasicColor, CardId, Combat, CommonCard, CommonDeck, DICE_PER_TURN, DeedDeck,
    EnemyColor, EnemyToken, Feature, GameContent, GameEndReason, Hero, Hex, Interaction, Inventory,
    LastTurn, LevelToken, ManaColor, Map, Modifier, MoveRules, Offer, OwnedUnit, PlacedRuin,
    PlayedCard, PlayerId, Reputation, RuinToken, Scenario, Site, TileDeck, TimeOfDay, TurnFact,
    level_token, reputation_bonus,
};

/// Tout l'état de la partie, information cachée comprise.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameState {
    /// Le joueur d'indice i est `PlayerId(i)`.
    pub(crate) players: Vec<PlayerState>,
    pub(crate) scenario: Option<Scenario>,
    pub(crate) map: Map,
    pub(crate) tile_deck: TileDeck,
    /// Commun à tous les joueurs ; Jour dès `GameStarted` (spec 014).
    pub(crate) time_of_day: TimeOfDay,
    /// Numéro du tour dans la manche ; 1 dès `GameStarted` (spec 016).
    pub(crate) turn: u32,
    /// Numéro de la manche ; 1 dès `GameStarted` (spec 023).
    pub(crate) round: u32,
    /// Vrai entre `RoundEnded` et le `RoundStarted` suivant : plus de décision (spec 023).
    pub(crate) round_over: bool,
    /// Couleur de chaque dé de la Source, dans l'ordre du lancer (spec 020).
    pub(crate) source: Vec<ManaColor>,
    /// Piles d'ennemis, le dessus en premier : ordre caché (spec 030).
    pub(crate) enemy_piles: BTreeMap<EnemyColor, Vec<EnemyToken>>,
    /// Jetons d'ennemis sur le plateau de jeu, une liste par case (spec 044, Q1) ; face visible
    /// et publics, sauf les défenseurs de `face_down` (spec 030).
    pub(crate) enemy_tokens: BTreeMap<Hex, Vec<EnemyToken>>,
    /// Défausses d'ennemis, une par couleur, publiques (spec 031).
    pub(crate) enemy_discards: BTreeMap<EnemyColor, Vec<EnemyToken>>,
    /// Dernier tour annoncé par une cité révélée (spec 063) ; changé par `LastTurnAnnounced`,
    /// puis par le `TurnStarted` qui ouvre le dernier tour.
    pub(crate) last_turn: LastTurn,
    /// Partie terminée, et pourquoi ; changé seulement par `GameEnded` (spec 063).
    pub(crate) game_end: Option<GameEndReason>,
    /// Pioches communes, le dessus en premier : ordre caché (spec 050).
    pub(crate) common_decks: BTreeMap<CommonDeck, Vec<CommonCard>>,
    /// Offres, publiques, la première place en tête (spec 050).
    pub(crate) offers: BTreeMap<Offer, Vec<CommonCard>>,
    /// Jetons Bouclier posés sur le plateau de jeu, par case : le joueur qui l'a posé (spec 041).
    pub(crate) shields: BTreeMap<Hex, PlayerId>,
    /// Cases des jetons d'`enemy_tokens` encore face cachée : les défenseurs pas encore retournés
    /// (spec 043). Le moteur connaît le jeton, pas le joueur.
    pub(crate) face_down: BTreeSet<Hex>,
    /// Pile des jetons de ruines, le dessus en premier : ordre caché (spec 045).
    pub(crate) ruin_pile: Vec<RuinToken>,
    /// Défausse des jetons de ruines des ruines conquises (Compendium, *Miscellaneous* S6).
    pub(crate) ruin_discard: Vec<RuinToken>,
    /// Le jeton de ruines de chaque ruine non conquise, et sa face (spec 045) ; une ruine conquise
    /// n'y est plus (son Bouclier est dans `shields`).
    pub(crate) ruins: BTreeMap<Hex, PlacedRuin>,
    /// Les cités révélées, par case (spec 062).
    pub(crate) cities: BTreeMap<Hex, crate::CityState>,
    /// Offre de compétences communes, dans l'ordre d'arrivée (MKUE p. 3 ; spec 051).
    pub(crate) common_skills: Vec<crate::Skill>,
    /// Tactiques retirées de la partie, dans l'ordre du retrait (spec 060).
    pub(crate) removed_tactics: Vec<crate::Tactic>,
    /// Le choix des tactiques de la manche attend (spec 060) : posé par `GameStarted` et
    /// `RoundStarted` quand le scénario a des tactiques, levé par `TacticChosen`.
    pub(crate) tactic_choice: bool,
    /// Le joueur fantôme, absent sans fantôme (spec 061).
    pub(crate) dummy: Option<crate::DummyState>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlayerState {
    pub(crate) hero: Option<Hero>,
    pub(crate) deck: DeedDeck,
    pub(crate) hand: Vec<CardId>,
    pub(crate) figure: Option<Hex>,
    /// Cartes jouées ce tour, dans l'ordre (spec 013).
    pub(crate) play_area: Vec<PlayedCard>,
    pub(crate) move_points: u32,
    /// Signés : un malus de réputation donne une influence négative (spec 040).
    pub(crate) influence_points: i32,
    pub(crate) discard_pile: Vec<CardId>,
    /// Cristaux de la carte de héros (spec 021).
    pub(crate) inventory: Inventory,
    /// Position dans la Source des dés pris ce tour, dans l'ordre (spec 022).
    pub(crate) dice_taken: Vec<u8>,
    /// Jetons de mana de la zone de jeu, dans l'ordre où ils ont été gagnés (spec 025).
    /// Toutes couleurs : or et noir viennent des Mages d'Altem (spec 042).
    pub(crate) mana_tokens: Vec<ManaColor>,
    /// Dés de plus ce tour, et combien d'entre eux font d'un dé noir toute couleur (spec 025).
    pub(crate) extra_dice: u8,
    pub(crate) black_any_dice: u8,
    /// Position dans la Source des dés mis sur une couleur ce tour, dans l'ordre (spec 025).
    pub(crate) dice_set: Vec<u8>,
    /// Vrai après une provocation, jusqu'à la fin du tour (spec 030).
    pub(crate) movement_ended: bool,
    /// Cases des ennemis provoqués ou défiés ce tour (specs 030 et 031).
    pub(crate) enemies_to_fight: Vec<Hex>,
    /// Le combat du tour, engagé ou fini ; vidé par la fin du tour (spec 031).
    pub(crate) combat: Option<Combat>,
    /// Gloire gagnée, jamais remise à zéro (spec 031).
    pub(crate) fame: u32,
    /// Blessures déjà données au joueur : le numéro d'exemplaire de la prochaine suit (spec 031).
    pub(crate) wounds_given: u16,
    /// Vrai si le héros s'est déplacé, a exploré, a défié, a combattu ou a interagi ce tour : plus
    /// de repos (MKUE p. 6, 7 ; specs 034 et 040). Remis à faux par la fin du tour.
    pub(crate) moved_or_fought: bool,
    /// Niveau du héros, 1 à la mise en place ; changé seulement par `LevelGained`, à la fin du
    /// tour (spec 035).
    pub(crate) level: u8,
    /// Niveaux gagnés à la dernière fin de tour, pour le bandeau du tour suivant (spec 035) ;
    /// vidés par la fin du tour suivant ou de la manche.
    pub(crate) levels_gained: Vec<u8>,
    /// Choix d'une action avancée en attente, un par niveau pair gagné à la fin du tour
    /// (spec 050) ; 0 hors de cette fin de tour.
    pub(crate) advanced_action_choices: u8,
    /// Réputation, 0 à la mise en place ; changée seulement par `ReputationChanged` (spec 040).
    pub(crate) reputation: Reputation,
    /// L'interaction du tour, depuis `InteractionStarted` ; vidée par la fin du tour (spec 040).
    pub(crate) interaction: Option<Interaction>,
    /// Zone des unités, dans l'ordre du recrutement (spec 042).
    pub(crate) units: Vec<OwnedUnit>,
    /// Réserve de points de soin du tour (spec 042, Q1).
    pub(crate) healing: u32,
    /// Le monastère incendié ce tour, depuis `MonasteryBurnStarted` : son combat exclut les unités
    /// (spec 041) ; vidé par la fin du tour.
    pub(crate) burning: Option<Hex>,
    /// Récompenses artefact gagnées ce tour, prises à la fin du tour (spec 041) ; vidées par le
    /// tour suivant.
    pub(crate) artifact_rewards: u8,
    /// Artefacts tirés pour une récompense, dans l'ordre du tirage, en attente du choix (spec 041).
    pub(crate) drawn_artifacts: Vec<CardId>,
    /// Récompenses sort gagnées ce tour (tour de mage conquise, spec 043) ; à la fin du tour,
    /// elles deviennent des choix en attente, ou sont perdues si l'offre de sorts est vide.
    pub(crate) spell_rewards: u8,
    /// Choix de sort en attente à la fin du tour (spec 043) ; vidés par le tour suivant.
    pub(crate) spell_choices: u8,
    /// Une unité activée ce tour : le tour est commencé, plus de pillage (spec 041) ; vidé par la
    /// fin du tour.
    pub(crate) unit_activated: bool,
    /// Le village pillé ce tour (spec 041) ; vidé par la fin du tour.
    pub(crate) plundered: bool,
    /// Unités recrutées ce tour, renvoyées ensuite comprises (Récit héroïque, spec 053) ; vidé par
    /// la fin du tour.
    pub(crate) units_recruited: u8,
    /// L'assaut du tour, depuis `AssaultStarted` ; vidé par la fin du tour (spec 043).
    pub(crate) assault: Option<Assault>,
    /// Le site d'aventure où le héros est entré ce tour, depuis `AdventureSiteEntered`, ou l'autel
    /// de son offrande (`AltarTributePaid`, spec 045) ; vidé par la fin du tour (spec 044).
    pub(crate) adventure: Option<Hex>,
    /// Vrai si ce site est un donjon ou un tombeau : son combat se livre seul et de nuit (spec 044).
    pub(crate) night_adventure: bool,
    /// Cristaux aléatoires à lancer à la fin du tour (antre, Sombre Terrier ; spec 044).
    pub(crate) crystal_rewards: u8,
    /// Dés de récompense or en attente du choix de la couleur du cristal (spec 044).
    pub(crate) crystal_choices: u8,
    /// Dés or de la Poche de mana en attente du choix de la couleur, pendant le tour (spec
    /// 055).
    pub(crate) pouch_choices: u8,
    /// Le dé de récompense lancé à la conquête d'un donjon ce tour, vidé par la fin du tour : l'écran
    /// l'annonce (spec 044, C45).
    pub(crate) reward_die: Option<ManaColor>,
    /// Artefacts à garder parmi ceux tirés pour les récompenses (*n* parmi *n* + 1, MKUE p. 11,
    /// 5.b ; spec 045) ; vidés par la fin des choix.
    pub(crate) artifacts_to_keep: u8,
    /// Récompenses « un cristal de chaque couleur » gagnées ce tour (ruines, spec 045).
    pub(crate) crystal_set_rewards: u8,
    /// Récompenses action avancée gagnées ce tour (ruines, spec 045) ; à la fin du tour, elles
    /// deviennent des choix d'action avancée, ou sont perdues si l'offre est vide.
    pub(crate) advanced_action_rewards: u8,
    /// Récompenses unité gagnées ce tour (ruines, spec 045) ; à la fin du tour, elles deviennent
    /// des choix d'unité, ou sont perdues sans unité dans l'offre.
    pub(crate) unit_rewards: u8,
    /// Choix d'unité de récompense en attente à la fin du tour (spec 045).
    pub(crate) unit_choices: u8,
    /// Ruines révélées par le dernier pas ou trajet, de Nuit (spec 045, C40) ; vidé par l'action
    /// suivante.
    pub(crate) revealed_ruins: Vec<Hex>,
    /// Faits du tour, dans l'ordre des événements ; vidés au début du tour suivant (spec 073).
    pub(crate) turn_facts: Vec<TurnFact>,
    /// Jetons de mana dépensés ce tour, dans l'ordre ; vidés avec les jetons à la fin du tour
    /// (spec 073).
    pub(crate) spent_mana_tokens: Vec<ManaColor>,
    /// Effets en cours, dans l'ordre où ils ont été posés (spec 052, ADR 0008).
    pub(crate) modifiers: Vec<Modifier>,
    /// Trajet du tour (spec 052) : la case du début du tour, puis chaque case entrée ; la retraite
    /// d'un assaut retire le site. La retraite forcée le remonte.
    pub(crate) turn_path: Vec<Hex>,
    /// Le choix en attente de Rituel de sang renforcé (spec 056).
    pub(crate) mana_choice: Option<ManaChoice>,
    /// Les cristaux dépensés ce tour, dans l'ordre (`CrystalUsed`), pour Maîtrise du cristal
    /// renforcée (spec 056) ; vidés à la fin du tour.
    pub(crate) crystals_spent: Vec<BasicColor>,
    /// Tempête de mana renforcée (spec 056) : un dé noir ou or pris vaut toute couleur de base, de
    /// jour comme de nuit, même épuisé ; pour le tour.
    pub(crate) any_color_dice: bool,
    /// Cartes écartées jusqu'à la fin de la manche, remélangées dans le deck à la nouvelle manche
    /// (Distorsion temporelle, spec 057).
    pub(crate) set_aside: Vec<CardId>,
    /// Le tour en cours est un tour supplémentaire (Distorsion temporelle, spec 057) ; vidé par le
    /// tour suivant.
    pub(crate) extra_turn: bool,
    /// Un tour supplémentaire suit la fin de ce tour, à la place de la pioche (spec 057).
    pub(crate) extra_turn_pending: bool,
    /// Les cartes de Méditation qui attendent leur place sur ou sous le deck, et combien en
    /// remettre (spec 057).
    pub(crate) meditation: Option<Meditation>,
    /// Le dé de la Source gardé par Puits de mana jusqu'à la fin de la manche, par sa position
    /// (spec 057).
    pub(crate) kept_die: Option<u8>,
    /// Spec 055 : une carte jouée ce tour n'est plus dans la zone de jeu (artefact détruit,
    /// bannière assignée) ; elle compte pour le tour minimal.
    pub(crate) played_elsewhere: bool,
    /// Spec 055 (Q3) : la couleur des sorts lancés ce tour, pour la gloire des anneaux.
    pub(crate) spells_cast: Vec<BasicColor>,
    /// Spec 055 (Bannière de gloire renforcée) : les unités qui ont attaqué ou paré ce tour.
    pub(crate) units_fought: Vec<crate::UnitCard>,
    /// Spec 055 (Bannière de protection renforcée) : les Blessures reçues ce tour en main (moins
    /// celles soignées depuis) et dans la défausse.
    pub(crate) wounds_received_hand: u32,
    pub(crate) wounds_received_discard: u32,
    /// Spec 055 (Q2) : la manche finie, le choix des bannières à reprendre attend.
    pub(crate) banner_reclaim: bool,
    /// Pile de compétences, le dessus en premier : ordre caché (MKUE p. 3 ; spec 051).
    pub(crate) skill_pile: Vec<crate::Skill>,
    /// Compétences gagnées, dans l'ordre du gain, et leur état (spec 051).
    pub(crate) skills: Vec<crate::OwnedSkill>,
    /// Les deux compétences révélées en attente de choix, à la fin d'un tour à niveau pair.
    pub(crate) skill_choice: Vec<crate::Skill>,
    /// Niveaux pairs dont la compétence reste à choisir, à la fin du tour (spec 051).
    pub(crate) skill_choices: u8,
    /// Une compétence prise dans l'offre commune : l'action avancée en dernière place seulement
    /// (MKUE p. 11, 6.b).
    pub(crate) lowest_advanced_action_only: bool,
    /// Motivation : 2 à l'utilisation, 1 de moins à chaque fin de tour ; une autre Motivation
    /// attend 0 (texte de la compétence ; Compendium, *Motivation* S5).
    pub(crate) motivation_block: u8,
    /// Influence d'interaction de Fortune clinquante (spec 058, Q3) ; vidée par la fin du tour.
    pub(crate) interaction_influence: u32,
    /// Plus aucun dé de la Source ce tour, après Magie inutile à +3 (spec 058) ; vidé par la fin du
    /// tour.
    pub(crate) source_dice_forbidden: bool,
    /// Avantages des compétences posées au centre (spec 058, G), dans l'ordre où elles l'ont été.
    pub(crate) center_advantages: Vec<crate::CenterAdvantage>,
    /// L'unité sous le jeton de Loyauté sans faille (spec 058), ou aucune.
    pub(crate) bonds_unit: Option<crate::UnitCard>,
    /// Le prochain dé pris est celui de l'avantage d'Embrasure de la Source (spec 058).
    pub(crate) opening_die_due: bool,
    /// Le dé pris avec l'avantage d'Embrasure de la Source ce tour, et s'il est gardé tel quel à la
    /// fin du tour (spec 058) ; vidés par la fin du tour.
    pub(crate) opening_die: Option<u8>,
    pub(crate) opening_die_kept: bool,
    /// La tactique de la manche et son état (spec 060) ; vidée par `RoundEnded`.
    pub(crate) tactic: Option<crate::TacticState>,
    /// Le choix qui suit la prise de la tactique attend : cartes de Faux départ, dé de Vol de
    /// mana, carte de Préparation (spec 060, Q3) ; vidé par `TurnStarted`.
    pub(crate) tactic_follow_up: bool,
    /// La pioche de fin de tour arrêtée, deck épuisé : cartes qui manquent, Longue Nuit proposée
    /// (spec 060, Q5) ; vidé par `TurnStarted`.
    pub(crate) draw_stopped: Option<usize>,
    /// Le choix de Provisions attend, avant le tour (spec 060, C31) ; posé par `start_turn`.
    pub(crate) sparing_pending: bool,
}

/// Le choix en attente de Méditation (spec 057) : `pick` cartes parmi `cards` (toutes, si elles
/// ont été tirées au hasard ; toute la défausse pour Transe).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meditation {
    pub cards: Vec<CardId>,
    pub pick: usize,
}

/// Le choix en attente de Rituel de sang renforcé (spec 056) : les couleurs de jetons restant à
/// choisir, puis le cristal proposé.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManaChoice {
    /// Jetons restant à choisir, un par un.
    pub tokens_left: u8,
    /// Les jetons choisis, le cristal payé d'un mana de base est proposé, une fois.
    pub crystal_offered: bool,
}

impl PlayerState {
    /// Le moment pour le mana or (spec 055) : le Jour avec l'Amulette du soleil (`DayRules`),
    /// sinon `time`.
    pub(crate) fn gold_time(&self, time: TimeOfDay) -> TimeOfDay {
        if self
            .modifiers
            .iter()
            .any(|modifier| modifier.kind == crate::ModifierKind::DayRules)
        {
            TimeOfDay::Day
        } else {
            time
        }
    }

    /// Le moment de la journée pour le combat en cours du joueur (spec 044) : la Nuit pendant un
    /// combat de donjon ou de tombeau, sinon `time` (cartes de site ; Compendium, *Combat* Q2).
    pub(crate) fn combat_time(&self, time: TimeOfDay) -> TimeOfDay {
        let fighting = self.combat.as_ref().is_some_and(|combat| !combat.ended);
        if fighting && self.night_adventure {
            TimeOfDay::Night
        } else {
            time
        }
    }

    /// Vrai si l'avantage du centre de `skill` sert ce tour, pas encore utilisé (spec 058, Q1).
    pub(crate) fn center_advantage(&self, skill: crate::Skill) -> bool {
        self.center_advantages
            .iter()
            .any(|advantage| advantage.skill == skill && advantage.active && !advantage.used)
    }

    /// L'Influence d'un achat d'interaction : l'ordinaire et celle de Fortune clinquante (spec 058,
    /// Q3).
    pub(crate) fn purchase_influence(&self) -> i32 {
        self.influence_points
            .saturating_add(i32::try_from(self.interaction_influence).unwrap_or(i32::MAX))
    }

    /// Paie un achat d'interaction : l'Influence d'interaction d'abord (spec 058, Q3).
    pub(crate) fn spend_purchase_influence(&mut self, cost: u8) {
        let from_interaction = self.interaction_influence.min(u32::from(cost));
        self.interaction_influence -= from_interaction;
        self.influence_points -= i32::from(cost) - i32::try_from(from_interaction).unwrap_or(0);
    }

    /// Vrai si les unités ne participent pas au combat en cours : incendie d'un monastère
    /// (spec 041), donjon ou tombeau (spec 044).
    pub(crate) fn units_sit_out(&self) -> bool {
        self.burning.is_some() || self.night_adventure
    }
}

impl PlayerState {
    /// Le jeton de niveau en vigueur : armure, limite de main non modifiée (spec 035).
    pub(crate) fn level_token(&self) -> LevelToken {
        level_token(self.level)
    }

    /// Le niveau de la ligne où est la gloire, atteint à la fin du tour (MKUE p. 11, 6).
    pub(crate) fn level_by_fame(&self) -> u8 {
        GameContent::embedded()
            .fame_track()
            .level_for_fame(self.fame)
            .max(self.level)
    }
}

impl GameState {
    pub(crate) fn player(&self, player: PlayerId) -> Option<&PlayerState> {
        self.players.get(player.index())
    }

    pub(crate) fn player_mut(&mut self, player: PlayerId) -> Option<&mut PlayerState> {
        self.players.get_mut(player.index())
    }

    /// Deck du joueur, dans l'ordre : information cachée, jamais dans une vue.
    pub fn deck(&self, player: PlayerId) -> &[CardId] {
        self.player(player).map_or(&[], |p| p.deck.cards())
    }

    pub fn hand(&self, player: PlayerId) -> &[CardId] {
        self.player(player).map_or(&[], |p| p.hand.as_slice())
    }

    pub fn scenario(&self) -> Option<&Scenario> {
        self.scenario.as_ref()
    }

    pub fn hero(&self, player: PlayerId) -> Option<Hero> {
        self.player(player).and_then(|p| p.hero)
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    /// Pioche de tuiles, dans l'ordre : information cachée, jamais dans une vue.
    pub fn tile_deck(&self) -> &TileDeck {
        &self.tile_deck
    }

    /// Case de la figurine du héros, si elle est sur le plateau de jeu.
    pub fn figure(&self, player: PlayerId) -> Option<Hex> {
        self.player(player).and_then(|p| p.figure)
    }

    /// Zone de jeu : les cartes jouées ce tour, dans l'ordre.
    pub fn play_area(&self, player: PlayerId) -> &[PlayedCard] {
        self.player(player).map_or(&[], |p| p.play_area.as_slice())
    }

    /// Spec 057 : les cartes écartées jusqu'à la fin de la manche (Distorsion temporelle).
    pub fn set_aside(&self, player: PlayerId) -> &[CardId] {
        self.player(player).map_or(&[], |p| p.set_aside.as_slice())
    }

    /// Spec 057 : le dé de la Source gardé par Puits de mana, par sa position.
    pub fn kept_die(&self, player: PlayerId) -> Option<u8> {
        self.player(player).and_then(|p| p.kept_die)
    }

    /// Spec 057 : le tour en cours est un tour supplémentaire (Distorsion temporelle).
    pub fn extra_turn(&self, player: PlayerId) -> bool {
        self.player(player).is_some_and(|p| p.extra_turn)
    }

    pub fn move_points(&self, player: PlayerId) -> u32 {
        self.player(player).map_or(0, |p| p.move_points)
    }

    pub fn influence_points(&self, player: PlayerId) -> i32 {
        self.player(player).map_or(0, |p| p.influence_points)
    }

    /// L'Influence d'interaction (Fortune clinquante, spec 058, Q3) : dépensée d'abord pour un
    /// achat d'interaction, perdue à la fin du tour.
    /// Spec 058 : les avantages du centre de ce tour.
    pub fn center_advantages(&self, player: PlayerId) -> &[crate::CenterAdvantage] {
        self.player(player).map_or(&[], |p| &p.center_advantages)
    }

    /// Spec 058 : plus de dé de la Source ce tour (Magie inutile à +3).
    pub fn source_dice_forbidden(&self, player: PlayerId) -> bool {
        self.player(player).is_some_and(|p| p.source_dice_forbidden)
    }

    pub fn interaction_influence(&self, player: PlayerId) -> u32 {
        self.player(player).map_or(0, |p| p.interaction_influence)
    }

    /// Spec 060 : la tactique du joueur pour la manche et son état.
    pub fn tactic(&self, player: PlayerId) -> Option<&crate::TacticState> {
        self.player(player).and_then(|p| p.tactic.as_ref())
    }

    /// Spec 060 : les tactiques retirées de la partie.
    pub fn removed_tactics(&self) -> &[crate::Tactic] {
        &self.removed_tactics
    }

    pub fn time_of_day(&self) -> TimeOfDay {
        self.time_of_day
    }

    pub fn turn(&self) -> u32 {
        self.turn
    }

    pub fn round(&self) -> u32 {
        self.round
    }

    pub fn is_round_over(&self) -> bool {
        self.round_over
    }

    /// Dernier tour annoncé, à venir ou en cours ; public (spec 063).
    pub fn last_turn(&self) -> LastTurn {
        self.last_turn
    }

    /// Raison de la fin de partie, une fois `GameEnded` appliqué ; publique (spec 063).
    pub fn game_end(&self) -> Option<GameEndReason> {
        self.game_end
    }

    /// Le joueur fantôme, s'il y en a un (spec 061) ; ordre de son deck compris.
    pub fn dummy(&self) -> Option<&crate::DummyState> {
        self.dummy.as_ref()
    }

    /// Qui joue en premier dans la manche, une fois les deux tactiques prises (spec 061 ; MKUE
    /// p. 4, 3.c) ; aucun ordre sans fantôme.
    pub fn round_order(&self) -> Option<crate::RoundOrder> {
        let dummy = self.dummy.as_ref()?.tactic?;
        let player = self.players.first()?.tactic.as_ref()?.tactic;
        Some(if dummy.number() < player.number() {
            crate::RoundOrder::DummyFirst
        } else {
            crate::RoundOrder::PlayerFirst
        })
    }

    /// Source : couleur de chaque dé, dans l'ordre du lancer ; publique.
    pub fn source(&self) -> &[ManaColor] {
        &self.source
    }

    /// Inventaire : cristaux par couleur de base ; public.
    pub fn inventory(&self, player: PlayerId) -> Inventory {
        self.player(player)
            .map_or_else(Inventory::default, |p| p.inventory)
    }

    /// Dés pris ce tour : leur position dans la Source, dans l'ordre ; public.
    pub fn dice_taken(&self, player: PlayerId) -> &[u8] {
        self.player(player).map_or(&[], |p| p.dice_taken.as_slice())
    }

    /// Nombre de dés que le joueur peut prendre ce tour : un, plus les dés de plus (spec 025) ;
    /// public.
    pub fn dice_limit(&self, player: PlayerId) -> usize {
        DICE_PER_TURN + self.player(player).map_or(0, |p| usize::from(p.extra_dice))
    }

    /// Dés mis sur une couleur ce tour : leur position dans la Source ; publics (spec 025).
    pub fn dice_set(&self, player: PlayerId) -> &[u8] {
        self.player(player).map_or(&[], |p| p.dice_set.as_slice())
    }

    /// Jetons de mana de la zone de jeu, dans l'ordre où ils ont été gagnés ; publics.
    pub fn mana_tokens(&self, player: PlayerId) -> &[ManaColor] {
        self.player(player)
            .map_or(&[], |p| p.mana_tokens.as_slice())
    }

    /// Faits du tour du joueur, dans l'ordre ; publics (spec 073).
    pub fn turn_facts(&self, player: PlayerId) -> &[TurnFact] {
        self.player(player).map_or(&[], |p| p.turn_facts.as_slice())
    }

    /// Les règles de déplacement du joueur : le moment de la journée et ses modificateurs de terrain
    /// (spec 052).
    pub fn move_rules(&self, player: PlayerId) -> MoveRules {
        MoveRules::new(self.time_of_day, self.modifiers(player))
    }

    /// Effets en cours du joueur, dans l'ordre où ils ont été posés ; publics (spec 052).
    pub fn modifiers(&self, player: PlayerId) -> &[Modifier] {
        self.player(player).map_or(&[], |p| p.modifiers.as_slice())
    }

    /// Le choix en attente de Rituel de sang renforcé, public (spec 056).
    pub fn mana_choice(&self, player: PlayerId) -> Option<ManaChoice> {
        self.player(player).and_then(|p| p.mana_choice)
    }

    /// Les cristaux qui reviendront à la fin du tour avec Maîtrise du cristal renforcée, au plus 3
    /// par couleur (spec 056, Q8) : les cristaux dépensés ce tour, dans l'ordre.
    pub fn crystals_to_return(&self, player: PlayerId) -> Vec<BasicColor> {
        let Some(p) = self.player(player) else {
            return Vec::new();
        };
        if !p
            .modifiers
            .iter()
            .any(|modifier| modifier.kind == crate::ModifierKind::ReturnSpentCrystals)
        {
            return Vec::new();
        }
        let mut inventory = p.inventory;
        let mut colors = Vec::new();
        for &color in &p.crystals_spent {
            if !inventory.is_full(color) {
                inventory.add(color);
                colors.push(color);
            }
        }
        colors
    }

    /// Jetons de mana dépensés ce tour, dans l'ordre ; publics (spec 073).
    pub fn spent_mana_tokens(&self, player: PlayerId) -> &[ManaColor] {
        self.player(player)
            .map_or(&[], |p| p.spent_mana_tokens.as_slice())
    }

    /// Pile d'ennemis d'une couleur, le dessus en premier : information cachée, jamais dans une vue.
    pub fn enemy_pile(&self, color: EnemyColor) -> &[EnemyToken] {
        self.enemy_piles.get(&color).map_or(&[], Vec::as_slice)
    }

    /// Jetons d'ennemis sur le plateau de jeu, une liste par case ; publics (spec 030), sauf les
    /// défenseurs face cachée (spec 043).
    pub fn enemy_tokens(&self) -> &BTreeMap<Hex, Vec<EnemyToken>> {
        &self.enemy_tokens
    }

    /// Les jetons d'ennemis de cette case, dans l'ordre où ils y sont arrivés (spec 044).
    pub fn enemy_tokens_at(&self, hex: Hex) -> &[EnemyToken] {
        self.enemy_tokens.get(&hex).map_or(&[], Vec::as_slice)
    }

    /// La case de ce jeton d'ennemi sur le plateau de jeu.
    pub(crate) fn enemy_token_hex(&self, token: EnemyToken) -> Option<Hex> {
        self.enemy_tokens
            .iter()
            .find(|(_, tokens)| tokens.contains(&token))
            .map(|(&hex, _)| hex)
    }

    /// Vrai si cette case porte au moins un jeton d'ennemi.
    pub(crate) fn has_enemy_token(&self, hex: Hex) -> bool {
        !self.enemy_tokens_at(hex).is_empty()
    }

    /// Ce jeton quitte sa case pour la défausse de sa couleur ; rien s'il n'y est pas.
    pub(crate) fn discard_enemy_token(&mut self, hex: Hex, token: EnemyToken) {
        let Some(tokens) = self.enemy_tokens.get_mut(&hex) else {
            return;
        };
        let Some(index) = tokens.iter().position(|&held| held == token) else {
            return;
        };
        tokens.remove(index);
        if tokens.is_empty() {
            self.enemy_tokens.remove(&hex);
            self.face_down.remove(&hex);
        }
        self.enemy_discards
            .entry(token.color())
            .or_default()
            .push(token);
    }

    /// Les défenseurs face cachée, par case : seule leur couleur est publique (spec 043).
    /// Une cité en a plusieurs sur sa case (spec 062).
    pub fn face_down_defenders(&self) -> BTreeMap<Hex, Vec<EnemyColor>> {
        self.face_down
            .iter()
            .map(|hex| {
                let colors = self
                    .enemy_tokens_at(*hex)
                    .iter()
                    .map(|token| token.color())
                    .collect();
                (*hex, colors)
            })
            .filter(|(_, colors): &(Hex, Vec<EnemyColor>)| !colors.is_empty())
            .collect()
    }

    /// Vrai si le jeton de cette case est un défenseur face cachée (spec 043).
    pub fn is_face_down(&self, hex: Hex) -> bool {
        self.face_down.contains(&hex)
    }

    /// Défausse d'ennemis d'une couleur, dans l'ordre où les jetons y sont arrivés ; publique
    /// (spec 031).
    pub fn enemy_discard(&self, color: EnemyColor) -> &[EnemyToken] {
        self.enemy_discards.get(&color).map_or(&[], Vec::as_slice)
    }

    /// Gloire du joueur, publique (spec 031).
    pub fn fame(&self, player: PlayerId) -> u32 {
        self.player(player).map_or(0, |p| p.fame)
    }

    /// Niveau du héros, public (spec 035) ; 1 hors partie.
    pub fn level(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(1, |p| p.level.max(1))
    }

    /// Armure du héros, celle du jeton de niveau (MKUE p. 9, *Assign Damage Phase* 5.a).
    pub fn armor(&self, player: PlayerId) -> u8 {
        level_token(self.level(player)).armor
    }

    /// Limite de main du jeton de niveau, sans bonus (MKUE p. 11, 7.b ; p. 9, 5.c).
    pub fn hand_limit(&self, player: PlayerId) -> usize {
        usize::from(level_token(self.level(player)).hand_limit)
    }

    /// Bonus de limite de main de la pioche (carte Keep ; Compendium, *End of Turn* S8 ; spec
    /// 043) : le nombre de forteresses possédées, si la figurine est sur l'une d'elles ou à côté.
    /// Le seuil d'assommé n'en tient pas compte (MKUE p. 9, 5.c).
    pub fn keep_hand_bonus(&self, player: PlayerId) -> usize {
        let owned: Vec<Hex> = self
            .shields
            .iter()
            .filter(|&(&hex, &owner)| {
                owner == player
                    && self.map.space(hex).and_then(|space| space.feature)
                        == Some(Feature::Site(Site::Keep))
            })
            .map(|(&hex, _)| hex)
            .collect();
        let near = self
            .player(player)
            .and_then(|p| p.figure)
            .is_some_and(|figure| {
                owned
                    .iter()
                    .any(|&keep| keep == figure || keep.neighbors().contains(&figure))
            });
        if near { owned.len() } else { 0 }
    }

    /// Bonus de limite de main d'une cité conquise (carte de cité ; Compendium, *Cities* S20 ;
    /// spec 062) : +1 avec un Bouclier dessus, +2 pour le Seigneur, sur la cité ou à côté.
    pub fn city_hand_bonus(&self, player: PlayerId) -> usize {
        let Some(figure) = self.player(player).and_then(|p| p.figure) else {
            return 0;
        };
        self.cities
            .iter()
            .filter(|&(&hex, city)| {
                city.is_conquered() && (hex == figure || hex.neighbors().contains(&figure))
            })
            .map(|(_, city)| {
                if city.leader == Some(player) {
                    2
                } else if city.shields_of(player) > 0 {
                    1
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0)
    }

    /// Bonus de limite de main de la pioche : le plus grand des bonus de forteresse et de cité
    /// (Compendium, *End of Turn* S8 ; spec 062). Le seuil d'assommé n'en tient pas compte.
    pub fn hand_bonus(&self, player: PlayerId) -> usize {
        self.keep_hand_bonus(player)
            .max(self.city_hand_bonus(player))
    }

    /// Jetons de commandement du jeton de niveau (MKUE p. 3, *Unit Area*).
    pub fn command_tokens(&self, player: PlayerId) -> u8 {
        level_token(self.level(player)).command_tokens
    }

    /// Le niveau qui sera atteint à la fin du tour, s'il change (MKUE p. 9, 5.b).
    pub fn pending_level(&self, player: PlayerId) -> Option<u8> {
        self.player(player)
            .map(PlayerState::level_by_fame)
            .filter(|&level| level > self.level(player))
    }

    /// Niveaux gagnés à la dernière fin de tour, dans l'ordre (spec 035).
    pub fn levels_gained(&self, player: PlayerId) -> &[u8] {
        self.player(player)
            .map_or(&[], |p| p.levels_gained.as_slice())
    }

    /// Vrai si une provocation a terminé le déplacement de ce tour (spec 030).
    pub fn movement_ended(&self, player: PlayerId) -> bool {
        self.player(player).is_some_and(|p| p.movement_ended)
    }

    /// Le combat du tour, engagé ou fini ; public (spec 031).
    pub fn combat(&self, player: PlayerId) -> Option<&Combat> {
        self.player(player).and_then(|p| p.combat.as_ref())
    }

    /// Cases des ennemis provoqués ou défiés ce tour, à combattre (specs 030 et 031).
    pub fn enemies_to_fight(&self, player: PlayerId) -> &[Hex] {
        self.player(player)
            .map_or(&[], |p| p.enemies_to_fight.as_slice())
    }

    /// Vrai si le héros peut encore se reposer ce tour : il ne s'est ni déplacé, ni n'a exploré,
    /// défié, combattu ou interagi (MKUE p. 6, 7 ; specs 034 et 040) ; public.
    pub fn can_rest(&self, player: PlayerId) -> bool {
        self.player(player).is_some_and(|p| !p.moved_or_fought)
    }

    /// Vrai si l'action du tour n'est plus possible, ou pas encore (règle du moteur) ; public
    /// (issue privée 141).
    pub fn turn_action_taken(&self, player: PlayerId) -> bool {
        self.player(player)
            .is_some_and(crate::engine::turn_action_taken)
    }

    pub fn discard_pile(&self, player: PlayerId) -> &[CardId] {
        self.player(player)
            .map_or(&[], |p| p.discard_pile.as_slice())
    }

    /// Pioche commune, le dessus en premier : information cachée, jamais dans une vue (spec 050).
    pub fn common_deck(&self, deck: CommonDeck) -> &[CommonCard] {
        self.common_decks.get(&deck).map_or(&[], Vec::as_slice)
    }

    /// Offre, la première place en tête ; publique (spec 050).
    pub fn offer(&self, offer: Offer) -> &[CommonCard] {
        self.offers.get(&offer).map_or(&[], Vec::as_slice)
    }

    /// Réputation du héros, publique (spec 040).
    pub fn reputation(&self, player: PlayerId) -> Reputation {
        self.player(player)
            .map_or_else(Reputation::default, |p| p.reputation)
    }

    /// Bonus (ou malus) d'influence de la réputation à l'interaction ; `None` sur X (spec 040).
    pub fn reputation_bonus(&self, player: PlayerId) -> Option<i8> {
        reputation_bonus(self.reputation(player))
    }

    /// L'interaction en cours ce tour, publique (spec 040).
    pub fn interaction(&self, player: PlayerId) -> Option<Interaction> {
        self.player(player).and_then(|p| p.interaction)
    }

    /// Zone des unités, dans l'ordre du recrutement ; publique (spec 042).
    pub fn units(&self, player: PlayerId) -> &[OwnedUnit] {
        self.player(player).map_or(&[], |p| p.units.as_slice())
    }

    /// L'unité sous le jeton de Loyauté sans faille (spec 058), publique.
    pub fn bonds_unit(&self, player: PlayerId) -> Option<crate::UnitCard> {
        self.player(player).and_then(|p| p.bonds_unit)
    }

    /// Points de soin du tour, publics (spec 042, Q1).
    pub fn healing_points(&self, player: PlayerId) -> u32 {
        self.player(player).map_or(0, |p| p.healing)
    }

    /// Jetons Bouclier du plateau de jeu, par case ; publics (spec 041).
    pub fn shields(&self) -> &BTreeMap<Hex, PlayerId> {
        &self.shields
    }

    /// Les monastères incendiés : les cases de monastère qui portent un Bouclier (spec 041).
    pub fn burned_monasteries(&self) -> Vec<Hex> {
        self.shields
            .keys()
            .copied()
            .filter(|&hex| self.is_burned_monastery(hex))
            .collect()
    }

    /// Vrai si cette case est un monastère incendié (spec 041).
    pub fn is_burned_monastery(&self, hex: Hex) -> bool {
        self.shields.contains_key(&hex)
            && self
                .map
                .space(hex)
                .is_some_and(|space| space.feature == Some(Feature::Site(Site::Monastery)))
    }

    /// Les sites fortifiés conquis : forteresses et tours de mage qui portent un Bouclier
    /// (spec 043), puis les cités conquises (spec 062).
    pub fn conquered_sites(&self) -> Vec<Hex> {
        let mut sites: Vec<Hex> = self
            .shields
            .keys()
            .copied()
            .filter(|&hex| {
                matches!(
                    self.map.space(hex).and_then(|space| space.feature),
                    Some(Feature::Site(Site::Keep | Site::MageTower))
                )
            })
            .collect();
        sites.extend(
            self.cities
                .iter()
                .filter(|(_, city)| city.is_conquered())
                .map(|(&hex, _)| hex),
        );
        sites
    }

    /// Vrai si cette case est une forteresse ou une tour de mage conquise (spec 043).
    pub fn is_conquered(&self, hex: Hex) -> bool {
        self.conquered_sites().contains(&hex)
    }

    /// Le monastère incendié ce tour, dont le combat exclut les unités (spec 041).
    pub fn burning(&self, player: PlayerId) -> Option<Hex> {
        self.player(player).and_then(|p| p.burning)
    }

    /// Récompenses artefact gagnées ce tour, prises à la fin du tour (spec 041).
    pub fn artifact_rewards(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.artifact_rewards)
    }

    /// Artefacts tirés en attente du choix, dans l'ordre du tirage ; publics (spec 041).
    pub fn drawn_artifacts(&self, player: PlayerId) -> &[CardId] {
        self.player(player)
            .map_or(&[], |p| p.drawn_artifacts.as_slice())
    }

    /// Récompenses sort gagnées ce tour, prises à la fin du tour (spec 043).
    pub fn spell_rewards(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.spell_rewards)
    }

    /// Choix de sort en attente à la fin du tour (spec 043).
    pub fn spell_choices(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.spell_choices)
    }

    /// L'assaut du tour, public (spec 043).
    pub fn assault(&self, player: PlayerId) -> Option<Assault> {
        self.player(player).and_then(|p| p.assault)
    }

    /// Le moment de la journée pour le combat du joueur (spec 044) : la Nuit pendant un combat de
    /// donjon ou de tombeau (règles de Nuit), sinon celui de la manche.
    pub fn combat_time_of_day(&self, player: PlayerId) -> TimeOfDay {
        self.player(player)
            .map_or(self.time_of_day, |p| p.combat_time(self.time_of_day))
    }

    /// Les cristaux aléatoires gagnés ce tour, lancés à la fin du tour, publics (spec 044).
    pub fn crystal_rewards(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.crystal_rewards)
    }

    /// Les choix de couleur de cristal en attente, publics (spec 044), Poche de mana compris (spec
    /// 055).
    pub fn crystal_choices(&self, player: PlayerId) -> u8 {
        self.player(player)
            .map_or(0, |p| p.crystal_choices.saturating_add(p.pouch_choices))
    }

    /// Le dé de récompense du donjon conquis ce tour, public (spec 044, C45).
    pub fn reward_die(&self, player: PlayerId) -> Option<ManaColor> {
        self.player(player).and_then(|p| p.reward_die)
    }

    /// Le site d'aventure où le héros est entré ce tour, public (spec 044).
    pub fn adventure(&self, player: PlayerId) -> Option<Hex> {
        self.player(player).and_then(|p| p.adventure)
    }

    /// Choix d'une action avancée en attente du joueur (spec 050).
    pub fn advanced_action_choices(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.advanced_action_choices)
    }

    /// Pile des jetons de ruines, le dessus en premier : information cachée, jamais dans une vue
    /// (spec 045).
    /// Les compétences gagnées par le joueur, dans l'ordre du gain, et leur état (spec 051).
    pub fn skills(&self, player: PlayerId) -> &[crate::OwnedSkill] {
        self.player(player).map_or(&[], |p| p.skills.as_slice())
    }

    /// Les deux compétences révélées en attente du choix (spec 051).
    pub fn skill_choice(&self, player: PlayerId) -> &[crate::Skill] {
        self.player(player)
            .map_or(&[], |p| p.skill_choice.as_slice())
    }

    /// Après une compétence de l'offre commune, seule la dernière action avancée est prenable.
    pub fn lowest_advanced_action_only(&self, player: PlayerId) -> bool {
        self.player(player)
            .is_some_and(|p| p.lowest_advanced_action_only)
    }

    /// Une Motivation a servi : pas d'autre jusqu'à la fin du tour suivant (spec 051).
    pub fn motivation_blocked(&self, player: PlayerId) -> bool {
        self.player(player).is_some_and(|p| p.motivation_block > 0)
    }

    /// L'offre de compétences communes, dans l'ordre d'arrivée (spec 051).
    pub fn common_skills(&self) -> &[crate::Skill] {
        &self.common_skills
    }

    /// La pile de compétences du joueur, le dessus en premier : information cachée (spec 051).
    pub fn skill_pile(&self, player: PlayerId) -> &[crate::Skill] {
        self.player(player).map_or(&[], |p| p.skill_pile.as_slice())
    }

    pub fn ruin_pile(&self) -> &[RuinToken] {
        &self.ruin_pile
    }

    /// Défausse des jetons de ruines, publique (spec 045).
    pub fn ruin_discard(&self) -> &[RuinToken] {
        &self.ruin_discard
    }

    /// Les jetons de ruines des ruines non conquises, par case (spec 045) : un jeton face cachée
    /// est une information cachée (la vue n'en donne que la case).
    pub fn ruins(&self) -> &BTreeMap<Hex, PlacedRuin> {
        &self.ruins
    }

    /// Les cités révélées, par case ; publiques (spec 062).
    pub fn cities(&self) -> &BTreeMap<Hex, crate::CityState> {
        &self.cities
    }

    /// La cité révélée de cette case (spec 062).
    pub fn city(&self, hex: Hex) -> Option<&crate::CityState> {
        self.cities.get(&hex)
    }

    /// Artefacts à garder parmi ceux tirés en attente du choix (spec 045).
    pub fn artifacts_to_keep(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.artifacts_to_keep)
    }

    /// Récompenses « un cristal de chaque couleur » gagnées ce tour (spec 045).
    pub fn crystal_set_rewards(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.crystal_set_rewards)
    }

    /// Récompenses action avancée gagnées ce tour (spec 045).
    pub fn advanced_action_rewards(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.advanced_action_rewards)
    }

    /// Récompenses unité gagnées ce tour (spec 045).
    pub fn unit_rewards(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.unit_rewards)
    }

    /// Choix d'unité de récompense en attente à la fin du tour (spec 045).
    pub fn unit_choices(&self, player: PlayerId) -> u8 {
        self.player(player).map_or(0, |p| p.unit_choices)
    }

    /// Ruines révélées par le dernier pas ou trajet du joueur (spec 045, C40).
    pub fn revealed_ruins(&self, player: PlayerId) -> &[Hex] {
        self.player(player).map_or(&[], |p| &p.revealed_ruins)
    }
}
