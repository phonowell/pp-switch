//! Flat native surfaces following the system appearance.
mod system;
use fltk::{
    app,
    button::Button,
    draw,
    enums::{Align, Color, Font, FrameType},
    prelude::*,
};
use std::sync::atomic::{AtomicBool, Ordering};

static DARK: AtomicBool = AtomicBool::new(false);
type Rgb = (u8, u8, u8);
const LIGHT: [Rgb; 14] = [
    (247, 248, 250),
    (40, 45, 56),
    (97, 106, 121),
    (54, 93, 216),
    (228, 231, 236),
    (237, 241, 251),
    (233, 236, 241),
    (172, 64, 64),
    (240, 242, 246),
    (219, 227, 245),
    (183, 190, 202),
    (57, 121, 95),
    (116, 125, 139),
    (46, 83, 200),
];
const DARK_PALETTE: [Rgb; 14] = [
    (30, 32, 38),
    (235, 238, 245),
    (169, 177, 192),
    (75, 110, 224),
    (62, 68, 82),
    (40, 52, 82),
    (46, 50, 60),
    (242, 132, 132),
    (39, 43, 52),
    (68, 89, 132),
    (121, 132, 153),
    (120, 204, 166),
    (133, 143, 162),
    (62, 91, 190),
];
// FLTK's 16 user color slots retain their identity in existing widgets. Changing
// their RGB values recolors open dialogs without rebuilding widgets or drafts.
macro_rules! colors {
    ($($name:ident = $slot:expr),* $(,)?) => { $(
        pub fn $name() -> Color { Color::by_index(16 + $slot) }
    )* };
}
colors! { background=0, ink=1, muted=2, accent=3, line=4, selected=5,
hover=6, danger=7, field_fill=8, selected_border=9, radio_outline=10,
active_badge=11, disabled_text=12, pressed_accent=13 }

pub fn is_dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

fn apply_appearance(dark: bool) {
    DARK.store(dark, Ordering::Relaxed);
    let palette = if dark { DARK_PALETTE } else { LIGHT };
    for (slot, (r, g, b)) in palette.into_iter().enumerate() {
        app::set_color(Color::by_index(16 + slot as u8), r, g, b);
    }
    let (r, g, b) = palette[0];
    app::background(r, g, b);
    let (r, g, b) = palette[1];
    app::foreground(r, g, b);
    fltk::misc::Tooltip::set_color(hover());
    fltk::misc::Tooltip::set_text_color(ink());
    app::redraw();
}

#[cfg(target_os = "macos")]
extern "C" fn appearance_changed(dark: i32) {
    apply_appearance(dark != 0);
}

pub fn menu_frame() -> FrameType {
    FrameType::FreeBoxType
}

fn menu_surface(x: i32, y: i32, w: i32, h: i32, _color: Color) {
    #[cfg(target_os = "macos")]
    for window in app::windows().unwrap_or_default() {
        if window.shown() && !window.border() && window.frame() == menu_frame() {
            unsafe extern "C" {
                fn pp_round_menu(window: *mut std::ffi::c_void);
            }
            // Clip the native surface, including its system shadow silhouette.
            unsafe { pp_round_menu(window.raw_handle()) };
        }
    }
    draw::set_draw_color(background());
    draw::draw_rectf(x, y, w, h);
}

#[derive(Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Quiet,
    Danger,
}

/// One renderer for all application actions, including message dialogs.
pub fn button(button: &mut Button, style: ButtonStyle) {
    button.set_frame(FrameType::RFlatBox);
    button.set_down_frame(FrameType::RFlatBox);
    button.set_label_font(Font::Helvetica);
    button.set_label_size(13);
    button.draw(move |b| {
        let enabled = b.active_r();
        let pressed = enabled && b.value();
        let primary = matches!(style, ButtonStyle::Primary);
        let fill = if !enabled {
            hover()
        } else if primary {
            if pressed { pressed_accent() } else { accent() }
        } else if pressed {
            hover()
        } else {
            background()
        };
        draw::set_draw_color(background());
        draw::draw_rectf(b.x(), b.y(), b.w(), b.h());
        draw::set_draw_color(fill);
        draw::draw_rounded_rectf(b.x(), b.y(), b.w(), b.h(), 7);
        if matches!(style, ButtonStyle::Secondary) {
            draw::set_draw_color(line());
            draw::draw_rounded_rect(b.x(), b.y(), b.w(), b.h(), 7);
        }
        draw::set_draw_color(if !enabled {
            disabled_text()
        } else if primary {
            Color::White
        } else if matches!(style, ButtonStyle::Danger) {
            danger()
        } else {
            muted()
        });
        draw::set_font(b.label_font(), b.label_size());
        draw::push_clip(b.x() + 4, b.y(), b.w() - 8, b.h());
        draw::draw_text2(&b.label(), b.x(), b.y(), b.w(), b.h(), Align::Center);
        draw::pop_clip();
        if enabled && b.has_focus() {
            draw::set_draw_color(if primary { Color::White } else { accent() });
            draw::draw_rounded_rect(b.x() + 2, b.y() + 2, b.w() - 4, b.h() - 4, 5);
        }
    });
}

fn flat(x: i32, y: i32, w: i32, h: i32, color: Color) {
    draw::set_draw_color(background());
    draw::draw_rectf(x, y, w, h);
    draw::set_draw_color(if app::draw_frame_active() {
        color
    } else {
        color.inactive()
    });
    draw::draw_rounded_rectf(x, y, w, h, 7);
}
fn field(x: i32, y: i32, w: i32, h: i32, color: Color) {
    flat(x, y, w, h, color);
    draw::set_draw_color(line());
    draw::draw_rounded_rect(x, y, w, h, 7);
}

fn radio(x: i32, y: i32, w: i32, h: i32, _color: Color) {
    draw::set_draw_color(background());
    draw::draw_pie(x, y, w, h, 0., 360.);
    draw::set_draw_color(muted());
    draw::draw_arc(x, y, w, h, 0., 360.);
}

pub fn init() {
    #[cfg(any(target_os = "linux", windows))]
    {
        // Xft selects one face, unlike Cocoa/GDI's glyph fallback. Use installed
        // CJK faces (also covering Latin) without scanning or bundling fonts.
        static FONTS: std::sync::Once = std::sync::Once::new();
        FONTS.call_once(|| {
            #[cfg(target_os = "linux")]
            let (regular, bold) = (
                " Noto Sans CJK SC,Noto Sans CJK TC,Noto Sans CJK JP,WenQuanYi Zen Hei,sans",
                "BNoto Sans CJK SC,Noto Sans CJK TC,Noto Sans CJK JP,WenQuanYi Zen Hei,sans",
            );
            #[cfg(windows)]
            let (regular, bold) = (" Microsoft YaHei UI", "BMicrosoft YaHei UI");
            fltk::enums::Font::set_font(fltk::enums::Font::Helvetica, regular);
            fltk::enums::Font::set_font(fltk::enums::Font::HelveticaBold, bold);
        });
    }
    fltk::dialog::message_title_default("pp-switch");
    apply_appearance(system::dark());
    static WATCH: std::sync::Once = std::sync::Once::new();
    WATCH.call_once(|| {
        #[cfg(target_os = "macos")]
        unsafe {
            unsafe extern "C" {
                fn pp_watch_appearance(callback: extern "C" fn(i32));
            }
            pp_watch_appearance(appearance_changed);
        }
        #[cfg(not(target_os = "macos"))]
        app::add_timeout3(1.0, |handle| {
            let dark = system::dark();
            if dark != is_dark() {
                apply_appearance(dark);
            }
            app::repeat_timeout3(1.0, handle);
        });
    });
    app::set_frame_type_cb(FrameType::RFlatBox, flat, 5, 4, 10, 8);
    app::set_frame_type_cb(FrameType::BorderBox, field, 7, 5, 14, 10);
    app::set_frame_type_cb(FrameType::RoundDownBox, radio, 2, 2, 4, 4);
    app::set_frame_type_cb(menu_frame(), menu_surface, 10, 10, 20, 20);
    app::set_menu_linespacing(15);
    app::set_visible_focus(true);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn luminance((r, g, b): Rgb) -> f64 {
        let linear = |v: u8| {
            let v = f64::from(v) / 255.;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    }
    fn contrast(a: Rgb, b: Rgb) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    #[test]
    fn readable_text_in_both_appearances() {
        for p in [LIGHT, DARK_PALETTE] {
            for (text, surface) in [
                (p[1], p[0]),
                (p[2], p[0]),
                (p[1], p[8]),
                (p[11], p[5]),
                ((255, 255, 255), p[3]),
            ] {
                assert!(
                    contrast(text, surface) >= 4.5,
                    "Insufficient contrast: {text:?} on {surface:?}"
                );
            }
        }
    }
}
