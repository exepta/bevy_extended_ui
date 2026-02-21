use crate::io::CssAsset;
use crate::styles::parser::load_css;
use crate::styles::{AnimationKeyframe, CssClass, CssID, ParsedCss, Style, StylePair, TagName};
use bevy::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

static CSS_CACHE: Lazy<RwLock<HashMap<AssetId<CssAsset>, ParsedCss>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Component that stores parsed CSS styles and keyframes for an entity.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UiStyle {
    pub css: Handle<CssAsset>,
    pub styles: HashMap<String, StylePair>,
    pub keyframes: HashMap<String, Vec<AnimationKeyframe>>,
    pub active_style: Option<Style>,
}

impl UiStyle {
    /// Creates a `UiStyle` by parsing or reusing cached CSS data.
    pub fn from_asset_handle(css: Handle<CssAsset>, css_assets: &Assets<CssAsset>) -> Self {
        let id = css.id();

        if let Some(cached) = CSS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(&id).cloned())
        {
            return Self {
                css,
                styles: cached.styles,
                keyframes: cached.keyframes,
                active_style: None,
            };
        }

        let Some(asset) = css_assets.get(&css) else {
            return Self {
                css,
                styles: HashMap::new(),
                keyframes: HashMap::new(),
                active_style: None,
            };
        };

        let parsed = load_css(&asset.text);

        if let Ok(mut cache) = CSS_CACHE.write() {
            cache.insert(id, parsed.clone());
        }

        Self {
            css,
            styles: parsed.styles,
            keyframes: parsed.keyframes,
            active_style: None,
        }
    }

    /// Reloads the CSS data by invalidating the cache for this asset.
    pub fn reload_from_assets(&mut self, css_assets: &Assets<CssAsset>) {
        let id = self.css.id();

        if let Ok(mut cache) = CSS_CACHE.write() {
            cache.remove(&id);
        }

        *self = UiStyle::from_asset_handle(self.css.clone(), css_assets);
    }

    /// Returns a clone containing only rules relevant for the given selectors.
    pub fn filtered_clone(
        &self,
        id: Option<&CssID>,
        classes: Option<&CssClass>,
        tag: Option<&TagName>,
    ) -> Self {
        let mut filtered = HashMap::new();
        let mut priority_map = HashMap::<String, u8>::new();

        let pseudo_classes = ["hover", "focus", "read-only", "disabled", "invalid"];

        let mut insert_with_pseudo = |base: &str, prio: u8| {
            for (key, style) in self.styles.iter() {
                let selector = if style.selector.is_empty() {
                    key.as_str()
                } else {
                    style.selector.as_str()
                };

                if selector == base
                    || selector.starts_with(&format!("{base}:"))
                    || selector.ends_with(base)
                    || selector.contains(&format!("{base} "))
                    || selector.contains(&format!(" {base}"))
                {
                    let current = priority_map.get(key).copied().unwrap_or(u8::MAX);
                    if prio <= current {
                        filtered.insert(key.clone(), style.clone());
                        priority_map.insert(key.clone(), prio);
                    }
                }

                for pseudo in &pseudo_classes {
                    let full = format!("{base}:{pseudo}");
                    if selector == full.as_str()
                        || selector.ends_with(&full)
                        || selector.contains(&format!("{full} "))
                        || selector.contains(&format!(" {full}"))
                    {
                        let current = priority_map.get(key).copied().unwrap_or(u8::MAX);
                        if prio <= current {
                            filtered.insert(key.clone(), style.clone());
                            priority_map.insert(key.clone(), prio);
                        }
                    }
                }
            }
        };

        if let Some(tag) = tag {
            insert_with_pseudo(&tag.0, 3);
        }

        insert_with_pseudo("*", 0);

        if let Some(classes) = classes {
            for class in &classes.0 {
                let normalized = if class.starts_with('.') {
                    class.clone()
                } else {
                    format!(".{class}")
                };
                insert_with_pseudo(&normalized, 2);
            }
        }

        if let Some(id) = id {
            insert_with_pseudo(&format!("#{}", id.0), 1);
        }

        Self {
            css: self.css.clone(),
            styles: filtered,
            keyframes: self.keyframes.clone(),
            active_style: None,
        }
    }
}
