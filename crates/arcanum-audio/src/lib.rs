//! Son du jeu (spec 081, ADR 0010) : le son écoute la partie, il ne la change jamais.
//!
//! - `director` (pur, testé) : situation et événements nouveaux → ordres de son ;
//! - `bank` : les fichiers trouvés dans `assets/sounds/` ;
//! - `player` : exécute les ordres sur la carte son (Kira), ou ne fait rien.

mod bank;
mod cues;
mod director;
mod licenses;
mod player;

use std::time::Duration;

use arcanum_core::TimeOfDay;

pub use bank::{SOUNDS_DIR, SoundBank};
pub use director::{AudioDirector, fade};
pub use licenses::{
    FREE_LICENSES, LICENSES_FILE, LicenseReport, NON_COMMERCIAL_LICENSES, check_licenses,
};
pub use player::{AudioPlayer, KiraPlayer, PlayerError, SilentPlayer};

/// Morceau en boucle choisi selon la situation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Music {
    Title,
    Day,
    Night,
    Combat,
}

impl Music {
    pub const ALL: [Music; 4] = [Music::Title, Music::Day, Music::Night, Music::Combat];

    /// Nom du fichier, sans extension, dans `music/`.
    pub fn file_stem(self) -> &'static str {
        match self {
            Music::Title => "title",
            Music::Day => "day",
            Music::Night => "night",
            Music::Combat => "combat",
        }
    }
}

/// Fond sonore en boucle sous la musique.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ambience {
    Day,
    Night,
}

impl Ambience {
    pub const ALL: [Ambience; 2] = [Ambience::Day, Ambience::Night];

    /// Nom du fichier, sans extension, dans `ambience/`.
    pub fn file_stem(self) -> &'static str {
        match self {
            Ambience::Day => "day",
            Ambience::Night => "night",
        }
    }

    /// L'ambiance du moment de la journée.
    pub fn of(time_of_day: TimeOfDay) -> Self {
        match time_of_day {
            TimeOfDay::Day => Ambience::Day,
            TimeOfDay::Night => Ambience::Night,
        }
    }
}

/// Effet sonore, joué une fois. L'ordre des variantes est celui de la table de la spec, qui fait
/// la priorité quand une image en a trop (C15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cue {
    Rooster,
    Owl,
    Footstep,
    TileRevealed,
    CombatStart,
    Block,
    Miss,
    Wound,
    EnemyDefeated,
    CombatEnd,
    Spell,
    Conquest,
    LevelUp,
    Recruit,
    KnockedOut,
    LastTurn,
    Victory,
    Defeat,
    Preview,
}

impl Cue {
    pub const ALL: [Cue; 19] = [
        Cue::Rooster,
        Cue::Owl,
        Cue::Footstep,
        Cue::TileRevealed,
        Cue::CombatStart,
        Cue::Block,
        Cue::Miss,
        Cue::Wound,
        Cue::EnemyDefeated,
        Cue::CombatEnd,
        Cue::Spell,
        Cue::Conquest,
        Cue::LevelUp,
        Cue::Recruit,
        Cue::KnockedOut,
        Cue::LastTurn,
        Cue::Victory,
        Cue::Defeat,
        Cue::Preview,
    ];

    /// Nom du fichier, sans extension ni numéro de variante, dans `effects/` (snake_case).
    pub fn file_stem(self) -> &'static str {
        match self {
            Cue::Rooster => "rooster",
            Cue::Owl => "owl",
            Cue::Footstep => "footstep",
            Cue::TileRevealed => "tile_revealed",
            Cue::CombatStart => "combat_start",
            Cue::Block => "block",
            Cue::Miss => "miss",
            Cue::Wound => "wound",
            Cue::EnemyDefeated => "enemy_defeated",
            Cue::CombatEnd => "combat_end",
            Cue::Spell => "spell",
            Cue::Conquest => "conquest",
            Cue::LevelUp => "level_up",
            Cue::Recruit => "recruit",
            Cue::KnockedOut => "knocked_out",
            Cue::LastTurn => "last_turn",
            Cue::Victory => "victory",
            Cue::Defeat => "defeat",
            Cue::Preview => "preview",
        }
    }
}

/// Type de son : chacun sa piste et son volume, sous le volume général.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SoundCategory {
    Master,
    Music,
    Ambience,
    Effects,
}

/// Issue d'une partie finie, pour le jingle (C28, C29).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Outcome {
    Victory,
    Defeat,
}

/// Ce que l'écran montre, pour choisir musique et ambiance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundSituation {
    /// Écran de lancement, ou écran de nouvelle partie ouvert en cours de partie.
    Title,
    Exploration(TimeOfDay),
    Combat(TimeOfDay),
    GameOver(TimeOfDay, Outcome),
}

impl SoundSituation {
    /// La musique de la situation (aucune en fin de partie).
    pub fn music(self) -> Option<Music> {
        match self {
            SoundSituation::Title => Some(Music::Title),
            SoundSituation::Exploration(TimeOfDay::Day) => Some(Music::Day),
            SoundSituation::Exploration(TimeOfDay::Night) => Some(Music::Night),
            SoundSituation::Combat(_) => Some(Music::Combat),
            SoundSituation::GameOver(..) => None,
        }
    }

    /// L'ambiance de la situation : celle du moment, sauf sur l'écran de lancement (Q5).
    pub fn ambience(self) -> Option<Ambience> {
        match self {
            SoundSituation::Title => None,
            SoundSituation::Exploration(time)
            | SoundSituation::Combat(time)
            | SoundSituation::GameOver(time, _) => Some(Ambience::of(time)),
        }
    }
}

/// Durées d'un fondu enchaîné : sortie de l'ancien morceau, entrée du nouveau.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fade {
    pub fade_out: Duration,
    pub fade_in: Duration,
}

impl Fade {
    pub const fn millis(fade_out: u64, fade_in: u64) -> Self {
        Self {
            fade_out: Duration::from_millis(fade_out),
            fade_in: Duration::from_millis(fade_in),
        }
    }
}

/// Ce que le directeur demande au lecteur.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioCommand {
    /// Changer de musique (`None` : silence), avec fondu.
    Music {
        music: Option<Music>,
        fade: Fade,
    },
    /// Changer d'ambiance (`None` : silence), avec fondu.
    Ambience {
        ambience: Option<Ambience>,
        fade: Fade,
    },
    /// Jouer un effet après `delay`. `nth` compte les effets `cue` déjà joués : la banque en
    /// tire la variante, à tour de rôle (C16).
    Cue {
        cue: Cue,
        nth: usize,
        delay: Duration,
    },
    /// Rejouer un effet en boucle (échantillon des curseurs, C21, C30), jusqu'à `StopCueLoop`.
    StartCueLoop(Cue),
    StopCueLoop {
        fade_out: Duration,
    },
    /// Régler un volume, de 0 à 100.
    Volume {
        category: SoundCategory,
        volume: u8,
    },
}
