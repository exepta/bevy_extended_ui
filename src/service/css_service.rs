use std::collections::HashMap;
use std::fs;
use std::path::Path;
use bevy::prelude::*;
use crate::styling::convert::{CssClass, CssID, CssSource, ExistingCssIDs, TagName};
use crate::styling::Style;
use crate::styling::system::WidgetStyle;

pub const DEFAULT_CORE_CSS: &str = include_str!("../../assets/css/core.css");

pub struct CssService;

impl Plugin for CssService {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExistingCssIDs>();
        app.add_systems(Update, (
            update_css_conventions,
        ).chain());
    }
}

/// Applies CSS styles to entities based on their CSS selector metadata and updates their [`WidgetStyle`].
///
/// This system reacts to changes or additions to the [`CssSource`] component. It loads the CSS file,
/// evaluates selectors, and assigns matching styles to entities based on their ID, class, tag name,
/// and optionally their parent hierarchy. The result is stored in the entity’s [`WidgetStyle`] component.
///
/// # Parameters
/// - `commands`: Bevy's command buffer for inserting updated styles.
/// - `query`: Entities that changed or received a [`CssSource`], along with optional metadata like
///   [`CssID`], [`CssClass`], [`TagName`], and [`ChildOf`] for CSS matching.
/// - `parent_query`: A query to allow recursive resolution of parent styles during selector matching.
/// - `widget_query`: Access to current [`WidgetStyle`] components on target entities.
///
/// # CSS Matching
/// - Simple selectors (`#id`, `.class`, `tag`) are supported.
/// - Selector chains (e.g., `div .button`) are resolved recursively using [`ChildOf`] relationships.
/// - Pseudo-classes (e.g., `:hover`) are ignored during base matching but preserved in style data.
///
/// # Behavior
/// - If a matching style is found, it replaces or inserts a [`WidgetStyle`] on the entity.
/// - CSS is only reloaded and applied if the file path changed or on new addition.
///
/// # Errors
/// Logs an error if the CSS file doesn't exist or is unreadable.
///
/// # Example
/// ```css
/// .button {
///     width: 100px;
/// }
///
/// div .button {
///     color: red;
/// }
/// ```
///
/// Will match `.button` and also `.button` inside a `div`, based on entity hierarchy.
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

        let mut merged_styles: HashMap<String, Style> = HashMap::new();
        let mut css_path_combined = String::new();


        for path in &file.0 {
            let css_path = resolve_css_path(path);

            if !Path::new(&css_path).exists() {
                error!("CSS File not found {}", css_path);
                continue;
            }

            css_path_combined += &format!("{};", css_path);

            let style = WidgetStyle::load_from_file(&css_path);
            for (selector, value) in style.styles {
                let parts: Vec<&str> = selector.split_whitespace().collect();

                if matches_selector_chain(&parts, id_opt, class_opt, tag_opt, parent_opt, &parent_query) {
                    merged_styles.insert(selector.clone(), value.clone());
                }
            }
        }

        let final_style = WidgetStyle {
            styles: merged_styles,
            css_path: css_path_combined.to_string(),
            active_style: None,
        };

        if let Ok(Some(existing_style)) = widget_query.get_mut(entity) {
            if existing_style.styles != final_style.styles {
                commands.entity(entity).insert(final_style);
            }
        } else {
            commands.entity(entity).insert(final_style);
        }
    }
}

/// Recursively matches a list of CSS selectors against an entity and its ancestry.
///
/// Supports compound selectors like `div .button` by walking up the hierarchy using [`ChildOf`].
/// Each selector is matched in reverse order (the last selector is the current entity,
/// previous ones are parents).
///
/// # Parameters
/// - `selectors`: Slice of selector strings (e.g., `["div", ".button"]`)
/// - `id_opt`: Optional CSS ID of the current entity.
/// - `class_opt`: Optional CSS class list of the current entity.
/// - `tag_opt`: Optional tag name of the current entity.
/// - `parent_opt`: Optional parent reference (`ChildOf`) of the current entity.
/// - `parent_query`: A query used to walk up the entity tree.
///
/// # Returns
/// `true` if the full selector chain matches; otherwise `false`.
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

/// Matches a single CSS selector (e.g., `.class`, `#id`, or `tag`) against an entity's metadata.
///
/// Ignores any pseudo-classes (e.g., `:hover`) in the selector.
///
/// # Parameters
/// - `selector`: The raw selector string.
/// - `id_opt`: Optional ID of the entity.
/// - `class_opt`: Optional class list of the entity.
/// - `tag_opt`: Optional tag name of the entity.
///
/// # Returns
/// `true` if the selector matches one of the metadata fields; otherwise `false`.
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

/// Resolves a CSS file path to a valid file on disk.
///
/// If the path is `"assets/css/core.css"` and it doesn’t exist, it writes a fallback to
/// the system temp directory and returns that path instead.
///
/// # Parameters
/// - `original`: The original string path.
///
/// # Returns
/// A path guaranteed to point to a valid file.
fn resolve_css_path(original: &str) -> String {
    if Path::new(original).exists() {
        original.to_string()
    } else if original == "assets/css/core.css" {
        let tmp_path = std::env::temp_dir().join("bevy_extended_ui_core.css");
        let _ = fs::write(&tmp_path, DEFAULT_CORE_CSS);
        tmp_path.to_string_lossy().to_string()
    } else {
        original.to_string()
    }
}