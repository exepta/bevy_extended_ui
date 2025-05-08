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
    pub styles: HashMap<String, Style>,
}

impl WidgetStyle {
    pub fn load_from_file(path: &str) -> Self {
        if let Some(cached) = CSS_CACHE.read().ok().and_then(|cache| cache.get(path).cloned()) {
            return Self {
                css_path: path.to_string(),
                styles: cached,
            };
        }

        let styles = load_css(path).unwrap_or_else(|_| HashMap::new());

        if let Ok(mut cache) = CSS_CACHE.write() {
            cache.insert(path.to_string(), styles.clone());
        }

        Self {
            css_path: path.to_string(),
            styles,
        }
    }

    pub fn reload(&mut self) {
        *self = Self::load_from_file(&self.css_path);
    }

    pub fn ensure_class_loaded(&mut self, class: &str) {
        if self.styles.keys().any(|k| k.contains(class)) {
            return;
        }

        if let Ok(new_styles) = load_css(&self.css_path) {
            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(self.css_path.clone(), new_styles.clone());
            }

            for (k, v) in new_styles {
                self.styles.insert(k, v);
            }
        }
    }
}