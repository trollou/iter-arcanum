//! Sites d'aventure du panneau latéral (spec 044) : entrer et sa confirmation, dé du donjon,
//! cristaux aléatoires et choix de leur couleur, jeton de la clairière.

use arcanum_core::{Feature, Site};

use super::*;
use crate::enemy_text::token_names;

/// Le site imprimé sur cette case, s'il y en a un.
pub(crate) fn site_at(view: &PlayerView, hex: Hex) -> Option<Site> {
    match view.map.space(hex).and_then(|space| space.feature) {
        Some(Feature::Site(site)) => Some(site),
        _ => None,
    }
}

/// Donjon ou tombeau : combat seul et de nuit (cartes de site).
pub(crate) fn is_night_site(site: Site) -> bool {
    matches!(site, Site::Dungeon | Site::Tomb)
}

/// Clé du nom d'un site d'aventure.
fn site_name_key(site: Site) -> &'static str {
    match site {
        Site::Tomb => "site.tomb.name",
        Site::MonsterDen => "site.monster_den.name",
        Site::SpawningGrounds => "site.spawning_grounds.name",
        Site::Ruins => "site.ruins.name",
        _ => "site.dungeon.name",
    }
}

/// Ce qu'on gagne à vaincre les ennemis du site : la gloire seulement pour un donjon ou un tombeau
/// déjà conquis (cartes de site). Partagée avec l'infobulle du plateau (C43).
pub(crate) fn adventure_reward(site: Site, conquered: bool, catalog: &Catalog) -> String {
    catalog.text(match site {
        Site::Dungeon | Site::Tomb if conquered => "adventure.reward.fame_only",
        Site::Tomb => "adventure.reward.tomb",
        Site::MonsterDen => "adventure.reward.monster_den",
        Site::SpawningGrounds => "adventure.reward.spawning_grounds",
        _ => "adventure.reward.dungeon",
    })
}

/// L'entrée : son bouton et les points qu'elle ferait perdre (MKUE p. 6, 6.e), ou sa
/// confirmation (Q4).
pub(super) struct AdventureLines {
    pub button: Option<Button>,
    pub warning: Option<String>,
    pub confirm: Option<AnnouncePrompt>,
}

pub(super) fn adventure_lines(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> AdventureLines {
    let none = AdventureLines {
        button: None,
        warning: None,
        confirm: None,
    };
    let Some(index) = legal_actions
        .iter()
        .position(|action| *action == Action::EnterAdventureSite)
    else {
        return none;
    };
    let Some((hex, site)) = view
        .figure
        .and_then(|hex| site_at(view, hex).map(|site| (hex, site)))
    else {
        return none;
    };
    match mode {
        ScreenMode::ConfirmingAdventure => AdventureLines {
            confirm: Some(AnnouncePrompt {
                text: confirm_text(view, hex, site, catalog),
                buttons: vec![
                    Button {
                        label: catalog.text("adventure.confirm_yes"),
                        command: Command::Play(index),
                        shortcut: None,
                    },
                    Button {
                        label: catalog.text("adventure.cancel"),
                        command: Command::CancelChoice,
                        shortcut: shortcut_label(Shortcut::Cancel, catalog),
                    },
                ],
            }),
            ..none
        },
        ScreenMode::Normal => AdventureLines {
            button: Some(Button {
                label: catalog.format(
                    "adventure.enter",
                    &[("site", &catalog.text(site_name_key(site)))],
                ),
                command: Command::OpenAdventure,
                shortcut: shortcut_label(Shortcut::EnterSite, catalog),
            }),
            warning: (view.move_points > 0 || view.influence_points > 0).then(|| {
                catalog.format(
                    "adventure.lost_points",
                    &[
                        ("move", &view.move_points.to_string()),
                        ("influence", &with_minus(view.influence_points)),
                    ],
                )
            }),
            confirm: None,
        },
        _ => none,
    }
}

/// Ce qui sera combattu (les ennemis restés, puis les jetons à tirer), les règles de Nuit, la
/// récompense, et « Pas de retour en arrière » quand un jeton sera tiré.
fn confirm_text(view: &PlayerView, hex: Hex, site: Site, catalog: &Catalog) -> String {
    if site == Site::Ruins {
        return ruins_confirm_text(view, hex, catalog);
    }
    // Cartes de site : un jeton marron (deux au Sombre Terrier), rouge au tombeau ; l'antre et le
    // Sombre Terrier gardent les leurs, le donjon et le tombeau en tirent un neuf.
    let (color, wanted): (&str, usize) = match site {
        Site::Tomb => ("enemy.pile_color.red", 1),
        Site::SpawningGrounds => ("enemy.pile_color.brown", 2),
        _ => ("enemy.pile_color.brown", 1),
    };
    let waiting: Vec<EnemyToken> = if is_night_site(site) {
        Vec::new()
    } else {
        view.enemy_tokens.get(&hex).cloned().unwrap_or_default()
    };
    let drawn = wanted.saturating_sub(waiting.len());
    let mut lines = Vec::new();
    if !waiting.is_empty() {
        lines.push(catalog.format(
            "adventure.confirm.fight_waiting",
            &[("enemies", &token_names(waiting, catalog))],
        ));
    }
    if drawn > 0 {
        lines.push(catalog.format(
            "adventure.confirm.fight_drawn",
            &[
                ("count", &drawn.to_string()),
                ("color", &catalog.text(color)),
            ],
        ));
    }
    if is_night_site(site) {
        lines.push(catalog.text("adventure.confirm.night_no_units"));
    }
    let mut reward = catalog.format(
        "adventure.confirm.reward",
        &[(
            "reward",
            &adventure_reward(site, view.shields.contains_key(&hex), catalog),
        )],
    );
    if drawn > 0 {
        reward = format!("{reward} {}", catalog.text("adventure.confirm.no_undo"));
    }
    lines.push(reward);
    lines.join("\n")
}

/// Une ruine (spec 045, C35) : les ennemis restés, ou à la première entrée les couleurs du jeton à
/// tirer ; puis la récompense du jeton, et « Pas de retour en arrière » quand on tire.
fn ruins_confirm_text(view: &PlayerView, hex: Hex, catalog: &Catalog) -> String {
    let Some(token) = super::ruins::ruin_token_at(view, hex) else {
        return String::new();
    };
    let waiting = view.enemy_tokens.get(&hex).cloned().unwrap_or_default();
    let drawing = waiting.is_empty();
    let fight = if drawing {
        catalog.format(
            "adventure.confirm.fight_colors",
            &[(
                "colors",
                &super::ruins::enemy_colors_text(token.definition().enemies(), catalog),
            )],
        )
    } else {
        catalog.format(
            "adventure.confirm.fight_waiting",
            &[("enemies", &token_names(waiting, catalog))],
        )
    };
    let mut reward = catalog.format(
        "adventure.confirm.reward",
        &[("reward", &super::ruins::token_reward_text(token, catalog))],
    );
    if drawing {
        reward = format!("{reward} {}", catalog.text("adventure.confirm.no_undo"));
    }
    format!("{fight}\n{reward}")
}

/// « Dé de récompense : vert → un artefact », après la conquête d'un donjon, jusqu'à la fin du
/// tour (carte du donjon).
pub(super) fn reward_die_line(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.reward_die.map(|color| {
        let reward = if matches!(color, ManaColor::Gold | ManaColor::Black) {
            "reward.die_spell"
        } else {
            "reward.die_artifact"
        };
        catalog.format(
            "reward.die",
            &[
                ("color", &mana_name(color, catalog)),
                ("reward", &catalog.text(reward)),
            ],
        )
    })
}

/// « Récompense en fin de tour : 2 cristal(aux) aléatoire(s) », tant que les dés attendent.
pub(super) fn crystal_reward_pending(view: &PlayerView, catalog: &Catalog) -> Option<String> {
    (view.crystal_rewards > 0).then(|| {
        catalog.format(
            "reward.pending_crystals",
            &[("count", &view.crystal_rewards.to_string())],
        )
    })
}

/// La fenêtre du choix de la couleur du cristal, après un dé or (Compendium, *End of Turn* Q4) :
/// les quatre couleurs de base, grisées si pleines.
pub(super) fn crystal_choice(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<CrystalChoice> {
    (view.crystal_choices > 0).then(|| CrystalChoice {
        title: catalog.text("reward.choose_crystal"),
        colors: BasicColor::ALL
            .into_iter()
            .map(|color| CrystalButton {
                color,
                label: mana_name(ManaColor::Basic(color), catalog),
                command: legal_actions
                    .iter()
                    .position(|action| *action == Action::ChooseCrystalColor { color })
                    .map(Command::Play),
                // Spec 071, C22 : 1 à 4 dans l'ordre rouge, bleu, blanc, vert, grisé compris.
                shortcut: BasicColor::ALL
                    .iter()
                    .position(|other| *other == color)
                    .and_then(|place| crate::take_label(place + 1, catalog)),
            })
            .collect(),
    })
}

/// La couleur du jeton de la clairière : or le jour, noir la nuit (carte de la clairière).
fn glade_color(view: &PlayerView) -> ManaColor {
    match view.time_of_day {
        TimeOfDay::Day => ManaColor::Gold,
        TimeOfDay::Night => ManaColor::Black,
    }
}

/// La figurine est sur une clairière magique.
pub(super) fn on_glade(view: &PlayerView) -> bool {
    view.figure
        .and_then(|hex| site_at(view, hex))
        .is_some_and(|site| site == Site::MagicalGlade)
}

/// « Clairière magique : +1 jeton de mana or », au début du tour, tant que rien n'a été décidé
/// (comme les bandeaux de niveau, spec 035).
pub(super) fn glade_token(view: &PlayerView, playing: bool, catalog: &Catalog) -> Option<String> {
    let untouched = view.play_area.is_empty() && view.dice_taken.is_empty() && view.can_rest;
    let color = glade_color(view);
    (playing && untouched && on_glade(view) && view.mana_tokens.contains(&color))
        .then(|| catalog.format("glade.token", &[("color", &mana_name(color, catalog))]))
}
