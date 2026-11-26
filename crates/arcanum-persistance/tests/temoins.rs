//! Spec #52, C10 et C11 : les sauvegardes témoins des versions publiées gardées
//! (`tests/temoins/v<X.Y.Z>/`, jamais régénérées) se chargent avec la vraie chaîne de migrations
//! et se rejouent, et la chaîne s'accorde avec les versions gardées.

use std::fs;
use std::path::{Path, PathBuf};

use arcanum_core::{EVENT_FORMAT_VERSION, PlayerId, legal_actions};
use arcanum_persistance::migration::{
    MIGRATIONS, OLDEST_SUPPORTED_FORMAT, PREVIOUS_RELEASE_FORMAT,
};
use arcanum_persistance::{SaveGame, load};

fn tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// « v0.1.0 » → (0, 1, 0), pour ranger les versions.
fn version_of(name: &str) -> Option<(u32, u32, u32)> {
    let mut parts = name.strip_prefix('v')?.split('.').map(str::parse);
    let version = (
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    );
    parts.next().is_none().then_some(version)
}

/// Les noms (`v<X.Y.Z>`) des entrées de `dir` qui finissent par `suffix`, rangés par version.
fn versions(dir: &Path, suffix: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("dossier lisible")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.strip_suffix(suffix).map(String::from))
        .filter(|name| version_of(name).is_some())
        .collect();
    names.sort_by_key(|name| version_of(name));
    names
}

fn witnesses(version: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(tests_dir().join("temoins").join(version))
        .expect("dossier des témoins")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    paths.sort();
    paths
}

// Spec #52, C10.
#[test]
fn every_witness_loads_and_resumes() {
    let mut count = 0;
    for version in versions(&tests_dir().join("temoins"), "") {
        for path in witnesses(&version) {
            let name = path.display();
            let game = load(&path).unwrap_or_else(|error| panic!("{name} : {error}"));
            let summary = game.summary.clone();
            let runner = game
                .resume(Vec::new())
                .unwrap_or_else(|error| panic!("{name} : reprise impossible ({error})"));
            let resumed = SaveGame::from_runner(&runner, &summary.saved_at)
                .unwrap_or_else(|error| panic!("{name} : {error}"));
            assert_eq!(resumed.summary, summary, "{name} : résumé rejoué");
            let state = runner.state();
            assert!(
                state.game_end().is_some() || !legal_actions(state, PlayerId::SOLO).is_empty(),
                "{name} : partie reprise bloquée"
            );
            count += 1;
        }
    }
    assert!(count > 0, "aucun témoin");
}

// Spec #52, C11. Les constantes vérifiées changent avec le code : c'est ce que le test surveille.
#[test]
#[allow(clippy::assertions_on_constants)]
fn migration_chain_matches_supported_versions() {
    assert!(
        EVENT_FORMAT_VERSION == PREVIOUS_RELEASE_FORMAT
            || EVENT_FORMAT_VERSION == PREVIOUS_RELEASE_FORMAT + 1,
        "EVENT_FORMAT_VERSION change au plus une fois par version publiée"
    );
    assert_eq!(
        MIGRATIONS.len(),
        usize::from(EVENT_FORMAT_VERSION - OLDEST_SUPPORTED_FORMAT),
        "une migration par format entre le plus ancien gardé et le courant"
    );
    assert!(MIGRATIONS.len() <= 3, "trois migrations au plus");

    let witnessed = versions(&tests_dir().join("temoins"), "");
    let fingerprinted = versions(&tests_dir().join("format"), ".yaml");
    assert!(
        !witnessed.is_empty() && witnessed.len() <= 3,
        "témoins : {witnessed:?}"
    );
    assert_eq!(
        witnessed, fingerprinted,
        "un dossier de témoins par empreinte figée"
    );

    for path in witnesses(&witnessed[0]) {
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("lu")).expect("JSON");
        assert_eq!(
            value["version"],
            OLDEST_SUPPORTED_FORMAT,
            "{} : format du plus ancien témoin",
            path.display()
        );
    }
}
