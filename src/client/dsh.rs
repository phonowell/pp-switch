//! DSH's home patch is shared by CLI and Web; never rewrite either profile.
use super::{ConfigStatus, ModelState, Paths};
use crate::{
    Result,
    provider::Provider,
    storage::{self, Change, Store},
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use yaml_edit::{Mapping, Sequence, SyntaxKind, YamlFile, YamlNode};

const SETUP: &str = "Initialize DSH headless and web profiles with the base bundle first.";
const YAML: &str =
    "DSH YAML must be a single valid document without duplicate keys, aliases, anchors or tags.";

fn string(node: &YamlNode) -> Result<String> {
    let scalar = node.as_scalar().ok_or(YAML)?;
    let raw = scalar.value();
    // Bound this integration to single-line string scalars. JSON double quotes
    // are also YAML; unsupported YAML-only escapes fail rather than misread keys.
    if raw.contains(['\n', '\r']) || raw.is_empty() {
        return Err(YAML.into());
    }
    if raw.starts_with('"') {
        return serde_json::from_str(&raw).map_err(|_| YAML.into());
    }
    if raw.starts_with('\'') && raw.ends_with('\'') {
        return Ok(raw[1..raw.len() - 1].replace("''", "'"));
    }
    if yaml_edit::lex(&raw).iter().any(|(kind, _)| {
        matches!(
            kind,
            SyntaxKind::INT | SyntaxKind::FLOAT | SyntaxKind::BOOL | SyntaxKind::NULL
        )
    }) {
        return Err(YAML.into());
    }
    Ok(scalar.as_string())
}
fn optional(map: &Mapping, key: &str) -> Result<Option<String>> {
    map.get(key).map(|n| string(&n)).transpose()
}
fn mapping(map: &Mapping, key: &str) -> Result<Option<Mapping>> {
    map.get(key)
        .map(|n| n.as_mapping().cloned().ok_or_else(|| YAML.to_owned()))
        .transpose()
}
fn validate(node: &YamlNode, depth: usize) -> Result<()> {
    if depth > 64 {
        return Err(YAML.into());
    }
    match node {
        YamlNode::Mapping(map) => {
            let mut keys = HashSet::new();
            for (key, value) in map.iter() {
                if !keys.insert(string(&key)?) {
                    return Err(YAML.into());
                }
                validate(&value, depth + 1)?;
            }
        }
        YamlNode::Sequence(seq) => {
            for value in seq.values() {
                validate(&value, depth + 1)?;
            }
        }
        YamlNode::Scalar(_) => (),
        _ => return Err(YAML.into()),
    }
    Ok(())
}
fn parse(text: &str) -> Result<YamlFile> {
    // Avoid letting deeply nested or executable input reach the recursive parser.
    let mut depth = 0usize;
    for (kind, _) in yaml_edit::lex(text) {
        match kind {
            SyntaxKind::ANCHOR | SyntaxKind::REFERENCE | SyntaxKind::TAG => return Err(YAML.into()),
            SyntaxKind::LEFT_BRACKET | SyntaxKind::LEFT_BRACE => {
                depth += 1;
                if depth > 64 {
                    return Err(YAML.into());
                }
            }
            SyntaxKind::RIGHT_BRACKET | SyntaxKind::RIGHT_BRACE => depth = depth.saturating_sub(1),
            _ => (),
        }
    }
    if text
        .lines()
        .any(|line| line.len() - line.trim_start_matches(' ').len() > 128)
    {
        return Err(YAML.into());
    }
    let parsed = YamlFile::parse(text);
    if parsed.has_errors() {
        return Err(YAML.into());
    }
    let file = parsed.tree();
    if file.documents().count() != 1 || file.directives().next().is_some() {
        return Err(YAML.into());
    }
    let doc = file.document().ok_or(YAML)?;
    let node = if let Some(m) = doc.as_mapping() {
        YamlNode::Mapping(m)
    } else if let Some(s) = doc.as_sequence() {
        YamlNode::Sequence(s)
    } else {
        return Err(YAML.into());
    };
    validate(&node, 0)?;
    Ok(file)
}
struct Patches {
    file: YamlFile,
    sequence: Sequence,
    configs: HashMap<String, Mapping>,
}
impl Patches {
    fn parse(text: &str) -> Result<Self> {
        let file = parse(text)?;
        let sequence = file.document().and_then(|d| d.as_sequence()).ok_or(YAML)?;
        let mut configs = HashMap::new();
        let mut ids = HashSet::new();
        for node in sequence.values() {
            let entry = node.as_mapping().ok_or(YAML)?;
            let id = optional(entry, "id")?.ok_or(SETUP)?;
            if !ids.insert(id.clone()) {
                return Err("DSH patch contains duplicate entry IDs.".into());
            }
            // Arbitrary patch operations can disable, replace or insert the target.
            if entry
                .keys()
                .any(|k| string(&k).map_or(true, |k| k != "id" && k != "config"))
            {
                return Err(
                    "DSH patch operations outside id/config overrides are unsupported.".into(),
                );
            }
            if let Some(config) = mapping(entry, "config")? {
                configs.insert(id, config);
            }
        }
        Ok(Self {
            file,
            sequence,
            configs,
        })
    }
    fn config(&self, id: &str) -> Option<Mapping> {
        self.configs.get(id).cloned()
    }
    fn writable_config(&mut self, id: &str, inherited: &Mapping, indent: usize) -> Result<Mapping> {
        if let Some(config) = self.config(id) {
            return Ok(config);
        }
        // Copy the WHOLE effective config: Cordis replaces config, it doesn't
        // deep-merge it. A partial route override would drop catalogs/settings.
        // Relocate a validated mapping span; don't reserialize its contents.
        // yaml-edit's nested-node builders currently lose indentation here.
        let raw = inherited.to_string();
        let addition = if raw.trim_start().starts_with('{') {
            if raw.contains(['\r', '\n']) {
                return Err(YAML.into());
            }
            format!("- id: {id}\n  config: {raw}\n")
        } else {
            let mut relocated = String::new();
            for (index, line) in raw.lines().enumerate() {
                let line = if index == 0 || line.trim().is_empty() {
                    line
                } else {
                    line.strip_prefix(&" ".repeat(indent)).ok_or(YAML)?
                };
                relocated.push_str("    ");
                relocated.push_str(line);
                relocated.push('\n');
            }
            format!("- id: {id}\n  config:\n{relocated}")
        };
        let range = self.sequence.byte_range();
        let mut text = self.file.to_string();
        if self.sequence.is_empty() {
            text.replace_range(range.start as usize..range.end as usize, &addition);
        } else {
            if self.sequence.to_string().trim_start().starts_with('[') {
                return Err("DSH shared patches must use a block-style list when adding an inherited override.".into());
            }
            text.insert_str(range.end as usize, &format!("\n{addition}\n"));
        }
        *self = Self::parse(&text)?;
        self.config(id).ok_or_else(|| YAML.to_owned())
    }
}
struct Shared {
    common: Patches,
    common_before: Option<String>,
    profiles: [Patches; 2],
    selection: Mapping,
    selection_indent: usize,
    guards: Vec<(PathBuf, Option<String>)>,
}
struct Snapshot {
    common: Patches,
    common_before: Option<String>,
    credentials: YamlFile,
    credentials_before: Option<String>,
    from_account: bool,
    route_config: Mapping,
    route_indent: usize,
    selection: Mapping,
    selection_indent: usize,
    entry_id: &'static str,
    route: String,
    reference: String,
    key: String,
    url: String,
    model: String,
    guards: Vec<(PathBuf, Option<String>)>,
}
fn column(map: &Mapping, source: &Patches) -> usize {
    let text = source.file.to_string();
    let start = map.byte_range().start as usize;
    start - text[..start].rfind('\n').map_or(0, |n| n + 1)
}
fn merged_config(
    common: &Patches,
    profiles: &[Patches; 2],
    id: &str,
    default: Mapping,
) -> Result<(Mapping, usize)> {
    if let Some(c) = common.config(id) {
        let indent = column(&c, common);
        return Ok((c, indent));
    }
    let a = profiles[0].config(id).unwrap_or_else(|| default.clone());
    let b = profiles[1].config(id).unwrap_or(default);
    if !yaml_edit::yaml_eq(&a, &b) {
        return Err("DSH CLI and Web have different route settings. Configure a common home-level route first; PPS preserves both profiles.".into());
    }
    let indent = if profiles[0].config(id).is_some() {
        column(&a, &profiles[0])
    } else {
        0
    };
    Ok((a, indent))
}
impl Paths {
    fn dsh_shared(&self) -> Result<Shared> {
        let mut guards = Vec::new();
        let mut profiles = Vec::new();
        for name in ["headless", "web"] {
            let dir = self.dsh.join("profiles").join(name);
            let manifest_path = dir.join("package.json");
            let manifest = storage::read(&manifest_path)?.ok_or(SETUP)?;
            let json: serde_json::Value = serde_json::from_str(&manifest).map_err(|_| SETUP)?;
            if !json
                .pointer("/dsh/profile/bundles")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|v| {
                    v.iter()
                        .any(|p| p.as_str() == Some("@deepseek-ai/dsh-base"))
                })
            {
                return Err(SETUP.into());
            }
            guards.push((manifest_path, Some(manifest)));
            let patch_path = dir.join("cordis.patch.yml");
            let patch = storage::read(&patch_path)?;
            profiles.push(Patches::parse(patch.as_deref().unwrap_or("[]"))?);
            guards.push((patch_path, patch));
        }
        let profiles: [Patches; 2] = profiles.try_into().ok().expect("two profiles");
        let common_before = storage::read(&self.dsh.join("cordis.patch.yml"))?;
        let common = Patches::parse(common_before.as_deref().unwrap_or("[]"))?;
        for p in [&common, &profiles[0], &profiles[1]] {
            if let Some(c) = p.config("credentials")
                && (c.get("path").is_some() || c.get("dshHome").is_some())
            {
                return Err("DSH uses an external credential store; only the default home store is supported.".into());
            }
        }
        let defaults = Mapping::new();
        defaults.set("provider", "deepseek-official");
        defaults.set("model", "deepseek-flash");
        let (selection, selection_indent) =
            merged_config(&common, &profiles, "agent-default-model", defaults)?;
        Ok(Shared {
            common,
            common_before,
            profiles,
            selection,
            selection_indent,
            guards,
        })
    }
    fn dsh_snapshot(&self, allow_account: bool) -> Result<Snapshot> {
        self.dsh_snapshot_from_shared(self.dsh_shared()?, allow_account)
    }
    fn dsh_snapshot_from_shared(&self, shared: Shared, allow_account: bool) -> Result<Snapshot> {
        let Shared {
            common,
            common_before,
            profiles,
            selection,
            selection_indent,
            guards,
        } = shared;
        let mut route = optional(&selection, "provider")?
            .filter(|s| !s.is_empty())
            .ok_or(SETUP)?;
        let mut model = optional(&selection, "model")?
            .filter(|s| !s.is_empty())
            .ok_or(SETUP)?;
        let from_account = route == "deepseek-account";
        if from_account {
            if !allow_account {
                return Err("DSH uses native account authentication.".into());
            }
            route = "deepseek-official".to_owned();
            model = "deepseek-flash".to_owned();
        }
        let entry_id = if route == "deepseek-official" {
            "llm-deepseek"
        } else {
            "llm-pi-ai"
        };
        let (route_config, route_indent) =
            merged_config(&common, &profiles, entry_id, Mapping::new())?;
        let destination = if entry_id == "llm-deepseek" {
            route_config.clone()
        } else {
            mapping(&route_config, "providers")?
                .and_then(|p| p.get_mapping(&route))
                .ok_or(SETUP)?
        };
        // An explicit reference is mandatory for pi-ai; otherwise it can select
        // OAuth/native ambient auth. The native route's API-key default is fixed.
        let reference = optional(&destination, "apiKeyEnv")?
            .or_else(|| (entry_id == "llm-deepseek").then(|| "DEEPSEEK_API_KEY".to_owned()))
            .filter(|s| {
                !s.is_empty()
                    && s.len() <= 128
                    && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
            .ok_or(SETUP)?;
        if destination.get("headers").is_some() {
            return Err("DSH routes with custom headers are unsupported; use an explicit API-key reference.".into());
        }
        let url = optional(&destination, "baseURL")?
            .or_else(|| {
                (entry_id == "llm-deepseek")
                    .then(|| "https://api.deepseek.com/anthropic".to_owned())
            })
            .filter(|s| !s.is_empty())
            .ok_or(SETUP)?;
        let credentials_before = storage::read(&self.dsh.join(".credentials.yaml"))?;
        let credentials_text = match credentials_before.as_deref() {
            Some(text) => text,
            None if from_account => "version: 1\nrefs: {}\n",
            None => {
                return Err(
                    "Store a DSH API key first; PPS does not initialize credentials.".into(),
                );
            }
        };
        let credentials = parse(credentials_text)?;
        let root = credentials
            .document()
            .and_then(|d| d.as_mapping())
            .ok_or(YAML)?;
        if root
            .get("version")
            .and_then(|v| v.as_scalar().map(|s| s.value()))
            .as_deref()
            != Some("1")
        {
            return Err("DSH credentials must use version 1.".into());
        }
        let refs = match mapping(&root, "refs")? {
            Some(refs) => refs,
            None if from_account => {
                let refs = Mapping::new();
                root.set("refs", refs);
                mapping(&root, "refs")?.ok_or(YAML)?
            }
            None => return Err(SETUP.into()),
        };
        let key = optional(&refs, &reference)?.filter(|s| !s.is_empty());
        let key = match key {
            Some(key) => key,
            None if from_account => String::new(),
            None => return Err(
                "Store the selected DSH API-key reference first; PPS preserves account records."
                    .into(),
            ),
        };
        Ok(Snapshot {
            common,
            common_before,
            credentials,
            credentials_before,
            from_account,
            route_config,
            route_indent,
            selection,
            selection_indent,
            entry_id,
            route,
            reference,
            key,
            url,
            model,
            guards,
        })
    }
    pub(super) fn dsh_is_official(&self) -> Result<bool> {
        let s = self.dsh_shared()?;
        if optional(&s.selection, "provider")?.as_deref() != Some("deepseek-account") {
            return Ok(false);
        }
        let (account, _) = merged_config(
            &s.common,
            &s.profiles,
            "llm-deepseek-account",
            Mapping::new(),
        )?;
        optional(&account, "baseURL")?;
        Ok(account.get("baseURL").is_none())
    }
    pub(super) fn apply_dsh_official(&self, store: &Store) -> Result<()> {
        let mut s = self.dsh_shared()?;
        let (account, indent) = merged_config(
            &s.common,
            &s.profiles,
            "llm-deepseek-account",
            Mapping::new(),
        )?;
        optional(&account, "baseURL")?;
        if std::env::var_os("DEEPSEEK_BASE_URL").is_some_and(|v| !v.is_empty()) {
            return Err(
                "DSH has an inherited URL/key environment override. Clear it before switching."
                    .into(),
            );
        }
        let selection =
            s.common
                .writable_config("agent-default-model", &s.selection, s.selection_indent)?;
        selection.set("provider", "deepseek-account");
        selection.set("model", "deepseek-flash");
        if account.get("baseURL").is_some() {
            let writable = s
                .common
                .writable_config("llm-deepseek-account", &account, indent)?;
            writable.remove("baseURL");
        }
        let after = preserve_newlines(
            s.common.file.to_string(),
            s.common_before.as_deref().unwrap_or(""),
        );
        Patches::parse(&after)?;
        store.commit_guarded(
            vec![Change {
                path: self.dsh.join("cordis.patch.yml"),
                before: s.common_before,
                after,
            }],
            &s.guards,
        )
    }
    pub(super) fn dsh_status(&self, include_models: bool) -> Result<ConfigStatus> {
        let shared = self.dsh_shared()?;
        if optional(&shared.selection, "provider")?.as_deref() == Some("deepseek-account") {
            let (account, _) = merged_config(
                &shared.common,
                &shared.profiles,
                "llm-deepseek-account",
                Mapping::new(),
            )?;
            optional(&account, "baseURL")?;
            return Ok(ConfigStatus {
                official: account.get("baseURL").is_none(),
                ..ConfigStatus::default()
            });
        }
        let s = self.dsh_snapshot_from_shared(shared, false)?;
        Ok(ConfigStatus {
            connection: Some((s.url, s.key)),
            models: include_models.then(|| ModelState::model(Some(s.model))),
            official: false,
        })
    }
    pub(super) fn dsh_current(&self) -> Result<Option<(String, String)>> {
        let shared = self.dsh_shared()?;
        if optional(&shared.selection, "provider")?.as_deref() == Some("deepseek-account") {
            return Ok(None);
        }
        let s = self.dsh_snapshot_from_shared(shared, false)?;
        Ok(Some((s.url, s.key)))
    }
    pub(super) fn dsh_model(&self) -> Result<Option<String>> {
        Ok(Some(self.dsh_snapshot(false)?.model))
    }
    pub(super) fn apply_dsh(&self, store: &Store, provider: &Provider) -> Result<()> {
        let mut s = self.dsh_snapshot(true)?;
        if std::env::var_os(&s.reference).is_some_and(|v| !v.is_empty())
            || (s.entry_id == "llm-deepseek"
                && std::env::var_os("DEEPSEEK_BASE_URL").is_some_and(|v| !v.is_empty()))
        {
            return Err(
                "DSH has an inherited URL/key environment override. Clear it before switching."
                    .into(),
            );
        }
        if !s.from_account
            && s.url == provider.base_url
            && s.key == provider.api_key
            && provider.models.model.as_ref().is_none_or(|m| m == &s.model)
        {
            return Ok(());
        }
        let config = s
            .common
            .writable_config(s.entry_id, &s.route_config, s.route_indent)?;
        let dest = if s.entry_id == "llm-deepseek" {
            config
        } else {
            config
                .get_mapping("providers")
                .and_then(|p| p.get_mapping(&s.route))
                .ok_or(SETUP)?
        };
        dest.set("baseURL", provider.base_url.as_str());
        // Keep the inherited native reference explicit when making a common
        // config; neither the reference identity nor authentication mode changes.
        if dest.get("apiKeyEnv").is_none() {
            dest.set("apiKeyEnv", s.reference.as_str());
        }
        if s.from_account {
            let selection = s.common.writable_config(
                "agent-default-model",
                &s.selection,
                s.selection_indent,
            )?;
            selection.set("provider", "deepseek-official");
            selection.set(
                "model",
                provider.models.model.as_deref().unwrap_or("deepseek-flash"),
            );
        }
        if let Some(model) = &provider.models.model {
            let selection = s.common.writable_config(
                "agent-default-model",
                &s.selection,
                s.selection_indent,
            )?;
            selection.set("model", model.as_str());
        }
        let root = s
            .credentials
            .document()
            .and_then(|d| d.as_mapping())
            .ok_or(YAML)?;
        root.get_mapping("refs")
            .ok_or(YAML)?
            .set(s.reference.as_str(), provider.api_key.as_str());
        let patch_after = preserve_newlines(
            s.common.file.to_string(),
            s.common_before.as_deref().unwrap_or(""),
        );
        let credentials_after = preserve_newlines(
            s.credentials.to_string(),
            s.credentials_before.as_deref().unwrap_or(""),
        );
        Patches::parse(&patch_after)?;
        parse(&credentials_after)?;
        store.commit_guarded(
            vec![
                Change {
                    path: self.dsh.join("cordis.patch.yml"),
                    before: s.common_before,
                    after: patch_after,
                },
                Change {
                    path: self.dsh.join(".credentials.yaml"),
                    before: s.credentials_before,
                    after: credentials_after,
                },
            ],
            &s.guards,
        )
    }
}
fn preserve_newlines(next: String, before: &str) -> String {
    if before.contains("\r\n") && !before.replace("\r\n", "").contains('\n') {
        next.replace("\r\n", "\n").replace('\n', "\r\n")
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appended_common_overrides_reparse() {
        let mut p = Patches::parse("[]").unwrap();
        let source = Mapping::new();
        let c = p.writable_config("llm-deepseek", &source, 0).unwrap();
        c.set("baseURL", "https://fixture.example");
        c.set("apiKeyEnv", "DEEPSEEK_API_KEY");
        let text = p.file.to_string();
        assert!(Patches::parse(&text).is_ok(), "synthetic output: {text}");
    }
    #[test]
    fn appended_inherited_config_reparses() {
        let mut p = Patches::parse("[]").unwrap();
        let inherited=parse("providers:\n  relay:\n    baseURL: https://old.example\n    apiKeyEnv: RELAY_KEY\n    models: [{id: old, contextWindow: 200000}]\n").unwrap().document().unwrap().as_mapping().unwrap();
        let c = p.writable_config("llm-pi-ai", &inherited, 0).unwrap();
        c.get_mapping("providers")
            .unwrap()
            .get_mapping("relay")
            .unwrap()
            .set("baseURL", "https://new.example");
        let selection = Mapping::new();
        selection.set("provider", "relay");
        selection.set("model", "old");
        let c = p
            .writable_config("agent-default-model", &selection, 0)
            .unwrap();
        c.set("model", "new");
        let text = p.file.to_string();
        assert!(Patches::parse(&text).is_ok(), "synthetic output: {text}");
    }
}
