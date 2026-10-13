use super::{Paths, claude_config, grok_profile};
use crate::{
    Result, config,
    provider::{Client, Models, Provider},
    storage::{self, Change},
};

/// Visible editor suggestion, not a compatibility promise for every endpoint.
pub const CLAUDE_EDITOR_DEFAULT: &str = "claude-sonnet-5";

/// Snapshot of model destinations. Never represents a live client session.
#[derive(Default)]
pub struct ModelState {
    values: Models,
    selector: Option<String>,
    legacy_fast: Option<String>,
    invalid_legacy_fast: bool,
}

impl ModelState {
    fn editor_default(&self, client: Client) -> Option<&str> {
        let model = self.values.model.as_deref().or(self.selector.as_deref());
        if client == Client::Claude {
            match model {
                Some("sonnet") => self.values.sonnet.as_deref().or(model),
                Some("opus") => self.values.opus.as_deref().or(model),
                Some("haiku") => self.values.haiku.as_deref().or(model),
                _ => model.or(self.values.sonnet.as_deref()),
            }
        } else {
            model
        }
    }
    pub(super) fn model(model: Option<String>) -> Self {
        Self {
            values: Models {
                model,
                ..Models::default()
            },
            ..Self::default()
        }
    }
    pub(super) fn claude(root: &config::JsonObject<'_>, env: &config::JsonObject<'_>) -> Self {
        let legacy = env.string("ANTHROPIC_SMALL_FAST_MODEL");
        Self {
            values: Models {
                model: env.string("ANTHROPIC_MODEL").ok().flatten(),
                haiku: env.string("ANTHROPIC_DEFAULT_HAIKU_MODEL").ok().flatten(),
                sonnet: env.string("ANTHROPIC_DEFAULT_SONNET_MODEL").ok().flatten(),
                opus: env.string("ANTHROPIC_DEFAULT_OPUS_MODEL").ok().flatten(),
            },
            selector: root.string("model").ok().flatten(),
            invalid_legacy_fast: legacy.is_err(),
            legacy_fast: legacy.ok().flatten(),
        }
    }
    pub fn matches(&self, client: Client, desired: &Models) -> bool {
        if let Some(model) = &desired.model {
            if self.values.model.as_ref() != Some(model) {
                return false;
            }
            if matches!(client, Client::Claude | Client::Gemini)
                && self.selector.as_ref() != Some(model)
            {
                return false;
            }
        }
        if client == Client::Claude {
            if desired.claude_mappings()[0].1.is_some() && self.invalid_legacy_fast {
                return false;
            }
            for ((_, desired), actual) in desired.claude_mappings().into_iter().zip([
                &self.values.haiku,
                &self.values.sonnet,
                &self.values.opus,
            ]) {
                if desired.is_some() && desired != actual.as_deref() {
                    return false;
                }
            }
            if let (Some(desired), Some(actual)) =
                (desired.claude_mappings()[0].1, self.legacy_fast.as_deref())
                && desired != actual
            {
                return false;
            }
        }
        true
    }
}

impl Paths {
    /// Read-only suggestion for an unsaved editor draft, scoped to its exact
    /// connection. Claude additionally has a visible product default; a new
    /// endpoint or key rotation never inherits a different connection's model.
    pub fn editor_models(&self, provider: &Provider) -> Models {
        let mut models = provider.models.clone();
        if models.model.is_some() {
            return models;
        }
        if provider.client == Client::Claude {
            models.model = models
                .sonnet
                .clone()
                .or_else(|| models.opus.clone())
                .or_else(|| models.haiku.clone());
        }
        if models.model.is_none()
            && let Ok(status) = self.status(provider.client, true)
            && !status.official
            && status
                .connection
                .as_ref()
                .is_some_and(|(url, key)| url == &provider.base_url && key == &provider.api_key)
            && let Some(current) = status.models
        {
            models.model = current.editor_default(provider.client).map(str::to_owned);
        }
        let mut candidate = provider.clone();
        candidate.models = models.clone();
        if candidate.validate().is_err() {
            models = provider.models.clone();
        }
        if provider.client == Client::Claude && models.model.is_none() {
            models.model = Some(CLAUDE_EDITOR_DEFAULT.into());
        }
        models
    }

    pub fn current_models(&self, client: Client) -> Result<ModelState> {
        if client == Client::Dsh {
            return self.dsh_model().map(|model| ModelState {
                values: Models {
                    model,
                    ..Models::default()
                },
                ..ModelState::default()
            });
        }
        let Some(text) = storage::read(&self.settings(client))? else {
            return Ok(ModelState::default());
        };
        let mut state = ModelState::default();
        match client {
            Client::Claude => {
                let (root, env, _) = claude_config(&text)?;
                state = ModelState::claude(&root, &env);
            }
            Client::Codex | Client::Grok => {
                let doc = toml_edit::Document::parse(text.as_str())
                    .map_err(|_| "Configuration is not valid TOML.")?;
                let item = if client == Client::Codex {
                    doc.get("model")
                } else {
                    let id = grok_profile(&doc)?;
                    doc["model"][id].get("model")
                };
                state.values.model = item
                    .map(|v| {
                        v.as_str()
                            .map(str::to_owned)
                            .ok_or_else(|| "Configured model must be a string.".to_owned())
                    })
                    .transpose()?;
            }
            Client::Dsh => unreachable!("DSH handles models in the shared patch"),
            Client::Gemini => {
                self.gemini_mode()?;
                state.values.model = config::dotenv_value(&text, "GEMINI_MODEL")?;
                let settings = storage::read(&self.gemini.join("settings.json"))?
                    .ok_or("Gemini settings are missing.")?;
                state.selector = config::json_string(&settings, &["model", "name"])?;
            }
        }
        Ok(state)
    }

    pub(super) fn apply_models(
        &self,
        client: Client,
        desired: &Models,
        text: &str,
    ) -> Result<(String, Option<Change>)> {
        if desired.is_empty() {
            return Ok((text.to_owned(), None));
        }
        model_environment_conflict(client, desired)?;
        let mut next = text.to_owned();
        let mut extra = None;
        match client {
            Client::Claude => {
                let (_, env, _) = claude_config(text)?;
                if let Some(model) = &desired.model {
                    config::json_string(text, &["model"])?;
                    env.string("ANTHROPIC_MODEL")?;
                    next = config::patch_json(&next, &["model"], model)?;
                    next = config::patch_json(&next, &["env", "ANTHROPIC_MODEL"], model)?;
                }
                for (field, value) in desired.claude_mappings() {
                    if let Some(value) = value {
                        env.string(field)?;
                        next = config::patch_json(&next, &["env", field], value)?;
                    }
                }
                if let Some(haiku) = desired.claude_mappings()[0].1
                    && env.get("ANTHROPIC_SMALL_FAST_MODEL").is_some()
                {
                    env.string("ANTHROPIC_SMALL_FAST_MODEL")?;
                    next =
                        config::patch_json(&next, &["env", "ANTHROPIC_SMALL_FAST_MODEL"], haiku)?;
                }
            }
            Client::Codex => {
                if let Some(model) = &desired.model {
                    let doc = toml_edit::Document::parse(text)
                        .map_err(|_| "Configuration is not valid TOML.")?;
                    next = if doc.get("model").is_some() {
                        config::patch_toml(text, &["model"], model)?
                    } else {
                        let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
                        format!(
                            "model = {}{newline}{text}",
                            toml_edit::Value::from(model.as_str())
                        )
                    };
                }
            }
            Client::Dsh => unreachable!("DSH handles models in the shared patch"),
            Client::Gemini => {
                if let Some(model) = &desired.model {
                    let path = self.gemini.join("settings.json");
                    let before = storage::read(&path)?;
                    let settings = before.as_deref().ok_or("Gemini settings are missing.")?;
                    if config::json_string(settings, &["security", "auth", "selectedType"])?
                        .as_deref()
                        != Some("gemini-api-key")
                    {
                        return Err("Set Gemini /auth to API key first; pp-switch does not change auth mode.".into());
                    }
                    config::json_string(settings, &["model", "name"])?;
                    let after = config::patch_json(settings, &["model", "name"], model)?;
                    next = config::patch_dotenv(text, "GEMINI_MODEL", model)?;
                    extra = Some(Change {
                        path,
                        before,
                        after,
                    });
                }
            }
            Client::Grok => {
                if let Some(model) = &desired.model {
                    let doc = toml_edit::Document::parse(text)
                        .map_err(|_| "Configuration is not valid TOML.")?;
                    next =
                        config::patch_toml(text, &["model", grok_profile(&doc)?, "model"], model)?;
                }
            }
        }
        Ok((next, extra))
    }
}

pub(super) fn model_environment_conflict(client: Client, desired: &Models) -> Result<()> {
    let names: &[&str] = match client {
        Client::Claude => &[
            "ANTHROPIC_MODEL",
            "ANTHROPIC_DEFAULT_HAIKU_MODEL",
            "ANTHROPIC_DEFAULT_SONNET_MODEL",
            "ANTHROPIC_DEFAULT_OPUS_MODEL",
            "ANTHROPIC_SMALL_FAST_MODEL",
        ],
        Client::Gemini => &["GEMINI_MODEL"],
        _ => &[],
    };
    if let Some(name) = names.iter().find(|name| {
        let affected = match **name {
            "ANTHROPIC_MODEL" | "GEMINI_MODEL" => desired.model.is_some(),
            "ANTHROPIC_DEFAULT_HAIKU_MODEL" | "ANTHROPIC_SMALL_FAST_MODEL" => {
                desired.claude_mappings()[0].1.is_some()
            }
            "ANTHROPIC_DEFAULT_SONNET_MODEL" => desired.claude_mappings()[1].1.is_some(),
            "ANTHROPIC_DEFAULT_OPUS_MODEL" => desired.claude_mappings()[2].1.is_some(),
            _ => false,
        };
        affected && std::env::var_os(name).is_some_and(|v| !v.is_empty())
    }) {
        return Err(format!(
            "Environment override {name} takes precedence. Clear it before switching."
        ));
    }
    Ok(())
}
