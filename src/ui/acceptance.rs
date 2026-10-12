//! Development acceptance through native input. No real configuration or keys.
use super::*;
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub enum NativeKey<'a> {
    Escape,
    Space,
    Tab,
    BackTab,
    Down,
    Up,
    Right,
    Enter,
    SelectAll,
    Backspace,
    Text(&'a str),
}
pub trait InputDriver {
    fn appearance(&self, dark: Option<bool>);
    fn menu_has_shadow(&self, window: &dyn WindowExt) -> bool;
    fn menu_is_rounded(&self, window: &dyn WindowExt) -> bool;
    fn title_icon_visible(&self, window: &Window) -> bool;
    fn capture_chrome(&self, window: &dyn WindowExt, path: &Path) -> Result<()>;
    fn mouse(&self, window: &Window, x: i32, y: i32, down: bool);
    fn key(&self, window: Option<&Window>, key: NativeKey<'_>);
    fn close(&self, window: &Window);
    fn visible_windows(&self) -> i32;
}

fn pump() {
    let deadline = Instant::now() + Duration::from_millis(65);
    while Instant::now() < deadline {
        let _ = app::wait_for(0.005);
    }
}
fn click(driver: &dyn InputDriver, window: &Window, widget: &impl WidgetExt) {
    driver.mouse(
        window,
        widget.x() + widget.w() / 2,
        widget.y() + widget.h() / 2,
        true,
    );
    driver.mouse(
        window,
        widget.x() + widget.w() / 2,
        widget.y() + widget.h() / 2,
        false,
    );
    pump();
    // A popup or message callback can outlast the first pump's deadline.
    // Drain the queued release after that nested event loop has returned.
    pump();
}
fn type_field(driver: &dyn InputDriver, window: &Window, widget: &impl WidgetExt, text: &str) {
    click(driver, window, widget);
    driver.key(Some(window), NativeKey::SelectAll);
    pump();
    if text.is_empty() {
        driver.key(Some(window), NativeKey::Backspace);
        pump();
    } else {
        for character in text.chars() {
            let mut bytes = [0; 4];
            driver.key(
                Some(window),
                NativeKey::Text(character.encode_utf8(&mut bytes)),
            );
            pump();
        }
    }
}
fn require(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn initial_focus(c: &Controls) -> Result<()> {
    require(
        app::focus().is_none_or(|w| w.as_widget_ptr() == c.window.as_widget_ptr()),
        "Initial activation focused a control instead of only the main window.",
    )?;
    no_tooltips(&c.window)?;
    no_tooltips(&c.editor)
}
fn snapshot<W: WindowExt>(window: &mut W, path: &Path) -> Result<()> {
    require(
        window.shown(),
        &format!("Cannot capture hidden window: {}", path.display()),
    )?;
    app::flush();
    require(window.shown(), "Capture window was hidden during redraw.")?;
    window.make_current();
    let image =
        draw::capture_window(window).map_err(|_| "Cannot capture input acceptance window.")?;
    use std::io::Write;
    let mut file = std::fs::File::create(path).map_err(|_| "Cannot create input evidence.")?;
    write!(file, "P6\n{} {}\n255\n", image.data_w(), image.data_h())
        .map_err(|_| "Cannot write input evidence.")?;
    file.write_all(&image.to_rgb_data())
        .map_err(|_| "Cannot write input pixels.".into())
}

fn badge_reference(language: Language, output: &Path) -> Result<()> {
    let mut window = Window::new(100, 100, 100, 69, "Badge reference");
    window.set_color(theme::selected());
    let mut label = Frame::new(0, 0, 100, 69, "");
    label.draw(move |f| {
        draw::set_draw_color(theme::active_badge());
        draw::set_font(Font::Helvetica, 10);
        draw::draw_text2(
            tr(language, "In use"),
            f.x(),
            f.y(),
            f.w(),
            f.h(),
            Align::Right | Align::Inside,
        );
    });
    window.end();
    window.show();
    pump();
    let result = snapshot(
        &mut window,
        &output.join(format!("{}-badge-reference.ppm", language.code())),
    );
    window.hide();
    Window::delete(window);
    result
}
fn capture_active(path: &Path) -> Result<()> {
    if let Some(mut window) = app::grab() {
        snapshot(&mut window, path)
    } else if let Some(mut window) = app::modal() {
        snapshot(&mut window, path)
    } else if let Some(mut window) = app::first_window() {
        snapshot(&mut window, path)
    } else {
        Err("No active message window.".into())
    }
}

fn dismiss_message(
    driver: Rc<dyn InputDriver>,
    path: std::path::PathBuf,
    accept: bool,
) -> Rc<RefCell<Result<()>>> {
    let result = Rc::new(RefCell::new(Ok(())));
    let captured = result.clone();
    let count = driver.visible_windows();
    app::add_timeout3(0.12, move |_| {
        *captured.borrow_mut() = require(
            driver.visible_windows() > count && app::modal().is_some(),
            "Native action did not show the expected message dialog.",
        )
        .and_then(|()| capture_active(&path));
        if accept {
            driver.key(None, NativeKey::Tab);
            driver.key(None, NativeKey::Space);
        } else {
            driver.key(None, NativeKey::Escape);
        }
    });
    result
}

fn language_popup(
    c: &Controls,
    driver: Rc<dyn InputDriver>,
    path: std::path::PathBuf,
) -> Result<()> {
    let result = Rc::new(RefCell::new(Err("Language popup did not open.".into())));
    let captured = result.clone();
    let input = driver.clone();
    let timer = app::add_timeout3(0.12, move |_| {
        if let Some(mut popup) = app::grab() {
            *captured.borrow_mut() = require(
                input.menu_has_shadow(&popup) && input.menu_is_rounded(&popup),
                "Language popup lost its native rounding/shadow.",
            )
            .and_then(|()| input.capture_chrome(&popup, &path.with_extension("png")))
            .and_then(|()| snapshot(&mut popup, &path));
            input.key(None, NativeKey::Escape);
        }
    });
    let escape = driver.clone();
    let watchdog = app::add_timeout3(1., move |_| {
        if app::grab().is_some() {
            escape.key(None, NativeKey::Escape);
        }
    });
    click(driver.as_ref(), &c.window, &c.languages);
    app::remove_timeout3(timer);
    app::remove_timeout3(watchdog);
    result.borrow().clone()
}

fn no_tooltips(widget: &impl WidgetExt) -> Result<()> {
    require(widget.tooltip().is_none(), "A widget has a hover tooltip.")?;
    if let Some(group) = widget.as_group() {
        for index in 0..group.children() {
            if let Some(child) = group.child(index) {
                no_tooltips(&child)?;
            }
        }
    }
    Ok(())
}

struct Windows {
    main: Window,
    editor: Window,
}
impl Drop for Windows {
    fn drop(&mut self) {
        self.editor.hide();
        self.main.hide();
        Window::delete(self.editor.clone());
        Window::delete(self.main.clone());
    }
}

fn row_action(c: &Controls, index: usize, edit: bool) -> Button {
    let rows = c.rows.borrow();
    if edit {
        rows[index].edit.clone()
    } else {
        rows[index].switch.clone()
    }
}

pub fn run(
    store: Store,
    paths: Paths,
    output: &Path,
    driver: Rc<dyn InputDriver>,
    full_input: bool,
) -> Result<()> {
    if full_input {
        interaction_regression(
            Store {
                root: store.root.join("input"),
            },
            paths.clone(),
            output,
            driver.clone(),
        )?;
    }
    let mut results = vec![];
    for language in Language::ALL {
        let result = visual_locale(
            Store {
                root: store.root.join(language.code()),
            },
            paths.clone(),
            language,
            output,
            driver.clone(),
        );
        results.push(serde_json::json!({"language":language.code(),"scope":"visual capture only","captured":result.is_ok(),"error":result.as_ref().err()}));
        std::fs::write(
            output.join("results.json"),
            serde_json::to_vec_pretty(&results).unwrap(),
        )
        .map_err(|_| "Cannot save style results.")?;
        result?;
        println!(
            "VISUAL {}: main/settings/editor, badge, row actions, overflow; style only",
            language.code()
        );
    }
    Ok(())
}

fn visual_locale(
    store: Store,
    paths: Paths,
    language: Language,
    output: &Path,
    driver: Rc<dyn InputDriver>,
) -> Result<()> {
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(language);
    let _windows = Windows {
        main: c.window.clone(),
        editor: c.editor.clone(),
    };
    let shot = |name: &str| output.join(format!("{}-{name}.ppm", language.code()));
    refresh(&mut c, state.clone(), &store, &paths)?;
    c.window.show();
    pump();
    initial_focus(&c)?;
    require(
        !driver.title_icon_visible(&c.window),
        "macOS title contains an application icon.",
    )?;
    require(
        !c.new.label().contains(['+', '＋']),
        "Add label contains a font plus beside the Lucide icon.",
    )?;
    snapshot(&mut c.window, &shot("empty"))?;
    driver.capture_chrome(
        &c.window,
        &output.join(format!("{}-chrome.png", language.code())),
    )?;
    badge_reference(language, output)?;
    let provider = Provider::new(
        Client::Claude,
        tr(language, "Everyday"),
        "https://api.example.com",
        "visual-fake-key",
    )?;
    for (index, client) in Client::ALL.into_iter().enumerate() {
        let mut sample = provider.clone();
        sample.client = client;
        sample.id = format!("visual-{index}");
        store.save(sample)?;
        c.client.set(client);
        refresh(&mut c, state.clone(), &store, &paths)?;
        snapshot(&mut c.window, &shot(&format!("client-{index}")))?;
    }
    c.client.set(Client::Claude);
    refresh(&mut c, state.clone(), &store, &paths)?;
    snapshot(&mut c.window, &shot("selected"))?;
    state.borrow_mut().current = Some((provider.base_url.clone(), provider.api_key.clone()));
    selection(&mut c, &state.borrow());
    snapshot(&mut c.window, &shot("active"))?;
    open_editor(&mut c, &mut state.borrow_mut(), None);
    pump();
    c.name.set_value(&provider.name);
    c.url.set_value(&provider.base_url);
    c.key.set_value(&provider.api_key);
    snapshot(&mut c.editor, &shot("add"))?;
    open_editor(&mut c, &mut state.borrow_mut(), Some(&provider));
    snapshot(&mut c.editor, &shot("edit"))?;
    close_editor(&mut c, &mut state.borrow_mut());
    for index in 0..7 {
        store.save(Provider::new(
            Client::Claude,
            &format!("Long provider {index} — 長い名前と地址"),
            &format!("https://long.example.com/{}", "path".repeat(24)),
            "visual-overflow-fake",
        )?)?;
    }
    refresh(&mut c, state.clone(), &store, &paths)?;
    state.borrow_mut().current = Some((provider.base_url.clone(), provider.api_key.clone()));
    selection(&mut c, &state.borrow());
    snapshot(&mut c.window, &shot("overflow"))?;
    c.list.scroll_to(0, 24);
    snapshot(&mut c.window, &shot("scrolled"))?;
    c.main_page.hide();
    c.settings_page.show();
    c.window.redraw();
    snapshot(&mut c.window, &shot("settings"))?;
    language_popup(&c, driver.clone(), shot("language-popup"))?;
    c.updating.set(true);
    c.update.deactivate();
    c.translate();
    c.message("Checking for updates…");
    snapshot(&mut c.window, &shot("update-busy"))?;
    c.updating.set(false);
    c.update.activate();
    c.translate();
    c.message("Cannot contact GitHub. Check your connection or try later.");
    snapshot(&mut c.window, &shot("update-error"))?;
    no_tooltips(&c.window)?;
    no_tooltips(&c.editor)?;
    Ok(())
}

/// Chinese native regression for the new navigation and record-specific actions.
pub fn interaction_regression(
    store: Store,
    paths: Paths,
    output: &Path,
    driver: Rc<dyn InputDriver>,
) -> Result<()> {
    store.save_language(Language::Zh)?;
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(Language::Zh);
    let _windows = Windows {
        main: c.window.clone(),
        editor: c.editor.clone(),
    };
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire_with_updates(&mut c, state.clone(), store.clone(), paths.clone(), || {
        std::thread::sleep(Duration::from_millis(400));
        Err("Cannot contact GitHub. Check your connection or try later.".into())
    });
    c.window.show();
    pump();
    initial_focus(&c)?;
    driver.key(Some(&c.window), NativeKey::Tab);
    pump();
    require(
        c.settings.has_focus(),
        "Initial Tab did not focus Settings.",
    )?;
    let count = driver.visible_windows();
    driver.key(Some(&c.window), NativeKey::Space);
    pump();
    require(
        c.settings_page.visible() && !c.main_page.visible() && driver.visible_windows() == count,
        "Settings did not replace content within the main window.",
    )?;
    click(driver.as_ref(), &c.window, &c.back);
    require(
        c.main_page.visible() && !c.settings_page.visible(),
        "Back did not restore the list.",
    )?;
    let before = paths.current(Client::Claude)?;
    // Add duplicates through native controls; saving never applies.
    for expected in 1..=2 {
        click(driver.as_ref(), &c.window, &c.new);
        type_field(driver.as_ref(), &c.editor, &c.name, "Same");
        type_field(driver.as_ref(), &c.editor, &c.url, "https://same.example");
        type_field(driver.as_ref(), &c.editor, &c.key, "fake");
        click(driver.as_ref(), &c.editor, &c.save);
        require(
            store.load()?.len() == expected && paths.current(Client::Claude)? == before,
            "Save deduplicated records or applied client configuration.",
        )?;
    }
    let other = Provider::new(
        Client::Claude,
        "Other",
        "https://other.example",
        "other-fake",
    )?;
    store.save(other)?;
    refresh(&mut c, state.clone(), &store, &paths)?;
    let pick = c.rows.borrow()[2].pick.clone();
    click(driver.as_ref(), &c.window, &pick);
    require(
        paths.current(Client::Claude)? == before,
        "Row selection applied configuration.",
    )?;
    // Switch a different row than the selected one.
    let switch = row_action(&c, 0, false);
    click(driver.as_ref(), &c.window, &switch);
    require(
        paths.current(Client::Claude)? == Some(("https://same.example".into(), "fake".into())),
        "Switch acted on selected row rather than its own row.",
    )?;
    require(
        c.rows
            .borrow()
            .iter()
            .take(2)
            .all(|r| r.active.get() && !r.switch.active()),
        "All matching duplicates must be active and unswitchable.",
    )?;
    require(
        c.rows.borrow()[2].switch.active(),
        "Nonmatching Switch was disabled.",
    )?;
    let current = paths.current(Client::Claude)?;
    for index in 0..2 {
        let id = c.rows.borrow()[index].id.clone();
        let edit = row_action(&c, index, true);
        click(driver.as_ref(), &c.window, &edit);
        require(
            state.borrow().editing.as_ref() == Some(&id),
            "Edit acted on a different record.",
        )?;
        type_field(driver.as_ref(), &c.editor, &c.name, "Discard");
        driver.close(&c.editor);
        pump();
        require(
            !c.editor.shown() && app::modal().is_none() && store.load()?[index].name == "Same",
            "Editor title-close saved a draft or left a modal.",
        )?;
    }
    require(
        paths.current(Client::Claude)? == current,
        "Editing duplicates applied configuration.",
    )?;
    snapshot(&mut c.window, &output.join("zh-duplicate-active.ppm"))?;
    println!(
        "INPUT zh: startup focus/Tab/Space, in-window settings/back, duplicate save, selection-only, record-specific switch/edit passed"
    );
    // Invalid fields keep the editor open and never save.
    click(driver.as_ref(), &c.window, &c.new);
    for (name, url, key) in [
        ("", "https://safe.example", "fake"),
        ("Valid", "file:///tmp/fake", "fake"),
        ("Valid", "https://safe.example", ""),
    ] {
        c.name.set_value(name);
        c.url.set_value(url);
        c.key.set_value(key);
        let evidence = dismiss_message(driver.clone(), output.join("zh-validation.ppm"), false);
        click(driver.as_ref(), &c.editor, &c.save);
        evidence.borrow().clone()?;
        require(
            c.editor.shown()
                && store.load()?.len() == 3
                && paths.current(Client::Claude)? == current,
            "Invalid Save lost its draft or wrote files.",
        )?;
    }
    click(driver.as_ref(), &c.editor, &c.cancel);
    // Editing changes saved data only, and Cancel preserves it.
    let edit = row_action(&c, 2, true);
    click(driver.as_ref(), &c.window, &edit);
    type_field(driver.as_ref(), &c.editor, &c.name, "Edited");
    click(driver.as_ref(), &c.editor, &c.save);
    require(
        store.load()?[2].name == "Edited" && paths.current(Client::Claude)? == current,
        "Edit Save applied or failed.",
    )?;
    let edit = row_action(&c, 2, true);
    click(driver.as_ref(), &c.window, &edit);
    let evidence = dismiss_message(driver.clone(), output.join("zh-delete-cancel.ppm"), false);
    click(driver.as_ref(), &c.editor, &c.delete);
    evidence.borrow().clone()?;
    require(
        c.editor.shown() && store.load()?.len() == 3,
        "Delete Cancel deleted a row.",
    )?;
    let evidence = dismiss_message(driver.clone(), output.join("zh-delete-confirm.ppm"), true);
    click(driver.as_ref(), &c.editor, &c.delete);
    evidence.borrow().clone()?;
    require(
        !c.editor.shown() && store.load()?.len() == 2 && paths.current(Client::Claude)? == current,
        "Delete wrote client files or failed.",
    )?;
    // Real tab routing and row Switch for each supported client.
    std::fs::create_dir_all(&paths.codex).map_err(|_| "Cannot initialize Codex fixture.")?;
    std::fs::write(paths.codex.join("config.toml"),"# preserve\nmodel_provider='fixture'\n[model_providers.fixture]\nbase_url='https://old.example'\nexperimental_bearer_token='old-fake'\nwire_api='responses'\n").map_err(|_|"Cannot write fixture.")?;
    std::fs::create_dir_all(&paths.gemini).map_err(|_| "Cannot initialize Gemini fixture.")?;
    std::fs::write(
        paths.gemini.join("settings.json"),
        r#"{"security":{"auth":{"selectedType":"gemini-api-key"}},"fixture":true}"#,
    )
    .map_err(|_| "Cannot write fixture.")?;
    for index in [1, 2] {
        store.save(Provider::new(
            Client::ALL[index],
            "Fixture",
            "https://fixture.example",
            "fixture-fake",
        )?)?;
        click(driver.as_ref(), &c.window, &c.clients[index]);
        require(
            c.client.get() == Client::ALL[index] && c.rows.borrow().len() == 1,
            "Client tab routed incorrectly.",
        )?;
        let switch = row_action(&c, 0, false);
        click(driver.as_ref(), &c.window, &switch);
        require(
            paths.current(Client::ALL[index])?
                == Some(("https://fixture.example".into(), "fixture-fake".into())),
            "Client row Switch failed.",
        )?;
    }
    click(driver.as_ref(), &c.window, &c.clients[0]);
    for index in 0..7 {
        store.save(Provider::new(
            Client::Claude,
            &format!("Long provider {index} — 長い名前と地址"),
            "https://overflow.example/path",
            "overflow-fake",
        )?)?;
    }
    refresh(&mut c, state.clone(), &store, &paths)?;
    let scrollbar = c.list.scrollbar();
    driver.mouse(
        &c.window,
        scrollbar.x() + scrollbar.w() / 2,
        scrollbar.y() + scrollbar.h() - 6,
        true,
    );
    driver.mouse(
        &c.window,
        scrollbar.x() + scrollbar.w() / 2,
        scrollbar.y() + scrollbar.h() - 6,
        false,
    );
    pump();
    require(c.list.yposition() > 0, "Native scrollbar did not scroll.")?;
    let visible = c
        .rows
        .borrow()
        .iter()
        .rposition(|row| {
            row.edit.y() >= c.list.y() && row.edit.y() + row.edit.h() <= c.list.y() + c.list.h()
        })
        .ok_or("No scrolled action is fully visible.")?;
    let edit = row_action(&c, visible, true);
    click(driver.as_ref(), &c.window, &edit);
    require(c.editor.shown(), "Scrolled Edit is unreachable.")?;
    click(driver.as_ref(), &c.editor, &c.cancel);
    c.list.scroll_to(0, 0);
    println!(
        "INPUT zh: validation, save/cancel, delete confirm/cancel, three clients and overflow actions passed"
    );
    click(driver.as_ref(), &c.window, &c.settings);
    // Actual Choice popup navigation, rather than callback invocation.
    for (key, expected) in [
        (NativeKey::Up, Language::En),
        (NativeKey::Down, Language::Zh),
    ] {
        let input = driver.clone();
        let timer = app::add_timeout3(0.12, move |_| {
            input.key(None, key);
            input.key(None, NativeKey::Enter);
        });
        let escape = driver.clone();
        let watchdog = app::add_timeout3(1.0, move |_| {
            if app::grab().is_some() {
                escape.key(None, NativeKey::Escape);
            }
        });
        click(driver.as_ref(), &c.window, &c.languages);
        app::remove_timeout3(timer);
        app::remove_timeout3(watchdog);
        require(
            c.lang() == expected && store.language()? == expected,
            "Settings language selector did not persist native selection.",
        )?;
    }
    let unchanged = store.load()?;
    require(!c.updating.get(), "Update started without user action.")?;
    click(driver.as_ref(), &c.window, &c.update);
    require(
        c.updating.get() && !c.update.active(),
        "Update did not start or failed to disable action.",
    )?;
    // Navigation remains responsive during the asynchronous worker.
    click(driver.as_ref(), &c.window, &c.back);
    click(driver.as_ref(), &c.window, &c.settings);
    let deadline = Instant::now() + Duration::from_secs(2);
    while c.updating.get() && Instant::now() < deadline {
        pump();
    }
    require(
        !c.updating.get()
            && c.update.active()
            && c.raw_status.borrow().contains("Cannot contact GitHub"),
        "Update error did not restore settings action/status.",
    )?;
    require(
        store.load()? == unchanged && paths.current(Client::Claude)? == current,
        "Settings/update wrote provider/client data.",
    )?;
    no_tooltips(&c.window)?;
    no_tooltips(&c.editor)?;
    // Both native modal dismissal routes remain responsive; unknown hang trigger remains unresolved.
    for title_close in [false, true] {
        let input = driver.clone();
        app::add_timeout3(0.12, move |_| {
            if let Some(modal) = app::modal() {
                let modal: Window = unsafe { modal.into_widget() };
                if title_close {
                    input.close(&modal);
                } else {
                    input.key(Some(&modal), NativeKey::Escape);
                }
            }
        });
        i18n::alert(Language::Zh, "Select a saved provider first.");
        pump();
        require(app::modal().is_none(), "Message dismissal left a modal.")?;
    }
    driver.close(&c.window);
    pump();
    require(
        !c.window.shown() && app::modal().is_none(),
        "Main close failed.",
    )?;
    println!(
        "INPUT zh: settings language, async update/error/navigation, no hover tips, modal Escape/title-close and main close passed"
    );
    std::fs::write(
        output.join("interaction-results.json"),
        "{\"language\":\"zh\",\"scope\":\"new UI native input\",\"passed\":true}\n",
    )
    .map_err(|_| "Cannot save input results.")?;
    Ok(())
}

/// Bounded cross-platform callback check of changed controls; does not certify native input.
pub fn callback_smoke(
    store: Store,
    paths: Paths,
    screenshot: Option<std::path::PathBuf>,
) -> Result<()> {
    store.save_language(Language::Zh)?;
    let state = Rc::new(RefCell::new(State::default()));
    let mut c = controls(Language::Zh);
    let _windows = Windows {
        main: c.window.clone(),
        editor: c.editor.clone(),
    };
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire(&mut c, state.clone(), store.clone(), paths.clone());
    c.window.show();
    pump();
    c.new.do_callback();
    c.name.set_value("Fixture");
    c.url.set_value("https://fixture.example");
    c.key.set_value("fake");
    let before = paths.current(Client::Claude)?;
    c.save.do_callback();
    require(
        store.load()?.len() == 1 && paths.current(Client::Claude)? == before,
        "Callback Save applied.",
    )?;
    row_action(&c, 0, false).do_callback();
    require(
        c.rows.borrow()[0].active.get() && !c.rows.borrow()[0].switch.active(),
        "Callback Switch failed.",
    )?;
    row_action(&c, 0, true).do_callback();
    c.name.set_value("Draft");
    let selected = state.borrow().selected.clone();
    c.languages.set_value(Language::Ja.index() as i32);
    c.languages.do_callback();
    require(
        c.name.value() == "Draft"
            && state.borrow().selected == selected
            && store.language()? == Language::Ja,
        "Language lost draft or selection.",
    )?;
    c.languages.set_value(Language::Zh.index() as i32);
    c.languages.do_callback();
    c.cancel.do_callback();
    c.settings.do_callback();
    require(
        c.settings_page.visible() && !c.main_page.visible(),
        "Settings routing failed.",
    )?;
    c.back.do_callback();
    no_tooltips(&c.window)?;
    no_tooltips(&c.editor)?;
    if let Some(path) = screenshot {
        snapshot(&mut c.window, &path)?;
    }
    Ok(())
}

pub fn appearance_regression(
    store: Store,
    paths: Paths,
    output: &Path,
    driver: Rc<dyn InputDriver>,
) -> Result<()> {
    driver.appearance(Some(false));
    let provider = Provider::new(
        Client::Claude,
        "Theme fixture",
        "https://theme.example",
        "theme-fake",
    )?;
    store.save(provider.clone())?;
    let state = Rc::new(RefCell::new(State {
        selected: Some(provider.id.clone()),
        ..State::default()
    }));
    let mut c = controls(Language::Zh);
    let _windows = Windows {
        main: c.window.clone(),
        editor: c.editor.clone(),
    };
    refresh(&mut c, state.clone(), &store, &paths)?;
    wire_with_updates(&mut c, state.clone(), store.clone(), paths.clone(), || {
        Ok(None)
    });
    c.window.show();
    pump();
    require(
        !theme::is_dark(),
        "Native light appearance not detected on startup.",
    )?;
    let edit = c.rows.borrow()[0].edit.clone();
    click(driver.as_ref(), &c.window, &edit);
    type_field(driver.as_ref(), &c.editor, &c.name, "Unsaved theme draft");
    let selection = state.borrow().selected.clone();
    let current = paths.current(Client::Claude)?;
    let (position, mark) = (c.name.position(), c.name.mark());
    for dark in [true, false, true] {
        let before = c.window.color().to_rgb();
        driver.appearance(Some(dark));
        pump();
        require(
            theme::is_dark() == dark && c.window.color().to_rgb() != before,
            "Live native appearance did not recolor an existing window.",
        )?;
        require(
            c.name.value() == "Unsaved theme draft"
                && c.name.position() == position
                && c.name.mark() == mark
                && c.name.has_focus()
                && c.key.value() == "theme-fake"
                && state.borrow().selected == selection,
            "Appearance change lost draft, cursor, focus or selection.",
        )?;
        require(
            c.name.color() == theme::field_fill()
                && c.name.text_color() == theme::ink()
                && c.status.label_color() == theme::muted()
                && c.languages.text_color() == theme::ink(),
            "Field/status/language uses a fixed color outside the palette.",
        )?;
        snapshot(
            &mut c.editor,
            &output.join(if dark {
                "zh-live-dark-editor.ppm"
            } else {
                "zh-live-light-editor.ppm"
            }),
        )?;
        snapshot(
            &mut c.window,
            &output.join(if dark {
                "zh-live-dark-main.ppm"
            } else {
                "zh-live-light-main.ppm"
            }),
        )?;
    }
    require(
        store.load()?.len() == 1
            && store.load()?[0].name == "Theme fixture"
            && paths.current(Client::Claude)? == current,
        "Appearance changes wrote a draft or client configuration.",
    )?;
    driver.close(&c.editor);
    pump();
    require(
        !c.editor.shown() && app::modal().is_none(),
        "Appearance change broke modal close.",
    )?;
    println!(
        "APPEARANCE zh: native light/dark/light/dark, existing windows, draft/cursor/focus/selection and modal close passed"
    );
    Ok(())
}
