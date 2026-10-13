//! The production window and event loop, with an isolated home and a test-only timer.
mod support;
use fltk::app;
use std::io::Write;

fn main() {
    support::require_desktop_input();
    let root = tempfile::tempdir().expect("temporary measurement directory");
    let paths = pp_switch::client::Paths::from_home(root.path().to_owned());
    let store = pp_switch::storage::Store {
        root: root.path().join("store"),
    };
    let _app = app::App::default();
    app::add_timeout3(0.0, |_| {
        println!("READY");
        std::io::stdout().flush().unwrap();
        app::add_timeout3(3.0, |_| {
            // Cocoa's wait() processes timers before waiting for native events.
            // After hiding the last window, bound that final native wait too.
            app::add_timeout3(0.0, |_| {});
            app::quit();
        });
    });
    if std::env::args().any(|arg| arg == "--editor") {
        // Saved synthetic row exercises the editor-sized scenario without links.
        store
            .save(
                pp_switch::provider::Provider::new(
                    pp_switch::provider::Client::Claude,
                    "Example",
                    "https://api.example.com",
                    "measurement-fake-key",
                )
                .unwrap(),
            )
            .unwrap();
        pp_switch::ui::run_measure_editor(store, paths).expect("measurement editor");
    } else {
        pp_switch::ui::run_with(store, paths).expect("measurement window");
    }
}
