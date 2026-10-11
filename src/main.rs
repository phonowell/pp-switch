#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = pp_switch::ui::run() {
        fltk::dialog::alert_default(&error);
        std::process::exit(1);
    }
}
