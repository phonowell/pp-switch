mod support;

fn main() {
    support::require_desktop_input();
    let root = tempfile::tempdir().expect("temporary smoke directory");
    let paths = pp_switch::client::Paths::from_home(root.path().to_owned());
    let store = pp_switch::storage::Store {
        root: root.path().join("store"),
    };
    let screenshot = std::env::args_os()
        .skip(1)
        .find(|arg| arg != "--allow-desktop-input")
        .map(std::path::PathBuf::from);
    pp_switch::ui::smoke(store, paths, screenshot).expect("GUI behavior smoke test");
    println!(
        "PP-01/02/10/13/14: Simplified Chinese GUI save, switch, new, row actions, settings, draft preservation and close passed."
    );
}
