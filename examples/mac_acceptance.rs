//! Dedicated-desktop native readiness/import/termination checks, isolated files.
mod support;

#[cfg(target_os = "macos")]
fn main() {
    support::require_desktop_input();
    let root = tempfile::tempdir().expect("fixture directory");
    let paths = pp_switch::client::Paths::from_home(root.path().join("home"));
    let store = pp_switch::storage::Store {
        root: root.path().join("store"),
    };
    let args: Vec<_> = std::env::args().collect();
    let scenario = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--scenario="))
        .unwrap_or("empty");
    assert!(
        [
            "empty",
            "editor",
            "import",
            "quit-main",
            "quit-settings",
            "quit-editor",
            "quit-dock-main",
            "quit-dock-settings",
            "quit-dock-editor"
        ]
        .contains(&scenario)
    );
    let app = fltk::app::App::default();
    let _app = app;
    pp_switch::ui::mac_acceptance(store.clone(), paths.clone(), scenario)
        .expect("native acceptance");
    assert!(
        store.load().unwrap().is_empty(),
        "acceptance wrote provider store"
    );
    for client in pp_switch::provider::Client::ALL {
        assert!(
            !paths.settings(client).exists(),
            "acceptance wrote client configuration"
        );
    }
    assert!(
        !scenario.starts_with("quit-"),
        "Cocoa did not terminate the process"
    );
}

#[cfg(not(target_os = "macos"))]
fn main() {
    support::require_desktop_input();
    eprintln!("macOS-only acceptance harness");
    std::process::exit(2);
}
