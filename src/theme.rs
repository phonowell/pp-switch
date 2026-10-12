//! Flat native surfaces; no shadows, cached images or background rendering.
use fltk::{
    app, draw,
    enums::{Color, FrameType},
};

pub fn background() -> Color {
    Color::from_rgb(247, 248, 250)
}
pub fn ink() -> Color {
    Color::from_rgb(40, 45, 56)
}
pub fn muted() -> Color {
    Color::from_rgb(97, 106, 121)
}
pub fn accent() -> Color {
    Color::from_rgb(54, 93, 216)
}
pub fn line() -> Color {
    Color::from_rgb(228, 231, 236)
}
pub fn selected() -> Color {
    Color::from_rgb(237, 241, 251)
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
    app::background(247, 248, 250);
    app::foreground(40, 45, 56);
    app::set_frame_type_cb(FrameType::RFlatBox, flat, 5, 4, 10, 8);
    app::set_frame_type_cb(FrameType::BorderBox, field, 7, 5, 14, 10);
    app::set_visible_focus(true);
}
