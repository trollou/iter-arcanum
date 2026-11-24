//! Page de référence des coordonnées du bac à sable (issue privée 43) : `docs/reference/bac-a-sable.md`
//! et son schéma `docs/reference/bac-a-sable.svg` sont générés ici depuis la mise en place réelle,
//! pour ne pas dériver de `content/tiles.ron` ni de la géométrie des emplacements.
//!
//! Si le test échoue après un changement voulu (tuiles, scénario), régénérer avec
//! `ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-core --test reference_bac_a_sable` et relire le diff.

use std::path::PathBuf;

use arcanum_core::{
    BasicColor, Direction, Feature, GameConfig, GameState, Hero, Hex, Rampaging, Scenario, Site,
    SpaceContent, Terrain, TileId, TileKind, TilePosition, apply, new_game, tile,
};

const BEGIN: &str =
    "<!-- Généré par crates/arcanum-core/tests/reference_bac_a_sable.rs : début -->";
const END: &str = "<!-- Généré : fin -->";

/// Emplacements décrits : la forme en coin jusqu'à trois pas de la tuile de départ.
const MAX_STEPS: i32 = 3;

fn docs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("reference")
}

/// Mise en place du bac à sable : elle ne dépend pas de la graine pour le plateau de jeu
/// ni pour les tuiles de campagne (ordre par numéro, MKUE p. 18).
fn sandbox() -> GameState {
    let config = GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    };
    let mut rng = config.rngs();
    new_game(&config, &mut rng)
        .iter()
        .fold(GameState::default(), apply)
}

fn position_name(i: i32, j: i32) -> String {
    let term = |n: i32, letter: &str| match n {
        0 => String::new(),
        1 => letter.to_string(),
        _ => format!("{n}{letter}"),
    };
    match (term(i, "A"), term(j, "B")) {
        (a, b) if a.is_empty() && b.is_empty() => "départ".to_string(),
        (a, b) if b.is_empty() => a,
        (a, b) if a.is_empty() => b,
        (a, b) => format!("{a} + {b}"),
    }
}

fn wedge_positions() -> Vec<(i32, i32)> {
    (0..=MAX_STEPS)
        .flat_map(|steps| (0..=steps).rev().map(move |i| (i, steps - i)))
        .collect()
}

fn hex(h: Hex) -> String {
    format!("`({}, {})`", h.q, h.r)
}

fn tile_name(id: TileId) -> String {
    match id {
        TileId::StartingA => "départ A".to_string(),
        TileId::StartingB => "départ B".to_string(),
        TileId::Countryside(n) => format!("campagne {n}"),
        TileId::Core(n) => format!("royaume {n}"),
    }
}

fn direction_name(direction: Option<Direction>) -> &'static str {
    match direction {
        None => "centre",
        Some(Direction::E) => "E",
        Some(Direction::NE) => "N-E",
        Some(Direction::NW) => "N-O",
        Some(Direction::W) => "O",
        Some(Direction::SW) => "S-O",
        Some(Direction::SE) => "S-E",
    }
}

fn terrain_name(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::Plains => "Plaine",
        Terrain::Hills => "Colline",
        Terrain::Forest => "Forêt",
        Terrain::Wasteland => "Terre désolée",
        Terrain::Desert => "Désert",
        Terrain::Swamp => "Marais",
        Terrain::Lake => "Lac",
        Terrain::Mountain => "Montagne",
        Terrain::Sea => "Littoral",
    }
}

fn color_name(color: BasicColor) -> &'static str {
    match color {
        BasicColor::Red => "rouge",
        BasicColor::Blue => "bleue",
        BasicColor::White => "blanche",
        BasicColor::Green => "verte",
    }
}

fn feature_name(feature: Feature) -> String {
    match feature {
        Feature::Site(site) => match site {
            Site::Portal => "portail".to_string(),
            Site::Village => "village".to_string(),
            Site::Monastery => "monastère".to_string(),
            Site::MagicalGlade => "clairière magique".to_string(),
            Site::Mine(color) => format!("mine {}", color_name(color)),
            Site::Keep => "forteresse".to_string(),
            Site::MageTower => "tour de mage".to_string(),
            Site::Dungeon => "donjon".to_string(),
            Site::Tomb => "tombeau".to_string(),
            Site::MonsterDen => "antre".to_string(),
            Site::SpawningGrounds => "sombre terrier".to_string(),
            Site::Ruins => "ruines".to_string(),
            Site::City(color) => format!("cité {}", color_name(color)),
        },
        Feature::Rampaging(Rampaging::OrcMarauders) => "orques maraudeurs (errants)".to_string(),
        Feature::Rampaging(Rampaging::Draconum) => "draconien (errant)".to_string(),
    }
}

fn content_name(content: SpaceContent) -> String {
    match content.feature {
        None => terrain_name(content.terrain).to_string(),
        Some(feature) => format!(
            "{}, {}",
            terrain_name(content.terrain),
            feature_name(feature)
        ),
    }
}

/// Les sept cases d'une tuile avec leur contenu : centre, puis l'ordre de `Direction::ALL`.
fn spaces(id: TileId) -> Vec<(Option<Direction>, SpaceContent)> {
    let definition = tile(id).expect("tuile du jeu de base");
    std::iter::once((None, definition.center))
        .chain(
            Direction::ALL
                .into_iter()
                .map(|direction| (Some(direction), definition.side(direction))),
        )
        .collect()
}

fn hex_of(position: TilePosition, direction: Option<Direction>) -> Hex {
    direction.map_or(position.center(), |direction| position.hex_at(direction))
}

fn generated_markdown(state: &GameState) -> String {
    let mut out = String::new();
    out.push_str("### Emplacements de la forme en coin\n\n");
    out.push_str("Cases de chaque emplacement `(i, j)` = `i A + j B`, jusqu'à trois pas de la tuile de départ.\n\n");
    out.push_str("| Emplacement | `(i, j)` | Centre | E | N-E | N-O | O | S-O | S-E |\n");
    out.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for (i, j) in wedge_positions() {
        let position = TilePosition::from_lattice(i, j);
        let cells: Vec<String> = position.hexes().into_iter().map(hex).collect();
        out.push_str(&format!(
            "| {} | `({i}, {j})` | {} |\n",
            position_name(i, j),
            cells.join(" | ")
        ));
    }

    out.push_str("\n### Cases posées à la mise en place\n\n");
    out.push_str("| Case | Tuile | Emplacement | Côté | Terrain, élément |\n");
    out.push_str("|---|---|---|---|---|\n");
    for (position, id) in state.map().tiles() {
        let (i, j) = position.lattice();
        for (direction, content) in spaces(id) {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                hex(hex_of(position, direction)),
                tile_name(id),
                position_name(i, j),
                direction_name(direction),
                content_name(content)
            ));
        }
    }

    out.push_str("\n### Tuiles de campagne de la pioche, dans l'ordre\n\n");
    out.push_str("La première est le dessus de la pioche. Les cases d'une tuile posée à l'emplacement `(i, j)` \
sont celles de la ligne de cet emplacement, côté par côté.\n\n");
    out.push_str("| Tuile | Centre | E | N-E | N-O | O | S-O | S-E |\n");
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    for id in state
        .tile_deck()
        .tiles()
        .iter()
        .filter(|id| id.kind() == TileKind::Countryside)
    {
        let cells: Vec<String> = spaces(*id)
            .into_iter()
            .map(|(_, content)| content_name(content))
            .collect();
        out.push_str(&format!("| {} | {} |\n", tile_name(*id), cells.join(" | ")));
    }
    out
}

// Schéma : cases à pointe en haut, `x = √3 (q + r/2)`, `y = 3/2 r`, en rayons.
const SIZE: f64 = 34.0;

fn pixel(h: Hex) -> (f64, f64) {
    let q = f64::from(h.q);
    let r = f64::from(h.r);
    (SIZE * 3f64.sqrt() * (q + r / 2.0), SIZE * 1.5 * r)
}

fn polygon(h: Hex) -> String {
    let (x, y) = pixel(h);
    let points: Vec<String> = (0..6)
        .map(|k| {
            let angle = std::f64::consts::PI / 180.0 * (60.0 * f64::from(k) - 30.0);
            format!(
                "{:.1},{:.1}",
                x + SIZE * angle.cos(),
                y + SIZE * angle.sin()
            )
        })
        .collect();
    points.join(" ")
}

fn fill(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::Plains => "#e9e3b4",
        Terrain::Hills => "#d9b98a",
        Terrain::Forest => "#a8c99a",
        Terrain::Wasteland => "#c9c2b6",
        Terrain::Desert => "#f0d79a",
        Terrain::Swamp => "#a9bfae",
        Terrain::Lake => "#a9cbe6",
        Terrain::Mountain => "#b3aba4",
        Terrain::Sea => "#7fa7c9",
    }
}

/// Libellé court d'un élément pour tenir dans une case.
fn short_feature(feature: Feature) -> String {
    match feature {
        Feature::Site(Site::MagicalGlade) => "clairière".to_string(),
        Feature::Site(Site::MageTower) => "tour de mage".to_string(),
        Feature::Rampaging(Rampaging::OrcMarauders) => "orques".to_string(),
        Feature::Rampaging(Rampaging::Draconum) => "draconien".to_string(),
        other => feature_name(other),
    }
}

fn generated_svg(state: &GameState) -> String {
    let placed: Vec<(TilePosition, TileId)> = state.map().tiles().collect();
    let free: Vec<(i32, i32)> = wedge_positions()
        .into_iter()
        .filter(|(i, j)| i + j <= 2)
        .filter(|(i, j)| {
            state
                .map()
                .tile_at(TilePosition::from_lattice(*i, *j))
                .is_none()
        })
        .collect();

    let all_hexes: Vec<Hex> = placed
        .iter()
        .map(|(position, _)| *position)
        .chain(free.iter().map(|(i, j)| TilePosition::from_lattice(*i, *j)))
        .flat_map(|position| position.hexes())
        .collect();
    let xs: Vec<f64> = all_hexes.iter().map(|h| pixel(*h).0).collect();
    let ys: Vec<f64> = all_hexes.iter().map(|h| pixel(*h).1).collect();
    let fold_min = |values: &[f64]| values.iter().copied().fold(f64::INFINITY, f64::min);
    let fold_max = |values: &[f64]| values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let margin = SIZE + 10.0;
    let left = fold_min(&xs) - margin;
    let top = fold_min(&ys) - margin - 30.0;
    let width = fold_max(&xs) - fold_min(&xs) + 2.0 * margin;
    let height = fold_max(&ys) - fold_min(&ys) + 2.0 * margin + 50.0;

    let mut out = String::new();
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{left:.1} {top:.1} {width:.1} {height:.1}\" width=\"{:.0}\" font-family=\"sans-serif\" text-anchor=\"middle\">\n",
        width * 1.4
    ));
    out.push_str("<!-- Généré par crates/arcanum-core/tests/reference_bac_a_sable.rs : ne pas modifier à la main. -->\n");
    out.push_str(&format!(
        "<rect x=\"{left:.1}\" y=\"{top:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" fill=\"#fbfcfa\"/>\n"
    ));
    out.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"14\" font-weight=\"700\" fill=\"#1d2420\" text-anchor=\"start\">Bac à sable après la mise en place : cases (q, r)</text>\n",
        left + 10.0,
        top + 20.0
    ));
    out.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"11\" fill=\"#4a524c\" text-anchor=\"start\">Pointillés : emplacements libres à un ou deux pas, nommés i A + j B.</text>\n",
        left + 10.0,
        top + height - 12.0
    ));

    for (i, j) in &free {
        let position = TilePosition::from_lattice(*i, *j);
        for h in position.hexes() {
            let (x, y) = pixel(h);
            out.push_str(&format!(
                "<polygon points=\"{}\" fill=\"#ffffff\" stroke=\"#9aa39a\" stroke-width=\"1\" stroke-dasharray=\"4 3\"/>\n",
                polygon(h)
            ));
            out.push_str(&format!(
                "<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#6b736d\">{}, {}</text>\n",
                y + 4.0,
                h.q,
                h.r
            ));
        }
        let (x, y) = pixel(position.center());
        out.push_str(&format!(
            "<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"12\" font-weight=\"700\" fill=\"#4a524c\">{}</text>\n",
            y - 10.0,
            position_name(*i, *j)
        ));
    }

    for (position, id) in &placed {
        for (direction, content) in spaces(*id) {
            let h = hex_of(*position, direction);
            let (x, y) = pixel(h);
            out.push_str(&format!(
                "<polygon points=\"{}\" fill=\"{}\" stroke=\"#4a524c\" stroke-width=\"1\"/>\n",
                polygon(h),
                fill(content.terrain)
            ));
            out.push_str(&format!(
                "<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"11\" font-weight=\"700\" fill=\"#1d2420\">{}, {}</text>\n",
                y - 8.0,
                h.q,
                h.r
            ));
            out.push_str(&format!(
                "<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"9\" fill=\"#1d2420\">{}</text>\n",
                y + 4.0,
                terrain_name(content.terrain)
            ));
            if let Some(feature) = content.feature {
                out.push_str(&format!(
                    "<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"9\" font-style=\"italic\" fill=\"#1d2420\">{}</text>\n",
                    y + 15.0,
                    short_feature(feature)
                ));
            }
        }
        let (x, y) = pixel(position.center());
        out.push_str(&format!(
            "<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"8\" fill=\"#4a524c\">{}</text>\n",
            y - 20.0,
            tile_name(*id)
        ));
    }
    out.push_str("</svg>\n");
    out
}

fn regenerate() -> bool {
    std::env::var_os("ARCANUM_REGENERER_DOCS").is_some()
}

const HINT: &str = "page de référence périmée : régénérer avec \
ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-core --test reference_bac_a_sable";

#[test]
fn sandbox_reference_page_matches_setup() {
    let path = docs_dir().join("bac-a-sable.md");
    let page = std::fs::read_to_string(&path).expect("docs/reference/bac-a-sable.md existe");
    let start = page.find(BEGIN).expect("marqueur de début") + BEGIN.len();
    let end = page.find(END).expect("marqueur de fin");
    let expected = format!("\n\n{}\n", generated_markdown(&sandbox()));
    if regenerate() {
        let updated = format!("{}{expected}{}", &page[..start], &page[end..]);
        std::fs::write(&path, updated).expect("écriture de la page");
        return;
    }
    assert_eq!(&page[start..end], expected, "{HINT}");
}

#[test]
fn sandbox_reference_diagram_matches_setup() {
    let path = docs_dir().join("bac-a-sable.svg");
    let expected = generated_svg(&sandbox());
    if regenerate() {
        std::fs::write(&path, &expected).expect("écriture du schéma");
        return;
    }
    let diagram = std::fs::read_to_string(&path).expect("docs/reference/bac-a-sable.svg existe");
    assert_eq!(diagram, expected, "{HINT}");
}

/// Les deux tuiles de campagne de la mise en place sont toujours la 1 en `B` et la 2 en `A`
/// (MKUE p. 18, « Première reconnaissance ») : la page ne dépend pas de la graine.
#[test]
fn sandbox_setup_does_not_depend_on_seed() {
    let at_seed = |seed| {
        let config = GameConfig {
            seed,
            scenario: Scenario::sandbox(),
            hero: Hero::Arythea,
            dummy_hero: Default::default(),
        };
        let mut rng = config.rngs();
        let state = new_game(&config, &mut rng)
            .iter()
            .fold(GameState::default(), apply);
        let countryside: Vec<TileId> = state
            .tile_deck()
            .tiles()
            .iter()
            .copied()
            .filter(|id| id.kind() == TileKind::Countryside)
            .collect();
        (state.map().tiles().collect::<Vec<_>>(), countryside)
    };
    assert_eq!(at_seed(0), at_seed(42));
}
