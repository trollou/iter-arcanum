//! Production des événements par thème, pour `decide` : cartes jouées, effets, paiements,
//! fin du tour et de la manche, récompenses.
//! Un sous-module par thème, comme `legal/` et `apply/`.

use super::*;

mod cards;
mod dummy;
mod effects;
mod payments;
mod round;
mod turn_end;

// Chaque thème partage ses fonctions avec le reste du moteur à la visibilité qu'elles ont :
// un glob ne réexporte un élément qu'à sa propre visibilité, au plus celle du `use`.
pub(super) use cards::*;
pub(super) use dummy::*;
pub(super) use effects::*;
pub(super) use payments::*;
pub(super) use round::*;
pub(super) use turn_end::*;
