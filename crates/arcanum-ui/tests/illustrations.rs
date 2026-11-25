//! Spec 072 : illustrations générées des cartes et des unités, côté écran.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use arcanum_core::{
    AdvancedAction, Artifact, BasicAction, CardId, CommonCard, DeedCard, Offer, OwnedUnit,
    PlayerView, Spell, Unit, UnitCard, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    GameScreen, IllustrationFamily, IllustrationIndex, ScreenMode, ScreenPhase, card_illustration,
    game_screen, known_illustrations, offers_window, repository_assets_dir, unit_illustration,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// Rage et une Blessure en main ; le reste de la vue se règle champ par champ.
fn base_view() -> PlayerView {
    view(&state_with(&[card(BasicAction::Rage, 1), wound(1)]), SOLO)
}

fn screen_of(view: &PlayerView, catalog: &Catalog) -> GameScreen {
    game_screen(
        view,
        ScreenPhase::Playing { legal_actions: &[] },
        &ScreenMode::Normal,
        catalog,
    )
}

fn deed(card: DeedCard) -> CardId {
    CardId::new(card, 1)
}

/// Le fichier des descriptions d'une famille, dans `scripts/illustrations/`.
fn descriptions_file(family: IllustrationFamily) -> std::path::PathBuf {
    let name = match family {
        IllustrationFamily::Unit => "units.json",
        IllustrationFamily::Card => "cards.json",
        IllustrationFamily::Hero => "heroes.json",
    };
    repository_assets_dir()
        .join("..")
        .join("scripts")
        .join("illustrations")
        .join(name)
}

/// Les PNG d'un dossier de famille, par nom sans extension.
fn png_stems(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut stems: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .filter_map(|path| Some(path.file_stem()?.to_str()?.to_owned()))
        .collect();
    stems.sort();
    stems
}

// Spec 072, C1 : l'identifiant d'un élément est celui de sa clé de libellé.
#[test]
fn illustration_path_follows_label_key() {
    let images_dir = Path::new("images");
    assert_eq!(
        unit_illustration(Unit::Peasants).path(images_dir),
        images_dir.join("unit-art").join("peasants.png")
    );
    assert_eq!(
        card_illustration(DeedCard::Basic(BasicAction::BattleVersatility)).path(images_dir),
        images_dir.join("card-art").join("battle_versatility.png")
    );
    assert_eq!(
        card_illustration(DeedCard::Wound).path(images_dir),
        images_dir.join("card-art").join("wound.png")
    );
}

// Spec 072, C1 : deux éléments n'ont jamais le même fichier ; 20 unités, et 20 actions de base,
// la Blessure, 28 actions avancées, 20 sorts et 16 artefacts.
#[test]
fn known_illustrations_cover_every_unit_and_card() {
    let units = known_illustrations(IllustrationFamily::Unit);
    let cards = known_illustrations(IllustrationFamily::Card);
    assert_eq!(units.len(), 20);
    assert_eq!(cards.len(), 20 + 1 + 28 + 20 + 16);
    let distinct: BTreeSet<_> = units.iter().chain(&cards).collect();
    assert_eq!(distinct.len(), units.len() + cards.len());
    assert!(units.contains(&unit_illustration(Unit::Peasants)));
    assert!(cards.contains(&card_illustration(DeedCard::Wound)));
}

// Spec 072, C2, révisée par la spec #9, C5 : chaque PNG porte l'identifiant d'un élément de sa
// famille, se décode, a une palette de 256 couleurs au plus (issue privée 167) et mesure 404 × 168,
// le double de la bande ; la Blessure, affichée en grand, reste en 256 × 256.
#[test]
fn every_illustration_is_known_and_valid() {
    let images_dir = repository_assets_dir().join("images");
    for family in IllustrationFamily::ALL {
        let known: BTreeSet<&str> = known_illustrations(family)
            .into_iter()
            .map(|illustration| illustration.id)
            .collect();
        let dir = images_dir.join(family.dir());
        for stem in png_stems(&dir) {
            assert!(
                known.contains(stem.as_str()),
                "{}/{stem}.png : aucun élément de ce nom",
                family.dir()
            );
            let path = dir.join(format!("{stem}.png"));
            let image = image::open(&path).unwrap_or_else(|error| panic!("{path:?} : {error}"));
            // Spec 078, C9b : un portrait de héros, vu en grand, garde 512 × 512 et ses couleurs.
            if family == IllustrationFamily::Hero {
                assert_eq!((image.width(), image.height()), (512, 512), "{path:?}");
                continue;
            }
            let expected = if family == IllustrationFamily::Card && stem == "wound" {
                (256, 256)
            } else {
                (404, 168)
            };
            assert_eq!(
                (image.width(), image.height()),
                expected,
                "{path:?} n'a pas le format de sa famille"
            );
            // Octet 25 du PNG : type de couleur de l'en-tête IHDR, 3 pour une image à palette.
            let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{path:?} : {error}"));
            assert_eq!(
                bytes.get(25),
                Some(&3),
                "{path:?} n'est pas un PNG à palette"
            );
        }
    }
}

// Spec 072, C3 : chaque PNG a son entrée avec description, modèle et date ; chaque entrée porte
// un identifiant connu et une description.
#[test]
fn every_illustration_has_its_description() {
    let images_dir = repository_assets_dir().join("images");
    for family in IllustrationFamily::ALL {
        let file = descriptions_file(family);
        let text = fs::read_to_string(&file).unwrap_or_else(|_| panic!("{file:?} manque"));
        let entries: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{file:?} : {error}"));
        let known: BTreeSet<&str> = known_illustrations(family)
            .into_iter()
            .map(|illustration| illustration.id)
            .collect();
        for (id, entry) in &entries {
            assert!(known.contains(id.as_str()), "{file:?} : {id} inconnu");
            let description = entry["description"].as_str().unwrap_or_default();
            assert!(!description.is_empty(), "{file:?} : {id} sans description");
        }
        for stem in png_stems(&images_dir.join(family.dir())) {
            let entry = entries
                .get(&stem)
                .unwrap_or_else(|| panic!("{file:?} : pas d'entrée pour {stem}"));
            for field in ["model", "date"] {
                let value = entry[field].as_str().unwrap_or_default();
                assert!(!value.is_empty(), "{file:?} : {stem} sans {field}");
            }
        }
    }
}

// Spec #9, C6 : chaque unité et chaque carte du contenu a son PNG et son entrée dans le fichier des
// descriptions de sa famille ; le lot est complet à la fusion.
#[test]
fn every_unit_and_card_has_its_illustration() {
    let images_dir = repository_assets_dir().join("images");
    for family in [IllustrationFamily::Unit, IllustrationFamily::Card] {
        let file = descriptions_file(family);
        let text = fs::read_to_string(&file).unwrap_or_else(|_| panic!("{file:?} manque"));
        let entries: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{file:?} : {error}"));
        let present: BTreeSet<String> = png_stems(&images_dir.join(family.dir()))
            .into_iter()
            .collect();
        let missing: Vec<&str> = known_illustrations(family)
            .into_iter()
            .map(|illustration| illustration.id)
            .filter(|id| !present.contains(*id) || !entries.contains_key(*id))
            .collect();
        assert!(
            missing.is_empty(),
            "{} : sans image ou sans entrée : {missing:?}",
            family.dir()
        );
    }
}

// Spec 072, C4 : l'index sait quelles images sont présentes.
#[test]
fn illustrations_index_lists_present_files() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let cards = dir.path().join("card-art");
    fs::create_dir_all(&cards).expect("card-art");
    fs::write(cards.join("rage.png"), b"png").expect("rage.png");

    let index = IllustrationIndex::load(dir.path());

    assert!(index.contains(card_illustration(DeedCard::Basic(BasicAction::Rage))));
    assert!(!index.contains(card_illustration(DeedCard::Basic(BasicAction::March))));
    assert!(!index.contains(unit_illustration(Unit::Peasants)));
}

// Spec 072, L2 : sans dossier des images, l'index est vide et le jeu se joue sans image.
#[test]
fn illustrations_index_without_directory_is_empty() {
    let dir = tempfile::tempdir().expect("dossier temporaire");

    let index = IllustrationIndex::load(&dir.path().join("absent"));

    assert_eq!(index, IllustrationIndex::default());
    assert!(!index.contains(unit_illustration(Unit::Peasants)));
}

// Spec 072, L3 : un fichier qui ne porte le nom d'aucun élément de sa famille est ignoré en jeu.
// (Exemple de la spec, `fireball`, remplacé : Boule de feu est un sort du contenu.)
#[test]
fn illustrations_index_ignores_unknown_files() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let cards = dir.path().join("card-art");
    let units = dir.path().join("unit-art");
    fs::create_dir_all(&cards).expect("card-art");
    fs::create_dir_all(&units).expect("unit-art");
    fs::write(cards.join("not_a_card.png"), b"png").expect("png");
    fs::write(units.join("rage.png"), b"png").expect("png");
    fs::write(units.join("peasants.txt"), b"txt").expect("txt");

    let index = IllustrationIndex::load(dir.path());

    assert_eq!(index, IllustrationIndex::default());
}

// --- Écran : chaque élément porte son illustration ---

// Spec 072, C5 : une carte de la main porte l'illustration de sa carte, Blessure comprise.
#[test]
fn hand_card_has_its_illustration() {
    let catalog = french();
    let screen = screen_of(&base_view(), &catalog);
    let illustrations: Vec<_> = screen
        .hand
        .iter()
        .take(2)
        .map(|card| card.illustration)
        .collect();
    assert_eq!(
        illustrations,
        [
            card_illustration(DeedCard::Basic(BasicAction::Rage)),
            card_illustration(DeedCard::Wound),
        ]
    );
}

// Spec 077, C1 : la main de la partie préparée `main-blessures` ; l'image d'une Blessure occupe
// la carte (elle n'a ni effet ni texte renforcé, spec 031), pas celle de Marche.
#[test]
fn wound_art_fills_card() {
    let catalog = french();
    let hand = [
        wound(1),
        wound(2),
        card(BasicAction::March, 1),
        card(BasicAction::Promise, 1),
        card(BasicAction::Rage, 1),
    ];
    let screen = screen_of(&view(&state_with(&hand), SOLO), &catalog);
    let fills: Vec<_> = screen
        .hand
        .iter()
        .map(|held| (held.card, held.art_fills_card))
        .collect();
    assert_eq!(
        fills,
        [
            (wound(1), true),
            (wound(2), true),
            (hand[2], false),
            (hand[3], false),
            (hand[4], false),
        ]
    );
}

// Spec 072, C6 : les cartes de la fenêtre Offres (action avancée, sort, action avancée d'un
// monastère, unité) et des fenêtres de choix de l'artefact et du sort portent leur illustration.
#[test]
fn offer_card_has_its_illustration() {
    let catalog = french();
    let mut view = base_view();
    let advanced = DeedCard::Advanced(AdvancedAction::ALL[0]);
    let monastery = DeedCard::Advanced(AdvancedAction::ALL[1]);
    let fireball = DeedCard::Spell(Spell::Fireball);
    let peasants = UnitCard::new(Unit::Peasants, 1);
    view.offers.insert(
        Offer::AdvancedActions,
        vec![CommonCard::Deed(deed(advanced))],
    );
    view.offers
        .insert(Offer::Spells, vec![CommonCard::Deed(deed(fireball))]);
    view.offers.insert(
        Offer::Units,
        vec![
            CommonCard::Unit(peasants),
            CommonCard::Deed(deed(monastery)),
        ],
    );
    let window = offers_window(&view, &[], &catalog);
    let illustrations: Vec<Vec<_>> = window
        .columns
        .iter()
        .map(|column| column.cards.iter().map(|card| card.illustration).collect())
        .collect();
    assert_eq!(
        illustrations,
        [
            vec![card_illustration(advanced)],
            vec![card_illustration(fireball)],
            vec![
                unit_illustration(Unit::Peasants),
                card_illustration(monastery)
            ],
        ]
    );

    let artifact = DeedCard::Artifact(Artifact::ALL[0]);
    view.drawn_artifacts = vec![deed(artifact)];
    view.spell_choices = 1;
    let screen = screen_of(&view, &catalog);
    let artifacts = screen.artifact_choice.expect("choix de l'artefact");
    assert_eq!(artifacts.cards[0].illustration, card_illustration(artifact));
    let spells = screen.spell_choice.expect("choix du sort");
    assert_eq!(spells.cards[0].illustration, card_illustration(fireball));
}

// Spec 072, C7 : une ligne d'unité porte l'illustration de son unité.
#[test]
fn unit_line_has_its_illustration() {
    let catalog = french();
    let mut view = base_view();
    view.units = vec![
        OwnedUnit::recruited(UnitCard::new(Unit::Peasants, 1)),
        OwnedUnit::recruited(UnitCard::new(Unit::FireGolems, 1)),
    ];
    let screen = screen_of(&view, &catalog);
    let illustrations: Vec<_> = screen
        .units
        .units
        .iter()
        .map(|line| line.illustration)
        .collect();
    assert_eq!(
        illustrations,
        [
            unit_illustration(Unit::Peasants),
            unit_illustration(Unit::FireGolems)
        ]
    );
}
