//! Gloire et niveau du panneau latéral, bandeau du changement de niveau (spec 035).

use arcanum_core::{GameContent, LevelReward, PlayerView, level_token};
use arcanum_i18n::Catalog;

use super::IconLine;
use crate::{Icon, icon_value};

/// L'icône Armure et l'armure du héros, puis « Limite de main 5 · Jetons de commandement 2 »
/// (spec 071, C5).
pub(super) fn level_stats(view: &PlayerView, catalog: &Catalog) -> IconLine {
    IconLine {
        icons: vec![icon_value(Icon::Armor, view.armor.to_string(), catalog)],
        text: catalog.format(
            "level.stats",
            &[
                ("hand_limit", &view.hand_limit.to_string()),
                ("command", &view.command_tokens.to_string()),
            ],
        ),
    }
}

/// Un bandeau par niveau gagné à la dernière fin de tour, tant que rien n'a été décidé ce tour :
/// ni carte jouée, ni dé pris, ni déplacement ou combat (comme `round.new`).
pub(super) fn level_gained_banners(
    view: &PlayerView,
    playing: bool,
    catalog: &Catalog,
) -> Vec<String> {
    let untouched = view.play_area.is_empty() && view.dice_taken.is_empty() && view.can_rest;
    if !playing || !untouched {
        return Vec::new();
    }
    let track = GameContent::embedded().fame_track();
    view.levels_gained
        .iter()
        .map(|&level| {
            let number = level.to_string();
            match track.definition(level).reward {
                LevelReward::SkillAndAdvancedAction => {
                    catalog.format("level.gained_even", &[("level", &number)])
                }
                LevelReward::LevelToken | LevelReward::Start => {
                    let token = level_token(level);
                    catalog.format(
                        "level.gained",
                        &[
                            ("level", &number),
                            ("armor", &token.armor.to_string()),
                            ("hand_limit", &token.hand_limit.to_string()),
                        ],
                    )
                }
            }
        })
        .collect()
}
