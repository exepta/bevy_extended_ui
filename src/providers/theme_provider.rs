use super::{
    ProviderChildPolicy, ProviderEffect, ProviderResolveContext, ProviderRules, UiProvider,
};
use crate::io::CssAsset;
use crate::styles::CssSource;
use crate::ExtendedUiConfiguration;
use bevy::asset::AssetId;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum ThemeSwitchRequest {
    ByName(String),
    Next,
}

static THEME_SWITCH_REQUESTS: Lazy<Mutex<Vec<ThemeSwitchRequest>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

/// Runtime state for ThemeProvider.
#[derive(Resource, Debug, Clone)]
pub struct ThemeProviderState {
    themes_fs_path: String,
    themes_asset_dir: String,
    themes: HashMap<String, Handle<CssAsset>>,
    theme_ids: HashSet<AssetId<CssAsset>>,
    default_theme: Option<String>,
    active_theme: Option<String>,
}

impl Default for ThemeProviderState {
    fn default() -> Self {
        Self {
            themes_fs_path: "assets/themes".to_string(),
            themes_asset_dir: "themes".to_string(),
            themes: HashMap::new(),
            theme_ids: HashSet::new(),
            default_theme: None,
            active_theme: None,
        }
    }
}

impl ThemeProviderState {
    pub fn themes_asset_dir(&self) -> &str {
        self.themes_asset_dir.as_str()
    }

    pub fn known_themes(&self) -> HashSet<String> {
        self.themes.keys().cloned().collect()
    }

    pub fn default_theme(&self) -> Option<&str> {
        self.default_theme.as_deref()
    }

    pub fn active_theme(&self) -> Option<&str> {
        self.active_theme.as_deref()
    }

    pub fn set_default_theme(&mut self, requested: &str) {
        let requested = requested.trim();
        if requested.is_empty() {
            return;
        }

        if self.themes.contains_key(requested) {
            self.default_theme = Some(requested.to_string());
        } else {
            warn!(
                "ThemeProvider default theme '{}' not found. Keeping current fallback.",
                requested
            );
            self.ensure_default_theme();
        }
    }

    fn ensure_default_theme(&mut self) {
        if let Some(current) = self.default_theme.as_deref() {
            if self.themes.contains_key(current) {
                return;
            }
        }

        self.default_theme = self.themes.keys().min().cloned();
    }

    fn resolve_theme_or_default(&self, requested: &str) -> Option<String> {
        let requested = requested.trim();
        if self.themes.contains_key(requested) {
            return Some(requested.to_string());
        }

        if let Some(default) = self.default_theme.as_deref() {
            if self.themes.contains_key(default) {
                return Some(default.to_string());
            }
        }

        self.themes.keys().min().cloned()
    }

    fn next_theme_name(&self) -> Option<String> {
        let mut names: Vec<&String> = self.themes.keys().collect();
        names.sort();

        if names.is_empty() {
            return None;
        }

        if let Some(current) = self
            .active_theme
            .as_deref()
            .or(self.default_theme.as_deref())
            && let Some(index) = names.iter().position(|name| name.as_str() == current)
        {
            let next_index = (index + 1) % names.len();
            return Some(names[next_index].clone().to_string());
        }

        names.first().cloned().map(|name| name.to_string())
    }

    fn theme_handle(&self, name: &str) -> Option<Handle<CssAsset>> {
        self.themes.get(name).cloned()
    }
}

/// Default provider that maps `<theme-provider ...>` to a css file in the configured themes folder.
#[derive(Debug, Default, Clone, Copy)]
pub struct ThemeProvider;

impl ThemeProvider {
    /// Queues a global theme switch request.
    ///
    /// The request is applied by the provider system on the next frame.
    /// If the requested theme does not exist, the provider fallback/default theme is used.
    pub fn switch_theme(theme: &str) {
        let trimmed = theme.trim();
        if trimmed.is_empty() {
            warn!("ThemeProvider::switch_theme called with empty theme name.");
            return;
        }

        if let Ok(mut queue) = THEME_SWITCH_REQUESTS.lock() {
            queue.push(ThemeSwitchRequest::ByName(trimmed.to_string()));
        }
    }

    /// Queues a switch request to the next discovered theme (sorted by name).
    pub fn switch_next_theme() {
        if let Ok(mut queue) = THEME_SWITCH_REQUESTS.lock() {
            queue.push(ThemeSwitchRequest::Next);
        }
    }
}

impl UiProvider for ThemeProvider {
    fn tag(&self) -> &'static str {
        "theme-provider"
    }

    fn rules(&self) -> ProviderRules {
        ProviderRules {
            requires_body_child: true,
            child_policy: ProviderChildPolicy::Only(vec!["body".to_string()]),
            allow_in_head: false,
        }
    }

    fn resolve(&self, ctx: ProviderResolveContext<'_>) -> Result<ProviderEffect, String> {
        let requested = ctx
            .active_theme()
            .or_else(|| ctx.attr("default"))
            .or_else(|| ctx.attr("theme"))
            .unwrap_or("default")
            .trim();

        if requested.is_empty() {
            return Err("attribute 'default' must not be empty".to_string());
        }

        let is_valid_name = requested
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !is_valid_name {
            return Err("theme name may only contain [A-Za-z0-9_-] characters".to_string());
        }

        let selected = if let Some(known) = ctx.known_themes() {
            if known.contains(requested) {
                requested.to_string()
            } else if let Some(fallback) = ctx
                .fallback_theme()
                .filter(|fallback| known.contains(*fallback))
            {
                warn!(
                    "Theme '{}' not found. Falling back to default theme '{}'.",
                    requested, fallback
                );
                fallback.to_string()
            } else if let Some(first) = known.iter().min() {
                warn!(
                    "Theme '{}' not found. Falling back to discovered theme '{}'.",
                    requested, first
                );
                first.to_string()
            } else {
                requested.to_string()
            }
        } else {
            requested.to_string()
        };

        let asset_dir = ctx
            .theme_asset_dir()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("themes")
            .trim_matches('/');

        Ok(ProviderEffect {
            extra_css_paths: vec![format!("/{asset_dir}/{selected}.css")],
        })
    }
}

pub(crate) fn refresh_theme_provider_state(
    mut state: ResMut<ThemeProviderState>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
) {
    let themes_fs_path = config.themes_path.trim();
    let themes_fs_path = if themes_fs_path.is_empty() {
        "assets/themes".to_string()
    } else {
        themes_fs_path.to_string()
    };
    let themes_asset_dir = normalize_themes_asset_dir(&themes_fs_path);

    let discovered = discover_theme_names(&themes_fs_path);

    state.themes_fs_path = themes_fs_path.clone();
    state.themes_asset_dir = themes_asset_dir.clone();
    state.themes.clear();
    state.theme_ids.clear();

    for name in discovered {
        let asset_path = format!("{themes_asset_dir}/{name}.css");
        let handle: Handle<CssAsset> = asset_server.load(asset_path);
        state.theme_ids.insert(handle.id());
        state.themes.insert(name, handle);
    }

    if state.themes.is_empty() {
        warn!(
            "ThemeProvider found no themes in '{}'. Configure `ExtendedUiConfiguration.themes_path` or add theme files.",
            themes_fs_path
        );
    }

    state.ensure_default_theme();

    if let Some(active) = state.active_theme.as_deref() {
        if !state.themes.contains_key(active) {
            state.active_theme = None;
        }
    }
}

pub(crate) fn apply_theme_switch_requests(
    mut state: ResMut<ThemeProviderState>,
    mut css_query: Query<&mut CssSource>,
) {
    let requests = if let Ok(mut queue) = THEME_SWITCH_REQUESTS.lock() {
        if queue.is_empty() {
            return;
        }
        std::mem::take(&mut *queue)
    } else {
        return;
    };

    for request in requests {
        let selected = match request {
            ThemeSwitchRequest::ByName(requested) => {
                let Some(selected) = state.resolve_theme_or_default(&requested) else {
                    warn!(
                        "ThemeProvider::switch_theme('{}') ignored: no themes are available.",
                        requested
                    );
                    continue;
                };

                if selected != requested {
                    warn!(
                        "ThemeProvider::switch_theme('{}') fallback to '{}'.",
                        requested, selected
                    );
                }
                selected
            }
            ThemeSwitchRequest::Next => {
                let Some(selected) = state.next_theme_name() else {
                    warn!("ThemeProvider::switch_next_theme ignored: no themes are available.");
                    continue;
                };
                selected
            }
        };

        let Some(handle) = state.theme_handle(&selected) else {
            continue;
        };
        state.active_theme = Some(selected);

        for mut css_source in &mut css_query {
            for source_handle in &mut css_source.0 {
                if state.theme_ids.contains(&source_handle.id()) {
                    *source_handle = handle.clone();
                }
            }
        }
    }
}

fn discover_theme_names(folder: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };

    let mut themes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_css = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("css"))
            .unwrap_or(false);
        if !is_css {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if stem.is_empty() {
            continue;
        }

        themes.push(stem.to_string());
    }

    themes.sort();
    themes.dedup();
    themes
}

fn normalize_themes_asset_dir(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let trimmed = normalized.trim().trim_end_matches('/').trim_start_matches("./");
    let trimmed = trimmed.trim_start_matches('/');

    if let Some(rest) = trimmed.strip_prefix("assets/") {
        if !rest.is_empty() {
            return rest.to_string();
        }
    }

    if let Some(index) = trimmed.rfind("/assets/") {
        let rest = &trimmed[index + "/assets/".len()..];
        if !rest.is_empty() {
            return rest.to_string();
        }
    }

    if trimmed.is_empty() {
        "themes".to_string()
    } else {
        trimmed.to_string()
    }
}
