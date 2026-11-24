//! Directeur audio (spec 081) : pur, sans I/O ni Kira. Il reçoit à chaque image la situation et
//! les événements nouveaux, et rend les ordres de son.

use std::collections::BTreeMap;
use std::time::Duration;

use arcanum_core::PlayerEvent;

use crate::cues::cue_of;
use crate::{Ambience, AudioCommand, Cue, Fade, Music, SoundCategory, SoundSituation};

/// Effets différents joués au plus par image (C15).
const MAX_CUES_PER_FRAME: usize = 4;

/// Fondu de sortie d'un échantillon, au lâcher du curseur.
const PREVIEW_FADE_OUT_MS: u64 = 500;

/// Ce que l'échantillon d'un curseur a ajouté à la mise en son.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Preview {
    Music,
    Ambience,
    Cue,
}

/// Garde la musique et l'ambiance en cours, et ne rend un ordre que sur un changement.
#[derive(Clone, Debug)]
pub struct AudioDirector {
    step_interval: Duration,
    situation: Option<SoundSituation>,
    music: Option<Music>,
    ambience: Option<Ambience>,
    /// Lectures de chaque effet depuis le lancement, pour tourner entre ses variantes (C16).
    plays: BTreeMap<Cue, usize>,
    preview: Option<Preview>,
}

impl AudioDirector {
    /// `step_interval` : durée d'un pas de l'animation du trajet (spec 075), pour espacer les pas.
    pub fn new(step_interval: Duration) -> Self {
        Self {
            step_interval,
            situation: None,
            music: None,
            ambience: None,
            plays: BTreeMap::new(),
            preview: None,
        }
    }

    /// Une image : la situation de l'écran et les événements apparus depuis l'image d'avant.
    pub fn update(
        &mut self,
        situation: SoundSituation,
        events: &[PlayerEvent],
    ) -> Vec<AudioCommand> {
        let mut commands = Vec::new();
        if self.situation != Some(situation) {
            let fade = fade(self.situation, situation);
            self.situation = Some(situation);
            if self.music != situation.music() {
                self.music = situation.music();
                commands.push(AudioCommand::Music {
                    music: self.music,
                    fade,
                });
            }
            if self.ambience != situation.ambience() {
                self.ambience = situation.ambience();
                commands.push(AudioCommand::Ambience {
                    ambience: self.ambience,
                    fade,
                });
            }
        }
        commands.extend(self.cues(situation, events));
        commands
    }

    /// Les effets d'une image : chacun une fois (les pas, un par case), quatre au plus dans
    /// l'ordre de la table (C13 à C15).
    fn cues(&mut self, situation: SoundSituation, events: &[PlayerEvent]) -> Vec<AudioCommand> {
        let mut heard: BTreeMap<Cue, usize> = BTreeMap::new();
        for cue in events.iter().filter_map(|event| cue_of(event, situation)) {
            *heard.entry(cue).or_default() += 1;
        }
        if heard.len() > MAX_CUES_PER_FRAME {
            let left: Vec<Cue> = heard.keys().skip(MAX_CUES_PER_FRAME).copied().collect();
            log::debug!("trop d'effets sonores dans une image, laissés : {left:?}");
        }
        let mut commands = Vec::new();
        for (cue, count) in heard.into_iter().take(MAX_CUES_PER_FRAME) {
            let repeats = if cue == Cue::Footstep { count } else { 1 };
            for step in 0..repeats {
                let plays = self.plays.entry(cue).or_default();
                commands.push(AudioCommand::Cue {
                    cue,
                    nth: *plays,
                    delay: self.step_interval * step as u32,
                });
                *plays += 1;
            }
        }
        commands
    }

    /// Échantillon du curseur tenu (`Some`), ou son arrêt (`None`) (C21, C30).
    pub fn preview(&mut self, category: Option<SoundCategory>) -> Vec<AudioCommand> {
        let Some(category) = category else {
            return self.stop_preview();
        };
        let (added, command) = match category {
            SoundCategory::Music if self.music.is_none() => (
                Preview::Music,
                AudioCommand::Music {
                    music: Some(Music::Title),
                    fade: Fade::millis(0, 0),
                },
            ),
            SoundCategory::Ambience if self.ambience.is_none() => (
                Preview::Ambience,
                AudioCommand::Ambience {
                    ambience: Some(Ambience::Day),
                    fade: Fade::millis(0, 0),
                },
            ),
            // La musique ou l'ambiance en cours sert d'échantillon.
            SoundCategory::Music | SoundCategory::Ambience => return Vec::new(),
            SoundCategory::Effects | SoundCategory::Master => {
                (Preview::Cue, AudioCommand::StartCueLoop(Cue::Preview))
            }
        };
        self.preview = Some(added);
        vec![command]
    }

    /// Arrête ce que l'échantillon avait ajouté, par un fondu court.
    fn stop_preview(&mut self) -> Vec<AudioCommand> {
        let fade = Fade::millis(PREVIEW_FADE_OUT_MS, 0);
        match self.preview.take() {
            None => Vec::new(),
            Some(Preview::Music) => vec![AudioCommand::Music { music: None, fade }],
            Some(Preview::Ambience) => vec![AudioCommand::Ambience {
                ambience: None,
                fade,
            }],
            Some(Preview::Cue) => vec![AudioCommand::StopCueLoop {
                fade_out: fade.fade_out,
            }],
        }
    }

    /// Un volume changé : transmis au lecteur, sans rien changer à la mise en son (L4).
    pub fn set_volume(&mut self, category: SoundCategory, volume: u8) -> Vec<AudioCommand> {
        vec![AudioCommand::Volume { category, volume }]
    }
}

/// Fondu d'une transition entre deux situations (table de la spec, Q4) : des durées à régler à
/// l'oreille, pas une mécanique de jeu.
pub fn fade(from: Option<SoundSituation>, to: SoundSituation) -> Fade {
    use SoundSituation::{Combat, Exploration, GameOver, Title};
    match (from, to) {
        (None, Title) => Fade::millis(0, 1000),
        (Some(Title), Exploration(_)) => Fade::millis(1500, 1500),
        (Some(Exploration(_)), Combat(_)) => Fade::millis(500, 0),
        (Some(Combat(_)), Exploration(_)) => Fade::millis(2000, 3000),
        (Some(Exploration(_)), Exploration(_)) => Fade::millis(4000, 4000),
        (_, GameOver(..)) => Fade::millis(3000, 0),
        _ => Fade::millis(1000, 1000),
    }
}
