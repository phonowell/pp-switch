use crate::{
    Result,
    client::{Paths, environment_conflict},
    i18n::{self, Language, tr},
    provider::{Client, Provider},
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
mod lucide;
use lucide::Icon;

#[derive(Clone)]
struct Row {
    id: String,
    pick: Button,
    edit: Button,
    switch: Button,
    selected: Rc<Cell<bool>>,
    active: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct Controls {
    window: Window,
    clients: [Button; 3],
    client: Rc<Cell<Client>>,
    language: Rc<Cell<Language>>,
    main_page: Group,
    settings_page: Group,
    settings: Button,
    back: Button,
    languages: Choice,
    update: Button,
    labels: Vec<(Frame, &'static str)>,
    raw_status: Rc<RefCell<String>>,
    updating: Rc<Cell<bool>>,
    list: Scroll,
    rows: Rc<RefCell<Vec<Row>>>,
    empty: Frame,
    editor: Window,
    name: Input,
    url: Input,
    key: SecretInput,
    new: Button,
    save: Button,
    cancel: Button,
    delete: Button,
    status: Frame,
}

#[derive(Default)]
struct State {
    providers: Vec<Provider>,
    selected: Option<String>,
    editing: Option<String>,
    current: Option<(String, String)>,
    setup_error: Option<String>,
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
        let message = self.raw_status.borrow().clone();
        self.message(&message);
        self.window.redraw();
        self.editor.redraw();
    }
    fn hide_dialogs(&mut self) {
        self.editor.hide();
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
fn icon_button(button: &mut Button, icon: Icon, selected: Option<Rc<Cell<bool>>>) {
    button.set_frame(FrameType::NoBox);
    button.draw(move |b| {
        draw::set_draw_color(if selected.as_ref().is_some_and(|v| v.get()) {
            theme::selected()
        } else {
            theme::background()
        });
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
    let mut window = Window::new(100, 100, 440, 502, "pp-switch");
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
    icon_button(&mut settings, Icon::Settings, None);
    let client = Rc::new(Cell::new(Client::Claude));
    let mut track = Frame::new(24, 24, 348, 38, "");
    track.draw(|f| {
        draw::set_draw_color(theme::hover());
        draw::draw_rounded_rectf(f.x(), f.y(), f.w(), f.h(), 7);
    });
    let clients = ["Claude Code", "Codex", "Gemini CLI"].map(|name| {
        let index = ["Claude Code", "Codex", "Gemini CLI"]
            .iter()
            .position(|s| *s == name)
            .unwrap();
        let mut b = Button::new(24 + index as i32 * 116, 24, 116, 38, name);
        button(&mut b, false);
        b.set_frame(FrameType::NoBox);
        b.set_down_frame(FrameType::NoBox);
        b.set_label_size(12);
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
            draw::set_font(Font::Helvetica, 12);
            draw::set_draw_color(if selected {
                theme::ink()
            } else {
                theme::muted()
            });
            draw::draw_text2(&b.label(), b.x(), b.y(), b.w(), b.h(), Align::Center);
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
    let mut heading = Frame::new(124, 24, 292, 32, "Settings");
    heading.set_label_size(15);
    heading.set_label_font(Font::HelveticaBold);
    heading.set_label_color(theme::ink());
    heading.set_align(Align::Left | Align::Inside);
    labels.push((heading, "Settings"));
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
    let mut status = Frame::new(24, 208, 392, 84, "");
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
        labels,
        raw_status: Rc::new(RefCell::new(String::new())),
        updating: Rc::new(Cell::new(false)),
        list,
        rows: Rc::new(RefCell::new(vec![])),
        empty,
        editor,
        name,
        url,
        key,
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
}
fn selection(c: &mut Controls, state: &State) {
    for row in c.rows.borrow_mut().iter_mut() {
        row.selected.set(Some(&row.id) == state.selected.as_ref());
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
        row.pick.redraw();
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
}
fn open_editor(c: &mut Controls, state: &mut State, p: Option<&Provider>) {
    state.editing = p.map(|p| p.id.clone());
    c.name.set_value(p.map_or("", |p| &p.name));
    c.url.set_value(p.map_or("", |p| &p.base_url));
    c.key.set_value(p.map_or("", |p| &p.api_key));
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
    c.editor.set_pos(c.window.x() + 27, c.window.y() + 53);
    c.editor.show();
    let _ = c.name.take_focus();
}
fn read_current(state: &mut State, paths: &Paths, client: Client) {
    let current = paths.current(client);
    state.setup_error = environment_conflict(client)
        .err()
        .or_else(|| current.as_ref().err().cloned());
    state.current = current.ok().flatten();
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
        if !s
            .providers
            .iter()
            .any(|p| p.client == c.client.get() && Some(&p.id) == s.selected.as_ref())
        {
            s.selected = s
                .providers
                .iter()
                .filter(|p| p.client == c.client.get())
                .find(|p| matching(&s, p))
                .or_else(|| s.providers.iter().find(|p| p.client == c.client.get()))
                .map(|p| p.id.clone());
        }
    }
    c.rows.borrow_mut().clear();
    c.list.clear();
    c.list.scroll_to(0, 0);
    c.list.begin();
    let current_state = state.borrow();
    let providers: Vec<_> = current_state
        .providers
        .iter()
        .filter(|p| p.client == c.client.get())
        .collect();
    let row_width = if providers.len() * 74 > c.list.h() as usize {
        380
    } else {
        392
    };
    for (index, p) in providers.iter().enumerate() {
        let selected = Rc::new(Cell::new(false));
        let active = Rc::new(Cell::new(matching(&state.borrow(), p)));
        let active_draw = active.clone();
        let mut pick = Button::new(24, 124 + index as i32 * 74, row_width, 69, "");
        pick.set_frame(FrameType::NoBox);
        let chosen = selected.clone();
        let language = c.language.clone();
        let name = p.name.clone();
        let url = p.base_url.clone();
        pick.draw(move |b| {
            draw::set_draw_color(theme::background());
            draw::draw_rectf(b.x(), b.y(), b.w(), b.h());
            draw::set_draw_color(if chosen.get() {
                theme::selected()
            } else {
                theme::background()
            });
            draw::draw_rounded_rectf(b.x(), b.y(), b.w(), b.h(), 8);
            if chosen.get() {
                draw::set_draw_color(theme::selected_border());
                draw::draw_rounded_rect(b.x(), b.y(), b.w(), b.h(), 8);
            }
            draw::set_draw_color(if chosen.get() {
                theme::accent()
            } else {
                theme::radio_outline()
            });
            lucide::draw(
                if chosen.get() {
                    Icon::Selected
                } else {
                    Icon::Circle
                },
                b.x() + 11,
                b.y() + 26,
                16,
            );
            draw::set_font(Font::Helvetica, 10);
            let badge = tr(language.get(), "In use");
            let badge_width = draw::measure(badge, false).0 + 6;
            // Reserve both icon hit targets and an 8px gap before the badge.
            let badge_left = b.w() - 75 - 8 - badge_width;
            let text_width = if active_draw.get() {
                badge_left - 12 - 39
            } else {
                b.w() - 126
            };
            draw::push_clip(b.x() + 39, b.y() + 12, text_width, 46);
            draw::set_draw_color(theme::ink());
            draw::set_font(Font::HelveticaBold, 14);
            draw::draw_text2(
                &name,
                b.x() + 39,
                b.y() + 12,
                text_width,
                23,
                Align::Left | Align::Inside,
            );
            draw::set_draw_color(theme::muted());
            draw::set_font(Font::Helvetica, 11);
            draw::draw_text2(
                &url,
                b.x() + 39,
                b.y() + 37,
                text_width,
                18,
                Align::Left | Align::Inside,
            );
            draw::pop_clip();
            if active_draw.get() {
                draw::set_font(Font::Helvetica, 10);
                draw::set_draw_color(theme::active_badge());
                draw::draw_text2(
                    badge,
                    b.x() + badge_left,
                    b.y(),
                    badge_width,
                    b.h(),
                    Align::Right | Align::Inside,
                );
            }
            if b.has_focus() {
                draw::set_draw_color(theme::accent());
                draw::draw_rounded_rect(b.x() + 2, b.y() + 2, b.w() - 4, b.h() - 4, 7);
            }
        });
        let mut view = c.clone();
        let chosen_state = state.clone();
        let id = p.id.clone();
        let current_paths = paths.clone();
        pick.set_callback(move |_| {
            let mut s = chosen_state.borrow_mut();
            s.selected = Some(id.clone());
            read_current(&mut s, &current_paths, view.client.get());
            selection(&mut view, &s);
        });
        let y = 124 + index as i32 * 74 + 19;
        let mut edit = Button::new(24 + row_width - 75, y, 30, 30, "");
        icon_button(&mut edit, Icon::Edit, Some(selected.clone()));
        let mut view = c.clone();
        let edit_state = state.clone();
        let id = p.id.clone();
        edit.set_callback(move |_| {
            let provider = edit_state
                .borrow()
                .providers
                .iter()
                .find(|p| p.id == id)
                .cloned();
            if let Some(provider) = provider {
                open_editor(&mut view, &mut edit_state.borrow_mut(), Some(&provider));
            }
        });
        let mut switch = Button::new(24 + row_width - 39, y, 30, 30, "");
        icon_button(&mut switch, Icon::Switch, Some(selected.clone()));
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
                switch_state.borrow_mut().selected = Some(id.clone());
                refresh(
                    &mut view,
                    switch_state.clone(),
                    &switch_store,
                    &switch_paths,
                )
            })();
            if let Err(error) = result {
                i18n::alert(view.lang(), &error);
            }
        });
        c.rows.borrow_mut().push(Row {
            id: p.id.clone(),
            pick,
            edit,
            switch,
            selected,
            active,
        });
    }
    c.list.end();
    if providers.is_empty() {
        c.empty.show();
    } else {
        c.empty.hide();
    }
    selection(c, &state.borrow());
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
    let mut view = c.clone();
    c.window.set_callback(move |w| {
        view.hide_dialogs();
        w.hide();
        app::quit();
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
    c.new
        .set_callback(move |_| open_editor(&mut view, &mut new_state.borrow_mut(), None));
    for (i, mut b) in c.clients.clone().into_iter().enumerate() {
        let mut view = c.clone();
        let state = state.clone();
        let store = store.clone();
        let paths = paths.clone();
        b.set_callback(move |_| {
            view.client.set(Client::ALL[i]);
            state.borrow_mut().selected = None;
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
            if let Some(id) = &state_save.borrow().editing {
                p.id.clone_from(id);
            }
            store_save.save(p.clone())?;
            state_save.borrow_mut().selected = Some(p.id);
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

enum UpdateEvent {
    Checked(Result<Option<update::Update>>),
    Staged(Result<update::Staged>),
}

fn check_updates(c: &mut Controls, check: fn() -> Result<Option<update::Update>>) {
    let mut view = c.clone();
    {
        if view.updating.replace(true) {
            return;
        }
        view.update.deactivate();
        view.update.set_label(tr(view.lang(), "Updating…"));
        view.message("Checking for updates…");
        let (sender, receiver) = std::sync::mpsc::channel();
        let checked = sender.clone();
        std::thread::spawn(move || {
            let _ = checked.send(UpdateEvent::Checked(check()));
        });
        let mut view = view.clone();
        let mut deadline = std::time::Instant::now() + std::time::Duration::from_secs(70);
        app::add_timeout3(0.1, move |handle| {
            let mut finished = false;
            if std::time::Instant::now() >= deadline {
                view.message("Update request timed out. Try again.");
                view.updating.set(false);
                view.update.activate();
                view.update.set_label(tr(view.lang(), "Check for updates"));
                app::remove_timeout3(handle);
                return;
            }
            match receiver.try_recv() {
                Ok(UpdateEvent::Checked(Ok(Some(release)))) => {
                    let prompt = tr(
                        view.lang(),
                        "Install version {version} and restart? Unsaved edits will be lost.",
                    )
                    .replace("{version}", &release.version.to_string());
                    if i18n::confirm(
                        view.lang(),
                        &prompt,
                        tr(view.lang(), "Cancel"),
                        Some(tr(view.lang(), "Install and restart")),
                    ) {
                        view.message("Downloading and verifying update…");
                        let language = view.lang();
                        let staged = sender.clone();
                        deadline = std::time::Instant::now() + std::time::Duration::from_secs(70);
                        std::thread::spawn(move || {
                            let _ =
                                staged.send(UpdateEvent::Staged(update::stage(&release, language)));
                        });
                    } else {
                        view.message("Update cancelled. Nothing was installed.");
                        finished = true;
                    }
                }
                Ok(UpdateEvent::Checked(Ok(None))) => {
                    view.message("You are using the latest available version.");
                    finished = true;
                }
                Ok(UpdateEvent::Checked(Err(error)) | UpdateEvent::Staged(Err(error))) => {
                    view.message(&error);
                    finished = true;
                }
                Ok(UpdateEvent::Staged(Ok(mut staged))) => {
                    staged.plan.language = view.lang();
                    match update::launch_helper(staged) {
                        Ok(()) => {
                            view.message("Installing update…");
                            // Bound Cocoa's native wait after the timer hides the final window.
                            app::add_timeout3(0.0, |_| {});
                            app::quit();
                        }
                        Err(error) => {
                            view.message(&error);
                            finished = true;
                        }
                    }
                    if !finished {
                        return;
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    view.message("Update worker stopped. Try again.");
                    finished = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
            if finished {
                view.updating.set(false);
                view.update.activate();
                view.update.set_label(tr(view.lang(), "Check for updates"));
                app::remove_timeout3(handle);
            } else {
                app::repeat_timeout3(0.1, handle);
            }
        });
    }
}

pub fn run() -> Result<()> {
    run_with(Store::system()?, Paths::system()?)
}

/// Production UI with isolated paths available to development tools.
pub fn run_with(store: Store, paths: Paths) -> Result<()> {
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(store.language()?);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state, store, paths);
    c.window.show();
    while app::wait() {}
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
    wire(&mut c, state.clone(), store, paths);
    c.window.show();
    let provider = state.borrow().providers.first().cloned();
    open_editor(&mut c, &mut state.borrow_mut(), provider.as_ref());
    while app::wait() {}
    Ok(())
}
