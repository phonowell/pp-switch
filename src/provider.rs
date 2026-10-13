use crate::Result;
use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Client {
    Claude,
    Codex,
    Gemini,
}

impl Client {
    pub const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::Gemini];

    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini CLI",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "gemini" => Ok(Self::Gemini),
            _ => Err("This client is not supported. Choose Claude, Codex or Gemini.".into()),
        }
    }
}

// Deliberately no Debug: a provider contains a credential.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Provider {
    pub id: String,
    pub client: Client,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
}

impl Provider {
    pub fn new(client: Client, name: &str, base_url: &str, api_key: &str) -> Result<Self> {
        let provider = Self {
            id: format!(
                "{:x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| "System clock is before the Unix epoch.")?
                    .as_nanos()
            ),
            client,
            name: name.trim().into(),
            base_url: base_url.trim().into(),
            api_key: api_key.trim().into(),
        };
        provider.validate()?;
        Ok(provider)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.name.len() > 128
            || self.name.chars().any(char::is_control)
        {
            return Err("Name must contain 1–128 bytes without control characters.".into());
        }
        validate_url(&self.base_url)?;
        if self.api_key.trim().is_empty()
            || self.api_key.len() > 8192
            || self.api_key.chars().any(char::is_control)
        {
            return Err("API key is required and must not contain control characters.".into());
        }
        Ok(())
    }
}

pub fn validate_url(value: &str) -> Result<()> {
    let url = Url::parse(value).map_err(|_| "Base URL must be a valid HTTP(S) URL.")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || value.chars().any(char::is_control)
        || value.len() > 8192
    {
        return Err(
            "Base URL must use HTTP(S), with a host and no embedded credentials or fragment."
                .into(),
        );
    }
    Ok(())
}

fn decode_config(raw: &str) -> Result<String> {
    let restored = raw.trim().replace(' ', "+");
    for engine in [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ] {
        if let Ok(bytes) = engine.decode(&restored) {
            return String::from_utf8(bytes).map_err(|_| "Inline config must be UTF-8.".into());
        }
    }
    Err("Inline config is not valid Base64.".into())
}

fn text_at(value: &Value, path: &str) -> Option<String> {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

pub fn import_link(text: &str) -> Result<Provider> {
    if text.len() > 256 * 1024 {
        return Err("Import link is too large (maximum 256 KiB).".into());
    }
    let link = Url::parse(text.trim()).map_err(|_| "Invalid provider link.")?;
    if !matches!(link.scheme(), "ccswitch" | "ppswitch")
        || link.host_str() != Some("v1")
        || link.path() != "/import"
        || link.fragment().is_some()
    {
        return Err("Expected ccswitch://v1/import or ppswitch://v1/import.".into());
    }
    let params: HashMap<String, String> = link.query_pairs().into_owned().collect();
    if params.get("resource").map(String::as_str) != Some("provider") {
        return Err(
            "Only Provider links are supported; MCP, prompts and skills are outside scope.".into(),
        );
    }
    let client = Client::parse(params.get("app").ok_or("Link is missing app.")?)?;
    let name = params.get("name").ok_or("Link is missing name.")?;
    let mut endpoint = params.get("endpoint").filter(|s| !s.is_empty()).cloned();
    let mut key = params.get("apiKey").filter(|s| !s.is_empty()).cloned();

    if let Some(raw) = params.get("config") {
        let config = decode_config(raw)?;
        let format = params
            .get("configFormat")
            .map(String::as_str)
            .unwrap_or("json");
        let (config_endpoint, config_key) = match format {
            "json" => {
                let json: Value = serde_json::from_str(&config)
                    .map_err(|_| "Inline config is not valid JSON.")?;
                match client {
                    Client::Claude => (
                        text_at(&json, "/env/ANTHROPIC_BASE_URL"),
                        text_at(&json, "/env/ANTHROPIC_AUTH_TOKEN")
                            .or_else(|| text_at(&json, "/env/ANTHROPIC_API_KEY")),
                    ),
                    Client::Gemini => (
                        text_at(&json, "/GOOGLE_GEMINI_BASE_URL")
                            .or_else(|| text_at(&json, "/GEMINI_BASE_URL"))
                            .or_else(|| text_at(&json, "/env/GOOGLE_GEMINI_BASE_URL")),
                        text_at(&json, "/GEMINI_API_KEY")
                            .or_else(|| text_at(&json, "/env/GEMINI_API_KEY")),
                    ),
                    Client::Codex => {
                        let toml = json.get("config").and_then(Value::as_str).unwrap_or("");
                        let (url, bearer) = extract_codex(toml)?;
                        (url, text_at(&json, "/auth/OPENAI_API_KEY").or(bearer))
                    }
                }
            }
            "toml" if client == Client::Codex => extract_codex(&config)?,
            "toml" => {
                let doc = config
                    .parse::<toml_edit::DocumentMut>()
                    .map_err(|_| "Inline config is not valid TOML.")?;
                let env = doc.get("env").unwrap_or(doc.as_item());
                let (url_name, key_name) = match client {
                    Client::Claude => ("ANTHROPIC_BASE_URL", "ANTHROPIC_AUTH_TOKEN"),
                    _ => ("GOOGLE_GEMINI_BASE_URL", "GEMINI_API_KEY"),
                };
                (
                    env.get(url_name)
                        .and_then(toml_edit::Item::as_str)
                        .map(str::to_owned),
                    env.get(key_name)
                        .and_then(toml_edit::Item::as_str)
                        .map(str::to_owned),
                )
            }
            _ => return Err("Inline configFormat must be json or toml.".into()),
        };
        endpoint = endpoint.or(config_endpoint);
        key = key.or(config_key);
    } else if params.contains_key("configUrl") && (endpoint.is_none() || key.is_none()) {
        return Err(
            "Remote config URLs are not supported. Use endpoint/apiKey or inline config.".into(),
        );
    }
    let endpoint = endpoint.ok_or("Link is missing the base URL.")?;
    let first = endpoint
        .split(',')
        .map(str::trim)
        .find(|s| !s.is_empty())
        .ok_or("Link has no endpoint.")?;
    Provider::new(
        client,
        name,
        first,
        &key.ok_or("Link is missing the API key.")?,
    )
}

fn extract_codex(config: &str) -> Result<(Option<String>, Option<String>)> {
    let doc = config
        .parse::<toml_edit::DocumentMut>()
        .map_err(|_| "Inline Codex config is not valid TOML.")?;
    let selected = doc.get("model_provider").and_then(toml_edit::Item::as_str);
    let providers = doc.get("model_providers");
    let table = selected
        .and_then(|id| providers.and_then(|p| p.get(id)))
        .or_else(|| {
            providers
                .and_then(toml_edit::Item::as_table_like)
                .and_then(|p| p.iter().next().map(|(_, v)| v))
        });
    let get = |key: &str| {
        table
            .and_then(|p| p.get(key))
            .and_then(toml_edit::Item::as_str)
            .map(str::to_owned)
    };
    Ok((get("base_url"), get("experimental_bearer_token")))
}
