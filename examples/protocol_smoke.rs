#[cfg(target_os = "macos")]
fn main() {
    let exe = std::env::current_exe().unwrap();
    let resources = exe.parent().unwrap().parent().unwrap().join("Resources");
    let root = std::fs::read_to_string(resources.join("fixture-path.txt")).unwrap();
    let root = std::path::PathBuf::from(root);
    if root.join("register-only.txt").exists() {
        let finished = root.clone();
        fltk::app::add_timeout3(0.0, move |_| {
            pp_switch::protocol::register().expect("register isolated CI bundle");
            std::fs::write(finished.join("registered.txt"), "ok").unwrap();
            fltk::app::add_timeout3(0.0, |_| {});
            fltk::app::quit();
        });
        pp_switch::ui::run_with(
            pp_switch::storage::Store {
                root: root.join("store"),
            },
            pp_switch::client::Paths::from_home(root.join("home")),
        )
        .expect("registration after the production window launches");
        return;
    }
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
