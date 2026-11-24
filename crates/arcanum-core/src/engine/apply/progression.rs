//! `apply` : renommée, niveaux, réputation et compétences.

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn progression_event(state: &mut GameState, event: &Event) {
    match event {
        Event::FameGained { player, fame } => {
            if let Some(p) = state.player_mut(*player) {
                p.fame += fame;
            }
        }
        Event::LevelGained { player, level } => {
            // Spec 050 : un niveau pair donne une action avancée, s'il y en a dans l'offre (L1).
            let offer_has_cards = !state.offer(Offer::AdvancedActions).is_empty();
            if let Some(p) = state.player_mut(*player) {
                p.level = (*level).clamp(1, MAX_LEVEL);
                p.levels_gained.push(p.level);
                if level_reward(p.level) == LevelReward::SkillAndAdvancedAction {
                    if offer_has_cards {
                        p.advanced_action_choices += 1;
                    }
                    // Spec 051 : deux compétences à révéler, si la pile les a.
                    if p.skill_pile.len() > 2 * usize::from(p.skill_choices) {
                        p.skill_choices += 1;
                    }
                }
            }
        }
        Event::ReputationChanged { player, change } => {
            if let Some(p) = state.player_mut(*player) {
                p.reputation = p.reputation.changed(*change);
                // Spec 040 : dans l'historique du combat, après l'attaque qui l'a rapportée.
                if let Some(combat) = p.combat.as_mut().filter(|combat| !combat.ended) {
                    combat.record_reputation(*change);
                }
            }
        }
        // Spec 051 : les deux du dessus quittent la pile, en attente du choix.
        Event::SkillsRevealed { player, skills } => {
            if let Some(p) = state.player_mut(*player) {
                p.skill_pile.retain(|skill| !skills.contains(skill));
                p.skill_choice = skills.clone();
            }
        }
        // MKUE p. 5, *Using Skills* 2.a : face visible, prête.
        Event::SkillGained { player, skill } => {
            if let Some(p) = state.player_mut(*player)
                && !p.skills.iter().any(|owned| owned.skill == *skill)
            {
                p.skills.push(OwnedSkill {
                    skill: *skill,
                    state: SkillState::Ready,
                });
                p.skill_choices = p.skill_choices.saturating_sub(1);
                p.skill_choice.clear();
            }
        }
        // MKUE p. 11, 6.b : l'action avancée sera la dernière de l'offre.
        Event::SkillTakenFromCommonOffer { player, skill } => {
            state.common_skills.retain(|common| common != skill);
            if let Some(p) = state.player_mut(*player) {
                p.lowest_advanced_action_only = true;
            }
        }
        // MKUE p. 5, *Using Skills* 4 : utilisée ce tour, ou retournée jusqu'à la manche suivante ;
        // ses points s'ajoutent à ceux des cartes, sans être une carte jouée (Compendium, *Skill
        // Tokens* S1).
        Event::SkillUsed {
            player,
            skill,
            option,
            effect,
        } => apply_skill_used(state, player, skill, option, effect),
        // Spec 058 (MKUE p. 5, 4.b) : au centre, son avantage attend le tour suivant.
        Event::SkillPlacedInCenter {
            player,
            skill,
            color,
        } => {
            if let Some(p) = state.player_mut(*player) {
                if let Some(owned) = p.skills.iter_mut().find(|owned| owned.skill == *skill) {
                    owned.state = SkillState::InCenter;
                }
                p.center_advantages.push(CenterAdvantage {
                    skill: *skill,
                    color: *color,
                    active: false,
                    used: false,
                });
            }
        }
        Event::CenterAdvantageUsed { player, skill } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(advantage) = p
                    .center_advantages
                    .iter_mut()
                    .find(|advantage| advantage.skill == *skill)
            {
                advantage.used = true;
                // Compendium, *SOURCE Opening* S4 : un dé de plus, qui ne prend pas celui du tour.
                if *skill == Skill::SourceOpening {
                    p.extra_dice += 1;
                    p.opening_die_due = true;
                }
            }
        }
        // MKUE p. 4, 2.g : retournée jusqu'à la manche suivante.
        Event::CenterSkillReturned { player, skill } => {
            if let Some(p) = state.player_mut(*player) {
                if let Some(owned) = p.skills.iter_mut().find(|owned| owned.skill == *skill) {
                    owned.state = SkillState::Flipped;
                }
                p.center_advantages
                    .retain(|advantage| advantage.skill != *skill);
            }
        }
        Event::SkillToCommonOffer { skill } => {
            if !state.common_skills.contains(skill) {
                state.common_skills.push(*skill);
            }
        }
        Event::SkillPileShuffled { player, order } => {
            if let Some(p) = state.player_mut(*player) {
                p.skill_pile = order.clone();
            }
        }
        _ => {}
    }
}

/// `Event::SkillUsed` : la compétence utilisée et son effet.
fn apply_skill_used(
    state: &mut GameState,
    player: &PlayerId,
    skill: &Skill,
    option: &u8,
    effect: &Effect,
) {
    if let Some(p) = state.player_mut(*player) {
        let Some(owned) = p.skills.iter_mut().find(|owned| owned.skill == *skill) else {
            return;
        };
        owned.state = match skill.definition().usage {
            SkillUsage::Round => SkillState::Flipped,
            SkillUsage::Turn | SkillUsage::Permanent => SkillState::UsedThisTurn,
        };
        // Texte de Motivation : pas d'autre jusqu'à la fin du tour suivant.
        if skill.is_motivation() {
            p.motivation_block = 2;
        }
        // Compendium, *Who Needs Magic* Q1 : après +3, plus de dé de la Source ce tour.
        if *skill == Skill::WhoNeedsMagic && *option == 1 {
            p.source_dice_forbidden = true;
        }
        // Texte de Temps calme : tous les terrains −2 ce tour.
        if *effect == Effect::CalmWeather {
            p.modifiers.push(calm_weather(CALM_NOW));
        }
        add_effect_points(p, *effect);
        let effect = with_attack_changes(&p.modifiers, p.combat.as_ref(), *effect);
        if let Some(combat) = p.combat.as_mut() {
            combat.add_converted(effect);
        }
    }
}
