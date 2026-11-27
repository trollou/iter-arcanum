//! Ce que l'écran affiche et les modes de saisie : des données déjà traduites, sans calcul.

use super::*;

/// Ce que fait un bouton de l'écran de partie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Choisir l'action d'index donné parmi les actions légales.
    Play(usize),
    /// Passer en mode choix de la carte à défausser pour cette option d'effet de base (spec 013).
    ChooseDiscard {
        card: CardId,
        option: u8,
    },
    /// Même chose pour une option d'effet renforcé (spec 022).
    ChooseStrongDiscard {
        card: CardId,
        option: u8,
    },
    /// Passer en mode choix du mana pour cette option d'effet renforcé (spec 022).
    ChooseMana {
        card: CardId,
        option: u8,
        discard: Option<CardId>,
    },
    /// Même chose pour une option d'effet de base qui coûte du mana (spec 025).
    ChooseBasicMana {
        card: CardId,
        option: u8,
    },
    /// En mode fin du tour, choisir la carte à défausser pour reprendre Euphorie cristalline,
    /// ou aucune (spec 025).
    ChooseReclaim(Option<CardId>),
    /// Passer en mode « mettre un dé sur une couleur » pour cette option d'effet renforcé
    /// (Source de mana, Attraction de mana : spec 025).
    ChooseSetDice {
        card: CardId,
        option: u8,
    },
    /// Dans ce mode, choisir le dé de la Source à cette position…
    PickSetDie(u8),
    /// … puis sa couleur, quand il reste un dé à choisir.
    PickSetColor(BasicColor),
    /// Dés choisis, plusieurs paiements possibles : passer au choix du mana.
    ChooseSetDiceMana {
        card: CardId,
        option: u8,
        set_dice: [Option<DieSetting>; 2],
    },
    /// Passer à cette étape du mode Concentration renforcée (spec 025), ou Effet maximum (spec
    /// 056).
    ChooseBoost(BoostChoice),
    /// Passer à cette étape du choix d'une carte d'offre (Entraînement, Don pour la magie ; spec
    /// 056) : la carte à détruire, la carte prise, puis le mana.
    ChooseGain(GainChoice),
    /// Passer au choix du mana et de l'ennemi visé d'un sort (spec 054).
    ChooseCast(CastChoice),
    /// Paiement choisi d'un déplacement par effet : l'arrivée se choisit sur le plateau (spec 057,
    /// issue privée 257).
    ChooseDestination(DestinationChoice),
    /// Choisir, ou retirer, une carte de Méditation : l'ordre des clics est celui du deck (spec
    /// 057).
    PickMeditationCard(CardId),
    /// Passer au choix du dé de la Source de cette option (Tempête de mana, spec 056).
    ChooseDie {
        card: CardId,
        option: u8,
    },
    /// Quitter le mode choix ou le mode fin du tour sans rien jouer.
    CancelChoice,
    /// Passer en mode fin du tour (spec 016).
    OpenEndTurn,
    /// Cocher ou décocher une carte à défausser avant de repiocher.
    ToggleEndTurnDiscard(CardId),
    /// Finir le tour avec les cartes cochées.
    ConfirmEndTurn,
    /// Garder ou non Temporisation à la fin du tour (spec 052, C39).
    ToggleSteadyTempo,
    /// Demander la confirmation de l'annonce de la fin de la manche (spec 023).
    OpenAnnounce,
    /// Revenir en arrière : reprendre la dernière décision (spec 024).
    Undo,
    /// Écrire une nouvelle sauvegarde (spec 024).
    Save,
    /// Ouvrir la fenêtre de chargement (spec 024).
    OpenLoad,
    /// Charger la sauvegarde de cette ligne de la fenêtre de chargement.
    Load(usize),
    /// Fermer la fenêtre de chargement.
    CloseLoad,
    /// Cocher ou décocher un ennemi pour la prochaine attaque (spec 031).
    ToggleTarget(EnemyToken),
    /// Cocher ou décocher un ennemi visé par le sort en cours (Ailes de la nuit, issue privée 258).
    ToggleCastEnemy(EnemyToken),
    /// Cocher ou décocher une unité visée par le sort en cours (Renaissance, Flux d'énergie,
    /// issue privée 258).
    ToggleCastUnit(UnitCard),
    /// Passer à cette étape du choix d'un artefact joué (spec 055) : cartes à défausser, ennemis
    /// visés et couleurs cochés jusqu'ici.
    ChooseArtifact(ArtifactPlay),
    /// Choisir l'unité à qui assigner cette bannière de la main (spec 055, C6).
    AssignBanner(CardId),
    /// Fin de manche : les bannières cochées pour être reprises dans le deck (spec 055, C9).
    ChooseReclaimed(arcanum_core::Picks<CardId>),
    /// « Voir le plateau » ou « Retour au combat » (spec 031).
    ToggleCombatBoard,
    /// Le bouton bulle pendant une interaction : replier ou déplier le menu du site (spec 075,
    /// C30).
    ToggleSiteMenu,
    /// « Annuler » une parade insuffisante : revenir en arrière jusqu'avant « Parer » (spec 070).
    CancelBlock,
    /// « Annuler » une attaque insuffisante : revenir en arrière jusqu'avant « Attaquer ».
    CancelAttack,
    /// « Voir le plateau » : fermer la fenêtre de fin (spec 063).
    CloseScore,
    /// « Score final » : rouvrir la fenêtre de fin.
    ReopenScore,
    /// « Nouvelle partie… » : ouvrir la fenêtre de nouvelle partie (spec 065).
    OpenNewGame,
    /// Cocher ce héros dans la fenêtre de nouvelle partie.
    ChooseHero(arcanum_core::Hero),
    /// Cocher ce scénario.
    ChooseScenario(arcanum_core::ScenarioId),
    /// Afficher cette étape de l'écran de nouvelle partie (spec 078) : onglet ou « Suivant ».
    ChooseStep(crate::NewGameStep),
    /// Ouvrir ou refermer le volet « Avancé » (spec 078, C13).
    ToggleAdvanced,
    /// Spec 061 : coche ou décoche le joueur fantôme.
    ToggleDummy,
    /// Spec 061 : choisit le héros du fantôme.
    ChooseDummyHero(arcanum_core::DummyHeroChoice),
    /// « Commencer » : lancer la partie du formulaire.
    StartNewGame,
    /// « Annuler » : fermer la fenêtre et revenir à la partie en cours.
    CancelNewGame,
    /// « Continuer » : reprendre la sauvegarde automatique, au lancement.
    ContinueGame,
    /// Ouvrir ou fermer la fenêtre Offres (spec 050).
    OpenOffers,
    CloseOffers,
    /// Passer au choix de l'unité visée par cette option de compétence (Inspiration, spec 051).
    ChooseSkillUnit {
        skill: arcanum_core::Skill,
        option: u8,
    },
    /// Passer au choix de la cible de cette option de compétence : carte jouée de côté et son
    /// mana, case, ennemi, couleur ou dé (spec 058).
    ChooseSkillTarget(SkillChoice),
    /// Passer au choix de l'option, du mana ou de l'unité visée de cette capacité (spec 042).
    ChooseUnitAbility {
        unit: UnitCard,
        ability: u8,
        /// Le choix réduit à cette option, depuis son bouton d'effet (spec 073, C22).
        option: Option<u8>,
    },
    /// Recruter cette unité, jetons de commandement tous pris : choisir l'unité à renvoyer.
    ChooseDisband(UnitCard),
    /// Demander la confirmation de l'incendie du monastère (spec 041).
    OpenBurn,
    /// Passer au choix du mana qui paie l'achat de ce sort (spec 043, C41).
    ChooseSpellMana(CardId),
    /// Demander la confirmation de l'entrée dans le site d'aventure (spec 044, C40).
    OpenAdventure,
    /// Ouvrir la fenêtre de l'offrande à l'autel (spec 045, C34).
    OpenAltar,
    /// En mode fin du tour sur une clairière, jeter une Blessure de la main, de la défausse, ou
    /// aucune (spec 044, C47).
    ChooseGlade(Option<WoundPile>),
    /// Ouvrir ou fermer la fenêtre de la défausse (spec 073, C35).
    OpenDiscard,
    CloseDiscard,
    /// Spec 061 : ouvre la fenêtre de la défausse du fantôme ; `CloseDiscard` la ferme.
    OpenDummyDiscard,
    /// Commandes de la fenêtre, du Menu (spec 074) : recadrer le plateau, passer en plein écran
    /// ou en sortir, quitter le jeu.
    Recenter,
    ToggleFullscreen,
    Quit,
    /// Boutons de la confirmation de Quitter (spec 074, Q1).
    ConfirmQuit,
    CancelQuit,
    /// Fenêtre des préférences (spec 080) : l'ouvrir (ou la fermer), la fermer.
    OpenPreferences,
    ClosePreferences,
    /// Fenêtre « À propos » (spec #33) : l'ouvrir (ou la fermer), la fermer.
    OpenAbout,
    CloseAbout,
    /// Choisir la langue des préférences : sa place dans la liste des langues disponibles, `None`
    /// pour la langue du système. Une place et non la `Language`, pour que `Command` reste `Copy`.
    SetLanguage(Option<usize>),
    /// Cocher ou décocher « Plein écran au lancement ».
    SetFullscreenAtLaunch(bool),
    /// Spec #5 : cocher ou décocher « Animations ».
    SetAnimations(bool),
    /// Spec 081 : changer un volume pour la session pendant qu'on glisse le curseur, sans écrire
    /// le fichier ; le premier changement d'un glissé demande l'échantillon de ce type de son.
    SetVolume(arcanum_audio::SoundCategory, u8),
    /// Spec 081 : le curseur lâché : écrire le fichier et arrêter l'échantillon.
    SaveVolumes,
    /// Encadrer cette tactique dans la fenêtre de choix (spec 060).
    SelectTactic(arcanum_core::Tactic),
    /// Cocher ou décocher une carte de la main : Faux départ, Méditation nocturne (spec 060).
    ToggleTacticCard(CardId),
    /// Ouvrir le choix des cartes de Méditation nocturne (spec 060).
    OpenMidnightMeditation,
    /// Ouvrir le choix des dés d'Appel du mana (spec 060).
    OpenManaSearch,
    /// Cocher ou décocher un dé de la Source pour Appel du mana (spec 060).
    ToggleTacticDie(u8),
}

/// Fenêtre de la défausse (spec 073, C35) : ses cartes, la dernière défaussée en tête, sans
/// bouton.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscardWindow {
    pub title: String,
    pub cards: Vec<HandCard>,
    pub close: Button,
}

/// Mode de l'écran : un état de l'UI, jamais du moteur, qui ne reçoit que l'action complète.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ScreenMode {
    #[default]
    Normal,
    /// Choix de la carte à défausser (spec 013).
    ChoosingDiscard(DiscardChoice),
    /// Choix des cartes à défausser avant de repiocher (spec 016).
    EndingTurn(EndTurnChoice),
    /// Choix de la carte à défausser pour un effet renforcé (spec 022).
    ChoosingStrongDiscard(DiscardChoice),
    /// Choix du mana qui paie un effet renforcé (spec 022).
    ChoosingMana(ManaChoice),
    /// Choix du mana qui paie une option d'effet de base (spec 025).
    ChoosingBasicMana(ManaChoice),
    /// Choix des dés à mettre sur une couleur, un par un (spec 025).
    SettingDice(SetDiceChoice),
    /// Dés choisis : choix du mana qui paie l'effet renforcé (spec 025).
    ChoosingSetDiceMana(SetDiceChoice),
    /// Concentration renforcée : la carte à renforcer, puis ses choix et le mana (spec 025).
    Boosting(BoostChoice),
    /// Confirmation de l'annonce de la fin de la manche, cartes en main (spec 023).
    ConfirmingEndOfRound,
    /// Choix de l'option, du mana ou de l'unité visée d'une capacité d'unité (spec 042).
    ChoosingUnitAbility {
        unit: UnitCard,
        ability: u8,
        option: Option<u8>,
    },
    /// Choix de l'unité à renvoyer pour recruter celle-ci (spec 042).
    Disbanding(UnitCard),
    /// Confirmation de l'incendie du monastère : le jeton tiré empêche le retour (spec 041).
    ConfirmingBurn,
    /// Choix du mana qui paie l'achat de ce sort, à une tour de mage conquise (spec 043, C41).
    ChoosingSpellMana(CardId),
    /// Confirmation de l'entrée dans un site d'aventure : le jeton tiré empêche le retour
    /// (spec 044, C40).
    ConfirmingAdventure,
    /// Fenêtre de l'offrande à l'autel : le choix du paiement (spec 045, C34).
    PayingTribute,
    /// Entraînement, Don pour la magie : la carte à détruire, la carte d'offre, puis le mana
    /// (spec 056).
    ChoosingGain(GainChoice),
    /// Méditation : les cartes choisies, dans l'ordre où elles iront sur ou sous le deck (spec
    /// 057).
    Meditating(Vec<CardId>),
    /// Tempête de mana : le dé de la Source (spec 056, C13).
    ChoosingDie(DiscardChoice),
    /// Sort lancé : le mana, le noir, le mana en plus et l'ennemi visé (spec 054).
    Casting(CastChoice),
    /// Déplacement par effet payé : l'arrivée, d'un clic sur le plateau (spec 057, issue privée 257).
    ChoosingDestination(DestinationChoice),
    /// Artefact joué : ses choix, cochés un par un (spec 055).
    PlayingArtifact(ArtifactPlay),
    /// Bannière de la main : l'unité qui la portera, d'un clic (spec 055, C6).
    AssigningBanner(CardId),
    /// Fin de manche : les bannières cochées pour être reprises (spec 055, C9).
    ReclaimingBanners(arcanum_core::Picks<CardId>),
    /// Option de compétence choisie : l'unité visée, d'un clic dans la section Unités
    /// (Inspiration, spec 051, C22).
    ChoosingSkillUnit {
        skill: arcanum_core::Skill,
        option: u8,
    },
    /// Option de compétence choisie : sa cible (spec 058) ; pour une carte jouée de côté, la carte
    /// puis le mana de Pouvoir universel.
    ChoosingSkillTarget(SkillChoice),
    /// Spec 060 : la tactique encadrée dans la fenêtre de choix, pas encore prise.
    ChoosingTactic(arcanum_core::Tactic),
    /// Spec 060 : les cartes de la main cochées pour Faux départ ou Méditation nocturne.
    PickingTacticCards(Vec<CardId>),
    /// Spec 060 : les dés de la Source cochés pour Appel du mana.
    PickingTacticDice(Vec<u8>),
}

/// Option de compétence en attente de sa cible (spec 058) : la carte jouée de côté et son effet,
/// une fois choisie (le mana de Pouvoir universel reste à choisir).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillChoice {
    pub skill: arcanum_core::Skill,
    pub option: u8,
    pub card: Option<(CardId, arcanum_core::SidewaysEffect)>,
}

/// Artefact en attente de ses choix (spec 055) : son effet renforcé (détruit) ou de base, l'option,
/// et ce qui est déjà choisi (cartes défaussées, ennemis, couleurs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactPlay {
    pub card: CardId,
    pub strong: bool,
    pub option: u8,
    pub picks: arcanum_core::ArtifactChoices,
}

/// La bannière que porte une unité (spec 055, C42) : son nom, l'infobulle, retournée ou non.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BannerTag {
    pub name: String,
    pub tooltip: String,
    pub flipped: bool,
}

/// Fenêtre de fin de manche (spec 055, C9) : une ligne par bannière assignée, « Garder sur
/// l'unité » ou « Reprendre », puis « Valider ».
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReclaimWindow {
    pub title: String,
    pub banners: Vec<ReclaimLine>,
    pub confirm: Option<Button>,
}

/// Une bannière de la fenêtre de fin de manche : « Bannière de courage (Paysans) », reprise ou non.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReclaimLine {
    pub text: String,
    pub taken: bool,
    pub keep: Button,
    pub take: Button,
}

/// Sort de déplacement par effet dont le paiement est choisi (Voyage souterrain, Attaque
/// souterraine, Ailes du vent) : `payment` est le lancer sans son arrivée (spec 057, issue privée 257).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DestinationChoice {
    pub choice: CastChoice,
    pub payment: CardPlay,
}

/// Arrivée légale d'un déplacement par effet, en évidence sur le plateau : un clic y joue
/// `command` (spec 057, issue privée 257).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastDestination {
    pub hex: Hex,
    pub command: Command,
}

/// Sort en attente de ses choix (spec 054) : l'option, ou toutes celles du sort quand il est
/// lancé depuis l'offre (`None`), puis le mana, le noir, le mana en plus et l'ennemi visé, en un
/// seul bandeau.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastChoice {
    pub card: CardId,
    /// L'effet renforcé (`true`) ou de base, et son option.
    pub option: Option<(bool, u8)>,
    pub from_offer: bool,
    /// Les cibles cochées jusqu'ici (issue privée 258).
    pub picks: CastPicks,
}

/// Les cibles cochées d'un sort de la spec 057 (issue privée 258) : les unités de Renaissance et de
/// Flux d'énergie dans le panneau du bas, les ennemis d'Ailes de la nuit sur l'écran de combat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CastPicks {
    pub units: arcanum_core::Picks<UnitCard>,
    pub enemies: arcanum_core::Picks<EnemyToken>,
}

/// Carte qui prend une carte d'une offre, en attente de ses choix, dans l'ordre : la carte action
/// à détruire (Entraînement), la carte de l'offre, puis le mana de l'effet renforcé (spec 056).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GainChoice {
    pub card: CardId,
    pub option: u8,
    pub strong: bool,
    pub discard: Option<CardId>,
    pub gain: Option<CardId>,
}

/// Effet renforcé en attente du choix du mana : un état de l'écran, jamais du moteur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManaChoice {
    pub card: CardId,
    pub option: u8,
    pub discard: Option<CardId>,
}

/// Source de mana ou Attraction de mana renforcée en attente du choix des dés : les dés déjà
/// mis sur une couleur, et le dé choisi dont on attend la couleur (spec 025).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetDiceChoice {
    pub card: CardId,
    pub option: u8,
    pub settings: [Option<DieSetting>; 2],
    pub die: Option<u8>,
}

/// Concentration renforcée (spec 025) ou Effet maximum (spec 056) en attente de ses choix, dans
/// l'ordre : la carte à renforcer et son option, la carte que celle-ci défausse, ses dés mis sur
/// une couleur (avec le dé choisi dont on attend la couleur), puis le mana qui paie la carte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoostChoice {
    pub card: CardId,
    pub option: u8,
    /// Effet maximum de base (spec 056) : la cible joue son effet de base, sans mana.
    pub basic: bool,
    pub target: Option<CardId>,
    pub target_option: u8,
    pub discard: Option<CardId>,
    pub settings: [Option<DieSetting>; 2],
    pub die: Option<u8>,
}

/// Cartes cochées en mode fin du tour, dans l'ordre où elles l'ont été, et carte défaussée pour
/// reprendre Euphorie cristalline (spec 025).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndTurnChoice {
    pub discard: Vec<CardId>,
    pub reclaim: Option<CardId>,
    /// Sur une clairière, la Blessure à jeter choisie ; `None` tant que rien n'est choisi : le
    /// choix par défaut s'applique (spec 044, C47).
    pub glade: Option<Option<WoundPile>>,
    /// Temporisation gardée sur ou sous le deck ; `None` tant que rien n'est choisi : gardée si
    /// c'est légal (spec 052, Q5).
    pub steady_tempo: Option<bool>,
}

impl EndTurnChoice {
    /// L'action `EndTurn` correspondante : les cartes cochées encore en main, dans l'ordre de
    /// la main, sauf celle de la reprise ; sur une clairière, la Blessure à jeter choisie, ou
    /// celle du choix par défaut (spec 044, C47).
    pub fn action(&self, view: &PlayerView, legal_actions: &[Action]) -> Action {
        // Spec 052, C21 : la clairière d'arrivée de la retraite forcée et ses Blessures comptent.
        let view = &*super::end_turn::after_withdrawal(view);
        let hand = &view.hand;
        let reclaim = self.reclaim.filter(|card| hand.contains(card));
        let discard: Vec<CardId> = hand
            .iter()
            .filter(|card| self.discard.contains(card) && reclaim != Some(**card))
            .copied()
            .collect();
        let glade = glade_choice(view, self.glade, &discard, reclaim);
        Action::EndTurn {
            discard,
            reclaim,
            glade,
            steady_tempo: self.steady_tempo.unwrap_or(true)
                && super::end_turn::steady_tempo_offered(legal_actions),
        }
    }
}

/// Une ligne du panneau qui commence par des valeurs à icône, suivies d'un texte (spec 071) :
/// Mouvement et Influence puis le Soin, Gloire puis le niveau, Armure puis la limite de main.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IconLine {
    pub icons: Vec<crate::IconValue>,
    /// Vide quand il n'y a rien après les icônes.
    pub text: String,
}

impl IconLine {
    /// Le texte de repli, sans fichier d'icône (L1) : « Mouvement 3 · Influence 2 · Soin : 2 ».
    pub fn fallback_text(&self) -> String {
        self.icons
            .iter()
            .map(crate::IconValue::fallback_text)
            .chain((!self.text.is_empty()).then(|| self.text.clone()))
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

/// Bouton : son libellé et ce qu'il fait.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Button {
    pub label: String,
    pub command: Command,
    /// Libellé de sa touche, déjà traduit : « Ctrl+Z », « Entrée » ; `None` sans touche (spec 071).
    pub shortcut: Option<String>,
}

/// Carte en cours de jeu en attendant le choix de la carte à défausser : un état de l'écran,
/// jamais du moteur, qui ne reçoit que l'action complète.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiscardChoice {
    pub card: CardId,
    pub option: u8,
}

/// Une carte de la main, déjà traduite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandCard {
    pub card: CardId,
    pub name: String,
    /// `None` pour une Blessure, qui n'a pas de couleur (spec 031).
    pub color: Option<BasicColor>,
    /// Texte de l'effet de base.
    pub effect: String,
    /// Boutons sous la carte : effet de base, ou « Défausser » en mode choix.
    pub plays: Vec<Button>,
    /// Un bouton-icône par effet de côté légal, dans l'ordre Mouvement, Influence, Attaque,
    /// Parade (spec 071, C14) ; vide en mode choix.
    pub sideways: Vec<crate::IconButton>,
    /// Texte de l'effet renforcé : « Renforcé : Mouvement 4 » (spec 022).
    pub strong_effect: String,
    /// Boutons « Renforcer : … », un par option jouable hors combat ; vide en mode choix.
    pub strong_plays: Vec<StrongButton>,
    /// Options de l'effet de base qui coûtent du mana : « Payer rouge : cristal rouge »,
    /// grisées quand on ne peut pas payer (spec 025) ; vide en mode choix.
    pub paid_plays: Vec<StrongButton>,
    /// Case « Défausser », seulement en mode fin du tour (spec 016).
    pub discard_toggle: Option<DiscardToggle>,
    /// « siège seulement » : ses attaques à distance ne se jouent pas contre le groupe fortifié
    /// visé (spec 032, C21).
    pub combat_note: Option<String>,
    /// Son illustration, dessinée en bande sous le nom si l'image est présente (spec 072, C5 ;
    /// spec 077).
    pub illustration: crate::Illustration,
    /// Spec 077, C1 : l'image occupe la carte, seule sous le nom (une Blessure, sans texte).
    pub art_fills_card: bool,
    /// Spec 073 : un bouton d'effet par option de l'effet de base, grisé quand l'option n'est pas
    /// légale ; vide en mode choix (les `plays` les remplacent) et pour une Blessure.
    pub basic_effects: Vec<EffectButton>,
    /// Le texte de l'effet renforcé, sans « Renforcé : » (le bloc a son titre).
    pub strong_text: String,
    /// Un bouton d'effet par option renforcée, puis un par option payante (spec 025).
    pub strong_effects: Vec<EffectButton>,
    /// Rajeunissement : « Jouer : Rendre disponible … · Paysans », sous les blocs (spec 042).
    pub unit_plays: Vec<Button>,
    /// Spec #11 : le texte complet de la carte, derrière le bouton « i » ; `None` pour une
    /// Blessure et un artefact.
    pub info: Option<CardInfo>,
    /// Spec #11 (Q1) : la carte écrit son texte sur elle (un artefact seulement).
    pub show_text: bool,
    /// Spec #11 : le titre du bloc renforcé, « Renforcé », le nom renforcé d'un sort, ou « si
    /// détruit » pour un artefact.
    pub strong_heading: String,
    /// Spec #11 : les pastilles de ce titre, la couleur de la carte (puis le noir pour un sort).
    pub strong_colors: Vec<ManaColor>,
}

/// Le texte complet d'une carte de la main, montré par son bouton « i » (spec #11).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardInfo {
    /// Le nom de base.
    pub name: String,
    pub basic: String,
    /// « Renforcé », ou le nom renforcé d'un sort.
    pub strong_title: String,
    pub strong: String,
}

/// Le visage d'un bouton d'effet (spec 073) : une icône et sa valeur, un cristal, ou le texte de
/// l'effet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectFace {
    Icon(crate::IconValue),
    /// Jeton ou cristal gagné : le cristal de la couleur, sans valeur ; `label`, son texte de
    /// repli.
    Crystal {
        color: ManaColor,
        label: String,
    },
    Text(String),
}

/// Bouton d'effet d'une carte ou d'une unité (spec 073) : grisé (sans commande) quand le moteur
/// ne rend pas l'option légale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectButton {
    pub face: EffectFace,
    /// « ÷2 » quand l'effet compte à moitié (spec 032).
    pub mark: Option<String>,
    /// Pastille du mana à payer : effet renforcé, option payante, capacité d'unité.
    pub color: Option<ManaColor>,
    pub command: Option<Command>,
    /// « Jouer : Attaque 2 », puis la raison du grisé s'il y en a une.
    pub tooltip: String,
}

impl EffectButton {
    /// Le texte de repli, sans fichier d'icône (spec 071, L1) : « Attaque 2 ».
    pub fn fallback_text(&self) -> String {
        let text = match &self.face {
            EffectFace::Icon(value) => value.fallback_text(),
            EffectFace::Crystal { label, .. } | EffectFace::Text(label) => label.clone(),
        };
        match &self.mark {
            Some(mark) => format!("{text} {mark}"),
            None => text,
        }
    }
}

/// Bouton d'un effet payé en mana : grisé, avec la raison, quand on ne peut pas payer
/// (specs 022 et 025).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrongButton {
    pub label: String,
    /// Couleur du mana à payer : celle de la carte, ou celle de l'option (spec 025).
    pub color: BasicColor,
    pub command: Command,
    pub enabled: bool,
    pub tooltip: Option<String>,
}

/// Bandeau du mode choix du mana (spec 022).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManaPrompt {
    pub text: String,
    /// Un bouton par paiement distinct.
    pub payments: Vec<Button>,
    pub cancel: Button,
}

/// Case à cocher d'une carte en mode fin du tour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscardToggle {
    pub label: String,
    pub checked: bool,
    pub command: Command,
}

/// Bandeau du mode fin du tour (spec 016).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndTurnPrompt {
    pub text: String,
    /// Nombre de cartes qui seront piochées.
    pub draw: String,
    /// Présent quand rien n'a été joué ni coché (MKUE p. 6, 5.a).
    pub must_discard: Option<String>,
    pub confirm: Button,
    pub confirm_enabled: bool,
    pub cancel: Button,
    /// Menu de reprise d'Euphorie cristalline, quand elle a été jouée (spec 025).
    pub reclaim: Option<ReclaimMenu>,
    /// « Pas de repos… » : Blessures en main, repos interdit (spec 034).
    pub no_rest: Option<String>,
    /// « Clairière magique : jeter une Blessure » et ses trois choix (spec 044, C47).
    pub glade: Option<GladeMenu>,
    /// « Retraite forcée : 1 case(s), 1 Blessure(s) en main », sur une case non sécurisée (spec
    /// 052, C26).
    pub forced_withdrawal: Option<String>,
    /// « Garder Temporisation sous le deck », cochée d'office, quand garder est légal (spec 052,
    /// C39, C40 ; Q5).
    pub steady_tempo: Option<DiscardToggle>,
}

/// La ligne de la clairière en mode fin du tour : de la main, de la défausse, ne pas jeter
/// (spec 044, C47).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GladeMenu {
    pub label: String,
    pub options: Vec<GladeOption>,
}

/// Un choix à choix unique de la clairière : grisé (sans Blessure à cet endroit), coché ou non.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GladeOption {
    pub label: String,
    pub command: Command,
    pub enabled: bool,
    pub selected: bool,
}

/// Fenêtre « Récompense : choisissez la couleur du cristal » (spec 044, C46) : un bouton par
/// couleur de base, grisé si l'inventaire en est plein.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrystalChoice {
    pub title: String,
    /// Rouge, bleu, blanc, vert.
    pub colors: Vec<CrystalButton>,
}

/// Bouton d'une couleur du choix du cristal : grisé (sans commande) si l'inventaire en est plein.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrystalButton {
    pub color: BasicColor,
    pub label: String,
    pub command: Option<Command>,
    /// « 1 » à « 4 », dans l'ordre rouge, bleu, blanc, vert (spec 071, C22).
    pub shortcut: Option<String>,
}

/// « Reprendre Euphorie cristalline en défaussant : » et le choix de la carte (spec 025).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReclaimMenu {
    pub label: String,
    /// Le choix en cours : une carte de la main, ou « (ne pas reprendre) ».
    pub selected: String,
    /// « (ne pas reprendre) », puis une entrée par carte de la main.
    pub options: Vec<Button>,
}

/// Un jeton de mana de la zone de jeu (spec 025).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenView {
    /// Toutes couleurs : or et noir des Mages d'Altem (spec 042).
    pub color: ManaColor,
    pub tooltip: String,
}

/// « Effets en cours » de la zone de jeu : une ligne par modificateur, sa source puis son texte
/// (spec 052, C4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModifierList {
    pub title: String,
    pub lines: Vec<String>,
}

/// Ligne « Jetons : » de la zone de jeu (spec 025).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenLine {
    pub title: String,
    /// Dans l'ordre où ils ont été gagnés.
    pub tokens: Vec<TokenView>,
    /// Les jetons dépensés ce tour, grisés, dans l'ordre (spec 073, C26).
    pub spent: Vec<TokenView>,
}

/// « Choisissez un dé de la Source, puis sa couleur. », et les couleurs une fois le dé choisi
/// (spec 025).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetDicePrompt {
    pub text: String,
    /// Une par couleur de base, vide tant que le dé n'est pas choisi.
    pub colors: Vec<StrongButton>,
    pub cancel: Button,
}

/// Bandeau d'annonce de la fin de la manche (spec 023) : ce n'est pas un mode, sauf pendant la
/// confirmation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnouncePrompt {
    pub text: String,
    /// « Annoncer la fin de la manche », ou « Confirmer » et « Annuler ».
    pub buttons: Vec<Button>,
}

/// Icône du moment de la journée devant la ligne de la manche (spec 023).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clock {
    pub time_of_day: TimeOfDay,
    /// « Jour » ou « Nuit ».
    pub tooltip: String,
}

/// Une carte de la zone de jeu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayedLine {
    pub color: BasicColor,
    pub text: String,
    /// Le nom de la carte, son effet de base et son effet renforcé (spec 073, C27).
    pub tooltip: String,
}

/// Bandeau du mode choix de la carte à défausser.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscardPrompt {
    pub text: String,
    pub cancel: Button,
}

/// Un dé de la Source, déjà traduit (spec 020).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDie {
    pub color: ManaColor,
    /// Nom de la couleur : « vert ».
    pub name: String,
    pub tooltip: String,
    /// En mode « mettre un dé sur une couleur », le clic qui choisit ce dé (spec 025).
    pub pick: Option<Command>,
    /// Grisé : plus de dé de la Source ce tour (Magie inutile à +3, spec 058, C6).
    pub forbidden: bool,
    /// Coché pour Appel du mana (spec 060).
    pub checked: bool,
}

/// Ligne « Source » du panneau latéral (spec 020).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLine {
    pub title: String,
    /// Dés utilisables, dans l'ordre du lancer.
    pub dice: Vec<SourceDie>,
    /// « épuisé : », présent s'il y a des dés épuisés.
    pub depleted_title: Option<String>,
    /// Dés épuisés, à part (MKUE p. 5, 3.c).
    pub depleted: Vec<SourceDie>,
    /// « pris : », présent si un dé a été pris ce tour (spec 022).
    pub taken_title: Option<String>,
    /// Dés pris ce tour, relancés à la fin du tour.
    pub taken: Vec<SourceDie>,
    /// « Garder ce dé tel quel » : le dé pris avec l'avantage d'Embrasure de la Source, pas
    /// relancé à la fin du tour (spec 058, C39).
    pub keep: Option<Button>,
}

/// Les cristaux d'une couleur de l'inventaire (spec 021).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrystalGroup {
    pub color: BasicColor,
    pub count: u8,
    /// Emplacements dessinés : la limite de trois (MKUE p. 5, 1.b).
    pub limit: u8,
    /// « Cristaux verts : 1 sur 3 ».
    pub tooltip: String,
}

/// Ligne « Inventaire » du panneau latéral (spec 021).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryLine {
    pub title: String,
    /// Une par couleur de base, dans l'ordre rouge, bleu, blanc, vert.
    pub groups: Vec<CrystalGroup>,
}

/// Portrait du héros dans le panneau latéral (spec 065) : sa figurine (spec 075) et son nom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeroPortrait {
    pub image: crate::ImageId,
    pub fallback: eframe::egui::Color32,
    pub name: String,
}

/// Section « Unités » du panneau latéral (spec 042).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitsSection {
    /// « Unités : 1/2 » : unités de la zone sur jetons de commandement.
    pub header: String,
    /// Dans l'ordre du recrutement.
    pub units: Vec<UnitLine>,
    /// « Aucune unité », sans unité (spec 073, C8).
    pub empty: Option<String>,
    /// « Jeton de Loyauté : recrutement −5 » : l'emplacement libre de Loyauté sans faille (spec
    /// 058, C30).
    pub bonds_token: Option<String>,
}

/// Une unité de la zone, déjà traduite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitLine {
    pub card: UnitCard,
    /// « Paysans · niveau I » (spec 071, C13).
    pub text: String,
    /// L'icône Armure et l'armure de l'unité.
    pub armor: crate::IconValue,
    /// Une icône par résistance, sans valeur.
    pub resistances: Vec<crate::IconValue>,
    /// « Disponible » ou « Épuisée », suivi de « Blessée (1) » si elle l'est.
    pub state: String,
    /// Un bouton par capacité, dans l'ordre de la carte.
    pub abilities: Vec<AbilityButton>,
    /// « Renvoyer », pendant le choix de l'unité à renvoyer ; « Assigner », pendant celui de
    /// l'unité d'une bannière (spec 055).
    pub disband: Option<Button>,
    /// Case « cible » du sort en cours qui peut la viser (Renaissance, Flux d'énergie, issue
    /// privée 258).
    pub spell_target: Option<DiscardToggle>,
    /// La bannière qu'elle porte (spec 055, C22, C42).
    pub banner: Option<BannerTag>,
    /// Les capacités de sa bannière : « Annuler l'attaque : Swordsmen (Gloire +1) », « Rendre
    /// disponible » (spec 055, C15, C22).
    pub banner_uses: Vec<Button>,
    /// Son illustration, dessinée en vignette si l'image est présente (spec 072, C7).
    pub illustration: crate::Illustration,
    /// Spec 073 : un bouton d'effet par option de chaque capacité, avec la pastille du mana
    /// qu'elle coûte ; une capacité pas encore codée garde un bouton texte grisé.
    pub effects: Vec<EffectButton>,
}

/// Bouton d'achat pendant l'interaction (spec 041) : grisé (sans commande) quand l'influence ne
/// suffit pas ; le texte d'une action avancée en infobulle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurchaseButton {
    pub label: String,
    pub command: Option<Command>,
    pub tooltip: Option<String>,
    /// « E » pour l'offrande à l'autel (spec 071, C21) ; `None` sans touche.
    pub shortcut: Option<String>,
}

/// Fenêtre « Récompense : prenez un artefact » (spec 041) : les artefacts tirés, chacun avec son
/// bouton « Prendre ».
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactChoice {
    pub title: String,
    pub cards: Vec<super::OfferCardView>,
}

/// Fenêtre « Offrande : 3 mana bleu, 7 Gloire » (spec 045, C34) : une ligne-bouton par paiement
/// légal, la note et « Annuler ».
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AltarWindow {
    pub title: String,
    /// « dé bleu, cristal bleu, jeton bleu », dans l'ordre des actions légales.
    pub payments: Vec<Button>,
    /// « La gloire est gagnée aussitôt. »
    pub note: String,
    pub cancel: Button,
}

/// Bouton d'une capacité d'unité : grisé (sans commande) quand elle ne s'active pas maintenant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbilityButton {
    pub label: String,
    pub command: Option<Command>,
    /// « pas encore disponible » pour une capacité pas encore codée.
    pub tooltip: Option<String>,
}

/// Sorte d'un bouton d'action du plateau (spec 075, issue privée 169) : son icône et sa place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoardActionKind {
    Challenge,
    StartCombat,
    Interact,
    Plunder,
    Burn,
    Enter,
    Tribute,
}

/// Un bouton d'action du plateau, sans sa position : la case où il se pose, ce qu'il fait et son
/// infobulle (spec 075). Il existe exactement quand existait le bouton de la section Ici.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardActionButton {
    pub hex: Hex,
    pub kind: BoardActionKind,
    /// Nom court (`board.action.*`) : lecteurs d'écran et repli sans icône.
    pub label: String,
    /// `None` : grisé (offrande sans mana).
    pub command: Option<Command>,
    /// Infobulle : le libellé du bouton d'avant, son avertissement, puis « Raccourci : … ».
    pub tooltip: Vec<String>,
}

/// Menu du site pendant une interaction, à côté de sa case (spec 075, C30) : ses lignes sont
/// `GameScreen::recruit` puis `GameScreen::purchases`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiteMenu {
    pub hex: Hex,
    /// « Interaction : Village ».
    pub title: String,
    /// Replié par le bouton bulle (`Command::ToggleSiteMenu`).
    pub open: bool,
}

/// Fenêtre du choix en attente de Rituel de sang renforcé (spec 056, C4, C5) : les six couleurs de
/// jeton tant qu'il en reste à choisir, puis les paiements du cristal et « Non merci ».
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenChoice {
    pub title: String,
    /// Une par couleur de mana, or et noir compris ; vide pendant le choix du cristal.
    pub tokens: Vec<TokenButton>,
    /// « Cristal vert · jeton vert », un par paiement légal ; vide pendant le choix des jetons.
    pub payments: Vec<Button>,
    /// « Non merci », pendant le choix du cristal.
    pub decline: Option<Button>,
}

/// La fenêtre de Méditation (spec 057) : les cartes en attente, puis « Sur le deck » et « Sous le
/// deck » une fois l'ordre connu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeditationWindow {
    pub title: String,
    pub cards: Vec<MeditationCard>,
    pub top: Option<Button>,
    pub bottom: Option<Button>,
}

/// Une carte de Méditation : son nom, sa place choisie (1 = la plus haute), et le clic qui la
/// choisit ou la retire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeditationCard {
    pub name: String,
    pub place: Option<usize>,
    pub pick: Option<Command>,
}

/// Bouton d'une couleur de jeton de Rituel de sang (spec 056).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenButton {
    pub color: ManaColor,
    pub label: String,
    pub command: Command,
    /// « 1 » à « 6 », dans l'ordre des couleurs.
    pub shortcut: Option<String>,
}

/// Section « Compétences » du panneau du bas, à droite de la main (spec 051).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillsSection {
    /// « Compétences ».
    pub title: String,
    /// « Pile : 8 » : les compétences encore dans la pile, jamais leur ordre.
    pub pile: String,
    /// Dans l'ordre où elles ont été gagnées.
    pub skills: Vec<SkillLine>,
}

/// Une compétence gagnée, déjà traduite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillLine {
    pub skill: arcanum_core::Skill,
    pub name: String,
    /// « Une fois par tour » ou « Une fois par manche ».
    pub usage: String,
    /// « Utilisée ce tour », « Retournée jusqu'à la prochaine manche », « Pas encore disponible »
    /// ou le blocage de Motivation ; `None` quand elle est prête.
    pub state: Option<String>,
    /// Un bouton par option, grisé quand elle ne sert pas maintenant.
    pub options: Vec<EffectButton>,
    /// Nom, fréquence, « Hors combat » s'il y a lieu, puis le texte.
    pub tooltip: String,
}

/// Fenêtre « Niveau 2 : choisir une compétence » (spec 051, C4 à C8) : les deux compétences
/// révélées, puis l'offre commune et sa note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillChoiceWindow {
    pub title: String,
    pub revealed_title: String,
    pub revealed: Vec<SkillOffer>,
    pub common_title: String,
    pub common_note: String,
    /// Vide sans offre commune.
    pub common: Vec<SkillOffer>,
}

/// Une compétence proposée : nom, fréquence, texte, et « Prendre » si elle est prenable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillOffer {
    pub skill: arcanum_core::Skill,
    pub name: String,
    pub usage: String,
    pub text: String,
    pub take: Option<Button>,
    /// « Compétence de votre héros : pas prenable ici ».
    pub note: Option<String>,
}

/// Fenêtre du choix de la tactique (spec 060) : six places, une par numéro, en 2 lignes de 3 ;
/// `None` pour une tactique retirée de la partie (un trou).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TacticWindow {
    pub title: String,
    pub slots: Vec<Option<TacticCard>>,
    /// « Prendre cette tactique », grisé tant qu'aucune n'est encadrée.
    pub take: PurchaseButton,
}

/// Une tactique de la fenêtre de choix : grand numéro, nom, texte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TacticCard {
    pub tactic: arcanum_core::Tactic,
    pub number: u8,
    pub name: String,
    pub text: String,
    /// Encadrée : celle que « Prendre » prendrait.
    pub selected: bool,
    /// Le clic qui l'encadre ; `None` si elle n'est pas proposée (prise par un autre joueur).
    pub select: Option<Command>,
}

/// La tactique du joueur dans le panneau de droite (spec 060).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TacticLine {
    /// « N° 3 »
    pub number: String,
    pub name: String,
    /// Son texte, en infobulle.
    pub text: String,
    /// « Retournée : a servi ».
    pub flipped: Option<String>,
    /// Dé volé, cartes dessous, Au bon moment annoncé.
    pub notes: Vec<String>,
    /// Ses boutons légaux : Au bon moment, Longue Nuit, Appel du mana, Méditation nocturne.
    pub buttons: Vec<Button>,
}

/// Préparation (spec 060, C30) : les cartes du deck, une par bouton.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparationWindow {
    pub title: String,
    pub cards: Vec<Button>,
}

/// Fenêtre d'une décision de tactique : Provisions avant le tour, Longue Nuit pendant la pioche
/// (spec 060, C24, C31 à C33).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TacticDecision {
    pub text: String,
    pub buttons: Vec<PurchaseButton>,
}

/// Spec 061 : la section « Joueur fantôme » du panneau de droite (C37).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummySection {
    /// Le portrait de son héros, en petit (spec 065).
    pub portrait: HeroPortrait,
    /// « Joueur fantôme : Goldyx ».
    pub title: String,
    /// « Tactique : 2 Faux départ », une fois tirée.
    pub tactic: Option<String>,
    /// « Deck : 11 carte(s) », en évidence (MKUE p. 15, *Note 1*).
    pub deck: String,
    /// « Défausse : 5 carte(s) », qui ouvre la fenêtre de sa défausse ; grisé sans carte (C38).
    pub discard: PurchaseButton,
    pub crystals: DummyCrystals,
    pub skills: String,
    /// « Dernier tour : Rage, Célérité, Marche ».
    pub last_turn: String,
    /// « A annoncé la fin de la manche : c'est votre dernier tour. », en orange (C40).
    pub announced: Option<String>,
}

/// Spec 061 : les cristaux du fantôme, l'icône colorée de l'inventaire une fois par cristal, rangés
/// par couleur, sans emplacement vide (retour de Mathieu).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummyCrystals {
    /// « Cristaux ».
    pub title: String,
    pub crystals: Vec<BasicColor>,
    /// « bleu : 1, vert : 2 ».
    pub tooltip: String,
}
