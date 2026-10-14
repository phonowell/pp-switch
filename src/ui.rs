use crate::{
    Result,
    client::{ModelState, Paths, environment_conflict},
    i18n::{self, Language, tr},
    protocol,
    provider::{Client, Models, Provider, import_link},
    storage::Store,
    theme, update,
};
use fltk::{
    app,
    button::Button,
    draw,
    enums::{Align, Event, Font, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Group, Scroll, ScrollType},
    input::{Input, SecretInput},
    menu::Choice,
    prelude::*,
    window::Window,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub mod acceptance;
mod client_icons;
mod lucide;
mod update_job;
use lucide::Icon;

#[derive(Clone)]
struct Row {
    id: String,
    surface: Frame,
    edit: Button,
    switch: Button,
    active: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct OfficialRow {
    surface: Frame,
    switch: Button,
    active: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct Controls {
    window: Window,
    clients: [Button; 5],
    client: Rc<Cell<Client>>,
    language: Rc<Cell<Language>>,
    main_page: Group,
    settings_page: Group,
    settings: Button,
    back: Button,
    languages: Choice,
    update: Button,
    register: Button,
    labels: Vec<(Frame, &'static str)>,
    raw_status: Rc<RefCell<String>>,
    updating: Rc<Cell<bool>>,
    list: Scroll,
    rows: Rc<RefCell<Vec<Row>>>,
    official: Rc<RefCell<Option<OfficialRow>>>,
    empty: Frame,
    editor: Window,
    name: Input,
    url: Input,
    key: SecretInput,
    model: Input,
    model_aliases: [Input; 3],
    model_group: Group,
    model_toggle: Button,
    model_hint: Frame,
    models_expanded: Rc<Cell<bool>>,
    new: Button,
    save: Button,
    cancel: Button,
    delete: Button,
    status: Frame,
}

#[derive(Default)]
struct State {
    providers: Vec<Provider>,
    editing: Option<String>,
    current: Option<(String, String)>,
    current_models: Option<ModelState>,
    setup_error: Option<String>,
    official: bool,
}

impl Controls {
    fn lang(&self) -> Language {
        self.language.get()
    }
    fn message(&mut self, english: &str) {
        let message = i18n::message(self.lang(), english);
        self.status.set_label(&message);
        *self.raw_status.borrow_mut() = english.to_owned();
    }
    fn translate(&mut self) {
        let lang = self.lang();
        for (label, key) in &mut self.labels {
            label.set_label(tr(lang, key));
        }
        for (button, key) in [
            (&mut self.new, "Add"),
            (&mut self.save, "Save"),
            (&mut self.cancel, "Cancel"),
            (&mut self.delete, "Delete"),
            (&mut self.back, "Back"),
            (&mut self.register, "Register import links"),
        ] {
            button.set_label(tr(lang, key));
        }
        self.editor.set_label(tr(lang, "Edit provider"));
        self.update.set_label(tr(
            lang,
            if self.updating.get() {
                "Updating…"
            } else {
                "Check for updates"
            },
        ));
        self.languages.set_value(lang.index() as i32);
        self.empty.set_label(tr(lang, "Nothing here yet."));
        self.model_layout(self.models_expanded.get());
        let message = self.raw_status.borrow().clone();
        self.message(&message);
        self.window.redraw();
        self.editor.redraw();
    }
    fn hide_dialogs(&mut self) {
        self.editor.hide();
    }
    fn model_layout(&mut self, expanded: bool) {
        let claude = self.client.get() == Client::Claude;
        self.models_expanded.set(expanded && claude);
        self.model_hint.set_label(tr(
            self.lang(),
            if claude {
                "Blank keeps models. Mappings follow a supplied default."
            } else {
                "Blank keeps the client's model."
            },
        ));
        let y = if claude {
            self.model_toggle.show();
            self.model_toggle.set_label(tr(
                self.lang(),
                if expanded {
                    "Hide model mappings"
                } else {
                    "Claude model mappings"
                },
            ));
            if expanded {
                self.model_group.show();
                518
            } else {
                self.model_group.hide();
                432
            }
        } else {
            self.model_toggle.hide();
            self.model_group.hide();
            394
        };
        for b in [&mut self.delete, &mut self.cancel, &mut self.save] {
            b.set_pos(b.x(), y);
        }
        self.editor.set_size(386, y + 60);
        self.position_editor();
        self.editor.redraw();
    }
    fn position_editor(&mut self) {
        let (x, y, w, h) = app::screen_work_area(app::screen_num(self.window.x(), self.window.y()));
        if w > 0 && h > 0 {
            self.editor.set_pos(
                (self.window.x() + 27).clamp(x, (x + w - self.editor.w()).max(x)),
                (self.window.y() + 53).clamp(y + 28, (y + h - self.editor.h() - 8).max(y + 28)),
            );
        }
    }
    fn model_fields(&mut self, models: &Models) {
        self.model.set_value(models.model.as_deref().unwrap_or(""));
        for (input, value) in
            self.model_aliases
                .iter_mut()
                .zip([&models.haiku, &models.sonnet, &models.opus])
        {
            input.set_value(value.as_deref().unwrap_or(""));
        }
        self.model_layout(
            [&models.haiku, &models.sonnet, &models.opus]
                .iter()
                .any(|v| v.is_some()),
        );
    }
}

fn label(x: i32, y: i32, w: i32, text: &'static str) -> (Frame, &'static str) {
    let mut f = Frame::new(x, y, w, 20, text);
    f.set_align(Align::Left | Align::Inside);
    f.set_label_size(12);
    f.set_label_color(theme::muted());
    (f, text)
}
fn field<T: InputExt + WidgetExt>(input: &mut T) {
    input.set_frame(FrameType::BorderBox);
    input.set_color(theme::field_fill());
    input.set_text_color(theme::ink());
    input.set_selection_color(theme::accent());
    input.set_text_size(13);
}
fn button(button: &mut Button, primary: bool) {
    theme::button(
        button,
        if primary {
            theme::ButtonStyle::Primary
        } else {
            theme::ButtonStyle::Secondary
        },
    );
}
fn icon_button(button: &mut Button, icon: Icon) {
    button.set_frame(FrameType::NoBox);
    button.draw(move |b| {
        draw::set_draw_color(theme::background());
        draw::draw_rectf(b.x(), b.y(), b.w(), b.h());
        if b.value() {
            draw::set_draw_color(theme::hover());
            draw::draw_rounded_rectf(b.x(), b.y(), b.w(), b.h(), 7);
        }
        draw::set_draw_color(if b.active() {
            theme::muted()
        } else {
            theme::disabled_text()
        });
        lucide::draw(icon, b.x() + (b.w() - 18) / 2, b.y() + (b.h() - 18) / 2, 18);
        if b.has_focus() {
            draw::set_draw_color(theme::accent());
            draw::draw_rounded_rect(b.x(), b.y(), b.w(), b.h(), 5);
        }
    });
}

fn text_icon_button(b: &mut Button, icon: Icon) {
    b.set_frame(FrameType::NoBox);
    b.draw(move |b| {
        draw::set_draw_color(theme::background());
        draw::draw_rectf(b.x(), b.y(), b.w(), b.h());
        if b.value() {
            draw::set_draw_color(theme::hover());
            draw::draw_rounded_rectf(b.x(), b.y(), b.w(), b.h(), 7);
        }
        draw::set_font(Font::Helvetica, 12);
        let text = b.label();
        let text = text.trim();
        let width = draw::measure(text, false).0;
        let left = b.x() + (b.w() - width - 24) / 2;
        draw::set_draw_color(theme::muted());
        lucide::draw(icon, left, b.y() + (b.h() - 16) / 2, 16);
        draw::draw_text2(
            text,
            left + 24,
            b.y(),
            width,
            b.h(),
            Align::Left | Align::Inside,
        );
        if b.has_focus() {
            draw::set_draw_color(theme::accent());
            draw::draw_rounded_rect(b.x(), b.y(), b.w(), b.h(), 5);
        }
    });
}

fn controls(language: Language) -> Controls {
    let _app = app::App::default().with_scheme(app::Scheme::Base);
    app::set_font(Font::Helvetica);
    theme::init();
    fltk::misc::Tooltip::disable();
    let mut window = Window::new(100, 100, 440, 502, "Pure Provider Switch");
    window.set_color(theme::background());
    // Keep initial native activation on the window, rather than letting
    // Fl_Group focus the first interactive child. User input restores
    // normal mouse and keyboard focus, including Tab navigation.
    let mut interacted = false;
    // The default wrapper calls Fl_Group first; intercept Focus before it
    // selects a child, rather than merely changing the returned event result.
    window.super_handle_first(false);
    window.handle(move |w, event| match event {
        Event::Focus => !interacted,
        Event::KeyDown | Event::Shortcut if !interacted && app::event_key() == Key::Tab => {
            interacted = true;
            // Group navigation expects focus in a child already. Start at
            // the first/last eligible child for the first Tab/Shift-Tab.
            let reverse = app::event_state().contains(Shortcut::Shift);
            (0..w.children()).any(|index| {
                let index = if reverse {
                    w.children() - 1 - index
                } else {
                    index
                };
                w.child(index)
                    .is_some_and(|mut child| child.take_focus().is_ok())
            })
        }
        Event::Push | Event::KeyDown | Event::Shortcut => {
            interacted = true;
            false
        }
        _ => false,
    });
    #[cfg(not(target_os = "macos"))]
    {
        let icon = fltk::image::RgbImage::new(
            include_bytes!("../assets/icon-64.rgba"),
            64,
            64,
            fltk::enums::ColorDepth::Rgba8,
        )
        .expect("compiled application icon");
        window.set_icon(Some(icon));
    }
    let main_page = Group::new(0, 0, 440, 502, "");
    let mut settings = Button::new(384, 27, 32, 32, "");
    icon_button(&mut settings, Icon::Settings);
    let client = Rc::new(Cell::new(Client::Claude));
    let mut track = Frame::new(24, 24, 348, 38, "");
    track.draw(|f| {
        draw::set_draw_color(theme::hover());
        draw::draw_rounded_rectf(f.x(), f.y(), f.w(), f.h(), 7);
    });
    let clients = Client::ALL.map(|kind| {
        let index = Client::ALL
            .iter()
            .position(|client| *client == kind)
            .unwrap();
        let left = 24 + index as i32 * 348 / 5;
        let right = 24 + (index as i32 + 1) * 348 / 5;
        // Only our clipped short caption is drawn. A native full-name label can
        // be painted separately by the toolkit and overflow the compact tab.
        let mut b = Button::new(left, 24, right - left, 38, "");
        button(&mut b, false);
        b.set_frame(FrameType::NoBox);
        b.set_down_frame(FrameType::NoBox);
        b.set_label_size(11);
        let client = client.clone();
        b.draw(move |b| {
            draw::push_clip(b.x(), b.y(), b.w(), b.h());
            draw::set_draw_color(theme::background());
            draw::draw_rectf(b.x(), b.y(), b.w(), b.h());
            draw::set_draw_color(theme::hover());
            draw::draw_rounded_rectf(24, 24, 348, 38, 7);
            let selected = client.get() == Client::ALL[index];
            if selected || b.value() {
                draw::set_draw_color(if b.value() {
                    theme::selected()
                } else {
                    theme::background()
                });
                draw::draw_rounded_rectf(b.x() + 3, b.y() + 3, b.w() - 6, b.h() - 6, 5);
            }
            draw::set_font(Font::Helvetica, 11);
            draw::set_draw_color(if selected {
                theme::ink()
            } else {
                theme::muted()
            });
            let text = ["Claude", "Codex", "Gemini", "Grok", "DSH"][index];
            let width = draw::measure(text, false).0;
            let left = b.x() + (b.w() - width - 19) / 2;
            client_icons::draw(kind, left, b.y() + (b.h() - 14) / 2, 14);
            draw::draw_text2(
                text,
                left + 19,
                b.y(),
                width,
                b.h(),
                Align::Left | Align::Inside,
            );
            if b.has_focus() {
                draw::set_draw_color(theme::accent());
                draw::draw_rounded_rect(b.x() + 3, b.y() + 3, b.w() - 6, b.h() - 6, 5);
            }
            draw::pop_clip();
        });
        b
    });
    let mut labels = vec![label(24, 89, 180, "Providers")];
    let mut new = Button::new(328, 84, 88, 28, "Add");
    theme::button(&mut new, theme::ButtonStyle::Quiet);
    new.set_label_size(12);
    text_icon_button(&mut new, Icon::Add);
    let mut list = Scroll::new(24, 124, 392, 354, "");
    list.set_frame(FrameType::FlatBox);
    list.set_color(theme::background());
    list.set_type(ScrollType::Vertical);
    list.set_scrollbar_size(10);
    let mut scrollbar = list.scrollbar();
    scrollbar.set_frame(FrameType::FlatBox);
    scrollbar.set_color(theme::background());
    scrollbar.set_selection_color(theme::hover());
    scrollbar.set_slider_frame(FrameType::RFlatBox);
    list.end();
    let mut empty = Frame::new(24, 274, 392, 28, "Nothing here yet.");
    empty.set_label_color(theme::muted());
    empty.set_label_size(13);
    main_page.end();
    let mut settings_page = Group::new(0, 0, 440, 502, "");
    let mut back = Button::new(24, 24, 88, 32, "Back");
    theme::button(&mut back, theme::ButtonStyle::Quiet);
    text_icon_button(&mut back, Icon::Back);
    labels.push(label(24, 94, 180, "Language"));
    let mut languages = Choice::new(196, 84, 220, 36, "");
    languages.add_choice("English|简体中文|繁體中文|日本語");
    languages.set_frame(FrameType::FlatBox);
    languages.set_menu_frame(theme::menu_frame());
    languages.set_down_frame(FrameType::RFlatBox);
    languages.set_color(theme::background());
    languages.set_text_color(theme::ink());
    languages.set_text_size(13);
    languages.set_selection_color(theme::selected());
    languages.draw(|choice| {
        draw::set_draw_color(theme::background());
        draw::draw_rectf(choice.x(), choice.y(), choice.w(), choice.h());
        draw::set_draw_color(theme::field_fill());
        draw::draw_rounded_rectf(choice.x(), choice.y(), choice.w(), choice.h(), 7);
        draw::set_draw_color(theme::ink());
        draw::set_font(Font::Helvetica, 13);
        let value = Language::ALL
            .get(choice.value() as usize)
            .copied()
            .unwrap_or(Language::En);
        let text = ["English", "简体中文", "繁體中文", "日本語"][value.index()];
        draw::draw_text2(
            text,
            choice.x() + 12,
            choice.y(),
            choice.w() - 44,
            choice.h(),
            Align::Left | Align::Inside,
        );
        draw::set_draw_color(theme::muted());
        lucide::draw(
            Icon::Down,
            choice.x() + choice.w() - 28,
            choice.y() + (choice.h() - 16) / 2,
            16,
        );
        if choice.has_focus() {
            draw::set_draw_color(theme::accent());
            draw::draw_rounded_rect(choice.x(), choice.y(), choice.w(), choice.h(), 7);
        }
    });
    let mut update = Button::new(24, 156, 392, 36, "Check for updates");
    button(&mut update, false);
    let mut register = Button::new(24, 208, 392, 36, "Register import links");
    button(&mut register, false);
    let mut status = Frame::new(24, 260, 392, 108, "");
    status.set_label_color(theme::muted());
    status.set_label_size(12);
    status.set_align(Align::Left | Align::Inside | Align::Wrap);
    settings_page.end();
    settings_page.hide();
    window.end();

    let mut editor = Window::new(130, 150, 386, 396, "Edit provider");
    editor.make_modal(true);
    editor.set_color(theme::background());
    labels.push(label(24, 24, 338, "Provider name"));
    let mut name = Input::new(24, 49, 338, 40, "");
    field(&mut name);
    labels.push(label(24, 106, 338, "Base URL"));
    let mut url = Input::new(24, 131, 338, 40, "");
    field(&mut url);
    labels.push(label(24, 188, 338, "API key"));
    let mut key = SecretInput::new(24, 213, 338, 40, "");
    field(&mut key);
    labels.push(label(24, 270, 338, "Default model (optional)"));
    let mut model = Input::new(24, 295, 338, 40, "");
    field(&mut model);
    let mut model_hint = Frame::new(24, 346, 338, 28, "");
    model_hint.set_label_size(11);
    model_hint.set_label_color(theme::muted());
    model_hint.set_align(Align::Left | Align::Inside | Align::Wrap);
    let mut model_toggle = Button::new(24, 382, 338, 28, "");
    button(&mut model_toggle, false);
    let mut model_group = Group::new(24, 422, 338, 69, "");
    let model_aliases = ["Haiku (optional)", "Sonnet (optional)", "Opus (optional)"].map(|name| {
        let index = ["Haiku (optional)", "Sonnet (optional)", "Opus (optional)"]
            .iter()
            .position(|s| *s == name)
            .unwrap();
        let x = 24 + index as i32 * 116;
        labels.push(label(x, 422, 106, name));
        let mut input = Input::new(x, 445, 106, 40, "");
        field(&mut input);
        input
    });
    model_group.end();
    model_group.hide();
    let mut delete = Button::new(24, 328, 66, 36, "Delete");
    theme::button(&mut delete, theme::ButtonStyle::Danger);
    let mut cancel = Button::new(168, 328, 88, 36, "Cancel");
    button(&mut cancel, false);
    let mut save = Button::new(268, 328, 94, 36, "Save");
    button(&mut save, true);
    save.set_shortcut(Shortcut::Ctrl | Key::Enter);
    editor.end();
    let mut c = Controls {
        window,
        clients,
        client,
        language: Rc::new(Cell::new(language)),
        main_page,
        settings_page,
        settings,
        back,
        languages,
        update,
        register,
        labels,
        raw_status: Rc::new(RefCell::new(String::new())),
        updating: Rc::new(Cell::new(false)),
        list,
        rows: Rc::new(RefCell::new(vec![])),
        official: Rc::new(RefCell::new(None)),
        empty,
        editor,
        name,
        url,
        key,
        model,
        model_aliases,
        model_group,
        model_toggle,
        model_hint,
        models_expanded: Rc::new(Cell::new(false)),
        new,
        save,
        cancel,
        delete,
        status,
    };
    c.translate();
    c
}

fn matching(state: &State, p: &Provider) -> bool {
    state
        .current
        .as_ref()
        .is_some_and(|(url, key)| *url == p.base_url && *key == p.api_key)
        && (p.models.is_empty()
            || state
                .current_models
                .as_ref()
                .is_some_and(|m| m.matches(p.client, &p.models)))
}

fn row_status(c: &mut Controls, state: &State) {
    if let Some(row) = c.official.borrow_mut().as_mut() {
        row.active.set(state.official);
        if state.official {
            row.switch.deactivate();
        } else {
            row.switch.activate();
        }
        row.surface.redraw();
        row.switch.redraw();
    }
    for row in c.rows.borrow_mut().iter_mut() {
        row.active.set(
            state
                .providers
                .iter()
                .find(|p| p.id == row.id)
                .is_some_and(|p| matching(state, p)),
        );
        if row.active.get() {
            row.switch.deactivate();
        } else {
            row.switch.activate();
        }
        row.surface.redraw();
        row.edit.redraw();
        row.switch.redraw();
    }
    c.list.redraw();
}
fn close_editor(c: &mut Controls, state: &mut State) {
    c.editor.hide();
    state.editing = None;
    c.name.set_value("");
    c.url.set_value("");
    c.key.set_value("");
    c.model_fields(&Models::default());
}
fn open_editor(c: &mut Controls, state: &mut State, p: Option<&Provider>, paths: &Paths) {
    state.editing = p.map(|p| p.id.clone());
    c.name.set_value(p.map_or("", |p| &p.name));
    c.url.set_value(p.map_or("", |p| &p.base_url));
    c.key.set_value(p.map_or("", |p| &p.api_key));
    let models = p.map_or_else(
        || {
            let mut models = Models::default();
            if c.client.get() == Client::Claude {
                models.model = Some(crate::client::CLAUDE_EDITOR_DEFAULT.into());
            }
            models
        },
        |p| paths.editor_models(p),
    );
    c.model_fields(&models);
    c.model_layout(c.models_expanded.get());
    if p.is_some() {
        c.delete.show();
    } else {
        c.delete.hide();
    }
    c.editor.set_label(tr(
        c.lang(),
        if p.is_some() {
            "Edit provider"
        } else {
            "Add provider"
        },
    ));
    c.position_editor();
    c.editor.show();
    let _ = c.name.take_focus();
}
fn read_current(state: &mut State, paths: &Paths, client: Client) {
    let status = paths.status(
        client,
        state
            .providers
            .iter()
            .any(|p| p.client == client && !p.models.is_empty()),
    );
    state.setup_error = environment_conflict(client)
        .err()
        .or_else(|| status.as_ref().err().cloned());
    let status = status.unwrap_or_default();
    state.official = status.official && state.setup_error.is_none();
    state.current = status.connection;
    state.current_models = status.models;
}

fn provider_surface(
    y: i32,
    width: i32,
    language: Rc<Cell<Language>>,
    name: String,
    url: String,
    active: Rc<Cell<bool>>,
    official: bool,
) -> Frame {
    let mut surface = Frame::new(24, y, width, 69, "");
    surface.set_frame(FrameType::NoBox);
    let active_draw = active;
    surface.draw(move |b| {
        draw::set_draw_color(theme::background());
        draw::draw_rectf(b.x(), b.y(), b.w(), b.h());
        draw::set_font(Font::Helvetica, 10);
        let badge = tr(language.get(), "In use");
        let badge_width = draw::measure(badge, false).0 + 6;
        // Reserve both icon hit targets and an 8px gap before the badge.
        let actions = if official { 39 } else { 75 };
        let badge_left = b.w() - actions - 8 - badge_width;
        let text_width = if active_draw.get() {
            badge_left - 12 - 16
        } else {
            b.w() - actions - 28
        };
        draw::push_clip(b.x() + 16, b.y() + 12, text_width, 46);
        draw::set_draw_color(if active_draw.get() {
            theme::active_text()
        } else {
            theme::ink()
        });
        draw::set_font(Font::HelveticaBold, 14);
        draw::draw_text2(
            if official {
                tr(language.get(), "Official")
            } else {
                &name
            },
            b.x() + 16,
            b.y() + 12,
            text_width,
            23,
            Align::Left | Align::Inside,
        );
        draw::set_draw_color(if active_draw.get() {
            theme::active_text()
        } else {
            theme::muted()
        });
        draw::set_font(Font::Helvetica, 11);
        draw::draw_text2(
            if official {
                tr(language.get(), "Native login")
            } else {
                &url
            },
            b.x() + 16,
            b.y() + 37,
            text_width,
            18,
            Align::Left | Align::Inside,
        );
        draw::pop_clip();
        if active_draw.get() {
            draw::set_font(Font::Helvetica, 10);
            draw::set_draw_color(theme::active_text());
            draw::draw_text2(
                badge,
                b.x() + badge_left,
                b.y(),
                badge_width,
                b.h(),
                Align::Right | Align::Inside,
            );
        }
    });
    surface
}

fn refresh(
    c: &mut Controls,
    state: Rc<RefCell<State>>,
    store: &Store,
    paths: &Paths,
) -> Result<()> {
    {
        let mut s = state.borrow_mut();
        s.providers = store.load()?;
        read_current(&mut s, paths, c.client.get());
    }
    c.rows.borrow_mut().clear();
    c.official.borrow_mut().take();
    c.list.clear();
    c.list.scroll_to(0, 0);
    c.list.begin();
    let current_state = state.borrow();
    let providers: Vec<_> = current_state
        .providers
        .iter()
        .filter(|p| p.client == c.client.get())
        .collect();
    let row_width = if (providers.len() + 1) * 74 > c.list.h() as usize {
        380
    } else {
        392
    };
    let official_active = Rc::new(Cell::new(current_state.official));
    let official_surface = provider_surface(
        124,
        row_width,
        c.language.clone(),
        String::new(),
        String::new(),
        official_active.clone(),
        true,
    );
    let mut restore = Button::new(24 + row_width - 39, 143, 30, 30, "");
    icon_button(&mut restore, Icon::Switch);
    let mut view = c.clone();
    let restore_state = state.clone();
    let restore_store = store.clone();
    let restore_paths = paths.clone();
    restore.set_callback(move |_| {
        if let Err(error) = restore_paths.apply_official(&restore_store, view.client.get()) {
            i18n::alert(view.lang(), &error);
            return;
        }
        let mut s = restore_state.borrow_mut();
        read_current(&mut s, &restore_paths, view.client.get());
        row_status(&mut view, &s);
    });
    *c.official.borrow_mut() = Some(OfficialRow {
        surface: official_surface,
        switch: restore,
        active: official_active,
    });
    for (index, p) in providers.iter().enumerate() {
        let active = Rc::new(Cell::new(matching(&state.borrow(), p)));
        let surface = provider_surface(
            198 + index as i32 * 74,
            row_width,
            c.language.clone(),
            p.name.clone(),
            p.base_url.clone(),
            active.clone(),
            false,
        );
        let y = 198 + index as i32 * 74 + 19;
        let mut edit = Button::new(24 + row_width - 75, y, 30, 30, "");
        icon_button(&mut edit, Icon::Edit);
        let mut view = c.clone();
        let edit_state = state.clone();
        let edit_paths = paths.clone();
        let id = p.id.clone();
        edit.set_callback(move |_| {
            let provider = edit_state
                .borrow()
                .providers
                .iter()
                .find(|p| p.id == id)
                .cloned();
            if let Some(provider) = provider {
                open_editor(
                    &mut view,
                    &mut edit_state.borrow_mut(),
                    Some(&provider),
                    &edit_paths,
                );
            }
        });
        let mut switch = Button::new(24 + row_width - 39, y, 30, 30, "");
        icon_button(&mut switch, Icon::Switch);
        let mut view = c.clone();
        let switch_state = state.clone();
        let switch_store = store.clone();
        let switch_paths = paths.clone();
        let id = p.id.clone();
        switch.set_callback(move |_| {
            let result = (|| -> Result<()> {
                let provider = switch_state
                    .borrow()
                    .providers
                    .iter()
                    .find(|p| p.id == id)
                    .cloned()
                    .ok_or("Select a saved provider first.")?;
                environment_conflict(provider.client)?;
                switch_paths.apply(&switch_store, &provider)?;
                // Applying changes status, not list structure. Keep widgets,
                // scroll and keyboard focus rather than rebuilding the list.
                let mut state = switch_state.borrow_mut();
                read_current(&mut state, &switch_paths, view.client.get());
                row_status(&mut view, &state);
                Ok(())
            })();
            if let Err(error) = result {
                i18n::alert(view.lang(), &error);
            }
        });
        c.rows.borrow_mut().push(Row {
            id: p.id.clone(),
            surface,
            edit,
            switch,
            active,
        });
    }
    c.list.end();
    c.empty.hide();
    row_status(c, &state.borrow());
    Ok(())
}

fn preview_link(
    c: &mut Controls,
    state: Rc<RefCell<State>>,
    store: &Store,
    paths: &Paths,
    link: &str,
) -> Result<()> {
    if c.editor.shown() {
        return Err("Finish or cancel your edits, then open the import link again.".into());
    }
    let mut p = import_link(link)?;
    c.client.set(p.client);
    refresh(c, state.clone(), store, paths)?;
    p.import_name(&state.borrow().providers);
    c.settings_page.hide();
    c.main_page.show();
    c.window.show();
    for b in &mut c.clients {
        b.redraw();
    }
    open_editor(c, &mut state.borrow_mut(), None, paths);
    c.name.set_value(&p.name);
    c.url.set_value(&p.base_url);
    c.key.set_value(&p.api_key);
    c.model_fields(&paths.editor_models(&p));
    c.model_layout(c.models_expanded.get());
    Ok(())
}

fn wire(c: &mut Controls, state: Rc<RefCell<State>>, store: Store, paths: Paths) {
    wire_with_updates(c, state, store, paths, update::check);
}

fn wire_with_updates(
    c: &mut Controls,
    state: Rc<RefCell<State>>,
    store: Store,
    paths: Paths,
    check_update: fn() -> Result<Option<update::Update>>,
) {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn pp_listen_quit(callback: extern "C" fn());
        }
        extern "C" fn native_quit() {
            quit();
        }
        unsafe { pp_listen_quit(native_quit) };
    }
    let mut view = c.clone();
    c.window.set_callback(move |w| {
        view.hide_dialogs();
        w.hide();
        quit();
    });
    let mut view = c.clone();
    let editor_state = state.clone();
    c.editor
        .set_callback(move |_| close_editor(&mut view, &mut editor_state.borrow_mut()));
    let mut view = c.clone();
    let editor_state = state.clone();
    c.cancel
        .set_callback(move |_| close_editor(&mut view, &mut editor_state.borrow_mut()));
    let mut view = c.clone();
    let new_state = state.clone();
    let new_paths = paths.clone();
    c.new.set_callback(move |_| {
        open_editor(&mut view, &mut new_state.borrow_mut(), None, &new_paths)
    });
    let mut view = c.clone();
    c.model_toggle.set_callback(move |_| {
        let expanded = !view.models_expanded.get();
        view.model_layout(expanded);
        let _ = view.model_toggle.take_focus();
    });
    for (i, mut b) in c.clients.clone().into_iter().enumerate() {
        let mut view = c.clone();
        let state = state.clone();
        let store = store.clone();
        let paths = paths.clone();
        b.set_callback(move |_| {
            view.client.set(Client::ALL[i]);
            if let Err(e) = refresh(&mut view, state.clone(), &store, &paths) {
                i18n::alert(view.lang(), &e);
            }
            for button in &mut view.clients {
                button.redraw();
            }
        });
    }
    let mut view = c.clone();
    c.settings.set_callback(move |_| {
        view.main_page.hide();
        view.settings_page.show();
        let _ = view.back.take_focus();
        view.window.redraw();
    });
    let mut view = c.clone();
    c.back.set_callback(move |_| {
        view.settings_page.hide();
        view.main_page.show();
        let _ = view.settings.take_focus();
        view.window.redraw();
    });
    let mut view = c.clone();
    let language_state = state.clone();
    let preferences = store.clone();
    c.languages.set_callback(move |choice| {
        let Some(language) = Language::ALL.get(choice.value() as usize).copied() else {
            return;
        };
        if let Err(error) = preferences.save_language(language) {
            view.translate();
            i18n::alert(view.lang(), &error);
            return;
        }
        view.language.set(language);
        view.translate();
        view.editor.set_label(tr(
            language,
            if language_state.borrow().editing.is_some() {
                "Edit provider"
            } else {
                "Add provider"
            },
        ));
    });
    let mut view = c.clone();
    c.update
        .set_callback(move |_| check_updates(&mut view, check_update));
    let mut view = c.clone();
    c.register
        .set_callback(move |_| match protocol::register() {
            Ok(()) => view.message("Import links registered. Existing handlers were kept."),
            Err(error) => view.message(&error),
        });
    let mut view = c.clone();
    let state_save = state.clone();
    let store_save = store.clone();
    let paths_save = paths.clone();
    c.save.set_callback(move |_| {
        let result = (|| -> Result<()> {
            let mut p = Provider::new(
                view.client.get(),
                &view.name.value(),
                &view.url.value(),
                &view.key.value(),
            )?;
            p.models = Models::from_fields(
                &view.model.value(),
                &view.model_aliases[0].value(),
                &view.model_aliases[1].value(),
                &view.model_aliases[2].value(),
            );
            p.validate()?;
            if let Some(id) = &state_save.borrow().editing {
                p.id.clone_from(id);
            }
            store_save.save(p.clone())?;
            close_editor(&mut view, &mut state_save.borrow_mut());
            refresh(&mut view, state_save.clone(), &store_save, &paths_save)
        })();
        if let Err(e) = result {
            i18n::alert(view.lang(), &e);
        }
    });
    let mut view = c.clone();
    let state_delete = state;
    let store_delete = store;
    let paths_delete = paths;
    c.delete.set_callback(move |_| {
        if !i18n::confirm(
            view.lang(),
            tr(
                view.lang(),
                "Delete this saved provider? Client configuration will stay as it is.",
            ),
            tr(view.lang(), "Cancel"),
            Some(tr(view.lang(), "Delete")),
        ) {
            return;
        }
        let result = (|| -> Result<()> {
            let id = state_delete
                .borrow()
                .editing
                .clone()
                .ok_or("Select a saved provider first.")?;
            store_delete.delete(&id)?;
            close_editor(&mut view, &mut state_delete.borrow_mut());
            refresh(
                &mut view,
                state_delete.clone(),
                &store_delete,
                &paths_delete,
            )
        })();
        if let Err(e) = result {
            i18n::alert(view.lang(), &e);
        }
    });
}

fn quit() {
    app::program_should_quit(true);
    app::quit();
}

fn event_loop() {
    while !app::should_program_quit() && app::wait() {}
}

const UPDATE_POLL_INTERVAL: f64 = 0.5;

fn confirm_update(language: Language, release: &update::Update) -> bool {
    let prompt = tr(
        language,
        "Install version {version} and restart? Unsaved edits will be lost.",
    )
    .replace("{version}", &release.version.to_string());
    i18n::confirm_with_link(
        language,
        &prompt,
        tr(language, "Cancel"),
        Some(tr(language, "Install and restart")),
        Some(&release.release_notes_url()),
    )
}

fn check_updates(c: &mut Controls, check: fn() -> Result<Option<update::Update>>) {
    check_updates_with(
        c,
        check,
        update::stage,
        update::launch_helper,
        confirm_update,
        quit,
    );
}

fn check_updates_with(
    c: &mut Controls,
    check: fn() -> Result<Option<update::Update>>,
    stage: fn(&update::Update, Language) -> Result<update::Staged>,
    launch: fn(update::Staged) -> Result<()>,
    confirm: fn(Language, &update::Update) -> bool,
    exit: fn(),
) {
    use update_job::{Job, Progress};
    let mut view = c.clone();
    if view.updating.replace(true) {
        return;
    }
    view.update.deactivate();
    view.update.set_label(tr(view.lang(), "Updating…"));
    // message() translates and retains the English key for live language changes.
    view.message("Checking for updates…");
    let mut job = Job::start(check, stage, launch);
    app::add_timeout3(UPDATE_POLL_INTERVAL, move |handle| {
        let mut progress = job.poll(std::time::Instant::now(), view.lang());
        if let Progress::Confirm(release) = progress {
            let accepted = confirm(view.lang(), &release);
            progress = job.confirm(release, accepted, view.lang(), std::time::Instant::now());
            if accepted {
                view.message("Downloading and verifying update…");
            }
        }
        match progress {
            Progress::Finished(result) => {
                view.message(result.as_deref().unwrap_or_else(|error| error));
                view.updating.set(false);
                view.update.activate();
                view.update.set_label(tr(view.lang(), "Check for updates"));
                app::remove_timeout3(handle);
            }
            Progress::Installed => {
                view.message("Installing update…");
                app::remove_timeout3(handle);
                // Bound Cocoa's native wait after the timer hides the final window.
                app::add_timeout3(0.0, |_| {});
                exit();
            }
            Progress::Waiting => app::repeat_timeout3(UPDATE_POLL_INTERVAL, handle),
            Progress::Confirm(_) => unreachable!(),
        }
    });
}

pub fn run(link: Option<&str>) -> Result<()> {
    run_with_link(Store::system()?, Paths::system()?, link)
}

/// Production UI with isolated paths available to development tools.
pub fn run_with(store: Store, paths: Paths) -> Result<()> {
    run_with_link(store, paths, None)
}

pub fn run_with_link(store: Store, paths: Paths, link: Option<&str>) -> Result<()> {
    #[cfg(target_os = "macos")]
    protocol::prepare();
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(store.language()?);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    #[cfg(target_os = "macos")]
    {
        let mut view = c.clone();
        let state = state.clone();
        let store = store.clone();
        let paths = paths.clone();
        protocol::listen(move |link| {
            if let Err(error) = preview_link(&mut view, state.clone(), &store, &paths, link) {
                i18n::alert(view.lang(), &error);
            }
        });
    }
    if let Some(link) = link
        && let Err(error) = preview_link(&mut c, state, &store, &paths, link)
    {
        i18n::alert(c.lang(), &error);
    }
    event_loop();
    Ok(())
}

/// Cross-platform callback smoke; native input acceptance is separate.
pub fn smoke(store: Store, paths: Paths, screenshot: Option<std::path::PathBuf>) -> Result<()> {
    acceptance::callback_smoke(store, paths, screenshot)
}

/// Isolated development measurement of the actual editor, with synthetic data.
pub fn run_measure_editor(store: Store, paths: Paths) -> Result<()> {
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(store.language()?);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store, paths.clone());
    c.window.show();
    let provider = state.borrow().providers.first().cloned();
    open_editor(&mut c, &mut state.borrow_mut(), provider.as_ref(), &paths);
    event_loop();
    Ok(())
}

/// Dedicated macOS fixture: READY requires real Cocoa key dispatch to the
/// shipped Settings action, followed by a displayed frame. Never moves pointer.
#[cfg(target_os = "macos")]
pub fn mac_acceptance(store: Store, paths: Paths, scenario: &str) -> Result<()> {
    use std::{ffi::c_void, io::Write};
    unsafe extern "C" {
        fn pp_smoke_key(
            handle: *mut c_void,
            key: u16,
            modifiers: u32,
            text: *const std::ffi::c_char,
        );
        fn pp_smoke_display(handle: *mut c_void);
        fn pp_smoke_terminate(apple_event: i32);
    }
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(Language::Zh);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    let scenario = scenario.to_owned();
    let view = c.clone();
    // Posting events differs from invoking a callback or a READY timer. The
    // marker below waits for the shipped action's observable page transition.
    app::add_timeout3(0.0, move |_| unsafe {
        pp_smoke_display(view.window.raw_handle());
        pp_smoke_key(view.window.raw_handle(), 48, 0, c"\t".as_ptr());
        pp_smoke_key(view.window.raw_handle(), 49, 0, c" ".as_ptr());
    });
    let mut view = c.clone();
    let started = std::time::Instant::now();
    app::add_timeout3(0.01, move |timer| {
        if !view.settings_page.visible() {
            assert!(
                started.elapsed().as_secs() < 10,
                "native Settings key dispatch did not respond"
            );
            app::repeat_timeout3(0.01, timer);
            return;
        }
        if !scenario.ends_with("settings") {
            view.back.do_callback();
        }
        if scenario == "editor" || scenario.ends_with("editor") {
            view.new.do_callback();
        } else if scenario == "import" {
            // Eight maximum-bound links, valid scoped credentials plus ignored
            // inline JSON. Preview/cancel exercise the production import path.
            let mut url = url::Url::parse("ccswitch://v1/import").unwrap();
            let prefix = "ccswitch://v1/import?resource=provider&app=claude&name=Stress&endpoint=https%3A%2F%2Ffixture.example&apiKey=fake&configFormat=json&config=";
            let payload = serde_json::json!({"ignored": "x".repeat(196_000)}).to_string();
            use base64::Engine;
            url.set_query(Some(&format!(
                "{}{}",
                prefix.split_once('?').unwrap().1,
                base64::engine::general_purpose::STANDARD.encode(payload)
            )));
            assert!(url.as_str().len() <= 256 * 1024);
            for index in 0..8 {
                preview_link(&mut view, state.clone(), &store, &paths, url.as_str())
                    .expect("bounded import fixture");
                assert!(view.editor.shown() && view.name.value() == "Stress");
                assert_eq!(view.model.value(), crate::client::CLAUDE_EDITOR_DEFAULT);
                if index != 7 {
                    view.cancel.do_callback();
                }
            }
            assert!(store.load().unwrap().is_empty());
        }
        unsafe {
            pp_smoke_display(if view.editor.shown() {
                view.editor.raw_handle()
            } else {
                view.window.raw_handle()
            });
        }
        println!("READY");
        std::io::stdout().flush().unwrap();
        if scenario.starts_with("quit-") {
            assert!(store.load().unwrap().is_empty());
            for client in Client::ALL {
                assert!(!paths.settings(client).exists());
            }
            println!("QUIT FIXTURES UNCHANGED");
            std::io::stdout().flush().unwrap();
            unsafe { pp_smoke_terminate(i32::from(scenario.starts_with("quit-dock-"))) };
        } else {
            app::add_timeout3(3.0, |_| quit());
        }
    });
    event_loop();
    Ok(())
}

/// Background lifecycle fixture; example-only native code suppresses ordering
/// and activation. Uses shipped widgets, Quit callback and real FLTK event wait.
#[cfg(target_os = "macos")]
pub fn mac_quit_background(store: Store, paths: Paths, scenario: &str) -> Result<()> {
    unsafe extern "C" {
        fn pp_quit_background_request(apple_event: i32);
    }
    assert!(
        [
            "menu-main",
            "dock-main",
            "menu-settings",
            "dock-settings",
            "menu-editor",
            "dock-editor",
            "dock-unhandled-main"
        ]
        .contains(&scenario)
    );
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(Language::Zh);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    if scenario.ends_with("settings") {
        c.settings.do_callback();
    } else if scenario.ends_with("editor") {
        c.new.do_callback();
    }
    let apple_event = if scenario == "dock-unhandled-main" {
        2
    } else {
        i32::from(scenario.starts_with("dock-"))
    };
    app::add_timeout3(0.1, move |_| {
        assert!(store.load().unwrap().is_empty());
        for client in Client::ALL {
            assert!(!paths.settings(client).exists());
        }
        println!("QUIT FIXTURES UNCHANGED");
        unsafe { pp_quit_background_request(apple_event) };
    });
    event_loop();
    Ok(())
}

/// Example-only isolated callback regression; presentation suppression is
/// installed by mac_quit_background before entering this fixture.
#[cfg(target_os = "macos")]
pub fn mac_model_prefill_background(store: Store, paths: Paths) -> Result<()> {
    unsafe extern "C" {
        fn pp_quit_background_request(apple_event: i32);
    }
    let provider = Provider::new(Client::Claude, "Fixture", "https://fixture.example", "fake")?;
    let config = r#"{"model":"sonnet","env":{"ANTHROPIC_BASE_URL":"https://fixture.example","ANTHROPIC_AUTH_TOKEN":"fake","ANTHROPIC_DEFAULT_SONNET_MODEL":"fixture-default"},"hooks":{}}"#;
    let file = paths.settings(Client::Claude);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, config).unwrap();
    store.save(provider.clone())?;
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(Language::Zh);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    let mut action = c.rows.borrow()[0].switch.clone();
    assert!(!action.active());
    action.do_callback();
    assert!(!c.editor.shown());
    let mut action = c.rows.borrow()[0].edit.clone();
    action.do_callback();
    assert!(c.editor.shown());
    assert_eq!(c.model.value(), "fixture-default");
    c.cancel.do_callback();
    assert!(store.load()?[0].models.is_empty());
    let mut action = c.rows.borrow()[0].edit.clone();
    action.do_callback();
    assert_eq!(c.model.value(), "fixture-default");
    c.save.do_callback();
    assert_eq!(
        store.load()?[0].models.model.as_deref(),
        Some("fixture-default")
    );
    let make_link = |endpoint: &str, model: Option<&str>| {
        let mut link = url::Url::parse("ccswitch://v1/import").unwrap();
        link.query_pairs_mut().extend_pairs([
            ("resource", "provider"),
            ("app", "claude"),
            ("name", "Import"),
            ("endpoint", endpoint),
            ("apiKey", "fake"),
        ]);
        if let Some(model) = model {
            link.query_pairs_mut().append_pair("model", model);
        }
        link.to_string()
    };
    for (endpoint, model, expected) in [
        ("https://fixture.example", None, "fixture-default"),
        (
            "https://fixture.example",
            Some("link-default"),
            "link-default",
        ),
        (
            "https://different.example",
            None,
            crate::client::CLAUDE_EDITOR_DEFAULT,
        ),
    ] {
        preview_link(
            &mut c,
            state.clone(),
            &store,
            &paths,
            &make_link(endpoint, model),
        )?;
        assert_eq!(c.model.value(), expected);
        c.cancel.do_callback();
    }
    c.new.do_callback();
    assert_eq!(c.model.value(), crate::client::CLAUDE_EDITOR_DEFAULT);
    c.cancel.do_callback();
    let mut action = c.rows.borrow()[0].edit.clone();
    action.do_callback();
    c.model.set_value("");
    c.save.do_callback();
    assert!(store.load()?[0].models.is_empty());
    let mut action = c.rows.borrow()[0].switch.clone();
    action.do_callback();
    assert!(!c.editor.shown());
    for index in 1..Client::ALL.len() {
        c.clients[index].do_callback();
        c.new.do_callback();
        assert!(c.model.value().is_empty());
        c.cancel.do_callback();
    }
    assert_eq!(store.load()?.len(), 1);
    assert_eq!(std::fs::read_to_string(file).unwrap(), config);
    println!("MODEL PREFILL EDIT SWITCH IMPORT SAVE CANCEL PASSED; CLIENT FIXTURE UNCHANGED");
    app::add_timeout3(0.1, |_| unsafe { pp_quit_background_request(0) });
    event_loop();
    Ok(())
}

/// Isolated real Apple URL-event acceptance; never uses system client paths.
#[cfg(target_os = "macos")]
pub fn protocol_smoke(store: Store, paths: Paths, output: std::path::PathBuf) -> Result<()> {
    protocol::prepare();
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(Language::Zh);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    let mut step = 0;
    let finished = output.clone();
    protocol::listen(move |link| {
        let result = (|| -> Result<()> {
            let before = store.load()?;
            let current = paths.current(Client::Claude)?;
            let result = preview_link(&mut c, state.clone(), &store, &paths, link);
            match step {
                0 | 2 => {
                    result?;
                    if !c.editor.shown()
                        || c.name.value() != "Native link"
                        || c.key.value() != "native-fake-key"
                    {
                        return Err("Native URL event did not open import preview.".into());
                    }
                    app::check();
                    acceptance::snapshot(
                        &mut c.editor,
                        &output.join(format!("native-{step}.ppm")),
                    )?;
                    if step == 2 {
                        c.cancel.do_callback();
                    }
                }
                1 => {
                    if result.is_ok() || c.name.value() != "Native link" {
                        return Err("Native URL overwrote an open draft.".into());
                    }
                    c.cancel.do_callback();
                }
                3 => {
                    if result.is_ok() || c.editor.shown() {
                        return Err("Malformed native URL was not rejected.".into());
                    }
                }
                _ => return Err("Unexpected extra native URL event.".into()),
            }
            if store.load()? != before || paths.current(Client::Claude)? != current {
                return Err("Native URL event wrote provider/client data.".into());
            }
            Ok(())
        })();
        if let Err(e) = result {
            let _ = std::fs::write(output.join("error.txt"), e);
            app::add_timeout3(0.0, |_| {});
            app::quit();
            return;
        }
        step += 1;
        let _ = std::fs::write(output.join("step.txt"), step.to_string());
        if step == 4 {
            app::add_timeout3(0.0, |_| {});
            app::quit();
        }
    });
    event_loop();
    if finished.join("error.txt").exists()
        || std::fs::read_to_string(finished.join("step.txt"))
            .ok()
            .as_deref()
            != Some("4")
    {
        return Err("Native URL acceptance did not finish.".into());
    }
    Ok(())
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn pp18_model_less_claude_matches_connection_but_explicit_model_requires_a_match() {
        let mut provider =
            Provider::new(Client::Claude, "Fixture", "https://fixture.example", "fake").unwrap();
        let state = State {
            current: Some((provider.base_url.clone(), provider.api_key.clone())),
            ..State::default()
        };
        assert!(matching(&state, &provider));
        provider.models.model = Some("compatible-model".into());
        assert!(!matching(&state, &provider));
        provider.client = Client::Codex;
        provider.models = Models::default();
        assert!(matching(&state, &provider));
    }
}
