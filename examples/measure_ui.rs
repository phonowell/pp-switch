//! The production window and event loop, with an isolated home and a test-only timer.
use fltk::app;
use std::io::Write;

fn main() {
    let root = tempfile::tempdir().expect("temporary measurement directory");
    let paths = pp_switch::client::Paths::from_home(root.path().to_owned());
    let store = pp_switch::storage::Store {
        root: root.path().join("store"),
    };
    let _app = app::App::default();
    app::add_timeout3(0.0, |_| {
        println!("READY");
        std::io::stdout().flush().unwrap();
        app::add_timeout3(3.0, |_| app::quit());
    });
    pp_switch::ui::run_with(store, paths).expect("measurement window");
}
