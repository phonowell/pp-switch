//! Scoped native-mode transitions. No OAuth/account credential copies.
use super::{Paths, claude_config, environment_conflict};
use crate::{
    Result, config,
    provider::{Client, Provider},
    storage::{self, Change, Store},
};
use toml_edit::{DocumentMut, Item, Table, value};

const CLAUDE_ENV: &[&str] = &[
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL",
];
const GEMINI_ENV: &[&str] = &[
    "GOOGLE_GEMINI_BASE_URL",
    "GEMINI_BASE_URL",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "GEMINI_MODEL",
];
const MANAGED: &str = "# Pure Provider Switch managed route\n";
pub(super) fn claude_native(env: &config::JsonObject<'_>) -> bool {
    !CLAUDE_ENV.iter().any(|key| env.get(key).is_some())
}
fn toml(text: &str) -> Result<DocumentMut> {
    text.parse()
        .map_err(|_| "Configuration is not valid TOML.".into())
}
fn no_profile(doc: &DocumentMut) -> Result<()> {
    if doc.get("profile").is_some() {
        return Err(
            "Codex profile overrides must be cleared before switching official mode.".into(),
        );
    }
    Ok(())
}
fn table_shape(doc: &DocumentMut, field: &str) -> Result<()> {
    if doc.get(field).is_some_and(|v| v.as_table_like().is_none()) {
        return Err("Provider catalog must be a table.".into());
    }
    Ok(())
}
fn codex_shape(doc: &DocumentMut) -> Result<()> {
    no_profile(doc)?;
    table_shape(doc, "model_providers")?;
    for field in ["model_provider", "model", "forced_login_method"] {
        if doc.get(field).is_some_and(|v| v.as_str().is_none()) {
            return Err("Configured provider must be a string.".into());
        }
    }
    if doc
        .get("forced_login_method")
        .is_some_and(|v| !matches!(v.as_str(), Some("api" | "chatgpt")))
    {
        return Err("Codex forced login method is unsupported.".into());
    }
    Ok(())
}
fn codex_auth_mode(auth: &str) -> Result<Option<String>> {
    let mode = config::json_string(auth, &["auth_mode"])?;
    if !matches!(
        mode.as_deref(),
        None | Some("chatgpt" | "api_key" | "apikey")
    ) {
        return Err("Codex authentication mode is unsupported.".into());
    }
    Ok(mode)
}
fn gemini_auth(text: &str) -> Result<Option<String>> {
    let mode = config::json_string(text, &["security", "auth", "selectedType"])?;
    if !matches!(
        mode.as_deref(),
        None | Some("gemini-api-key" | "oauth-personal")
    ) {
        return Err(
            "Gemini cloud/external authentication is outside official switching scope.".into(),
        );
    }
    Ok(mode)
}
fn render(doc: &DocumentMut, before: &str) -> String {
    let next = doc.to_string();
    if before.contains("\r\n") && !before.replace("\r\n", "").contains('\n') {
        next.replace("\r\n", "\n").replace('\n', "\r\n")
    } else {
        next
    }
}
fn edited(path: std::path::PathBuf, before: Option<String>, after: String) -> Change {
    Change {
        path,
        before,
        after,
    }
}
impl Paths {
    pub fn is_official(&self, client: Client) -> Result<bool> {
        if client == Client::Dsh {
            return self.dsh_is_official();
        }
        let text = storage::read(&self.settings(client))?;
        match client {
            Client::Claude => {
                let (_, env, _) = claude_config(text.as_deref().unwrap_or("{}"))?;
                Ok(claude_native(&env))
            }
            Client::Codex => {
                let doc = toml(text.as_deref().unwrap_or(""))?;
                codex_shape(&doc)?;
                let native = match doc.get("model_provider") {
                    None => true,
                    Some(item) => {
                        item.as_str()
                            .ok_or("Configured provider must be a string.")?
                            == "openai"
                    }
                };
                let auth = storage::read(&self.codex.join("auth.json"))?;
                let key = auth
                    .as_deref()
                    .map(|s| config::json_string(s, &["OPENAI_API_KEY"]))
                    .transpose()?
                    .flatten();
                let mode = auth.as_deref().map(codex_auth_mode).transpose()?.flatten();
                Ok(native
                    && doc
                        .get("model_providers")
                        .and_then(|p| p.get("openai"))
                        .is_none()
                    && key.is_none_or(|s| s.is_empty())
                    && mode.is_none_or(|s| s == "chatgpt")
                    && doc.get("forced_login_method").and_then(Item::as_str) != Some("api"))
            }
            Client::Gemini => {
                let settings = storage::read(&self.gemini.join("settings.json"))?;
                let mode = gemini_auth(settings.as_deref().unwrap_or("{}"))?;
                let text = text.as_deref().unwrap_or("");
                let mut clean = true;
                for key in &GEMINI_ENV[..4] {
                    clean &= config::dotenv_value(text, key)?.is_none();
                }
                Ok(clean && matches!(mode.as_deref(), None | Some("oauth-personal")))
            }
            Client::Grok => {
                let doc = toml(text.as_deref().unwrap_or(""))?;
                table_shape(&doc, "models")?;
                table_shape(&doc, "model")?;
                let selected = doc.get("models").and_then(|m| m.get("default"));
                match selected {
                    None => Ok(true),
                    Some(item) => {
                        let id = item
                            .as_str()
                            .ok_or("Configured model selector must be a string.")?;
                        if doc.get("model").and_then(|m| m.get(id)).is_none() {
                            return Err("Grok Build selected profile is missing.".into());
                        }
                        Ok(false)
                    }
                }
            }
            Client::Dsh => unreachable!(),
        }
    }

    pub fn apply_official(&self, store: &Store, client: Client) -> Result<()> {
        store.recover()?;
        environment_conflict(client)?;
        if client == Client::Dsh {
            return self.apply_dsh_official(store);
        }
        let path = self.settings(client);
        let before = storage::read(&path)?;
        let mut changes = Vec::new();
        let after = match client {
            Client::Claude => {
                let text = before.as_deref().unwrap_or("{}");
                claude_config(text)?;
                for key in CLAUDE_ENV {
                    if std::env::var_os(key).is_some_and(|v| !v.is_empty()) {
                        return Err("Claude model/environment overrides must be cleared before restoring official mode.".into());
                    }
                }
                let (root, env, _) = claude_config(text)?;
                root.string("model")?;
                for key in CLAUDE_ENV {
                    env.string(key)?;
                }
                let mut out = text.to_owned();
                for key in CLAUDE_ENV {
                    out = config::remove_json(&out, &["env", key])?;
                }
                config::remove_json(&out, &["model"])?
            }
            Client::Codex => {
                let mut doc = toml(before.as_deref().unwrap_or(""))?;
                codex_shape(&doc)?;
                if doc
                    .get("model_providers")
                    .and_then(|p| p.get("openai"))
                    .is_some()
                {
                    return Err(
                        "Custom openai table overrides the native route; remove it in Codex first."
                            .into(),
                    );
                }
                doc.remove("model_provider");
                doc.remove("model");
                if doc.get("forced_login_method").and_then(Item::as_str) == Some("api") {
                    doc.remove("forced_login_method");
                }
                let auth_path = self.codex.join("auth.json");
                if let Some(auth) = storage::read(&auth_path)? {
                    // Validate the root even if neither API field exists.
                    config::JsonObject::parse(&auth)?;
                    config::json_string(&auth, &["OPENAI_API_KEY"])?;
                    codex_auth_mode(&auth)?;
                    let mut out = config::remove_json(&auth, &["OPENAI_API_KEY"])?;
                    if config::json_string(&out, &["auth_mode"])?
                        .is_some_and(|m| m == "api_key" || m == "apikey")
                    {
                        out = config::remove_json(&out, &["auth_mode"])?;
                    }
                    changes.push(edited(auth_path, Some(auth), out));
                }
                render(&doc, before.as_deref().unwrap_or(""))
            }
            Client::Gemini => {
                if std::env::var_os("GEMINI_MODEL").is_some_and(|v| !v.is_empty()) {
                    return Err("Gemini model/environment overrides must be cleared before restoring official mode.".into());
                }
                let settings_path = self.gemini.join("settings.json");
                let settings = storage::read(&settings_path)?;
                let text = settings.as_deref().unwrap_or("{}");
                gemini_auth(text)?;
                let out = config::patch_json(
                    text,
                    &["security", "auth", "selectedType"],
                    "oauth-personal",
                )?;
                let out = config::remove_json(&out, &["model", "name"])?;
                changes.push(edited(settings_path, settings, out));
                let mut out = before.as_deref().unwrap_or("").to_owned();
                for key in GEMINI_ENV {
                    out = config::remove_dotenv(&out, key)?;
                }
                out
            }
            Client::Grok => {
                let mut doc = toml(before.as_deref().unwrap_or(""))?;
                table_shape(&doc, "models")?;
                table_shape(&doc, "model")?;
                if let Some(models) = doc.get_mut("models") {
                    let models = models
                        .as_table_like_mut()
                        .ok_or("Grok models must be a table.")?;
                    let id = models
                        .get("default")
                        .map(|i| {
                            i.as_str()
                                .ok_or("Configured model selector must be a string.")
                        })
                        .transpose()?
                        .map(str::to_owned);
                    if let Some(id) = id {
                        if models.get("session_summary").and_then(Item::as_str) == Some(id.as_str())
                        {
                            models.remove("session_summary");
                        }
                        models.remove("default");
                    }
                }
                render(&doc, before.as_deref().unwrap_or(""))
            }
            Client::Dsh => unreachable!(),
        };
        if before.is_some() || !after.is_empty() {
            changes.push(edited(path, before, after));
        }
        store.commit(changes)
    }

    pub(super) fn native_route(&self, client: Client) -> Result<bool> {
        let text = storage::read(&self.settings(client))?.unwrap_or_default();
        let doc = toml(&text)?;
        if client == Client::Codex {
            codex_shape(&doc)?;
            Ok(doc
                .get("model_provider")
                .is_none_or(|v| v.as_str() == Some("openai")))
        } else {
            self.is_official(client)
        }
    }

    pub(super) fn apply_from_native(&self, store: &Store, provider: &Provider) -> Result<()> {
        let path = self.settings(provider.client);
        let before = storage::read(&path)?;
        let mut doc = toml(before.as_deref().unwrap_or(""))?;
        super::models::model_environment_conflict(provider.client, &provider.models)?;
        let (catalog, selector) = if provider.client == Client::Codex {
            ("model_providers", "model_provider")
        } else {
            ("model", "default")
        };
        if doc
            .get(catalog)
            .is_some_and(|v| v.as_table_like().is_none())
        {
            return Err("Provider catalog must be a table.".into());
        }
        if let Some(existing) = doc.get(catalog).and_then(|c| c.get("pps")) {
            let marked = existing
                .as_table()
                .and_then(|t| t.decor().prefix())
                .and_then(|s| s.as_str())
                .is_some_and(|p| p.contains(MANAGED.trim()));
            if !marked {
                return Err(
                    "A non-PPS pps profile already exists; rename it before switching.".into(),
                );
            }
        }
        if doc.get(catalog).is_none() {
            doc[catalog] = Item::Table(Table::new());
        }
        let mut route = doc
            .get(catalog)
            .and_then(|c| c.get("pps"))
            .and_then(Item::as_table)
            .cloned()
            .unwrap_or_else(Table::new);
        if route.is_empty() {
            route.decor_mut().set_prefix(MANAGED);
        }
        for field in [
            "env_key",
            "auth_provider",
            "env_http_headers",
            "mtls_cert_dir",
        ] {
            if route.get(field).is_some() {
                return Err("Managed PPS route uses an external credential source; restore its API-key setup first.".into());
            }
        }
        route["base_url"] = value(provider.base_url.as_str());
        if provider.client == Client::Codex {
            no_profile(&doc)?;
            if doc.get("forced_login_method").is_some() {
                return Err("Codex forced login method must be cleared before switching a native bearer route.".into());
            }
            doc[selector] = value("pps");
            if route.get("name").is_none() {
                route["name"] = value("Pure Provider Switch");
            }
            if route
                .get("wire_api")
                .is_some_and(|v| v.as_str() != Some("responses"))
                || route
                    .get("requires_openai_auth")
                    .is_some_and(|v| v.as_bool() != Some(false))
            {
                return Err("Managed PPS route has an incompatible protocol or auth mode.".into());
            }
            route["wire_api"] = value("responses");
            route["requires_openai_auth"] = value(false);
            route["experimental_bearer_token"] = value(provider.api_key.as_str());
            if let Some(model) = &provider.models.model {
                doc["model"] = value(model.as_str());
            }
        } else {
            let model = provider
                .models
                .model
                .as_deref()
                .ok_or("Set a default model before switching Grok from official mode.")?;
            if doc.get("models").is_none() {
                doc["models"] = Item::Table(Table::new());
            }
            if doc.get("models").and_then(Item::as_table_like).is_none() {
                return Err("Grok models must be a table.".into());
            }
            doc["models"][selector] = value("pps");
            route["api_key"] = value(provider.api_key.as_str());
            route["model"] = value(model);
            if route
                .get("api_backend")
                .is_some_and(|v| v.as_str() != Some("responses"))
            {
                return Err("Managed PPS route has an incompatible protocol or auth mode.".into());
            }
            route["api_backend"] = value("responses");
        }
        let table = doc
            .get_mut(catalog)
            .and_then(Item::as_table_like_mut)
            .ok_or("Provider catalog must be a table.")?;
        table.insert("pps", Item::Table(route));
        let after = render(&doc, before.as_deref().unwrap_or(""));
        store.commit(vec![edited(path, before, after)])
    }

    pub(super) fn apply_gemini(&self, store: &Store, provider: &Provider) -> Result<()> {
        let path = self.settings(Client::Gemini);
        let before = storage::read(&path)?;
        let text = before.as_deref().unwrap_or("");
        for field in ["GOOGLE_API_KEY", "GEMINI_BASE_URL"] {
            if config::dotenv_value(text, field)?.is_some() {
                return Err(
                    "Gemini alternate URL/key sources must be cleared before switching.".into(),
                );
            }
        }
        let mut out = config::patch_dotenv(
            before.as_deref().unwrap_or(""),
            "GOOGLE_GEMINI_BASE_URL",
            &provider.base_url,
        )?;
        out = config::patch_dotenv(&out, "GEMINI_API_KEY", &provider.api_key)?;
        let settings_path = self.gemini.join("settings.json");
        let settings = storage::read(&settings_path)?;
        let text = settings.as_deref().unwrap_or("{}");
        gemini_auth(text)?;
        let mut selected = config::patch_json(
            text,
            &["security", "auth", "selectedType"],
            "gemini-api-key",
        )?;
        if let Some(model) = &provider.models.model {
            // Reuse the same environment-override validation as model writes.
            super::models::model_environment_conflict(Client::Gemini, &provider.models)?;
            config::json_string(&selected, &["model", "name"])?;
            selected = config::patch_json(&selected, &["model", "name"], model)?;
            out = config::patch_dotenv(&out, "GEMINI_MODEL", model)?;
        }
        store.commit(vec![
            edited(path, before, out),
            edited(settings_path, settings, selected),
        ])
    }
}
