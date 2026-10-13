//! Real shipped FLTK lifecycle with isolated files and suppressed presentation.
#[cfg(target_os = "macos")]
fn main() {
    unsafe extern "C" {
        fn pp_quit_background_prepare();
    }
    unsafe { pp_quit_background_prepare() };
    let root = tempfile::tempdir().expect("fixture directory");
    let paths = pp_switch::client::Paths::from_home(root.path().join("home"));
    let store = pp_switch::storage::Store {
        root: root.path().join("store"),
    };
    let scenario = std::env::args().nth(1).expect("quit route and page");
    if scenario == "model-prefill" {
        pp_switch::ui::mac_model_prefill_background(store, paths).expect("model prefill fixture");
        panic!("Cocoa quit returned without process termination");
    }
    pp_switch::ui::mac_quit_background(store, paths, &scenario).expect("background lifecycle");
    panic!("Cocoa quit returned without process termination");
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("macOS-only background lifecycle test");
    std::process::exit(2);
}
