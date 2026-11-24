//! Joueur automatique « premier choix » des tests (bilan de la spec 033, issue privée 77) : une seule
//! version, partagée par les tests d'arcanum-core et lue par arcanum-ui par
//! `#[path = "../../arcanum-core/tests/premier_choix/mod.rs"] mod premier_choix;`.

use arcanum_core::{Action, Feature, Hex, Player, PlayerView, Site};

/// Joueur qui prend la première action légale, sauf défier (`Action::Challenge`, spec 031),
/// interagir (`Action::Interact`, spec 040 : sinon il interagirait au village au lieu de finir son
/// tour, et ses parties changeraient), incendier un monastère et piller un village
/// (`Action::BurnMonastery`, `Action::PlunderVillage`, spec 041, pour la même raison), entrer dans
/// un site d'aventure (`Action::EnterAdventureSite`, spec 044, ruines comprises), faire l'offrande
/// d'un autel (`Action::PayAltarTribute`, spec 045), et assaillir une forteresse ou une tour de mage non conquise (`MoveHero`, `FollowRoute`, spec 043 :
/// ses parcours ne changent pas).
///
/// Pourquoi éviter le défi : défier à chaque tour donne des Blessures sans fin (2 par tour, et
/// depuis la spec 033 les Invocateurs orques blessent sans être vaincus), le deck ne se vide plus
/// et la partie ne finit jamais. S'il ne reste que des défis ou l'interaction, il prend la première action.
#[derive(Default)]
pub struct FirstChoice {
    choice: Option<usize>,
}

impl Player for FirstChoice {
    fn request(&mut self, view: &PlayerView, legal_actions: &[Action]) {
        self.choice = legal_actions
            .iter()
            .position(|action| {
                !matches!(
                    action,
                    Action::Challenge { .. }
                        | Action::Interact
                        | Action::BurnMonastery
                        | Action::PlunderVillage
                        | Action::EnterAdventureSite
                        | Action::PayAltarTribute { .. }
                ) && !is_assault(view, action)
            })
            .or(Some(0));
    }

    fn poll(&mut self) -> Option<usize> {
        self.choice.take()
    }
}

/// Vrai si l'action finit sur une forteresse ou une tour de mage non conquise (spec 043).
fn is_assault(view: &PlayerView, action: &Action) -> bool {
    let to: Hex = match action {
        Action::MoveHero { to } | Action::FollowRoute { to } => *to,
        _ => return false,
    };
    let fortified = matches!(
        view.map.space(to).and_then(|space| space.feature),
        Some(Feature::Site(Site::Keep | Site::MageTower))
    );
    fortified && !view.shields.contains_key(&to)
}
