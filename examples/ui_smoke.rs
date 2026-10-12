fn main() {
    let root = tempfile::tempdir().expect("temporary smoke directory");
    let paths = pp_switch::client::Paths::from_home(root.path().to_owned());
    let store = pp_switch::storage::Store {
        root: root.path().join("store"),
    };
    let screenshot = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    pp_switch::ui::smoke(store, paths, screenshot).expect("GUI behavior smoke test");
    println!(
        "PP-01/02/10/13/14: Simplified Chinese GUI save, switch, new, row actions, settings, selection, draft preservation and close passed."
    );
}
