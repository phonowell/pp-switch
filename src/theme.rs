//! Native vector surfaces: a fixed number of drawing operations, no cached images.
use fltk::{
    app, draw,
    enums::{Color, FrameType},
};
use fltk::{enums::Align, menu::Choice, prelude::*};

/// FLTK Choice hardcodes its default box. Override drawing, retain native events/menu.
pub fn choice(choice: &mut Choice, elevated: bool) {
    choice.set_selection_color(accent());
    choice.draw(move |c| {
        draw::set_draw_color(background());
        draw::draw_rectf(c.x(), c.y(), c.w(), c.h());
        if elevated {
            raised(c.x(), c.y(), c.w(), c.h(), background());
        } else {
            inset(c.x(), c.y(), c.w(), c.h(), background());
        }
        draw::push_clip(c.x() + 10, c.y() + 5, c.w() - 38, c.h() - 10);
        draw::set_font(c.text_font(), c.text_size());
        draw::set_draw_color(if c.active_r() {
            ink()
        } else {
            ink().inactive()
        });
        draw::draw_text2(
            &c.text(c.value()).unwrap_or_default(),
            c.x() + 12,
            c.y(),
            c.w() - 42,
            c.h(),
            Align::Left | Align::Inside,
        );
        draw::pop_clip();
        let x = c.x() + c.w() - 19;
        let y = c.y() + c.h() / 2;
        draw::draw_line(x - 4, y - 2, x, y + 2);
        draw::draw_line(x, y + 2, x + 4, y - 2);
        if c.has_focus() {
            draw::set_draw_color(accent());
            draw::draw_rounded_rect(c.x() + 3, c.y() + 3, c.w() - 6, c.h() - 6, 8);
        }
    });
}

pub fn background() -> Color {
    Color::from_rgb(232, 237, 242)
}
pub fn ink() -> Color {
    Color::from_rgb(37, 51, 73)
}
pub fn muted() -> Color {
    Color::from_rgb(83, 99, 117)
}
pub fn accent() -> Color {
    Color::from_rgb(70, 91, 171)
}

fn fill(x: i32, y: i32, w: i32, h: i32, radius: i32, color: Color) {
    draw::set_draw_color(color);
    draw::draw_rounded_rectf(x, y, w, h, radius);
}

fn raised(x: i32, y: i32, w: i32, h: i32, color: Color) {
    let color = if app::draw_frame_active() {
        color
    } else {
        color.inactive()
    };
    let radius = 12.min(h / 3);
    // Nested offset contours approximate a soft shadow without a blur bitmap.
    for spread in (1..=6).rev() {
        let weight = (7 - spread) as f32 / 35.0;
        fill(
            x + spread,
            y + spread,
            w,
            h,
            radius,
            Color::color_average(Color::from_rgb(160, 176, 194), background(), weight),
        );
        fill(
            x - spread / 2,
            y - spread / 2,
            w,
            h,
            radius,
            Color::color_average(Color::White, background(), weight * 1.8),
        );
    }
    fill(x, y, w, h, radius, color);
    draw::set_draw_color(Color::color_average(Color::White, color, 0.35));
    draw::draw_rounded_rect(x, y, w, h, radius);
}

fn inset(x: i32, y: i32, w: i32, h: i32, color: Color) {
    let color = if app::draw_frame_active() {
        color
    } else {
        color.inactive()
    };
    let radius = 10.min(h / 3);
    fill(x, y, w, h, radius, Color::from_rgb(196, 207, 220));
    fill(x + 1, y + 1, w - 1, h - 1, radius, Color::White);
    fill(x + 2, y + 2, w - 4, h - 4, radius - 1, color);
    draw::set_draw_color(Color::color_average(
        Color::from_rgb(158, 174, 193),
        color,
        0.25,
    ));
    draw::draw_line(x + radius, y + 2, x + w - radius, y + 2);
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
    app::background(232, 237, 242);
    app::foreground(37, 51, 73);
    app::set_frame_type_cb(FrameType::RShadowBox, raised, 7, 7, 14, 14);
    app::set_frame_type_cb(FrameType::RFlatBox, inset, 7, 6, 14, 12);
    app::set_frame_type_cb(FrameType::RoundDownBox, inset, 7, 6, 14, 12);
    app::set_visible_focus(true);
}
