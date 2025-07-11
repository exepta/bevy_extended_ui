use std::collections::HashMap;
use std::sync::RwLock;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use crate::styling::convert::{CssClass, CssID, TagName};
use crate::styling::css::load_css;
use crate::styling::Style;

static CSS_CACHE: Lazy<RwLock<HashMap<String, HashMap<String, Style>>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// Component representing style information loaded from a CSS file.
///
/// This struct holds the path to a CSS file, a map of selector strings to [`Style`] objects,
/// and an optional currently active style. It can filter styles by ID, class, or tag name.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct WidgetStyle {
    pub css_path: String,
    pub styles: HashMap<String, Style>,
    pub active_style: Option<Style>,
}

impl WidgetStyle {

    /// Loads styles from a CSS file and constructs a [`WidgetStyle`] instance.
    ///
    /// Uses a global cache to avoid reparsing already loaded CSS files.
    ///
    /// # Arguments
    /// * `path` - Path to the CSS file.
    ///
    /// # Returns
    /// A new `WidgetStyle` containing the parsed styles.
    pub fn load_from_file(path: &str) -> Self {
        if let Some(cached) = CSS_CACHE.read().ok().and_then(|cache| cache.get(path).cloned()) {
            return Self {
                css_path: path.to_string(),
                styles: cached,
                active_style: None,
            };
        }

        let styles = load_css(path).unwrap_or_else(|_| HashMap::new());

        if let Ok(mut cache) = CSS_CACHE.write() {
            cache.insert(path.to_string(), styles.clone());
        }

        Self {
            css_path: path.to_string(),
            styles,
            active_style: None,
        }
    }

    /// Filters the styles based on the given ID, class, and tag name.
    ///
    /// Applies priority in the order: ID (highest), Class, Tag (lowest).
    /// Pseudo-classes like `:hover`, `:focus`, etc., are also considered.
    ///
    /// # Arguments
    /// * `id` - Optional CSS ID component.
    /// * `classes` - Optional CSS classes component.
    /// * `tag` - Optional tag name component.
    ///
    /// # Returns
    /// A new `WidgetStyle` with only the matching filtered styles.
    pub fn filtered_clone(
        &self,
        id: Option<&CssID>,
        classes: Option<&CssClass>,
        tag: Option<&TagName>,
    ) -> Self {
        let mut filtered = HashMap::new();
        let mut priority_map = HashMap::new(); // <Selector, Priority>

        let pseudo_classes = ["hover", "focus", "read-only", "disabled"];

        let mut insert_with_pseudo = |base: &str, prio: u8| {
            for (selector, style) in self.styles.iter() {
                if selector == base || selector.starts_with(&format!("{}:", base)) || selector.contains(&format!("{} ", base)) {
                    let current_prio = priority_map.get(selector).copied().unwrap_or(u8::MAX);
                    if prio <= current_prio {
                        filtered.insert(selector.clone(), style.clone());
                        priority_map.insert(selector.clone(), prio);
                    }
                }

                for pseudo in &pseudo_classes {
                    let full = format!("{}:{}", base, pseudo);
                    if selector == &full || selector.contains(&format!("{} ", full)) {
                        let current_prio = priority_map.get(selector).copied().unwrap_or(u8::MAX);
                        if prio <= current_prio {
                            filtered.insert(selector.clone(), style.clone());
                            priority_map.insert(selector.clone(), prio);
                        }
                    }
                }
            }
        };

        // Prio 3: Tag
        if let Some(tag) = tag {
            insert_with_pseudo(&tag.0, 3);
        }

        // Prio 2: Classes
        if let Some(classes) = classes {
            for class in &classes.0 {
                let normalized = if class.starts_with('.') {
                    class.to_string()
                } else {
                    format!(".{}", class)
                };
                insert_with_pseudo(&normalized, 2);
            }
        }

        // Prio 1: ID
        if let Some(id) = id {
            let selector = format!("#{}", id.0);
            insert_with_pseudo(&selector, 1);
        }

        Self {
            css_path: self.css_path.clone(),
            styles: filtered,
            active_style: None,
        }
    }
    
    /// Reloads the CSS file and updates the internal style map and cache.
    pub fn reload(&mut self) {
        *self = Self::load_from_file(&self.css_path);
    }

    /// Ensures that a given class selector is loaded in the style map.
    ///
    /// If the class is not found, the method attempts to reload the CSS file
    /// and merge any new styles into the existing map and cache.
    ///
    /// # Arguments
    /// * `class` - Name of the CSS class to ensure it is available.
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