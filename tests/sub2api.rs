//! Source fixture: Wei-Shaw/sub2api ab80abc6d1f929a6864c8804fd077b509d21fda2,
//! frontend/src/utils/ccswitchImport.ts. Fake secrets; no script execution/network.
use pp_switch::provider::{Client, import_link};

#[test]
fn pp06_sub2api_links_preserve_supplied_models_without_inventing_claude_defaults() {
    for (app, endpoint, model) in [
        ("claude", "https://relay.example/", None),
        ("claude", "https://relay.example/antigravity", None),
        ("gemini", "https://relay.example/", None),
        ("gemini", "https://relay.example/antigravity", None),
        ("codex", "https://relay.example", Some("gpt-5.5")),
        ("grokbuild", "https://relay.example/v1", Some("grok-4.5")),
    ] {
        let mut url = url::Url::parse("ccswitch://v1/import").unwrap();
        let mut query = url.query_pairs_mut();
        query.extend_pairs([
            ("resource", "provider"),
            ("app", app),
            ("name", "Sub2API Fixture"),
            ("homepage", "https://relay.example/"),
            ("endpoint", endpoint),
            ("apiKey", "example-key"),
            ("configFormat", "json"),
            ("usageEnabled", "true"),
            ("usageScript", "dGhyb3cgbmV3IEVycm9yKCduZXZlciBydW4nKQ=="),
            ("usageAutoInterval", "30"),
        ]);
        if let Some(model) = model {
            query.append_pair("model", model);
        }
        drop(query);
        let provider = import_link(url.as_str()).unwrap();
        assert_eq!(provider.client, Client::parse(app).unwrap());
        assert_eq!(provider.base_url, endpoint);
        assert_eq!(provider.api_key, "example-key");
        assert_eq!(provider.models.model.as_deref(), model);
        assert!(
            provider.models.haiku.is_none()
                && provider.models.sonnet.is_none()
                && provider.models.opus.is_none()
        );
        let saved = serde_json::to_string(&provider).unwrap();
        for ignored in [
            "usageScript",
            "usageEnabled",
            "usageAutoInterval",
            "homepage",
        ] {
            assert!(!saved.contains(ignored));
        }
    }
}
