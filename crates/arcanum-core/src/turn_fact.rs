//! Faits du tour (spec 073) : ce que le tour a donné ou coûté, pour la zone de jeu de l'écran.

use crate::{
    BasicColor, CardId, CardPlay, CommonCard, EnemyToken, Event, Hex, ManaColor, PlayedCard,
    PlayerId, TilePosition, UnitCard,
};

/// Un fait du tour, tiré d'un événement du joueur (tableau de la spec 073).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnFact {
    /// Tuile révélée par une exploration (`Explored`).
    TileRevealed { position: TilePosition },
    /// Ennemis vaincus et la gloire rapportée (`EnemiesDefeated`).
    EnemiesDefeated { enemies: Vec<EnemyToken>, fame: u32 },
    /// Gloire gagnée hors combat (`FameGained`).
    FameGained { fame: u32 },
    /// Réputation changée, signée (`ReputationChanged`).
    ReputationChanged { change: i8 },
    /// Blessures reçues en main (`WoundsTaken`).
    WoundsTaken { count: usize },
    /// Blessures soignées (`WoundsHealed`).
    WoundsHealed { count: usize },
    /// Cristal gagné (`CrystalGained`).
    CrystalGained { color: BasicColor },
    /// Carte gagnée : offre, achat, artefact (`OfferCardGained`, `AdvancedActionBought`,
    /// `SpellBought`, `ArtifactTaken`).
    CardGained { card: CardId },
    /// Unité gagnée (`UnitRecruited`, `UnitRewardTaken`).
    UnitGained { unit: UnitCard },
    /// Unité perdue (`UnitDisbanded`, `UnitDestroyed`).
    UnitLost { unit: UnitCard },
    /// Unité blessée hors dégâts (`UnitWounded`).
    UnitWounded { unit: UnitCard },
    /// Unité soignée (`UnitHealed`).
    UnitHealed { unit: UnitCard },
    /// Site conquis (`SiteConquered`).
    SiteConquered { hex: Hex },
    /// Village pillé (`VillagePlundered`).
    VillagePlundered,
    /// Monastère incendié (`MonasteryBurned`).
    MonasteryBurned { hex: Hex },
    /// Carte action détruite (`CardDestroyed`, spec 056).
    CardDestroyed { card: CardId },
    /// Cristaux rendus par Maîtrise du cristal (`CrystalsReturned`, spec 056).
    CrystalsReturned { count: usize },
    /// Ennemi détruit par un sort (`EnemyDestroyed`, spec 054).
    EnemyDestroyed { enemy: EnemyToken },
    /// Sort lancé depuis l'offre (`CardPlayed` avec `from_offer`, spec 054, issue privée 246) : il reste
    /// dans l'offre, la zone de jeu de l'écran le montre par ce fait.
    SpellCastFromOffer { played: PlayedCard },
    /// Dé de mana lancé par un artefact (`ArtifactDieRolled`, spec 055) : la zone de jeu de l'écran
    /// montre les faces lancées.
    ArtifactDieRolled { card: CardId, color: ManaColor },
}

/// Le fait du tour que donne cet événement, et le joueur à qui il revient ; `None` pour les
/// événements que la zone de jeu montre déjà (déplacement, mana, cartes jouées, dés…).
pub(crate) fn turn_fact(event: &Event) -> Option<(PlayerId, TurnFact)> {
    let (player, fact) = site_fact(event)
        .or_else(|| combat_fact(event))
        .or_else(|| gain_fact(event))
        .or_else(|| unit_fact(event))?;
    Some((*player, fact))
}

/// Les faits de la carte et des sites : exploration, conquête, pillage, incendie.
fn site_fact(event: &Event) -> Option<(&PlayerId, TurnFact)> {
    Some(match event {
        Event::Explored {
            player, position, ..
        } => (
            player,
            TurnFact::TileRevealed {
                position: *position,
            },
        ),
        Event::SiteConquered { player, hex } => (player, TurnFact::SiteConquered { hex: *hex }),
        // Spec 062 : la conquête d'une cité, au nom de son Seigneur.
        Event::CityConquered { hex, leader } => (leader, TurnFact::SiteConquered { hex: *hex }),
        Event::VillagePlundered { player } => (player, TurnFact::VillagePlundered),
        Event::MonasteryBurned { player, hex } => (player, TurnFact::MonasteryBurned { hex: *hex }),
        _ => return None,
    })
}

/// Les faits du combat et des Blessures : ennemis vaincus ou détruits, Blessures reçues ou
/// soignées.
fn combat_fact(event: &Event) -> Option<(&PlayerId, TurnFact)> {
    Some(match event {
        Event::EnemiesDefeated {
            player,
            enemies,
            fame,
        } => (
            player,
            TurnFact::EnemiesDefeated {
                enemies: enemies.clone(),
                fame: *fame,
            },
        ),
        Event::EnemyDestroyed { player, enemy } => {
            (player, TurnFact::EnemyDestroyed { enemy: *enemy })
        }
        Event::WoundsTaken { player, wounds, .. } => (
            player,
            TurnFact::WoundsTaken {
                count: wounds.len(),
            },
        ),
        Event::WoundsHealed { player, wounds } => (
            player,
            TurnFact::WoundsHealed {
                count: wounds.len(),
            },
        ),
        _ => return None,
    })
}

/// Les gains et pertes du héros : gloire, réputation, cristaux, cartes, sorts de l'offre, dés
/// d'artefact.
fn gain_fact(event: &Event) -> Option<(&PlayerId, TurnFact)> {
    Some(match event {
        Event::FameGained { player, fame } => (player, TurnFact::FameGained { fame: *fame }),
        Event::ReputationChanged { player, change } => {
            (player, TurnFact::ReputationChanged { change: *change })
        }
        Event::CrystalGained { player, color } => {
            (player, TurnFact::CrystalGained { color: *color })
        }
        Event::CrystalsReturned { player, colors } => (
            player,
            TurnFact::CrystalsReturned {
                count: colors.len(),
            },
        ),
        Event::OfferCardGained {
            player,
            card: CommonCard::Deed(card),
            ..
        }
        | Event::AdvancedActionBought { player, card, .. }
        | Event::SpellBought { player, card, .. }
        | Event::ArtifactTaken { player, card } => (player, TurnFact::CardGained { card: *card }),
        Event::CardDestroyed { player, card, .. } => {
            (player, TurnFact::CardDestroyed { card: *card })
        }
        Event::CardPlayed {
            player,
            card,
            play: play @ CardPlay::Cast {
                from_offer: true, ..
            },
            effect,
        } => (
            player,
            TurnFact::SpellCastFromOffer {
                played: PlayedCard {
                    card: *card,
                    play: *play,
                    effect: *effect,
                },
            },
        ),
        Event::ArtifactDieRolled {
            player,
            card,
            color,
        } => (
            player,
            TurnFact::ArtifactDieRolled {
                card: *card,
                color: *color,
            },
        ),
        _ => return None,
    })
}

/// Les faits des unités : gagnées, perdues, blessées, soignées.
fn unit_fact(event: &Event) -> Option<(&PlayerId, TurnFact)> {
    Some(match event {
        Event::OfferCardGained {
            player,
            card: CommonCard::Unit(unit),
            ..
        }
        | Event::UnitRecruited { player, unit, .. }
        | Event::UnitRewardTaken { player, unit } => (player, TurnFact::UnitGained { unit: *unit }),
        Event::UnitDisbanded { player, unit } | Event::UnitDestroyed { player, unit } => {
            (player, TurnFact::UnitLost { unit: *unit })
        }
        Event::UnitWounded { player, unit, .. } => (player, TurnFact::UnitWounded { unit: *unit }),
        Event::UnitHealed { player, unit, .. } => (player, TurnFact::UnitHealed { unit: *unit }),
        _ => return None,
    })
}
