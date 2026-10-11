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
    browser::HoldBrowser,
    button::Button,
    enums::{Align, Color, Font, FrameType},
    frame::Frame,
    input::{Input, SecretInput},
    menu::Choice,
    output::MultilineOutput,
    prelude::*,
    window::Window,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone)]
struct Controls {
    window: Window,
    client: Choice,
    language: Choice,
    saved_language: Rc<Cell<Language>>,
    labels: Vec<(Frame, &'static str)>,
    raw_status: Rc<RefCell<String>>,
    updating: Rc<Cell<bool>>,
    empty: Frame,
    list: HoldBrowser,
    name: Input,
    url: Input,
    key: SecretInput,
    link: Input,
    new: Button,
    save: Button,
    apply: Button,
    delete: Button,
    import: Button,
    update: Button,
    status: MultilineOutput,
}

impl Controls {
    fn lang(&self) -> Language {
        Language::ALL[self.language.value().clamp(0, 3) as usize]
    }

    fn message(&mut self, english: &str) {
        self.status.set_value(&i18n::message(self.lang(), english));
        *self.raw_status.borrow_mut() = english.to_owned();
    }

    fn translate(&mut self) {
        let lang = self.lang();
        for (label, key) in &mut self.labels {
            label.set_label(tr(lang, key));
        }
        for (button, key) in [
            (&mut self.new, "+ New"),
            (&mut self.save, "Save"),
            (&mut self.apply, "Switch provider"),
            (&mut self.delete, "Delete"),
            (&mut self.import, "Import"),
        ] {
            button.set_label(tr(lang, key));
        }
        self.update.set_label(tr(
            lang,
            if self.updating.get() {
                "Updating…"
            } else {
                "Check for updates"
            },
        ));
        self.empty.set_label(tr(lang, "Nothing here yet."));
        self.language.set_tooltip(tr(lang, "Language"));
        let message = self.raw_status.borrow().clone();
        self.message(&message);
        self.window.redraw();
    }
}

struct State {
    providers: Vec<Provider>,
    editing: Option<String>,
}

fn label(x: i32, y: i32, width: i32, text: &'static str, title: bool) -> (Frame, &'static str) {
    let mut label = Frame::new(x, y, width, 22, text);
    label.set_align(Align::Left | Align::Inside);
    label.set_label_size(if title { 16 } else { 12 });
    label.set_label_color(if title { theme::ink() } else { theme::muted() });
    if title {
        label.set_label_font(Font::HelveticaBold);
    }
    (label, text)
}

fn field<T: InputExt + WidgetExt>(input: &mut T) {
    input.set_frame(FrameType::RFlatBox);
    input.set_color(theme::background());
    input.set_text_color(theme::ink());
    input.set_selection_color(theme::accent());
    input.set_text_size(14);
}

fn button(button: &mut Button, primary: bool) {
    button.set_frame(FrameType::RShadowBox);
    button.set_down_frame(FrameType::RoundDownBox);
    button.set_color(if primary {
        theme::accent()
    } else {
        theme::background()
    });
    button.set_selection_color(if primary {
        Color::from_rgb(55, 73, 148)
    } else {
        Color::from_rgb(221, 228, 237)
    });
    button.set_label_color(if primary { Color::White } else { theme::ink() });
    button.set_label_font(Font::HelveticaBold);
    button.set_label_size(13);
}

fn panel(x: i32, y: i32, w: i32, h: i32) {
    let mut frame = Frame::new(x, y, w, h, "");
    frame.set_frame(FrameType::RShadowBox);
    frame.set_color(theme::background());
}

fn controls(language: Language) -> Controls {
    let _app = app::App::default().with_scheme(app::Scheme::Base);
    app::set_font(Font::Helvetica);
    theme::init();
    let window = Window::new(100, 100, 780, 678, "pp-switch · Pure Provider");
    panel(24, 25, 48, 48);
    let mut mark = Frame::new(24, 23, 48, 48, "pp");
    mark.set_label_size(22);
    mark.set_label_font(Font::HelveticaBold);
    mark.set_label_color(theme::accent());
    let mut title = Frame::new(92, 23, 220, 38, "pp-switch");
    title.set_label_size(29);
    title.set_label_font(Font::HelveticaBold);
    title.set_label_color(theme::ink());
    title.set_align(Align::Left | Align::Inside);
    let mut labels = vec![label(
        92,
        65,
        460,
        "Pure Provider. Just your base URL and API key.",
        false,
    )];
    let mut version = Frame::new(476, 31, 95, 30, concat!("v", env!("CARGO_PKG_VERSION")));
    version.set_label_color(theme::muted());
    version.set_label_size(12);
    let mut language_choice = Choice::new(612, 32, 144, 36, "");
    language_choice.add_choice("English|简体中文|繁體中文|日本語");
    language_choice.set_value(language.index() as i32);
    language_choice.set_frame(FrameType::RShadowBox);
    language_choice.set_color(theme::background());
    language_choice.set_text_color(theme::ink());
    language_choice.set_text_size(13);
    theme::choice(&mut language_choice, true);
    panel(24, 118, 238, 376);
    panel(282, 118, 474, 376);
    labels.push(label(42, 134, 200, "Your providers", true));
    let mut client = Choice::new(42, 172, 202, 36, "");
    client.add_choice("Claude Code|Codex|Gemini CLI");
    client.set_value(0);
    client.set_frame(FrameType::RFlatBox);
    client.set_color(theme::background());
    client.set_text_color(theme::ink());
    client.set_text_size(14);
    theme::choice(&mut client, false);
    let mut list = HoldBrowser::new(42, 224, 202, 192, "");
    list.set_frame(FrameType::RFlatBox);
    list.set_color(theme::background());
    list.set_selection_color(Color::from_rgb(202, 213, 235));
    list.set_text_size(14);
    list.set_format_char('\u{1}');
    let mut empty = Frame::new(54, 291, 178, 36, "Nothing here yet.");
    empty.set_label_size(13);
    empty.set_label_color(theme::muted());
    let mut new = Button::new(42, 437, 202, 38, "+ New");
    button(&mut new, false);
    labels.push(label(300, 134, 438, "Connection details", true));
    labels.push(label(300, 171, 438, "Provider name", false));
    let mut name = Input::new(300, 197, 438, 38, "");
    field(&mut name);
    labels.push(label(300, 247, 438, "Base URL", false));
    let mut url = Input::new(300, 273, 438, 38, "");
    field(&mut url);
    labels.push(label(300, 323, 438, "API key", false));
    let mut key = SecretInput::new(300, 349, 438, 38, "");
    field(&mut key);
    let mut save = Button::new(300, 411, 100, 38, "Save");
    button(&mut save, false);
    let mut delete = Button::new(412, 411, 100, 38, "Delete");
    button(&mut delete, false);
    let mut apply = Button::new(526, 411, 212, 38, "Switch provider");
    button(&mut apply, true);
    labels.push(label(
        300,
        463,
        438,
        "Saving keeps client files unchanged. Switching updates URL + key.",
        false,
    ));
    labels.last_mut().unwrap().0.set_label_size(11);
    panel(24, 514, 732, 90);
    labels.push(label(
        42,
        523,
        694,
        "Import a cc-switch Provider link",
        false,
    ));
    let mut link = Input::new(42, 553, 568, 34, "");
    field(&mut link);
    link.set_type(fltk::input::InputType::Secret);
    let mut import = Button::new(628, 553, 110, 34, "Import");
    button(&mut import, false);
    let mut status = MultilineOutput::new(24, 625, 574, 42, "");
    status.set_frame(FrameType::FlatBox);
    status.set_color(theme::background());
    status.set_text_color(theme::muted());
    status.set_text_size(12);
    status.set_wrap(true);
    let mut update = Button::new(616, 631, 140, 30, "Check for updates");
    button(&mut update, false);
    update.set_label_size(11);
    window.end();
    let mut c = Controls {
        window,
        client,
        language: language_choice,
        saved_language: Rc::new(Cell::new(language)),
        labels,
        raw_status: Rc::new(RefCell::new(String::new())),
        empty,
        updating: Rc::new(Cell::new(false)),
        list,
        name,
        url,
        key,
        link,
        new,
        save,
        apply,
        delete,
        import,
        update,
        status,
    };
    c.translate();
    c
}

fn selected_client(c: &Controls) -> Client {
    Client::ALL[c.client.value().clamp(0, 2) as usize]
}

fn clear(c: &mut Controls, state: &mut State) {
    state.editing = None;
    c.list.deselect(0);
    c.name.set_value("");
    c.url.set_value("");
    c.key.set_value("");
    c.apply.deactivate();
    c.delete.deactivate();
}

fn fill(c: &mut Controls, p: &Provider, state: &mut State) {
    c.name.set_value(&p.name);
    c.url.set_value(&p.base_url);
    c.key.set_value(&p.api_key);
    state.editing = Some(p.id.clone());
    c.apply.activate();
    c.delete.activate();
}

fn refresh(c: &mut Controls, state: &mut State, store: &Store, paths: &Paths) -> Result<()> {
    state.providers = store.load()?;
    let client = selected_client(c);
    let current = paths.current(client);
    c.list.clear();
    for p in state.providers.iter().filter(|p| p.client == client) {
        let active = current
            .as_ref()
            .ok()
            .and_then(Option::as_ref)
            .is_some_and(|(url, key)| *url == p.base_url && *key == p.api_key);
        c.list
            .add(&format!("{}{}", if active { "* " } else { "  " }, p.name));
    }
    if c.list.size() == 0 {
        c.empty.show();
    } else {
        c.empty.hide();
    }
    let message = match environment_conflict(client) {
        Err(e) => e,
        Ok(()) => match current {
            Err(e) => e,
            Ok(None) => "Add a provider or paste a link. Only URL and key will be changed.".into(),
            Ok(Some((url, _))) => format!("Current: {url}  ·  * marks a matching provider"),
        },
    };
    c.message(&message);
    Ok(())
}

fn wire(c: &mut Controls, state: Rc<RefCell<State>>, store: Store, paths: Paths) {
    wire_update(c);
    let mut view = c.clone();
    let preferences = store.clone();
    c.language.set_callback(move |choice| {
        let language = view.lang();
        if let Err(error) = preferences.save_language(language) {
            choice.set_value(view.saved_language.get().index() as i32);
            view.message(&error);
            return;
        }
        view.saved_language.set(language);
        view.translate();
    });
    let mut view = c.clone();
    let state_new = state.clone();
    c.new.set_callback(move |_| {
        clear(&mut view, &mut state_new.borrow_mut());
        view.message("New provider. Save first, then switch.");
        let _ = view.name.take_focus();
    });

    let mut view = c.clone();
    let state_client = state.clone();
    let store_client = store.clone();
    let paths_client = paths.clone();
    c.client.set_callback(move |_| {
        let mut state = state_client.borrow_mut();
        clear(&mut view, &mut state);
        if let Err(e) = refresh(&mut view, &mut state, &store_client, &paths_client) {
            view.message(&e);
        }
    });

    let mut view = c.clone();
    let state_list = state.clone();
    c.list.set_callback(move |list| {
        let mut state = state_list.borrow_mut();
        let index = list.value() - 1;
        if index < 0 {
            return;
        }
        let client = selected_client(&view);
        let p = state
            .providers
            .iter()
            .filter(|p| p.client == client)
            .nth(index as usize)
            .cloned();
        if let Some(p) = p {
            fill(&mut view, &p, &mut state);
        }
    });

    let mut view = c.clone();
    let state_import = state.clone();
    c.import
        .set_callback(move |_| match import_link(&view.link.value()) {
            Ok(p) => {
                view.client.set_value(
                    Client::ALL
                        .iter()
                        .position(|client| *client == p.client)
                        .unwrap() as i32,
                );
                view.client.do_callback();
                let mut state = state_import.borrow_mut();
                clear(&mut view, &mut state);
                view.name.set_value(&p.name);
                view.url.set_value(&p.base_url);
                view.key.set_value(&p.api_key);
                view.link.set_value("");
                view.message(
                    "Imported URL + key only. Review and save; client files are unchanged.",
                );
            }
            Err(e) => view.message(&e),
        });

    let mut view = c.clone();
    let state_save = state.clone();
    let store_save = store.clone();
    let paths_save = paths.clone();
    c.save.set_callback(move |_| {
        let action = (|| -> Result<()> {
            let mut state = state_save.borrow_mut();
            let mut p = Provider::new(
                selected_client(&view),
                &view.name.value(),
                &view.url.value(),
                &view.key.value(),
            )?;
            if let Some(id) = &state.editing {
                p.id.clone_from(id);
            }
            store_save.save(p.clone())?;
            refresh(&mut view, &mut state, &store_save, &paths_save)?;
            let index = state
                .providers
                .iter()
                .filter(|item| item.client == p.client)
                .position(|item| item.id == p.id)
                .unwrap();
            view.list.select(index as i32 + 1);
            fill(&mut view, &p, &mut state);
            Ok(())
        })();
        view.message(match &action {
            Ok(()) => "Saved locally. Client files are unchanged.",
            Err(e) => e,
        });
    });

    let mut view = c.clone();
    let state_apply = state.clone();
    let store_apply = store.clone();
    let paths_apply = paths.clone();
    c.apply.set_callback(move |_| {
        let action = (|| -> Result<()> {
            let mut state = state_apply.borrow_mut();
            let p = state
                .providers
                .iter()
                .find(|p| Some(&p.id) == state.editing.as_ref())
                .cloned()
                .ok_or("Save a provider before switching.")?;
            if p.name != view.name.value().trim()
                || p.base_url != view.url.value().trim()
                || p.api_key != view.key.value().trim()
            {
                return Err("Save your edits before switching this provider.".into());
            }
            environment_conflict(p.client)?;
            paths_apply.apply(&store_apply, &p)?;
            refresh(&mut view, &mut state, &store_apply, &paths_apply)?;
            let index = state
                .providers
                .iter()
                .filter(|item| item.client == p.client)
                .position(|item| item.id == p.id)
                .unwrap();
            view.list.select(index as i32 + 1);
            Ok(())
        })();
        view.message(match &action {
            Ok(()) => {
                "Switched URL + key. Restart the client if it does not reload its configuration."
            }
            Err(e) => e,
        });
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
        let action = (|| -> Result<()> {
            let mut state = state_delete.borrow_mut();
            let id = state
                .editing
                .as_deref()
                .ok_or("Select a saved provider first.")?;
            store_delete.delete(id)?;
            clear(&mut view, &mut state);
            refresh(&mut view, &mut state, &store_delete, &paths_delete)
        })();
        view.message(match &action {
            Ok(()) => "Provider deleted. Client configuration is unchanged.",
            Err(e) => e,
        });
    });
}

enum UpdateEvent {
    Checked(Result<Option<update::Update>>),
    Staged(Result<update::Staged>),
}

fn wire_update(c: &mut Controls) {
    let mut view = c.clone();
    c.update.set_callback(move |_| {
        if view.updating.replace(true) {
            return;
        }
        view.update.deactivate();
        view.update.set_label(tr(view.lang(), "Updating…"));
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
                view.update.activate();
                view.update.set_label(tr(view.lang(), "Check for updates"));
                app::remove_timeout3(handle);
            } else {
                app::repeat_timeout3(0.1, handle);
            }
        });
    });
}

pub fn run() -> Result<()> {
    let store = Store::system()?;
    let paths = Paths::system()?;
    run_with(store, paths)
}

/// The normal application loop, also usable with isolated paths by development tools.
pub fn run_with(store: Store, paths: Paths) -> Result<()> {
    let state = Rc::new(RefCell::new(State {
        providers: vec![],
        editing: None,
    }));
    let mut c = controls(store.language()?);
    refresh(&mut c, &mut state.borrow_mut(), &store, &paths)?;
    clear(&mut c, &mut state.borrow_mut());
    wire(&mut c, state, store, paths);
    c.window.show();
    while app::wait() {}
    Ok(())
}

/// Development-only harness, called by an example rather than the shipped CLI.
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

fn smoke_language(
    store: Store,
    paths: Paths,
    screenshot: Option<std::path::PathBuf>,
    language: Language,
) -> Result<()> {
    let state = Rc::new(RefCell::new(State {
        providers: vec![],
        editing: None,
    }));
    let mut c = controls(store.language()?);
    refresh(&mut c, &mut state.borrow_mut(), &store, &paths)?;
    clear(&mut c, &mut state.borrow_mut());
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    app::check();
    c.name.set_value("Example Provider");
    c.url.set_value("https://api.example.com");
    c.key.set_value("smoke-test-key");
    c.save.do_callback();
    app::check();
    if store.load()?.len() != 1 {
        return Err("GUI save callback failed.".into());
    }
    c.apply.do_callback();
    app::check();
    if paths
        .current(Client::Claude)?
        .as_ref()
        .map(|v| v.0.as_str())
        != Some("https://api.example.com")
    {
        return Err(format!("GUI switch callback failed: {}", c.status.value()));
    }
    c.new.do_callback();
    if !c.key.value().is_empty() {
        return Err("New provider did not clear the credential field.".into());
    }
    c.link.set_value("ccswitch://v1/import?resource=provider&app=claude&name=Imported&endpoint=https%3A%2F%2Fother.example.com&apiKey=imported-test-key&model=ignored");
    c.import.do_callback();
    if c.name.value() != "Imported" || !c.link.value().is_empty() || store.load()?.len() != 1 {
        return Err(
            "GUI import preview changed the store or failed to populate the editor.".into(),
        );
    }
    c.list.select(1);
    c.list.do_callback();
    app::check();
    if c.name.value() != "Example Provider" {
        return Err("GUI list selection failed.".into());
    }
    // Exercise the real locale callback with an unsaved draft and a selected row.
    c.url.set_value("https://edited.example.com");
    let selected = state.borrow().editing.clone();
    let next = Language::ALL[(language.index() + 1) % Language::ALL.len()];
    c.language.set_value(next.index() as i32);
    c.language.do_callback();
    if c.url.value() != "https://edited.example.com"
        || c.key.value() != "smoke-test-key"
        || state.borrow().editing != selected
        || store.language()? != next
    {
        return Err("GUI language switch changed unsaved edits or failed to persist.".into());
    }
    c.message("Save your edits before switching this provider.");
    c.language.set_value(language.index() as i32);
    c.language.do_callback();
    if c.status.value() != tr(language, "Save your edits before switching this provider.") {
        return Err("GUI language switch did not translate the existing status.".into());
    }
    c.url.set_value("https://api.example.com");
    c.message("Switched URL + key. Restart the client if it does not reload its configuration.");
    c.window.redraw();
    app::check();
    if let Some(path) = screenshot {
        // PPM keeps screenshot capture independent of image features in the application.
        let capture = fltk::draw::capture_window(&mut c.window)
            .map_err(|_| "Cannot capture GUI screenshot.")?;
        let data = capture.to_rgb_data();
        use std::io::Write;
        let mut file = std::fs::File::create(path).map_err(|_| "Cannot create screenshot.")?;
        write!(file, "P6\n{} {}\n255\n", capture.data_w(), capture.data_h())
            .map_err(|_| "Cannot write screenshot header.")?;
        file.write_all(&data)
            .map_err(|_| "Cannot write screenshot pixels.")?;
    }
    c.window.hide();
    app::check();
    if app::first_window().is_some() {
        return Err("Window remains after closing.".into());
    }
    Ok(())
}
