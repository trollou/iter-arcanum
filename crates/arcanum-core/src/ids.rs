use crate::{Artifact, CardPlay, DeedCard, Effect, GameContent, ModifierKind};

/// Identifiant d'un joueur. Existe dès le solo pour ne pas le figer.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct PlayerId(pub u8);

impl PlayerId {
    /// Le seul joueur d'une partie solo.
    pub const SOLO: PlayerId = PlayerId(0);

    pub(crate) fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// Un exemplaire de carte : la carte et son numéro d'exemplaire (Marche n° 1, Marche n° 2).
/// L'identifiant suffit à connaître la carte, sans table dans l'état (spec 013).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CardId {
    card: DeedCard,
    copy: u16,
}

impl CardId {
    pub fn new(card: DeedCard, copy: u16) -> Self {
        Self { card, copy }
    }

    pub fn card(self) -> DeedCard {
        self.card
    }

    pub fn copy(self) -> u16 {
        self.copy
    }

    /// L'effet obtenu en jouant cette carte ainsi, d'après sa définition : la seule source de ce
    /// calcul pour le moteur, l'écran, les joueurs automatiques et les tests (issue privée 42).
    /// Une Blessure n'en a aucun, pas même de côté (MKUE p. 4, 3.d).
    pub fn effect(self, play: CardPlay) -> Option<Effect> {
        // MKUE p. 4, 3.e : toute carte sauf une Blessure se joue de côté, un artefact aussi, qui
        // n'a pas de définition de couleur (spec 041, C27).
        if let CardPlay::Sideways(sideways) = play {
            return (!self.card.is_wound()).then(|| sideways.effect());
        }
        // Spec 058 : la valeur remplacée par la compétence, Blessure comprise.
        if let CardPlay::SidewaysSkill {
            sideways,
            skill,
            option,
            mana,
        } = play
        {
            return skill
                .sideways_value(option, self.card, mana.map(|mana| mana.color))
                .map(|amount| sideways.effect_with(amount));
        }
        // Spec 055 : un artefact n'a pas de couleur ; ses effets sont lus à part.
        if let DeedCard::Artifact(artifact) = self.card {
            return artifact_effect(artifact, play);
        }
        let definition = self.card.definition()?;
        match play {
            CardPlay::Basic { option, .. } => definition.basic.option(option),
            CardPlay::Sideways(sideways) => Some(sideways.effect()),
            CardPlay::SidewaysSkill { .. } => None,
            CardPlay::Strong { option, .. } | CardPlay::Boosted { option, .. } => {
                definition.strong.option(option)
            }
            // Spec 054 : renforcé si payé du noir.
            CardPlay::Cast { option, black, .. } => match black {
                Some(_) => definition.strong.option(option),
                None => definition.basic.option(option),
            },
            CardPlay::Artifact { .. } | CardPlay::ThrowAway { .. } => None,
        }
    }

    /// Les modificateurs posés en jouant cette carte ainsi (spec 052, ADR 0008) : ceux de l'option
    /// jouée ; aucun de côté.
    pub fn modifiers(self, play: CardPlay) -> &'static [ModifierKind] {
        if let DeedCard::Artifact(artifact) = self.card {
            return artifact_modifiers(artifact, play);
        }
        let Some(definition) = self.card.definition() else {
            return &[];
        };
        match play {
            CardPlay::Sideways(_) | CardPlay::SidewaysSkill { .. } => &[],
            CardPlay::Basic { option, .. } => definition.basic.option_modifiers(option),
            CardPlay::Strong { option, .. } | CardPlay::Boosted { option, .. } => {
                definition.strong.option_modifiers(option)
            }
            CardPlay::Cast { option, black, .. } => match black {
                Some(_) => definition.strong.option_modifiers(option),
                None => definition.basic.option_modifiers(option),
            },
            CardPlay::Artifact { .. } | CardPlay::ThrowAway { .. } => &[],
        }
    }
}

/// L'effet d'un artefact joué ainsi (spec 055) : son effet de base, ou son effet renforcé en le
/// détruisant.
fn artifact_effect(artifact: Artifact, play: CardPlay) -> Option<Effect> {
    let (basic, strong) = GameContent::embedded().artifact(artifact);
    match play {
        CardPlay::Artifact { option, .. } => basic.option(option),
        CardPlay::ThrowAway { option, .. } => strong.option(option),
        _ => None,
    }
}

/// Les modificateurs posés par un artefact joué ainsi (spec 055).
fn artifact_modifiers(artifact: Artifact, play: CardPlay) -> &'static [ModifierKind] {
    let (basic, strong) = GameContent::embedded().artifact(artifact);
    match play {
        CardPlay::Artifact { option, .. } => basic.option_modifiers(option),
        CardPlay::ThrowAway { option, .. } => strong.option_modifiers(option),
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasicAction, SidewaysEffect};

    fn rage() -> CardId {
        CardId::new(DeedCard::Basic(BasicAction::Rage), 1)
    }

    // MKUE p. 4, 3.a : l'effet de base, au choix parmi ses options.
    #[test]
    fn basic_play_gives_the_chosen_option_of_the_basic_effect() {
        let play = |option| CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option,
            discard: None,
            mana: None,
        };
        assert_eq!(
            rage().effect(play(1)),
            BasicAction::Rage.definition().basic.option(1)
        );
        assert!(rage().effect(play(1)).is_some());
        assert_eq!(rage().effect(play(2)), None);
    }

    // MKUE p. 4, 3.a : l'effet renforcé (ici obtenu par Concentration, spec 025).
    #[test]
    fn boosted_play_gives_the_option_of_the_strong_effect() {
        let play = CardPlay::Boosted {
            option: 0,
            discard: None,
            set_dice: [None, None],
        };
        assert_eq!(
            rage().effect(play),
            BasicAction::Rage.definition().strong.option(0)
        );
        assert!(rage().effect(play).is_some());
    }

    // MKUE p. 4, 3.e : jouée de côté, la carte vaut 1 point.
    #[test]
    fn sideways_play_gives_one_point() {
        let play = CardPlay::Sideways(SidewaysEffect::Move);
        assert_eq!(rage().effect(play), Some(Effect::Move(1)));
    }
}
