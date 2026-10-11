use base64::{Engine, engine::general_purpose};
use pp_switch::{
    client::Paths,
    config,
    provider::{Client, Provider, import_link},
    storage::{self, Change, Store},
};
use std::{fs, path::Path};

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn provider(client: Client) -> Provider {
    Provider::new(client, "测试 Provider", "https://new.example/v1", "new-key").unwrap()
}
fn setup() -> (tempfile::TempDir, Paths, Store) {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::from_home(dir.path().join("中文 home"));
    let store = Store {
        root: dir.path().join("store"),
    };
    (dir, paths, store)
}

#[test]
fn pp01_local_provider_crud_does_not_touch_client_config() {
    let (_dir, paths, store) = setup();
    let mut p = provider(Client::Claude);
    store.save(p.clone()).unwrap();
    assert_eq!(store.load().unwrap().len(), 1);
    p.name = "Renamed".into();
    store.save(p.clone()).unwrap();
    assert_eq!(store.load().unwrap()[0].name, "Renamed");
    store.delete(&p.id).unwrap();
    assert!(store.load().unwrap().is_empty());
    assert!(!paths.settings(Client::Claude).exists());
}

#[test]
fn pp01_validation_rejects_control_characters_and_embedded_credentials() {
    assert!(Provider::new(Client::Claude, "", "https://example.com", "key").is_err());
    assert!(Provider::new(Client::Claude, "test", "file:///tmp/a", "key").is_err());
    assert!(
        Provider::new(
            Client::Claude,
            "test",
            "https://user:pass@example.com",
            "key"
        )
        .is_err()
    );
    assert!(Provider::new(Client::Claude, "test", "https://example.com", "key\nextra").is_err());
}

#[test]
fn pp02_pp03_claude_preserves_unrelated_bytes_and_is_idempotent() {
    let (_dir, paths, store) = setup();
    let original = "{\r\n  \"model\" : \"my-model\",\r\n  \"env\": {\"ANTHROPIC_BASE_URL\" : \"https://old.example\", \"ANTHROPIC_AUTH_TOKEN\":\"old-key\", \"KEEP\":\"a } brace\"},\r\n  \"hooks\" : {\"中文\": [1,2]}\r\n}\r\n";
    let file = paths.settings(Client::Claude);
    put(&file, original);
    let p = provider(Client::Claude);
    paths.apply(&store, &p).unwrap();
    let expected = original
        .replace("https://old.example", &p.base_url)
        .replace("old-key", &p.api_key);
    assert_eq!(fs::read_to_string(&file).unwrap(), expected);
    let modified = fs::metadata(&file).unwrap().modified().unwrap();
    paths.apply(&store, &p).unwrap();
    assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), modified);
    assert_eq!(
        paths.current(Client::Claude).unwrap().unwrap().0,
        p.base_url
    );
}

#[test]
fn pp03_claude_uses_existing_api_key_field_and_can_add_missing_containers() {
    let (_dir, paths, store) = setup();
    let file = paths.settings(Client::Claude);
    put(
        &file,
        "{\"env\":{\"ANTHROPIC_API_KEY\":\"old\"},\"permissions\":{\"allow\":[]}}",
    );
    paths.apply(&store, &provider(Client::Claude)).unwrap();
    let text = fs::read_to_string(file).unwrap();
    assert!(!text.contains("AUTH_TOKEN"));
    assert!(text.contains("\"permissions\":{\"allow\":[]}"));
    let out = config::patch_json(
        "{\"other\":true}\n",
        &["env", "ANTHROPIC_BASE_URL"],
        "https://x.example",
    )
    .unwrap();
    assert_eq!(
        config::json_string(&out, &["env", "ANTHROPIC_BASE_URL"])
            .unwrap()
            .unwrap(),
        "https://x.example"
    );
    assert!(out.starts_with("{\"other\":true"));
    assert!(out.ends_with('\n'));
}

#[test]
fn pp03_unsupported_claude_modes_never_write() {
    for text in [
        "{\"env\":{\"ANTHROPIC_AUTH_TOKEN\":\"a\",\"ANTHROPIC_API_KEY\":\"b\"}}",
        "{\"apiKeyHelper\":\"external\"}",
        "{\"env\":{\"CLAUDE_CODE_USE_BEDROCK\":\"1\"}}",
        "{\"env\":null}",
        "{\"env\":{},\"env\":{}}",
        "invalid JSON",
    ] {
        let (_dir, paths, store) = setup();
        let file = paths.settings(Client::Claude);
        put(&file, text);
        assert!(paths.apply(&store, &provider(Client::Claude)).is_err());
        assert_eq!(fs::read_to_string(file).unwrap(), text);
    }
}

fn codex_config() -> &'static str {
    "# keep\r\nmodel_provider = 'active'\r\nmodel = \"my-model\"\r\nmodel_reasoning_effort = 'medium'\r\n\r\n[model_providers.stale]\r\nbase_url = 'https://stale.example'\r\n\r\n[model_providers.active]\r\nname = 'My API'\r\nbase_url = 'https://old.example/v1' # keep comment\r\nwire_api = 'responses'\r\nrequires_openai_auth = true\r\n"
}

#[test]
fn pp02_pp04_codex_changes_only_selected_url_and_auth_key() {
    let (_dir, paths, store) = setup();
    let file = paths.settings(Client::Codex);
    let auth = paths.codex.join("auth.json");
    let text = codex_config();
    let auth_text = "{ \"OPENAI_API_KEY\" : \"old-key\", \"keep\": 9 }\n";
    put(&file, text);
    put(&auth, auth_text);
    let p = provider(Client::Codex);
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        text.replace("'https://old.example/v1'", "\"https://new.example/v1\"")
    );
    assert_eq!(
        fs::read_to_string(&auth).unwrap(),
        auth_text.replace("old-key", "new-key")
    );
    assert_eq!(paths.current(Client::Codex).unwrap().unwrap().0, p.base_url);
    assert!(!store.root.join("pending.json").exists());
}

#[test]
fn pp04_codex_bearer_key_preserves_oauth_cache() {
    let (_dir, paths, store) = setup();
    let file = paths.settings(Client::Codex);
    let auth = paths.codex.join("auth.json");
    let text = "model_provider='custom'\n[model_providers.custom]\nbase_url='https://old.example'\nexperimental_bearer_token='old-key'\nrequires_openai_auth=false\n";
    let oauth = "{\"tokens\":{\"access_token\":\"untouched\"}}";
    put(&file, text);
    put(&auth, oauth);
    paths.apply(&store, &provider(Client::Codex)).unwrap();
    assert_eq!(fs::read_to_string(auth).unwrap(), oauth);
    assert_eq!(
        fs::read_to_string(file).unwrap(),
        text.replace("'https://old.example'", "\"https://new.example/v1\"")
            .replace("'old-key'", "\"new-key\"")
    );
}

#[test]
fn pp04_codex_null_oauth_placeholders_are_preserved_in_api_key_mode() {
    let (_dir, paths, store) = setup();
    let auth = paths.codex.join("auth.json");
    let text = "{\"auth_mode\":\"api_key\",\"OPENAI_API_KEY\":\"old-key\",\"tokens\":null,\"last_refresh\":null}";
    put(&paths.settings(Client::Codex), codex_config());
    put(&auth, text);
    paths.apply(&store, &provider(Client::Codex)).unwrap();
    assert_eq!(
        fs::read_to_string(auth).unwrap(),
        text.replace("old-key", "new-key")
    );
}

#[test]
fn pp04_codex_unsupported_auth_or_selector_never_writes() {
    for (text, auth) in [
        ("model='x'", "{}"),
        (
            codex_config(),
            "{\"tokens\":{\"access_token\":\"oauth\"},\"OPENAI_API_KEY\":\"key\"}",
        ),
        (codex_config(), "{}"),
        (codex_config(), "invalid JSON"),
    ] {
        let (_dir, paths, store) = setup();
        let file = paths.settings(Client::Codex);
        let auth_path = paths.codex.join("auth.json");
        put(&file, text);
        put(&auth_path, auth);
        assert!(paths.apply(&store, &provider(Client::Codex)).is_err());
        assert_eq!(fs::read_to_string(file).unwrap(), text);
        assert_eq!(fs::read_to_string(auth_path).unwrap(), auth);
    }
    for suffix in [
        "env_key='OPENAI_API_KEY'\n",
        "experimental_bearer_token='x'\n",
    ] {
        let (_dir, paths, store) = setup();
        let file = paths.settings(Client::Codex);
        let text = format!("{}{suffix}", codex_config());
        put(&file, &text);
        assert!(paths.apply(&store, &provider(Client::Codex)).is_err());
        assert_eq!(fs::read_to_string(file).unwrap(), text);
    }
}

#[test]
fn pp02_pp05_gemini_preserves_settings_and_dotenv_comments() {
    let (_dir, paths, store) = setup();
    let file = paths.settings(Client::Gemini);
    let settings = paths.gemini.join("settings.json");
    let settings_text = "{\"security\":{\"auth\":{\"selectedType\":\"gemini-api-key\"}},\"model\":{\"name\":\"keep\"}}\n";
    let env = "# keep this\r\n export GEMINI_API_KEY = 'old-key' # credential\r\nGOOGLE_GEMINI_BASE_URL=\"https://old.example\"  # address\r\nGEMINI_MODEL=keep\r\n";
    put(&settings, settings_text);
    put(&file, env);
    paths.apply(&store, &provider(Client::Gemini)).unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        env.replace("'old-key'", "\"new-key\"")
            .replace("https://old.example", "https://new.example/v1")
    );
    assert_eq!(fs::read_to_string(settings).unwrap(), settings_text);
    assert_eq!(paths.current(Client::Gemini).unwrap().unwrap().1, "new-key");
}

#[test]
fn pp05_gemini_oauth_and_duplicate_env_are_refused_without_writes() {
    let (_dir, paths, store) = setup();
    let file = paths.settings(Client::Gemini);
    let settings = paths.gemini.join("settings.json");
    put(&file, "GEMINI_API_KEY='old'\n");
    put(
        &settings,
        "{\"security\":{\"auth\":{\"selectedType\":\"oauth-personal\"}}}",
    );
    assert!(paths.apply(&store, &provider(Client::Gemini)).is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), "GEMINI_API_KEY='old'\n");
    put(
        &settings,
        "{\"security\":{\"auth\":{\"selectedType\":\"gemini-api-key\"}}}",
    );
    put(&file, "GEMINI_API_KEY=a\nGEMINI_API_KEY=b\n");
    assert!(paths.apply(&store, &provider(Client::Gemini)).is_err());
    assert_eq!(
        fs::read_to_string(file).unwrap(),
        "GEMINI_API_KEY=a\nGEMINI_API_KEY=b\n"
    );
}

#[test]
fn pp06_import_explicit_fields_win_and_extras_are_not_retained() {
    let config = general_purpose::URL_SAFE_NO_PAD.encode(r#"{"env":{"ANTHROPIC_BASE_URL":"https://inline.example","ANTHROPIC_AUTH_TOKEN":"inline-key","ANTHROPIC_MODEL":"ignored"},"apiKeyHelper":"never-run"}"#);
    let link = format!(
        "ccswitch://v1/import?resource=provider&app=claude&name=%E6%B5%8B%E8%AF%95&endpoint=https%3A%2F%2Fexplicit.example%2Cv2&apiKey=explicit-key&config={config}&usageScript=ignored&enabled=true"
    );
    let p = import_link(&link).unwrap();
    assert_eq!(p.name, "测试");
    assert_eq!(p.base_url, "https://explicit.example");
    assert_eq!(p.api_key, "explicit-key");
    let serialized = serde_json::to_string(&p).unwrap();
    assert!(!serialized.contains("ignored"));
    assert!(!serialized.contains("apiKeyHelper"));
}

#[test]
fn pp06_import_base64_variants_and_all_supported_clients() {
    let payload = r#"{"env":{"ANTHROPIC_BASE_URL":"https://inline.example","ANTHROPIC_AUTH_TOKEN":"inline-key"}}"#;
    for engine in [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ] {
        let encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("config", &engine.encode(payload))
            .finish();
        let p = import_link(&format!(
            "ppswitch://v1/import?resource=provider&app=claude&name=Test&{encoded}"
        ))
        .unwrap();
        assert_eq!(p.api_key, "inline-key");
    }
    let codex = serde_json::json!({"auth":{"OPENAI_API_KEY":"codex-key"},"config":"model_provider='chosen'\n[model_providers.stale]\nbase_url='https://stale.example'\n[model_providers.chosen]\nbase_url='https://chosen.example'\n"});
    let p = import_link(&format!(
        "ccswitch://v1/import?resource=provider&app=codex&name=Test&config={}",
        general_purpose::URL_SAFE_NO_PAD.encode(codex.to_string())
    ))
    .unwrap();
    assert_eq!(p.base_url, "https://chosen.example");
    assert_eq!(p.api_key, "codex-key");
    let p = import_link(&format!(
        "ccswitch://v1/import?resource=provider&app=gemini&name=Test&config={}",
        general_purpose::URL_SAFE_NO_PAD.encode(
            r#"{"GOOGLE_GEMINI_BASE_URL":"https://gemini.example","GEMINI_API_KEY":"gemini-key"}"#
        )
    ))
    .unwrap();
    assert_eq!(p.api_key, "gemini-key");
    let toml = "model_provider='chosen'\n[model_providers.chosen]\nbase_url='https://chosen.example'\nexperimental_bearer_token='bearer-key'\n";
    let p = import_link(&format!(
        "ccswitch://v1/import?resource=provider&app=codex&name=Test&configFormat=toml&config={}",
        general_purpose::URL_SAFE_NO_PAD.encode(toml)
    ))
    .unwrap();
    assert_eq!(p.api_key, "bearer-key");
}

#[test]
fn pp06_unsupported_or_incomplete_imports_return_no_secret_in_errors() {
    for link in [
        "ccswitch://v2/import?resource=provider",
        "ccswitch://v1/import?resource=mcp",
        "ccswitch://v1/import?resource=provider&app=opencode&name=x",
        "ccswitch://v1/import?resource=provider&app=claude&name=x&configUrl=https://example.com&apiKey=secret-never-print",
        "ccswitch://v1/import?resource=provider&app=claude&name=x&endpoint=file:///tmp/secret-never-print&apiKey=key",
        "ccswitch://v1/import?resource=provider&app=claude&name=x&config=secret-never-print",
    ] {
        let error = import_link(link).err().expect("must reject");
        assert!(!error.contains("secret-never-print"));
    }
}

#[test]
fn pp08_credentials_and_journal_use_private_files() {
    let (_dir, _paths, store) = setup();
    store.save(provider(Client::Claude)).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(store.root.join("providers.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(&store.root).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}

#[test]
fn pp09_stale_snapshot_never_overwrites_external_changes() {
    let (_dir, paths, store) = setup();
    let file = paths.settings(Client::Claude);
    put(&file, "external edit");
    let result = store.commit(vec![Change {
        path: file.clone(),
        before: Some("old".into()),
        after: "replacement".into(),
    }]);
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(file).unwrap(), "external edit");
    assert!(!store.root.join("pending.json").exists());
}

#[test]
fn pp09_restart_recovers_a_partially_published_multi_file_operation() {
    let (_dir, paths, store) = setup();
    let first = paths.codex.join("auth.json");
    let second = paths.codex.join("config.toml");
    put(&first, "new-auth");
    put(&second, "old-config");
    let changes = vec![
        Change {
            path: first.clone(),
            before: Some("old-auth".into()),
            after: "new-auth".into(),
        },
        Change {
            path: second.clone(),
            before: Some("old-config".into()),
            after: "new-config".into(),
        },
    ];
    put(
        &store.root.join("pending.json"),
        &serde_json::json!({"changes":changes}).to_string(),
    );
    store.recover().unwrap();
    assert_eq!(fs::read_to_string(first).unwrap(), "old-auth");
    assert_eq!(fs::read_to_string(second).unwrap(), "old-config");
    assert!(!store.root.join("pending.json").exists());
    store.recover().unwrap();
}

#[test]
fn pp09_recovery_conflict_preserves_every_file_and_journal() {
    let (_dir, paths, store) = setup();
    let first = paths.codex.join("auth.json");
    let second = paths.codex.join("config.toml");
    put(&first, "new-auth");
    put(&second, "external-edit");
    let changes = vec![
        Change {
            path: first.clone(),
            before: Some("old-auth".into()),
            after: "new-auth".into(),
        },
        Change {
            path: second.clone(),
            before: Some("old-config".into()),
            after: "new-config".into(),
        },
    ];
    put(
        &store.root.join("pending.json"),
        &serde_json::json!({"changes":changes}).to_string(),
    );
    assert!(store.recover().is_err());
    assert_eq!(fs::read_to_string(first).unwrap(), "new-auth");
    assert_eq!(fs::read_to_string(second).unwrap(), "external-edit");
    assert!(store.root.join("pending.json").exists());
}

#[test]
fn pp09_lock_prevents_a_second_writer_and_invalid_store_is_not_overwritten() {
    let (_dir, _paths, store) = setup();
    store.recover().unwrap();
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(store.root.join("lock"))
        .unwrap();
    file.lock().unwrap();
    assert!(store.save(provider(Client::Claude)).is_err());
    drop(file);
    put(&store.root.join("providers.json"), "not JSON");
    assert!(store.save(provider(Client::Claude)).is_err());
    assert_eq!(
        storage::read(&store.root.join("providers.json"))
            .unwrap()
            .unwrap(),
        "not JSON"
    );
}

#[test]
fn pp09_every_input_is_validated_before_the_first_write() {
    let (_dir, paths, store) = setup();
    let first = paths.codex.join("auth.json");
    let invalid = paths.codex.join("directory");
    put(&first, "old");
    fs::create_dir_all(&invalid).unwrap();
    assert!(
        store
            .commit(vec![
                Change {
                    path: first.clone(),
                    before: Some("old".into()),
                    after: "new".into()
                },
                Change {
                    path: invalid,
                    before: None,
                    after: "new".into()
                }
            ])
            .is_err()
    );
    assert_eq!(fs::read_to_string(first).unwrap(), "old");
}

#[test]
#[cfg(unix)]
fn pp09_symlink_destinations_are_preserved_without_writing_the_target() {
    use std::os::unix::fs::symlink;
    let (dir, paths, store) = setup();
    let real = dir.path().join("real-settings.json");
    let config = "{\"env\":{\"ANTHROPIC_AUTH_TOKEN\":\"old\"}}";
    put(&real, config);
    let link = paths.settings(Client::Claude);
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    symlink(&real, &link).unwrap();
    assert!(paths.apply(&store, &provider(Client::Claude)).is_err());
    assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_to_string(real).unwrap(), config);
}
