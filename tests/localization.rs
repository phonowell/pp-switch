use pp_switch::{
    i18n::{self, Language},
    storage::Store,
};
use std::{collections::HashSet, fs};

#[test]
fn pp13_system_locale_mapping_and_catalog_coverage() {
    for (locale, expected) in [
        ("en-US", Language::En),
        ("zh_CN.UTF-8", Language::Zh),
        ("zh-Hans-CN", Language::Zh),
        ("zh-Hant", Language::ZhTw),
        ("zh_HK", Language::ZhTw),
        ("zh-TW", Language::ZhTw),
        ("ja_JP.UTF-8", Language::Ja),
        ("de_DE", Language::En),
        ("C", Language::En),
    ] {
        assert_eq!(Language::from_locale(locale), expected);
    }
    let mut keys = HashSet::new();
    for line in i18n::CATALOG.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 4, "{line}");
        assert!(keys.insert(fields[0]), "Duplicate key: {}", fields[0]);
        assert!(fields.iter().all(|f| !f.trim().is_empty()));
        for translated in &fields[1..] {
            for token in ["{path}", "{name}", "{url}", "{version}"] {
                assert_eq!(
                    fields[0].contains(token),
                    translated.contains(token),
                    "{line}"
                );
            }
        }
    }
    for lang in Language::ALL.into_iter().skip(1) {
        assert_ne!(
            i18n::message(lang, "Cannot read /tmp/example."),
            "Cannot read /tmp/example."
        );
        assert!(
            i18n::message(
                lang,
                "Current: https://example.com  ·  * marks a matching provider"
            )
            .contains("https://example.com")
        );
        assert_ne!(
            i18n::tr(lang, "Update checksum mismatch. Nothing was installed."),
            "Update checksum mismatch. Nothing was installed."
        );
    }
}

#[test]
fn pp13_preferences_are_private_and_do_not_touch_provider_data() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store {
        root: temp.path().join("store"),
    };
    assert_eq!(store.language().unwrap(), Language::system());
    for lang in Language::ALL {
        store.save_language(lang).unwrap();
        assert_eq!(store.language().unwrap(), lang);
    }
    assert!(!store.root.join("providers.json").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(store.root.join("ui.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let invalid = b"{\"language\":\"unknown\"}";
    fs::write(store.root.join("ui.json"), invalid).unwrap();
    assert!(store.language().is_err());
    assert!(store.save_language(Language::En).is_err());
    assert_eq!(fs::read(store.root.join("ui.json")).unwrap(), invalid);
}
