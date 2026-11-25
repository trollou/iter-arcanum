//! Écran de partie : calcule depuis la vue du joueur ce que la fenêtre affiche.
//! `game_screen` assemble l'écran ; chaque sous-module calcule une zone ou un mode.

use arcanum_core::{
    Action, AttackKind, BasicColor, BasicEffect, BoostTarget, CRYSTAL_LIMIT, CardId, CardPlay,
    Combat, CombatEnemy, CombatPhase, CombatRecord, ConvertedPoints, Cost, DamageTarget,
    DieSetting, Effect, Efficiency, Element, EnemyAbility, EnemyAttack, EnemyToken, Hex,
    InProgress, Inventory, ManaColor, ManaPayment, ModifierKind, ModifierSource, PlayedCard,
    PlayerView, Resistance, SidewaysEffect, TerrainChange, TimeOfDay, UnitCard, WoundPile,
    end_turn_allowed, glade_wound_available, level_token,
};
use arcanum_i18n::Catalog;

use crate::{Icon, Shortcut, hero_color, hero_figure_image, icon_value, shortcut_label};

mod adventure;
mod artifacts;
mod board_actions;
mod boost;
mod combat;
mod dummy;
mod effects;
mod end_turn;
mod enemies;
mod gain;
mod hand;
mod interaction;
mod level;
mod mana;
mod meditation;
mod offers;
mod panels;
mod play_area;
mod ritual;
mod ruins;
mod score;
mod set_dice;
mod sites;
mod skills;
mod source;
mod spells;
mod tactics;
mod text;
mod types;
mod units;

pub(crate) use adventure::adventure_reward;
use adventure::*;
pub(crate) use artifacts::artifact_enemy_toggles;
use artifacts::*;
use board_actions::board_action_buttons;
use boost::*;
pub use combat::*;
pub use dummy::dummy_discard_window;
use dummy::*;
use effects::*;
use end_turn::*;
use enemies::provoked_banner;
pub(crate) use gain::choosing_offer_card;
use gain::*;
use hand::*;
use interaction::*;
use level::*;
use mana::*;
use meditation::*;
pub use offers::{OfferCardView, OfferColumn, OffersWindow, offers_window, offers_window_in};
pub use panels::discard_window;
use panels::*;
use play_area::*;
use ritual::*;
use ruins::*;
pub(crate) use ruins::{enemy_groups_text, token_reward_text};
pub use score::ScoreWindow;
pub(crate) use score::reopen_score_button;
use score::{cities_progress, end_message, last_turn_line, last_turn_next, score_window};
use set_dice::*;
use sites::*;
use skills::*;
use source::*;
pub(crate) use spells::cast_enemy_toggles;
use spells::*;
use tactics::*;
pub use text::effect_text;
use text::*;
pub(crate) use text::{card_effect_texts, card_strong_text};
pub use types::*;
use units::*;

/// Ce que la fenêtre affiche, déjà traduit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameScreen {
    pub title: String,
    /// Portrait et nom du héros, en tête des données du joueur (spec 065).
    pub portrait: Option<HeroPortrait>,
    /// « Dernier tour », sous la ligne de la manche, pendant le dernier tour (spec 063).
    pub last_turn: Option<String>,
    /// Bandeau du tour où la cité est révélée (spec 063), ou la dernière conquise (spec 064).
    pub last_turn_next: Option<String>,
    /// « Cités conquises : 1/2 », sous la ligne de la manche (spec 064, C28).
    pub cities_progress: Option<String>,
    /// Un bandeau par niveau gagné à la fin du tour d'avant, jusqu'à la première décision du tour
    /// (spec 035), au-dessus du rappel de nouvelle manche.
    pub level_gained: Vec<String>,
    /// Rappel au tour 1 d'une nouvelle manche, tant qu'aucune carte n'est jouée.
    pub new_round: Option<String>,
    /// Source : ses dés de mana (spec 020).
    pub source: SourceLine,
    /// Inventaire : les cristaux (spec 021).
    pub inventory: InventoryLine,
    /// Points de mouvement et d'influence du tour (spec 013), en icônes, puis le Soin (spec 071, C3).
    pub points: IconLine,
    /// Armure en icône, puis « Limite de main 5 · Jetons de commandement 2 » (spec 035 ; spec 071, C5).
    pub level_stats: IconLine,
    /// « Réputation : +2 · bonus d'interaction +1 », ou « Réputation : X · … » (spec 040).
    pub reputation: String,
    /// « Influence d'interaction : 2 », à côté de l'Influence (Fortune clinquante, spec 058, C18).
    pub interaction_influence: Option<String>,
    /// « Boucliers dans la cité : Influence +3 », sous la réputation, pendant l'interaction dans une
    /// cité conquise (spec 062, C40).
    pub city_shields: Option<String>,
    /// Bouton « Interagir avec les habitants », quand l'interaction est légale (spec 040).
    pub interact: Option<Button>,
    /// Sous le bouton : les points de mouvement et d'influence que l'interaction ferait perdre.
    pub interaction_warning: Option<String>,
    /// « Piller le village (2 cartes, Réputation −1) », au début du tour (spec 041).
    pub plunder: Option<Button>,
    /// En interaction, sous le recrutement : actions avancées du monastère, puis le point de soin
    /// (spec 041).
    pub purchases: Vec<PurchaseButton>,
    /// « Incendier le monastère », quand c'est légal, et les points que l'incendie ferait perdre
    /// (spec 041).
    pub burn: Option<Button>,
    pub burn_warning: Option<String>,
    /// Confirmation de l'incendie : son prix, « Incendier » et « Annuler ».
    pub burn_confirm: Option<AnnouncePrompt>,
    /// « Entrer dans : Donjon », sur un site d'aventure où l'on peut entrer, et les points que
    /// l'entrée ferait perdre (spec 044, C39).
    pub adventure: Option<Button>,
    pub adventure_warning: Option<String>,
    /// Confirmation de l'entrée : ce qui sera combattu, la récompense, « Entrer » et « Annuler »
    /// (spec 044, C40).
    pub adventure_confirm: Option<AnnouncePrompt>,
    /// « Faire une offrande à l'autel », sur un autel ; grisé sans 3 mana (spec 045, C33).
    pub altar: Option<PurchaseButton>,
    /// Sous le bouton : les points que l'offrande ferait perdre, ou « Il faut 3 mana bleu ».
    pub altar_warning: Option<String>,
    /// Fenêtre de l'offrande : le choix du paiement (spec 045, C34).
    pub altar_window: Option<AltarWindow>,
    /// « Récompense en fin de tour : un artefact, une action avancée », après la conquête d'une
    /// ruine, à la place des lignes séparées (spec 045, C37).
    pub ruin_reward_pending: Option<String>,
    /// « Ruines révélées : le trajet s'arrête », jusqu'à l'action suivante (spec 045, C40).
    pub revealed_ruins: Option<String>,
    /// « Dé de récompense : vert → un artefact », après la conquête d'un donjon, jusqu'à la fin du
    /// tour (spec 044, C45).
    pub reward_die: Option<String>,
    /// « Récompense en fin de tour : 2 cristal(aux) aléatoire(s) » (spec 044, C45).
    pub crystal_reward_pending: Option<String>,
    /// Fenêtre du choix de la couleur du cristal, après un dé or (spec 044, C46).
    pub crystal_choice: Option<CrystalChoice>,
    /// Fenêtre de fin de manche : les bannières à reprendre (spec 055, C9).
    pub banner_reclaim: Option<ReclaimWindow>,
    /// Fenêtre du choix en attente de Rituel de sang renforcé (spec 056, C4, C5).
    pub blood_ritual: Option<TokenChoice>,
    /// Spec 057 : les cartes de Méditation qui attendent leur place.
    pub meditation: Option<MeditationWindow>,
    /// « Clairière magique : +1 jeton de mana or », au début du tour (spec 044, C48).
    pub glade_token: Option<String>,
    /// « Récompense en fin de tour : 1 artefact(s) », sous les points (spec 041).
    pub reward_pending: Option<String>,
    /// Fenêtre du choix de l'artefact, à la fin du tour (spec 041).
    pub artifact_choice: Option<ArtifactChoice>,
    /// « Récompense en fin de tour : 1 sort(s) », sous les points (spec 043, C45).
    pub spell_reward_pending: Option<String>,
    /// Fenêtre « Récompense : prenez un sort », à la fin du tour (spec 043, C43).
    pub spell_choice: Option<ArtifactChoice>,
    /// « Assaut repoussé : retour sur la case de départ », après une retraite, jusqu'à la fin du
    /// tour (spec 043, C40).
    pub withdrawal: Option<String>,
    /// « Limite de main : 6 (+1 forteresse) », près d'une forteresse possédée (spec 043, C44), ou
    /// « Limite de main : 7 (+2 cité) » près d'une cité conquise (spec 062, C41).
    pub keep_hand_limit: Option<String>,
    /// « Niveau 3 à la fin du tour », quand une ligne de la piste de gloire est franchie (spec 035).
    pub fame_pending: Option<String>,
    pub hand_title: String,
    pub hand: Vec<HandCard>,
    /// Présent quand la main est vide.
    pub hand_empty: Option<String>,
    /// « Jouer de côté » : infobulle de la carte couchée, en tête de l'encart de côté (spec 071).
    pub sideways_title: String,
    /// « Renforcé », titre du bloc renforcé des cartes (spec 073).
    pub strong_title: String,
    pub discard_prompt: Option<DiscardPrompt>,
    /// Bandeau du mode choix du mana (spec 022).
    pub mana_prompt: Option<ManaPrompt>,
    /// Bandeau du mode « mettre un dé sur une couleur » (spec 025).
    pub set_dice_prompt: Option<SetDicePrompt>,
    /// Bandeau du mode Concentration, pendant le choix de la carte à renforcer (spec 025).
    pub boost_prompt: Option<DiscardPrompt>,
    /// Bandeau du mode fin du tour.
    pub end_turn_prompt: Option<EndTurnPrompt>,
    pub play_area_title: String,
    /// Jetons de mana, présents s'il y en a (spec 025).
    pub tokens: Option<TokenLine>,
    pub play_area: Vec<PlayedLine>,
    /// Présent quand aucune carte n'est jouée.
    pub play_area_empty: Option<String>,
    /// « Effets en cours », sous les cartes jouées, s'il y en a (spec 052, C4).
    pub modifiers: Option<ModifierList>,
    pub deck: String,
    /// Spec 057, C27 : les cartes écartées par Distorsion temporelle jusqu'à la fin de la manche,
    /// sous le bouton de la défausse.
    pub set_aside: Option<String>,
    /// Fin de partie ou arrêt sur erreur.
    pub message: Option<String>,
    /// Fenêtre de fin avec le décompte final (spec 063) ; la session la retire une fois fermée.
    pub score_window: Option<ScoreWindow>,
    /// Bouton « Score final » qui rouvre la fenêtre de fin fermée ; rempli par la session.
    pub reopen_score: Option<Button>,
    /// En interaction : « Recruter : Gardes d'Utem (5) », un par unité de l'offre recrutable dans
    /// ce lieu (spec 042), grisé quand l'Influence manque (issue privée 341).
    pub recruit: Vec<PurchaseButton>,
    /// Bandeau du choix de l'unité à renvoyer (spec 042).
    pub disband_prompt: Option<DiscardPrompt>,
    /// La section « Unités » du panneau (spec 042).
    pub units: UnitsSection,
    /// Spec 051 : la section Compétences, avec au moins une compétence gagnée.
    pub skills: Option<SkillsSection>,
    /// Spec 051 : la fenêtre du choix de compétence d'un niveau pair.
    pub skill_choice: Option<SkillChoiceWindow>,
    /// Bandeau du choix d'une capacité d'unité : option, mana ou unité visée (spec 042).
    pub unit_prompt: Option<ManaPrompt>,
    /// « Jeter une Blessure (1) », « Soigner Gardes d'Utem (2) », hors combat (spec 042).
    pub healing: Vec<Button>,
    /// « Niveau 3 », sous le nom du héros (spec 073, C23).
    pub player_level: String,
    /// L'icône Gloire et « 2 / 15 », ou « 2 » seul au niveau 10 (C23).
    pub player_fame: crate::IconValue,
    /// « Défausse : 2 carte(s) », qui ouvre la fenêtre de la défausse ; grisé sans carte (C24).
    pub discard_button: PurchaseButton,
    /// Boutons d'action posés sur les cases du plateau (spec 075, C24 à C29).
    pub board_actions: Vec<BoardActionButton>,
    /// Arrivées d'un déplacement par effet payé, en évidence sur le plateau (spec 057, issue
    /// privée 257).
    pub cast_destinations: Vec<CastDestination>,
    /// Pendant une interaction, le menu du site à côté de sa case (spec 075, C30).
    pub site_menu: Option<SiteMenu>,
    /// Case où s'attachent les confirmations de l'incendie et de l'entrée (spec 075, C31).
    pub site_prompt_hex: Option<Hex>,
    /// « Ce tour » (C28).
    pub this_turn_title: String,
    /// Les faits du tour, dans l'ordre (C28).
    pub turn_facts: Vec<String>,
    /// « Rien de marquant », sans fait ni récompense en attente (C28, C29).
    pub nothing_this_turn: Option<String>,
    /// Les boutons du bas du panneau de droite, toujours visibles : « Offres… », « Finir la
    /// manche… » s'il y a lieu, « Finir le tour… » ; « Annuler » et « Finir le tour » en mode fin
    /// du tour (C30 à C34).
    pub bottom_buttons: Vec<PurchaseButton>,
    /// La confirmation de l'annonce de la fin de la manche, dans le bandeau du panneau du bas (C33).
    pub round_end_confirm: Option<AnnouncePrompt>,
    /// Spec 060 : la fenêtre du choix de la tactique, au début de la manche.
    pub tactic_choice: Option<TacticWindow>,
    /// Spec 060 : le bandeau de Faux départ, Vol de mana, Appel du mana ou Méditation nocturne.
    pub tactic_prompt: Option<AnnouncePrompt>,
    /// Spec 060 : la fenêtre de Préparation.
    pub preparation: Option<PreparationWindow>,
    /// Spec 060 : la fenêtre de Provisions ou de Longue Nuit.
    pub tactic_decision: Option<TacticDecision>,
    /// Spec 060 : la tactique du joueur, dans le panneau de droite.
    pub tactic: Option<TacticLine>,
    /// Spec 061 : la section du joueur fantôme, sous celle du joueur (C37).
    pub dummy: Option<DummySection>,
    /// Spec 061 : « Vous jouez en premier » ou « Le joueur fantôme joue en premier » (C39).
    pub round_order: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenPhase<'a> {
    Playing { legal_actions: &'a [Action] },
    Finished,
    Stopped,
}

/// Calculé depuis la vue seule : l'écran ne peut rien montrer de caché.
pub fn game_screen(
    view: &PlayerView,
    phase: ScreenPhase<'_>,
    mode: &ScreenMode,
    catalog: &Catalog,
) -> GameScreen {
    // Spec 023 : sans scénario (partie pas commencée), la manche en cours tient lieu de limite.
    let round_limit = view
        .scenario
        .map_or(view.round, |scenario| u32::from(scenario.rounds()));
    let (legal_actions, message): (&[Action], _) = match phase {
        ScreenPhase::Playing { legal_actions } => (legal_actions, None),
        // Spec 063 : la raison de la fin de partie, quand `GameEnded` est appliqué.
        ScreenPhase::Finished if view.game_end.is_some() => (
            &[],
            view.game_end.map(|reason| end_message(reason, catalog)),
        ),
        // MKUE p. 4, 5.a : la partie s'arrête à la fin de la dernière manche (sauvegarde d'avant
        // la spec 063, sans `GameEnded`).
        ScreenPhase::Finished if view.scenario.is_some() && view.round >= round_limit => {
            (&[], Some(catalog.text("game.last_round")))
        }
        ScreenPhase::Finished => (&[], Some(catalog.text("game.finished"))),
        ScreenPhase::Stopped => (&[], Some(catalog.text("error.game_stopped"))),
    };
    let can_end_turn = legal_actions
        .iter()
        .any(|action| matches!(action, Action::EndTurn { .. }));
    let (choice, ending, mana_choice) = match mode {
        ScreenMode::Normal => (None, None, None),
        ScreenMode::ChoosingDiscard(choice) => (Some((*choice, false)), None, None),
        ScreenMode::ChoosingStrongDiscard(choice) => (Some((*choice, true)), None, None),
        // Sans fin de tour légale (partie finie), le mode est ignoré.
        ScreenMode::EndingTurn(ending) => (None, can_end_turn.then_some(ending), None),
        ScreenMode::ChoosingMana(choice) => (None, None, Some((*choice, true, NO_SET_DICE))),
        ScreenMode::ChoosingBasicMana(choice) => (None, None, Some((*choice, false, NO_SET_DICE))),
        ScreenMode::ChoosingSetDiceMana(choice) => (
            None,
            None,
            Some((
                ManaChoice {
                    card: choice.card,
                    option: choice.option,
                    discard: None,
                },
                true,
                choice.settings,
            )),
        ),
        ScreenMode::SettingDice(_)
        | ScreenMode::PlayingArtifact(_)
        | ScreenMode::AssigningBanner(_)
        | ScreenMode::ReclaimingBanners(_)
        | ScreenMode::ChoosingSkillUnit { .. }
        | ScreenMode::ChoosingSkillTarget(_)
        | ScreenMode::Meditating(_)
        | ScreenMode::Boosting(_)
        | ScreenMode::ConfirmingEndOfRound
        | ScreenMode::ChoosingUnitAbility { .. }
        | ScreenMode::Disbanding(_)
        | ScreenMode::ConfirmingBurn
        | ScreenMode::ConfirmingAdventure
        | ScreenMode::PayingTribute
        | ScreenMode::ChoosingGain(_)
        | ScreenMode::ChoosingDie(_)
        | ScreenMode::ChoosingSpellMana(_)
        | ScreenMode::Casting(_)
        | ScreenMode::ChoosingDestination(_)
        | ScreenMode::ChoosingTactic(_)
        | ScreenMode::PickingTacticCards(_)
        | ScreenMode::PickingTacticDice(_) => (None, None, None),
    };
    // Spec 025 : un choix de dés qui ne mène plus à aucune action légale est ignoré.
    let set_choice = match mode {
        ScreenMode::SettingDice(choice) => Some(*choice),
        _ => None,
    }
    .filter(|choice| !set_dice_candidates(legal_actions, choice).is_empty());
    let boost = match mode {
        ScreenMode::Boosting(choice) => Some(*choice),
        _ => None,
    }
    .and_then(|choice| {
        let candidates = boost_candidates(legal_actions, &choice);
        (!candidates.is_empty()).then(|| {
            let stage = boost_stage(&choice, &candidates);
            (choice, candidates, stage)
        })
    });
    // Spec 056 : le choix d'une carte d'offre et le dé de Tempête de mana, ignorés s'ils ne mènent
    // plus à aucune action légale.
    let gain = active_gain(mode, legal_actions);
    let die_choice = match mode {
        ScreenMode::ChoosingDie(choice) => Some(*choice),
        _ => None,
    }
    .map(|choice| (choice, die_candidates(legal_actions, &choice)))
    .filter(|(_, candidates)| !candidates.is_empty());
    let announce = legal_actions
        .iter()
        .position(|action| *action == Action::AnnounceEndOfRound);
    // Sans carte en main, l'annonce est obligatoire : rien à confirmer (Q2).
    let confirming = matches!(mode, ScreenMode::ConfirmingEndOfRound)
        && announce.is_some()
        && !view.hand.is_empty();
    // Spec 041 : la confirmation de l'incendie fige la main, comme celle de l'annonce.
    let burn = burn_lines(view, legal_actions, mode, catalog);
    // Spec 044 : celle de l'entrée dans un site d'aventure aussi.
    let adventure = adventure_lines(view, legal_actions, mode, catalog);
    // Spec 045 : la fenêtre de l'offrande aussi.
    let altar = altar_lines(view, legal_actions, mode, catalog);
    let frozen = confirming
        || burn.confirm.is_some()
        || adventure.confirm.is_some()
        || altar.window.is_some();
    // Un choix dont la carte n'a plus d'action légale (déjà jouée) est ignoré.
    let choice = choice.filter(|(choice, strong)| {
        legal_actions.iter().any(|action| match action {
            Action::PlayCard {
                card,
                play:
                    CardPlay::Basic {
                        option,
                        discard: Some(_),
                        ..
                    },
            } => !strong && *card == choice.card && *option == choice.option,
            Action::PlayCard {
                card,
                play:
                    CardPlay::Strong {
                        option,
                        discard: Some(_),
                        ..
                    },
            } => *strong && *card == choice.card && *option == choice.option,
            _ => false,
        })
    });
    let mana_choice = mana_choice.filter(|(choice, strong, set_dice)| {
        !payments(legal_actions, &view.source, choice, *strong, *set_dice).is_empty()
            || (*strong && !extra_payments(legal_actions, &view.source, choice).is_empty())
    });
    // Spec 055 : le choix d'un artefact (cartes, ennemis, couleurs, bonus).
    let artifact = active_artifact(mode, legal_actions);
    // Spec 058 : la cible d'une compétence (carte de côté, mana, case, ennemi, couleur, dé).
    let skill_target = skill_targeting(mode, legal_actions);
    let busy_card = choice
        .map(|(choice, _)| choice.card)
        .or(mana_choice.map(|(choice, _, _)| choice.card))
        .or(set_choice.map(|choice| choice.card))
        .or(boost.as_ref().map(|(choice, _, _)| choice.card))
        .or(gain.as_ref().map(|(choice, _, _)| choice.card))
        .or(die_choice.as_ref().map(|(choice, _)| choice.card))
        .or(assigning(mode, legal_actions))
        .or(artifact.as_ref().map(|(choice, _)| choice.card));
    // La carte renforcée avec Concentration est en cours de jeu, elle aussi.
    let boost_target = boost.as_ref().and_then(|(choice, _, _)| choice.target);

    // Spec 060 : Faux départ et Méditation nocturne cochent des cartes de la main.
    let tactic_cards = card_picking(mode, legal_actions);
    let hand: Vec<HandCard> = view
        .hand
        .iter()
        .filter(|&&card| busy_card != Some(card) && boost_target != Some(card))
        .map(|&card| match (ending, mana_choice) {
            (Some(ending), _) => ending_turn_card(card, ending, legal_actions, catalog),
            (None, None) if boost.is_some() => {
                boost_hand_card(card, view, legal_actions, boost.as_ref(), catalog)
            }
            (None, None) if gain.is_some() => match &gain {
                Some(gain) => gain_hand_card(card, view, legal_actions, gain, catalog),
                None => idle_card(card, catalog),
            },
            (None, None) if die_choice.is_some() => idle_card(card, catalog),
            (None, None) if assigning(mode, legal_actions).is_some() => idle_card(card, catalog),
            (None, None) if artifact.is_some() => match &artifact {
                Some(artifact) => artifact_hand_card(card, artifact, catalog),
                None => idle_card(card, catalog),
            },
            (None, Some(_)) => idle_card(card, catalog),
            (None, None) if skill_target.is_some() => match &skill_target {
                Some(targeting) => skill_hand_card(card, targeting, legal_actions, catalog),
                None => idle_card(card, catalog),
            },
            (None, None) if tactic_cards.is_some() => match &tactic_cards {
                Some(picking) => picking_hand_card(card, picking, catalog),
                None => idle_card(card, catalog),
            },
            (None, None) if frozen || set_choice.is_some() => idle_card(card, catalog),
            (None, None) => hand_card(card, view, legal_actions, choice, catalog),
        })
        .collect();
    let end_turn_prompt =
        ending.map(|ending| end_turn_prompt(view, ending, legal_actions, catalog));
    let announce_prompt = announce
        .filter(|_| {
            busy_card.is_none()
                && ending.is_none()
                && burn.confirm.is_none()
                && adventure.confirm.is_none()
                && altar.window.is_none()
        })
        .map(|index| announce_prompt(view, index, confirming, catalog));
    let end_turn =
        (can_end_turn && busy_card.is_none() && ending.is_none() && !frozen).then(|| Button {
            // Main vide : pas de choix à venir, donc pas de points de suspension (issue privée 25).
            label: if view.hand.is_empty() {
                catalog.text("end_turn.confirm")
            } else {
                catalog.text("end_turn.button")
            },
            command: Command::OpenEndTurn,
            shortcut: shortcut_label(Shortcut::EndTurn, catalog),
        });
    let boost_prompts = boost
        .as_ref()
        .map(|boost| boost_prompts(view, legal_actions, boost, catalog));
    let discard_prompt = choice.map(|(choice, strong)| DiscardPrompt {
        text: catalog.format(
            // Spec 056 : Décomposition détruit la carte choisie.
            if destroys(choice.card, strong) {
                "discard_choice.destroy_prompt"
            } else {
                "discard_choice.prompt"
            },
            &[
                ("card", &card_name(choice.card, catalog)),
                (
                    "effect",
                    &option_effect_of(choice.card, choice.option, strong)
                        .map(|effect| effect_text(effect, catalog))
                        .unwrap_or_default(),
                ),
            ],
        ),
        cancel: Button {
            label: catalog.text("discard_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    });
    let mana_prompt = mana_choice.map(|(choice, strong, set_dice)| {
        // Spec 053 : une option renforcée qui coûte un mana en plus fait choisir les deux.
        let extra = extra_payments(legal_actions, &view.source, &choice);
        if strong && !extra.is_empty() {
            return extra_mana_prompt(view, &choice, extra, catalog);
        }
        let found = payments(legal_actions, &view.source, &choice, strong, set_dice);
        mana_prompt(view, &choice, strong, found, catalog)
    });
    let set_dice_prompt = set_choice
        .map(|choice| set_dice_prompt(view, legal_actions, &choice, catalog))
        .or_else(|| {
            die_choice
                .as_ref()
                .map(|(choice, _)| die_prompt(choice, catalog))
        });
    let (discard_prompt, mana_prompt) = match &gain {
        Some(gain) => (
            gain_prompt(gain, catalog),
            gain_mana_prompt(view, gain, catalog),
        ),
        None => (discard_prompt, mana_prompt),
    };
    let (discard_prompt, mana_prompt, set_dice_prompt, boost_prompt, boost_picks) =
        match boost_prompts {
            Some(prompts) => (
                prompts.discard,
                prompts.mana,
                prompts.set_dice,
                prompts.target,
                prompts.picks,
            ),
            None => (
                discard_prompt,
                mana_prompt,
                set_dice_prompt,
                None,
                Vec::new(),
            ),
        };
    let mut picks: Vec<(u8, Command)> = set_choice
        .filter(|choice| choice.die.is_none())
        .map(|choice| {
            let mut dice: Vec<u8> = set_dice_candidates(legal_actions, &choice)
                .iter()
                .flat_map(|(_, set_dice, _)| set_dice.iter().flatten().map(|setting| setting.die))
                .filter(|die| {
                    !choice
                        .settings
                        .iter()
                        .flatten()
                        .any(|setting| setting.die == *die)
                })
                .collect();
            dice.sort_unstable();
            dice.dedup();
            dice.into_iter()
                .map(|die| (die, Command::PickSetDie(die)))
                .collect()
        })
        .unwrap_or_default();
    picks.extend(boost_picks);
    if let Some((_, candidates)) = &die_choice {
        picks.extend(candidates.iter().copied());
    }
    picks.extend(skill_die_picks(skill_target.as_ref(), legal_actions));
    // Spec 060 : le dé de Vol de mana, les dés d'Appel du mana.
    let (tactic_picks, checked_dice) = tactic_die_picks(mode, legal_actions);
    picks.extend(tactic_picks);
    let (interact, interaction_warning) = if matches!(mode, ScreenMode::Normal) {
        interact_button(view, legal_actions, catalog)
    } else {
        (None, None)
    };
    let recruit = if matches!(mode, ScreenMode::Normal) {
        recruit_buttons(view, legal_actions, catalog)
    } else {
        Vec::new()
    };
    // Spec 054, issue privée 246 : les sorts lancés depuis l'offre, qui y restent, après les cartes jouées.
    let play_area: Vec<PlayedLine> = view
        .play_area
        .iter()
        .chain(offer_casts(view))
        .filter_map(|played| played_line(played, view, catalog))
        .collect();

    let bottom = bottom_buttons(
        view,
        BottomState {
            mode,
            end_turn: end_turn.as_ref(),
            end_turn_prompt: end_turn_prompt.as_ref(),
            announce: announce_prompt.as_ref().filter(|_| !confirming),
            game_over: message.is_some(),
        },
        catalog,
    );
    let round_end_confirm = announce_prompt.clone().filter(|_| confirming);
    let mut screen = GameScreen {
        title: catalog.text("app.title"),
        // Spec 075 : la figurine debout du héros, comme sur le plateau (retour de Mathieu).
        portrait: view.hero.map(|hero| HeroPortrait {
            image: hero_figure_image(hero),
            fallback: hero_color(hero),
            name: catalog.text(hero.label_key()),
        }),
        last_turn: last_turn_line(view, catalog),
        last_turn_next: last_turn_next(view, catalog),
        cities_progress: cities_progress(view, catalog),
        level_gained: level_gained_banners(view, message.is_none(), catalog),
        new_round: (message.is_none()
            && view.round > 1
            && view.turn == 1
            && !view.extra_turn
            && view.play_area.is_empty())
        .then(|| catalog.text("round.new")),
        source: SourceLine {
            keep: keep_die_button(legal_actions, catalog),
            ..source_line(view, &picks, &checked_dice, catalog)
        },
        inventory: inventory_line(view.inventory, catalog),
        points: IconLine {
            icons: vec![
                icon_value(Icon::Move, view.move_points.to_string(), catalog),
                icon_value(Icon::Influence, with_minus(view.influence_points), catalog),
            ],
            text: healing_points(view, catalog).unwrap_or_default(),
        },
        reputation: reputation_line(view, catalog),
        city_shields: city_shields_line(view, catalog),
        interaction_influence: (view.interaction_influence > 0).then(|| {
            catalog.format(
                "skill.interaction_influence",
                &[("amount", &view.interaction_influence.to_string())],
            )
        }),
        interact,
        interaction_warning,
        plunder: if matches!(mode, ScreenMode::Normal) {
            plunder_button(legal_actions, catalog)
        } else {
            None
        },
        purchases: if matches!(mode, ScreenMode::Normal) {
            purchase_buttons(view, legal_actions, catalog)
        } else {
            Vec::new()
        },
        burn: burn.button,
        burn_warning: burn.warning,
        burn_confirm: burn.confirm,
        adventure: adventure.button,
        adventure_warning: adventure.warning,
        adventure_confirm: adventure.confirm,
        altar: altar.button,
        altar_warning: altar.warning,
        altar_window: altar.window,
        ruin_reward_pending: ruin_reward_pending(view, catalog),
        revealed_ruins: revealed_ruins_line(view, catalog),
        reward_die: reward_die_line(view, catalog),
        crystal_reward_pending: crystal_reward_pending(view, catalog),
        crystal_choice: crystal_choice(view, legal_actions, catalog),
        banner_reclaim: reclaim_window(view, legal_actions, mode, catalog),
        blood_ritual: blood_ritual_window(view, legal_actions, catalog),
        meditation: meditation_window(view, legal_actions, mode, catalog),
        glade_token: glade_token(view, message.is_none(), catalog),
        // Spec 045 : après une ruine conquise, une seule ligne pour toute la récompense du jeton.
        reward_pending: reward_pending(view, catalog).filter(|_| !replaced_by_ruin_rewards(view)),
        artifact_choice: artifact_choice(view, legal_actions, catalog),
        spell_reward_pending: spell_reward_pending(view, catalog)
            .filter(|_| !replaced_by_ruin_rewards(view)),
        spell_choice: spell_choice(view, legal_actions, catalog),
        withdrawal: withdrawal(view, catalog),
        keep_hand_limit: keep_hand_limit(view, catalog),
        level_stats: level_stats(view, catalog),
        fame_pending: view
            .pending_level
            .map(|level| catalog.format("fame.pending", &[("level", &level.to_string())])),
        hand_title: catalog.text("hand.title"),
        hand_empty: hand.is_empty().then(|| catalog.text("hand.empty")),
        hand,
        sideways_title: catalog.text("play.sideways"),
        strong_title: catalog.text("panel.strong"),
        discard_prompt: discard_prompt
            .or_else(|| skill_card_prompt(skill_target.as_ref(), catalog)),
        // Spec 043 : le choix du mana d'un sort acheté s'affiche comme celui d'un effet renforcé.
        mana_prompt: mana_prompt
            .or_else(|| spell_mana_prompt(view, legal_actions, mode, catalog))
            // Spec 054 : le mana et l'ennemi visé d'un sort lancé.
            .or_else(|| cast_prompt(view, legal_actions, mode, catalog))
            // Spec 055 : les choix d'un artefact.
            .or_else(|| artifact_prompt(legal_actions, mode, catalog))
            // Spec 058 : le mana ou la cible d'une compétence.
            .or_else(|| skill_target_prompt(view, skill_target.as_ref(), legal_actions, catalog)),
        set_dice_prompt,
        boost_prompt,
        end_turn_prompt,
        play_area_title: catalog.text("play_area.title"),
        tokens: token_line(view, catalog),
        play_area_empty: play_area
            .is_empty()
            .then(|| catalog.text("play_area.empty")),
        play_area,
        modifiers: modifier_list(view, catalog),
        deck: catalog.format("deck.count", &[("count", &view.deck_size.to_string())]),
        set_aside: set_aside_line(view, catalog),
        message,
        score_window: score_window(view, catalog),
        reopen_score: None,
        recruit,
        disband_prompt: disband_prompt(mode, legal_actions, catalog)
            .or_else(|| assign_prompt(mode, legal_actions, catalog))
            // Spec 051, C22 : l'unité visée par Inspiration.
            .or_else(|| skill_unit_prompt(mode, legal_actions, catalog)),
        units: units_section(view, legal_actions, mode, catalog),
        skills: skills_section(view, legal_actions, catalog),
        skill_choice: skill_choice_window(view, legal_actions, catalog),
        unit_prompt: unit_prompt(view, legal_actions, mode, catalog),
        healing: healing_buttons(legal_actions, catalog),
        player_level: player_level(view, catalog),
        player_fame: player_fame(view, catalog),
        discard_button: discard_button(view, catalog),
        board_actions: Vec::new(),
        cast_destinations: cast_destinations(view, legal_actions, mode, catalog)
            .into_iter()
            .chain(skill_destinations(skill_target.as_ref(), legal_actions))
            .collect(),
        // Spec 075, C30 : ouvert ; la session le replie à la demande.
        site_menu: interaction_header(view, catalog)
            .zip(view.figure)
            .map(|(title, hex)| SiteMenu {
                hex,
                title,
                open: true,
            }),
        site_prompt_hex: None,
        this_turn_title: catalog.text("panel.this_turn"),
        turn_facts: turn_fact_lines(view, catalog),
        nothing_this_turn: None,
        bottom_buttons: bottom,
        round_end_confirm,
        tactic_choice: tactic_window(view, legal_actions, mode, catalog),
        tactic_prompt: tactic_prompt(view, legal_actions, mode, catalog),
        preparation: preparation_window(view, legal_actions, catalog),
        tactic_decision: tactic_decision(view, legal_actions, catalog),
        tactic: tactic_line(view, legal_actions, catalog),
        dummy: dummy_section(view, catalog),
        round_order: round_order_line(view, catalog),
    };
    // Spec 075, C31 : les confirmations s'ouvrent à côté de la case du site.
    if screen.burn_confirm.is_some() || screen.adventure_confirm.is_some() {
        screen.site_prompt_hex = view.figure;
    }
    // Spec 075, C33 : hors des modes seulement, comme les boutons de la section Ici.
    if matches!(mode, ScreenMode::Normal) {
        screen.board_actions = board_action_buttons(view, legal_actions, &screen, catalog);
    }
    screen.nothing_this_turn = (screen.turn_facts.is_empty() && !screen.has_pending_lines())
        .then(|| catalog.text("panel.nothing_this_turn"));
    screen
}

/// Les cartes écartées par Distorsion temporelle jusqu'à la fin de la manche (spec 057, C27).
fn set_aside_line(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    if view.set_aside.is_empty() {
        return None;
    }
    let names: Vec<String> = view
        .set_aside
        .iter()
        .map(|&card| card_name(card, catalog))
        .collect();
    Some(catalog.format("set_aside.title", &[("cards", &names.join(", "))]))
}
