//! Value-range edits using serde_json and toml_edit; unrelated bytes stay intact.
use crate::Result;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::ops::Range;

struct Entries<'a>(Vec<(String, &'a RawValue)>);

/// Borrow values from one parsed object for repeated status/credential reads.
/// Keep duplicate-property rejection, including in nested credential objects.
pub(crate) struct JsonObject<'a>(Entries<'a>);

impl<'a> JsonObject<'a> {
    pub(crate) fn parse(text: &'a str) -> Result<Self> {
        serde_json::from_str(text)
            .map(Self)
            .map_err(|_| "Configuration must be a JSON object without duplicate properties.".into())
    }

    pub(crate) fn get(&self, key: &str) -> Option<&'a RawValue> {
        self.0
            .0
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| *value)
    }

    pub(crate) fn object(&self, key: &str) -> Result<Self> {
        Self::parse(self.get(key).map_or("{}", RawValue::get))
    }

    pub(crate) fn string(&self, key: &str) -> Result<Option<String>> {
        self.get(key)
            .map(|v| {
                serde_json::from_str(v.get())
                    .map_err(|_| "Configured URL/key must be a string.".into())
            })
            .transpose()
    }
}

impl<'de> Deserialize<'de> for Entries<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct ObjectVisitor;
        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = Entries<'de>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<M: MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                let mut entries = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, &'de RawValue>()? {
                    if entries.iter().any(|(existing, _)| existing == &key) {
                        return Err(serde::de::Error::custom("duplicate JSON property"));
                    }
                    entries.push((key, value));
                }
                Ok(Entries(entries))
            }
        }
        deserializer.deserialize_map(ObjectVisitor)
    }
}

pub fn json_value<'a>(text: &'a str, path: &[&str]) -> Result<Option<&'a RawValue>> {
    if path.is_empty() {
        return Err("Empty JSON property path.".into());
    }
    let Entries(entries) = serde_json::from_str::<Entries<'_>>(text)
        .map_err(|_| "Configuration must be a JSON object without duplicate properties.")?;
    if let Some((_, value)) = entries.into_iter().find(|(key, _)| key == path[0]) {
        if path.len() == 1 {
            return Ok(Some(value));
        }
        return json_value(value.get(), &path[1..]);
    }
    Ok(None)
}

pub fn json_string(text: &str, path: &[&str]) -> Result<Option<String>> {
    json_value(text, path)?
        .map(|v| {
            serde_json::from_str(v.get()).map_err(|_| "Configured URL/key must be a string.".into())
        })
        .transpose()
}

pub fn patch_json(text: &str, path: &[&str], value: &str) -> Result<String> {
    let replacement =
        serde_json::to_string(value).map_err(|_| "Cannot encode configuration value.")?;
    if let Some(raw) = json_value(text, path)? {
        if serde_json::from_str::<String>(raw.get()).ok().as_deref() == Some(value) {
            return Ok(text.into());
        }
        let start = raw.get().as_ptr() as usize - text.as_ptr() as usize;
        return replace(text, start..start + raw.get().len(), &replacement);
    }
    if path.len() > 1
        && let Some(parent) = json_value(text, &path[..1])?
    {
        let updated = patch_json(parent.get(), &path[1..], value)?;
        let start = parent.get().as_ptr() as usize - text.as_ptr() as usize;
        return replace(text, start..start + parent.get().len(), &updated);
    }
    // Missing parents are created only as the minimal containers for the two fields.
    let mut nested = replacement;
    for key in path.iter().skip(1).rev() {
        nested = format!("{{{}:{nested}}}", serde_json::to_string(key).unwrap());
    }
    let Entries(entries) = serde_json::from_str::<Entries<'_>>(text)
        .map_err(|_| "Configuration must be a JSON object.")?;
    let end = text.rfind('}').ok_or("Invalid JSON object.")?;
    let insert = format!(
        "{}{}:{nested}",
        if entries.is_empty() { "" } else { "," },
        serde_json::to_string(path[0]).unwrap()
    );
    replace(text, end..end, &insert)
}

/// Delete only the named property and its separator; keep unrelated bytes.
pub fn remove_json(text: &str, path: &[&str]) -> Result<String> {
    let key = path.first().ok_or("Empty JSON property path.")?;
    let Entries(entries) = serde_json::from_str::<Entries<'_>>(text)
        .map_err(|_| "Configuration must be a JSON object without duplicate properties.")?;
    let Some(index) = entries.iter().position(|(name, _)| name == key) else {
        return Ok(text.to_owned());
    };
    let raw = entries[index].1.get();
    let start = raw.as_ptr() as usize - text.as_ptr() as usize;
    if path.len() > 1 {
        let next = remove_json(raw, &path[1..])?;
        return replace(text, start..start + raw.len(), &next);
    }
    let end = start + raw.len();
    let range = if index > 0 {
        let previous = entries[index - 1].1.get();
        let previous_end = previous.as_ptr() as usize - text.as_ptr() as usize + previous.len();
        previous_end..end
    } else {
        let begin = text.find('{').ok_or("Invalid JSON object.")? + 1;
        let end = if entries.len() > 1 {
            end + text[end..].find(',').ok_or("Invalid JSON separator.")? + 1
        } else {
            end
        };
        begin..end
    };
    replace(text, range, "")
}

pub fn patch_toml(text: &str, path: &[&str], value: &str) -> Result<String> {
    let doc = toml_edit::Document::parse(text).map_err(|_| "Configuration is not valid TOML.")?;
    let mut item = doc.as_item();
    for part in path {
        item = item
            .get(part)
            .ok_or("Required active provider field is missing; initialize the client first.")?;
    }
    let old = item
        .as_str()
        .ok_or("Configured URL/key must be a string.")?;
    if old == value {
        return Ok(text.into());
    }
    let span = item.span().ok_or("Cannot locate TOML value.")?;
    replace(text, span, &toml_edit::Value::from(value).to_string())
}

pub fn replace(text: &str, range: Range<usize>, replacement: &str) -> Result<String> {
    if text.get(range.clone()).is_none() {
        return Err("Invalid configuration value range.".into());
    }
    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..range.start]);
    out.push_str(replacement);
    out.push_str(&text[range.end..]);
    Ok(out)
}

pub fn dotenv_value(text: &str, name: &str) -> Result<Option<String>> {
    let mut found = None;
    for line in text.lines() {
        if let Some((key, value)) = dotenv_assignment(line)
            && key == name
        {
            if found.is_some() {
                return Err("Duplicate URL/key assignments in .env; resolve them first.".into());
            }
            found = Some(decode_dotenv(value)?);
        }
    }
    Ok(found)
}

fn dotenv_assignment(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_start();
    let line = line
        .strip_prefix("export ")
        .map(str::trim_start)
        .unwrap_or(line);
    let (key, value) = line.split_once('=')?;
    if key.starts_with('#') {
        return None;
    }
    Some((key.trim_end(), value.trim_start()))
}

fn value_end(value: &str) -> Result<usize> {
    if let Some(quote) = value.chars().next().filter(|c| *c == '\'' || *c == '"') {
        let mut escaped = false;
        for (index, c) in value.char_indices().skip(1) {
            if c == quote && !escaped {
                return Ok(index + 1);
            }
            escaped = c == '\\' && !escaped;
        }
        return Err("Multiline or unterminated URL/key in .env is not supported.".into());
    }
    Ok(value.find('#').unwrap_or(value.len()))
}

fn decode_dotenv(value: &str) -> Result<String> {
    let raw = value[..value_end(value)?].trim();
    if raw.starts_with('"') {
        serde_json::from_str(raw).map_err(|_| "Unsupported escaping in .env URL/key.".into())
    } else if raw.starts_with('\'') {
        Ok(raw[1..raw.len() - 1].into())
    } else {
        Ok(raw.into())
    }
}

pub fn patch_dotenv(text: &str, name: &str, value: &str) -> Result<String> {
    let current = dotenv_value(text, name)?;
    if current.as_deref() == Some(value) {
        return Ok(text.into());
    }
    let encoded = serde_json::to_string(value).map_err(|_| "Cannot encode .env value.")?;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        if let Some((key, raw)) = dotenv_assignment(content)
            && key == name
        {
            let start = offset + raw.as_ptr() as usize - content.as_ptr() as usize;
            let end = value_end(raw)?;
            let end = raw[..end].trim_end().len();
            return replace(text, start..start + end, &encoded);
        }
        offset += line.len();
    }
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    Ok(format!(
        "{text}{}{name}={encoded}{newline}",
        if text.is_empty() || text.ends_with('\n') {
            ""
        } else {
            newline
        }
    ))
}

pub fn remove_dotenv(text: &str, name: &str) -> Result<String> {
    dotenv_value(text, name)?;
    Ok(text
        .split_inclusive('\n')
        .filter(|line| {
            dotenv_assignment(line.trim_end_matches(['\r', '\n']))
                .is_none_or(|(key, _)| key != name)
        })
        .collect())
}
