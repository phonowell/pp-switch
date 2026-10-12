#[cfg(target_os = "macos")]
fn main() {
    let exe = std::env::current_exe().unwrap();
    let resources = exe.parent().unwrap().parent().unwrap().join("Resources");
    let root = std::fs::read_to_string(resources.join("fixture-path.txt")).unwrap();
    let root = std::path::PathBuf::from(root);
    std::fs::write(root.join("pid.txt"), std::process::id().to_string()).unwrap();
    pp_switch::ui::protocol_smoke(
        pp_switch::storage::Store {
            root: root.join("store"),
        },
        pp_switch::client::Paths::from_home(root.join("home")),
        root,
    )
    .expect("native URL events with disposable files");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
