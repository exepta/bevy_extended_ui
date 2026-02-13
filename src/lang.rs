use bevy::prelude::*;
use std::collections::HashMap;
#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use std::collections::HashSet;
use std::env;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use std::fs;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use std::path::{Path, PathBuf};

#[cfg(feature = "fluent")]
use i18n_fluent::{FluentBundle, FluentResource};

#[cfg(feature = "fluent")]
use unic_langid::LanguageIdentifier;

#[cfg(feature = "properties-lang")]
use i18n_properties::try_split;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use once_cell::sync::Lazy;

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
use regex::Regex;

/// Represents the source of the HTML `lang` setting after parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlLangSetting {
    /// Let the system and app selection decide the language.
    Auto,
    /// Force a specific language tag from the HTML attribute.
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
    /// Creates a language state seeded with the detected system language.
    fn default() -> Self {
        Self {
            forced: None,
            selected: None,
            system: detect_system_language(),
        }
    }
}

impl Default for UiLangState {
    /// Creates a fresh language resolution cache state.
    fn default() -> Self {
        Self {
            last_resolved: None,
            last_language_path: None,
            last_vars_fingerprint: None,
        }
    }
}

impl UILang {
    /// Returns the currently resolved language tag in priority order.
    pub fn resolved(&self) -> Option<&str> {
        self.forced
            .as_deref()
            .or(self.selected.as_deref())
            .or(self.system.as_deref())
    }

    /// Sets the selected language and returns whether it changed.
    pub fn set_selected(&mut self, lang: Option<&str>) -> bool {
        let normalized = normalize_lang_tag(lang);
        if self.selected == normalized {
            return false;
        }
        self.selected = normalized;
        true
    }

    /// Applies a `lang` value from HTML and returns whether it changed the forced state.
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

/// Cached resolution information for language resources.
#[derive(Resource, Debug, Clone)]
pub struct UiLangState {
    pub last_resolved: Option<String>,
    pub last_language_path: Option<String>,
    pub last_vars_fingerprint: Option<u64>,
}

/// Runtime variables used for placeholder substitution during localization.
#[derive(Resource, Debug, Default, Clone)]
pub struct UiLangVariables {
    pub vars: HashMap<String, String>,
}

impl UiLangVariables {
    /// Inserts or replaces a localization variable.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(key.into(), value.into());
    }
}

/// Parses an HTML `lang` attribute into a normalized setting.
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

/// Detects the system language from environment variables.
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

/// Normalizes an environment language string into a language tag.
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

/// Normalizes a raw language tag into lowercase hyphenated form.
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
static PLACEHOLDER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)\{\{\s*(.+?)\s*\}\}").unwrap());

/// Returns the input HTML unchanged when no localization backends are enabled.
#[cfg(not(any(feature = "fluent", feature = "properties-lang")))]
pub fn localize_html(
    html: &str,
    _lang: Option<&str>,
    _language_path: &str,
    _vars: &UiLangVariables,
) -> String {
    html.to_string()
}

/// Localizes HTML placeholders using the enabled backend(s).
#[cfg(any(feature = "fluent", feature = "properties-lang"))]
pub fn localize_html(
    html: &str,
    lang: Option<&str>,
    language_path: &str,
    vars: &UiLangVariables,
) -> String {
    if !PLACEHOLDER_RE.is_match(html) {
        return html.to_string();
    }

    let Some(normalized) = normalize_lang_tag(lang) else {
        return html.to_string();
    };

    let mut localized = html.to_string();

    #[cfg(feature = "fluent")]
    if let Some(out) = localize_html_fluent(&localized, &normalized, language_path, vars) {
        localized = out;
    }

    #[cfg(feature = "properties-lang")]
    if let Some(out) = localize_html_properties(&localized, &normalized, language_path, vars) {
        localized = out;
    }

    localized
}

/// Localizes HTML using Fluent files if available.
#[cfg(feature = "fluent")]
fn localize_html_fluent(
    html: &str,
    lang: &str,
    language_path: &str,
    vars: &UiLangVariables,
) -> Option<String> {
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
            let inner = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let resolved = resolve_placeholder(
                inner,
                |key| {
                    let message = bundle.get_message(key)?;
                    let pattern = message.value()?;
                    errors.clear();
                    let value = bundle.format_pattern(pattern, None, &mut errors);
                    if errors.is_empty() {
                        Some(value.to_string())
                    } else {
                        None
                    }
                },
                vars,
            );
            resolved.unwrap_or_else(|| {
                caps.get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string()
            })
        })
        .into_owned();

    Some(localized)
}

/// Localizes HTML using Java properties files if available.
#[cfg(feature = "properties-lang")]
fn localize_html_properties(
    html: &str,
    lang: &str,
    language_path: &str,
    vars: &UiLangVariables,
) -> Option<String> {
    let base = Path::new(language_path);
    let path = find_lang_file(base, lang, "properties")?;
    let content = fs::read_to_string(path).ok()?;
    let map = parse_properties(&content);

    let localized = PLACEHOLDER_RE
        .replace_all(html, |caps: &regex::Captures| {
            let inner = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let resolved = resolve_placeholder(inner, |key| map.get(key).cloned(), vars);
            resolved.unwrap_or_else(|| {
                caps.get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string()
            })
        })
        .into_owned();

    Some(localized)
}

/// Parses a .properties file into a key/value map.
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

/// Finds the most suitable language file, falling back to English if needed.
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

/// Locates a language file by checking candidate filenames and directory entries.
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

/// Builds a list of candidate language tags for lookup.
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

/// Computes a stable fingerprint for the current localization variables.
pub fn vars_fingerprint(vars: &UiLangVariables) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut items: Vec<_> = vars.vars.iter().collect();
    items.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut hasher = DefaultHasher::new();
    for (key, value) in items {
        key.hash(&mut hasher);
        value.hash(&mut hasher);
    }

    hasher.finish()
}

/// Resolves a placeholder token, returning `None` if unchanged.
#[cfg(any(feature = "fluent", feature = "properties-lang"))]
fn resolve_placeholder<F>(inner: &str, mut translate: F, vars: &UiLangVariables) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    let resolved = resolve_inner_tokens(inner, &mut translate, vars);
    if resolved == inner {
        None
    } else {
        Some(resolved)
    }
}

/// Resolves variables and translation keys within a placeholder string.
#[cfg(any(feature = "fluent", feature = "properties-lang"))]
fn resolve_inner_tokens<F>(inner: &str, translate: &mut F, vars: &UiLangVariables) -> String
where
    F: FnMut(&str) -> Option<String>,
{
    let mut out = String::new();
    let mut chars = inner.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            out.push(ch);
            continue;
        }

        if ch == '%' {
            let mut name = String::new();
            let mut closed = false;
            let mut valid = true;

            while let Some(next) = chars.next() {
                if next == '%' {
                    closed = true;
                    break;
                }
                if !is_key_char(next) {
                    valid = false;
                }
                name.push(next);
            }

            if closed && valid && !name.is_empty() {
                if let Some(value) = vars.vars.get(&name) {
                    out.push_str(value);
                } else {
                    out.push('%');
                    out.push_str(&name);
                    out.push('%');
                }
            } else {
                out.push('%');
                out.push_str(&name);
                if closed {
                    out.push('%');
                }
            }

            continue;
        }

        if is_key_char(ch) {
            let mut key = String::new();
            key.push(ch);
            while let Some(next) = chars.peek().copied() {
                if is_key_char(next) {
                    chars.next();
                    key.push(next);
                } else {
                    break;
                }
            }

            if let Some(value) = translate(&key) {
                out.push_str(&value);
            } else {
                out.push_str(&key);
            }

            continue;
        }

        out.push(ch);
    }

    out
}

/// Returns true for characters that are valid in translation keys.
#[cfg(any(feature = "fluent", feature = "properties-lang"))]
fn is_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | ':' | '-')
}

#[cfg(any(feature = "fluent", feature = "properties-lang"))]
pub use i18n;

#[cfg(feature = "fluent")]
pub use i18n_fluent;

#[cfg(feature = "properties-lang")]
pub use i18n_properties;

#[cfg(all(test, any(feature = "fluent", feature = "properties-lang")))]
mod tests {
    use super::{UiLangVariables, localize_html, resolve_placeholder};

    #[test]
    fn unresolved_reactive_placeholder_stays_unchanged() {
        let vars = UiLangVariables::default();
        let resolved = resolve_placeholder("user.name", |_| None, &vars);
        assert_eq!(resolved, None);
    }

    #[test]
    fn dotted_language_keys_can_still_be_localized() {
        let vars = UiLangVariables::default();
        let resolved = resolve_placeholder("app.title", |key| Some(format!("tr:{key}")), &vars);
        assert_eq!(resolved, Some("tr:app.title".to_string()));
    }

    #[cfg(feature = "fluent")]
    #[test]
    fn fluent_localization_replaces_placeholders_from_assets() {
        let html = "<h2>{{ LANGUAGE_TITLE }}</h2><p>{{ WELCOME_START_TEXT %player_name% WELCOME_END_TEXT }}</p>";
        let mut vars = UiLangVariables::default();
        vars.set("player_name", "Tester");

        let out = localize_html(html, Some("de"), "assets/lang", &vars);

        assert!(out.contains("Sprachbeispiel"), "output: {out}");
        assert!(out.contains("Willkommen Tester !"), "output: {out}");
    }

    #[cfg(feature = "properties-lang")]
    #[test]
    fn properties_localization_replaces_placeholders_from_assets() {
        let html = "<h2>{{ LANGUAGE_TITLE }}</h2><p>{{ WELCOME_START_TEXT %player_name% WELCOME_END_TEXT }}</p>";
        let mut vars = UiLangVariables::default();
        vars.set("player_name", "Tester");

        let out = localize_html(html, Some("de"), "assets/lang", &vars);

        assert!(out.contains("Sprachbeispiel"), "output: {out}");
        assert!(out.contains("Willkommen Tester !"), "output: {out}");
    }
}
