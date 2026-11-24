//! Décision : une action validée devient des événements (`decide`), sans toucher à l'état.
//! La validation, l'aiguillage et les suites communes sont ici ; chaque thème décide de ses
//! actions dans son sous-module, comme `apply/` et `legal/`.

use super::*;

mod cards;
mod combat;
mod mana;
mod map;
mod progression;
mod sites;
mod turn;
mod units;

use cards::*;
use combat::*;
use mana::*;
use map::*;
use progression::*;
use sites::*;
use turn::*;
use units::*;

/// Valide l'action et produit ses événements, sans toucher à l'état.
pub fn decide(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    validate(state, player, action)?;
    // Aiguillage exhaustif : une nouvelle action doit être rangée dans un thème.
    let events = match action {
        Action::PlayCard { .. }
        | Action::PlayCardOnUnit { .. }
        | Action::PlaceOnDeck { .. }
        | Action::LearnAdvancedAction { .. }
        | Action::SpendHealing { .. } => card_actions(state, player, action, rngs),
        Action::ChooseTokenColor { .. }
        | Action::PayForCrystal { .. }
        | Action::DeclineCrystal
        | Action::ChooseCrystalColor { .. }
        | Action::KeepSourceDie => mana_actions(state, player, action, rngs),
        Action::MoveHero { .. } | Action::FollowRoute { .. } | Action::Explore { .. } => {
            map_actions(state, player, action, rngs)
        }
        Action::AnnounceEndOfRound
        | Action::ChooseTactic { .. }
        | Action::RethinkDiscard { .. }
        | Action::PrepareCard { .. }
        | Action::StealDie { .. }
        | Action::ManaSearch { .. }
        | Action::MidnightMeditation { .. }
        | Action::SparingPower { .. }
        | Action::UseLongNight
        | Action::DeclineLongNight
        | Action::AnnounceRightMoment
        | Action::EndTurn { .. }
        | Action::ReclaimBanners { .. } => turn_actions(state, player, action, rngs),
        Action::Challenge { .. }
        | Action::StartCombat
        | Action::NextCombatPhase
        | Action::EndCombat
        | Action::DeclareAttack { .. }
        | Action::ResolveAttack
        | Action::DeclareBlock { .. }
        | Action::ResolveBlock
        | Action::AssignDamage { .. }
        | Action::SpendMoveAsAttack
        | Action::SpendInfluenceAsBlock
        | Action::UseDeferredAttack => combat_actions(state, player, action, rngs),
        Action::Interact
        | Action::RecruitUnit { .. }
        | Action::BuyHealing
        | Action::BuyAdvancedActionFromDeck
        | Action::BuyArtifact
        | Action::AddEliteUnit
        | Action::BuyAdvancedAction { .. }
        | Action::BuySpell { .. }
        | Action::PlunderVillage
        | Action::EnterAdventureSite
        | Action::BurnMonastery
        | Action::PayAltarTribute { .. } => site_actions(state, player, action, rngs),
        Action::UseSkill { .. }
        | Action::ChooseSkill { .. }
        | Action::GainAdvancedAction { .. }
        | Action::TakeSpell { .. }
        | Action::TakeArtifact { .. }
        | Action::TakeUnitReward { .. }
        | Action::DeclineUnitReward => progression_actions(state, player, action, rngs),
        Action::ActivateUnit { .. } | Action::UseBanner { .. } | Action::AssignBanner { .. } => {
            unit_actions(state, player, action)
        }
    }?;
    Ok(with_grail(
        state,
        with_turn_fame(
            state,
            with_banner_losses(
                state,
                with_glade_token(
                    state,
                    player,
                    with_round_end(state, player, with_dummy_turns(state, player, events), rngs),
                ),
            ),
        ),
    ))
}

/// L'action est-elle celle du joueur attendu, et légale ?
fn validate(state: &GameState, player: PlayerId, action: &Action) -> Result<(), ActionError> {
    let decision = pending_decision(state).ok_or(ActionError::NotLegal)?;
    if decision.player != player {
        return Err(ActionError::WrongPlayer { player });
    }
    let legal = legal_actions(state, player);
    let accepted = match action {
        // Spec 034 : les Blessures étant identiques, une fin de tour en nomme d'autres que les
        // actions légales ; elle est légale si une fin de tour l'est et si sa défausse l'est.
        Action::EndTurn {
            discard,
            reclaim,
            glade,
            steady_tempo,
        } => {
            let after = after_forced_withdrawal(state, player);
            let p = after.player(player).ok_or(ActionError::NotLegal)?;
            legal
                .iter()
                .any(|action| matches!(action, Action::EndTurn { .. }))
                && end_turn_allowed(
                    &p.hand,
                    &p.play_area,
                    p.played_elsewhere,
                    !p.moved_or_fought,
                    discard,
                    *reclaim,
                )
                && glade_allowed(&after, p, discard, *reclaim, *glade)
                && (!*steady_tempo || steady_tempo_keep(p).is_some())
        }
        _ => legal.contains(action),
    };
    if accepted {
        Ok(())
    } else {
        Err(ActionError::NotLegal)
    }
}
