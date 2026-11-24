//! Lecteurs audio (spec 081) : exécutent les ordres du directeur. `KiraPlayer` joue sur la carte
//! son ; `SilentPlayer` ne fait rien (pas de carte son, tests). `KiraPlayer` n'a pas de test : il
//! se vérifie à l'oreille, comme `app/` à l'œil (ADR 0006). Seul `main.rs` le crée (L5).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use kira::sound::FromFileError;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::track::{TrackBuilder, TrackHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, StartTime, Tween};
use thiserror::Error;

use crate::{AudioCommand, Cue, SoundBank, SoundCategory};

/// Exécute les ordres de son.
pub trait AudioPlayer {
    fn execute(&mut self, command: &AudioCommand);
}

/// Le lecteur muet : sans carte son, le jeu tourne quand même (L1).
#[derive(Clone, Copy, Debug, Default)]
pub struct SilentPlayer;

impl AudioPlayer for SilentPlayer {
    fn execute(&mut self, _command: &AudioCommand) {}
}

/// Échec de la création du lecteur Kira.
#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("aucune sortie audio : {0}")]
    Output(String),
    #[error("piste audio impossible à créer : {0}")]
    Track(String),
}

type Streaming = StreamingSoundHandle<FromFileError>;

/// Le lecteur sur la carte son : une piste par type de son sous la piste principale (volume
/// général), musique et ambiance lues en flux, effets chargés en mémoire.
pub struct KiraPlayer {
    manager: AudioManager<DefaultBackend>,
    music_track: TrackHandle,
    ambience_track: TrackHandle,
    effects_track: TrackHandle,
    bank: SoundBank,
    music: Option<Streaming>,
    ambience: Option<Streaming>,
    cue_loop: Option<StaticSoundHandle>,
    effects: BTreeMap<PathBuf, StaticSoundData>,
    /// Fichiers illisibles déjà signalés (un seul `warn` chacun, L3).
    unreadable: BTreeSet<PathBuf>,
}

impl KiraPlayer {
    /// Ouvre la sortie audio par défaut. Échoue sans carte son : l'appelant prend alors
    /// `SilentPlayer` (L1).
    pub fn new(bank: SoundBank) -> Result<Self, PlayerError> {
        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|error| PlayerError::Output(error.to_string()))?;
        let mut track = || {
            manager
                .add_sub_track(TrackBuilder::new())
                .map_err(|error| PlayerError::Track(error.to_string()))
        };
        let music_track = track()?;
        let ambience_track = track()?;
        let effects_track = track()?;
        Ok(Self {
            manager,
            music_track,
            ambience_track,
            effects_track,
            bank,
            music: None,
            ambience: None,
            cue_loop: None,
            effects: BTreeMap::new(),
            unreadable: BTreeSet::new(),
        })
    }

    /// Change le morceau d'une piste : l'ancien s'efface pendant que le nouveau monte.
    fn switch(
        current: &mut Option<Streaming>,
        track: &mut TrackHandle,
        path: Option<PathBuf>,
        fade_out: Duration,
        fade_in: Duration,
        unreadable: &mut BTreeSet<PathBuf>,
    ) {
        if let Some(mut handle) = current.take() {
            handle.stop(tween(fade_out));
        }
        let Some(path) = path else {
            return;
        };
        let data = match StreamingSoundData::from_file(&path) {
            Ok(data) => data.loop_region(..).fade_in_tween(fade_in_tween(fade_in)),
            Err(error) => {
                warn_unreadable(unreadable, &path, &error.to_string());
                return;
            }
        };
        match track.play(data) {
            Ok(handle) => *current = Some(handle),
            Err(error) => log::warn!("lecture impossible de {} : {error}", path.display()),
        }
    }

    /// Les données d'un effet, lues une fois.
    fn effect_data(&mut self, cue: Cue, nth: usize) -> Option<StaticSoundData> {
        let path = self.bank.cue_file(cue, nth)?.to_path_buf();
        if let Some(data) = self.effects.get(&path) {
            return Some(data.clone());
        }
        if self.unreadable.contains(&path) {
            return None;
        }
        match StaticSoundData::from_file(&path) {
            Ok(data) => {
                self.effects.insert(path, data.clone());
                Some(data)
            }
            Err(error) => {
                warn_unreadable(&mut self.unreadable, &path, &error.to_string());
                None
            }
        }
    }

    fn set_volume(&mut self, category: SoundCategory, volume: u8) {
        let decibels = decibels(volume);
        let now = Tween::default();
        match category {
            SoundCategory::Master => self.manager.main_track().set_volume(decibels, now),
            SoundCategory::Music => self.music_track.set_volume(decibels, now),
            SoundCategory::Ambience => self.ambience_track.set_volume(decibels, now),
            SoundCategory::Effects => self.effects_track.set_volume(decibels, now),
        }
    }
}

impl AudioPlayer for KiraPlayer {
    fn execute(&mut self, command: &AudioCommand) {
        match *command {
            AudioCommand::Music { music, fade } => {
                let path = music.and_then(|music| self.bank.music(music).map(Path::to_path_buf));
                Self::switch(
                    &mut self.music,
                    &mut self.music_track,
                    path,
                    fade.fade_out,
                    fade.fade_in,
                    &mut self.unreadable,
                );
            }
            AudioCommand::Ambience { ambience, fade } => {
                let path = ambience
                    .and_then(|ambience| self.bank.ambience(ambience).map(Path::to_path_buf));
                Self::switch(
                    &mut self.ambience,
                    &mut self.ambience_track,
                    path,
                    fade.fade_out,
                    fade.fade_in,
                    &mut self.unreadable,
                );
            }
            AudioCommand::Cue { cue, nth, delay } => {
                if let Some(data) = self.effect_data(cue, nth) {
                    let data = data.start_time(StartTime::Delayed(delay));
                    if let Err(error) = self.effects_track.play(data) {
                        log::warn!("effet sonore {cue:?} non joué : {error}");
                    }
                }
            }
            AudioCommand::StartCueLoop(cue) => {
                if let Some(mut handle) = self.cue_loop.take() {
                    handle.stop(Tween::default());
                }
                if let Some(data) = self.effect_data(cue, 0) {
                    match self.effects_track.play(data.loop_region(..)) {
                        Ok(handle) => self.cue_loop = Some(handle),
                        Err(error) => log::warn!("échantillon {cue:?} non joué : {error}"),
                    }
                }
            }
            AudioCommand::StopCueLoop { fade_out } => {
                if let Some(mut handle) = self.cue_loop.take() {
                    handle.stop(tween(fade_out));
                }
            }
            AudioCommand::Volume { category, volume } => self.set_volume(category, volume),
        }
    }
}

fn tween(duration: Duration) -> Tween {
    Tween {
        duration,
        ..Tween::default()
    }
}

fn fade_in_tween(duration: Duration) -> Option<Tween> {
    (!duration.is_zero()).then(|| tween(duration))
}

fn warn_unreadable(unreadable: &mut BTreeSet<PathBuf>, path: &Path, cause: &str) {
    if unreadable.insert(path.to_path_buf()) {
        log::warn!(
            "fichier de son illisible, sauté : {} ({cause})",
            path.display()
        );
    }
}

/// Volume de 0 à 100 en décibels, sur une courbe perceptive (amplitude au carré), à régler à
/// l'oreille ; 0 = silence.
fn decibels(volume: u8) -> Decibels {
    let amplitude = (f32::from(volume.min(100)) / 100.0).powi(2);
    if amplitude <= 0.0 {
        Decibels::SILENCE
    } else {
        Decibels((20.0 * amplitude.log10()).max(Decibels::SILENCE.0))
    }
}
