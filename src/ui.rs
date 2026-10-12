use crate::{
    Result,
    client::{Paths, environment_conflict},
    i18n::{self, Language, tr},
    provider::{Client, Provider, import_link},
    storage::Store,
    theme, update,
};
use fltk::{
    app,
    button::Button,
    draw,
    enums::{Align, Color, Event, Font, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Scroll, ScrollType},
    input::{Input, SecretInput},
    menu::{MenuButton, MenuFlag},
    output::MultilineOutput,
    prelude::*,
    window::Window,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone)]
struct Row {
    id: String,
    pick: Button,
    edit: Button,
    selected: Rc<Cell<bool>>,
    active: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct Controls {
    window: Window,
    clients: [Button; 3],
    client: Rc<Cell<Client>>,
    language: Rc<Cell<Language>>,
    more: MenuButton,
    import_item: i32,
    update_item: i32,
    language_items: [i32; 4],
    labels: Vec<(Frame, &'static str)>,
    raw_status: Rc<RefCell<String>>,
    updating: Rc<Cell<bool>>,
    list: Scroll,
    rows: Rc<RefCell<Vec<Row>>>,
    empty: Frame,
    editor: Window,
    importer: Window,
    name: Input,
    url: Input,
    key: SecretInput,
    link: Input,
    new: Button,
    save: Button,
    cancel: Button,
    apply: Button,
    delete: Button,
    import: Button,
    import_cancel: Button,
    status: MultilineOutput,
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
    fn menu_text(&mut self, index: i32, text: &str) {
        if let Some(mut item) = self.more.at(index) {
            item.set_label(text);
        }
    }
    fn message(&mut self, english: &str) {
        let message = i18n::message(self.lang(), english);
        self.status.set_value(&message);
        self.status.set_tooltip(&message);
        *self.raw_status.borrow_mut() = english.to_owned();
    }
    fn translate(&mut self) {
        let lang = self.lang();
        for (label, key) in &mut self.labels {
            label.set_label(tr(lang, key));
        }
        for (button, key) in [
            (&mut self.new, "+ Add"),
            (&mut self.save, "Save"),
            (&mut self.cancel, "Cancel"),
            (&mut self.delete, "Delete"),
            (&mut self.import, "Preview"),
            (&mut self.import_cancel, "Cancel"),
        ] {
            button.set_label(tr(lang, key));
        }
        self.editor.set_label(tr(lang, "Edit provider"));
        self.importer.set_label(tr(lang, "Import link"));
        self.more.set_tooltip(tr(lang, "More"));
        self.menu_text(self.import_item, tr(lang, "Import link"));
        self.menu_text(
            self.update_item,
            tr(
                lang,
                if self.updating.get() {
                    "Updating…"
                } else {
                    "Check for updates"
                },
            ),
        );
        self.menu_text(self.language_items[0] - 1, tr(lang, "Language"));
        for (index, language) in self.language_items.into_iter().zip(Language::ALL) {
            self.more.set_mode(
                index,
                MenuFlag::Radio
                    | if language == lang {
                        MenuFlag::Value
                    } else {
                        MenuFlag::Normal
                    },
            );
        }
        self.empty.set_label(tr(lang, "Nothing here yet."));
        for row in self.rows.borrow_mut().iter_mut() {
            row.edit.set_tooltip(tr(lang, "Edit provider"));
        }
        let message = self.raw_status.borrow().clone();
        self.message(&message);
        self.window.redraw();
        self.editor.redraw();
        self.importer.redraw();
    }
    fn hide_dialogs(&mut self) {
        self.editor.hide();
        self.importer.hide();
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
    input.set_color(Color::from_rgb(240, 242, 246));
    input.set_text_color(theme::ink());
    input.set_selection_color(theme::accent());
    input.set_text_size(13);
}
fn button(button: &mut Button, primary: bool) {
    button.set_frame(FrameType::RFlatBox);
    button.set_down_frame(FrameType::RFlatBox);
    button.set_color(if primary {
        theme::accent()
    } else {
        theme::background()
    });
    button.set_selection_color(if primary {
        Color::from_rgb(46, 83, 200)
    } else {
        Color::from_rgb(233, 236, 241)
    });
    button.set_label_color(if primary {
        Color::White
    } else {
        theme::muted()
    });
    button.set_label_size(13);
}
fn controls(language: Language) -> Controls {
    let _app = app::App::default().with_scheme(app::Scheme::Base);
    app::set_font(Font::Helvetica);
    theme::init();
    let mut window = Window::new(100, 100, 440, 502, "pp-switch");
    window.set_color(theme::background());
    let icon = fltk::image::RgbImage::new(
        include_bytes!("../assets/icon-64.rgb"),
        64,
        64,
        fltk::enums::ColorDepth::Rgb8,
    )
    .expect("compiled application icon");
    window.set_icon(Some(icon));
    let mut title = Frame::new(24, 22, 200, 28, "pp-switch");
    title.set_label_size(15);
    title.set_label_font(Font::HelveticaBold);
    title.set_label_color(theme::ink());
    title.set_align(Align::Left | Align::Inside);
    let mut more = MenuButton::new(384, 22, 32, 28, "");
    more.set_frame(FrameType::NoBox);
    more.set_color(theme::background());
    more.set_text_size(13);
    more.set_text_color(theme::ink());
    more.set_selection_color(theme::selected());
    more.draw(|m| {
        draw::set_draw_color(theme::background());
        draw::draw_rectf(m.x(), m.y(), m.w(), m.h());
        draw::set_draw_color(theme::muted());
        for dx in [9, 16, 23] {
            draw::draw_circle((m.x() + dx) as f64, (m.y() + 14) as f64, 1.2);
        }
        if m.has_focus() {
            draw::set_draw_color(theme::accent());
            draw::draw_rounded_rect(m.x(), m.y(), m.w(), m.h(), 5);
        }
    });
    more.add_choice("Import link|Check for updates|Language/English|Language/简体中文|Language/繁體中文|Language/日本語");
    let import_item = more.find_index("Import link");
    let update_item = more.find_index("Check for updates");
    let language_items = ["English", "简体中文", "繁體中文", "日本語"]
        .map(|name| more.find_index(&format!("Language/{name}")));
    let client = Rc::new(Cell::new(Client::Claude));
    let clients = ["Claude Code", "Codex", "Gemini CLI"].map(|name| {
        let index = ["Claude Code", "Codex", "Gemini CLI"]
            .iter()
            .position(|s| *s == name)
            .unwrap();
        let mut b = Button::new(
            24 + index as i32 * 131,
            78,
            if index == 2 { 130 } else { 131 },
            38,
            name,
        );
        button(&mut b, false);
        b.set_label_size(12);
        let client = client.clone();
        b.draw(move |b| {
            draw::set_draw_color(Color::from_rgb(233, 236, 241));
            draw::draw_rounded_rectf(b.x(), b.y(), b.w(), b.h(), 7);
            let selected = client.get() == Client::ALL[index];
            if selected {
                draw::set_draw_color(theme::background());
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
        });
        b
    });
    let mut labels = vec![label(24, 143, 180, "Providers")];
    let mut new = Button::new(328, 138, 88, 28, "+ Add");
    button(&mut new, false);
    new.set_label_size(12);
    let mut list = Scroll::new(24, 178, 392, 248, "");
    list.set_frame(FrameType::FlatBox);
    list.set_color(theme::background());
    list.set_type(ScrollType::Vertical);
    list.set_scrollbar_size(10);
    list.end();
    let mut empty = Frame::new(24, 270, 392, 28, "Nothing here yet.");
    empty.set_label_color(theme::muted());
    empty.set_label_size(13);
    let mut line = Frame::new(24, 434, 392, 1, "");
    line.set_frame(FrameType::FlatBox);
    line.set_color(theme::line());
    let mut status = MultilineOutput::new(24, 447, 282, 40, "");
    status.set_frame(FrameType::FlatBox);
    status.set_color(theme::background());
    status.set_text_color(theme::muted());
    status.set_text_size(11);
    status.set_wrap(true);
    let mut apply = Button::new(318, 449, 98, 36, "Switch");
    button(&mut apply, true);
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
    button(&mut delete, false);
    delete.set_label_color(Color::from_rgb(172, 64, 64));
    let mut cancel = Button::new(168, 328, 88, 36, "Cancel");
    button(&mut cancel, false);
    let mut save = Button::new(268, 328, 94, 36, "Save");
    button(&mut save, true);
    save.set_shortcut(Shortcut::Ctrl | Key::Enter);
    editor.end();
    let mut importer = Window::new(130, 160, 386, 210, "Import link");
    importer.make_modal(true);
    importer.set_color(theme::background());
    labels.push(label(24, 22, 338, "Provider link"));
    let mut link = Input::new(24, 52, 338, 54, "");
    field(&mut link);
    link.set_type(fltk::input::InputType::Secret);
    let mut import_cancel = Button::new(168, 145, 88, 36, "Cancel");
    button(&mut import_cancel, false);
    let mut import = Button::new(268, 145, 94, 36, "Preview");
    button(&mut import, true);
    importer.end();
    let mut c = Controls {
        window,
        clients,
        client,
        language: Rc::new(Cell::new(language)),
        more,
        import_item,
        update_item,
        language_items,
        labels,
        raw_status: Rc::new(RefCell::new(String::new())),
        updating: Rc::new(Cell::new(false)),
        list,
        rows: Rc::new(RefCell::new(vec![])),
        empty,
        editor,
        importer,
        name,
        url,
        key,
        link,
        new,
        save,
        cancel,
        apply,
        delete,
        import,
        import_cancel,
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
    let provider = state
        .providers
        .iter()
        .find(|p| Some(&p.id) == state.selected.as_ref());
    let active = provider.is_some_and(|p| matching(state, p));
    for row in c.rows.borrow_mut().iter_mut() {
        let selected = Some(&row.id) == state.selected.as_ref();
        row.selected.set(selected);
        row.active.set(
            state
                .providers
                .iter()
                .find(|p| p.id == row.id)
                .is_some_and(|p| matching(state, p)),
        );
        if selected {
            row.edit.show();
        } else {
            row.edit.hide();
        }
        row.pick.redraw();
    }
    if provider.is_some() && !active && state.setup_error.is_none() {
        c.apply.activate();
    } else {
        c.apply.deactivate();
    }
    c.apply
        .set_label(tr(c.lang(), if active { "In use" } else { "Switch" }));
    let message = if let Some(error) = &state.setup_error {
        error.clone()
    } else if let Some(p) = provider.filter(|_| !active) {
        format!("Switch to: {}", p.name)
    } else if let Some(p) = state
        .providers
        .iter()
        .find(|p| p.client == c.client.get() && matching(state, p))
    {
        format!("Current: {}", p.name)
    } else if let Some((url, _)) = &state.current {
        format!("Current: {url}")
    } else {
        "Not configured".into()
    };
    c.message(&message);
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
    let providers: Vec<_> = state
        .borrow()
        .providers
        .iter()
        .filter(|p| p.client == c.client.get())
        .cloned()
        .collect();
    let row_width = if providers.len() * 74 > 248 { 380 } else { 392 };
    for (index, p) in providers.iter().enumerate() {
        let selected = Rc::new(Cell::new(false));
        let active = Rc::new(Cell::new(matching(&state.borrow(), p)));
        let active_draw = active.clone();
        let mut pick = Button::new(24, 178 + index as i32 * 74, row_width, 69, "");
        pick.set_frame(FrameType::NoBox);
        pick.set_tooltip(&format!("{}\n{}", p.name, p.base_url));
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
                draw::set_draw_color(Color::from_rgb(219, 227, 245));
                draw::draw_rounded_rect(b.x(), b.y(), b.w(), b.h(), 8);
            }
            draw::set_draw_color(if chosen.get() {
                theme::accent()
            } else {
                Color::from_rgb(183, 190, 202)
            });
            if chosen.get() {
                draw::draw_pie(b.x() + 12, b.y() + 27, 14, 14, 0., 360.);
                draw::set_draw_color(Color::White);
                draw::draw_pie(b.x() + 17, b.y() + 32, 4, 4, 0., 360.);
            } else {
                draw::draw_arc(b.x() + 12, b.y() + 27, 14, 14, 0., 360.);
            }
            let text_width = b.w() - if active_draw.get() { 152 } else { 86 };
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
                draw::set_draw_color(Color::from_rgb(57, 121, 95));
                draw::draw_text2(
                    tr(language.get(), "In use"),
                    b.x() + b.w() - 107,
                    b.y(),
                    70,
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
        let mut edit = Button::new(
            24 + row_width - 39,
            178 + index as i32 * 74 + 19,
            30,
            30,
            "",
        );
        let chosen = selected.clone();
        edit.draw(move |b| {
            draw::set_draw_color(if chosen.get() {
                theme::selected()
            } else {
                theme::background()
            });
            draw::draw_rectf(b.x(), b.y(), b.w(), b.h());
            draw::set_draw_color(theme::muted());
            draw::draw_line(b.x() + 9, b.y() + 20, b.x() + 12, b.y() + 19);
            draw::draw_line(b.x() + 9, b.y() + 20, b.x() + 10, b.y() + 16);
            draw::draw_line(b.x() + 10, b.y() + 16, b.x() + 19, b.y() + 7);
            draw::draw_line(b.x() + 12, b.y() + 19, b.x() + 22, b.y() + 10);
            draw::draw_line(b.x() + 19, b.y() + 7, b.x() + 22, b.y() + 10);
            draw::draw_line(b.x() + 17, b.y() + 9, b.x() + 20, b.y() + 12);
            if b.has_focus() {
                draw::set_draw_color(theme::accent());
                draw::draw_rounded_rect(b.x(), b.y(), b.w(), b.h(), 5);
            }
        });
        edit.set_tooltip(tr(c.lang(), "Edit provider"));
        let mut view = c.clone();
        let edit_state = state.clone();
        let provider = p.clone();
        edit.set_callback(move |_| {
            open_editor(&mut view, &mut edit_state.borrow_mut(), Some(&provider))
        });
        c.rows.borrow_mut().push(Row {
            id: p.id.clone(),
            pick,
            edit,
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
    c.importer.set_callback(move |_| {
        view.importer.hide();
        view.link.set_value("");
    });
    let mut view = c.clone();
    c.import_cancel.set_callback(move |_| {
        view.importer.hide();
        view.link.set_value("");
    });
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
                view.message(&e);
            }
            for button in &mut view.clients {
                button.redraw();
            }
        });
    }
    let mut view = c.clone();
    let menu_state = state.clone();
    let preferences = store.clone();
    c.more.set_callback(move |m| {
        let index = m.value();
        if index == view.import_item {
            view.importer
                .set_pos(view.window.x() + 27, view.window.y() + 100);
            view.importer.show();
            let _ = view.link.take_focus();
        } else if index == view.update_item {
            check_updates(&mut view);
        } else if let Some(i) = view.language_items.iter().position(|v| *v == index) {
            let language = Language::ALL[i];
            if let Err(e) = preferences.save_language(language) {
                view.translate();
                view.message(&e);
                return;
            }
            view.language.set(language);
            view.translate();
            let s = menu_state.borrow();
            let active = s
                .providers
                .iter()
                .find(|p| Some(&p.id) == s.selected.as_ref())
                .is_some_and(|p| matching(&s, p));
            view.apply
                .set_label(tr(language, if active { "In use" } else { "Switch" }));
            view.editor.set_label(tr(
                language,
                if s.editing.is_some() {
                    "Edit provider"
                } else {
                    "Add provider"
                },
            ));
        }
    });
    let mut view = c.clone();
    let state_import = state.clone();
    let store_import = store.clone();
    let paths_import = paths.clone();
    c.import
        .set_callback(move |_| match import_link(&view.link.value()) {
            Ok(p) => {
                view.client.set(p.client);
                state_import.borrow_mut().selected = None;
                if let Err(e) = refresh(
                    &mut view,
                    state_import.clone(),
                    &store_import,
                    &paths_import,
                ) {
                    i18n::alert(view.lang(), &e);
                    return;
                }
                for b in &mut view.clients {
                    b.redraw();
                }
                view.importer.hide();
                view.link.set_value("");
                open_editor(&mut view, &mut state_import.borrow_mut(), None);
                view.name.set_value(&p.name);
                view.url.set_value(&p.base_url);
                view.key.set_value(&p.api_key);
            }
            Err(e) => i18n::alert(view.lang(), &e),
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
    let state_apply = state.clone();
    let store_apply = store.clone();
    let paths_apply = paths.clone();
    c.apply.set_callback(move |_| {
        let result = (|| -> Result<()> {
            let p = {
                let s = state_apply.borrow();
                s.providers
                    .iter()
                    .find(|p| Some(&p.id) == s.selected.as_ref())
                    .cloned()
                    .ok_or("Select a saved provider first.")?
            };
            environment_conflict(p.client)?;
            paths_apply.apply(&store_apply, &p)?;
            refresh(&mut view, state_apply.clone(), &store_apply, &paths_apply)
        })();
        if let Err(e) = result {
            view.message(&e);
            i18n::alert(view.lang(), &e);
        }
    });
    let mut view = c.clone();
    let state_delete = state;
    let store_delete = store;
    let paths_delete = paths;
    c.delete.set_callback(move |_| {
        if fltk::dialog::choice2_default(
            tr(
                view.lang(),
                "Delete this saved provider? Client configuration will stay as it is.",
            ),
            tr(view.lang(), "Cancel"),
            tr(view.lang(), "Delete"),
            "",
        ) != Some(1)
        {
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
    let view = c.clone();
    c.status.handle(move |_, event| {
        if event == Event::Released {
            i18n::alert(view.lang(), &view.raw_status.borrow());
            true
        } else {
            false
        }
    });
}

enum UpdateEvent {
    Checked(Result<Option<update::Update>>),
    Staged(Result<update::Staged>),
}

fn check_updates(c: &mut Controls) {
    let mut view = c.clone();
    {
        if view.updating.replace(true) {
            return;
        }
        view.more.set_mode(view.update_item, MenuFlag::Inactive);
        view.menu_text(view.update_item, tr(view.lang(), "Updating…"));
        view.message("Checking for updates…");
        let (sender, receiver) = std::sync::mpsc::channel();
        let checked = sender.clone();
        std::thread::spawn(move || {
            let _ = checked.send(UpdateEvent::Checked(update::check()));
        });
        let mut view = view.clone();
        let mut deadline = std::time::Instant::now() + std::time::Duration::from_secs(70);
        app::add_timeout3(0.1, move |handle| {
            let mut finished = false;
            if std::time::Instant::now() >= deadline {
                view.message("Update request timed out. Try again.");
                view.updating.set(false);
                view.more.set_mode(view.update_item, MenuFlag::Normal);
                view.menu_text(view.update_item, tr(view.lang(), "Check for updates"));
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
                    if fltk::dialog::choice2_default(
                        &prompt,
                        tr(view.lang(), "Cancel"),
                        tr(view.lang(), "Install and restart"),
                        "",
                    ) == Some(1)
                    {
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
                view.more.set_mode(view.update_item, MenuFlag::Normal);
                view.menu_text(view.update_item, tr(view.lang(), "Check for updates"));
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

/// The production UI with isolated paths available to development tools.
pub fn run_with(store: Store, paths: Paths) -> Result<()> {
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(store.language()?);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state, store, paths);
    c.window.show();
    while app::wait() {}
    Ok(())
}

/// Development-only real-widget acceptance with fake keys and isolated client files.
pub fn smoke(store: Store, paths: Paths, screenshot: Option<std::path::PathBuf>) -> Result<()> {
    for language in Language::ALL {
        let localized_store = Store {
            root: store.root.join(language.code()),
        };
        localized_store.save_language(language)?;
        let capture = screenshot.as_ref().map(|path| {
            if language == Language::En {
                path.clone()
            } else {
                path.with_file_name(format!(
                    "{}.{}.ppm",
                    path.file_stem().unwrap().to_string_lossy(),
                    language.code()
                ))
            }
        });
        smoke_language(localized_store, paths.clone(), capture, language)?;
    }
    Ok(())
}
fn capture(window: &mut Window, path: &std::path::Path) -> Result<()> {
    let image = draw::capture_window(window).map_err(|_| "Cannot capture GUI screenshot.")?;
    use std::io::Write;
    let mut file = std::fs::File::create(path).map_err(|_| "Cannot create screenshot.")?;
    write!(file, "P6\n{} {}\n255\n", image.data_w(), image.data_h())
        .map_err(|_| "Cannot write screenshot header.")?;
    file.write_all(&image.to_rgb_data())
        .map_err(|_| "Cannot write screenshot pixels.".into())
}
fn smoke_language(
    store: Store,
    paths: Paths,
    screenshot: Option<std::path::PathBuf>,
    language: Language,
) -> Result<()> {
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(store.language()?);
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    app::check();
    c.new.do_callback();
    if !c.editor.shown() || c.delete.visible() {
        return Err("GUI add dialog did not open correctly.".into());
    }
    c.name.set_value(tr(language, "Everyday"));
    c.url.set_value("https://api.example.com");
    c.key.set_value("smoke-test-key");
    let before = paths.current(Client::Claude)?;
    c.save.do_callback();
    app::check();
    if store.load()?.len() != 1 || c.editor.shown() || paths.current(Client::Claude)? != before {
        return Err("GUI save did not close the editor, or changed client files.".into());
    }
    c.apply.do_callback();
    app::check();
    if paths
        .current(Client::Claude)?
        .as_ref()
        .map(|v| v.0.as_str())
        != Some("https://api.example.com")
        || c.apply.active()
    {
        return Err(format!(
            "GUI apply/matching-provider state failed: {}",
            c.status.value()
        ));
    }
    // An external client change must not leave the matching-provider guard stale.
    let external = Provider::new(
        Client::Claude,
        "External",
        "https://external.example.com",
        "external-test-key",
    )?;
    paths.apply(&store, &external)?;
    let mut pick = c.rows.borrow()[0].pick.clone();
    pick.do_callback();
    if !c.apply.active() {
        return Err("GUI selection did not reread changed client configuration.".into());
    }
    c.apply.do_callback();
    let first_row = c.rows.borrow()[0].clone();
    let mut edit = first_row.edit.clone();
    edit.do_callback();
    if !c.editor.shown() || !c.delete.visible() {
        return Err("GUI edit action did not open the saved provider.".into());
    }
    c.url.set_value("https://edited.example.com");
    let selected = state.borrow().selected.clone();
    let editing = state.borrow().editing.clone();
    let next = Language::ALL[(language.index() + 1) % Language::ALL.len()];
    c.more.set_value(c.language_items[next.index()]);
    c.more.do_callback();
    if c.url.value() != "https://edited.example.com"
        || c.key.value() != "smoke-test-key"
        || state.borrow().selected != selected
        || state.borrow().editing != editing
        || store.language()? != next
    {
        return Err("GUI language menu lost selection/draft or did not persist.".into());
    }
    c.message("Save your edits before switching this provider.");
    c.more.set_value(c.language_items[language.index()]);
    c.more.do_callback();
    if c.status.value() != tr(language, "Save your edits before switching this provider.") {
        return Err("GUI language menu did not translate the current status.".into());
    }
    c.cancel.do_callback();
    if c.editor.shown()
        || !c.key.value().is_empty()
        || store.load()?[0].base_url != "https://api.example.com"
    {
        return Err("GUI cancel failed to discard the unsaved draft.".into());
    }
    c.more.set_value(c.import_item);
    c.more.do_callback();
    if !c.importer.shown() {
        return Err("GUI import menu did not open the dialog.".into());
    }
    c.link.set_value("ccswitch://v1/import?resource=provider&app=claude&name=Imported&endpoint=https%3A%2F%2Fother.example.com&apiKey=imported-test-key&model=ignored");
    c.import.do_callback();
    if !c.editor.shown()
        || c.importer.shown()
        || c.name.value() != "Imported"
        || !c.link.value().is_empty()
        || store.load()?.len() != 1
        || paths.current(Client::Claude)?
            != Some(("https://api.example.com".into(), "smoke-test-key".into()))
    {
        return Err("GUI import preview changed files or failed to populate the editor.".into());
    }
    c.cancel.do_callback();
    c.clients[1].do_callback();
    if c.client.get() != Client::Codex || !c.rows.borrow().is_empty() {
        return Err("GUI client tab failed.".into());
    }
    c.clients[0].do_callback();
    for (name, url, key) in [
        (
            tr(language, "Backup"),
            "https://backup.example.com",
            "backup-test-key",
        ),
        (
            tr(language, "Local"),
            "http://localhost:8080",
            "local-test-key",
        ),
    ] {
        c.new.do_callback();
        c.name.set_value(name);
        c.url.set_value(url);
        c.key.set_value(key);
        c.save.do_callback();
    }
    // Save never applies: the original provider remains active, the backup is selected.
    if paths.current(Client::Claude)?
        != Some(("https://api.example.com".into(), "smoke-test-key".into()))
    {
        return Err("GUI save unexpectedly applied another provider.".into());
    }
    let mut pick = c.rows.borrow()[1].pick.clone();
    pick.do_callback();
    if !c.apply.active() || c.editor.shown() {
        return Err("GUI row selection opened the editor or failed to enable switching.".into());
    }
    c.window.redraw();
    app::check();
    if let Some(path) = screenshot {
        capture(&mut c.window, &path)?;
        let mut edit = c.rows.borrow()[1].edit.clone();
        edit.do_callback();
        app::check();
        capture(
            &mut c.editor,
            &path.with_file_name(format!(
                "{}.edit.ppm",
                path.file_stem().unwrap().to_string_lossy()
            )),
        )?;
        c.cancel.do_callback();
    }
    c.window.do_callback();
    app::check();
    if c.window.shown() || c.editor.shown() || c.importer.shown() {
        return Err("GUI close left a visible window.".into());
    }
    // Delete native widgets and their callbacks between locale runs.
    Window::delete(c.editor);
    Window::delete(c.importer);
    Window::delete(c.window);
    Ok(())
}
