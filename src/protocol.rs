//! User-triggered URL association. Existing ccswitch handlers keep ownership.
use crate::Result;

const REGISTRATION_ERROR: &str = "Cannot register import links. Run from the installed package.";

/// URL arguments are a delivery mechanism, never command-line management.
pub fn argument(args: &[std::ffi::OsString]) -> Option<&str> {
    if args.len() == 1 {
        args[0].to_str().filter(|s| {
            let scheme = s.split(':').next().unwrap_or_default();
            scheme.eq_ignore_ascii_case("ccswitch") || scheme.eq_ignore_ascii_case("ppswitch")
        })
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::{
        cell::RefCell,
        ffi::{CStr, c_char},
    };
    type Receiver = Box<dyn FnMut(&str)>;
    thread_local! { static RECEIVER: RefCell<Option<Receiver>> = RefCell::new(None); }
    thread_local! { static PENDING: RefCell<std::collections::VecDeque<String>> = const { RefCell::new(std::collections::VecDeque::new()) }; }
    unsafe extern "C" {
        fn pp_listen_urls(callback: extern "C" fn(*const c_char));
        pub fn pp_register_urls() -> i32;
    }
    extern "C" fn receive(url: *const c_char) {
        if url.is_null() {
            return;
        }
        // Cocoa supplies a valid NUL-terminated UTF-8 string during this callback.
        if let Ok(url) = unsafe { CStr::from_ptr(url) }.to_str() {
            PENDING.with(|pending| {
                let mut pending = pending.borrow_mut();
                if pending.len() < 8 {
                    pending.push_back(url.to_owned());
                }
            });
            dispatch();
        }
    }
    fn dispatch() {
        // A Cocoa event can reenter while show() runs. Queue it rather than
        // borrowing a callback that is already executing.
        let Some(mut callback) = RECEIVER.with(|receiver| receiver.borrow_mut().take()) else {
            return;
        };
        while let Some(url) = PENDING.with(|pending| pending.borrow_mut().pop_front()) {
            callback(&url);
        }
        RECEIVER.with(|receiver| *receiver.borrow_mut() = Some(callback));
    }
    pub fn prepare() {
        // Install before FLTK initializes Cocoa: finishLaunching may deliver the
        // first URL before the window and callbacks have been constructed.
        unsafe {
            pp_listen_urls(receive);
        }
    }
    pub fn listen(callback: impl FnMut(&str) + 'static) {
        RECEIVER.with(|receiver| *receiver.borrow_mut() = Some(Box::new(callback)));
        dispatch();
    }
}

#[cfg(target_os = "macos")]
pub use macos::{listen, prepare};

#[cfg(target_os = "macos")]
pub fn register() -> Result<()> {
    if unsafe { macos::pp_register_urls() } == 1 {
        Ok(())
    } else {
        Err(REGISTRATION_ERROR.into())
    }
}

#[cfg(windows)]
pub fn register() -> Result<()> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::{Foundation::ERROR_SUCCESS, System::Registry::*};
    fn wide(s: &OsStr) -> Vec<u16> {
        s.encode_wide().chain(Some(0)).collect()
    }
    fn set(path: &str, name: &str, value: &str) -> Result<()> {
        let mut key = std::ptr::null_mut();
        unsafe {
            if RegCreateKeyExW(
                HKEY_CURRENT_USER,
                wide(OsStr::new(path)).as_ptr(),
                0,
                std::ptr::null(),
                0,
                KEY_SET_VALUE,
                std::ptr::null(),
                &mut key,
                std::ptr::null_mut(),
            ) != ERROR_SUCCESS
            {
                return Err(REGISTRATION_ERROR.into());
            }
            let value = wide(OsStr::new(value));
            let status = RegSetValueExW(
                key,
                wide(OsStr::new(name)).as_ptr(),
                0,
                REG_SZ,
                value.as_ptr().cast(),
                (value.len() * 2) as u32,
            );
            RegCloseKey(key);
            if status != ERROR_SUCCESS {
                return Err(REGISTRATION_ERROR.into());
            }
        }
        Ok(())
    }
    let exe = std::env::current_exe().map_err(|_| REGISTRATION_ERROR)?;
    for scheme in ["ppswitch", "ccswitch"] {
        // HKCR is the merged user/system view. Do not replace an existing command.
        let command_key = wide(OsStr::new(&format!("{scheme}\\shell\\open\\command")));
        let mut key = std::ptr::null_mut();
        unsafe {
            let status = RegOpenKeyExW(
                HKEY_CLASSES_ROOT,
                command_key.as_ptr(),
                0,
                KEY_READ,
                &mut key,
            );
            if status == ERROR_SUCCESS {
                let mut data = vec![0u16; 16384];
                let mut bytes = (data.len() * 2) as u32;
                let mut kind = 0;
                let read = RegQueryValueExW(
                    key,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    &mut kind,
                    data.as_mut_ptr().cast(),
                    &mut bytes,
                );
                RegCloseKey(key);
                // Keep indirect/unreadable handlers. Reclaim a clearly missing
                // absolute executable (e.g. after cc-switch was uninstalled).
                let command = if read == ERROR_SUCCESS && kind == REG_SZ && bytes.is_multiple_of(2)
                {
                    String::from_utf16_lossy(&data[..bytes as usize / 2])
                        .trim_end_matches('\0')
                        .to_owned()
                } else {
                    String::new()
                };
                let program = command
                    .strip_prefix('"')
                    .and_then(|s| s.split_once('"').map(|v| v.0))
                    .or_else(|| command.split_whitespace().next());
                if !program.is_some_and(|p| {
                    std::path::Path::new(p).is_absolute() && !std::path::Path::new(p).exists()
                }) {
                    continue;
                }
            }
            if status != ERROR_SUCCESS
                && status != windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND
            {
                return Err(REGISTRATION_ERROR.into());
            }
        }
        let path = format!("Software\\Classes\\{scheme}");
        set(&path, "", "URL:pp-switch Provider import")?;
        set(&path, "URL Protocol", "")?;
        set(
            &format!("{path}\\shell\\open\\command"),
            "",
            &format!("\"{}\" \"%1\"", exe.display()),
        )?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn register() -> Result<()> {
    use std::{fs, process::Command};
    let exe = std::env::current_exe().map_err(|_| REGISTRATION_ERROR)?;
    let exe = exe.to_str().ok_or(REGISTRATION_ERROR)?;
    // Desktop Entry quoting is distinct from shell quoting. Escape Exec field codes.
    let quoted = desktop_exec(exe)?;
    let root = dirs::data_dir()
        .ok_or(REGISTRATION_ERROR)?
        .join("applications");
    fs::create_dir_all(&root).map_err(|_| REGISTRATION_ERROR)?;
    let desktop = root.join("io.github.phonowell.pp-switch.desktop");
    if fs::symlink_metadata(&desktop).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(REGISTRATION_ERROR.into());
    }
    fs::write(&desktop, format!("[Desktop Entry]\nType=Application\nName=pp-switch\nExec={quoted} %u\nTerminal=false\nNoDisplay=true\nMimeType=x-scheme-handler/ppswitch;x-scheme-handler/ccswitch;\n"))
        .map_err(|_| REGISTRATION_ERROR)?;
    for scheme in ["ppswitch", "ccswitch"] {
        let mime = format!("x-scheme-handler/{scheme}");
        let output = Command::new("xdg-mime")
            .args(["query", "default", &mime])
            .output()
            .map_err(|_| REGISTRATION_ERROR)?;
        if !output.status.success() {
            return Err(REGISTRATION_ERROR.into());
        }
        let handler = std::str::from_utf8(&output.stdout)
            .map_err(|_| REGISTRATION_ERROR)?
            .trim();
        let mut data_dirs = vec![root.parent().unwrap().to_owned()];
        data_dirs.extend(std::env::split_paths(
            &std::env::var_os("XDG_DATA_DIRS")
                .unwrap_or_else(|| "/usr/local/share:/usr/share".into()),
        ));
        if !handler.is_empty()
            && (handler.contains('/')
                || data_dirs
                    .iter()
                    .any(|dir| dir.join("applications").join(handler).is_file()))
        {
            continue;
        }
        if !Command::new("xdg-mime")
            .args(["default", "io.github.phonowell.pp-switch.desktop", &mime])
            .status()
            .map_err(|_| REGISTRATION_ERROR)?
            .success()
        {
            return Err(REGISTRATION_ERROR.into());
        }
    }
    Ok(())
}

#[cfg(any(target_os = "linux", test))]
fn desktop_exec(exe: &str) -> Result<String> {
    if exe.chars().any(char::is_control) {
        return Err(REGISTRATION_ERROR.into());
    }
    // The desktop parser unescapes strings before parsing Exec quoting.
    let mut quoted = String::from("\"");
    for ch in exe.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\\\\\"),
            '"' | '`' | '$' => {
                quoted.push_str("\\\\");
                quoted.push(ch);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    Ok(quoted)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pp06_only_one_scheme_argument_is_delivered() {
        assert!(argument(&[]).is_none());
        assert!(argument(&["--pp-install".into(), "ccswitch://fake".into()]).is_none());
        assert!(argument(&["https://example.com".into()]).is_none());
        assert_eq!(
            argument(&["CCSwitch://v1/import?resource=provider".into()]),
            Some("CCSwitch://v1/import?resource=provider")
        );
    }
    #[test]
    fn pp06_desktop_path_cannot_inject_exec_fields() {
        assert_eq!(
            desktop_exec("/tmp/a %u\"$`\\/pp-switch").unwrap(),
            "\"/tmp/a %%u\\\\\"\\\\$\\\\`\\\\\\\\/pp-switch\""
        );
        assert!(desktop_exec("/tmp/bad\nExec=other").is_err());
    }
}
