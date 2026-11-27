//! Scénarios de la spec #68 (texte des cartes et des compétences) : textes complets présents et
//! rédigés selon la charte, repères à icône des points, balises de couleur de mana. Aucune règle du
//! livret : c'est de l'affichage.

use arcanum_core::{
    AdvancedAction, Artifact, AttackKind, BasicAction, BasicColor, Element, ManaColor, Skill,
    Spell, Tactic,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{Icon, ValuePiece, icon_pieces, plain_text, repository_assets_dir};

// --- Outils ---

fn catalog(language: &str) -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language(language.into()),
    )
    .expect("fichier de langue du dépôt")
}

/// Le groupe `card.<id>` de chaque carte qui se joue : 20 actions de base, 28 actions avancées,
/// 20 sorts, 16 artefacts.
fn deed_card_groups() -> Vec<String> {
    let name_keys = BasicAction::ALL
        .map(BasicAction::label_key)
        .into_iter()
        .chain(AdvancedAction::ALL.map(AdvancedAction::label_key))
        .chain(Spell::ALL.map(Spell::label_key))
        .chain(Artifact::ALL.map(Artifact::label_key));
    name_keys
        .map(|key| key.trim_end_matches(".name").to_string())
        .collect()
}

/// Les clés des textes complets des cartes (de base et renforcé).
fn full_text_keys() -> Vec<String> {
    deed_card_groups()
        .into_iter()
        .flat_map(|group| {
            [
                format!("{group}.basic_text"),
                format!("{group}.strong_text"),
            ]
        })
        .collect()
}

/// Les textes soumis à la charte (C5) : textes complets, compétences, tactiques.
fn written_keys() -> Vec<String> {
    full_text_keys()
        .into_iter()
        .chain(Skill::ALL.map(Skill::text_key))
        .chain(Tactic::ALL.map(|tactic| format!("tactic.{}.text", tactic.id())))
        .collect()
}

/// Les libellés de bouton écrits : `basic_label`, `strong_label` (ou un par option,
/// `basic_label_<n>`, issue #107), capacités d'unité.
fn button_label_keys(catalog: &Catalog) -> Vec<String> {
    catalog
        .keys()
        .into_iter()
        .filter(|key| {
            (key.starts_with("card.")
                && (key.ends_with(".basic_label")
                    || key.ends_with(".strong_label")
                    || key.contains(".basic_label_")
                    || key.contains(".strong_label_")))
                || (key.starts_with("unit.") && key.contains(".ability_"))
        })
        .collect()
}

fn exists(catalog: &Catalog, key: &str) -> bool {
    catalog.parameters(key).is_some()
}

// --- Textes complets ---

// Spec #68 C1 : chaque carte qui se joue a ses deux textes complets, en fr et en en.
#[test]
fn every_deed_card_has_full_texts_in_both_languages() {
    for language in ["fr", "en"] {
        let catalog = catalog(language);
        for key in full_text_keys() {
            assert!(exists(&catalog, &key), "{language} : {key} manquante");
            assert!(
                !catalog.text(&key).trim().is_empty(),
                "{language} : {key} vide"
            );
        }
    }
}

// Spec #68 C5 : des phrases complètes, sans formule ni abréviation.
#[test]
fn full_texts_follow_the_writing_rules() {
    let mut faults = Vec::new();
    for language in ["fr", "en"] {
        let catalog = catalog(language);
        for key in written_keys() {
            let text = catalog.text(&key);
            let plain = plain_text(&text, &catalog);
            let mut broken: Vec<&str> = [";", "=", "min.", "Idem", "etc."]
                .into_iter()
                .filter(|forbidden| plain.contains(forbidden))
                .collect();
            if !plain.trim_end().ends_with('.') {
                broken.push("point final");
            }
            if !broken.is_empty() {
                faults.push(format!("{language} {key} {broken:?} : {plain}"));
            }
        }
    }
    assert!(
        faults.is_empty(),
        "{} textes :\n{}",
        faults.len(),
        faults.join("\n")
    );
}

// --- Repères à icône ---

fn icon_of(piece: &ValuePiece) -> Option<Icon> {
    match piece {
        ValuePiece::Icon(value) => Some(value.icon),
        _ => None,
    }
}

// Spec #68 C6 : `[[move]]` dessine l'icône Mouvement, suivie du mot.
#[test]
fn move_marker_draws_the_move_icon() {
    let catalog = catalog("fr");
    let pieces = icon_pieces("[[move]]Mouvement 4.", &catalog);
    assert_eq!(pieces.len(), 2, "{pieces:?}");
    assert_eq!(icon_of(&pieces[0]), Some(Icon::Move));
    assert_eq!(pieces[1], ValuePiece::Text("Mouvement 4.".into()));
    assert_eq!(plain_text("[[move]]Mouvement 4.", &catalog), "Mouvement 4.");
}

// Spec #68 C6 : un repère par famille d'icônes de points.
#[test]
fn point_markers_draw_their_icons() {
    let catalog = catalog("fr");
    for (marker, icon) in [
        (
            "fire_attack",
            Icon::Attack {
                kind: AttackKind::Melee,
                element: Element::Fire,
            },
        ),
        (
            "ranged_ice_attack",
            Icon::Attack {
                kind: AttackKind::Ranged,
                element: Element::Ice,
            },
        ),
        (
            "siege_attack",
            Icon::Attack {
                kind: AttackKind::Siege,
                element: Element::Physical,
            },
        ),
        ("ice_block", Icon::Block(Element::Ice)),
        ("block", Icon::Block(Element::Physical)),
        ("armor", Icon::Armor),
        ("fame", Icon::Fame),
        ("influence", Icon::Influence),
    ] {
        let pieces = icon_pieces(&format!("[[{marker}]]Mot"), &catalog);
        assert_eq!(
            pieces.first().and_then(icon_of),
            Some(icon),
            "{marker} : {pieces:?}"
        );
    }
}

// --- Balises de couleur ---

fn colored(text: &str, color: ManaColor) -> ValuePiece {
    ValuePiece::Colored {
        text: text.into(),
        color,
    }
}

// Spec #68 C9 : une balise connue donne un morceau coloré ; inconnue ou non fermée, du texte.
#[test]
fn color_tags_split_into_colored_text() {
    let catalog = catalog("fr");
    assert_eq!(
        icon_pieces(
            "Gagnez un cristal <red>rouge</red> ou <gold>or</gold>.",
            &catalog
        ),
        [
            ValuePiece::Text("Gagnez un cristal ".into()),
            colored("rouge", ManaColor::Basic(BasicColor::Red)),
            ValuePiece::Text(" ou ".into()),
            colored("or", ManaColor::Gold),
            ValuePiece::Text(".".into()),
        ]
    );
    let pieces = icon_pieces("[[block]]Parade 4. <blue>Bleu</blue> :", &catalog);
    assert_eq!(icon_of(&pieces[0]), Some(Icon::Block(Element::Physical)));
    assert_eq!(
        pieces[2],
        colored("Bleu", ManaColor::Basic(BasicColor::Blue))
    );
    for raw in ["Un <pink>rose</pink>.", "Un <red>rouge sans fin."] {
        assert_eq!(
            icon_pieces(raw, &catalog),
            [ValuePiece::Text(raw.into())],
            "{raw}"
        );
    }
}

// Spec #68 C10 : le texte sans mise en forme garde le mot, pas la balise.
#[test]
fn plain_text_drops_color_tags() {
    let catalog = catalog("fr");
    assert_eq!(
        plain_text("Cristal <red>rouge</red>", &catalog),
        "Cristal rouge"
    );
    assert_eq!(
        plain_text("[[move]]Mouvement 7. <green>Vert</green> :", &catalog),
        "Mouvement 7. Vert :"
    );
}

/// Les noms de balise d'un texte, triés.
fn tags(text: &str) -> Vec<String> {
    let mut found: Vec<String> = text
        .split('<')
        .skip(1)
        .filter(|rest| !rest.starts_with('/'))
        .filter_map(|rest| rest.split_once('>').map(|(name, _)| name.to_string()))
        .collect();
    found.sort();
    found
}

/// Les mots de couleur de mana sans balise à côté d'un mot de mana : après lui en français
/// (« cristal rouge »), avant lui en anglais (« red crystal »).
fn untagged_colors(text: &str, language: &str) -> Vec<String> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '<' && c != '>' && c != '/')
                .to_lowercase()
        })
        .collect();
    let (triggers, colors): (&[&str], &[&str]) = if language == "fr" {
        (
            &[
                "cristal", "cristaux", "jeton", "jetons", "mana", "dé", "dés", "sort", "sorts",
            ],
            &[
                "rouge", "rouges", "bleu", "bleue", "bleus", "blanc", "blanche", "blancs", "vert",
                "verte", "verts", "or", "noir", "noire", "noirs",
            ],
        )
    } else {
        (
            &[
                "crystal", "crystals", "token", "tokens", "mana", "die", "dice", "spell", "spells",
            ],
            &["red", "blue", "white", "green", "gold", "black"],
        )
    };
    words
        .windows(2)
        .filter_map(|pair| {
            let (trigger, color) = if language == "fr" {
                (&pair[0], &pair[1])
            } else {
                (&pair[1], &pair[0])
            };
            (triggers.contains(&trigger.as_str()) && colors.contains(&color.as_str()))
                .then(|| pair.join(" "))
        })
        .collect()
}

// Spec #68 C12 : les mots de couleur de mana portent leur balise, les mêmes dans les deux langues.
#[test]
fn mana_colors_are_tagged_in_texts() {
    let (fr, en) = (catalog("fr"), catalog("en"));
    let keys: Vec<String> = written_keys()
        .into_iter()
        .chain(button_label_keys(&fr))
        .collect();
    let mut faults = Vec::new();
    for key in &keys {
        for (language, catalog) in [("fr", &fr), ("en", &en)] {
            let untagged = untagged_colors(&catalog.text(key), language);
            if !untagged.is_empty() {
                faults.push(format!("{language} {key} : {untagged:?}"));
            }
        }
        if tags(&fr.text(key)) != tags(&en.text(key)) {
            faults.push(format!("{key} : balises différentes en fr et en en"));
        }
    }
    assert!(
        faults.is_empty(),
        "{} fautes :\n{}",
        faults.len(),
        faults.join("\n")
    );
    // Au moins un texte balisé par famille de couleurs de base.
    let tagged = keys
        .iter()
        .filter(|key| !tags(&fr.text(key)).is_empty())
        .count();
    assert!(tagged >= 30, "{tagged} textes balisés");
}

/// Les clés qui reçoivent une couleur de mana calculée (`{color}`, `{token}`, `{crystal}`…) :
/// invites de paiement, infobulles des dés et des jetons, lignes de la zone de jeu (issue #106).
fn color_parameter_keys(catalog: &Catalog) -> Vec<String> {
    catalog
        .keys()
        .into_iter()
        .filter(|key| {
            catalog.parameters(key).is_some_and(|parameters| {
                ["color", "token", "crystal"]
                    .iter()
                    .any(|name| parameters.contains(*name))
            })
        })
        .collect()
}

// Issue #106 (C12 étendu) : les textes à couleur calculée balisent aussi les mots de couleur
// qu'ils écrivent eux-mêmes (« Dé <gold>or</gold> (vaut {color}) »), les mêmes dans les deux
// langues.
#[test]
fn mana_colors_are_tagged_in_color_parameter_texts() {
    let (fr, en) = (catalog("fr"), catalog("en"));
    let keys = color_parameter_keys(&fr);
    assert!(keys.len() >= 40, "{} clés", keys.len());
    let mut faults = Vec::new();
    for key in &keys {
        for (language, catalog) in [("fr", &fr), ("en", &en)] {
            let untagged = untagged_colors(&catalog.text(key), language);
            if !untagged.is_empty() {
                faults.push(format!("{language} {key} : {untagged:?}"));
            }
        }
        if tags(&fr.text(key)) != tags(&en.text(key)) {
            faults.push(format!("{key} : balises différentes en fr et en en"));
        }
    }
    assert!(
        faults.is_empty(),
        "{} fautes :\n{}",
        faults.len(),
        faults.join("\n")
    );
}
