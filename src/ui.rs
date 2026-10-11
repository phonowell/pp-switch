use crate::{
    Result,
    client::{Paths, environment_conflict},
    provider::{Client, Provider, import_link},
    storage::Store,
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
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
struct Controls {
    window: Window,
    client: Choice,
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
    status: MultilineOutput,
}

struct State {
    providers: Vec<Provider>,
    editing: Option<String>,
}

fn label(x: i32, y: i32, width: i32, text: &str) {
    let mut label = Frame::new(x, y, width, 20, text);
    label.set_align(Align::Left | Align::Inside);
    label.set_label_size(12);
    label.set_label_color(Color::from_rgb(86, 91, 90));
}

fn field<T: InputExt + WidgetExt>(input: &mut T) {
    input.set_frame(FrameType::BorderBox);
    input.set_color(Color::White);
    input.set_text_size(14);
}

fn button(button: &mut Button, primary: bool) {
    button.set_frame(FrameType::FlatBox);
    button.set_color(if primary {
        Color::from_rgb(29, 69, 55)
    } else {
        Color::from_rgb(227, 232, 228)
    });
    button.set_label_color(if primary {
        Color::White
    } else {
        Color::from_rgb(28, 40, 34)
    });
    button.set_label_size(13);
    button.clear_visible_focus();
}

fn controls() -> Controls {
    let _app = app::App::default().with_scheme(app::Scheme::Base);
    app::set_font(Font::Helvetica);
    app::background(247, 248, 245);
    app::foreground(29, 36, 31);
    let window = Window::new(100, 100, 720, 530, "pp-switch · Pure Provider");
    let mut title = Frame::new(24, 17, 280, 32, "pp-switch");
    title.set_label_size(25);
    title.set_label_font(Font::HelveticaBold);
    title.set_align(Align::Left | Align::Inside);
    label(
        24,
        51,
        490,
        "Pure Provider. Just your base URL and API key.",
    );
    let mut version = Frame::new(610, 25, 86, 20, concat!("v", env!("CARGO_PKG_VERSION")));
    version.set_align(Align::Right | Align::Inside);
    version.set_label_size(12);
    let mut client = Choice::new(24, 86, 220, 32, "");
    client.add_choice("Claude Code|Codex|Gemini CLI");
    client.set_value(0);
    client.set_text_size(14);
    let mut new = Button::new(254, 86, 95, 32, "+ New");
    button(&mut new, false);
    let mut list = HoldBrowser::new(24, 133, 250, 271, "");
    list.set_frame(FrameType::BorderBox);
    list.set_color(Color::White);
    list.set_selection_color(Color::from_rgb(221, 237, 223));
    list.set_text_size(14);
    // Provider names are plain text, never FLTK browser formatting directives.
    list.set_format_char('\u{1}');
    label(298, 131, 395, "Provider name");
    let mut name = Input::new(298, 155, 398, 33, "");
    field(&mut name);
    label(298, 199, 395, "Base URL");
    let mut url = Input::new(298, 223, 398, 33, "");
    field(&mut url);
    label(298, 267, 395, "API key");
    let mut key = SecretInput::new(298, 291, 398, 33, "");
    field(&mut key);
    let mut save = Button::new(298, 347, 106, 34, "Save");
    button(&mut save, false);
    let mut apply = Button::new(418, 347, 174, 34, "Switch provider");
    button(&mut apply, true);
    let mut delete = Button::new(606, 347, 90, 34, "Delete");
    button(&mut delete, false);
    label(
        298,
        388,
        397,
        "Saving keeps client files unchanged. Switching updates URL + key.",
    );
    label(24, 419, 670, "Import a cc-switch Provider link");
    let mut link = Input::new(24, 444, 557, 31, "");
    field(&mut link);
    // Links can themselves contain API keys.
    link.set_type(fltk::input::InputType::Secret);
    let mut import = Button::new(593, 444, 103, 31, "Import");
    button(&mut import, false);
    let mut status = MultilineOutput::new(24, 488, 672, 34, "");
    status.set_frame(FrameType::FlatBox);
    status.set_color(Color::from_rgb(247, 248, 245));
    status.set_text_size(11);
    status.set_wrap(true);
    window.end();
    Controls {
        window,
        client,
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
        status,
    }
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
    let message = match environment_conflict(client) {
        Err(e) => e,
        Ok(()) => match current {
            Err(e) => e,
            Ok(None) => "Add a provider or paste a link. Only URL and key will be changed.".into(),
            Ok(Some((url, _))) => format!("Current: {url}  ·  * marks a matching provider"),
        },
    };
    c.status.set_value(&message);
    Ok(())
}

fn wire(c: &mut Controls, state: Rc<RefCell<State>>, store: Store, paths: Paths) {
    let mut view = c.clone();
    let state_new = state.clone();
    c.new.set_callback(move |_| {
        clear(&mut view, &mut state_new.borrow_mut());
        view.status
            .set_value("New provider. Save first, then switch.");
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
            view.status.set_value(&e);
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
                view.status.set_value(
                    "Imported URL + key only. Review and save; client files are unchanged.",
                );
            }
            Err(e) => view.status.set_value(&e),
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
        view.status.set_value(match &action {
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
        view.status.set_value(match &action {
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
            "Delete this saved provider? Client configuration will stay as it is.",
            "Cancel",
            "Delete",
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
        view.status.set_value(match &action {
            Ok(()) => "Provider deleted. Client configuration is unchanged.",
            Err(e) => e,
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
    let mut c = controls();
    refresh(&mut c, &mut state.borrow_mut(), &store, &paths)?;
    clear(&mut c, &mut state.borrow_mut());
    wire(&mut c, state, store, paths);
    c.window.show();
    while app::wait() {}
    Ok(())
}

/// Development-only harness, called by an example rather than the shipped CLI.
pub fn smoke(store: Store, paths: Paths, screenshot: Option<std::path::PathBuf>) -> Result<()> {
    let state = Rc::new(RefCell::new(State {
        providers: vec![],
        editing: None,
    }));
    let mut c = controls();
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
