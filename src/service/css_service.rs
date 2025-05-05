use std::path::Path;
use bevy::prelude::*;
use crate::styling::convert::{CssClass, CssSource};
use crate::styling::system::WidgetStyle;

pub struct CssService;

impl Plugin for CssService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_css_conventions,
            update_css_classes
        ));
    }
}

fn update_css_conventions(
    query: Query<(Entity, &CssSource)>,
    mut widget_query: Query<&mut WidgetStyle>,
) {
    for (entity, file) in query.iter() {
        if let Ok(mut widget_style) = widget_query.get_mut(entity) {
            if widget_style.css_path.eq(&file.0.to_string()) {
                continue;
            }
            if !Path::new(&file.0).exists() {
                continue;
            }
            widget_style.css_path = file.0.to_string();
            widget_style.reload();
        }
    }
}

fn update_css_classes(
    query: Query<(Entity, &CssClass), Or<(Changed<CssClass>, Added<CssClass>)>>,
    mut widget_query: Query<&mut WidgetStyle>,
) {
    for (entity, classes) in query.iter() {
        if let Ok(mut widget_style) = widget_query.get_mut(entity) {
            let active_classes: std::collections::HashSet<_> = classes.0
                .iter()
                .filter(|c| c.starts_with('.'))
                .cloned()
                .collect();

            widget_style.styles.retain(|key, _| {
                if !key.starts_with('.') {
                    return true;
                }

                let base_class = key.split(':').next().unwrap_or(key);

                active_classes.contains(base_class)
            });

            for class in &active_classes {
                widget_style.ensure_class_loaded(class);
            }
        }
    }
}
