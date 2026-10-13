use crate::{
    Result, config,
    provider::{Client, Provider},
    storage::{self, Change, Store},
};
use std::path::PathBuf;

mod dsh;
mod models;
mod official;
pub use models::{CLAUDE_EDITOR_DEFAULT, ModelState};

/// One file snapshot for row badges, never a live connection check.
#[derive(Default)]
pub struct ConfigStatus {
    pub connection: Option<(String, String)>,
    pub models: Option<ModelState>,
    pub official: bool,
}

#[derive(Clone)]
pub struct Paths {
    pub claude: PathBuf,
    pub codex: PathBuf,
    pub gemini: PathBuf,
    pub grok: PathBuf,
    pub dsh: PathBuf,
}

impl Paths {
    pub fn from_home(home: PathBuf) -> Self {
        Self {
            claude: home.join(".claude"),
            codex: home.join(".codex"),
            gemini: home.join(".gemini"),
            grok: home.join(".grok"),
            dsh: home.join(".dsh"),
        }
    }

    pub fn system() -> Result<Self> {
        let mut paths = Self::from_home(dirs::home_dir().ok_or("Cannot locate home directory.")?);
        if let Some(dir) = std::env::var_os("CLAUDE_CONFIG_DIR").filter(|s| !s.is_empty()) {
            paths.claude = PathBuf::from(dir);
        }
        if let Some(dir) = std::env::var_os("CODEX_HOME").filter(|s| !s.is_empty()) {
            paths.codex = PathBuf::from(dir);
        }
        if let Some(dir) = std::env::var_os("DSH_HOME").filter(|s| !s.is_empty()) {
            paths.dsh = PathBuf::from(dir);
        }
        Ok(paths)
    }

    pub fn settings(&self, client: Client) -> PathBuf {
        match client {
            Client::Claude => {
                let standard = self.claude.join("settings.json");
                let legacy = self.claude.join("claude.json");
                if !standard.exists() && legacy.exists() {
                    legacy
                } else {
                    standard
                }
            }
            Client::Codex => self.codex.join("config.toml"),
            Client::Gemini => self.gemini.join(".env"),
            Client::Grok => self.grok.join("config.toml"),
            Client::Dsh => self.dsh.join("cordis.patch.yml"),
        }
    }

    pub fn status(&self, client: Client, include_models: bool) -> Result<ConfigStatus> {
        if client == Client::Dsh {
            return self.dsh_status(include_models);
        }
        if client == Client::Claude {
            let Some(text) = storage::read(&self.settings(client))? else {
                return Ok(ConfigStatus {
                    official: true,
                    ..ConfigStatus::default()
                });
            };
            let (root, env, field) = claude_config(&text)?;
            return Ok(ConfigStatus {
                connection: env.string("ANTHROPIC_BASE_URL")?.zip(env.string(field)?),
                models: include_models.then(|| ModelState::claude(&root, &env)),
                official: official::claude_native(&env),
            });
        }
        if self.is_official(client)? {
            return Ok(ConfigStatus {
                official: true,
                ..ConfigStatus::default()
            });
        }
        Ok(ConfigStatus {
            connection: self.current(client)?,
            models: include_models
                .then(|| self.current_models(client).ok())
                .flatten(),
            official: false,
        })
    }

    pub fn current(&self, client: Client) -> Result<Option<(String, String)>> {
        if client == Client::Dsh {
            return self.dsh_current();
        }
        let Some(text) = storage::read(&self.settings(client))? else {
            return Ok(None);
        };
        let values = match client {
            Client::Claude => {
                let (_, env, field) = claude_config(&text)?;
                (env.string("ANTHROPIC_BASE_URL")?, env.string(field)?)
            }
            Client::Codex => {
                let doc = toml_edit::Document::parse(text.as_str())
                    .map_err(|_| "Codex config is not valid TOML.")?;
                let id = codex_provider(&doc)?;
                let table = &doc["model_providers"][id];
                let url = table
                    .get("base_url")
                    .and_then(toml_edit::Item::as_str)
                    .map(str::to_owned);
                let key = if let Some(bearer) = table.get("experimental_bearer_token") {
                    codex_bearer_mode(table)?;
                    bearer.as_str().map(str::to_owned)
                } else {
                    let auth = storage::read(&self.codex.join("auth.json"))?
                        .unwrap_or_else(|| "{}".into());
                    codex_file_auth(&doc, &auth)?;
                    config::json_string(&auth, &["OPENAI_API_KEY"])?
                };
                (url, key)
            }
            Client::Gemini => {
                self.gemini_mode()?;
                (
                    config::dotenv_value(&text, "GOOGLE_GEMINI_BASE_URL")?,
                    config::dotenv_value(&text, "GEMINI_API_KEY")?,
                )
            }
            Client::Dsh => unreachable!("DSH uses its shared configuration reader"),
            Client::Grok => {
                let doc = toml_edit::Document::parse(text.as_str())
                    .map_err(|_| "Grok Build config is not valid TOML.")?;
                let id = grok_profile(&doc)?;
                let table = &doc["model"][id];
                (
                    table
                        .get("base_url")
                        .and_then(toml_edit::Item::as_str)
                        .map(str::to_owned),
                    table
                        .get("api_key")
                        .and_then(toml_edit::Item::as_str)
                        .map(str::to_owned),
                )
            }
        };
        Ok(values.0.zip(values.1))
    }

    fn gemini_mode(&self) -> Result<()> {
        let settings = storage::read(&self.gemini.join("settings.json"))?
            .ok_or("Set Gemini /auth to API key first; pp-switch does not change auth mode.")?;
        if config::json_string(&settings, &["security", "auth", "selectedType"])?.as_deref()
            != Some("gemini-api-key")
        {
            return Err(
                "Set Gemini /auth to API key first; pp-switch does not change auth mode.".into(),
            );
        }
        Ok(())
    }

    pub fn apply(&self, store: &Store, provider: &Provider) -> Result<()> {
        provider.validate()?;
        store.recover()?;
        environment_conflict(provider.client)?;
        if matches!(provider.client, Client::Codex | Client::Grok)
            && self.native_route(provider.client)?
        {
            return self.apply_from_native(store, provider);
        }
        if provider.client == Client::Gemini {
            return self.apply_gemini(store, provider);
        }
        if provider.client == Client::Dsh {
            return self.apply_dsh(store, provider);
        }
        let path = self.settings(provider.client);
        let before = storage::read(&path)?;
        let text = before.as_deref().unwrap_or(match provider.client {
            Client::Claude => "{}",
            _ => "",
        });
        let mut changes = Vec::new();
        let after = match provider.client {
            Client::Claude => {
                let (_, env, field) = claude_config(text)?;
                // Refuse malformed destinations before preparing any changes,
                // consistently with the configuration status reader.
                env.string("ANTHROPIC_BASE_URL")?;
                env.string(field)?;
                let next =
                    config::patch_json(text, &["env", "ANTHROPIC_BASE_URL"], &provider.base_url)?;
                config::patch_json(&next, &["env", field], &provider.api_key)?
            }
            Client::Codex => {
                let doc = toml_edit::Document::parse(text)
                    .map_err(|_| "Codex config is not valid TOML.")?;
                let id = codex_provider(&doc)?;
                let table = &doc["model_providers"][id];
                let next = config::patch_toml(
                    text,
                    &["model_providers", id, "base_url"],
                    &provider.base_url,
                )?;
                if table.get("experimental_bearer_token").is_some() {
                    codex_bearer_mode(table)?;
                    config::patch_toml(
                        &next,
                        &["model_providers", id, "experimental_bearer_token"],
                        &provider.api_key,
                    )?
                } else {
                    let auth_path = self.codex.join("auth.json");
                    let auth_before = storage::read(&auth_path)?;
                    let auth = auth_before
                        .as_deref()
                        .ok_or("Initialize Codex API-key auth first; auth.json is missing.")?;
                    codex_file_auth(&doc, auth)?;
                    let auth_after =
                        config::patch_json(auth, &["OPENAI_API_KEY"], &provider.api_key)?;
                    changes.push(Change {
                        path: auth_path,
                        before: auth_before,
                        after: auth_after,
                    });
                    next
                }
            }
            Client::Gemini => {
                self.gemini_mode()?;
                let next =
                    config::patch_dotenv(text, "GOOGLE_GEMINI_BASE_URL", &provider.base_url)?;
                config::patch_dotenv(&next, "GEMINI_API_KEY", &provider.api_key)?
            }
            Client::Dsh => unreachable!("DSH uses a recoverable shared configuration write"),
            Client::Grok => {
                let doc = toml_edit::Document::parse(text)
                    .map_err(|_| "Grok Build config is not valid TOML.")?;
                let id = grok_profile(&doc)?;
                let next =
                    config::patch_toml(text, &["model", id, "base_url"], &provider.base_url)?;
                config::patch_toml(&next, &["model", id, "api_key"], &provider.api_key)?
            }
        };
        let mut after = after;
        if !provider.models.is_empty() {
            let (modeled, model_change) =
                self.apply_models(provider.client, &provider.models, &after)?;
            after = modeled;
            if let Some(change) = model_change {
                changes.push(change);
            }
        }
        changes.push(Change {
            path,
            before,
            after,
        });
        store.commit(changes)
    }
}

fn grok_profile<'a>(doc: &'a toml_edit::Document<&str>) -> Result<&'a str> {
    let id = doc.get("models").and_then(|m| m.get("default"))
        .and_then(toml_edit::Item::as_str)
        .ok_or("Initialize a custom Grok Build API-key profile first; pp-switch never changes models.default.")?;
    let table = doc
        .get("model")
        .and_then(|profiles| profiles.get(id))
        .ok_or("Selected Grok Build profile has no custom configuration.")?;
    for field in [
        "env_key",
        "auth_provider",
        "env_http_headers",
        "mtls_cert_dir",
    ] {
        if table.get(field).is_some() {
            return Err("Grok Build uses an external credential source; only an existing inline API-key profile is supported.".into());
        }
    }
    for field in ["base_url", "api_key"] {
        if table
            .get(field)
            .and_then(toml_edit::Item::as_str)
            .filter(|s| !s.trim().is_empty())
            .is_none()
        {
            return Err(
                "Initialize a custom Grok Build API-key profile with base_url and api_key first."
                    .into(),
            );
        }
    }
    Ok(id)
}

fn claude_config(
    text: &str,
) -> Result<(config::JsonObject<'_>, config::JsonObject<'_>, &'static str)> {
    let root = config::JsonObject::parse(text)?;
    if root.get("apiKeyHelper").is_some() {
        return Err(
            "Claude uses apiKeyHelper; pp-switch does not change external key sources.".into(),
        );
    }
    let env = root.object("env")?;
    for flag in [
        "CLAUDE_CODE_USE_BEDROCK",
        "CLAUDE_CODE_USE_VERTEX",
        "CLAUDE_CODE_USE_FOUNDRY",
    ] {
        if let Some(raw) = env.get(flag) {
            let value: serde_json::Value = serde_json::from_str(raw.get())
                .map_err(|_| "Invalid Claude cloud-provider flag.")?;
            if !matches!(
                value,
                serde_json::Value::Null | serde_json::Value::Bool(false)
            ) && value != "0"
                && value != "false"
                && value != 0
            {
                return Err("Claude cloud-provider mode is outside scope. Use an existing direct API setup.".into());
            }
        }
    }
    let token = env.get("ANTHROPIC_AUTH_TOKEN").is_some();
    let key = env.get("ANTHROPIC_API_KEY").is_some();
    match (token, key) {
        (true, true) => Err(
            "Claude has both AUTH_TOKEN and API_KEY. Resolve the credential source first.".into(),
        ),
        (false, true) => Ok((root, env, "ANTHROPIC_API_KEY")),
        _ => Ok((root, env, "ANTHROPIC_AUTH_TOKEN")),
    }
}

fn codex_provider<'a>(doc: &'a toml_edit::Document<&str>) -> Result<&'a str> {
    let id = doc
        .get("model_provider")
        .and_then(toml_edit::Item::as_str)
        .ok_or(
            "Initialize a custom Codex provider first; pp-switch never changes model_provider.",
        )?;
    let table = doc
        .get("model_providers")
        .and_then(|p| p.get(id))
        .ok_or("Selected Codex provider has no custom configuration.")?;
    if table.get("env_key").is_some() {
        return Err(
            "Codex uses env_key; pp-switch does not change environment credentials.".into(),
        );
    }
    Ok(id)
}

fn codex_file_auth(doc: &toml_edit::Document<&str>, auth: &str) -> Result<()> {
    let id = codex_provider(doc)?;
    if doc["model_providers"][id]
        .get("requires_openai_auth")
        .and_then(toml_edit::Item::as_bool)
        != Some(true)
    {
        return Err("Selected Codex provider is not configured to use file-based API-key auth. pp-switch does not change auth mode.".into());
    }
    if !matches!(
        doc.get("cli_auth_credentials_store")
            .and_then(toml_edit::Item::as_str),
        None | Some("file")
    ) {
        return Err("Codex uses an external/keyring credential store. Only existing file-based API-key auth is supported.".into());
    }
    for field in ["tokens", "access_token"] {
        if let Some(raw) = config::json_value(auth, &[field])? {
            let value: serde_json::Value =
                serde_json::from_str(raw.get()).map_err(|_| "Invalid Codex auth JSON.")?;
            if !value.is_null() {
                return Err("Codex auth.json contains OAuth credentials. pp-switch does not switch auth modes.".into());
            }
        }
    }
    if config::json_string(auth, &["OPENAI_API_KEY"])?
        .filter(|s| !s.is_empty())
        .is_none()
    {
        return Err("Initialize Codex API-key auth first; OPENAI_API_KEY is missing.".into());
    }
    if config::json_string(auth, &["auth_mode"])?.is_some_and(|s| s != "api_key" && s != "apikey") {
        return Err("Codex auth mode is outside the supported API-key setup.".into());
    }
    Ok(())
}

fn codex_bearer_mode(table: &toml_edit::Item) -> Result<()> {
    if table
        .get("requires_openai_auth")
        .and_then(toml_edit::Item::as_bool)
        == Some(true)
    {
        return Err("Codex bearer token setup is incompatible with requires_openai_auth=true. pp-switch does not change auth mode.".into());
    }
    Ok(())
}

pub fn environment_conflict(client: Client) -> Result<()> {
    let names: &[&str] = match client {
        Client::Claude => &[
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_AUTH_TOKEN",
            "ANTHROPIC_API_KEY",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
        ],
        Client::Codex => &["OPENAI_API_KEY", "OPENAI_BASE_URL"],
        Client::Gemini => &[
            "GEMINI_API_KEY",
            "GOOGLE_GEMINI_BASE_URL",
            "GEMINI_BASE_URL",
            "GOOGLE_API_KEY",
            "GOOGLE_GENAI_USE_VERTEXAI",
        ],
        // Grok's env_key is rejected at profile validation. Do not assume that
        // an unrelated XAI_API_KEY overrides an explicit inline api_key.
        Client::Grok | Client::Dsh => &[],
    };
    if let Some(name) = names
        .iter()
        .find(|name| std::env::var_os(name).is_some_and(|s| !s.is_empty()))
    {
        return Err(format!(
            "{name} is set in this process's environment. File edits may be overridden; launch pp-switch without this override."
        ));
    }
    Ok(())
}
