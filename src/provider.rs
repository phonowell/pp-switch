use crate::Result;
use serde::{Deserialize, Serialize};
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
