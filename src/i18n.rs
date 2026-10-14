//! Small compiled catalogs. No locale downloads, global mutable state or runtime.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Language {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh")]
    Zh,
    #[serde(rename = "zh-TW")]
    ZhTw,
    #[serde(rename = "ja")]
    Ja,
}

impl Language {
    pub const ALL: [Self; 4] = [Self::En, Self::Zh, Self::ZhTw, Self::Ja];

    pub fn code(self) -> &'static str {
        ["en", "zh", "zh-TW", "ja"][self.index()]
    }

    pub fn index(self) -> usize {
        match self {
            Self::En => 0,
            Self::Zh => 1,
            Self::ZhTw => 2,
            Self::Ja => 3,
        }
    }

    pub fn from_locale(locale: &str) -> Self {
        let locale = locale.to_ascii_lowercase().replace('_', "-");
        let language = locale.split(['-', '.', '@']).next().unwrap_or("");
        match language {
            "zh" if locale.contains("hant")
                || locale.starts_with("zh-tw")
                || locale.starts_with("zh-hk")
                || locale.starts_with("zh-mo") =>
            {
                Self::ZhTw
            }
            "zh" => Self::Zh,
            "ja" => Self::Ja,
            _ => Self::En,
        }
    }

    pub fn system() -> Self {
        if let Some(locale) = native_locale() {
            return Self::from_locale(&locale);
        }
        for name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(locale) = std::env::var(name)
                && !locale.is_empty()
            {
                return Self::from_locale(&locale);
            }
        }
        Self::En
    }
}

#[cfg(target_os = "macos")]
fn native_locale() -> Option<String> {
    use std::ffi::{CStr, c_void};
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFLocaleCopyPreferredLanguages() -> *const c_void;
        fn CFArrayGetCount(array: *const c_void) -> isize;
        fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
        fn CFStringGetCString(s: *const c_void, buf: *mut i8, size: isize, encoding: u32) -> u8;
        fn CFRelease(value: *const c_void);
    }
    // Copy owns the array; its first string is borrowed until we release it.
    unsafe {
        let array = CFLocaleCopyPreferredLanguages();
        if array.is_null() {
            return None;
        }
        let mut buf = [0i8; 128];
        let value = if CFArrayGetCount(array) > 0
            && CFStringGetCString(
                CFArrayGetValueAtIndex(array, 0),
                buf.as_mut_ptr(),
                buf.len() as isize,
                0x0800_0100,
            ) != 0
        {
            Some(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned())
        } else {
            None
        };
        CFRelease(array);
        value
    }
}

#[cfg(windows)]
fn native_locale() -> Option<String> {
    let mut buffer = [0u16; 85];
    // Windows guarantees a terminating NUL within the returned length.
    let count = unsafe {
        windows_sys::Win32::Globalization::GetUserDefaultLocaleName(buffer.as_mut_ptr(), 85)
    };
    (count > 1).then(|| String::from_utf16_lossy(&buffer[..count as usize - 1]))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn native_locale() -> Option<String> {
    None
}

pub const CATALOG: &str = include_str!("locales.tsv");

pub fn tr(language: Language, english: &str) -> &str {
    if language == Language::En {
        return english;
    }
    CATALOG
        .lines()
        .find_map(|line| {
            let mut fields = line.split('\t');
            (fields.next() == Some(english))
                .then(|| fields.nth(language.index() - 1))
                .flatten()
        })
        .unwrap_or(english)
}

/// Translate complete core errors without coupling config/storage logic to UI state.
pub fn message(language: Language, english: &str) -> String {
    let direct = tr(language, english);
    if direct != english || language == Language::En {
        return direct.to_owned();
    }
    if let Some(path) = english
        .strip_prefix("Cannot read ")
        .and_then(|s| s.strip_suffix('.'))
    {
        return tr(language, "Cannot read {path}.").replace("{path}", path);
    }
    if let Some(name) = english.strip_suffix(" is set in this process's environment. File edits may be overridden; launch pp-switch without this override.") {
        return tr(language, "{name} is set in this process's environment. File edits may be overridden; launch pp-switch without this override.").replace("{name}", name);
    }
    if let Some(url) = english
        .strip_prefix("Current: ")
        .and_then(|s| s.strip_suffix("  ·  * marks a matching provider"))
    {
        return tr(language, "Current: {url}  ·  * marks a matching provider")
            .replace("{url}", url);
    }
    for prefix in ["Current: ", "Switch to: "] {
        if let Some(name) = english.strip_prefix(prefix) {
            return tr(language, &format!("{prefix}{{name}}")).replace("{name}", name);
        }
    }
    english.to_owned()
}

/// Application messages use the same flat controls as editor/import windows.
pub fn alert(language: Language, english: &str) {
    let _ = confirm(
        language,
        &message(language, english),
        tr(language, "Close"),
        None,
    );
}

pub fn confirm(language: Language, text: &str, cancel: &str, action: Option<&str>) -> bool {
    confirm_with_link(language, text, cancel, action, None)
}

pub(crate) fn confirm_with_link(
    language: Language,
    text: &str,
    cancel: &str,
    action: Option<&str>,
    link: Option<&str>,
) -> bool {
    use crate::theme::{self, ButtonStyle};
    use fltk::{
        app,
        button::Button,
        enums::{Align, Font},
        frame::Frame,
        prelude::*,
        window::Window,
    };
    use std::{cell::Cell, rc::Rc};
    let _app = app::App::default();
    theme::init();
    // Bound wrapping for long localized messages, including scripts without spaces.
    fltk::draw::set_font(Font::Helvetica, 13);
    let mut line = String::new();
    let mut wrapped = String::new();
    for ch in text.chars() {
        if ch == '\n' {
            wrapped.push_str(&line);
            wrapped.push('\n');
            line.clear();
            continue;
        }
        line.push(ch);
        if fltk::draw::measure(&line, false).0 > 310 {
            // Prefer word boundaries in Latin text; CJK and long unbroken
            // tokens still need a character boundary to stay inside the frame.
            let split = line.rfind(' ').filter(|index| *index > 0);
            let remainder = if let Some(index) = split {
                let remainder = line[index + 1..].to_owned();
                wrapped.push_str(&line[..index]);
                remainder
            } else {
                line.pop();
                wrapped.push_str(&line);
                ch.to_string()
            };
            wrapped.push('\n');
            line = remainder;
        }
    }
    wrapped.push_str(&line);
    let lines = wrapped.lines().count() as i32;
    let text_height = (lines * 21).max(42);
    let height = text_height + 108 + if link.is_some() { 44 } else { 0 };
    let parent = app::first_window();
    let (x, y) = parent.map_or((130, 160), |w| {
        (w.x() + (w.w() - 386) / 2, w.y() + (w.h() - height) / 2)
    });
    let mut window = Window::new(x, y, 386, height, "pp-switch");
    window.make_modal(true);
    window.set_color(theme::background());
    let mut copy = Frame::new(24, 24, 338, text_height, None);
    copy.set_label(&wrapped);
    copy.set_label_font(Font::Helvetica);
    copy.set_label_size(13);
    copy.set_label_color(theme::ink());
    copy.set_align(Align::Inside | Align::Left | Align::Top | Align::Wrap);
    if let Some(url) = link {
        let mut notes = Button::new(24, text_height + 36, 338, 32, None);
        notes.set_label(tr(language, "Release notes"));
        theme::button(&mut notes, ButtonStyle::Secondary);
        let url = url.to_owned();
        notes.set_callback(move |_| {
            if fltk::utils::open_uri(&url).is_err() {
                alert(
                    language,
                    "Cannot open release notes. Visit the repository's Releases page.",
                );
            }
        });
    }
    let result = Rc::new(Cell::new(false));
    let mut close = Button::new(
        if action.is_some() { 146 } else { 258 },
        height - 60,
        104,
        36,
        None,
    );
    close.set_label(cancel);
    theme::button(&mut close, ButtonStyle::Secondary);
    let mut view = window.clone();
    close.set_callback(move |_| view.hide());
    if let Some(action) = action {
        let mut accept = Button::new(262, height - 60, 100, 36, None);
        accept.set_label(action);
        // Allow longer localized update actions without truncating the label.
        let width = fltk::draw::measure(action, false).0.max(76) + 24;
        accept.resize(362 - width, height - 60, width, 36);
        close.resize(362 - width - 116, height - 60, 104, 36);
        theme::button(&mut accept, ButtonStyle::Primary);
        let chosen = result.clone();
        let mut view = window.clone();
        accept.set_callback(move |_| {
            chosen.set(true);
            view.hide();
        });
    }
    window.end();
    window.set_callback(|w| w.hide());
    window.show();
    let _ = close.take_focus();
    while window.shown() {
        app::wait();
    }
    Window::delete(window);
    result.get()
}
