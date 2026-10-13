//! Development harnesses must not seize a shared workstation by default.
pub fn require_desktop_input() {
    let allowed = std::env::var("CI").as_deref() == Ok("true")
        || std::env::args_os().any(|arg| arg == "--allow-desktop-input");
    if !allowed {
        eprintln!(
            "SKIPPED: GUI/input acceptance requires --allow-desktop-input or a dedicated CI session."
        );
        std::process::exit(2);
    }
}
