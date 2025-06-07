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
        Option<&TagName>,
        Option<&ChildOf>
    )>,
    mut widget_query: Query<Option<&mut WidgetStyle>>,
) {
    for (entity, file, id_opt, class_opt, tag_opt, parent_opt) in query.iter() {

        let css_path = file.0.as_str();

        if !Path::new(css_path).exists() {
            error!("CSS File not found {}", css_path);
            continue;
        }
        
        let full_style = WidgetStyle::load_from_file(css_path);
        let mut merged_styles: HashMap<String, Style> = HashMap::new();

        for (selector, style) in full_style.styles.iter() {
            let parts: Vec<&str> = selector.split_whitespace().collect();

            if matches_selector_chain(&parts, id_opt, class_opt, tag_opt, parent_opt, &parent_query) {
                merged_styles.insert(selector.clone(), style.clone());
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

fn matches_selector_chain(
    selectors: &[&str],
    id_opt: Option<&CssID>,
    class_opt: Option<&CssClass>,
    tag_opt: Option<&TagName>,
    parent_opt: Option<&ChildOf>,
    parent_query: &Query<(Option<&CssID>, Option<&CssClass>, Option<&TagName>, Option<&ChildOf>)>,
) -> bool {

    if selectors.len() > 1 && parent_opt.is_none() {
        return false;
    }
    
    if selectors.is_empty() {
        return true;
    }

    let current_sel = selectors.last().unwrap();

    if !matches_selector(current_sel, id_opt, class_opt, tag_opt) {
        return false;
    }
    
    if selectors.len() == 1 {
        return true;
    }

    if let Some(parent) = parent_opt {
        if let Ok((pid_opt, p_class_opt, p_tag, p_parent_opt)) = parent_query.get(parent.parent()) {
            return matches_selector_chain(&selectors[..selectors.len()-1], pid_opt, p_class_opt, p_tag, p_parent_opt, parent_query);
        }
    }

    false
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
