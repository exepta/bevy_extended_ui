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

/// Updates the CSS styles for all entities that have had their `CssSource` added or changed.
///
/// <p>This system queries entities with a `CssSource` component and evaluates whether their CSS
/// should be reloaded and merged based on matching selector chains (ID, class, tag, and parent hierarchy).
/// It loads all defined CSS files, merges matching selectors, and applies the final `WidgetStyle`
/// to the entity if it has changed or doesn't yet exist.</p>
///
/// <p>The merge strategy allows multiple CSS files to contribute to the same selector. Properties
/// from later files override those from earlier ones (similar to cascading behavior in traditional CSS).</p>
///
/// @param commands Bevy `Commands` used to insert updated styles into entities.
/// @param query A query that returns entities with changed or added `CssSource`, along with optional `CssID`,
///              `CssClass`, `TagName`, and `ChildOf` components.
/// @param parent_query A query used to resolve parent hierarchy for matching selector chains.
/// @param widget_query A query for retrieving and mutating existing `WidgetStyle` components on entities.
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
    for (entity, css_source, id, class, tag, parent) in query.iter() {
        let (merged_styles, css_paths) =
            load_and_merge_styles(&css_source.0, id, class, tag, parent, &parent_query);

        let final_style = WidgetStyle {
            styles: merged_styles,
            css_path: css_paths.join(";"),
            active_style: None,
        };

        match widget_query.get_mut(entity) {
            Ok(Some(existing)) if existing.styles != final_style.styles => {
                commands.entity(entity).insert(final_style);
            }
            Ok(None) => {
                commands.entity(entity).insert(final_style);
            }
            _ => {}
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

/// Loads and merges applicable CSS styles from the provided list of paths.
///
/// This function resolves each CSS path, loads the associated styles using
/// `WidgetStyle::load_from_file`, and filters selectors based on whether they match
/// the current entity's identity (ID, class, tag, and parent chain).
///
/// Matching styles are merged together. If the same selector appears multiple times
/// across CSS files, later declarations will overwrite existing properties within
/// that selector, rather than replacing the entire style.
///
/// # Parameters
///
/// - `paths`: A list of relative CSS file paths to load and merge.
/// - `id`: Optional reference to the entity's `CssID` component.
/// - `class`: Optional reference to the entity's `CssClass` component.
/// - `tag`: Optional reference to the entity's `TagName` component.
/// - `parent`: Optional reference to the entity's `ChildOf` component, used for selector chain matching.
/// - `parent_query`: A query that provides access to parent entity style components, used to evaluate complex CSS selectors.
///
/// # Returns
///
/// A tuple containing:
/// - `HashMap<String, Style>`: A map of matched selectors to their merged `Style` content.
/// - `Vec<String>`: A list of successfully loaded and resolved CSS file paths.
///
/// # Behavior
///
/// - Logs an error and skips any CSS file that does not exist.
/// - Selectors are filtered via `matches_selector_chain`.
/// - Styles with the same selector are merged using `Style::merge`, allowing later CSS files
///   to override specific properties.
///
/// # Example
///
/// ```rust
/// let (merged_styles, paths) = load_and_merge_styles(
///     &vec!["base.css".into(), "theme.css".into()],
///     Some(&css_id),
///     Some(&css_class),
///     Some(&tag_name),
///     Some(&parent),
///     &parent_query,
/// );
/// ```
fn load_and_merge_styles(
    paths: &Vec<String>,
    id: Option<&CssID>,
    class: Option<&CssClass>,
    tag: Option<&TagName>,
    parent: Option<&ChildOf>,
    parent_query: &Query<(
        Option<&CssID>,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&ChildOf>,
    )>,
) -> (HashMap<String, Style>, Vec<String>) {
    let mut merged: HashMap<String, Style> = HashMap::new();
    let mut loaded_paths = Vec::new();

    for path in paths {
        let full_path = resolve_css_path(path);

        if !Path::new(&full_path).exists() {
            error!("CSS File not found: {}", full_path);
            continue;
        }

        loaded_paths.push(full_path.clone());
        let style = WidgetStyle::load_from_file(&full_path);

        for (selector, new_style) in style.styles {
            let selector_parts: Vec<&str> = selector.split_whitespace().collect();

            if matches_selector_chain(&selector_parts, id, class, tag, parent, parent_query) {
                merged
                    .entry(selector.clone())
                    .and_modify(|existing| existing.merge(&new_style))
                    .or_insert(new_style.clone());
            }
        }
    }

    (merged, loaded_paths)
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