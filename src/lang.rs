use bevy::prelude::*;
use std::env;
#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use std::collections::HashSet;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use std::fs;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use std::path::{Path, PathBuf};

#[cfg(feature = "fluent")]
use i18n_fluent::{FluentBundle, FluentResource};

#[cfg(feature = "fluent")]
use unic_langid::LanguageIdentifier;

#[cfg(feature = "properties-lang")]
use std::collections::HashMap;

#[cfg(feature = "properties-lang")]
use i18n_properties::try_split;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use once_cell::sync::Lazy;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlLangSetting {
    Auto,
    Forced(String),
}

/// Global UI language selection.
///
/// Resolution order:
/// 1) `forced` (from `<html lang="...">`)
/// 2) `selected` (set by the app via the resource)
/// 3) `system` (detected from the OS environment)
#[derive(Resource, Debug, Clone)]
pub struct UILang {
    pub forced: Option<String>,
    pub selected: Option<String>,
    pub system: Option<String>,
}

impl Default for UILang {
    fn default() -> Self {
        Self {
            forced: None,
            selected: None,
            system: detect_system_language(),
        }
    }
}

impl UILang {
    pub fn resolved(&self) -> Option<&str> {
        self.forced
            .as_deref()
            .or(self.selected.as_deref())
            .or(self.system.as_deref())
    }

    pub fn set_selected(&mut self, lang: Option<&str>) -> bool {
        let normalized = normalize_lang_tag(lang);
        if self.selected == normalized {
            return false;
        }
        self.selected = normalized;
        true
    }

    pub fn apply_html_lang(&mut self, lang: Option<&str>) -> bool {
        let new_forced = match parse_html_lang(lang) {
            HtmlLangSetting::Auto => None,
            HtmlLangSetting::Forced(tag) => Some(tag),
        };

        if self.forced == new_forced {
            return false;
        }

        self.forced = new_forced;
        true
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct UiLangState {
    pub last_resolved: Option<String>,
    pub last_language_path: Option<String>,
}

fn parse_html_lang(lang: Option<&str>) -> HtmlLangSetting {
    let Some(tag) = normalize_lang_tag(lang) else {
        return HtmlLangSetting::Auto;
    };

    if tag == "auto" || tag == "default" {
        HtmlLangSetting::Auto
    } else {
        HtmlLangSetting::Forced(tag)
    }
}

fn detect_system_language() -> Option<String> {
    const KEYS: [&str; 4] = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"];

    for key in KEYS {
        if let Ok(value) = env::var(key) {
            if let Some(tag) = normalize_env_lang(&value) {
                return Some(tag);
            }
        }
    }

    None
}

fn normalize_env_lang(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let first = trimmed
        .split(|c| c == ':' || c == ';' || c == ',')
        .next()
        .unwrap_or("");
    let first = first.split('.').next().unwrap_or("");

    normalize_lang_tag(Some(first))
}

fn normalize_lang_tag(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }

    let normalized = raw.replace('_', "-").to_lowercase();
    if normalized == "c" || normalized == "posix" {
        return None;
    }

    Some(normalized)
}

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
static PLACEHOLDER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\{\s*([A-Za-z0-9_.:-]+)\s*\}\}").unwrap());

#[cfg(not(any(feature = "fluent", feature = "properties-lang")))]
pub fn localize_html(html: &str, _lang: Option<&str>, _language_path: &str) -> String {
    html.to_string()
}

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
pub fn localize_html(html: &str, lang: Option<&str>, language_path: &str) -> String {
    if !PLACEHOLDER_RE.is_match(html) {
        return html.to_string();
    }

    let Some(normalized) = normalize_lang_tag(lang) else {
        return html.to_string();
    };

    let mut localized = html.to_string();

    #[cfg(feature = "fluent")]
    if let Some(out) = localize_html_fluent(&localized, &normalized, language_path) {
        localized = out;
    }

    #[cfg(feature = "properties-lang")]
    if let Some(out) = localize_html_properties(&localized, &normalized, language_path) {
        localized = out;
    }

    localized
}

#[cfg(feature = "fluent")]
fn localize_html_fluent(html: &str, lang: &str, language_path: &str) -> Option<String> {
    let base = Path::new(language_path);
    let path = find_lang_file(base, lang, "ftl")?;
    let content = fs::read_to_string(path).ok()?;
    let resource = FluentResource::try_new(content).ok()?;
    let langid: LanguageIdentifier = lang.parse().unwrap_or_else(|_| {
        "en-US"
            .parse()
            .expect("Failed to parse fallback language identifier")
    });

    let mut bundle = FluentBundle::new(vec![langid]);
    if bundle.add_resource(resource).is_err() {
        return None;
    }

    let mut errors = Vec::new();
    let localized = PLACEHOLDER_RE
        .replace_all(html, |caps: &regex::Captures| {
            let key = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let Some(message) = bundle.get_message(key) else {
                return caps.get(0).map(|m| m.as_str()).unwrap_or_default().to_string();
            };
            let Some(pattern) = message.value() else {
                return caps.get(0).map(|m| m.as_str()).unwrap_or_default().to_string();
            };
            errors.clear();
            let value = bundle.format_pattern(pattern, None, &mut errors);
            if errors.is_empty() {
                value.to_string()
            } else {
                caps.get(0).map(|m| m.as_str()).unwrap_or_default().to_string()
            }
        })
        .into_owned();

    Some(localized)
}

#[cfg(feature = "properties-lang")]
fn localize_html_properties(html: &str, lang: &str, language_path: &str) -> Option<String> {
    let base = Path::new(language_path);
    let path = find_lang_file(base, lang, "properties")?;
    let content = fs::read_to_string(path).ok()?;
    let map = parse_properties(&content);

    let localized = PLACEHOLDER_RE
        .replace_all(html, |caps: &regex::Captures| {
            let key = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            map.get(key)
                .cloned()
                .unwrap_or_else(|| caps.get(0).map(|m| m.as_str()).unwrap_or_default().to_string())
        })
        .into_owned();

    Some(localized)
}

#[cfg(feature = "properties-lang")]
fn parse_properties(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }

        let separator = if trimmed.contains('=') {
            Some('=')
        } else if trimmed.contains(':') {
            Some(':')
        } else {
            None
        };

        let prop = match separator {
            Some(sep) => try_split(trimmed, Some(sep), None),
            None => try_split(trimmed, None, None),
        };

        if let Some(prop) = prop {
            map.insert(prop.key(), prop.value());
        }
    }

    map
}

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
fn find_lang_file(base: &Path, lang: &str, extension: &str) -> Option<PathBuf> {
    let Some(primary) = find_lang_file_for(base, lang, extension) else {
        let fallback = "en";
        if normalize_lang_tag(Some(lang)) == normalize_lang_tag(Some(fallback)) {
            return None;
        }
        return find_lang_file_for(base, fallback, extension);
    };

    Some(primary)
}

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
fn find_lang_file_for(base: &Path, lang: &str, extension: &str) -> Option<PathBuf> {
    if !base.exists() {
        return None;
    }

    let target = normalize_lang_tag(Some(lang))?;

    for candidate in build_lang_candidates(lang) {
        let path = base.join(format!("{candidate}.{extension}"));
        if path.is_file() {
            return Some(path);
        }
    }

    let entries = fs::read_dir(base).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if ext.to_lowercase() != extension {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(normalized_stem) = normalize_lang_tag(Some(stem)) else {
            continue;
        };

        if normalized_stem == target
            || normalized_stem.starts_with(&format!("{target}-"))
            || normalized_stem.starts_with(&format!("{target}_"))
        {
            return Some(path);
        }
    }

    None
}

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
fn build_lang_candidates(lang: &str) -> Vec<String> {
    let Some(normalized) = normalize_lang_tag(Some(lang)) else {
        return Vec::new();
    };

    let parts: Vec<&str> = normalized.split('-').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    candidates.push(normalized.clone());
    candidates.push(normalized.replace('-', "_"));

    if parts.len() >= 2 {
        let region_upper = parts[1].to_uppercase();
        candidates.push(format!("{}_{}", parts[0], region_upper));
        candidates.push(format!("{}_{}", parts[0], parts[1]));
        candidates.push(format!("{}-{}", parts[0], parts[1]));
    }

    candidates.push(parts[0].to_string());

    let mut seen = HashSet::new();
    candidates.retain(|item| seen.insert(item.clone()));
    candidates
}

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
pub use i18n;

#[cfg(feature = "fluent")]
pub use i18n_fluent;

#[cfg(feature = "properties-lang")]
pub use i18n_properties;
