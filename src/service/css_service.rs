use std::path::Path;
use bevy::prelude::*;
use crate::styling::convert::{CssClass, CssID, CssSource, ExistingCssIDs};
use crate::styling::system::WidgetStyle;

pub struct CssService;

impl Plugin for CssService {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExistingCssIDs>();
        app.add_systems(Update, (
            cleanup_css_ids_by_remove,
            update_css_conventions,
            update_css_classes,
            update_css_id,
            validate_unique_css_ids,
        ).chain());
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

fn update_css_id(
    query: Query<(Entity, &CssID), Or<(Changed<CssClass>, Added<CssClass>)>>,
    mut widget_query: Query<&mut WidgetStyle>,
) {
    for (entity, css_id) in query.iter() {
        if let Ok(mut style) = widget_query.get_mut(entity) {
            let id_selector = format!("#{}", css_id.0);

            style.ensure_class_loaded(&id_selector);
        }
    }
}

fn validate_unique_css_ids(
    mut commands: Commands,
    mut existing_ids: ResMut<ExistingCssIDs>,
    query: Query<(Entity, &CssID), Added<CssID>>
) {
    for (entity, css_id) in query.iter() {
        if existing_ids.0.contains(&css_id.0) {
            error!("CSS ID already in use: {}, entity {:?} have no id set!", css_id.0, entity);
            commands.entity(entity).remove::<CssID>();
        } else {
            existing_ids.0.insert(css_id.0.clone());
        }
    }
}

fn cleanup_css_ids_by_remove(
    mut removed: RemovedComponents<CssID>,
    mut existing_ids: ResMut<ExistingCssIDs>,
    css_query: Query<&CssID>
) {
    for entity in removed.read()  {
        if let Ok(css_id) = css_query.get(entity) {
            existing_ids.0.remove(&css_id.0);
        }
    }
}
