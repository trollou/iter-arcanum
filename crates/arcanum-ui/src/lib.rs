//! Client du jeu : affichage et saisie uniquement (spec 002).
//! Tout ce qui se teste sans fenêtre est ici ; `main.rs` ne fait que lancer eframe.

mod about;
mod app;
mod assets;
mod board;
mod client;
mod controls;
mod departure;
mod desktop_entry;
mod enemy_text;
mod frame;
mod human;
mod icon;
mod icons;
mod illustrations;
mod keys;
mod language;
mod layout;
pub(crate) use keys::{take_label, take_label_of};
mod logging;
mod menu_bar;
mod new_game;
mod preferences;
mod saves;
mod screen;
mod session;
mod sound;

pub use about::{
    AboutWindow, REPOSITORY_URL, SOUND_CREDITS_FILE, about_window, build_version_label,
    short_commit, version_label,
};
pub use app::{IterArcanumApp, apply_theme};
pub use assets::{
    ASSETS_ENV_VAR, AssetsError, assets_candidates, catalog_or_empty, find_assets_dir,
    load_catalog, repository_assets_dir,
};
pub use board::{
    ANIMATION_SCALE_ENV_VAR, ARROW_SCROLL_SPEED, BOARD_ACTION_DIAMETER, BOARD_ACTION_KINDS,
    BOARD_BACKGROUND, BOARD_TILT, BREATHING_AMPLITUDE, BREATHING_PERIOD, BoardAction, BoardCamera,
    BoardDeparture, BoardInput, BoardMovement, BoardScreen, CameraBounds, DEFAULT_ZOOM,
    DEPARTURE_SECONDS, DepartingEnemy, EnemyAlert, ExploreGhost, FIT_ZOOM_OUT, FOG_IMAGE,
    FORBIDDEN_COLOR, FORBIDDEN_IMAGE, HeroWalk, HexDrawing, Hovered, ImageId, ImageLoad,
    ImagePaint, ImageWarnings, MAX_SCROLL_SECONDS, MAX_ZOOM, MIN_ZOOM, MoveMark, OVERLAY_MIN_ZOOM,
    Overlay, OverlayKind, RuinPip, RuinTokenMark, SLAB_DEPTH, STANDING_HEIGHT, Slab, TileOutline,
    WALK_STEP_SECONDS, ZOOM_STEP, animation_repaint, animation_scale, animation_time,
    board_action_image, board_animated, board_images, board_screen, enemy_color, feature_color,
    feature_image, figure_image, flag_image, hero_color, hero_figure_image, hero_image, hex_center,
    hex_corners, illustration_paint, image_paint, image_uri, night_veil, point_to_hex,
    relief_image, svg_size, terrain_color, terrain_image, time_of_day_image,
};
pub use client::{ClientSession, QuitWindow, WindowRequest, clock_seed};
pub use controls::{
    ControlButton, Controls, LoadLine, LoadWindow, Notice, controls_screen, load_window_screen,
};
pub use desktop_entry::{
    DESKTOP_MARK, DesktopError, DesktopInstall, desktop_entry, desktop_file_path,
    install_desktop_entry, theme_icon_path,
};
pub use enemy_text::{token_lines, token_name, token_names, token_values};
pub use frame::{FrameOutcome, FrameReport, MAX_STEPS_PER_FRAME, run_frame, run_frame_limited};
pub use human::{HumanHandle, HumanPlayer, HumanRequest, human_player};
pub use icon::{
    APP_ID, ICO_SIZES, ICON_SVG, IconError, WINDOW_ICON_SIZE, icon_ico_path, render_icon, viewport,
    window_icon,
};
pub use icons::{
    CRYSTAL_EMPTY_IMAGE, Icon, IconButton, IconValue, ValuePiece, attack_icon, block_icon,
    crystal_image, hero_emblem, icon_value, ui_images,
};
pub use illustrations::{
    Illustration, IllustrationFamily, IllustrationIndex, card_illustration, hero_illustration,
    known_illustrations, unit_illustration,
};
pub use keys::{
    KEY_BINDINGS, KeyBinding, KeyModifier, KeyPlatform, hover_text, key_label, key_label_on,
    shortcut_label,
};
#[cfg(windows)]
pub use language::windows_system_language;
pub use language::{
    FALLBACK_LANGUAGE, UNIX_LOCALE_VARIABLES, apple_system_language, launch_catalog,
    locale_to_code, requested_language, system_language, unix_system_language,
    windows_language_id_to_code,
};
#[cfg(target_os = "macos")]
pub use language::{macos_preferred_language, macos_system_language};
pub use layout::{
    BandUv, CARD_GAP, CARD_HEIGHT, CARD_WIDTH, FanLayout, MIN_SECTION_WIDTH, SECTION_GAP,
    SKILLS_WIDTH, band_uv, bottom_section_widths, card_height, cards_per_row, fan_layout,
    row_width, section_gap, section_widths,
};
pub use logging::{
    ArcanumLogger, KEPT_PREVIOUS_LOGS, LOG_DIR_ENV_VAR, LOG_FILE_NAME, LOG_LEVEL_ENV_VAR,
    LevelChoice, LogDirChoice, LogDirSource, PreparedLogFile, choose_logs_dir, format_line,
    init_logging, install_panic_hook, is_enabled, macos_logs_dir, parse_level, prepare_log_file,
    rotate_log_files, startup_message, user_base_dir, user_logs_dir,
};
pub use menu_bar::{MenuBar, MenuEntry, MenuItem, MenuState, menu_bar};
pub use new_game::{
    AdvancedPanel, DummyHeroItem, DummyOption, HeroChoice, HeroSheet, InvalidSeed, NewGameContext,
    NewGameForm, NewGameScreen, NewGameSheet, NewGameStep, OptionsSheet, OwnCardSheet,
    SEED_MAX_CHARS, ScenarioChoice, ScenarioSheet, SheetFact, SheetSection, StartButton, StepTab,
    dummy_hero, form_config, form_of, hero_sheet, new_game_screen, parse_seed, scenario_sheet,
    sheet_keys_of_hero, sheet_keys_of_scenario,
};
pub use preferences::{
    CONFIG_DIR_ENV_VAR, LanguageChoices, LanguageOption, PreferencesWindow, VolumeSlider,
    choose_config_dir, launch_preferences, preferences_window, user_config_dir, volume_value_label,
};
pub use saves::{
    SAVE_DIR_ENV_VAR, choose_saves_dir, display_date, save_stem, saved_at, user_data_dir,
};
pub use screen::{
    AbilityButton, ActionButton, AltarWindow, AnnouncePrompt, ArtifactPlay, BannerTag,
    BoardActionButton, BoardActionKind, BoostChoice, Button, CardChoiceWindow, CardInfo,
    CastChoice, CastDestination, CastPicks, Clock, CombatChoice, CombatScreen, Command,
    CrystalButton, CrystalChoice, CrystalGroup, DamageGroup, DestinationChoice, DiscardChoice,
    DiscardPrompt, DiscardToggle, DiscardWindow, DummyCrystals, DummySection, EffectButton,
    EffectFace, EndTurnChoice, EndTurnPrompt, EnemyCard, GainChoice, GameScreen, GladeMenu,
    GladeOption, HandCard, HeroPortrait, IconLine, InProgressPanel, InventoryLine, LevelCard,
    LevelChoiceWindow, ManaChoice, ManaPrompt, MeditationCard, MeditationWindow, OfferCardView,
    OfferColumn, OffersWindow, PhaseStep, PlayedLine, PreparationWindow, PurchaseButton,
    ReclaimLine, ReclaimMenu, ReclaimWindow, ScoreWindow, ScreenMode, ScreenPhase, SetDiceChoice,
    SetDicePrompt, SiteMenu, SkillChoice, SkillLine, SkillOffer, SkillsSection, SourceDie,
    SourceLine, StepState, StrongButton, TacticCard, TacticDecision, TacticLine, TacticWindow,
    TokenButton, TokenChoice, TokenLine, TokenView, UnitLine, UnitsSection, combat_screen,
    discard_window, dummy_discard_window, effect_text, game_screen, offers_window,
    offers_window_in,
};
pub use session::{GameSession, Shortcut};
pub use sound::{game_outcome, situation_of};
