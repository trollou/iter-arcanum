//! Spec 051 (compétences), sauvegarde : C27 ; spec 058 : C46.

use arcanum_core::{
    Action, BasicAction, BasicColor, CardPlay, Event, ManaColor, SidewaysEffect, Skill,
    SkillTarget, Unit, UnitCard,
};
use arcanum_persistance::{SaveGame, load, save};

#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

// C27 — spec 024 : compétences gagnées, utilisées et retournées identiques après rechargement.
#[test]
fn skills_survive_save_and_load() {
    let mut events = arena_events(VILLAGE, &move_hand(), &[], &[]);
    events.extend(
        [
            Skill::DoubleTime,
            Skill::DarkFireMagic,
            Skill::ColdSwordsmanship,
        ]
        .map(|skill| Event::SkillGained {
            player: SOLO,
            skill,
        }),
    );
    events.push(Event::SkillToCommonOffer {
        skill: Skill::FreezingPower,
    });
    let (mut runner, puppet) = runner(events);
    for (skill, option) in [(Skill::DoubleTime, 0), (Skill::DarkFireMagic, 1)] {
        play(
            &mut runner,
            &puppet,
            arcanum_core::Action::UseSkill {
                skill,
                option,
                unit: None,
                target: None,
            },
        );
    }
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(&runner, "2026-11-13T00:00:00+00:00").expect("partie");
    let path = save(dir.path(), "competences", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(Puppet::default())])
        .expect("partie reprise");
    let (before, after) = (runner.state(), reloaded.state());
    assert_eq!(after, before);
    assert_eq!(after.skills(SOLO), before.skills(SOLO));
    assert_eq!(after.skills(SOLO).len(), 3);
    assert_eq!(after.common_skills(), [Skill::FreezingPower]);
    assert_eq!(
        arcanum_core::view(after, SOLO).skills,
        arcanum_core::view(before, SOLO).skills
    );
}

// Spec 058, C46 — spec 024 : une compétence au centre, l'Influence d'interaction, l'unité sous
// Loyauté sans faille et le dé d'Embrasure de la Source se rechargent à l'identique.
#[test]
fn second_batch_skills_survive_save_and_load() {
    let march = card(BasicAction::March, 1);
    let crossbowmen = UnitCard::new(Unit::UtemCrossbowmen, 1);
    let mut events = Units {
        hand: vec![march, wound(1)],
        offer: vec![crossbowmen],
        dice: vec![
            ManaColor::Basic(BasicColor::White),
            ManaColor::Basic(BasicColor::Red),
            ManaColor::Basic(BasicColor::Green),
        ],
        ..Units::default()
    }
    .events();
    events.extend(
        [
            Skill::BondsOfLoyalty,
            Skill::GlitteringFortune,
            Skill::RitualOfPain,
            Skill::SourceOpening,
        ]
        .map(|skill| Event::SkillGained {
            player: SOLO,
            skill,
        }),
    );
    events.extend(
        [BasicColor::Red, BasicColor::Blue].map(|color| Event::CrystalGained {
            player: SOLO,
            color,
        }),
    );
    let (mut runner, puppet) = runner(events);
    let use_skill = |skill, target| Action::UseSkill {
        skill,
        option: 0,
        unit: None,
        target,
    };
    for action in [
        Action::Interact,
        Action::PlayCard {
            card: march,
            play: CardPlay::Sideways(SidewaysEffect::Influence),
        },
        Action::RecruitUnit {
            unit: crossbowmen,
            disband: None,
            bonds: true,
        },
        use_skill(Skill::GlitteringFortune, None),
        use_skill(Skill::RitualOfPain, None),
        use_skill(Skill::SourceOpening, Some(SkillTarget::Die(0))),
    ] {
        play(&mut runner, &puppet, action);
    }
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(&runner, "2026-11-14T00:00:00+00:00").expect("partie");
    let path = save(dir.path(), "competences-058", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(Puppet::default())])
        .expect("partie reprise");
    let (before, after) = (runner.state(), reloaded.state());
    assert_eq!(after, before);
    let (seen, was) = (
        arcanum_core::view(after, SOLO),
        arcanum_core::view(before, SOLO),
    );
    assert_eq!(seen.bonds_unit, Some(crossbowmen));
    assert_eq!(seen.interaction_influence, 2);
    assert_eq!(seen.center_advantages.len(), 2);
    assert_eq!(seen.center_advantages, was.center_advantages);
    assert_eq!(seen.skills, was.skills);
    assert_eq!(seen.source, was.source);
}
