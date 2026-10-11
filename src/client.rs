use crate::{
    Result, config,
    provider::{Client, Provider},
    storage::{self, Change, Store},
};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Paths {
    pub claude: PathBuf,
    pub codex: PathBuf,
    pub gemini: PathBuf,
}

impl Paths {
    pub fn from_home(home: PathBuf) -> Self {
        Self {
            claude: home.join(".claude"),
            codex: home.join(".codex"),
            gemini: home.join(".gemini"),
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
        }
    }

    pub fn current(&self, client: Client) -> Result<Option<(String, String)>> {
        let Some(text) = storage::read(&self.settings(client))? else {
            return Ok(None);
        };
        let values = match client {
            Client::Claude => {
                let field = claude_key_field(&text)?;
                (
                    config::json_string(&text, &["env", "ANTHROPIC_BASE_URL"])?,
                    config::json_string(&text, &["env", field])?,
                )
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
        let path = self.settings(provider.client);
        let before = storage::read(&path)?;
        let text = before.as_deref().unwrap_or(match provider.client {
            Client::Claude => "{}",
            _ => "",
        });
        let mut changes = Vec::new();
        let after = match provider.client {
            Client::Claude => {
                let field = claude_key_field(text)?;
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
        };
        changes.push(Change {
            path,
            before,
            after,
        });
        store.commit(changes)
    }
}

fn claude_key_field(text: &str) -> Result<&'static str> {
    if config::json_value(text, &["apiKeyHelper"])?.is_some() {
        return Err(
            "Claude uses apiKeyHelper; pp-switch does not change external key sources.".into(),
        );
    }
    for flag in [
        "CLAUDE_CODE_USE_BEDROCK",
        "CLAUDE_CODE_USE_VERTEX",
        "CLAUDE_CODE_USE_FOUNDRY",
    ] {
        if let Some(raw) = config::json_value(text, &["env", flag])? {
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
    let token = config::json_value(text, &["env", "ANTHROPIC_AUTH_TOKEN"])?.is_some();
    let key = config::json_value(text, &["env", "ANTHROPIC_API_KEY"])?.is_some();
    match (token, key) {
        (true, true) => Err(
            "Claude has both AUTH_TOKEN and API_KEY. Resolve the credential source first.".into(),
        ),
        (false, true) => Ok("ANTHROPIC_API_KEY"),
        _ => Ok("ANTHROPIC_AUTH_TOKEN"),
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
