//! Application : un événement change l'état (`apply`), sans hasard ni échec possible.
//! L'aiguillage et les aides communes sont ici ; chaque thème applique ses événements dans son
//! sous-module, comme `legal/`.

use super::*;

mod cards;
mod combat;
mod dummy;
mod mana;
mod map;
mod progression;
mod sites;
mod turn;
mod units;

use cards::*;
use combat::*;
use dummy::*;
use mana::*;
use map::*;
use progression::*;
use sites::*;
use turn::*;
use units::*;

/// Pure et infaillible : un événement sans objet (joueur inconnu) ne change rien.
pub fn apply(mut state: GameState, event: &Event) -> GameState {
    // Spec 045 (C40) : l'annonce des ruines révélées par un pas ou un trajet dure jusqu'à l'action
    // suivante ; toute autre conséquence l'efface.
    if !matches!(event, Event::RuinTokenRevealed { .. }) {
        for p in &mut state.players {
            p.revealed_ruins.clear();
        }
    }
    // Spec 073 : le fait du tour suit l'événement, seulement s'il a changé l'état (un événement
    // sans objet ne change rien) ; spec 055 : un dé d'artefact lancé, même sans rien changer (Cor
    // de colère), pour un joueur de la partie.
    let Some((player, fact)) = turn_fact(event) else {
        return apply_event(state, event);
    };
    let before = state.clone();
    let mut state = apply_event(state, event);
    let rolled = matches!(fact, crate::TurnFact::ArtifactDieRolled { .. });
    if (state != before || rolled)
        && let Some(p) = state.player_mut(player)
    {
        p.turn_facts.push(fact);
    }
    state
}

/// Le changement propre à chaque événement, pour `apply`.
fn apply_event(mut state: GameState, event: &Event) -> GameState {
    // Aiguillage exhaustif : un nouvel événement doit être rangé dans un thème.
    match event {
        Event::GameStarted { .. }
        | Event::TilePlaced { .. }
        | Event::TileDeckFormed { .. }
        | Event::TileRevealed { .. }
        | Event::HeroFigurePlaced { .. }
        | Event::HeroMoved { .. }
        | Event::Explored { .. }
        | Event::EnemyPileShuffled { .. }
        | Event::EnemyPileReshuffled { .. }
        | Event::EnemyTokenPlaced { .. }
        | Event::DefenderPlaced { .. }
        | Event::CityPlaced { .. }
        | Event::EnemyTokenRevealed { .. }
        | Event::LastTurnAnnounced { .. }
        | Event::GameEnded { .. }
        | Event::HeroWithdrew { .. }
        | Event::EnemyTokenDiscarded { .. }
        | Event::ForcedWithdrawal { .. }
        | Event::RuinPileShuffled { .. }
        | Event::RuinTokenPlaced { .. }
        | Event::RuinTokenRevealed { .. } => map_event(&mut state, event),
        Event::DeckShuffled { .. }
        | Event::CardDrawn { .. }
        | Event::CardPlayed { .. }
        | Event::CardDiscarded { .. }
        | Event::CardReturnedToHand { .. }
        | Event::DeckReshuffled { .. }
        | Event::WoundsHealed { .. }
        | Event::SteadyTempoKept { .. }
        | Event::WoundThrownAway { .. }
        | Event::CardDestroyed { .. }
        | Event::CardsReturnedToHand { .. }
        | Event::CardSetAside { .. }
        | Event::DiscardPicked { .. }
        | Event::CardsPlacedOnDeck { .. }
        | Event::DrawBonusUsed { .. } => card_event(&mut state, event),
        Event::SourceRolled { .. }
        | Event::CrystalGained { .. }
        | Event::DieTaken { .. }
        | Event::CrystalUsed { .. }
        | Event::DieReturned { .. }
        | Event::ManaTokenGained { .. }
        | Event::ManaTokenUsed { .. }
        | Event::DieSet { .. }
        | Event::EndlessManaUsed { .. }
        | Event::RewardDieRolled { .. }
        | Event::SourceDieKept { .. }
        | Event::CrystalDeclined { .. }
        | Event::CrystalsReturned { .. }
        | Event::DieKept { .. }
        | Event::DieRerolled { .. } => mana_event(&mut state, event),
        Event::TurnEnded { .. }
        | Event::TurnStarted { .. }
        | Event::TacticChosen { .. }
        | Event::DieStolen { .. }
        | Event::LongNightUsed { .. }
        | Event::CardStored { .. }
        | Event::StoredCardsTaken { .. }
        | Event::CardsShuffledIntoDeck { .. }
        | Event::ManaSearchUsed { .. }
        | Event::DrawStopped { .. }
        | Event::RightMomentAnnounced { .. }
        | Event::StolenDieUsed { .. }
        | Event::StolenDieReturned { .. }
        | Event::CardSearched { .. }
        | Event::TacticsRemoved { .. }
        | Event::RoundEnded { .. }
        | Event::RoundStarted { .. }
        | Event::ExtraTurnStarted { .. } => turn_event(&mut state, event),
        Event::DummySetUp { .. }
        | Event::DummyTacticDrawn { .. }
        | Event::DummyTurnPlayed { .. }
        | Event::DummyAnnouncedEndOfRound
        | Event::DummyCardAdded { .. }
        | Event::DummyCrystalGained { .. }
        | Event::DummyDeckShuffled { .. }
        | Event::DummySkillRevealed { .. } => dummy_event(&mut state, event),
        Event::AssaultStarted { .. }
        | Event::EnemiesProvoked { .. }
        | Event::EnemyChallenged { .. }
        | Event::CombatStarted { .. }
        | Event::CombatPhaseStarted { .. }
        | Event::CombatEnded { .. }
        | Event::AttackDeclared { .. }
        | Event::EnemiesDefeated { .. }
        | Event::AttackFailed { .. }
        | Event::BlockDeclared { .. }
        | Event::EnemyBlocked { .. }
        | Event::BlockFailed { .. }
        | Event::WoundsTaken { .. }
        | Event::WoundFromCard { .. }
        | Event::PoisonWoundsDiscarded { .. }
        | Event::HandParalyzed { .. }
        | Event::EnemySummoned { .. }
        | Event::SummonedEnemyDiscarded { .. }
        | Event::SummonsHeld { .. }
        | Event::KnockedOut { .. }
        | Event::MoveSpentAsAttack { .. }
        | Event::InfluenceSpentAsBlock { .. }
        | Event::ResistanceBroken { .. }
        | Event::EnemyDestroyed { .. }
        | Event::DeferredAttackUsed { .. } => combat_event(&mut state, event),
        Event::UnitRecruited { .. }
        | Event::BannerUsed { .. }
        | Event::BannersReclaimed { .. }
        | Event::BannerAssigned { .. }
        | Event::BannerLost { .. }
        | Event::UnitDisbanded { .. }
        | Event::UnitActivated { .. }
        | Event::UnitReadied { .. }
        | Event::UnitWounded { .. }
        | Event::UnitsReadied { .. }
        | Event::DamageAssignedToUnit { .. }
        | Event::UnitDestroyed { .. }
        | Event::UnitHealed { .. }
        | Event::UnitUnderBonds { .. }
        | Event::UnitRewardTaken { .. }
        | Event::UnitRewardDeclined { .. } => unit_event(&mut state, event),
        Event::CommonDeckShuffled { .. }
        | Event::OfferCardDealt { .. }
        | Event::OfferCardReturned { .. }
        | Event::OfferCardGained { .. }
        | Event::InteractionStarted { .. }
        | Event::ArtifactDieRolled { .. }
        | Event::HealingBought { .. }
        | Event::VillagePlundered { .. }
        | Event::ArtifactsDrawn { .. }
        | Event::ArtifactTaken { .. }
        | Event::ArtifactReturned { .. }
        | Event::MonasteryBurnStarted { .. }
        | Event::MonasteryBurned { .. }
        | Event::SiteConquered { .. }
        | Event::CityShieldPlaced { .. }
        | Event::CityConquered { .. }
        | Event::AdvancedActionDrawnBlind { .. }
        | Event::ArtifactBought { .. }
        | Event::EliteUnitAdded { .. }
        | Event::AdventureSiteEntered { .. }
        | Event::SpellBought { .. }
        | Event::AdvancedActionBought { .. }
        | Event::AltarTributePaid { .. }
        | Event::AdvancedActionLearned { .. } => site_event(&mut state, event),
        Event::FameGained { .. }
        | Event::LevelGained { .. }
        | Event::ReputationChanged { .. }
        | Event::SkillsRevealed { .. }
        | Event::SkillGained { .. }
        | Event::SkillTakenFromCommonOffer { .. }
        | Event::SkillUsed { .. }
        | Event::SkillPlacedInCenter { .. }
        | Event::CenterAdvantageUsed { .. }
        | Event::CenterSkillReturned { .. }
        | Event::SkillToCommonOffer { .. }
        | Event::SkillPileShuffled { .. } => progression_event(&mut state, event),
    }
    state
}

/// Les points (mouvement, influence) et les dés de plus d'un effet joué, carte ou unité (MKUE
/// p. 4, 4 : les effets semblables s'additionnent ; spec 025).
fn add_effect_points(p: &mut PlayerState, effect: Effect) {
    match effect {
        Effect::Move(amount) => p.move_points += u32::from(amount),
        // Spec 053 : aucun soin en combat, le Mouvement seul (MKUE p. 8, 6).
        Effect::MoveAndHeal { movement, heal } => {
            p.move_points += u32::from(movement);
            if !p.combat.as_ref().is_some_and(|combat| !combat.ended) {
                p.healing += u32::from(heal);
            }
        }
        Effect::HealAndReady { heal, .. } => p.healing += u32::from(heal),
        // MKUE p. 11, 3.c : une réserve, dépensée par `SpendHealing` (spec 042, Q1).
        Effect::Heal(amount) => p.healing += u32::from(amount),
        // Spec 058 (Q3) : une réserve à part, pour l'interaction.
        Effect::InteractionInfluence(amount) => p.interaction_influence += u32::from(amount),
        Effect::Influence(amount)
        | Effect::InfluenceReputation { amount, .. }
        | Effect::InfluenceInInteraction { amount, .. } => p.influence_points += i32::from(amount),
        Effect::Attack { .. }
        | Effect::Block { .. }
        | Effect::IceBlockPerAbility(_)
        | Effect::GainToken(_)
        | Effect::GainCrystal(_)
        | Effect::PayForCrystal(_)
        | Effect::SetDice { .. }
        | Effect::Boost(_)
        | Effect::DrawCards(_)
        | Effect::SwiftBlock(_)
        | Effect::ReadyUnit(_)
        | Effect::TokenAndCrystal(_)
        | Effect::GainTokens(..)
        // Spec 053 : Coup de sang et Besoin d'aide arrivent résolus (`played_events`).
        | Effect::Nothing
        | Effect::AttackWithWound { .. }
        | Effect::InfluencePerWound { .. }
        // Spec 056 : écrit avec chaque carte.
        | Effect::WoundCrystalAndToken { .. }
        | Effect::CrystalsFromDestroyed(_)
        | Effect::CrystalsOtherColors
        | Effect::GainOwnedCrystal(_)
        | Effect::CrystalFromDie
        | Effect::Repeat(_)
        | Effect::GainSpell(_)
        | Effect::TrainFromOffer(_)
        // Spec 054 : Déluge de flammes arrive résolu ; les autres, par leurs événements.
        | Effect::AttackPerEnemy { .. }
        | Effect::BlockPerEnemy { .. }
        | Effect::DestroyEnemy
        | Effect::CastFromOffer
        // Spec 057.
        | Effect::WoundsPerCrystal(_)
        | Effect::OthersOnly
        | Effect::HealByTerrain { .. }
        | Effect::HealAndReadyLevels { .. }
        | Effect::ReadyAnyUnit { .. }
        | Effect::UndergroundMove { .. }
        | Effect::FlyMove { .. }
        | Effect::DoesNotAttackPerMove
        | Effect::DiscardToDeck { .. }
        | Effect::ClaimDie { .. }
        // Spec 055 : la Bourse sans fond arrive résolue.
        | Effect::BannerAssigned
        | Effect::InfluenceAndFame { .. }
        | Effect::AmuletOfSun { .. }
        | Effect::AmuletOfDarkness { .. }
        | Effect::TokenCrystalAndFame(_)
        | Effect::CrystalAndToken { .. }
        | Effect::DrawAndToken { .. }
        | Effect::HealUnit
        | Effect::CrystalColorInfluence
        | Effect::MovePerUnit { .. }
        | Effect::DiscardWoundToDraw
        | Effect::ThrowWounds(_)
        | Effect::Flight { .. }
        | Effect::ResistanceBreak
        | Effect::SourceOpening
        | Effect::CalmWeather
        | Effect::ManaOverload
        | Effect::CancelAttacks(_)
        | Effect::ReadyAllUnits
        | Effect::SwordDiscard
        | Effect::HornOfWrath { .. }
        | Effect::GemPouchDice
        | Effect::TokensEachColorAndTime
        | Effect::SpellAndCrystalFromOffer => {}
        // Spec 056 : Tempête de mana renforcée (Q9).
        Effect::RerollSource => {
            p.extra_dice += 3;
            p.any_color_dice = true;
        }
        // Spec 056 : les jetons de Rituel de sang renforcé se choisissent un par un.
        Effect::WoundAndTokens(count) => {
            p.mana_choice = Some(ManaChoice {
                tokens_left: count,
                crystal_offered: false,
            });
        }
        // MKUE p. 5, 3 ; spec 025.
        Effect::ExtraDie => p.extra_dice += 1,
        Effect::ExtraDieBlackAnyColor => {
            p.extra_dice += 1;
            p.black_any_dice += 1;
        }
    }
}

/// L'Attaque ajoutée au combat avec l'élément des Mages d'Altem (`AttacksBecome`), sans effet
/// rétroactif : seulement les Attaques ajoutées après la capacité (Compendium, *Altem Mages* Q2 ;
/// spec 053). Le siège ne change que ce qui se joue (`PlayMoment`), pas les points. Une Attaque
/// physique doublée par l'Épée de justice l'est avant de changer d'élément (Compendium, *Sword of
/// Justice* S6 ; issue privée 272).
fn with_attack_changes(modifiers: &[Modifier], combat: Option<&Combat>, effect: Effect) -> Effect {
    let Effect::Attack {
        amount,
        element: original,
        kind,
    } = effect
    else {
        return effect;
    };
    match modifiers.iter().find_map(|modifier| match modifier.kind {
        ModifierKind::AttacksBecome(element) => Some(element),
        _ => None,
    }) {
        Some(element) if element != original => {
            let doubled =
                original == Element::Physical && combat.is_some_and(Combat::doubles_physical);
            Effect::Attack {
                amount: if doubled {
                    amount.saturating_mul(2)
                } else {
                    amount
                },
                element,
                kind,
            }
        }
        _ => effect,
    }
}

/// Spec 055 (Épée de justice ; Q3) : les ennemis vaincus ce tour, dans les faits du tour.
pub(super) fn enemies_defeated_this_turn(p: &PlayerState) -> usize {
    p.turn_facts
        .iter()
        .map(|fact| match fact {
            crate::TurnFact::EnemiesDefeated { enemies, .. } => enemies.len(),
            _ => 0,
        })
        .sum()
}

/// Spec 055 : une capacité d'Attaque ou de Parade (Bannière de gloire).
pub(super) fn is_attack_or_block(effect: Effect) -> bool {
    matches!(
        effect,
        Effect::Attack { .. } | Effect::Block { .. } | Effect::SwiftBlock(_)
    )
}

/// Spec 055 (Bannière de gloire ; Compendium, *Miscellaneous* S12) : +1 à l'Attaque ou à la Parade
/// de l'unité porteuse, du même élément.
pub(super) fn glory_banner(p: &PlayerState, unit: UnitCard) -> bool {
    p.units.iter().any(|owned| {
        owned.card == unit
            && owned
                .banner
                .is_some_and(|banner| banner.card() == DeedCard::Artifact(Artifact::BannerOfGlory))
    })
}

fn add_modifiers(p: &mut PlayerState, kinds: &[ModifierKind], source: ModifierSource) {
    // Spec 053 : ceux qui visent un ennemi vont sur l'ennemi du combat (`Combat::pose_on_enemy`).
    p.modifiers.extend(
        kinds
            .iter()
            .filter(|kind| kind.is_player_modifier())
            .map(|&kind| Modifier {
                kind,
                source,
                duration: kind.duration(),
            }),
    );
}

/// Retire `card` de l'offre ; `false` si elle n'y est pas.
fn take_from_offer(state: &mut GameState, offer: Offer, card: CommonCard) -> bool {
    let Some(cards) = state.offers.get_mut(&offer) else {
        return false;
    };
    match cards.iter().position(|&held| held == card) {
        Some(index) => {
            cards.remove(index);
            true
        }
        None => false,
    }
}

/// La récompense de la ligne de ce niveau sur la piste de gloire (spec 035).
pub(super) fn level_reward(level: u8) -> LevelReward {
    GameContent::embedded()
        .fame_track()
        .definition(level)
        .reward
}

/// Retire `card` de la main (ou d'une autre liste de cartes) ; `false` si elle n'y est pas.
fn take_from_hand(hand: &mut Vec<CardId>, card: CardId) -> bool {
    match hand.iter().position(|&held| held == card) {
        Some(index) => {
            hand.remove(index);
            true
        }
        None => false,
    }
}
