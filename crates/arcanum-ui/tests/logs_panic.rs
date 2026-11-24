//! Spec 003 C15, dans son propre binaire de test : le crochet de panic est global au processus.

mod support;

use log::Level;
use support::logs::captured;

#[test]
fn panic_is_written_to_the_log() {
    // Spec 003 C15
    captured();
    arcanum_ui::install_panic_hook();

    let result = std::panic::catch_unwind(|| panic!("boum de test"));

    // Le comportement habituel est conservé : le panic remonte toujours.
    assert!(result.is_err());
    let lines = captured();
    assert!(
        lines.iter().any(|(level, message)| *level == Level::Error
            && message.contains("boum de test")
            && message.contains("logs_panic.rs")),
        "{lines:?}"
    );
}
