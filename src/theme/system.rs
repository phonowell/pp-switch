//! Read appearance only; never change OS settings or launch resident helpers.
#[cfg(target_os = "macos")]
pub(super) fn dark() -> bool {
    unsafe extern "C" {
        fn pp_dark_appearance() -> i32;
    }
    unsafe { pp_dark_appearance() != 0 }
}

#[cfg(windows)]
pub(super) fn dark() -> bool {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
    let path: Vec<_> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let name: Vec<_> = "AppsUseLightTheme\0".encode_utf16().collect();
    let mut value: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut value as *mut u32).cast(),
            &mut size,
        )
    };
    result == 0 && value == 0
}

#[cfg(target_os = "linux")]
pub(super) fn dark() -> bool {
    if let Ok(theme) = std::env::var("GTK_THEME") {
        return theme.to_ascii_lowercase().contains("dark");
    }
    if let Some(dark) = gnome() {
        return dark;
    }
    let Some(config) = dirs::config_dir() else {
        return false;
    };
    for path in ["gtk-4.0/settings.ini", "gtk-3.0/settings.ini", "kdeglobals"] {
        if let Ok(text) = std::fs::read_to_string(config.join(path)) {
            for line in text.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    let value = value.trim().to_ascii_lowercase();
                    match key.trim() {
                        "gtk-application-prefer-dark-theme" => {
                            return value == "true" || value == "1";
                        }
                        "gtk-theme-name" | "ColorScheme" => return value.contains("dark"),
                        _ => (),
                    }
                }
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn gnome() -> Option<bool> {
    use std::{
        ffi::{c_char, c_void},
        sync::OnceLock,
    };
    type GetEnum = unsafe extern "C" fn(*mut c_void, *const c_char) -> i32;
    type Iterate = unsafe extern "C" fn(*mut c_void, i32) -> i32;
    struct Settings {
        object: usize,
        get: GetEnum,
        iterate: Iterate,
    }
    // GIO is optional and supplied by the desktop. Retain one settings object
    // for the GUI lifetime; no child process, service or new linked dependency.
    static SETTINGS: OnceLock<Option<Settings>> = OnceLock::new();
    let settings = SETTINGS.get_or_init(|| unsafe {
        let library = libc::dlopen(
            c"libgio-2.0.so.0".as_ptr(),
            libc::RTLD_LAZY | libc::RTLD_LOCAL,
        );
        if library.is_null() {
            return None;
        }
        macro_rules! symbol {
            ($name:literal, $ty:ty) => {{
                let ptr = libc::dlsym(library, $name.as_ptr());
                if ptr.is_null() {
                    return None;
                }
                std::mem::transmute::<*mut c_void, $ty>(ptr)
            }};
        }
        let source = symbol!(
            c"g_settings_schema_source_get_default",
            unsafe extern "C" fn() -> *mut c_void
        )();
        if source.is_null() {
            return None;
        }
        let lookup = symbol!(
            c"g_settings_schema_source_lookup",
            unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void
        );
        let schema = lookup(source, c"org.gnome.desktop.interface".as_ptr(), 1);
        if schema.is_null() {
            return None;
        }
        let has_key = symbol!(
            c"g_settings_schema_has_key",
            unsafe extern "C" fn(*mut c_void, *const c_char) -> i32
        );
        let unref = symbol!(
            c"g_settings_schema_unref",
            unsafe extern "C" fn(*mut c_void)
        );
        if has_key(schema, c"color-scheme".as_ptr()) == 0 {
            unref(schema);
            return None;
        }
        let new = symbol!(
            c"g_settings_new_full",
            unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char) -> *mut c_void
        );
        let object = new(schema, std::ptr::null_mut(), std::ptr::null());
        unref(schema);
        if object.is_null() {
            return None;
        }
        Some(Settings {
            object: object as usize,
            get: symbol!(c"g_settings_get_enum", GetEnum),
            iterate: symbol!(c"g_main_context_iteration", Iterate),
        })
    });
    settings.as_ref().and_then(|s| {
        // FLTK owns the event loop; drain desktop preference notifications
        // without blocking or introducing a second loop/thread.
        for _ in 0..8 {
            if unsafe { (s.iterate)(std::ptr::null_mut(), 0) } == 0 {
                break;
            }
        }
        match unsafe { (s.get)(s.object as *mut c_void, c"color-scheme".as_ptr()) } {
            1 => Some(true),
            2 => Some(false),
            _ => None,
        }
    })
}
