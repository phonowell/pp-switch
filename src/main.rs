#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 2 && args[0] == "--pp-install" {
        if let Err(error) = pp_switch::update::helper(std::path::Path::new(&args[1])) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    if args.len() == 3
        && args[0] == "--pp-clean-update"
        && let Some(pid) = args[2].to_str().and_then(|s| s.parse().ok())
    {
        pp_switch::update::cleanup(std::path::Path::new(&args[1]), pid);
    }
    if let Err(error) = pp_switch::ui::run(pp_switch::protocol::argument(&args)) {
        let language = pp_switch::storage::Store::system()
            .and_then(|s| s.language())
            .unwrap_or_else(|_| pp_switch::i18n::Language::system());
        pp_switch::i18n::alert(language, &error);
        std::process::exit(1);
    }
}
