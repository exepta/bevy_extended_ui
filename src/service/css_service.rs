use std::path::Path;
use bevy::prelude::*;
use crate::styling::convert::{CssClass, CssID, CssSource, ExistingCssIDs, TagName};
use crate::styling::system::WidgetStyle;

pub struct CssService;

impl Plugin for CssService {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExistingCssIDs>();
        app.add_systems(Update, (
            cleanup_css_ids_by_remove,
            update_css_classes,
            update_css_id,
            validate_unique_css_ids,
            update_css_conventions,
        ).chain());
    }
}

fn update_css_conventions(
    mut commands: Commands,
    query: Query<(Entity, &CssSource, Option<&CssID>, Option<&CssClass>, Option<&TagName>), Or<(Changed<CssSource>, Added<CssSource>)>>,
    mut widget_query: Query<Option<&mut WidgetStyle>>,
) {
    for (entity, file, id_opt, class_opt, tag_opt) in query.iter() {
        let css_path = file.0.as_str();

        // Skip if file doesn't exist
        if !Path::new(css_path).exists() {
            continue;
        }

        // Load full style from file
        let mut full_style = WidgetStyle::load_from_file(css_path);

        // Filter style based on entity attributes
        let mut filtered = full_style.filtered_clone(id_opt, class_opt, tag_opt);
        full_style.reload();
        filtered.css_path = css_path.to_string();
        
        // Check if entity already has WidgetStyle
        match widget_query.get_mut(entity) {
            Ok(Some(mut existing_style)) => {
                // Only update if file path is different
                if existing_style.css_path != css_path {
                    *existing_style = filtered;
                    commands.entity(entity).insert(existing_style.clone());
                } else {
                    // Still re-apply filtering, in case something changed
                    commands.entity(entity).insert(filtered);
                }
            }
            _ => {
                // Insert new style if none exists
                commands.entity(entity).insert(filtered);
            }
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
