use pp_switch::{
    client::Paths,
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
            "# preserve profile\n[]\n",
        );
    }
    put(
        &paths.dsh.join(".credentials.yaml"),
        "# private\nversion: 1\nrefs:\n  DEEPSEEK_API_KEY: old-key # keep\n  OTHER: other-key\nrecords:\n  account:\n    kind: grant\n    payload: {token: unrelated-fake, expires: 123}\n",
    );
    let store = Store {
        root: dir.path().join("store"),
    };
    (dir, paths, store)
}
fn provider() -> Provider {
    Provider::new(
        Client::Dsh,
        "DSH",
        "https://new.example/anthropic",
        "new-key",
    )
    .unwrap()
}
fn profiles(paths: &Paths) -> Vec<Vec<u8>> {
    ["headless", "web"]
        .into_iter()
        .flat_map(|name| {
            ["package.json", "cordis.patch.yml"]
                .map(move |f| fs::read(paths.dsh.join("profiles").join(name).join(f)).unwrap())
        })
        .collect()
}
#[test]
fn pp17_shared_native_connection_and_model_preserve_both_profiles_and_credentials() {
    let (_dir, paths, store) = fixture();
    let original = profiles(&paths);
    let creds = fs::read_to_string(paths.dsh.join(".credentials.yaml")).unwrap();
    assert_eq!(
        paths.current(Client::Dsh).unwrap(),
        Some((
            "https://api.deepseek.com/anthropic".into(),
            "old-key".into()
        ))
    );
    let mut p = provider();
    p.models = Models::from_fields("deepseek-reasoner", "", "", "");
    paths.apply(&store, &p).unwrap();
    assert_eq!(
        paths.current(Client::Dsh).unwrap(),
        Some((p.base_url.clone(), p.api_key.clone()))
    );
    assert!(
        paths
            .current_models(Client::Dsh)
            .unwrap()
            .matches(Client::Dsh, &p.models)
    );
    assert_eq!(profiles(&paths), original);
    assert_eq!(
        fs::read_to_string(paths.dsh.join(".credentials.yaml")).unwrap(),
        creds.replace("old-key", "new-key")
    );
    let common = fs::read(paths.settings(Client::Dsh)).unwrap();
    let mut draft = p.clone();
    draft.models = Models::default();
    assert_eq!(
        paths.editor_models(&draft).model.as_deref(),
        Some("deepseek-reasoner")
    );
    draft.api_key = "different-key".into();
    assert!(paths.editor_models(&draft).is_empty());
    assert_eq!(fs::read(paths.settings(Client::Dsh)).unwrap(), common);
    assert!(String::from_utf8_lossy(&common).contains("deepseek-official"));
    paths.apply(&store, &p).unwrap();
    assert_eq!(fs::read(paths.settings(Client::Dsh)).unwrap(), common);
    assert!(!store.root.join("pending.json").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for file in [
            paths.settings(Client::Dsh),
            paths.dsh.join(".credentials.yaml"),
        ] {
            assert_eq!(
                fs::metadata(file).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
const PI: &str = "# keep home\n- id: agent-default-model\n  config:\n    provider: relay\n    model: old-model # selection\n    reasoningEffort: high\n- id: llm-pi-ai\n  config:\n    providers:\n      relay:\n        baseURL: https://old.example/v1 # endpoint\n        apiKeyEnv: RELAY_KEY\n        api: openai-completions\n        models: [{id: old-model, contextWindow: 200000}]\n      other:\n        baseURL: https://other.example\n        apiKeyEnv: OTHER\n- id: unrelated\n  config:\n    keep: 中文\n";
#[test]
fn pp17_shared_pi_route_preserves_catalog_protocol_other_routes_and_crlf() {
    let (_dir, paths, store) = fixture();
    put(&paths.settings(Client::Dsh), &PI.replace('\n', "\r\n"));
    put(
        &paths.dsh.join(".credentials.yaml"),
        "version: 1\r\nrefs:\r\n  RELAY_KEY: old-key # keep\r\n  OTHER: untouched\r\n",
    );
    let mut p = provider();
    p.models = Models::from_fields("new-model", "", "", "");
    paths.apply(&store, &p).unwrap();
    let text = fs::read_to_string(paths.settings(Client::Dsh)).unwrap();
    assert_eq!(
        text,
        PI.replace("https://old.example/v1", &p.base_url)
            .replace(
                "model: old-model # selection",
                "model: new-model # selection"
            )
            .replace('\n', "\r\n")
    );
    assert!(
        paths
            .current_models(Client::Dsh)
            .unwrap()
            .matches(Client::Dsh, &p.models)
    );
    assert_eq!(
        fs::read_to_string(paths.dsh.join(".credentials.yaml")).unwrap(),
        "version: 1\r\nrefs:\r\n  RELAY_KEY: new-key # keep\r\n  OTHER: untouched\r\n"
    );
}
#[test]
fn pp17_inherited_config_is_copied_whole_not_deep_merged() {
    let (_dir, paths, store) = fixture();
    for name in ["headless", "web"] {
        put(
            &paths
                .dsh
                .join("profiles")
                .join(name)
                .join("cordis.patch.yml"),
            PI,
        );
    }
    put(
        &paths.dsh.join(".credentials.yaml"),
        "version: 1\nrefs:\n  RELAY_KEY: old-key\n",
    );
    let original = profiles(&paths);
    paths.apply(&store, &provider()).unwrap();
    let common = fs::read_to_string(paths.settings(Client::Dsh)).unwrap();
    assert!(common.contains("contextWindow: 200000"));
    assert!(common.contains("api: openai-completions"));
    assert!(common.contains("https://other.example"));
    assert_eq!(profiles(&paths), original);
    assert_eq!(paths.current(Client::Dsh).unwrap().unwrap().1, "new-key");
}
#[test]
fn pp17_conflicting_profiles_refused_but_explicit_common_selection_is_effective() {
    let (_dir, paths, store) = fixture();
    put(
        &paths.dsh.join("profiles/web/cordis.patch.yml"),
        "- id: agent-default-model\n  config: {provider: other, model: different}\n",
    );
    let before = fs::read(paths.dsh.join(".credentials.yaml")).unwrap();
    assert!(paths.apply(&store, &provider()).is_err());
    assert!(!paths.settings(Client::Dsh).exists());
    assert_eq!(
        fs::read(paths.dsh.join(".credentials.yaml")).unwrap(),
        before
    );
    put(
        &paths.settings(Client::Dsh),
        "- id: agent-default-model\n  config: {provider: deepseek-official, model: deepseek-flash}\n",
    );
    paths.apply(&store, &provider()).unwrap();
}
#[test]
fn pp17_ambiguous_executable_and_external_setups_never_write() {
    for bad in [
        "- id: llm-deepseek\n  config: {baseURL: x, baseURL: y}\n",
        "- id: llm-deepseek\n  config: {baseURL: *reference}\n",
        "- id: llm-deepseek\n  config: {baseURL: &reference x}\n",
        "- id: llm-deepseek\n  config: {baseURL: !!js dangerous}\n",
        "- id: llm-deepseek\n  config: {baseURL: 123}\n",
        "- id: llm-deepseek\n  config: {baseURL: null}\n",
        "- id: llm-deepseek\n  config: {baseURL: true}\n",
        "- id: llm-deepseek\n  disabled: true\n",
        "- id: llm-deepseek\n  config: {}\n- id: llm-deepseek\n  config: {}\n",
        "- id: credentials\n  config: {path: /external}\n",
        "- id: llm-deepseek\n  config: {headers: {Authorization: fake}}\n",
        "- id: llm-deepseek\n  config: {baseURL: 'unfinished}\n",
        "---\n[]\n---\n[]\n",
    ] {
        let (_dir, paths, store) = fixture();
        put(&paths.settings(Client::Dsh), bad);
        let creds = fs::read(paths.dsh.join(".credentials.yaml")).unwrap();
        assert!(
            paths.apply(&store, &provider()).is_err(),
            "unsafe fixture was accepted"
        );
        assert_eq!(
            fs::read_to_string(paths.settings(Client::Dsh)).unwrap(),
            bad
        );
        assert_eq!(
            fs::read(paths.dsh.join(".credentials.yaml")).unwrap(),
            creds
        );
    }
}
#[test]
fn pp17_invalid_credentials_and_uninitialized_profiles_never_write() {
    for bad in [
        "version: 2\nrefs: {DEEPSEEK_API_KEY: old-key}\n",
        "version: 1\nrefs: {DEEPSEEK_API_KEY: 123}\n",
        "version: 1\nrefs: {OTHER: old-key}\n",
        "version: 1\nrefs: {DEEPSEEK_API_KEY: x, DEEPSEEK_API_KEY: y}\n",
    ] {
        let (_dir, paths, store) = fixture();
        put(&paths.dsh.join(".credentials.yaml"), bad);
        assert!(paths.apply(&store, &provider()).is_err());
        assert!(!paths.settings(Client::Dsh).exists());
        assert_eq!(
            fs::read_to_string(paths.dsh.join(".credentials.yaml")).unwrap(),
            bad
        );
    }
    let (_dir, paths, store) = fixture();
    fs::remove_file(paths.dsh.join("profiles/web/package.json")).unwrap();
    assert!(paths.apply(&store, &provider()).is_err());
    assert!(!paths.settings(Client::Dsh).exists());
}
#[cfg(unix)]
#[test]
fn pp17_symlink_failure_is_validated_before_either_file_changes() {
    use std::os::unix::fs::symlink;
    let (dir, paths, store) = fixture();
    let real = dir.path().join("credentials-real.yaml");
    fs::rename(paths.dsh.join(".credentials.yaml"), &real).unwrap();
    symlink(&real, paths.dsh.join(".credentials.yaml")).unwrap();
    let before = fs::read(&real).unwrap();
    assert!(paths.apply(&store, &provider()).is_err());
    assert!(!paths.settings(Client::Dsh).exists());
    assert_eq!(fs::read(&real).unwrap(), before);
}
