//! Session du client (spec 065) : l'écran de nouvelle partie seul au lancement, ou une partie en
//! cours avec éventuellement la fenêtre de nouvelle partie par-dessus.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use arcanum_audio::{SoundCategory, SoundSituation};
use arcanum_core::{GameConfig, Hero, PlayerEvent, PlayerId, ScenarioId, filter_event};
use arcanum_i18n::{Catalog, Language};
use arcanum_persistance::preferences::{Preferences, Volumes, save as save_preferences_file};
use arcanum_persistance::{SaveEntry, SaveError, SaveSummary, list, load, save};
use chrono::{DateTime, Local};

use crate::new_game::{SEED_MAX_CHARS, dummy_hero, form_config, form_of};
use crate::{
    Button, Command, GameScreen, GameSession, LanguageChoices, LoadWindow, MenuBar, MenuState,
    NewGameContext, NewGameForm, NewGameScreen, NewGameStep, Notice, PreferencesWindow, Shortcut,
    load_window_screen, new_game_screen, parse_seed, preferences_window, save_stem, shortcut_label,
    situation_of,
};

/// Graine d'une partie au hasard : les 64 bits de poids faible des nanosecondes de l'horloge
/// (spec 002). Jamais affichée.
pub fn clock_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos() as u64)
}

/// Ce que la session demande à la fenêtre (spec 074) : l'application vide la file à chaque image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowRequest {
    /// Recadrer le plateau de jeu sur la partie.
    Recenter,
    /// Passer en plein écran (`true`) ou en sortir.
    Fullscreen(bool),
    /// Fermer la fenêtre, sans rien écrire (Q1).
    Close,
    /// Recharger le catalogue de cette langue et redonner le titre de la fenêtre (spec 080).
    Language(Language),
    /// Spec 081 : les volumes de la session, à passer au lecteur.
    Volumes(Volumes),
    /// Spec 081 : l'échantillon d'un type de son (curseur tenu), ou son arrêt (`None`).
    Preview(Option<SoundCategory>),
}

/// Confirmation de Quitter quand le tour a avancé depuis la sauvegarde automatique (Q1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuitWindow {
    pub title: String,
    pub text: String,
    /// « Quitter » (Entrée).
    pub quit: Button,
    /// « Annuler » (Échap).
    pub cancel: Button,
}

/// La sauvegarde automatique trouvée au lancement, si sa partie n'est pas terminée (C15).
#[derive(Clone, Debug)]
struct Autosave {
    path: PathBuf,
    summary: SaveSummary,
}

/// Où en est le son dans le journal de la partie (spec 081) : la partie et la version de son
/// journal, et le nombre d'événements déjà entendus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct HeardCursor {
    generation: u64,
    epoch: u64,
    len: usize,
}

/// L'application vue par l'UI : une partie ou aucune, et la fenêtre de nouvelle partie.
pub struct ClientSession {
    game: Option<GameSession>,
    /// Fenêtre de nouvelle partie ouverte, avec ce qui y est coché et saisi. Toujours ouverte
    /// sans partie.
    form: Option<NewGameForm>,
    /// Échec affiché dans la fenêtre : sauvegarde de la partie abandonnée (L2, `save.failed`)
    /// ou chargement au lancement (`load.failed`, spec 024).
    notice: Option<&'static str>,
    /// Fenêtre de chargement ouverte depuis l'écran de lancement (C17).
    launch_load: Option<Vec<SaveEntry>>,
    autosave: Option<Autosave>,
    saves_dir: Option<PathBuf>,
    clock: fn() -> DateTime<Local>,
    seed_source: fn() -> u64,
    /// Change à chaque partie lancée ou chargée : l'application recadre alors le plateau de jeu.
    generation: u64,
    /// La fenêtre est en plein écran, d'après elle (C11).
    fullscreen: bool,
    /// Demandes à transmettre à la fenêtre (spec 074).
    requests: Vec<WindowRequest>,
    /// Confirmation de Quitter ouverte (Q1).
    quit_confirm: bool,
    /// Fermeture décidée : la demande du système qui suit passe sans confirmation.
    closing: bool,
    /// Préférences du joueur (spec 080) et le fichier où les écrire ; sans fichier, elles valent
    /// pour la session seulement (L2).
    preferences: Preferences,
    preferences_path: Option<PathBuf>,
    /// Langues proposées par la fenêtre des préférences.
    languages: LanguageChoices,
    /// Fenêtre des préférences ouverte.
    preferences_open: bool,
    /// Le dernier enregistrement des préférences a échoué (C21).
    preferences_not_saved: bool,
    /// Spec 081 : les événements déjà entendus, et si la partie suivante s'entend depuis son
    /// début (partie neuve) plutôt que depuis la fin de son journal (partie chargée).
    heard: HeardCursor,
    hear_from_start: bool,
    /// Spec 081 : le curseur de volume tenu, dont l'échantillon joue.
    previewing: Option<SoundCategory>,
    /// Spec 081 : le lecteur audio a trouvé une sortie (sinon le jeu reste muet, L1).
    audio_output: bool,
}

impl ClientSession {
    /// Au lancement : l'écran de nouvelle partie, avec « Continuer » si la sauvegarde
    /// automatique contient une partie non terminée (Q1).
    pub fn launch(saves_dir: Option<PathBuf>) -> Self {
        let last = saves_dir.as_deref().and_then(readable_autosave);
        // Spec 078, Q1 : scénario et héros de la dernière partie, terminée ou non (C16) ; le
        // fantôme repart de son défaut (spec 061, C41).
        let form = last
            .as_ref()
            .and_then(|(_, _, _, config)| config.as_ref())
            .map_or_else(NewGameForm::default, form_of);
        let autosave = last
            .filter(|(_, _, finished, _)| !finished)
            .map(|(path, summary, _, _)| Autosave { path, summary });
        Self {
            game: None,
            form: Some(form),
            notice: None,
            launch_load: None,
            autosave,
            saves_dir,
            clock: Local::now,
            seed_source: clock_seed,
            generation: 0,
            fullscreen: false,
            requests: Vec::new(),
            quit_confirm: false,
            closing: false,
            preferences: Preferences::default(),
            preferences_path: None,
            languages: LanguageChoices::default(),
            preferences_open: false,
            preferences_not_saved: false,
            heard: HeardCursor::default(),
            hear_from_start: false,
            previewing: None,
            audio_output: true,
        }
    }

    /// Spec 081 : le résultat de la création du lecteur audio, pour la fenêtre (C23, L1).
    pub fn with_audio_output(mut self, available: bool) -> Self {
        self.audio_output = available;
        self
    }

    /// Les préférences lues au lancement, leur fichier et les langues proposées (spec 080).
    pub fn with_preferences(
        mut self,
        preferences: Preferences,
        path: Option<PathBuf>,
        languages: LanguageChoices,
    ) -> Self {
        self.preferences = preferences;
        self.preferences_path = path;
        self.languages = languages;
        self
    }

    /// Les préférences de la session.
    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    /// La fenêtre des préférences, si elle est ouverte (C15).
    pub fn preferences_window(&self, catalog: &Catalog) -> Option<PreferencesWindow> {
        self.preferences_open.then(|| {
            preferences_window(
                &self.preferences,
                &self.languages,
                self.preferences_not_saved,
                self.audio_output,
                catalog,
            )
        })
    }

    /// Commandes de la fenêtre des préférences ; `true` si traitée.
    fn preferences_command(&mut self, command: Command) -> bool {
        match command {
            Command::OpenPreferences => self.preferences_open = !self.preferences_open,
            Command::ClosePreferences => self.preferences_open = false,
            Command::SetLanguage(choice) => {
                let Some(language) = self.languages.language_of(choice) else {
                    return true;
                };
                // Le choix du joueur, tel quel ; « Langue du système » s'écrit `null` (C18).
                self.preferences.language = choice.map(|_| language.0.clone());
                self.requests.push(WindowRequest::Language(language));
                self.save_preferences();
            }
            Command::SetFullscreenAtLaunch(on) => {
                self.preferences.fullscreen_at_launch = on;
                self.save_preferences();
            }
            Command::SetAnimations(on) => {
                self.preferences.animations = on;
                self.save_preferences();
            }
            Command::SetVolume(category, volume) => {
                let volumes = &mut self.preferences.volumes;
                let slot = match category {
                    SoundCategory::Master => &mut volumes.master,
                    SoundCategory::Music => &mut volumes.music,
                    SoundCategory::Ambience => &mut volumes.ambience,
                    SoundCategory::Effects => &mut volumes.effects,
                };
                *slot = volume;
                self.requests.push(WindowRequest::Volumes(*volumes));
                // L'échantillon une seule fois par glissé (C21).
                if self.previewing.is_none() {
                    self.previewing = Some(category);
                    self.requests.push(WindowRequest::Preview(Some(category)));
                }
            }
            // Écart assumé avec la spec 080 : le fichier n'est écrit qu'au lâcher du curseur.
            Command::SaveVolumes => {
                self.save_preferences();
                if self.previewing.take().is_some() {
                    self.requests.push(WindowRequest::Preview(None));
                }
            }
            _ => return false,
        }
        true
    }

    /// Écrit le fichier des préférences ; un échec reste pour la session et s'affiche (C21, L2).
    fn save_preferences(&mut self) {
        let result = match &self.preferences_path {
            None => Err("aucun dossier de configuration".to_owned()),
            Some(path) => save_preferences_file(path, &self.preferences)
                .map_err(|error| format!("{} : {error}", path.display())),
        };
        match result {
            Ok(()) => self.preferences_not_saved = false,
            Err(cause) => {
                log::error!("préférences non enregistrées : {cause}");
                self.preferences_not_saved = true;
            }
        }
    }

    pub fn with_clock(mut self, clock: fn() -> DateTime<Local>) -> Self {
        self.clock = clock;
        self
    }

    /// Source de la graine d'une partie au hasard ; fixée par les tests.
    pub fn with_seed_source(mut self, seed_source: fn() -> u64) -> Self {
        self.seed_source = seed_source;
        self
    }

    /// Une partie déjà en cours (tests, parties préparées), fenêtre fermée.
    pub fn with_game(mut self, game: GameSession) -> Self {
        self.set_game(game);
        self
    }

    pub fn game(&self) -> Option<&GameSession> {
        self.game.as_ref()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Texte du champ de la graine, fenêtre ouverte.
    pub fn seed_text(&self) -> Option<&str> {
        self.form.as_ref().map(|form| form.seed.as_str())
    }

    /// Saisie dans le champ de la graine, coupée à 20 caractères (L7).
    pub fn set_seed_text(&mut self, text: &str) {
        if let Some(form) = &mut self.form {
            form.seed = text.chars().take(SEED_MAX_CHARS).collect();
        }
    }

    /// L'écran de la partie en cours ; la fenêtre de fin s'efface devant la fenêtre de nouvelle
    /// partie ouverte depuis elle (C14), pour ne pas la recouvrir.
    pub fn game_screen(&self, catalog: &Catalog) -> Option<GameScreen> {
        let mut screen = self.game.as_ref()?.screen(catalog);
        if self.form.is_some() {
            screen.score_window = None;
        }
        Some(screen)
    }

    /// L'état de la fenêtre, donné par l'application à chaque image (C11).
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    /// Les demandes à la fenêtre, retirées de la file.
    pub fn take_window_requests(&mut self) -> Vec<WindowRequest> {
        std::mem::take(&mut self.requests)
    }

    /// La croix de la fenêtre (C16) : `true` si elle peut se fermer, sinon la confirmation
    /// s'ouvre et la fermeture est à annuler.
    pub fn window_close_requested(&mut self) -> bool {
        if self.closing || !self.needs_quit_confirmation() {
            return true;
        }
        self.quit_confirm = true;
        false
    }

    /// La confirmation de Quitter, si elle est ouverte.
    pub fn quit_window(&self, catalog: &Catalog) -> Option<QuitWindow> {
        self.quit_confirm.then(|| QuitWindow {
            title: catalog.text("quit.confirm.title"),
            text: catalog.text("quit.confirm.text"),
            quit: Button {
                label: catalog.text("quit.confirm.quit"),
                command: Command::ConfirmQuit,
                shortcut: shortcut_label(Shortcut::Confirm, catalog),
            },
            cancel: Button {
                label: catalog.text("quit.confirm.cancel"),
                command: Command::CancelQuit,
                shortcut: shortcut_label(Shortcut::Cancel, catalog),
            },
        })
    }

    /// Quitter perdrait ce qui a été joué depuis la sauvegarde automatique, seule reprise par
    /// « Continuer » (Q1) ; une partie terminée n'a plus rien à perdre (C15).
    fn needs_quit_confirmation(&self) -> bool {
        self.game
            .as_ref()
            .is_some_and(|game| !game.is_finished() && game.advanced_since_autosave())
    }

    /// Quitter (C13, C14) : la confirmation d'abord si le tour a avancé.
    fn quit(&mut self) {
        if self.needs_quit_confirmation() {
            self.quit_confirm = true;
        } else {
            self.close();
        }
    }

    /// Ferme sans rien écrire (Q1).
    fn close(&mut self) {
        self.quit_confirm = false;
        self.closing = true;
        self.requests.push(WindowRequest::Close);
    }

    /// Commandes de la fenêtre et de la confirmation de Quitter ; `true` si traitée.
    fn window_command(&mut self, command: Command) -> bool {
        // L4 : la confirmation ouverte, seuls ses boutons agissent.
        if self.quit_confirm {
            match command {
                Command::ConfirmQuit => self.close(),
                Command::CancelQuit => self.quit_confirm = false,
                _ => {}
            }
            return true;
        }
        if self.preferences_command(command) {
            return true;
        }
        match command {
            Command::Recenter => self.requests.push(WindowRequest::Recenter),
            Command::ToggleFullscreen => {
                self.requests
                    .push(WindowRequest::Fullscreen(!self.fullscreen));
            }
            Command::Quit => self.quit(),
            _ => return false,
        }
        true
    }

    /// La barre de menu de la partie en cours (spec 074) ; aucune au lancement (Q5).
    pub fn menu_bar(&self, catalog: &Catalog) -> Option<MenuBar> {
        let state = MenuState {
            new_game_open: self.form.is_some(),
            fullscreen: self.fullscreen,
        };
        Some(self.game.as_ref()?.menu_bar_with(state, catalog))
    }

    /// L'écran de nouvelle partie, s'il est ouvert et répond (pas de fenêtre de chargement
    /// par-dessus).
    pub fn new_game_screen(&self, catalog: &Catalog) -> Option<NewGameScreen> {
        if self.launch_load.is_some() {
            return None;
        }
        self.form_screen(catalog)
    }

    /// L'écran de nouvelle partie à dessiner, et s'il répond : la fenêtre de chargement ouverte
    /// depuis lui le laisse dessiné derrière elle (spec 078, L2).
    pub fn new_game_page(&self, catalog: &Catalog) -> Option<(NewGameScreen, bool)> {
        Some((self.form_screen(catalog)?, self.launch_load.is_none()))
    }

    fn form_screen(&self, catalog: &Catalog) -> Option<NewGameScreen> {
        let form = self.form.as_ref()?;
        let context = match &self.game {
            None => NewGameContext::Launch {
                autosave: self.autosave.as_ref().map(|auto| auto.summary.clone()),
            },
            Some(game) => NewGameContext::InGame {
                finished: game.is_finished(),
            },
        };
        let mut screen = new_game_screen(form, &context, catalog);
        screen.notice = self.notice.map(|key| catalog.text(key));
        Some(screen)
    }

    /// Fenêtre de chargement : celle du lancement, ou celle de la partie (spec 024).
    pub fn load_window(&self, catalog: &Catalog) -> Option<LoadWindow> {
        match (&self.launch_load, &self.game) {
            (Some(entries), _) => Some(load_window_screen(entries, catalog)),
            (None, Some(game)) => game.load_window(catalog),
            (None, None) => None,
        }
    }

    /// Spec 081 : les événements filtrés pour le joueur apparus depuis le dernier appel. Une
    /// partie chargée, un retour en arrière, ne rejouent rien (C6 à C9).
    pub fn take_heard_events(&mut self) -> Vec<PlayerEvent> {
        let Some(game) = &self.game else {
            return Vec::new();
        };
        let entries = game.runner().log().entries();
        let now = HeardCursor {
            generation: self.generation,
            epoch: game.log_epoch(),
            len: entries.len(),
        };
        let from = if now.generation != self.heard.generation {
            // Une autre partie : neuve, entendue depuis son début ; chargée, rien d'ancien.
            if std::mem::take(&mut self.hear_from_start) {
                0
            } else {
                now.len
            }
        } else if now.epoch != self.heard.epoch || now.len < self.heard.len {
            now.len
        } else {
            self.heard.len
        };
        self.heard = now;
        entries[from..]
            .iter()
            .map(|entry| filter_event(&entry.event, PlayerId::SOLO))
            .collect()
    }

    /// Spec 081 : la situation sonore de l'écran (C10, C11).
    pub fn sound_situation(&self) -> SoundSituation {
        let view = self.game.as_ref().map(GameSession::view);
        situation_of(view.as_ref(), self.form.is_some())
    }

    /// Avance la partie en cours pendant une image ; `true` s'il faut redemander une image.
    pub fn run_frame(&mut self) -> bool {
        self.game.as_mut().is_some_and(GameSession::run_frame)
    }

    pub fn command(&mut self, command: Command) {
        // Spec 080, L4 : la fenêtre des préférences ouverte, la partie et l'écran de lancement
        // n'agissent pas.
        if self.window_command(command) || self.preferences_open {
            return;
        }
        if self.launch_load.is_some() {
            self.launch_load_command(command);
            return;
        }
        if self.form.is_some() {
            self.form_command(command);
            return;
        }
        if command == Command::OpenNewGame {
            self.open_new_game();
            return;
        }
        if let Some(game) = &mut self.game {
            game.command(command);
        }
    }

    pub fn shortcut(&mut self, shortcut: Shortcut) {
        // Ctrl+Q et F11 agissent partout, lancement compris (Q5) ; Entrée et Échap répondent à
        // la confirmation de Quitter, qui retient les autres touches (L4).
        let window = match (self.quit_confirm, shortcut) {
            (true, Shortcut::Confirm) => Some(Command::ConfirmQuit),
            (true, Shortcut::Cancel) => Some(Command::CancelQuit),
            (true, _) => return,
            (false, Shortcut::Quit) => Some(Command::Quit),
            (false, Shortcut::Fullscreen) => Some(Command::ToggleFullscreen),
            (false, Shortcut::Preferences) => Some(Command::OpenPreferences),
            // Spec 080 : Échap ferme la fenêtre des préférences avant toute autre (C22).
            (false, Shortcut::Cancel) if self.preferences_open => Some(Command::ClosePreferences),
            (false, _) => None,
        };
        if let Some(command) = window {
            self.window_command(command);
            return;
        }
        // L4 : les autres touches ne jouent pas derrière la fenêtre des préférences.
        if self.preferences_open {
            return;
        }
        if self.launch_load.is_some() {
            if shortcut == Shortcut::Cancel {
                self.launch_load = None;
            }
            return;
        }
        if let Some(form) = &self.form {
            let step = form.step;
            match shortcut {
                Shortcut::PreviousChoice => self.move_choice(-1),
                Shortcut::NextChoice => self.move_choice(1),
                Shortcut::PreviousStep => self.form_command(Command::ChooseStep(previous(step))),
                Shortcut::NextStep => self.form_command(Command::ChooseStep(next(step))),
                Shortcut::Confirm => self.form_command(Command::StartNewGame),
                Shortcut::Cancel => self.form_command(Command::CancelNewGame),
                // Au lancement, Ctrl+O fait comme « Charger… » (spec 024).
                Shortcut::OpenLoad => self.form_command(Command::OpenLoad),
                _ => {}
            }
            return;
        }
        if shortcut == Shortcut::NewGame {
            self.open_new_game();
            return;
        }
        if let Some(game) = &mut self.game {
            game.shortcut(shortcut);
        }
    }

    /// Ctrl+N ou « Nouvelle partie… » : hors mode choix (L4), pré-remplie avec la partie en
    /// cours, graine vide (C11).
    fn open_new_game(&mut self) {
        let Some(game) = &self.game else {
            return;
        };
        if !game.can_open_new_game() {
            return;
        }
        self.form = Some(
            game.config()
                .as_ref()
                .map_or_else(NewGameForm::default, form_of),
        );
        self.notice = None;
    }

    fn form_command(&mut self, command: Command) {
        let Some(form) = &mut self.form else {
            return;
        };
        match command {
            Command::ChooseHero(hero) => {
                form.hero = hero;
                // Spec 061, C41 : le héros du fantôme n'est jamais celui du joueur.
                form.dummy_hero = dummy_hero(form);
            }
            Command::ToggleDummy => form.dummy = !form.dummy,
            Command::ChooseDummyHero(choice) => form.dummy_hero = choice,
            Command::ChooseScenario(scenario) => form.scenario = scenario,
            Command::ChooseStep(step) => form.step = step,
            // C14 : refermer le volet est refusé tant que la graine est invalide.
            Command::ToggleAdvanced => {
                if !form.advanced_open || parse_seed(&form.seed).is_ok() {
                    form.advanced_open = !form.advanced_open;
                }
            }
            Command::StartNewGame => self.start(),
            // L3 : au lancement, il n'y a pas de partie où revenir.
            Command::CancelNewGame if self.game.is_some() => {
                self.form = None;
                self.notice = None;
            }
            Command::ContinueGame if self.game.is_none() => self.continue_autosave(),
            // L5 : « Charger… » ferme la fenêtre de nouvelle partie.
            Command::OpenLoad if self.game.is_none() => {
                self.launch_load = Some(self.saves_dir.as_deref().map(list).unwrap_or_default());
            }
            _ => {}
        }
    }

    /// ↑ ou ↓ : coche l'élément voisin de la liste de l'étape, sans boucler (C18).
    fn move_choice(&mut self, offset: isize) {
        let Some(form) = &mut self.form else {
            return;
        };
        let neighbour =
            |index: Option<usize>| index.and_then(|index| index.checked_add_signed(offset));
        match form.step {
            NewGameStep::Scenario => {
                let index = ScenarioId::ALL.iter().position(|s| *s == form.scenario);
                if let Some(&scenario) = neighbour(index).and_then(|i| ScenarioId::ALL.get(i)) {
                    form.scenario = scenario;
                }
            }
            NewGameStep::Hero => {
                let index = Hero::ALL.iter().position(|h| *h == form.hero);
                if let Some(&hero) = neighbour(index).and_then(|i| Hero::ALL.get(i)) {
                    form.hero = hero;
                }
            }
            NewGameStep::Options => {}
        }
    }

    fn launch_load_command(&mut self, command: Command) {
        match command {
            Command::CloseLoad => self.launch_load = None,
            Command::Load(index) => {
                let entry = self
                    .launch_load
                    .take()
                    .and_then(|entries| entries.into_iter().nth(index));
                if let Some(entry) = entry {
                    self.load_from(&entry.path);
                }
            }
            _ => {}
        }
    }

    /// « Commencer » : sauvegarde d'abord la partie qu'on abandonnerait (Q2), puis lance la
    /// partie du formulaire.
    fn start(&mut self) {
        let Some(form) = self.form.clone() else {
            return;
        };
        let Ok(seed) = parse_seed(&form.seed) else {
            return;
        };
        let previous = match self.keep_previous_game() {
            Ok(previous) => previous,
            Err(error) => {
                log::error!("sauvegarde de la partie abandonnée impossible : {error}");
                self.notice = Some("save.failed");
                return;
            }
        };
        let config = form_config(&form, seed.unwrap_or_else(self.seed_source));
        let mut game = GameSession::new(config);
        if let Some(name) = previous {
            game.set_notice(Notice::PreviousSaved(name));
        }
        self.set_game(game);
        self.hear_from_start = true;
    }

    /// Sauvegarde ce que la nouvelle partie ferait perdre : la partie en cours non terminée
    /// (C13), ou au lancement la partie non terminée de la sauvegarde automatique (C18), que le
    /// premier tour de la nouvelle partie va remplacer. Sans dossier, rien (L1). Rend le nom de
    /// la sauvegarde d'une partie abandonnée.
    fn keep_previous_game(&self) -> Result<Option<String>, SaveError> {
        let Some(dir) = &self.saves_dir else {
            return Ok(None);
        };
        match &self.game {
            Some(game) if !game.is_finished() => game.save_copy().map(Some),
            Some(_) => Ok(None),
            None => {
                if let Some(autosave) = &self.autosave {
                    let game =
                        load(&autosave.path).map_err(|error| SaveError::Io(error.to_string()))?;
                    fs::create_dir_all(dir).map_err(|error| SaveError::Io(error.to_string()))?;
                    save(dir, &save_stem(&(self.clock)()), &game)?;
                }
                Ok(None)
            }
        }
    }

    fn continue_autosave(&mut self) {
        if let Some(path) = self.autosave.as_ref().map(|auto| auto.path.clone()) {
            self.load_from(&path);
        }
    }

    /// Charge une sauvegarde ; en cas d'échec, l'écran de nouvelle partie reste (spec 024).
    fn load_from(&mut self, path: &Path) {
        match GameSession::from_save(path) {
            Ok(game) => {
                log::info!("partie chargée : {}", path.display());
                self.set_game(game);
            }
            Err(error) => {
                log::error!("chargement impossible de {} : {error}", path.display());
                self.notice = Some("load.failed");
            }
        }
    }

    fn set_game(&mut self, game: GameSession) {
        self.game = Some(
            game.with_saves_dir(self.saves_dir.clone())
                .with_clock(self.clock),
        );
        self.form = None;
        self.notice = None;
        self.launch_load = None;
        self.autosave = None;
        self.generation += 1;
    }
}

/// La sauvegarde automatique de `dir`, si elle se relit : son chemin, son résumé, et si sa
/// partie est terminée.
fn readable_autosave(dir: &Path) -> Option<(PathBuf, SaveSummary, bool, Option<GameConfig>)> {
    let entry = list(dir).into_iter().find(SaveEntry::is_auto)?;
    let summary = entry.summary.ok()?;
    let game = GameSession::from_save(&entry.path).ok()?;
    Some((entry.path, summary, game.is_finished(), game.config()))
}

/// L'étape d'avant ou d'après, sans boucler (C18).
fn previous(step: NewGameStep) -> NewGameStep {
    match step {
        NewGameStep::Scenario | NewGameStep::Hero => NewGameStep::Scenario,
        NewGameStep::Options => NewGameStep::Hero,
    }
}

fn next(step: NewGameStep) -> NewGameStep {
    match step {
        NewGameStep::Scenario => NewGameStep::Hero,
        NewGameStep::Hero | NewGameStep::Options => NewGameStep::Options,
    }
}
