//! Spec 081 : l'échantillon du type de son pendant qu'on glisse un curseur de volume (C30).

use std::time::Duration;

use arcanum_audio::{
    Ambience, AudioCommand, AudioDirector, Cue, Fade, Music, Outcome, SoundCategory, SoundSituation,
};
use arcanum_core::TimeOfDay::Day;

fn director_in(situation: SoundSituation) -> AudioDirector {
    let mut director = AudioDirector::new(Duration::from_millis(150));
    director.update(situation, &[]);
    director
}

#[test]
fn preview_uses_current_sound_or_a_sample() {
    // C30 : en fin de partie, plus de musique : un extrait de celle du titre, puis son arrêt.
    let mut over = director_in(SoundSituation::GameOver(Day, Outcome::Victory));
    assert_eq!(
        over.preview(Some(SoundCategory::Music)),
        vec![AudioCommand::Music {
            music: Some(Music::Title),
            fade: Fade::millis(0, 0)
        }]
    );
    assert_eq!(
        over.preview(None),
        vec![AudioCommand::Music {
            music: None,
            fade: Fade::millis(500, 0)
        }]
    );

    // La musique de la situation sert d'échantillon : rien à ajouter, rien à arrêter.
    let mut exploring = director_in(SoundSituation::Exploration(Day));
    assert_eq!(exploring.preview(Some(SoundCategory::Music)), vec![]);
    assert_eq!(exploring.preview(None), vec![]);

    // Sur l'écran de lancement, pas d'ambiance : celle du jour.
    let mut title = director_in(SoundSituation::Title);
    assert_eq!(
        title.preview(Some(SoundCategory::Ambience)),
        vec![AudioCommand::Ambience {
            ambience: Some(Ambience::Day),
            fade: Fade::millis(0, 0)
        }]
    );
    assert_eq!(
        title.preview(None),
        vec![AudioCommand::Ambience {
            ambience: None,
            fade: Fade::millis(500, 0)
        }]
    );

    // Effets et volume général : l'effet d'essai, rejoué tant qu'on tient le curseur.
    for category in [SoundCategory::Effects, SoundCategory::Master] {
        let mut director = director_in(SoundSituation::Exploration(Day));
        assert_eq!(
            director.preview(Some(category)),
            vec![AudioCommand::StartCueLoop(Cue::Preview)],
            "{category:?}"
        );
        assert_eq!(
            director.preview(None),
            vec![AudioCommand::StopCueLoop {
                fade_out: Duration::from_millis(500)
            }],
            "{category:?}"
        );
    }
}
