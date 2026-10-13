use pp_switch::{
    provider::{Client, Provider},
    storage::Store,
};
fn p(client: Client, name: &str) -> Provider {
    Provider::new(client, name, "https://example.com", "fake-key").unwrap()
}
#[test]
fn pp01_uniqueness_is_client_scoped_trimmed_casefolded_and_id_aware() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store {
        root: dir.path().to_owned(),
    };
    let mut first = p(Client::Claude, "Alice");
    store.save(first.clone()).unwrap();
    let before = std::fs::read(store.root.join("providers.json")).unwrap();
    assert!(store.save(p(Client::Claude, " alice ")).is_err());
    assert_eq!(
        std::fs::read(store.root.join("providers.json")).unwrap(),
        before
    );
    store.save(p(Client::Codex, "Alice")).unwrap();
    first.api_key = "changed".into();
    store.save(first.clone()).unwrap();
    let second = p(Client::Claude, "Bob");
    store.save(second.clone()).unwrap();
    first.name = "Bob".into();
    assert!(store.save(first).is_err());
    assert_eq!(store.load().unwrap().len(), 3);
}
#[test]
fn pp06_import_names_skip_existing_suffixes_and_bound_unicode() {
    let saved = vec![
        p(Client::Claude, "Relay"),
        p(Client::Claude, "Relay (2)"),
        p(Client::Claude, "Relay (4)"),
        p(Client::Codex, "Relay (3)"),
    ];
    let mut imported = p(Client::Claude, "Relay");
    imported.import_name(&saved);
    assert_eq!(imported.name, "Relay (3)");
    let name = "中".repeat(42);
    let mut imported = p(Client::Claude, &name);
    imported.import_name(&[p(Client::Claude, &name)]);
    assert!(imported.name.ends_with(" (2)"));
    assert!(imported.name.len() <= 128);
    imported.validate().unwrap();
}
#[test]
fn pp01_existing_duplicates_remain_readable_without_a_silent_migration() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store {
        root: dir.path().to_owned(),
    };
    let a = p(Client::Claude, "Legacy");
    let mut b = a.clone();
    b.id = "other-id".into();
    let text = serde_json::to_string(&vec![a, b]).unwrap();
    std::fs::write(store.root.join("providers.json"), &text).unwrap();
    assert_eq!(store.load().unwrap().len(), 2);
    assert_eq!(
        std::fs::read_to_string(store.root.join("providers.json")).unwrap(),
        text
    );
}
