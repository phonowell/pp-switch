use base64::{Engine, engine::general_purpose};
use pp_switch::{
    client::{CLAUDE_EDITOR_DEFAULT, Paths},
    provider::{Client, Models, Provider, import_link},
    storage::Store,
};
use std::{fs, path::Path};

fn fixture() -> (tempfile::TempDir, Paths, Store) {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::from_home(dir.path().join("home"));
    let store = Store {
        root: dir.path().join("store"),
    };
    (dir, paths, store)
}
fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn provider(client: Client, model: &str) -> Provider {
    let mut p = Provider::new(client, "Fixture", "https://new.example/v1", "new-key").unwrap();
    p.models = Models::from_fields(model, "", "", "");
    p
}
fn claude() -> &'static str {
    "{\r\n  \"model\":\"old-selection\",\r\n  \"env\":{\"ANTHROPIC_BASE_URL\":\"https://old.example\",\"ANTHROPIC_AUTH_TOKEN\":\"old-key\",\"ANTHROPIC_MODEL\":\"old-default\",\"ANTHROPIC_DEFAULT_HAIKU_MODEL\":\"old-haiku\",\"ANTHROPIC_DEFAULT_SONNET_MODEL\":\"old-sonnet\",\"ANTHROPIC_DEFAULT_OPUS_MODEL\":\"old-opus\",\"ANTHROPIC_SMALL_FAST_MODEL\":\"old-fast\",\"CLAUDE_CODE_MAX_CONTEXT_TOKENS\":\"200000\"},\r\n  \"modelPicker\":[{\"id\":\"old-default\",\"behavesAs\":\"sonnet\"}],\r\n  \"hooks\":{\"中文\":[]}\r\n}\r\n"
}
fn codex() -> &'static str {
    "# keep\r\nmodel_provider = 'chosen'\r\nmodel = 'old-model' # keep model comment\r\nmodel_reasoning_effort = 'high'\r\n[model_providers.chosen]\r\nbase_url = 'https://old.example'\r\nexperimental_bearer_token = 'old-key'\r\nwire_api = 'responses'\r\n[model_providers.other]\r\nbase_url = 'https://other.example'\r\n"
}
fn grok() -> &'static str {
    "# keep\r\n[models]\r\ndefault = 'chosen'\r\n[model.chosen]\r\nmodel = 'old-model' # keep model comment\r\nbase_url = 'https://old.example'\r\napi_key = 'old-key'\r\napi_backend = 'openai-responses'\r\ncontext_window = 200000\r\n[model.other]\r\nmodel = 'other-model'\r\nbase_url = 'https://other.example'\r\napi_key = 'other-key'\r\n"
}
fn gemini() -> &'static str {
    "{\r\n\"security\":{\"auth\":{\"selectedType\":\"gemini-api-key\"}},\r\n\"model\":{\"name\":\"old-model\",\"compressionThreshold\":0.5},\r\n\"mcpServers\":{}\r\n}\r\n"
}

#[test]
fn pp18_editor_prefill_is_connection_scoped_read_only_and_keeps_explicit_models() {
    let (_dir, paths, store) = fixture();
    for (client, text) in [
        (Client::Claude, claude()),
        (Client::Codex, codex()),
        (Client::Grok, grok()),
        (
            Client::Gemini,
            "GOOGLE_GEMINI_BASE_URL=https://old.example\nGEMINI_API_KEY=old-key\nGEMINI_MODEL=old-model\n",
        ),
    ] {
        put(&paths.settings(client), text);
        if client == Client::Gemini {
            put(&paths.gemini.join("settings.json"), gemini());
        }
        let before = fs::read(paths.settings(client)).unwrap();
        let p = Provider::new(client, "Existing", "https://old.example", "old-key").unwrap();
        let expected = if client == Client::Claude {
            "old-default"
        } else {
            "old-model"
        };
        assert_eq!(paths.editor_models(&p).model.as_deref(), Some(expected));
        for (url, key) in [
            ("https://different.example", "old-key"),
            ("https://old.example", "rotated-key"),
        ] {
            let changed = Provider::new(client, "Different", url, key).unwrap();
            let suggested = paths.editor_models(&changed);
            if client == Client::Claude {
                assert_eq!(suggested.model.as_deref(), Some(CLAUDE_EDITOR_DEFAULT));
            } else {
                assert!(suggested.is_empty());
            }
        }
        let mut explicit = p.clone();
        explicit.models = Models::from_fields("link-model", "", "", "");
        assert!(paths.editor_models(&explicit) == explicit.models);
        assert_eq!(fs::read(paths.settings(client)).unwrap(), before);
        assert!(store.load().unwrap().is_empty());
        assert!(p.models.is_empty());
    }
}

#[test]
fn pp18_claude_prefill_resolves_tier_alias_and_falls_back_to_product_default() {
    let (_dir, paths, _store) = fixture();
    let p = Provider::new(Client::Claude, "Existing", "https://old.example", "old-key").unwrap();
    for model in ["sonnet", "opus", "haiku"] {
        let text = serde_json::json!({"model": model, "env": {
            "ANTHROPIC_BASE_URL": p.base_url, "ANTHROPIC_AUTH_TOKEN": p.api_key,
            format!("ANTHROPIC_DEFAULT_{}_MODEL", model.to_uppercase()): "compatible-model"
        }})
        .to_string();
        put(&paths.settings(Client::Claude), &text);
        assert_eq!(
            paths.editor_models(&p).model.as_deref(),
            Some("compatible-model")
        );
        assert_eq!(
            fs::read_to_string(paths.settings(Client::Claude)).unwrap(),
            text
        );
    }
    let mut mapped = p.clone();
    mapped.models = Models::from_fields("", "fast", "provider-sonnet", "large");
    assert_eq!(
        paths.editor_models(&mapped).model.as_deref(),
        Some("provider-sonnet")
    );
    for text in [
        r#"{"env":{"ANTHROPIC_BASE_URL":"https://old.example","ANTHROPIC_AUTH_TOKEN":"old-key"}}"#,
        r#"{"model":"invalid model","env":{"ANTHROPIC_BASE_URL":"https://old.example","ANTHROPIC_AUTH_TOKEN":"old-key"}}"#,
        "{",
    ] {
        put(&paths.settings(Client::Claude), text);
        assert_eq!(
            paths.editor_models(&p).model.as_deref(),
            Some(CLAUDE_EDITOR_DEFAULT)
        );
        assert_eq!(
            fs::read_to_string(paths.settings(Client::Claude)).unwrap(),
            text
        );
    }
}

#[test]
fn pp10_combined_claude_status_matches_readers_and_rejects_invalid_connections() {
    let (_dir, paths, _store) = fixture();
    for text in [
        claude(),
        r#"{"env":{"ANTHROPIC_BASE_URL":"https://fixture.example","ANTHROPIC_API_KEY":"fake"}}"#,
    ] {
        put(&paths.settings(Client::Claude), text);
        for desired in [
            Models::default(),
            Models::from_fields("old-default", "", "", ""),
            Models::from_fields("", "old-haiku", "old-sonnet", "old-opus"),
        ] {
            let combined = paths.status(Client::Claude, true).unwrap();
            assert_eq!(combined.connection, paths.current(Client::Claude).unwrap());
            assert_eq!(
                combined.models.unwrap().matches(Client::Claude, &desired),
                paths
                    .current_models(Client::Claude)
                    .unwrap()
                    .matches(Client::Claude, &desired)
            );
        }
        assert!(
            paths
                .status(Client::Claude, false)
                .unwrap()
                .models
                .is_none()
        );
    }
    for text in [
        "{",
        r#"{"env":{"ANTHROPIC_AUTH_TOKEN":"one","ANTHROPIC_API_KEY":"two"}}"#,
        r#"{"env":{"ANTHROPIC_BASE_URL":42}}"#,
    ] {
        put(&paths.settings(Client::Claude), text);
        assert_eq!(
            paths.status(Client::Claude, true).err(),
            paths.current(Client::Claude).err()
        );
        assert!(paths.status(Client::Claude, true).is_err());
    }
}

#[test]
fn pp01_pp18_legacy_records_and_optional_model_saves_do_not_apply() {
    let (_dir, paths, store) = fixture();
    let old = r#"{"id":"old","client":"claude","name":"Old","base_url":"https://old.example","api_key":"fake"}"#;
    let mut p: Provider = serde_json::from_str(old).unwrap();
    assert!(p.models.is_empty());
    assert!(!serde_json::to_string(&p).unwrap().contains("models"));
    p.models = Models::from_fields(" new-default ", " fast-model ", "", "");
    assert_eq!(p.models.model.as_deref(), Some("new-default"));
    store.save(p.clone()).unwrap();
    assert!(store.load().unwrap()[0] == p);
    assert!(!paths.settings(Client::Claude).exists());
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
    }
    p.models = Models::default();
    store.save(p.clone()).unwrap();
    assert!(store.load().unwrap()[0].models.is_empty());
}

#[test]
fn pp18_invalid_models_or_cross_client_mappings_never_write() {
    for model in [
        "".to_owned(),
        "x y".to_owned(),
        "x\ny".to_owned(),
        "x\u{7f}".to_owned(),
        "x".repeat(513),
    ] {
        let (_dir, paths, store) = fixture();
        let mut p = provider(Client::Claude, "");
        p.models.model = Some(model);
        assert!(store.save(p.clone()).is_err());
        assert!(paths.apply(&store, &p).is_err());
        assert!(!paths.settings(Client::Claude).exists());
        assert!(!store.root.join("providers.json").exists());
    }
    for client in [Client::Codex, Client::Gemini, Client::Grok] {
        let p = Provider {
            models: Models::from_fields("valid", "haiku", "", ""),
            ..provider(client, "")
        };
        assert!(p.validate().is_err());
    }
}

#[test]
fn pp02_pp03_pp18_claude_default_replaces_stale_models_and_preserves_other_bytes() {
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Claude);
    put(&file, claude());
    let p = provider(Client::Claude, "new-model[1m]");
    paths.apply(&store, &p).unwrap();
    let expected = claude()
        .replace("https://old.example", &p.base_url)
        .replace("old-key", &p.api_key)
        .replace("\"old-selection\"", "\"new-model[1m]\"")
        .replace(
            "\"ANTHROPIC_MODEL\":\"old-default\"",
            "\"ANTHROPIC_MODEL\":\"new-model[1m]\"",
        )
        .replace("\"old-haiku\"", "\"new-model[1m]\"")
        .replace("\"old-sonnet\"", "\"new-model[1m]\"")
        .replace("\"old-opus\"", "\"new-model[1m]\"")
        .replace("\"old-fast\"", "\"new-model[1m]\"");
    assert_eq!(fs::read_to_string(&file).unwrap(), expected);
    assert!(
        paths
            .current_models(Client::Claude)
            .unwrap()
            .matches(Client::Claude, &p.models)
    );
    let modified = fs::metadata(&file).unwrap().modified().unwrap();
    paths.apply(&store, &p).unwrap();
    assert_eq!(fs::metadata(file).unwrap().modified().unwrap(), modified);
}

#[test]
fn pp18_claude_partial_and_explicit_mappings_preserve_unspecified_destinations() {
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Claude);
    put(&file, claude());
    let mut p = provider(Client::Claude, "");
    p.models = Models::from_fields("", "new-fast", "", "");
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        claude()
            .replace("https://old.example", &p.base_url)
            .replace("old-key", &p.api_key)
            .replace("old-haiku", "new-fast")
            .replace("old-fast", "new-fast")
    );
    p.models = Models::from_fields("new-default", "new-fast", "new-medium", "new-large");
    paths.apply(&store, &p).unwrap();
    assert!(
        paths
            .current_models(Client::Claude)
            .unwrap()
            .matches(Client::Claude, &p.models)
    );
    put(&file, "{}");
    paths.apply(&store, &p).unwrap();
    assert!(
        paths
            .current_models(Client::Claude)
            .unwrap()
            .matches(Client::Claude, &p.models)
    );
    assert!(
        !fs::read_to_string(file)
            .unwrap()
            .contains("ANTHROPIC_SMALL_FAST_MODEL")
    );
}

#[test]
fn pp10_pp18_matching_url_key_does_not_hide_a_different_model_switch() {
    let (_dir, paths, store) = fixture();
    let p = provider(Client::Claude, "new-model");
    put(&paths.settings(Client::Claude), claude());
    paths.apply(&store, &p).unwrap();
    let state = paths.current_models(Client::Claude).unwrap();
    assert!(state.matches(Client::Claude, &p.models));
    assert!(state.matches(Client::Claude, &Models::default()));
    let mut different = p.clone();
    different.models = Models::from_fields("another-model", "", "", "");
    assert!(!state.matches(Client::Claude, &different.models));
    paths.apply(&store, &different).unwrap();
    assert_eq!(
        paths.current(Client::Claude).unwrap().unwrap(),
        (p.base_url, p.api_key)
    );
    assert!(
        paths
            .current_models(Client::Claude)
            .unwrap()
            .matches(Client::Claude, &different.models)
    );
    // A conflicting settings-level selector also keeps the switch available.
    let file = paths.settings(Client::Claude);
    let changed = fs::read_to_string(&file).unwrap().replacen(
        "\"model\":\"another-model\"",
        "\"model\":\"stale-selector\"",
        1,
    );
    put(&file, &changed);
    assert!(
        !paths
            .current_models(Client::Claude)
            .unwrap()
            .matches(Client::Claude, &different.models)
    );
}

#[test]
fn pp04_pp16_pp18_toml_model_switch_keeps_selection_protocol_comments_and_other_profiles() {
    for (client, original) in [(Client::Codex, codex()), (Client::Grok, grok())] {
        let (_dir, paths, store) = fixture();
        let file = paths.settings(client);
        put(&file, original);
        let p = provider(client, "new-model");
        paths.apply(&store, &p).unwrap();
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            original
                .replace("'https://old.example'", "\"https://new.example/v1\"")
                .replace("'old-key'", "\"new-key\"")
                .replace("'old-model'", "\"new-model\"")
        );
        assert!(
            paths
                .current_models(client)
                .unwrap()
                .matches(client, &p.models)
        );
        let modified = fs::metadata(&file).unwrap().modified().unwrap();
        paths.apply(&store, &p).unwrap();
        assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), modified);
        // Leaving model fields blank must preserve client-owned model changes.
        put(&file, original);
        paths.apply(&store, &provider(client, "")).unwrap();
        assert!(
            fs::read_to_string(file)
                .unwrap()
                .contains("model = 'old-model' # keep model comment")
        );
    }
}

#[test]
fn pp04_pp18_codex_can_minimally_add_a_missing_root_model() {
    let (_dir, paths, store) = fixture();
    let original = codex().replace("model = 'old-model' # keep model comment\r\n", "");
    let file = paths.settings(Client::Codex);
    put(&file, &original);
    paths
        .apply(&store, &provider(Client::Codex, "new-model"))
        .unwrap();
    let text = fs::read_to_string(file).unwrap();
    assert!(text.starts_with("model = \"new-model\"\r\n# keep\r\n"));
    assert!(text.contains("model_provider = 'chosen'\r\n"));
}

#[test]
fn pp05_pp09_pp18_gemini_model_switch_preserves_auth_and_recovers_two_files() {
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Gemini);
    let settings = paths.gemini.join("settings.json");
    let env = "# keep\r\nGOOGLE_GEMINI_BASE_URL='https://old.example'\r\nGEMINI_API_KEY='old-key'\r\nGEMINI_MODEL='old-model' # keep\r\nKEEP=yes\r\n";
    put(&file, env);
    put(&settings, gemini());
    let p = provider(Client::Gemini, "new-model");
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        fs::read_to_string(&settings).unwrap(),
        gemini().replace("old-model", "new-model")
    );
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        env.replace("'https://old.example'", "\"https://new.example/v1\"")
            .replace("'old-key'", "\"new-key\"")
            .replace("'old-model'", "\"new-model\"")
    );
    assert!(!store.root.join("pending.json").exists());
    assert!(
        paths
            .current_models(Client::Gemini)
            .unwrap()
            .matches(Client::Gemini, &p.models)
    );
    let times = (
        fs::metadata(&file).unwrap().modified().unwrap(),
        fs::metadata(&settings).unwrap().modified().unwrap(),
    );
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        times,
        (
            fs::metadata(&file).unwrap().modified().unwrap(),
            fs::metadata(&settings).unwrap().modified().unwrap()
        )
    );
    let stale = fs::read_to_string(&settings)
        .unwrap()
        .replace("new-model", "old-model");
    put(&settings, &stale);
    assert!(
        !paths
            .current_models(Client::Gemini)
            .unwrap()
            .matches(Client::Gemini, &p.models)
    );
}

#[test]
fn pp09_pp18_bad_model_destinations_never_partially_switch_url_key() {
    for original in [
        r#"{"model":42}"#,
        r#"{"model":"one","model":"two"}"#,
        r#"{"env":{"ANTHROPIC_MODEL":null}}"#,
        r#"{"env":{"ANTHROPIC_DEFAULT_SONNET_MODEL":{}}}"#,
    ] {
        let (_dir, paths, store) = fixture();
        let file = paths.settings(Client::Claude);
        put(&file, original);
        assert!(
            paths
                .apply(&store, &provider(Client::Claude, "new-model"))
                .is_err()
        );
        assert_eq!(fs::read_to_string(file).unwrap(), original);
        assert!(!store.root.join("pending.json").exists());
    }
    for invalid in [
        gemini().replace("\"old-model\"", "42"),
        "invalid JSON".into(),
    ] {
        let (_dir, paths, store) = fixture();
        let file = paths.settings(Client::Gemini);
        let env = "GOOGLE_GEMINI_BASE_URL='https://old.example'\nGEMINI_API_KEY='old-key'\n";
        put(&file, env);
        put(&paths.gemini.join("settings.json"), &invalid);
        assert!(
            paths
                .apply(&store, &provider(Client::Gemini, "new-model"))
                .is_err()
        );
        assert_eq!(fs::read_to_string(file).unwrap(), env);
    }
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Codex);
    let auth = paths.codex.join("auth.json");
    let original = codex()
        .replace("model = 'old-model'", "model = 42")
        .replace(
            "experimental_bearer_token = 'old-key'\r\n",
            "requires_openai_auth = true\r\n",
        );
    put(&file, &original);
    put(&auth, r#"{"OPENAI_API_KEY":"old-key"}"#);
    assert!(
        paths
            .apply(&store, &provider(Client::Codex, "new-model"))
            .is_err()
    );
    assert_eq!(fs::read_to_string(file).unwrap(), original);
    assert_eq!(
        fs::read_to_string(auth).unwrap(),
        r#"{"OPENAI_API_KEY":"old-key"}"#
    );
}

#[test]
fn pp06_pp18_import_models_are_reviewed_and_direct_values_win() {
    let config = general_purpose::URL_SAFE_NO_PAD.encode(r#"{"model":"settings-model","env":{"ANTHROPIC_BASE_URL":"https://inline.example","ANTHROPIC_AUTH_TOKEN":"fake","ANTHROPIC_MODEL":"inline-default","ANTHROPIC_DEFAULT_HAIKU_MODEL":"inline-fast"},"modelPicker":[{"id":"ignored"}],"apiKeyHelper":"never-run"}"#);
    let p = import_link(&format!("ccswitch://v1/import?resource=provider&app=claude&name=Fixture&model=explicit-model&haikuModel=explicit-fast&config={config}")).unwrap();
    assert_eq!(p.models.model.as_deref(), Some("explicit-model"));
    assert_eq!(p.models.haiku.as_deref(), Some("explicit-fast"));
    let serialized = serde_json::to_string(&p).unwrap();
    assert!(!serialized.contains("modelPicker"));
    assert!(!serialized.contains("apiKeyHelper"));
    let (_dir, paths, store) = fixture();
    store.save(p).unwrap();
    assert!(!paths.settings(Client::Claude).exists());
    for (client, config) in [("codex", codex()), ("grokbuild", grok())] {
        let p = import_link(&format!("ppswitch://v1/import?resource=provider&app={client}&name=Fixture&configFormat=toml&config={}", general_purpose::URL_SAFE_NO_PAD.encode(config))).unwrap();
        assert_eq!(p.models.model.as_deref(), Some("old-model"));
    }
    let p = import_link(&format!("ccswitch://v1/import?resource=provider&app=gemini&name=Fixture&config={}", general_purpose::URL_SAFE_NO_PAD.encode(r#"{"env":{"GOOGLE_GEMINI_BASE_URL":"https://gemini.example","GEMINI_API_KEY":"fake","GEMINI_MODEL":"gemini-custom"}}"#))).unwrap();
    assert_eq!(p.models.model.as_deref(), Some("gemini-custom"));
}

#[test]
fn pp18_process_model_override_blocks_only_explicit_model_writes() {
    const MARKER: &str = "PPS_MODEL_OVERRIDE_FIXTURE";
    if std::env::var_os(MARKER).is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "pp18_process_model_override_blocks_only_explicit_model_writes",
                "--nocapture",
            ])
            .env(MARKER, "1")
            .env("ANTHROPIC_MODEL", "external-model")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stdout)
        );
        return;
    }
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Claude);
    put(&file, claude());
    let error = paths
        .apply(&store, &provider(Client::Claude, "new-model"))
        .unwrap_err();
    assert!(error.contains("ANTHROPIC_MODEL"));
    assert!(!error.contains("external-model"));
    assert_eq!(fs::read_to_string(&file).unwrap(), claude());
    // URL/key-only switching continues to preserve the model and its launch env.
    paths.apply(&store, &provider(Client::Claude, "")).unwrap();
    let mut p = provider(Client::Claude, "");
    p.models = Models::from_fields("", "fast-only", "", "");
    paths.apply(&store, &p).unwrap();
    assert!(
        paths
            .current_models(Client::Claude)
            .unwrap()
            .matches(Client::Claude, &p.models)
    );
}

#[test]
fn pp18_partial_mapping_ignores_unmanaged_model_fields_in_status() {
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Claude);
    put(
        &file,
        r#"{"model":null,"env":{"ANTHROPIC_MODEL":42,"ANTHROPIC_DEFAULT_SONNET_MODEL":false}}"#,
    );
    let mut p = provider(Client::Claude, "");
    p.models = Models::from_fields("", "fast-only", "", "");
    paths.apply(&store, &p).unwrap();
    assert!(
        paths
            .current_models(Client::Claude)
            .unwrap()
            .matches(Client::Claude, &p.models)
    );
    assert!(fs::read_to_string(&file).unwrap().starts_with(
        r#"{"model":null,"env":{"ANTHROPIC_MODEL":42,"ANTHROPIC_DEFAULT_SONNET_MODEL":false"#
    ));
}

#[cfg(unix)]
#[test]
fn pp09_pp18_gemini_model_symlink_is_refused_before_env_changes() {
    use std::os::unix::fs::symlink;
    let (dir, paths, store) = fixture();
    let file = paths.settings(Client::Gemini);
    let settings = paths.gemini.join("settings.json");
    let real = dir.path().join("real-settings.json");
    let original = "GOOGLE_GEMINI_BASE_URL='https://old.example'\nGEMINI_API_KEY='old-key'\n";
    put(&file, original);
    put(&real, gemini());
    symlink(&real, &settings).unwrap();
    assert!(
        paths
            .apply(&store, &provider(Client::Gemini, "new-model"))
            .is_err()
    );
    assert_eq!(fs::read_to_string(file).unwrap(), original);
    assert_eq!(fs::read_to_string(real).unwrap(), gemini());
    assert!(
        fs::symlink_metadata(settings)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn pp06_ccs_tier_only_links_get_a_visible_default_without_losing_mappings() {
    let base = "ccswitch://v1/import?resource=provider&app=claude&name=CCS&endpoint=https%3A%2F%2Fexample.com&apiKey=example-key";
    let p = import_link(&format!(
        "{base}&haikuModel=fast&sonnetModel=main&opusModel=large"
    ))
    .unwrap();
    assert_eq!(p.models.model.as_deref(), Some("main"));
    assert_eq!(p.models.haiku.as_deref(), Some("fast"));
    assert_eq!(p.models.opus.as_deref(), Some("large"));
    let p = import_link(&format!("{base}&model=explicit&sonnetModel=main")).unwrap();
    assert_eq!(p.models.model.as_deref(), Some("explicit"));
    let json = r#"{"env":{"ANTHROPIC_DEFAULT_SONNET_MODEL":"inline-main","ANTHROPIC_DEFAULT_OPUS_MODEL":"inline-large"}}"#;
    let config = general_purpose::URL_SAFE_NO_PAD.encode(json);
    let p = import_link(&format!("{base}&config={config}")).unwrap();
    assert_eq!(p.models.model.as_deref(), Some("inline-main"));
    let json = r#"{"model":"root-default","env":{"ANTHROPIC_MODEL":"env-default","ANTHROPIC_DEFAULT_SONNET_MODEL":"inline-main"}}"#;
    let p = import_link(&format!(
        "{base}&config={}",
        general_purpose::URL_SAFE_NO_PAD.encode(json)
    ))
    .unwrap();
    assert_eq!(p.models.model.as_deref(), Some("env-default"));
    let p = import_link(base).unwrap();
    assert!(p.models.is_empty());
}
