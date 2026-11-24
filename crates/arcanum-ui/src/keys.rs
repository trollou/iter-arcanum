//! Touches du clavier (spec 071, C25) : une seule table, lue par le clavier de `app/mod.rs` et
//! par les infobulles des boutons, pour que le texte affiché dise la touche qui agit.

use arcanum_core::Action;
use arcanum_i18n::Catalog;
use eframe::egui::Key;

use crate::Shortcut;

/// Ce qu'il faut tenir avec la touche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyModifier {
    /// Ctrl (Cmd sous macOS), la touche consommée pour qu'egui n'en fasse rien d'autre.
    Ctrl,
    /// Sans modificateur : O sans Ctrl n'est pas Ctrl+O.
    None,
    /// Avec ou sans modificateur (Échap, Fin, Entrée).
    Any,
}

/// Une touche et le raccourci qu'elle déclenche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: Key,
    pub modifier: KeyModifier,
    pub shortcut: Shortcut,
}

const fn bind(key: Key, modifier: KeyModifier, shortcut: Shortcut) -> KeyBinding {
    KeyBinding {
        key,
        modifier,
        shortcut,
    }
}

/// Les touches du jeu (`docs/raccourcis.md`).
pub const KEY_BINDINGS: [KeyBinding; 27] = [
    // Spec 024 et 065 : Ctrl+Z, Ctrl+S, Ctrl+O, Ctrl+N.
    bind(Key::Z, KeyModifier::Ctrl, Shortcut::Undo),
    bind(Key::S, KeyModifier::Ctrl, Shortcut::Save),
    bind(Key::O, KeyModifier::Ctrl, Shortcut::OpenLoad),
    bind(Key::N, KeyModifier::Ctrl, Shortcut::NewGame),
    // Spec 074 : Ctrl+Q quitte, F11 passe en plein écran ou en sort.
    bind(Key::Q, KeyModifier::Ctrl, Shortcut::Quit),
    bind(Key::F11, KeyModifier::Any, Shortcut::Fullscreen),
    // Spec 080 : Ctrl+, ouvre ou ferme la fenêtre des préférences.
    bind(Key::Comma, KeyModifier::Ctrl, Shortcut::Preferences),
    // Specs 013 et 016 : Échap quitte le mode, Fin ouvre la fin du tour, Entrée la confirme.
    bind(Key::Escape, KeyModifier::Any, Shortcut::Cancel),
    bind(Key::End, KeyModifier::Any, Shortcut::EndTurn),
    bind(Key::Enter, KeyModifier::Any, Shortcut::Confirm),
    // Spec 050 : O, Offres ; spec 040 : I, interagir ; spec 044 : E, entrer dans un site.
    bind(Key::O, KeyModifier::None, Shortcut::Offers),
    bind(Key::I, KeyModifier::None, Shortcut::Interact),
    bind(Key::E, KeyModifier::None, Shortcut::EnterSite),
    // Spec 073 : D, la fenêtre de la défausse.
    bind(Key::D, KeyModifier::None, Shortcut::Discard),
    // Specs 050, 044 et 045 : 1 à 9, les choix numérotés.
    bind(Key::Num1, KeyModifier::None, Shortcut::Take(1)),
    bind(Key::Num2, KeyModifier::None, Shortcut::Take(2)),
    bind(Key::Num3, KeyModifier::None, Shortcut::Take(3)),
    bind(Key::Num4, KeyModifier::None, Shortcut::Take(4)),
    bind(Key::Num5, KeyModifier::None, Shortcut::Take(5)),
    bind(Key::Num6, KeyModifier::None, Shortcut::Take(6)),
    bind(Key::Num7, KeyModifier::None, Shortcut::Take(7)),
    bind(Key::Num8, KeyModifier::None, Shortcut::Take(8)),
    bind(Key::Num9, KeyModifier::None, Shortcut::Take(9)),
    // Spec 078 : ↑ ↓ cochent dans la liste de l'écran de nouvelle partie, ← → changent d'étape.
    bind(Key::ArrowUp, KeyModifier::None, Shortcut::PreviousChoice),
    bind(Key::ArrowDown, KeyModifier::None, Shortcut::NextChoice),
    bind(Key::ArrowLeft, KeyModifier::None, Shortcut::PreviousStep),
    bind(Key::ArrowRight, KeyModifier::None, Shortcut::NextStep),
];

/// Le libellé d'une touche de la table : « Ctrl+Z », « Entrée », « O », « 2 ».
pub fn key_label(binding: &KeyBinding, catalog: &Catalog) -> String {
    // Les lettres et les chiffres ne se traduisent pas.
    let key = match binding.key {
        Key::Escape => catalog.text("shortcut.key.escape"),
        Key::End => catalog.text("shortcut.key.end"),
        Key::Enter => catalog.text("shortcut.key.enter"),
        // egui la nomme « Comma » : on écrit la touche elle-même (spec 080).
        Key::Comma => ",".to_owned(),
        key => key.name().to_owned(),
    };
    match binding.modifier {
        KeyModifier::Ctrl => catalog.format("shortcut.key.ctrl", &[("key", &key)]),
        KeyModifier::None | KeyModifier::Any => key,
    }
}

/// Le libellé de la touche d'un raccourci, lu dans la table.
pub fn shortcut_label(shortcut: Shortcut, catalog: &Catalog) -> Option<String> {
    KEY_BINDINGS
        .iter()
        .find(|binding| binding.shortcut == shortcut)
        .map(|binding| key_label(binding, catalog))
}

/// L'infobulle d'un bouton : sa description s'il en a une, puis « Raccourci : Ctrl+Z » s'il a une
/// touche (spec 071, C18, L2, L3).
pub fn hover_text(
    tooltip: Option<&str>,
    shortcut: Option<&str>,
    catalog: &Catalog,
) -> Option<String> {
    let shortcut = shortcut.map(|key| catalog.format("shortcut.hint", &[("key", key)]));
    match (tooltip, shortcut) {
        (Some(tooltip), Some(shortcut)) => Some(format!("{tooltip}\n{shortcut}")),
        (Some(tooltip), None) => Some(tooltip.to_owned()),
        (None, shortcut) => shortcut,
    }
}

/// La touche du `place`-ième (à partir de 1) choix numéroté : « 1 » à « 9 », rien au-delà.
pub(crate) fn take_label(place: usize, catalog: &Catalog) -> Option<String> {
    u8::try_from(place)
        .ok()
        .and_then(|place| shortcut_label(Shortcut::Take(place), catalog))
}

/// La touche de l'action légale d'index `index`, `place`-ième de celles qui vérifient `wanted` :
/// celle que `GameSession` joue pour cette touche (`take_nth`).
pub(crate) fn take_label_of(
    legal_actions: &[Action],
    index: usize,
    wanted: fn(&Action) -> bool,
    catalog: &Catalog,
) -> Option<String> {
    let place = legal_actions
        .iter()
        .take(index)
        .filter(|action| wanted(action))
        .count();
    take_label(place + 1, catalog)
}
