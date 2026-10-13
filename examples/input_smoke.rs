mod support;

#[cfg(target_os = "macos")]
mod native {
    use fltk::{prelude::*, window::Window};
    use pp_switch::ui::acceptance::{InputDriver, NativeKey};
    use std::{
        ffi::{CString, c_void},
        ptr,
    };
    unsafe extern "C" {
        fn pp_smoke_available() -> i32;
        fn pp_smoke_appearance(dark: i32);
        fn pp_smoke_has_shadow(window: *mut c_void) -> i32;
        fn pp_smoke_rounded_menu(window: *mut c_void) -> i32;
        fn pp_smoke_title_icon(window: *mut c_void) -> i32;
        fn pp_smoke_capture_chrome(window: *mut c_void, path: *const std::ffi::c_char) -> i32;
        fn pp_smoke_mouse(window: *mut c_void, x: i32, y: i32, down: i32);
        fn pp_smoke_key(
            window: *mut c_void,
            key: u16,
            modifiers: u32,
            text: *const std::ffi::c_char,
        );
        fn pp_smoke_close(window: *mut c_void);
        fn pp_smoke_windows() -> i32;
        fn pp_smoke_restore_mouse();
    }
    pub struct Cocoa;
    impl Cocoa {
        pub fn available() -> bool {
            unsafe { pp_smoke_available() != 0 }
        }
    }
    impl Drop for Cocoa {
        fn drop(&mut self) {
            unsafe {
                pp_smoke_restore_mouse();
            }
        }
    }
    impl InputDriver for Cocoa {
        fn appearance(&self, dark: Option<bool>) {
            unsafe {
                pp_smoke_appearance(dark.map_or(-1, i32::from));
            }
        }
        fn menu_has_shadow(&self, window: &dyn WindowExt) -> bool {
            unsafe { pp_smoke_has_shadow(window.raw_handle()) != 0 }
        }
        fn menu_is_rounded(&self, window: &dyn WindowExt) -> bool {
            unsafe { pp_smoke_rounded_menu(window.raw_handle()) != 0 }
        }
        fn title_icon_visible(&self, window: &Window) -> bool {
            unsafe { pp_smoke_title_icon(window.raw_handle()) != 0 }
        }
        fn capture_chrome(
            &self,
            window: &dyn WindowExt,
            path: &std::path::Path,
        ) -> pp_switch::Result<()> {
            let path = CString::new(path.to_str().ok_or("Invalid screenshot path.")?)
                .map_err(|_| "Invalid screenshot path.")?;
            if unsafe { pp_smoke_capture_chrome(window.raw_handle(), path.as_ptr()) } == 0 {
                return Err("Cannot capture native window title bar.".into());
            }
            Ok(())
        }
        fn mouse(&self, window: &Window, x: i32, y: i32, down: bool) {
            unsafe {
                pp_smoke_mouse(window.raw_handle(), x, y, down as i32);
            }
        }
        fn key(&self, window: Option<&Window>, key: NativeKey) {
            let (code, modifiers, text) = match key {
                NativeKey::Escape => (53, 0, "\u{1b}"),
                NativeKey::Space => (49, 0, " "),
                NativeKey::Tab => (48, 0, "\t"),
                NativeKey::BackTab => (48, 1 << 17, "\t"),
                NativeKey::Up => (126, 0, "\u{f700}"),
                NativeKey::Down => (125, 0, "\u{f701}"),
                NativeKey::Right => (124, 0, "\u{f703}"),
                NativeKey::Enter => (36, 0, "\r"),
                NativeKey::SelectAll => (0, 1 << 20, "a"),
                NativeKey::Backspace => (51, 0, "\u{8}"),
                NativeKey::Text(text) => (0, 0, text),
            };
            let text = CString::new(text).unwrap();
            unsafe {
                pp_smoke_key(
                    window.map_or(ptr::null_mut(), |w| w.raw_handle()),
                    code,
                    modifiers,
                    text.as_ptr(),
                );
            }
        }
        fn close(&self, window: &Window) {
            unsafe {
                pp_smoke_close(window.raw_handle());
            }
        }
        fn visible_windows(&self) -> i32 {
            unsafe { pp_smoke_windows() }
        }
    }
}

fn main() -> std::process::ExitCode {
    support::require_desktop_input();
    #[cfg(target_os = "macos")]
    {
        if !native::Cocoa::available() {
            eprintln!(
                "BLOCKED: macOS graphical session is unavailable; no native input acceptance ran."
            );
            return std::process::ExitCode::from(2);
        }
        let root = tempfile::tempdir().expect("isolated native input files");
        let output = std::env::args_os()
            .nth(1)
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "target/input-smoke".into());
        std::fs::create_dir_all(&output).unwrap();
        if std::env::args_os().any(|arg| arg == "--themes-only") {
            use pp_switch::ui::acceptance::InputDriver;
            let driver = std::rc::Rc::new(native::Cocoa);
            let result = (|| -> pp_switch::Result<()> {
                if !std::env::args_os().any(|arg| arg == "--style-only") {
                    pp_switch::ui::acceptance::appearance_regression(
                        pp_switch::storage::Store {
                            root: root.path().join("appearance-store"),
                        },
                        pp_switch::client::Paths::from_home(root.path().join("appearance-clients")),
                        &output,
                        driver.clone(),
                    )?;
                }
                for dark in [false, true] {
                    driver.appearance(Some(dark));
                    let directory = output.join(if dark { "dark" } else { "light" });
                    std::fs::create_dir_all(&directory)
                        .map_err(|_| "Cannot create theme evidence.")?;
                    pp_switch::ui::acceptance::run(
                        pp_switch::storage::Store {
                            root: root
                                .path()
                                .join(if dark { "dark-store" } else { "light-store" }),
                        },
                        pp_switch::client::Paths::from_home(root.path().join(if dark {
                            "dark-clients"
                        } else {
                            "light-clients"
                        })),
                        &directory,
                        driver.clone(),
                        false,
                    )?;
                }
                Ok(())
            })();
            driver.appearance(None);
            if let Err(error) = result {
                eprintln!("FAILED: appearance acceptance: {error}");
                return std::process::ExitCode::FAILURE;
            }
            return std::process::ExitCode::SUCCESS;
        }
        let style_only = std::env::args_os().any(|arg| arg == "--style-only");
        if !style_only {
            let result = pp_switch::ui::acceptance::interaction_regression(
                pp_switch::storage::Store {
                    root: root.path().join("interaction-store"),
                },
                pp_switch::client::Paths::from_home(root.path().join("interaction-clients")),
                &output,
                std::rc::Rc::new(native::Cocoa),
            );
            if let Err(error) = result {
                eprintln!("FAILED: targeted input acceptance: {error}");
                return std::process::ExitCode::FAILURE;
            }
        }
        let result = pp_switch::ui::acceptance::run(
            pp_switch::storage::Store {
                root: root.path().join("store"),
            },
            pp_switch::client::Paths::from_home(root.path().to_owned()),
            &output,
            std::rc::Rc::new(native::Cocoa),
            false,
        );
        if let Err(error) = result {
            eprintln!("FAILED: native input acceptance: {error}");
            return std::process::ExitCode::FAILURE;
        }
        std::process::ExitCode::SUCCESS
    }
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("Native Cocoa input acceptance requires macOS; no input check ran.");
        std::process::ExitCode::FAILURE
    }
}
