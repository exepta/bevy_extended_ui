use std::collections::HashMap;
use std::path::Path;
use bevy::prelude::*;
use crate::styling::convert::{CssClass, CssID, CssSource, ExistingCssIDs, TagName};
use crate::styling::Style;
use crate::styling::system::WidgetStyle;

pub struct CssService;

impl Plugin for CssService {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExistingCssIDs>();
        app.add_systems(Update, (
/*            cleanup_css_ids_by_remove,
            update_css_classes,
            update_css_id,
            validate_unique_css_ids,*/
            update_css_conventions,
        ).chain());
    }
}

fn update_css_conventions(
    mut commands: Commands,
    query: Query<(
        Entity,
        &CssSource,
        Option<&CssID>,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&ChildOf>,
    ), Or<(Changed<CssSource>, Added<CssSource>)>>,
    parent_query: Query<(
        Option<&CssID>,
        Option<&CssClass>,
        &TagName,
    )>,
    mut widget_query: Query<Option<&mut WidgetStyle>>,
) {
    for (entity, file, id_opt, class_opt, tag_opt, parent_opt) in query.iter() {

        let css_path = file.0.as_str();

        if !Path::new(css_path).exists() {
            continue;
        }
        
        let full_style = WidgetStyle::load_from_file(css_path);
        let mut merged_styles: HashMap<String, Style> = HashMap::new();

        for (selector, style) in full_style.styles.iter() {
            let parts: Vec<&str> = selector.split_whitespace().collect();

            match parts.len() {
                1 => {
                    // Simple selector: match directly
                    if matches_selector(parts[0], id_opt, class_opt, tag_opt) {
                        merged_styles.insert(selector.clone(), style.clone());
                    }
                }
                2 => {
                    // Parent > Child selector
                    if let Some(parent) = parent_opt {
                        if let Ok((pid_opt, p_class_opt, p_tag)) = parent_query.get(parent.parent()) {
                            let parent_sel = parts[0];
                            let child_sel = parts[1];

                            let parent_matches = matches_selector(parent_sel, pid_opt, p_class_opt, Some(p_tag));
                            let child_matches = matches_selector(child_sel, id_opt, class_opt, tag_opt);

                            if parent_matches && child_matches {
                                merged_styles.insert(selector.clone(), style.clone());
                            }
                        }
                    }
                }
                _ => {
                    // Ignore deeply nested selectors for now
                }
            }
        }

        let final_style = WidgetStyle {
            styles: merged_styles,
            css_path: css_path.to_string(),
            active_style: None,
        };

        match widget_query.get_mut(entity) {
            Ok(Some(mut existing_style)) => {
                if existing_style.css_path != css_path {
                    *existing_style = final_style.clone();
                    commands.entity(entity).insert(existing_style.clone());
                } else {
                    commands.entity(entity).insert(final_style.clone());
                }
            }
            _ => {
                commands.entity(entity).insert(final_style.clone());
            }
        }
    }
}

/// Matches a single selector against the given ID, class list, or tag name.
fn matches_selector(
    selector: &str,
    id_opt: Option<&CssID>,
    class_opt: Option<&CssClass>,
    tag_opt: Option<&TagName>,
) -> bool {
    let base_selector = selector.split(':').next().unwrap_or(selector);

    if let Some(id) = id_opt {
        if base_selector == format!("#{}", id.0) {
            return true;
        }
    }

    if let Some(classes) = class_opt {
        for class in &classes.0 {
            if base_selector == format!(".{}", class) {
                return true;
            }
        }
    }

    if let Some(tag) = tag_opt {
        if base_selector == tag.0 {
            return true;
        }
    }

    false
}



/*fn update_css_classes(
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
}*/
