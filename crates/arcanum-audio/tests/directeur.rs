//! Spec 081 : le directeur audio, musique et ambiance selon la situation (C1 à C5, C27, L4).
//! Aucune règle du livret : la mise en son seulement.

use std::time::Duration;

use arcanum_audio::{
    Ambience, AudioCommand, AudioDirector, Fade, Music, Outcome, SoundCategory, SoundSituation,
};
use arcanum_core::TimeOfDay::{Day, Night};

const STEP: Duration = Duration::from_millis(150);

fn director() -> AudioDirector {
    AudioDirector::new(STEP)
}

fn music(music: Option<Music>, fade: Fade) -> AudioCommand {
    AudioCommand::Music { music, fade }
}

fn ambience(ambience: Option<Ambience>, fade: Fade) -> AudioCommand {
    AudioCommand::Ambience { ambience, fade }
}

/// Le directeur déjà passé par ces situations, ordres oubliés.
fn director_after(situations: &[SoundSituation]) -> AudioDirector {
    let mut director = director();
    for situation in situations {
        director.update(*situation, &[]);
    }
    director
}

#[test]
fn title_starts_title_music() {
    // C1 : départ doux, aucune ambiance sur l'écran de lancement.
    let commands = director().update(SoundSituation::Title, &[]);
    assert_eq!(
        commands,
        vec![music(Some(Music::Title), Fade::millis(0, 1000))]
    );
}

#[test]
fn exploration_by_day_crossfades_once() {
    // C2
    let mut director = director_after(&[SoundSituation::Title]);
    let fade = Fade::millis(1500, 1500);
    assert_eq!(
        director.update(SoundSituation::Exploration(Day), &[]),
        vec![
            music(Some(Music::Day), fade),
            ambience(Some(Ambience::Day), fade)
        ]
    );
    assert_eq!(
        director.update(SoundSituation::Exploration(Day), &[]),
        vec![]
    );
}

#[test]
fn combat_switches_music_and_back() {
    // C3 : entrée en combat franche, ambiance inchangée (Q5) ; retour au calme lent.
    let mut director = director_after(&[SoundSituation::Title, SoundSituation::Exploration(Day)]);
    assert_eq!(
        director.update(SoundSituation::Combat(Day), &[]),
        vec![music(Some(Music::Combat), Fade::millis(500, 0))]
    );
    assert_eq!(
        director.update(SoundSituation::Exploration(Day), &[]),
        vec![music(Some(Music::Day), Fade::millis(2000, 3000))]
    );
}

#[test]
fn nightfall_switches_music_and_ambience() {
    // C4
    let mut director = director_after(&[SoundSituation::Title, SoundSituation::Exploration(Day)]);
    let fade = Fade::millis(4000, 4000);
    assert_eq!(
        director.update(SoundSituation::Exploration(Night), &[]),
        vec![
            music(Some(Music::Night), fade),
            ambience(Some(Ambience::Night), fade)
        ]
    );
}

#[test]
fn game_over_fades_music_out() {
    // C5 : la musique s'efface, l'ambiance de nuit reste.
    let mut director = director_after(&[SoundSituation::Title, SoundSituation::Exploration(Night)]);
    assert_eq!(
        director.update(SoundSituation::GameOver(Night, Outcome::Defeat), &[]),
        vec![music(None, Fade::millis(3000, 0))]
    );
}

#[test]
fn unlisted_transition_uses_fallback_fade() {
    // C27 : de combat à l'écran de lancement, valeur de repli ; l'ambiance s'arrête aussi.
    let mut director = director_after(&[
        SoundSituation::Title,
        SoundSituation::Exploration(Day),
        SoundSituation::Combat(Day),
    ]);
    let fade = Fade::millis(1000, 1000);
    assert_eq!(
        director.update(SoundSituation::Title, &[]),
        vec![music(Some(Music::Title), fade), ambience(None, fade)]
    );
}

#[test]
fn zero_master_volume_keeps_directing() {
    // L4 : le volume passe au lecteur, le directeur suit toujours la situation.
    let mut director = director_after(&[SoundSituation::Title]);
    assert_eq!(
        director.set_volume(SoundCategory::Master, 0),
        vec![AudioCommand::Volume {
            category: SoundCategory::Master,
            volume: 0
        }]
    );
    let commands = director.update(SoundSituation::Exploration(Day), &[]);
    assert!(commands.contains(&music(Some(Music::Day), Fade::millis(1500, 1500))));
}
