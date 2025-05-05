use std::collections::HashMap;
use std::sync::RwLock;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use crate::styling::css::load_css;
use crate::styling::Style;

static CSS_CACHE: Lazy<RwLock<HashMap<String, HashMap<String, Style>>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct WidgetStyle {
    pub css_path: String,
    pub css_selector: String,
    pub styles: HashMap<String, Style>,
}

impl WidgetStyle {
    pub fn load_from_file(path: &str, selector: &str) -> Self {
        if let Some(cached) = CSS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(path).cloned())
        {
            let relevant = cached
                .into_iter()
                .filter(|(key, _)| key.starts_with(selector))
                .collect();

            return Self {
                css_path: path.to_string(),
                css_selector: selector.to_string(),
                styles: relevant,
            };
        }

        if let Ok(styles) = load_css(path) {
            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(path.to_string(), styles.clone());
            }

            let relevant = styles
                .into_iter()
                .filter(|(key, _)| key.starts_with(selector))
                .collect();

            return Self {
                css_path: path.to_string(),
                css_selector: selector.to_string(),
                styles: relevant,
            };
        }

        // Fallback
        Self {
            css_path: String::from(""),
            css_selector: selector.to_string(),
            styles: HashMap::new(),
        }
    }

    pub fn load_selector(&self, selector: &str) -> Self {
        Self::load_from_file(&self.css_path, selector)
    }

    pub fn reload(&mut self) {
        if let Some(cached) = CSS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(&self.css_path).cloned())
        {
            self.styles = cached
                .into_iter()
                .filter(|(key, _)| key.starts_with(&self.css_selector))
                .collect();
        } else if let Ok(styles) = load_css(&self.css_path) {
            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(self.css_path.clone(), styles.clone());
            }

            self.styles = styles
                .into_iter()
                .filter(|(key, _)| key.starts_with(&self.css_selector))
                .collect();
        } else {
            self.styles.clear(); // fallback
        }
    }
}