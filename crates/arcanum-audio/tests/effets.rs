//! Spec 081 : les effets sonores du directeur (C12 à C15, C17, C29). Événements écrits à la main,
//! aucune partie à jouer. Aucune règle du livret.

use std::time::Duration;

use arcanum_audio::{AudioCommand, AudioDirector, Cue, Outcome, SoundSituation};
use arcanum_core::TimeOfDay::{Day, Night};
use arcanum_core::{
    BasicAction, CardId, CardPlay, DeedCard, Effect, Enemy, EnemyToken, GameConfig, GameEndReason,
    Hero, Hex, PlayerEvent, PlayerId, Scenario, SidewaysEffect, Spell, TileId, TilePosition, Unit,
    UnitCard,
};

const STEP: Duration = Duration::from_millis(150);
const SOLO: PlayerId = PlayerId::SOLO;
const EXPLORATION: SoundSituation = SoundSituation::Exploration(Day);

/// Le directeur déjà en exploration de jour : seules les commandes d'effets restent à voir.
fn director() -> AudioDirector {
    let mut director = AudioDirector::new(STEP);
    director.update(EXPLORATION, &[]);
    director
}

/// Les effets rendus : l'effet, son numéro de lecture et son délai.
fn cues(commands: &[AudioCommand]) -> Vec<(Cue, usize, Duration)> {
    commands
        .iter()
        .filter_map(|command| match *command {
            AudioCommand::Cue { cue, nth, delay } => Some((cue, nth, delay)),
            _ => None,
        })
        .collect()
}

/// Les seuls effets (sans numéro ni délai) d'une image.
fn cue_names(situation: SoundSituation, events: &[PlayerEvent]) -> Vec<Cue> {
    let mut director = director();
    cues(&director.update(situation, events))
        .into_iter()
        .map(|(cue, _, _)| cue)
        .collect()
}

fn enemy() -> EnemyToken {
    EnemyToken::new(Enemy::Prowlers, 1)
}

fn wound() -> CardId {
    CardId::new(DeedCard::Wound, 1)
}

fn hex() -> Hex {
    Hex::new(0, 0)
}

fn moved(player: PlayerId) -> PlayerEvent {
    PlayerEvent::HeroMoved {
        player,
        from: hex(),
        to: Hex::new(1, 0),
        cost: 2,
    }
}

fn wounds_taken() -> PlayerEvent {
    PlayerEvent::WoundsTaken {
        player: SOLO,
        enemy: enemy(),
        wounds: vec![wound()],
    }
}

fn played(card: DeedCard, play: CardPlay) -> PlayerEvent {
    PlayerEvent::CardPlayed {
        player: SOLO,
        card: CardId::new(card, 1),
        play,
        effect: Effect::Move(2),
    }
}

fn boosted() -> CardPlay {
    CardPlay::Boosted {
        option: 0,
        discard: None,
        set_dice: [None, None],
    }
}

fn basic() -> CardPlay {
    CardPlay::Basic {
        option: 0,
        discard: None,
        mana: None,
        with: None,
        gain: None,
        die: None,
    }
}

fn unit() -> UnitCard {
    UnitCard {
        unit: Unit::Peasants,
        copy: 1,
    }
}

/// Une ligne de la table des effets de la spec par événement, sauf la fin de partie (C29).
fn table() -> Vec<(PlayerEvent, Option<Cue>)> {
    vec![
        (
            PlayerEvent::RoundStarted {
                round: 1,
                time_of_day: Day,
            },
            Some(Cue::Rooster),
        ),
        (
            PlayerEvent::RoundStarted {
                round: 2,
                time_of_day: Night,
            },
            Some(Cue::Owl),
        ),
        // Le premier jour n'a pas de `RoundStarted` : son coq vient du début de la partie.
        (
            PlayerEvent::GameStarted {
                config: GameConfig {
                    seed: 1,
                    scenario: Scenario::first_reconnaissance(),
                    hero: Hero::Arythea,
                    dummy_hero: Default::default(),
                },
            },
            Some(Cue::Rooster),
        ),
        (moved(SOLO), Some(Cue::Footstep)),
        (
            PlayerEvent::TileRevealed {
                tile: TileId::Countryside(1),
                position: TilePosition::containing(hex()),
            },
            Some(Cue::TileRevealed),
        ),
        (
            PlayerEvent::CombatStarted {
                player: SOLO,
                enemies: vec![(hex(), enemy())],
            },
            Some(Cue::CombatStart),
        ),
        (
            PlayerEvent::EnemyBlocked {
                player: SOLO,
                enemy: enemy(),
            },
            Some(Cue::Block),
        ),
        (PlayerEvent::AttackFailed { player: SOLO }, Some(Cue::Miss)),
        (PlayerEvent::BlockFailed { player: SOLO }, Some(Cue::Miss)),
        (wounds_taken(), Some(Cue::Wound)),
        (
            PlayerEvent::WoundFromCard {
                player: SOLO,
                card: CardId::new(DeedCard::Basic(BasicAction::March), 1),
                wound: wound(),
            },
            Some(Cue::Wound),
        ),
        (
            PlayerEvent::UnitWounded {
                player: SOLO,
                unit: unit(),
                wounds: 1,
            },
            Some(Cue::Wound),
        ),
        (
            PlayerEvent::EnemiesDefeated {
                player: SOLO,
                enemies: vec![enemy()],
                fame: 2,
            },
            Some(Cue::EnemyDefeated),
        ),
        (
            PlayerEvent::EnemyDestroyed {
                player: SOLO,
                enemy: enemy(),
            },
            Some(Cue::EnemyDefeated),
        ),
        (
            PlayerEvent::CombatEnded { player: SOLO },
            Some(Cue::CombatEnd),
        ),
        // Démolition (spells.ron), effet de base ou renforcé : un sort lancé.
        (
            played(DeedCard::Spell(Spell::Demolish), boosted()),
            Some(Cue::Spell),
        ),
        // Joué de côté, un sort n'est pas lancé.
        (
            played(
                DeedCard::Spell(Spell::Demolish),
                CardPlay::Sideways(SidewaysEffect::Move),
            ),
            None,
        ),
        // Détermination n'est pas un sort.
        (
            played(DeedCard::Basic(BasicAction::Determination), basic()),
            None,
        ),
        (
            PlayerEvent::SiteConquered {
                player: SOLO,
                hex: hex(),
            },
            Some(Cue::Conquest),
        ),
        (
            PlayerEvent::CityConquered {
                hex: hex(),
                leader: SOLO,
            },
            Some(Cue::Conquest),
        ),
        (
            PlayerEvent::LevelGained {
                player: SOLO,
                level: 2,
            },
            Some(Cue::LevelUp),
        ),
        (
            PlayerEvent::UnitRecruited {
                player: SOLO,
                unit: unit(),
                cost: 4,
            },
            Some(Cue::Recruit),
        ),
        (
            PlayerEvent::KnockedOut {
                player: SOLO,
                cards: vec![],
            },
            Some(Cue::KnockedOut),
        ),
        (
            PlayerEvent::LastTurnAnnounced { player: SOLO },
            Some(Cue::LastTurn),
        ),
        // Rien d'autre ne sonne (Hors périmètre) : une carte piochée, par exemple.
        (
            PlayerEvent::CardDrawn {
                player: SOLO,
                card: None,
            },
            None,
        ),
    ]
}

#[test]
fn events_map_to_cues() {
    // C12
    for (event, expected) in table() {
        let heard = cue_names(EXPLORATION, std::slice::from_ref(&event));
        assert_eq!(heard, expected.into_iter().collect::<Vec<_>>(), "{event:?}");
    }
}

#[test]
fn same_cue_once_per_frame() {
    // C13
    let events = [wounds_taken(), wounds_taken(), wounds_taken()];
    assert_eq!(cue_names(EXPLORATION, &events), vec![Cue::Wound]);
}

#[test]
fn footsteps_follow_the_walk() {
    // C14 : un pas par case, espacés comme l'animation du trajet (spec 075).
    let mut director = director();
    let events = [moved(SOLO), moved(SOLO), moved(SOLO)];
    assert_eq!(
        cues(&director.update(EXPLORATION, &events)),
        vec![
            (Cue::Footstep, 0, Duration::ZERO),
            (Cue::Footstep, 1, Duration::from_millis(150)),
            (Cue::Footstep, 2, Duration::from_millis(300)),
        ]
    );
}

#[test]
fn at_most_four_cues_per_frame() {
    // C15 : les quatre premiers dans l'ordre de la table, quel que soit l'ordre des événements.
    let events = [
        PlayerEvent::LevelGained {
            player: SOLO,
            level: 2,
        },
        PlayerEvent::CombatEnded { player: SOLO },
        wounds_taken(),
        PlayerEvent::AttackFailed { player: SOLO },
        PlayerEvent::RoundStarted {
            round: 1,
            time_of_day: Day,
        },
        moved(SOLO),
    ];
    assert_eq!(
        cue_names(EXPLORATION, &events),
        vec![Cue::Rooster, Cue::Footstep, Cue::Miss, Cue::Wound]
    );
}

#[test]
fn cue_plays_are_numbered() {
    // C16, côté directeur : chaque lecture d'un effet a son numéro, pour la variante.
    let mut director = director();
    let numbers: Vec<usize> = (0..4)
        .flat_map(|_| cues(&director.update(EXPLORATION, &[wounds_taken()])))
        .map(|(_, nth, _)| nth)
        .collect();
    assert_eq!(numbers, vec![0, 1, 2, 3]);
}

#[test]
fn only_the_human_player_is_heard() {
    // C17 : le fantôme (spec 061) ne fait aucun bruit, sauf la fin de partie, pour tout joueur.
    let other = PlayerId(1);
    let events = [
        moved(other),
        PlayerEvent::CombatEnded { player: other },
        PlayerEvent::CityConquered {
            hex: hex(),
            leader: other,
        },
    ];
    assert_eq!(cue_names(EXPLORATION, &events), vec![]);
    let over = SoundSituation::GameOver(Day, Outcome::Victory);
    let ended = PlayerEvent::GameEnded {
        reason: GameEndReason::CityRevealed,
    };
    assert_eq!(cue_names(over, &[ended]), vec![Cue::Victory]);
}

#[test]
fn game_ended_plays_victory_or_defeat() {
    // C29 : le jingle suit l'issue portée par la situation ; défaite hors fin de partie.
    let ended = PlayerEvent::GameEnded {
        reason: GameEndReason::LastRound,
    };
    let cases = [
        (
            SoundSituation::GameOver(Night, Outcome::Victory),
            Cue::Victory,
        ),
        (
            SoundSituation::GameOver(Night, Outcome::Defeat),
            Cue::Defeat,
        ),
        (EXPLORATION, Cue::Defeat),
    ];
    for (situation, expected) in cases {
        assert_eq!(
            cue_names(situation, std::slice::from_ref(&ended)),
            vec![expected],
            "{situation:?}"
        );
    }
}
