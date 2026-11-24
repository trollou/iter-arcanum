//! Texte des ennemis à combattre (spec 030), infobulle d'« Engager le combat » (spec 075).

use arcanum_core::PlayerView;
use arcanum_i18n::Catalog;

use crate::enemy_text::token_names;

/// « À combattre : … » tant que des ennemis provoqués ou défiés attendent le combat (spec 030, Q1 ;
/// spec 031).
pub(super) fn provoked_banner(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    let tokens = view
        .enemies_to_fight
        .iter()
        .filter_map(|hex| view.enemy_tokens.get(hex))
        .flatten()
        .copied();
    (!view.enemies_to_fight.is_empty()).then(|| {
        catalog.format(
            "combat.to_fight",
            &[("enemies", &token_names(tokens, catalog))],
        )
    })
}
