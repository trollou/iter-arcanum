//! Spec 004, C5 : une icône impossible à préparer est écrite dans les logs.
//! Fichier à part : il installe un logger global de capture.

mod support;

use arcanum_ui::{WINDOW_ICON_SIZE, render_icon, viewport};
use log::Level;
use support::logs::logged;

// Spec 004, C5.
#[test]
fn icon_failure_is_logged() {
    logged(Level::Error);
    let _ = viewport(
        "Iter Arcanum",
        render_icon(b"pas un svg", WINDOW_ICON_SIZE),
        false,
    );
    let logged = logged(Level::Error);
    assert_eq!(logged.len(), 1, "{logged:?}");
    assert!(logged[0].contains("icône"), "{logged:?}");
}

// Spec 004, C5 : rien n'est écrit quand l'icône est prête.
#[test]
fn icon_success_logs_no_error() {
    logged(Level::Error);
    let _ = viewport("Iter Arcanum", arcanum_ui::window_icon(), false);
    assert_eq!(logged(Level::Error), Vec::<String>::new());
}
