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
    #[serde(
        rename = "grokbuild",
        alias = "grok",
        alias = "grok-build",
        alias = "grok_build"
    )]
    Grok,
    #[serde(rename = "dsh", alias = "deepseek-harness")]
    Dsh,
}

impl Client {
    pub const ALL: [Self; 5] = [
        Self::Claude,
        Self::Codex,
        Self::Gemini,
        Self::Grok,
        Self::Dsh,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini CLI",
            Self::Grok => "Grok Build",
            Self::Dsh => "DeepSeek Harness",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "gemini" => Ok(Self::Gemini),
            "grokbuild" | "grok-build" | "grok_build" | "grok" => Ok(Self::Grok),
            "dsh" | "deepseek-harness" => Ok(Self::Dsh),
            _ => Err(
                "This client is not supported. Choose Claude, Codex, Gemini, Grok or DSH.".into(),
            ),
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
    #[serde(default, skip_serializing_if = "Models::is_empty")]
    pub models: Models,
}

#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Models {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub haiku: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sonnet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opus: Option<String>,
}

impl Models {
    pub fn from_fields(model: &str, haiku: &str, sonnet: &str, opus: &str) -> Self {
        let optional = |s: &str| (!s.trim().is_empty()).then(|| s.trim().to_owned());
        Self {
            model: optional(model),
            haiku: optional(haiku),
            sonnet: optional(sonnet),
            opus: optional(opus),
        }
    }

    pub fn is_empty(&self) -> bool {
        [&self.model, &self.haiku, &self.sonnet, &self.opus]
            .iter()
            .all(|v| v.is_none())
    }

    pub(crate) fn claude_mappings(&self) -> [(&'static str, Option<&str>); 3] {
        [
            (
                "ANTHROPIC_DEFAULT_HAIKU_MODEL",
                self.haiku.as_deref().or(self.model.as_deref()),
            ),
            (
                "ANTHROPIC_DEFAULT_SONNET_MODEL",
                self.sonnet.as_deref().or(self.model.as_deref()),
            ),
            (
                "ANTHROPIC_DEFAULT_OPUS_MODEL",
                self.opus.as_deref().or(self.model.as_deref()),
            ),
        ]
    }

    fn fill_missing(&mut self, other: Self) {
        self.model = self.model.take().or(other.model);
        self.haiku = self.haiku.take().or(other.haiku);
        self.sonnet = self.sonnet.take().or(other.sonnet);
        self.opus = self.opus.take().or(other.opus);
    }

    fn validate(&self, client: Client) -> Result<()> {
        if client != Client::Claude
            && [&self.haiku, &self.sonnet, &self.opus]
                .iter()
                .any(|v| v.is_some())
        {
            return Err("Model mappings are supported only for Claude Code.".into());
        }
        for value in [&self.model, &self.haiku, &self.sonnet, &self.opus]
            .into_iter()
            .flatten()
        {
            if value.is_empty()
                || value.len() > 512
                || value.chars().any(|c| c.is_control() || c.is_whitespace())
            {
                return Err(
                    "Model IDs must contain 1–512 bytes without whitespace or control characters."
                        .into(),
                );
            }
        }
        Ok(())
    }
}

impl Provider {
    /// Names are scoped to the list users see; record identity remains the ID.
    pub fn same_name(&self, client: Client, name: &str) -> bool {
        self.client == client && self.name.trim().to_lowercase() == name.trim().to_lowercase()
    }

    pub fn import_name(&mut self, saved: &[Self]) {
        let original = self.name.clone();
        for number in 2.. {
            if !saved.iter().any(|p| p.same_name(self.client, &self.name)) {
                break;
            }
            let suffix = format!(" ({number})");
            let mut end = original.len().min(128 - suffix.len());
            while !original.is_char_boundary(end) {
                end -= 1;
            }
            self.name = format!("{}{suffix}", original[..end].trim_end());
        }
    }
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
            models: Models::default(),
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
        self.models.validate(self.client)
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
    let mut models = Models::from_fields(
        params.get("model").map_or("", String::as_str),
        params.get("haikuModel").map_or("", String::as_str),
        params.get("sonnetModel").map_or("", String::as_str),
        params.get("opusModel").map_or("", String::as_str),
    );

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
                models.fill_missing(match client {
                    Client::Claude => Models {
                        model: text_at(&json, "/env/ANTHROPIC_MODEL")
                            .or_else(|| text_at(&json, "/model")),
                        haiku: text_at(&json, "/env/ANTHROPIC_DEFAULT_HAIKU_MODEL"),
                        sonnet: text_at(&json, "/env/ANTHROPIC_DEFAULT_SONNET_MODEL"),
                        opus: text_at(&json, "/env/ANTHROPIC_DEFAULT_OPUS_MODEL"),
                    },
                    Client::Gemini => Models {
                        model: text_at(&json, "/GEMINI_MODEL")
                            .or_else(|| text_at(&json, "/env/GEMINI_MODEL"))
                            .or_else(|| text_at(&json, "/config/model/name")),
                        ..Models::default()
                    },
                    _ => Models::default(),
                });
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
                        let (url, bearer, inline_models) = extract_codex(toml)?;
                        models.fill_missing(inline_models);
                        (url, text_at(&json, "/auth/OPENAI_API_KEY").or(bearer))
                    }
                    Client::Grok => {
                        let (url, key, inline_models) =
                            extract_grok(json.get("config").and_then(Value::as_str).unwrap_or(""))?;
                        models.fill_missing(inline_models);
                        (url, key)
                    }
                    Client::Dsh => (text_at(&json, "/baseURL"), text_at(&json, "/apiKey")),
                }
            }
            "toml" if matches!(client, Client::Codex | Client::Grok) => {
                let (url, key, inline_models) = if client == Client::Codex {
                    extract_codex(&config)?
                } else {
                    extract_grok(&config)?
                };
                models.fill_missing(inline_models);
                (url, key)
            }
            "toml" => {
                let doc = config
                    .parse::<toml_edit::DocumentMut>()
                    .map_err(|_| "Inline config is not valid TOML.")?;
                let env = doc.get("env").unwrap_or(doc.as_item());
                let get = |field: &str| {
                    env.get(field)
                        .and_then(toml_edit::Item::as_str)
                        .map(str::to_owned)
                };
                models.fill_missing(if client == Client::Claude {
                    Models {
                        model: get("ANTHROPIC_MODEL").or_else(|| {
                            doc.get("model")
                                .and_then(toml_edit::Item::as_str)
                                .map(str::to_owned)
                        }),
                        haiku: get("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
                        sonnet: get("ANTHROPIC_DEFAULT_SONNET_MODEL"),
                        opus: get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
                    }
                } else {
                    Models {
                        model: get("GEMINI_MODEL"),
                        ..Models::default()
                    }
                });
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
    // Many CCS links contain only the tier mappings. Claude starts with Sonnet
    // by default, so use that explicit mapping instead of retaining a stale
    // provider's default. Never override a default actually supplied by the link.
    if client == Client::Claude && models.model.is_none() {
        models.model = models
            .sonnet
            .clone()
            .or_else(|| models.opus.clone())
            .or_else(|| models.haiku.clone());
    }
    let endpoint = endpoint.ok_or("Link is missing the base URL.")?;
    let first = endpoint
        .split(',')
        .map(str::trim)
        .find(|s| !s.is_empty())
        .ok_or("Link has no endpoint.")?;
    let mut provider = Provider::new(
        client,
        name,
        first,
        &key.ok_or("Link is missing the API key.")?,
    )?;
    provider.models = models;
    provider.validate()?;
    Ok(provider)
}

fn extract_grok(config: &str) -> Result<(Option<String>, Option<String>, Models)> {
    let doc = toml_edit::Document::parse(config)
        .map_err(|_| "Inline Grok Build config is not valid TOML.")?;
    let profiles = doc
        .get("model")
        .and_then(toml_edit::Item::as_table_like)
        .ok_or("Inline Grok Build config has no custom profile.")?;
    let selected = doc
        .get("models")
        .and_then(|m| m.get("default"))
        .and_then(toml_edit::Item::as_str)
        .and_then(|id| profiles.get(id));
    let table =
        if let Some(table) = selected {
            table
        } else {
            let mut entries = profiles.iter();
            match (entries.next(), entries.next()) {
                (Some((_, table)), None) => table,
                _ => return Err(
                    "Inline Grok Build config has multiple profiles; select models.default first."
                        .into(),
                ),
            }
        };
    let get = |key: &str| {
        table
            .get(key)
            .and_then(toml_edit::Item::as_str)
            .map(str::to_owned)
    };
    Ok((
        get("base_url"),
        get("api_key"),
        Models {
            model: get("model"),
            ..Models::default()
        },
    ))
}

fn extract_codex(config: &str) -> Result<(Option<String>, Option<String>, Models)> {
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
    Ok((
        get("base_url"),
        get("experimental_bearer_token"),
        Models {
            model: doc
                .get("model")
                .and_then(toml_edit::Item::as_str)
                .map(str::to_owned),
            ..Models::default()
        },
    ))
}
