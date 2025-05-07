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

        let mut styles = load_css(path).unwrap_or_else(|_| HashMap::new());

        Self::flatten_nested_selectors(&mut styles);

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
        if self.styles.contains_key(class) {
            return;
        }

        if let Ok(mut new_styles) = load_css(&self.css_path) {
            Self::flatten_nested_selectors(&mut new_styles);

            if let Ok(mut cache) = CSS_CACHE.write() {
                cache.insert(self.css_path.clone(), new_styles.clone());
            }

            for (k, v) in new_styles {
                self.styles.insert(k, v);
            }
        }
    }

    fn flatten_nested_selectors(styles: &mut HashMap<String, Style>) {
        let mut new_entries = vec![];

        for (selector, style) in styles.iter() {
            let parts: Vec<&str> = selector.split_whitespace().collect();
            if parts.len() >= 2 {
                let parent = parts[0];
                let child = parts[1];

                if let Some((prefix, name, pseudo)) = Self::extract_name_and_pseudo(parent, child) {
                    let new_selector = format!("{}{}:{}", prefix, name, pseudo);
                    new_entries.push((new_selector, style.clone()));
                }
            }
        }

        for (k, v) in new_entries {
            styles.insert(k, v);
        }
    }

    fn extract_name_and_pseudo(parent: &str, child: &str) -> Option<(char, String, String)> {
        let prefix = child.chars().next()?;
        if (prefix == '.' || prefix == '#') && parent.contains(':') {
            let name = child[1..].to_string();
            let pseudo = parent.split(':').nth(1)?.to_string();
            Some((prefix, name, pseudo))
        } else {
            None
        }
    }
}