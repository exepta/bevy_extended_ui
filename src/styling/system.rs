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

        if let Ok(styles) = load_css(path) {
            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(path.to_string(), styles.clone());
            }
            return Self {
                css_path: path.to_string(),
                styles,
            };
        }

        Self {
            css_path: String::new(),
            styles: HashMap::new(),
        }
    }

    pub fn reload(&mut self) {
        if let Some(cached) = CSS_CACHE.read().ok().and_then(|cache| cache.get(&self.css_path).cloned()) {
            self.styles = cached;
        } else if let Ok(styles) = load_css(&self.css_path) {
            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(self.css_path.clone(), styles.clone());
            }
            self.styles = styles;
        } else {
            self.styles.clear();
        }
    }

    pub fn ensure_class_loaded(&mut self, class: &str) {
        if self.styles.contains_key(class) {
            return;
        }

        if let Ok(styles) = load_css(&self.css_path) {
            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(self.css_path.clone(), styles.clone());
            }

            for (k, v) in styles.into_iter() {
                self.styles.insert(k, v);
            }
        }
    }
}