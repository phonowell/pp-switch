use pp_switch::{
    client::Paths,
    config,
    provider::{Client, Models, Provider},
    storage::Store,
};
use std::{fs, path::Path};
fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn fixture() -> (tempfile::TempDir, Paths, Store) {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::from_home(dir.path().join("中文 home"));
    let store = Store {
        root: dir.path().join("store"),
    };
    (dir, paths, store)
}
fn provider(client: Client) -> Provider {
    let mut p = Provider::new(client, "Fixture", "https://relay.example/v1", "fake-key").unwrap();
    p.models = Models::from_fields("compatible-model", "", "", "");
    p
}
fn dsh_setup(paths: &Paths) {
    for name in ["headless", "web"] {
        put(
            &paths.dsh.join("profiles").join(name).join("package.json"),
            r#"{"dsh":{"profile":{"bundles":["@deepseek-ai/dsh-base"]}}}"#,
        );
        put(
            &paths
                .dsh
                .join("profiles")
                .join(name)
                .join("cordis.patch.yml"),
            "# profile stays intact\n[]\n",
        );
    }
}
#[test]
fn pp19_claude_roundtrip_cleans_models_preserves_login_hooks_proxy_and_context() {
    let (_dir, paths, store) = fixture();
    let baseline = "{\r\n  \"env\":{\"HTTPS_PROXY\":\"http://proxy.example\",\"CLAUDE_CODE_MAX_CONTEXT_TOKENS\":\"200000\"},\r\n  \"hooks\":{\"中文\":[1,2]},\r\n  \"modelPicker\":[{\"id\":\"native\"}],\r\n  \"permissions\":{\"allow\":[]}\r\n}\r\n";
    let file = paths.settings(Client::Claude);
    put(&file, baseline);
    put(
        &paths.claude.join(".credentials.json"),
        "native-secret-fixture",
    );
    assert!(paths.status(Client::Claude, true).unwrap().official);
    let p = provider(Client::Claude);
    paths.apply(&store, &p).unwrap();
    assert!(!paths.status(Client::Claude, true).unwrap().official);
    paths.apply_official(&store, Client::Claude).unwrap();
    assert!(paths.status(Client::Claude, true).unwrap().official);
    let after = fs::read_to_string(&file).unwrap();
    assert!(config::json_string(&after, &["model"]).unwrap().is_none());
    for key in [
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_MODEL",
        "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        "ANTHROPIC_DEFAULT_SONNET_MODEL",
        "ANTHROPIC_DEFAULT_OPUS_MODEL",
    ] {
        assert!(
            config::json_string(&after, &["env", key])
                .unwrap()
                .is_none()
        );
    }
    for fragment in [
        "\"hooks\":{\"中文\":[1,2]}",
        "\"modelPicker\":[{\"id\":\"native\"}]",
        "\"permissions\":{\"allow\":[]}",
        "\"HTTPS_PROXY\":\"http://proxy.example\"",
        "\"CLAUDE_CODE_MAX_CONTEXT_TOKENS\":\"200000\"",
    ] {
        assert!(after.contains(fragment));
    }
    assert_eq!(
        fs::read_to_string(paths.claude.join(".credentials.json")).unwrap(),
        "native-secret-fixture"
    );
    let modified = fs::metadata(&file).unwrap().modified().unwrap();
    paths.apply_official(&store, Client::Claude).unwrap();
    assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), modified);
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Claude).unwrap(),
        Some((p.base_url, p.api_key))
    );
}
#[test]
fn pp19_codex_native_bearer_roundtrip_never_loses_oauth_or_other_tables() {
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Codex);
    put(
        &file,
        "# retained\nmodel='gpt-native'\nmodel_reasoning_effort='high'\n[projects.'/fixture']\ntrust_level='trusted'\n[model_providers.other]\nbase_url='https://other.example'\n",
    );
    let auth = "{\r\n\"auth_mode\":\"chatgpt\",\r\n\"tokens\":{\"access_token\":\"fake-native-token\",\"refresh_token\":\"fake-refresh\"},\r\n\"last_refresh\":\"native\"\r\n}\r\n";
    put(&paths.codex.join("auth.json"), auth);
    let p = provider(Client::Codex);
    assert!(paths.is_official(Client::Codex).unwrap());
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Codex).unwrap(),
        Some((p.base_url.clone(), p.api_key.clone()))
    );
    assert_eq!(
        fs::read_to_string(paths.codex.join("auth.json")).unwrap(),
        auth
    );
    paths.apply_official(&store, Client::Codex).unwrap();
    assert!(paths.status(Client::Codex, true).unwrap().official);
    let after = fs::read_to_string(&file).unwrap();
    assert!(after.contains("model_reasoning_effort='high'"));
    assert!(after.contains("base_url='https://other.example'"));
    assert!(!after.contains("model_provider =") && !after.contains("compatible-model"));
    assert_eq!(
        fs::read_to_string(paths.codex.join("auth.json")).unwrap(),
        auth
    );
    // A user-added field on the inactive managed table survives another switch.
    put(
        &file,
        &after.replace(
            "wire_api = \"responses\"",
            "wire_api = \"responses\"\nhttp_max_retries = 2 # user setting",
        ),
    );
    paths.apply(&store, &p).unwrap();
    assert!(
        fs::read_to_string(&file)
            .unwrap()
            .contains("http_max_retries = 2 # user setting")
    );
}
#[test]
fn pp19_codex_official_removes_only_api_auth_preserves_native_tokens() {
    let (_dir, paths, store) = fixture();
    put(
        &paths.settings(Client::Codex),
        "model_provider='chosen'\nmodel='relay-model'\nforced_login_method='api'\n[model_providers.chosen]\nbase_url='https://relay.example'\n",
    );
    let auth = "{\"OPENAI_API_KEY\":\"fake-api\",\"auth_mode\":\"api_key\",\"tokens\":{\"access_token\":\"fake-native\"},\"keep\" : [1,2]}";
    put(&paths.codex.join("auth.json"), auth);
    paths.apply_official(&store, Client::Codex).unwrap();
    let after = fs::read_to_string(paths.codex.join("auth.json")).unwrap();
    assert_eq!(
        after,
        "{\"tokens\":{\"access_token\":\"fake-native\"},\"keep\" : [1,2]}"
    );
    assert!(paths.is_official(Client::Codex).unwrap());
}
#[test]
fn pp19_gemini_roundtrip_changes_mode_preserves_oauth_and_unrelated_bytes() {
    let (_dir, paths, store) = fixture();
    let settings = paths.gemini.join("settings.json");
    put(
        &settings,
        "{\r\n\"security\":{\"auth\":{\"selectedType\":\"oauth-personal\"}},\r\n\"model\":{\"compressionThreshold\":0.5},\r\n\"mcpServers\":{\"keep\":{}}\r\n}\r\n",
    );
    put(
        &paths.settings(Client::Gemini),
        "# keep\r\nOTHER='中文' # comment\r\n",
    );
    put(&paths.gemini.join("oauth_creds.json"), "fake-native-oauth");
    let p = provider(Client::Gemini);
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Gemini).unwrap(),
        Some((p.base_url.clone(), p.api_key.clone()))
    );
    paths.apply_official(&store, Client::Gemini).unwrap();
    assert!(paths.status(Client::Gemini, true).unwrap().official);
    assert_eq!(
        fs::read_to_string(paths.settings(Client::Gemini)).unwrap(),
        "# keep\r\nOTHER='中文' # comment\r\n"
    );
    let after = fs::read_to_string(&settings).unwrap();
    assert_eq!(
        config::json_string(&after, &["security", "auth", "selectedType"])
            .unwrap()
            .as_deref(),
        Some("oauth-personal")
    );
    assert!(
        after.contains("\"compressionThreshold\":0.5")
            && after.contains("\"mcpServers\":{\"keep\":{}}")
    );
    assert_eq!(
        fs::read_to_string(paths.gemini.join("oauth_creds.json")).unwrap(),
        "fake-native-oauth"
    );
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Gemini).unwrap(),
        Some((p.base_url, p.api_key))
    );
}
#[test]
fn pp19_grok_roundtrip_preserves_login_inactive_profiles_and_advanced_settings() {
    let (_dir, paths, store) = fixture();
    let file = paths.settings(Client::Grok);
    put(
        &file,
        "# native\n[models]\nweb_search='grok-native'\n[model.inactive]\nmodel='other'\nbase_url='https://other.example'\napi_key='other-fake'\n",
    );
    put(&paths.grok.join("auth.json"), "native-grok-token-fixture");
    let p = provider(Client::Grok);
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Grok).unwrap(),
        Some((p.base_url.clone(), p.api_key.clone()))
    );
    let before = fs::read_to_string(&file).unwrap();
    put(
        &file,
        &before.replace(
            "default = \"pps\"",
            "default = \"pps\"\nsession_summary = \"pps\"",
        ),
    );
    paths.apply_official(&store, Client::Grok).unwrap();
    assert!(paths.status(Client::Grok, true).unwrap().official);
    let after = fs::read_to_string(&file).unwrap();
    assert!(!after.contains("default =") && !after.contains("session_summary"));
    assert!(
        after.contains("web_search='grok-native'")
            && after.contains("base_url='https://other.example'")
    );
    assert_eq!(
        fs::read_to_string(paths.grok.join("auth.json")).unwrap(),
        "native-grok-token-fixture"
    );
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Grok).unwrap(),
        Some((p.base_url, p.api_key))
    );
}
#[test]
fn pp19_dsh_account_api_roundtrip_keeps_account_records_and_profile_bytes() {
    let (_dir, paths, store) = fixture();
    dsh_setup(&paths);
    let credential = "version: 1\nrefs:\n  DEEPSEEK_API_KEY: old-fake\nrecords:\n  account: {token: native-fake, expires: 123}\n";
    put(&paths.dsh.join(".credentials.yaml"), credential);
    paths.apply(&store, &provider(Client::Dsh)).unwrap();
    let credentials = fs::read(paths.dsh.join(".credentials.yaml")).unwrap();
    paths.apply_official(&store, Client::Dsh).unwrap();
    assert!(paths.status(Client::Dsh, true).unwrap().official);
    assert_eq!(
        fs::read(paths.dsh.join(".credentials.yaml")).unwrap(),
        credentials
    );
    let p = provider(Client::Dsh);
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Dsh).unwrap(),
        Some((p.base_url, p.api_key))
    );
    paths.apply_official(&store, Client::Dsh).unwrap();
    for name in ["headless", "web"] {
        assert_eq!(
            fs::read_to_string(
                paths
                    .dsh
                    .join("profiles")
                    .join(name)
                    .join("cordis.patch.yml")
            )
            .unwrap(),
            "# profile stays intact\n[]\n"
        );
    }
    assert!(
        fs::read_to_string(paths.dsh.join(".credentials.yaml"))
            .unwrap()
            .contains("account: {token: native-fake, expires: 123}")
    );
}
#[test]
fn pp19_dsh_can_return_from_account_without_an_api_key_or_credential_file() {
    let (_dir, paths, store) = fixture();
    dsh_setup(&paths);
    paths.apply_official(&store, Client::Dsh).unwrap();
    assert!(!paths.dsh.join(".credentials.yaml").exists());
    let original = fs::read(paths.settings(Client::Dsh)).unwrap();
    let modified = fs::metadata(paths.settings(Client::Dsh))
        .unwrap()
        .modified()
        .unwrap();
    paths.apply_official(&store, Client::Dsh).unwrap();
    assert_eq!(fs::read(paths.settings(Client::Dsh)).unwrap(), original);
    assert_eq!(
        fs::metadata(paths.settings(Client::Dsh))
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
    paths.apply(&store, &provider(Client::Dsh)).unwrap();
    assert_eq!(paths.current(Client::Dsh).unwrap().unwrap().1, "fake-key");
}
#[test]
fn pp19_dsh_inherited_account_endpoint_is_cleared_without_dropping_catalog() {
    let (_dir, paths, store) = fixture();
    dsh_setup(&paths);
    let profile = "- id: llm-deepseek-account\n  config:\n    baseURL: https://custom.example/anthropic\n    models: [{id: deepseek-flash, contextWindow: 200000}]\n";
    for name in ["headless", "web"] {
        put(
            &paths
                .dsh
                .join("profiles")
                .join(name)
                .join("cordis.patch.yml"),
            profile,
        );
    }
    paths.apply_official(&store, Client::Dsh).unwrap();
    assert!(paths.is_official(Client::Dsh).unwrap());
    let patch = fs::read_to_string(paths.settings(Client::Dsh)).unwrap();
    assert!(!patch.contains("custom.example"));
    assert!(patch.contains("contextWindow: 200000"));
    for name in ["headless", "web"] {
        assert_eq!(
            fs::read_to_string(
                paths
                    .dsh
                    .join("profiles")
                    .join(name)
                    .join("cordis.patch.yml")
            )
            .unwrap(),
            profile
        );
    }
}
#[test]
fn pp19_missing_native_setup_for_first_four_is_initialized_minimally() {
    for client in [Client::Claude, Client::Codex, Client::Gemini, Client::Grok] {
        let (_dir, paths, store) = fixture();
        paths.apply_official(&store, client).unwrap();
        assert!(paths.status(client, true).unwrap().official);
        paths.apply(&store, &provider(client)).unwrap();
        assert!(paths.current(client).unwrap().is_some());
    }
}
#[test]
fn pp19_reserved_route_collision_and_active_codex_profile_never_write() {
    for (client, text) in [
        (
            Client::Codex,
            "[model_providers.pps]\nbase_url='https://user.example'\n",
        ),
        (
            Client::Grok,
            "[model.pps]\nmodel='user'\nbase_url='https://user.example'\n",
        ),
        (
            Client::Codex,
            "profile='user'\n[profiles.user]\nmodel_provider='relay'\n",
        ),
    ] {
        let (_dir, paths, store) = fixture();
        put(&paths.settings(client), text);
        assert!(paths.apply(&store, &provider(client)).is_err());
        assert_eq!(fs::read_to_string(paths.settings(client)).unwrap(), text);
    }
}
#[test]
fn pp19_malformed_modes_and_fields_refuse_official_without_writes() {
    for (client, text) in [
        (Client::Claude, "{\"env\":null}"),
        (Client::Claude, "{\"env\":{\"ANTHROPIC_BASE_URL\":123}}"),
        (Client::Claude, "{\"apiKeyHelper\":\"external\"}"),
        (
            Client::Claude,
            "{\"env\":{\"CLAUDE_CODE_USE_VERTEX\":\"1\"}}",
        ),
        (Client::Codex, "model_provider = ["),
        (Client::Codex, "model_provider = 42\n"),
        (Client::Codex, "model_providers = 42\n"),
        (Client::Codex, "forced_login_method = 'external'\n"),
        (
            Client::Codex,
            "[model_providers.openai]\nbase_url='https://custom.example'\n",
        ),
        (Client::Grok, "models = 3\n"),
    ] {
        let (_dir, paths, store) = fixture();
        put(&paths.settings(client), text);
        assert!(paths.apply_official(&store, client).is_err());
        assert_eq!(fs::read_to_string(paths.settings(client)).unwrap(), text);
        assert!(paths.status(client, true).is_err());
    }
    let (_dir, paths, store) = fixture();
    put(&paths.settings(Client::Gemini), "GEMINI_API_KEY='old'\n");
    put(
        &paths.gemini.join("settings.json"),
        r#"{"security":{"auth":{"selectedType":"vertex-ai"}}}"#,
    );
    assert!(paths.apply_official(&store, Client::Gemini).is_err());
    assert_eq!(
        fs::read_to_string(paths.settings(Client::Gemini)).unwrap(),
        "GEMINI_API_KEY='old'\n"
    );
}
#[test]
fn pp19_native_status_refuses_missing_grok_profile_and_bad_codex_auth() {
    let (_dir, paths, store) = fixture();
    put(
        &paths.settings(Client::Grok),
        "[models]\ndefault='missing'\n",
    );
    assert!(paths.status(Client::Grok, true).is_err());
    assert!(paths.apply(&store, &provider(Client::Grok)).is_err());
    for auth in [r#"{"OPENAI_API_KEY":123}"#, r#"{"auth_mode":"external"}"#] {
        put(&paths.codex.join("auth.json"), auth);
        assert!(paths.status(Client::Codex, true).is_err());
        assert!(paths.apply_official(&store, Client::Codex).is_err());
        assert_eq!(
            fs::read_to_string(paths.codex.join("auth.json")).unwrap(),
            auth
        );
        assert!(!paths.settings(Client::Codex).exists());
    }
}
#[test]
fn pp19_toml_roundtrips_preserve_crlf_and_unrelated_bytes() {
    for client in [Client::Codex, Client::Grok] {
        let (_dir, paths, store) = fixture();
        let file = paths.settings(client);
        let baseline = "# 中文 comment\r\n[unrelated]\r\nkeep = 'a,b' # same\r\n";
        put(&file, baseline);
        paths.apply(&store, &provider(client)).unwrap();
        for restore in [false, true, false] {
            if restore {
                paths.apply_official(&store, client).unwrap();
            } else {
                paths.apply(&store, &provider(client)).unwrap();
            }
            let after = fs::read_to_string(&file).unwrap();
            assert!(after.contains("keep = 'a,b' # same\r\n"));
            assert!(!after.replace("\r\n", "").contains('\n'));
        }
    }
}
#[test]
fn pp19_json_deletion_preserves_unicode_escaped_names_commas_and_other_bytes() {
    for text in [
        "{\"remove\":\"a,b\", \"保留\" : [1, 2]}",
        "{\"保留\" : [1, 2],\"remove\":\"a,b\"}",
        "{\"remove\":\"a,b\"}",
        "{\"env\":{\"remove\":\"a,b\",\"keep\":true}}",
    ] {
        let path: &[&str] = if text.contains("env") {
            &["env", "remove"]
        } else {
            &["remove"]
        };
        let after = config::remove_json(text, path).unwrap();
        assert!(config::json_value(&after, path).unwrap().is_none());
        if text.contains("保留") {
            assert!(after.contains("\"保留\" : [1, 2]"));
        }
        if text.contains("keep") {
            assert!(after.contains("\"keep\":true"));
        }
        assert_eq!(config::remove_json(&after, path).unwrap(), after);
    }
}
#[cfg(unix)]
#[test]
fn pp19_gemini_and_codex_symlink_failures_leave_all_real_files_intact() {
    use std::os::unix::fs::symlink;
    for client in [Client::Gemini, Client::Codex] {
        let (dir, paths, store) = fixture();
        let (settings, text) = if client == Client::Gemini {
            (
                paths.gemini.join("settings.json"),
                r#"{"security":{"auth":{"selectedType":"gemini-api-key"}}}"#,
            )
        } else {
            (
                paths.codex.join("auth.json"),
                r#"{"OPENAI_API_KEY":"fake","keep":true}"#,
            )
        };
        let main = paths.settings(client);
        let main_text = if client == Client::Gemini {
            "GEMINI_API_KEY='fake'\n"
        } else {
            "model_provider='relay'\nmodel='relay-model'\n"
        };
        put(&main, main_text);
        put(&settings, text);
        let real = dir.path().join("original.json");
        fs::rename(&settings, &real).unwrap();
        symlink(&real, &settings).unwrap();
        assert!(paths.apply_official(&store, client).is_err());
        assert_eq!(fs::read_to_string(&main).unwrap(), main_text);
        assert_eq!(fs::read_to_string(&real).unwrap(), text);
    }
}
