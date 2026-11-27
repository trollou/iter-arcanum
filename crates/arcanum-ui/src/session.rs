use std::fs;
use std::path::{Path, PathBuf};

use arcanum_core::{
    Action, BasicColor, CardId, CardPlay, DieSetting, Event, GameConfig, GameRunner, Hex,
    InProgress, PlayerId, PlayerView, UndoStatus, view,
};
use arcanum_i18n::Catalog;
use arcanum_persistance::{LoadError, SaveEntry, SaveError, SaveGame, list, load, save, save_auto};
use jiff::ZonedDateTime;

use crate::departure::DepartureTracker;
use crate::{
    CombatChoice, CombatScreen, Command, Controls, DepartingEnemy, DiscardChoice, DiscardWindow,
    EndTurnChoice, FrameOutcome, GameScreen, HumanHandle, LoadWindow, ManaChoice, MenuBar,
    MenuState, Notice, OffersWindow, ScreenMode, ScreenPhase, SetDiceChoice, combat_screen,
    controls_screen, discard_window, dummy_discard_window, game_screen, human_player,
    load_window_screen, menu_bar, offers_window_in, run_frame, save_stem, saved_at,
    screen::artifact_enemy_toggles, screen::cancel_command, screen::cast_enemy_toggles,
    screen::reopen_score_button,
};

/// Raccourcis clavier de la fin du tour (spec 016), traduits depuis les touches par l'application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    /// Fin : ouvrir le mode fin du tour.
    EndTurn,
    /// Entrée : finir le tour avec les cartes cochées, ou valider l'attaque ou la parade en cours
    /// (spec 031).
    Confirm,
    /// Échap : quitter le mode en cours, ou fermer la fenêtre de chargement.
    Cancel,
    /// Ctrl+Z : revenir en arrière (spec 024).
    Undo,
    /// Ctrl+S : sauvegarder.
    Save,
    /// Ctrl+O : ouvrir la fenêtre de chargement.
    OpenLoad,
    /// Ctrl+N : ouvrir la fenêtre de nouvelle partie (spec 065), traité par `ClientSession`.
    NewGame,
    /// O : ouvrir ou fermer la fenêtre Offres (spec 050).
    Offers,
    /// 1, 2, 3 : prendre l'action avancée de cette place pendant le choix (spec 050).
    Take(u8),
    /// I : interagir avec les habitants, quand le bouton est actif (spec 040).
    Interact,
    /// E : entrer dans le site d'aventure (ouvre la confirmation, spec 044).
    EnterSite,
    /// D : ouvrir ou fermer la fenêtre de la défausse (spec 073).
    Discard,
    /// Ctrl+Q : quitter le jeu (spec 074), traité par `ClientSession`.
    Quit,
    /// F11 : passer en plein écran ou en sortir (spec 074), traité par `ClientSession`.
    Fullscreen,
    /// Ctrl+, : ouvrir ou fermer la fenêtre des préférences (spec 080), traité par `ClientSession`.
    Preferences,
    /// ↑ et ↓ : cocher l'élément précédent ou suivant de la liste de l'écran de nouvelle partie
    /// (spec 078), traités par `ClientSession`.
    PreviousChoice,
    NextChoice,
    /// ← et → : passer à l'étape précédente ou suivante de l'écran de nouvelle partie (spec 078).
    PreviousStep,
    NextStep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Playing,
    Finished,
    Stopped,
}

/// Partie en cours vue par l'UI : la boucle, la poignée de l'humain et l'état d'avancement.
pub struct GameSession {
    runner: GameRunner,
    human: HumanHandle,
    phase: Phase,
    /// Mode choix de la carte à défausser (spec 013) ou fin du tour (spec 016), propre à l'écran.
    mode: ScreenMode,
    /// Ennemis cochés et « Voir le plateau » pendant le combat (spec 031).
    combat: CombatChoice,
    /// Dossier des sauvegardes (spec 024) ; sans dossier, sauvegarder échoue proprement.
    saves_dir: Option<PathBuf>,
    /// Horloge du nom et du résumé des sauvegardes ; fixée par les tests.
    clock: fn() -> ZonedDateTime,
    notice: Option<Notice>,
    /// Fenêtre de chargement ouverte, avec les sauvegardes listées à l'ouverture.
    load_entries: Option<Vec<SaveEntry>>,
    /// Taille du journal à la dernière sauvegarde automatique : une seule par début de tour.
    autosaved_len: Option<usize>,
    /// Fenêtre de fin fermée par « Voir le plateau » (spec 063).
    score_closed: bool,
    /// Fenêtre Offres ouverte pour consultation (spec 050).
    offers_open: bool,
    /// Fenêtre de la défausse ouverte (spec 073).
    discard_open: bool,
    /// Spec 061 : la fenêtre de la défausse du fantôme, à la place de celle du joueur.
    dummy_discard_open: bool,
    /// Menu du site replié par le bouton bulle, jusqu'à la fin de l'interaction (spec 075, C30).
    site_menu_folded: bool,
    /// Taille du journal au dernier `Play` : les pas joués depuis forment le trajet (spec 075).
    walk_log_from: Option<usize>,
    /// Dernier déplacement de la figurine : son numéro et ses cases (spec 075, C35).
    walk: Option<(u64, Vec<Hex>)>,
    walk_count: u64,
    /// Spec #5 : la dernière disparition des ennemis vaincus.
    departure: DepartureTracker,
    /// Change quand le journal est réécrit (retour en arrière, partie chargée) : le son ne
    /// rejoue pas les événements d'avant (spec 081, C7, C8).
    log_epoch: u64,
}

impl GameSession {
    /// Partie solo avec le joueur humain.
    pub fn new(config: GameConfig) -> Self {
        let (player, human) = human_player();
        Self::with_runner(GameRunner::new(config, vec![Box::new(player)]), human)
    }

    pub fn with_runner(runner: GameRunner, human: HumanHandle) -> Self {
        let runner_len = runner.log().len();
        Self {
            runner,
            human,
            phase: Phase::Playing,
            mode: ScreenMode::Normal,
            combat: CombatChoice::default(),
            saves_dir: None,
            clock: ZonedDateTime::now,
            notice: None,
            load_entries: None,
            autosaved_len: None,
            score_closed: false,
            offers_open: false,
            discard_open: false,
            dummy_discard_open: false,
            site_menu_folded: false,
            walk_log_from: None,
            walk: None,
            walk_count: 0,
            departure: DepartureTracker::from_log_len(runner_len),
            log_epoch: 0,
        }
    }

    pub fn with_saves_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.saves_dir = dir;
        self
    }

    pub fn with_clock(mut self, clock: fn() -> ZonedDateTime) -> Self {
        self.clock = clock;
        self
    }

    /// Partie reprise d'une sauvegarde (spec 024), pour « Continuer » et « Charger… » de l'écran
    /// de nouvelle partie (spec 065).
    pub fn from_save(path: &Path) -> Result<Self, LoadError> {
        let (runner, human) = resume(path)?;
        let mut session = Self::with_runner(runner, human);
        session.autosaved_len = Some(session.runner.log().len());
        Ok(session)
    }

    /// Numéro de version du journal : change à chaque retour en arrière ou chargement (spec 081).
    pub fn log_epoch(&self) -> u64 {
        self.log_epoch
    }

    /// La boucle de la partie, en lecture : sauvegardes et tests.
    pub fn runner(&self) -> &GameRunner {
        &self.runner
    }

    /// Configuration de départ, lue dans `GameStarted` : jamais affichée (spec 002, la graine).
    pub fn config(&self) -> Option<GameConfig> {
        self.runner
            .log()
            .entries()
            .iter()
            .find_map(|entry| match &entry.event {
                Event::GameStarted { config } => Some(config.clone()),
                _ => None,
            })
    }

    /// Partie terminée (spec 063), ou fin des manches.
    pub fn is_finished(&self) -> bool {
        self.phase == Phase::Finished || self.runner.state().game_end().is_some()
    }

    /// La fenêtre de nouvelle partie peut s'ouvrir : hors mode choix, comme Ctrl+S (spec 065, L4).
    pub fn can_open_new_game(&self) -> bool {
        !self.in_mode()
    }

    /// Le tour a avancé depuis la sauvegarde automatique : son journal est plus long qu'à son
    /// écriture (spec 074, Q1). Quitter ferait perdre ce qui a été joué depuis.
    pub fn advanced_since_autosave(&self) -> bool {
        self.autosaved_len
            .is_some_and(|len| self.runner.log().len() > len)
    }

    /// Message du panneau, posé par l'écran de nouvelle partie (`new_game.previous_saved`).
    pub fn set_notice(&mut self, notice: Notice) {
        self.notice = Some(notice);
    }

    /// Écrit une sauvegarde ordinaire, comme Ctrl+S, sans message : son nom, ou l'erreur.
    pub fn save_copy(&self) -> Result<String, SaveError> {
        let dir = self
            .saves_dir
            .clone()
            .ok_or_else(|| SaveError::Io("aucun dossier des sauvegardes".to_string()))?;
        let stem = save_stem(&(self.clock)());
        self.write(&dir, |dir, game| save(dir, &stem, game))
            .map(|path| file_stem(&path))
    }

    /// Avance pendant une image ; `true` s'il faut redemander une image.
    pub fn run_frame(&mut self) -> bool {
        if self.phase != Phase::Playing {
            return false;
        }
        let state = self.runner.state();
        self.departure
            .note_board(self.runner.log().len(), || view(state, PlayerId::SOLO));
        let again = match run_frame(&mut self.runner).outcome {
            FrameOutcome::Waiting => false,
            FrameOutcome::StepLimitReached => true,
            FrameOutcome::Finished => {
                self.phase = Phase::Finished;
                false
            }
            FrameOutcome::Error(_) => {
                self.phase = Phase::Stopped;
                false
            }
        };
        self.record_walk();
        self.departure.scan(self.runner.log().entries());
        self.autosave();
        again
    }

    /// Relève les pas de la figurine joués depuis le dernier `Play` (spec 075, C35).
    fn record_walk(&mut self) {
        let Some(from) = self.walk_log_from else {
            return;
        };
        let entries = self.runner.log().entries();
        if entries.len() <= from {
            return;
        }
        self.walk_log_from = None;
        let mut path: Vec<Hex> = Vec::new();
        for entry in &entries[from..] {
            if let Event::HeroMoved {
                player, from, to, ..
            } = entry.event
                && player == PlayerId::SOLO
            {
                if path.is_empty() {
                    path.push(from);
                }
                path.push(to);
            }
        }
        if !path.is_empty() {
            self.walk_count += 1;
            self.walk = Some((self.walk_count, path));
        }
    }

    /// Sauvegarde automatique au début de chaque tour (spec 024, Q5) : le dernier point de
    /// non-retour a fermé le tour et rien n'a été joué depuis.
    fn autosave(&mut self) {
        if self.runner.undo_status() != UndoStatus::NothingSinceTurnStart {
            return;
        }
        let len = self.runner.log().len();
        if self.autosaved_len == Some(len) {
            return;
        }
        self.autosaved_len = Some(len);
        let Some(dir) = self.saves_dir.clone() else {
            return;
        };
        match self.write(&dir, save_auto) {
            Ok(_) => {
                if self.notice == Some(Notice::AutosaveFailed) {
                    self.notice = None;
                }
            }
            Err(error) => {
                log::error!("sauvegarde automatique impossible : {error}");
                self.notice = Some(Notice::AutosaveFailed);
            }
        }
    }

    /// Crée le dossier s'il manque, puis écrit la partie avec `writer`.
    fn write(
        &self,
        dir: &Path,
        writer: impl Fn(&Path, &SaveGame) -> Result<PathBuf, SaveError>,
    ) -> Result<PathBuf, SaveError> {
        fs::create_dir_all(dir).map_err(|error| SaveError::Io(error.to_string()))?;
        let game = SaveGame::from_runner(&self.runner, &saved_at(&(self.clock)()))?;
        writer(dir, &game)
    }

    fn save_game(&mut self) {
        self.notice = Some(match self.save_copy() {
            Ok(name) => Notice::Saved(name),
            Err(error) => {
                log::error!("sauvegarde impossible : {error}");
                Notice::SaveFailed
            }
        });
    }

    fn load_game(&mut self, index: usize) {
        let Some(entry) = self
            .load_entries
            .take()
            .and_then(|entries| entries.into_iter().nth(index))
        else {
            return;
        };
        match resume(&entry.path) {
            Ok((runner, human)) => {
                log::info!("partie chargée : {}", entry.path.display());
                self.autosaved_len = Some(runner.log().len());
                self.runner = runner;
                self.log_epoch += 1;
                self.human = human;
                self.phase = Phase::Playing;
                self.mode = ScreenMode::Normal;
                self.notice = None;
                self.score_closed = false;
                self.offers_open = false;
                self.walk = None;
                self.walk_log_from = None;
                self.departure = DepartureTracker::from_log_len(self.runner.log().len());
            }
            Err(error) => {
                log::error!(
                    "chargement impossible de {} : {error}",
                    entry.path.display()
                );
                self.notice = Some(Notice::LoadFailed);
            }
        }
    }

    /// Un mode de choix ou la fenêtre de chargement est ouvert : ni retour en arrière, ni
    /// sauvegarde, ni chargement (spec 024, L7).
    fn in_mode(&self) -> bool {
        self.mode != ScreenMode::Normal || self.load_entries.is_some()
    }

    /// Boutons Revenir en arrière, Sauvegarder, Charger… (spec 024).
    pub fn controls(&self, catalog: &Catalog) -> Controls {
        controls_screen(
            self.runner.undo_status(),
            self.in_mode(),
            self.notice.as_ref(),
            catalog,
        )
    }

    /// La barre de menu (spec 074), sans fenêtre de nouvelle partie ni plein écran : la session
    /// du client y ajoute les siens.
    pub fn menu_bar(&self, catalog: &Catalog) -> MenuBar {
        self.menu_bar_with(MenuState::default(), catalog)
    }

    pub(crate) fn menu_bar_with(&self, state: MenuState, catalog: &Catalog) -> MenuBar {
        menu_bar(&self.view(), &self.controls(catalog), state, catalog)
    }

    /// Fenêtre de chargement, si elle est ouverte.
    pub fn load_window(&self, catalog: &Catalog) -> Option<LoadWindow> {
        self.load_entries
            .as_ref()
            .map(|entries| load_window_screen(entries, catalog))
    }

    pub fn screen(&self, catalog: &Catalog) -> GameScreen {
        let request = self.human.pending();
        let phase = match self.phase {
            Phase::Playing => ScreenPhase::Playing {
                legal_actions: request
                    .as_ref()
                    .map_or(&[], |request| &request.legal_actions),
            },
            Phase::Finished => ScreenPhase::Finished,
            Phase::Stopped => ScreenPhase::Stopped,
        };
        let mut screen = game_screen(&self.view(), phase, &self.mode, catalog);
        // Spec 063 : fermée, la fenêtre de fin laisse le bouton qui la rouvre.
        if self.score_closed && screen.score_window.take().is_some() {
            screen.reopen_score = Some(reopen_score_button(catalog));
        }
        if let Some(menu) = &mut screen.site_menu {
            menu.open = !self.site_menu_folded;
        }
        screen
    }

    /// La fenêtre de la défausse, quand elle est ouverte et que la défausse a des cartes
    /// (spec 073, C35, C36).
    pub fn discard_window(&self, catalog: &Catalog) -> Option<DiscardWindow> {
        if self.dummy_discard_open {
            return dummy_discard_window(&self.view(), catalog);
        }
        if !self.discard_open {
            return None;
        }
        discard_window(&self.view(), catalog)
    }

    /// La fenêtre Offres, ouverte pour consultation (spec 050, spec #20).
    pub fn offers_window(&self, catalog: &Catalog) -> Option<OffersWindow> {
        let view = self.view();
        let legal = self.legal_actions();
        // Spec #20, C18 : seulement à la demande ; chaque choix de carte a sa fenêtre.
        self.offers_open
            .then(|| offers_window_in(&view, &legal, &self.mode, catalog))
    }

    /// Choix d'une action avancée en attente (spec 050), après ceux de l'artefact (spec 041) et du
    /// sort (spec 043).
    fn is_choosing_advanced_action(&self) -> bool {
        let view = self.view();
        view.advanced_action_choices > 0
            && view.drawn_artifacts.is_empty()
            && view.spell_choices == 0
            // Spec 051, C6 : la compétence du niveau d'abord.
            && view.skill_choice.is_empty()
    }

    /// Spec #20, C5 : la compétence encadrée, puis l'action avancée, légale une fois la
    /// compétence jouée ; rien si la compétence encadrée ne laisse pas prendre cette carte.
    fn take_level_reward(&mut self, card: CardId) {
        let ScreenMode::ChoosingLevelSkill(skill) = self.mode else {
            return;
        };
        let allowed = self
            .view()
            .skill_choice_cards
            .iter()
            .any(|(takeable, cards)| *takeable == skill && cards.contains(&card));
        let Some(index) = self
            .legal_actions()
            .iter()
            .position(|action| *action == Action::ChooseSkill { skill })
            .filter(|_| allowed)
        else {
            return;
        };
        self.command(Command::Play(index));
        self.run_frame();
        if let Some(index) = self
            .legal_actions()
            .iter()
            .position(|action| *action == Action::GainAdvancedAction { card })
        {
            self.command(Command::Play(index));
        }
    }

    /// Spec #20, C10 : pendant le choix d'une compétence de niveau pair, 1 à 3 prennent la carte de
    /// cette place si la compétence encadrée la laisse prendre ; les autres touches de jeu n'agissent
    /// pas.
    fn level_shortcut(&mut self, shortcut: Shortcut) {
        let Shortcut::Take(place) = shortcut else {
            return;
        };
        let card = self
            .view()
            .offers
            .get(&arcanum_core::Offer::AdvancedActions)
            .and_then(|offer| offer.get(usize::from(place).wrapping_sub(1)))
            .and_then(|card| card.deed());
        if let Some(card) = card {
            self.take_level_reward(card);
        }
    }

    /// Choix de l'unité de la récompense en attente (spec 045), après tous les autres choix de la
    /// fin du tour.
    fn is_choosing_unit_reward(&self) -> bool {
        let view = self.view();
        view.unit_choices > 0
            && view.drawn_artifacts.is_empty()
            && view.spell_choices == 0
            && view.advanced_action_choices == 0
    }

    /// Choix du sort de la récompense en attente, après celui de l'artefact (spec 043).
    fn is_choosing_spell(&self) -> bool {
        let view = self.view();
        view.spell_choices > 0 && view.drawn_artifacts.is_empty()
    }

    /// Choix de l'artefact de la récompense en attente (spec 041).
    fn is_choosing_artifact(&self) -> bool {
        !self.view().drawn_artifacts.is_empty()
    }

    /// « Annuler » de l'attaque ou de la parade en cours, si elle ne suffit pas (spec 070).
    fn cancel_in_progress(&self) -> Option<Command> {
        let combat = self.view().combat.filter(|combat| !combat.ended)?;
        cancel_command(&combat, self.runner.undo_status())
    }

    /// L'écran de combat, pendant un combat (spec 031). Pendant le choix d'Ailes de la nuit, les
    /// cases « cible » sont celles du sort (issue privée 258).
    pub fn combat_screen(&self, catalog: &Catalog) -> Option<CombatScreen> {
        let legal = self.legal_actions();
        let mut screen = combat_screen(
            &self.view(),
            &legal,
            &self.combat,
            self.runner.undo_status(),
            catalog,
        )?;
        let mut toggles = cast_enemy_toggles(&legal, &self.mode, catalog);
        // Spec 055 : les ennemis de la Bannière de peur renforcée.
        toggles.extend(artifact_enemy_toggles(&legal, &self.mode, catalog));
        if !toggles.is_empty() {
            for enemy in &mut screen.enemies {
                enemy.target = toggles
                    .iter()
                    .find(|(token, _)| *token == enemy.token)
                    .map(|(_, toggle)| toggle.clone());
            }
        }
        Some(screen)
    }

    /// Actions légales offertes au joueur humain ; vide sans décision en attente.
    pub fn legal_actions(&self) -> Vec<Action> {
        match (self.phase, self.human.pending()) {
            (Phase::Playing, Some(request)) => request.legal_actions,
            _ => Vec::new(),
        }
    }

    /// La vue du joueur humain, jamais l'état complet.
    pub fn view(&self) -> PlayerView {
        view(self.runner.state(), PlayerId::SOLO)
    }

    /// Exécute le bouton cliqué.
    pub fn command(&mut self, command: Command) {
        // Fenêtre de chargement ouverte : seuls ses propres boutons agissent (L7).
        if self.load_entries.is_some() && !matches!(command, Command::Load(_) | Command::CloseLoad)
        {
            return;
        }
        // Spec 075, C30 : le menu du site se rouvre à l'interaction suivante.
        if self.view().interaction.is_none() {
            self.site_menu_folded = false;
        }
        match command {
            Command::Play(index) => {
                // Issue privée 247 : un sort lancé depuis l'offre referme la fenêtre Offres, même sans
                // passer par le bandeau du choix (un seul paiement).
                if matches!(
                    self.legal_actions().get(index),
                    Some(Action::PlayCard {
                        play: CardPlay::Cast {
                            from_offer: true,
                            ..
                        },
                        ..
                    })
                ) {
                    self.offers_open = false;
                }
                self.mode = ScreenMode::Normal;
                self.combat.targets.clear();
                if matches!(self.notice, Some(Notice::Saved(_))) {
                    self.notice = None;
                }
                self.walk_log_from = Some(self.runner.log().len());
                self.human.submit(index);
            }
            Command::Undo => {
                if !self.in_mode() && self.runner.undo().is_ok() {
                    self.log_epoch += 1;
                    self.phase = Phase::Playing;
                    self.notice = None;
                    // L6 : la figurine est aussitôt sur sa case.
                    self.walk = None;
                    self.walk_log_from = None;
                    self.departure.undone(self.runner.log().len());
                }
            }
            Command::Save => {
                if !self.in_mode() {
                    self.save_game();
                }
            }
            Command::OpenLoad => {
                if !self.in_mode() {
                    self.load_entries =
                        Some(self.saves_dir.as_deref().map(list).unwrap_or_default());
                }
            }
            Command::Load(index) => self.load_game(index),
            Command::CloseLoad => self.load_entries = None,
            Command::ToggleTarget(token) => {
                match self.combat.targets.iter().position(|&t| t == token) {
                    Some(index) => {
                        self.combat.targets.remove(index);
                    }
                    None => self.combat.targets.push(token),
                }
            }
            // Issue privée 258 : les cibles d'un sort se cochent une à une.
            Command::ToggleCastEnemy(enemy) => {
                if let ScreenMode::Casting(choice) = &mut self.mode {
                    choice.picks.enemies = toggled(choice.picks.enemies, enemy);
                }
            }
            Command::ToggleCastUnit(unit) => {
                if let ScreenMode::Casting(choice) = &mut self.mode {
                    choice.picks.units = toggled(choice.picks.units, unit);
                }
            }
            // Issue privée 307 : replier le menu d'une interaction où rien n'a été fait la reprend.
            Command::ToggleSiteMenu => {
                if !self.site_menu_folded && self.runner.cancel_untouched_interaction() {
                    self.phase = Phase::Playing;
                    self.notice = None;
                } else {
                    self.site_menu_folded = !self.site_menu_folded;
                }
            }
            Command::ToggleCombatBoard => {
                self.combat.viewing_board = !self.combat.viewing_board;
            }
            // Spec 070 : revenir en arrière tant que la parade, ou l'attaque, est en cours.
            Command::CancelBlock | Command::CancelAttack => {
                let block = command == Command::CancelBlock;
                while self
                    .view()
                    .combat
                    .and_then(|combat| combat.in_progress)
                    .is_some_and(|in_progress| {
                        matches!(in_progress, InProgress::Block { .. }) == block
                    })
                    && self.runner.undo().is_ok()
                {
                    self.phase = Phase::Playing;
                    self.notice = None;
                }
            }
            Command::CloseScore => self.score_closed = true,
            // Fenêtre de nouvelle partie : traitée par `ClientSession` (spec 065).
            Command::OpenNewGame
            | Command::ChooseHero(_)
            | Command::ChooseScenario(_)
            | Command::ChooseStep(_)
            | Command::ToggleAdvanced
            | Command::ToggleDummy
            | Command::ChooseDummyHero(_)
            | Command::StartNewGame
            | Command::CancelNewGame
            | Command::ContinueGame => {}
            // Commandes de la fenêtre : traitées par `ClientSession` (spec 074).
            Command::Recenter
            | Command::ToggleFullscreen
            | Command::Quit
            | Command::ConfirmQuit
            | Command::CancelQuit
            | Command::OpenPreferences
            | Command::ClosePreferences
            | Command::OpenAbout
            | Command::CloseAbout
            | Command::SetLanguage(_)
            | Command::SetFullscreenAtLaunch(_)
            | Command::SetAnimations(_)
            | Command::SetVolume(..)
            | Command::SaveVolumes => {}
            // Spec 060 : la tactique encadrée, les cartes et les dés cochés.
            Command::SelectTactic(tactic) => self.mode = ScreenMode::ChoosingTactic(tactic),
            // Spec #20 : la compétence encadrée n'est jouée qu'avec la carte (Q1).
            Command::SelectSkill(skill) => self.mode = ScreenMode::ChoosingLevelSkill(skill),
            Command::TakeLevelReward(card) => self.take_level_reward(card),
            Command::ToggleTacticCard(card) => {
                let mut picks = match &self.mode {
                    ScreenMode::PickingTacticCards(picks) => picks.clone(),
                    _ => Vec::new(),
                };
                toggle(&mut picks, card);
                self.mode = ScreenMode::PickingTacticCards(picks);
            }
            Command::OpenMidnightMeditation => {
                self.mode = ScreenMode::PickingTacticCards(Vec::new());
            }
            Command::OpenManaSearch => self.mode = ScreenMode::PickingTacticDice(Vec::new()),
            Command::ToggleTacticDie(die) => {
                let mut picks = match &self.mode {
                    ScreenMode::PickingTacticDice(picks) => picks.clone(),
                    _ => Vec::new(),
                };
                toggle(&mut picks, die);
                self.mode = ScreenMode::PickingTacticDice(picks);
            }
            Command::ReopenScore => self.score_closed = false,
            // Spec #20, C19 : pas pendant un choix de carte.
            Command::OpenOffers => {
                self.offers_open = !crate::screen::choosing_card(&self.view());
            }
            Command::CloseOffers => self.offers_open = false,
            // Spec 073 : la fenêtre de la défausse (étape 10).
            Command::OpenDiscard => {
                self.dummy_discard_open = false;
                self.discard_open = !self.view().discard.is_empty();
            }
            Command::CloseDiscard => {
                self.discard_open = false;
                self.dummy_discard_open = false;
            }
            Command::OpenDummyDiscard => {
                self.discard_open = false;
                self.dummy_discard_open = self
                    .view()
                    .dummy
                    .is_some_and(|dummy| !dummy.discard.is_empty());
            }
            // Spec 051, C22 : l'unité visée par Inspiration.
            Command::ChooseSkillUnit { skill, option } => {
                self.mode = ScreenMode::ChoosingSkillUnit { skill, option };
            }
            // Spec 058 : la cible d'une compétence.
            Command::ChooseSkillTarget(choice) => {
                self.mode = ScreenMode::ChoosingSkillTarget(choice);
            }
            Command::ChooseUnitAbility {
                unit,
                ability,
                option,
            } => {
                self.mode = ScreenMode::ChoosingUnitAbility {
                    unit,
                    ability,
                    option,
                };
            }
            Command::ChooseDisband(unit) => self.mode = ScreenMode::Disbanding(unit),
            Command::ChooseSpellMana(card) => {
                let buyable = self
                    .legal_actions()
                    .iter()
                    .any(|action| matches!(action, Action::BuySpell { card: c, .. } if *c == card));
                if buyable {
                    self.mode = ScreenMode::ChoosingSpellMana(card);
                }
            }
            Command::OpenAdventure => {
                if self.legal_actions().contains(&Action::EnterAdventureSite) {
                    self.mode = ScreenMode::ConfirmingAdventure;
                }
            }
            Command::OpenAltar => {
                let payable = self
                    .legal_actions()
                    .iter()
                    .any(|action| matches!(action, Action::PayAltarTribute { .. }));
                if payable {
                    self.mode = ScreenMode::PayingTribute;
                }
            }
            Command::ChooseGlade(glade) => {
                if let ScreenMode::EndingTurn(ending) = &mut self.mode {
                    ending.glade = Some(glade);
                }
            }
            Command::OpenBurn => {
                if self.legal_actions().contains(&Action::BurnMonastery) {
                    self.mode = ScreenMode::ConfirmingBurn;
                }
            }
            Command::ChooseDiscard { card, option } => {
                self.mode = ScreenMode::ChoosingDiscard(DiscardChoice { card, option });
            }
            Command::ChooseStrongDiscard { card, option } => {
                self.mode = ScreenMode::ChoosingStrongDiscard(DiscardChoice { card, option });
            }
            Command::ChooseMana {
                card,
                option,
                discard,
            } => {
                self.mode = ScreenMode::ChoosingMana(ManaChoice {
                    card,
                    option,
                    discard,
                });
            }
            Command::ChooseBasicMana { card, option } => {
                self.mode = ScreenMode::ChoosingBasicMana(ManaChoice {
                    card,
                    option,
                    discard: None,
                });
            }
            Command::ChooseReclaim(card) => {
                if let ScreenMode::EndingTurn(ending) = &mut self.mode {
                    ending.reclaim = card;
                }
            }
            Command::ChooseSetDice { card, option } => {
                self.mode = ScreenMode::SettingDice(SetDiceChoice {
                    card,
                    option,
                    settings: [None, None],
                    die: None,
                });
            }
            Command::PickSetDie(die) => {
                if let ScreenMode::SettingDice(choice) = &mut self.mode {
                    choice.die = Some(die);
                }
            }
            Command::PickSetColor(color) => {
                if let ScreenMode::SettingDice(choice) = &mut self.mode
                    && let Some(die) = choice.die.take()
                    && let Some(slot) = choice.settings.iter_mut().find(|slot| slot.is_none())
                {
                    *slot = Some(DieSetting { die, color });
                }
            }
            Command::ChooseSetDiceMana {
                card,
                option,
                set_dice,
            } => {
                self.mode = ScreenMode::ChoosingSetDiceMana(SetDiceChoice {
                    card,
                    option,
                    settings: set_dice,
                    die: None,
                });
            }
            Command::ChooseBoost(choice) => self.mode = ScreenMode::Boosting(choice),
            Command::ChooseGain(choice) => self.mode = ScreenMode::ChoosingGain(choice),
            Command::ChooseDie { card, option } => {
                self.mode = ScreenMode::ChoosingDie(DiscardChoice { card, option });
            }
            // Spec 054 : le bandeau du choix du sort, sous la fenêtre Offres refermée.
            Command::ChooseCast(choice) => {
                self.mode = ScreenMode::Casting(choice);
                if choice.from_offer {
                    self.offers_open = false;
                }
            }
            Command::ChooseDestination(choice) => {
                self.mode = ScreenMode::ChoosingDestination(choice);
            }
            // Spec 055 : les choix d'un artefact, l'unité d'une bannière, les bannières reprises.
            Command::ChooseArtifact(choice) => self.mode = ScreenMode::PlayingArtifact(choice),
            Command::AssignBanner(card) => self.mode = ScreenMode::AssigningBanner(card),
            Command::ChooseReclaimed(banners) => {
                self.mode = ScreenMode::ReclaimingBanners(banners);
            }
            // Spec 057 : l'ordre des cartes de Méditation, carte par carte.
            Command::PickMeditationCard(card) => {
                let mut picked = match &self.mode {
                    ScreenMode::Meditating(picked) => picked.clone(),
                    _ => Vec::new(),
                };
                match picked.iter().position(|&chosen| chosen == card) {
                    Some(index) => {
                        picked.remove(index);
                    }
                    None => picked.push(card),
                }
                self.mode = ScreenMode::Meditating(picked);
            }
            Command::CancelChoice => self.mode = ScreenMode::Normal,
            Command::OpenEndTurn => {
                if self.can_end_turn() {
                    self.mode = ScreenMode::EndingTurn(EndTurnChoice::default());
                    // Main vide : rien à défausser, le tour se finit tout de suite (issue privée 25).
                    if self.view().hand.is_empty() {
                        self.command(Command::ConfirmEndTurn);
                    }
                }
            }
            Command::ToggleEndTurnDiscard(card) => {
                if let ScreenMode::EndingTurn(ending) = &mut self.mode {
                    match ending.discard.iter().position(|&held| held == card) {
                        Some(index) => {
                            ending.discard.remove(index);
                        }
                        None => ending.discard.push(card),
                    }
                }
            }
            Command::OpenAnnounce => {
                let legal = self.legal_actions();
                if legal.contains(&Action::AnnounceEndOfRound) && !self.view().hand.is_empty() {
                    self.mode = ScreenMode::ConfirmingEndOfRound;
                }
            }
            Command::ToggleSteadyTempo => {
                if let ScreenMode::EndingTurn(ending) = &mut self.mode {
                    ending.steady_tempo = Some(!ending.steady_tempo.unwrap_or(true));
                }
            }
            Command::ConfirmEndTurn => {
                if let ScreenMode::EndingTurn(ending) = &self.mode {
                    let action = ending.action(&self.view(), &self.legal_actions());
                    // « Finir le tour » inactif : rien ne part (MKUE p. 6, 5.a).
                    if let Some(index) = self
                        .legal_actions()
                        .iter()
                        .position(|legal| *legal == action)
                    {
                        self.command(Command::Play(index));
                    }
                }
            }
        }
    }

    /// Raccourci clavier : Fin seulement hors des modes ; Entrée en mode fin du tour, ou hors des
    /// modes pour valider l'attaque ou la parade en cours.
    pub fn shortcut(&mut self, shortcut: Shortcut) {
        // La fenêtre de chargement passe avant tout : Échap la ferme, les autres touches attendent.
        if self.load_entries.is_some() {
            if shortcut == Shortcut::Cancel {
                self.command(Command::CloseLoad);
            }
            return;
        }
        // Spec 063 : Échap ferme la fenêtre de fin, comme « Voir le plateau ».
        if shortcut == Shortcut::Cancel && !self.score_closed && self.view().final_score.is_some() {
            self.command(Command::CloseScore);
            return;
        }
        // Spec 044 : pendant le choix de la couleur du cristal, 1 à 4 choisissent rouge, bleu,
        // blanc, vert ; les autres touches de jeu n'agissent pas.
        if self.view().crystal_choices > 0
            && !matches!(
                shortcut,
                Shortcut::Undo | Shortcut::Save | Shortcut::OpenLoad | Shortcut::NewGame
            )
        {
            if let Shortcut::Take(place) = shortcut
                && let Some(&color) = BasicColor::ALL.get(usize::from(place).wrapping_sub(1))
                && let Some(index) = self
                    .legal_actions()
                    .iter()
                    .position(|action| *action == Action::ChooseCrystalColor { color })
            {
                self.command(Command::Play(index));
            }
            return;
        }
        // Spec 041 : pendant le choix de l'artefact, 1 et 2 prennent un artefact tiré, dans l'ordre
        // du tirage ; les autres touches de jeu n'agissent pas.
        if self.is_choosing_artifact()
            && !matches!(
                shortcut,
                Shortcut::Undo | Shortcut::Save | Shortcut::OpenLoad | Shortcut::NewGame
            )
        {
            if let Shortcut::Take(place) = shortcut {
                self.take_nth(place, |action| {
                    matches!(action, Action::TakeArtifact { .. })
                });
            }
            return;
        }
        // Spec 043 : pendant le choix du sort, 1, 2 et 3 prennent un sort de l'offre.
        if self.is_choosing_spell()
            && !matches!(
                shortcut,
                Shortcut::Undo | Shortcut::Save | Shortcut::OpenLoad | Shortcut::NewGame
            )
        {
            if let Shortcut::Take(place) = shortcut {
                self.take_nth(place, |action| matches!(action, Action::TakeSpell { .. }));
            }
            return;
        }
        // Spec #20 : pendant le choix de la compétence d'un niveau pair, la fenêtre du niveau.
        if !self.view().skill_choice.is_empty()
            && !self.is_choosing_artifact()
            && !self.is_choosing_spell()
            && !matches!(
                shortcut,
                Shortcut::Undo | Shortcut::Save | Shortcut::OpenLoad | Shortcut::NewGame
            )
        {
            self.level_shortcut(shortcut);
            return;
        }
        // Spec 050 : pendant le choix d'une action avancée, 1, 2 et 3 prennent une carte ; O, Échap,
        // Fin et Entrée n'agissent pas.
        if self.is_choosing_advanced_action()
            && !matches!(
                shortcut,
                Shortcut::Undo | Shortcut::Save | Shortcut::OpenLoad | Shortcut::NewGame
            )
        {
            if let Shortcut::Take(place) = shortcut {
                self.take_nth(place, |action| {
                    matches!(action, Action::GainAdvancedAction { .. })
                });
            }
            return;
        }
        // Spec 045 : pendant le choix de l'unité de récompense, 1, 2 et 3 prennent une unité de
        // l'offre (ou ouvrent le choix de l'unité à renvoyer) ; Échap quitte ce choix-là.
        if self.is_choosing_unit_reward()
            && !matches!(
                shortcut,
                Shortcut::Undo | Shortcut::Save | Shortcut::OpenLoad | Shortcut::NewGame
            )
        {
            match shortcut {
                Shortcut::Take(place) => self.take_unit_reward(place),
                Shortcut::Cancel => self.command(Command::CancelChoice),
                _ => {}
            }
            return;
        }
        // Spec 060 : pendant le choix de la tactique, 1 à 6 encadrent la tactique de ce numéro,
        // Entrée la prend ; les autres touches de jeu n'agissent pas.
        if !self.view().tactic_choice.is_empty()
            && !matches!(
                shortcut,
                Shortcut::Undo | Shortcut::Save | Shortcut::OpenLoad | Shortcut::NewGame
            )
        {
            self.tactic_shortcut(shortcut);
            return;
        }
        // Spec 073 : Échap ferme la fenêtre de la défausse, avant tout mode.
        if shortcut == Shortcut::Cancel && (self.discard_open || self.dummy_discard_open) {
            self.command(Command::CloseDiscard);
            return;
        }
        // Spec 050 : Échap ferme la fenêtre Offres avant tout mode.
        if shortcut == Shortcut::Cancel && self.offers_open {
            self.offers_open = false;
            return;
        }
        match (shortcut, &self.mode) {
            (Shortcut::Offers, ScreenMode::Normal) => self.offers_open = !self.offers_open,
            // Spec 073 : D ouvre et ferme la fenêtre de la défausse, hors des modes.
            (Shortcut::Discard, ScreenMode::Normal) => {
                if self.discard_open || self.dummy_discard_open {
                    self.command(Command::CloseDiscard);
                } else {
                    self.command(Command::OpenDiscard);
                }
            }
            (Shortcut::EndTurn, ScreenMode::Normal) => self.command(Command::OpenEndTurn),
            // Spec 040 : I, quand le bouton « Interagir avec les habitants » est actif.
            (Shortcut::Interact, ScreenMode::Normal) => {
                if let Some(index) = self
                    .legal_actions()
                    .iter()
                    .position(|action| *action == Action::Interact)
                {
                    self.command(Command::Play(index));
                }
            }
            // Spec 044 : E, quand le bouton « Entrer dans » est actif ; sur un autel, l'offrande
            // (spec 045 : une ruine a l'un ou l'autre).
            (Shortcut::EnterSite, ScreenMode::Normal) => {
                self.command(Command::OpenAdventure);
                self.command(Command::OpenAltar);
            }
            // Spec 045 : 1 à 9 choisissent le paiement de l'offrande.
            (Shortcut::Take(place), ScreenMode::PayingTribute) => {
                self.take_nth(place, |action| {
                    matches!(action, Action::PayAltarTribute { .. })
                });
            }
            (Shortcut::Confirm, ScreenMode::EndingTurn(_)) => self.command(Command::ConfirmEndTurn),
            // Spec 031, Q8 : Entrée valide l'attaque ou la parade en cours ; spec 070 : pas si elle
            // ne suffit pas.
            (Shortcut::Confirm, ScreenMode::Normal) => {
                if self.cancel_in_progress().is_some() {
                    return;
                }
                if let Some(index) = self.legal_actions().iter().position(|action| {
                    matches!(action, Action::ResolveAttack | Action::ResolveBlock)
                }) {
                    self.command(Command::Play(index));
                }
            }
            // Spec 070 : Échap annule l'attaque ou la parade insuffisante.
            (Shortcut::Cancel, ScreenMode::Normal) if self.cancel_in_progress().is_some() => {
                if let Some(command) = self.cancel_in_progress() {
                    self.command(command);
                }
            }
            (Shortcut::Cancel, _) => self.command(Command::CancelChoice),
            (Shortcut::Undo, _) => self.command(Command::Undo),
            (Shortcut::Save, _) => self.command(Command::Save),
            (Shortcut::OpenLoad, _) => self.command(Command::OpenLoad),
            (
                Shortcut::EndTurn
                | Shortcut::Confirm
                | Shortcut::NewGame
                | Shortcut::Offers
                | Shortcut::Take(_)
                | Shortcut::Interact
                | Shortcut::EnterSite
                | Shortcut::Discard
                | Shortcut::Quit
                | Shortcut::Fullscreen
                | Shortcut::Preferences
                // Spec 078 : les flèches ne servent qu'à l'écran de nouvelle partie.
                | Shortcut::PreviousChoice
                | Shortcut::NextChoice
                | Shortcut::PreviousStep
                | Shortcut::NextStep,
                _,
            ) => {}
        }
    }

    /// Spec 060 : une touche pendant le choix de la tactique.
    fn tactic_shortcut(&mut self, shortcut: Shortcut) {
        let legal = self.legal_actions();
        match (shortcut, &self.mode) {
            (Shortcut::Take(place), _) => {
                if let Some(tactic) = self
                    .view()
                    .tactic_choice
                    .iter()
                    .find(|(tactic, _)| tactic.number() == place)
                    .map(|(tactic, _)| *tactic)
                    .filter(|&tactic| legal.contains(&Action::ChooseTactic { tactic }))
                {
                    self.command(Command::SelectTactic(tactic));
                }
            }
            (Shortcut::Confirm, ScreenMode::ChoosingTactic(tactic)) => {
                let wanted = Action::ChooseTactic { tactic: *tactic };
                if let Some(index) = legal.iter().position(|action| *action == wanted) {
                    self.command(Command::Play(index));
                }
            }
            _ => {}
        }
    }

    /// Joue la `place`-ième (à partir de 1) des actions légales qui vérifient `wanted`, s'il y en a
    /// autant.
    fn take_nth(&mut self, place: u8, wanted: fn(&Action) -> bool) {
        if let Some(index) = self
            .legal_actions()
            .iter()
            .enumerate()
            .filter(|(_, action)| wanted(action))
            .nth(usize::from(place).saturating_sub(1))
            .map(|(index, _)| index)
        {
            self.command(Command::Play(index));
        }
    }

    /// Prend la `place`-ième (à partir de 1) unité de l'offre proposée en récompense ; sans jeton
    /// libre, ouvre le choix de l'unité à renvoyer (spec 045, C39).
    fn take_unit_reward(&mut self, place: u8) {
        let legal = self.legal_actions();
        let mut offered = Vec::new();
        for action in &legal {
            if let Action::TakeUnitReward { unit, .. } = action
                && !offered.contains(unit)
            {
                offered.push(*unit);
            }
        }
        let Some(&unit) = offered.get(usize::from(place).wrapping_sub(1)) else {
            return;
        };
        let free = Action::TakeUnitReward {
            unit,
            disband: None,
        };
        match legal.iter().position(|action| *action == free) {
            Some(index) => self.command(Command::Play(index)),
            None => self.command(Command::ChooseDisband(unit)),
        }
    }

    fn can_end_turn(&self) -> bool {
        self.legal_actions()
            .iter()
            .any(|action| matches!(action, Action::EndTurn { .. }))
    }

    /// Mode choix de la carte à défausser en cours (Échap l'annule).
    pub fn is_choosing_discard(&self) -> bool {
        matches!(
            self.mode,
            ScreenMode::ChoosingDiscard(_) | ScreenMode::ChoosingStrongDiscard(_)
        )
    }

    /// Mode choix du mana en cours (Échap l'annule, specs 022 et 025).
    pub fn is_choosing_mana(&self) -> bool {
        matches!(
            self.mode,
            ScreenMode::ChoosingMana(_)
                | ScreenMode::ChoosingBasicMana(_)
                | ScreenMode::SettingDice(_)
                | ScreenMode::ChoosingSetDiceMana(_)
                | ScreenMode::Boosting(_)
                | ScreenMode::ChoosingSpellMana(_)
                | ScreenMode::ChoosingGain(_)
                | ScreenMode::ChoosingDie(_)
        )
    }

    /// Mode fin du tour en cours (Échap l'annule, Entrée le confirme).
    pub fn is_ending_turn(&self) -> bool {
        matches!(self.mode, ScreenMode::EndingTurn(_))
    }

    /// Confirmation de l'annonce de la fin de la manche en cours (Échap l'annule, spec 023).
    pub fn is_confirming_end_of_round(&self) -> bool {
        matches!(self.mode, ScreenMode::ConfirmingEndOfRound)
    }

    /// Le dernier déplacement de la figurine, que le plateau anime (spec 075, C35) : un numéro qui
    /// change à chaque déplacement, et les cases du trajet, de départ à arrivée. Absent après un
    /// retour en arrière ou un chargement (L6).
    pub fn last_walk(&self) -> Option<(u64, &[Hex])> {
        self.walk.as_ref().map(|(id, path)| (*id, path.as_slice()))
    }

    /// La dernière disparition (spec #5) : un numéro qui change à chaque combat, et les ennemis
    /// vaincus que le plateau montrait avant ce combat. Absente sans vaincu, après un retour en
    /// arrière, un chargement ou une nouvelle partie (C5, C7).
    pub fn last_departure(&self) -> Option<(u64, &[DepartingEnemy])> {
        self.departure.last()
    }

    /// Le plateau de jeu propose déplacements et explorations, sauf en mode fin du tour.
    pub fn board_shows_moves(&self) -> bool {
        !self.is_ending_turn()
    }
}

/// Relit une sauvegarde et reprend sa partie avec un joueur humain neuf.
fn resume(path: &Path) -> Result<(GameRunner, HumanHandle), LoadError> {
    let (player, human) = human_player();
    let runner = load(path)?.resume(vec![Box::new(player)])?;
    Ok((runner, human))
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `picks` avec `item` ajouté, ou retiré s'il y était (issue privée 258).
fn toggled<T: Copy + PartialEq>(picks: arcanum_core::Picks<T>, item: T) -> arcanum_core::Picks<T> {
    let mut items = picks.to_vec();
    match items.iter().position(|&picked| picked == item) {
        Some(index) => {
            items.remove(index);
        }
        None => items.push(item),
    }
    arcanum_core::Picks::of(&items)
}

/// Coche `item`, ou le décoche s'il l'était (spec 060).
fn toggle<T: PartialEq>(picks: &mut Vec<T>, item: T) {
    match picks.iter().position(|picked| *picked == item) {
        Some(at) => {
            picks.remove(at);
        }
        None => picks.push(item),
    }
}
