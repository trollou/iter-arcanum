//! Spec #52, C12 : l'empreinte du format des sauvegardes, la forme JSON de `SaveGame` et de tous
//! les types qu'il porte, générée depuis les types courants par `serde-reflection` et comparée
//! au fichier commité `tests/format/sauvegarde.yaml`. Son diff rend visible, dans la PR, tout
//! changement de forme ; `diff tests/format/v<X.Y.Z>.yaml tests/format/sauvegarde.yaml` donne ce
//! qu'une migration doit transformer depuis la version X.Y.Z.
//!
//! Régénérer : `ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-persistance --test format`.

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use arcanum_core::*;
use arcanum_persistance::{SaveGame, SaveSummary, SavedEvent};
use serde_reflection::{
    ContainerFormat, Format, Named, Registry, Samples, Tracer, TracerConfig, VariantFormat,
};

/// Une sauvegarde minimale, sérialisée pour tracer ce que la désérialisation ne sait pas
/// inventer : les positions des générateurs, écrites en texte et relues par `parse`.
fn sample() -> SaveGame {
    SaveGame {
        version: EVENT_FORMAT_VERSION,
        summary: SaveSummary {
            saved_at: "2026-01-01T00:00:00+00:00".to_string(),
            scenario: ScenarioId::Sandbox,
            hero: Hero::Arythea,
            round: 1,
            round_limit: 1,
            time_of_day: TimeOfDay::Day,
            turn: 1,
        },
        rng_positions: RngPositions::from([(RngUsage::Dummy, 1)]),
        events: vec![SavedEvent {
            decision: 0,
            event: Event::TurnStarted {
                player: PlayerId::SOLO,
                turn: 1,
            },
        }],
    }
}

/// Trace chaque enum de la liste jusqu'à en connaître toutes les variantes. Une enum imbriquée
/// ne se trace complètement que nommée ici : si l'empreinte en signale une, l'ajouter.
macro_rules! trace_enums {
    ($tracer:ident, $samples:ident; $($ty:ty),* $(,)?) => {
        $( $tracer.trace_type::<$ty>(&$samples).expect(stringify!($ty)); )*
    };
}

fn registry() -> Registry {
    let mut tracer = Tracer::new(TracerConfig::default().is_human_readable(true));
    let mut samples = Samples::new();
    tracer
        .trace_value(&mut samples, &sample())
        .expect("traçage de la sauvegarde");
    trace_enums!(tracer, samples;
        AdvancedAction, AdvancedActionSet, Artifact, AttackKind, BasicAction, BasicColor,
        CardDestination, CardPlay, CityRule, Columns, CombatPhase, CommonCard, CommonDeck,
        CountrysideOrder, DeedCard, DestroyedFrom, DummyHeroChoice, DummyRule, Effect, Element,
        Enemy, EnemyColor, FinalScoring, GameEndReason, Hero, ManaColor, ManaPayment, MapShape,
        Offer, PolarSource, RuinToken, ScenarioEnd, ScenarioId, SidewaysEffect, Site, Skill, Spell,
        SpellSet, Tactic, TacticOrder, TacticRemoval, TacticRule, TileId, TimeOfDay, Unit,
        WoundPile, Event,
    );
    tracer.registry().unwrap_or_else(|error| {
        panic!("empreinte incomplète ({error}) : ajouter les enums citées à trace_enums!")
    })
}

/// L'empreinte en texte YAML, un type par entrée, une ligne par champ ou variante : un
/// changement de forme se lit ligne à ligne dans le diff.
fn fingerprint() -> String {
    let mut text = String::from(
        "# Empreinte du format des sauvegardes (spec #52), générée par le test `format`.\n",
    );
    for (name, container) in registry() {
        match container {
            ContainerFormat::UnitStruct => writeln!(text, "{name}: unit"),
            ContainerFormat::NewTypeStruct(format) => {
                writeln!(text, "{name}: newtype {}", shape(&format))
            }
            ContainerFormat::TupleStruct(formats) => {
                writeln!(text, "{name}: tuple {}", tuple(&formats))
            }
            ContainerFormat::Struct(fields) => {
                writeln!(text, "{name}:").and_then(|()| write_fields(&mut text, &fields, 2))
            }
            ContainerFormat::Enum(variants) => {
                writeln!(text, "{name}:").and_then(|()| {
                    variants
                        .values()
                        .try_for_each(|variant| write_variant(&mut text, variant))
                })
            }
        }
        .expect("écriture en mémoire");
    }
    text
}

fn write_fields(text: &mut String, fields: &[Named<Format>], indent: usize) -> std::fmt::Result {
    fields.iter().try_for_each(|field| {
        writeln!(
            text,
            "{:indent$}{}: {}",
            "",
            field.name,
            shape(&field.value)
        )
    })
}

fn write_variant(text: &mut String, variant: &Named<VariantFormat>) -> std::fmt::Result {
    let name = &variant.name;
    match &variant.value {
        VariantFormat::Unit => writeln!(text, "  - {name}"),
        VariantFormat::NewType(format) => writeln!(text, "  - {name}: {}", shape(format)),
        VariantFormat::Tuple(formats) => writeln!(text, "  - {name}: {}", tuple(formats)),
        VariantFormat::Struct(fields) => {
            writeln!(text, "  - {name}:")?;
            write_fields(text, fields, 6)
        }
        VariantFormat::Variable(_) => writeln!(text, "  - {name}: inconnu"),
    }
}

fn tuple(formats: &[Format]) -> String {
    let inner: Vec<String> = formats.iter().map(shape).collect();
    format!("({})", inner.join(", "))
}

fn shape(format: &Format) -> String {
    match format {
        Format::TypeName(name) => name.clone(),
        Format::Option(inner) => format!("Option<{}>", shape(inner)),
        Format::Seq(inner) => format!("Vec<{}>", shape(inner)),
        Format::Map { key, value } => format!("Map<{}, {}>", shape(key), shape(value)),
        Format::Tuple(formats) => tuple(formats),
        Format::TupleArray { content, size } => format!("Array<{}, {size}>", shape(content)),
        Format::Unit => "()".to_string(),
        Format::Variable(_) => "inconnu".to_string(),
        primitive => format!("{primitive:?}").to_lowercase(),
    }
}

fn format_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/format")
}

const HINT: &str = "la forme d'une sauvegarde a changé. Changement compatible (variante ajoutée, \
champ ajouté avec #[serde(default)] qui redonne l'ancien sens) : régénérer l'empreinte avec \
ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-persistance --test format. Incompatible : passer \
EVENT_FORMAT_VERSION à PREVIOUS_RELEASE_FORMAT + 1 si ce n'est pas déjà fait dans ce cycle, et \
écrire ou compléter la migration (docs/architecture.md, « Ajouter un événement »), puis régénérer.";

// Spec #52, C12.
#[test]
fn save_format_matches_fingerprint() {
    let expected = fingerprint();
    let path = format_dir().join("sauvegarde.yaml");
    if std::env::var_os("ARCANUM_REGENERER_DOCS").is_some() {
        fs::write(&path, &expected).expect("écriture de l'empreinte");
        return;
    }
    let found = fs::read_to_string(&path).unwrap_or_default();
    assert!(found == expected, "{} : {HINT}", path.display());
}
