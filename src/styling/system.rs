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
    pub style: Style,
}

impl WidgetStyle {
    pub fn load_from_file(path: &str, selector: &str) -> Self {
        if let Some(style) = CSS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(path).and_then(|map| map.get(selector).cloned()))
        {
            return Self {
                css_path: path.to_string(),
                css_selector: selector.to_string(),
                style,
            };
        }
        
        let map = load_css(path);
        if let Ok(styles) = map {
            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(path.to_string(), styles.clone());
            }

            let style = styles.get(selector).cloned().unwrap_or_default();
            return Self {
                css_path: path.to_string(),
                css_selector: selector.to_string(),
                style,
            };
        }

        // Fallback
        Self {
            css_path: String::from(""),
            css_selector: String::from(""),
            style: Style::default(),
        }
    }

    pub fn load_selector(&self, selector: &str) -> Self {
        Self::load_from_file(&self.css_path, selector)
    }
}