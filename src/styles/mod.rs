pub mod style_types;
pub mod css_attributes;
pub mod css_colors;
pub mod css_watcher;

use std::collections::HashMap;
use bevy::prelude::*;
use crate::styles::css_attributes::{CssClass, CssID, TagName};
use crate::styles::css_watcher::UiParsedStyles;
use crate::styles::style_types::*;

/// Comprehensive style properties for UI elements.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Style {
    pub display: Option<Display>,
    pub position_type: Option<PositionType>,
    pub width: Option<Val>,
    pub min_width: Option<Val>,
    pub max_width: Option<Val>,
    pub height: Option<Val>,
    pub min_height: Option<Val>,
    pub max_height: Option<Val>,
    pub left: Option<Val>,
    pub top: Option<Val>,
    pub right: Option<Val>,
    pub bottom: Option<Val>,
    pub padding: Option<UiRect>,
    pub margin: Option<UiRect>,
    pub border: Option<UiRect>,
    pub overflow: Option<Overflow>,
    pub color: Option<Color>,
    pub background: Option<Background>,
    pub border_color: Option<Color>,
    pub border_width: Option<Val>,
    pub border_radius: Option<Radius>,
    pub font_size: Option<FontVal>,
    pub box_shadow: Option<BoxShadow>,
    pub justify_content: Option<JustifyContent>,
    pub justify_items: Option<JustifyItems>,
    pub justify_self: Option<JustifySelf>,
    pub align_content: Option<AlignContent>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub flex_direction: Option<FlexDirection>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Val>,
    pub flex_wrap: Option<FlexWrap>,
    pub grid_row: Option<GridPlacement>,
    pub grid_column: Option<GridPlacement>,
    pub grid_auto_flow: Option<GridAutoFlow>,
    pub grid_template_rows: Option<Vec<RepeatedGridTrack>>,
    pub grid_template_columns: Option<Vec<RepeatedGridTrack>>,
    pub grid_auto_rows: Option<Vec<GridTrack>>,
    pub grid_auto_columns: Option<Vec<GridTrack>>,
    pub gap: Option<Val>,
    pub text_wrap: Option<LineBreak>,
    pub z_index: Option<i32>,
    pub pointer_events: Option<Pickable>
}

impl Style {

    /**
    * Merges another `Style` into this one.
    * For each field, if the other style has a value set (`Some`), it overwrites this style's value.
    *
    * @param other The other style to merge from.
    */
    pub fn merge(&mut self, other: &Style) {
        if other.display.is_some()               { self.display = other.display.clone(); }
        if other.position_type.is_some()         { self.position_type = other.position_type.clone(); }
        if other.width.is_some()                 { self.width = other.width.clone(); }
        if other.min_width.is_some()             { self.min_width = other.min_width.clone(); }
        if other.max_width.is_some()             { self.max_width = other.max_width.clone(); }
        if other.height.is_some()                { self.height = other.height.clone(); }
        if other.min_height.is_some()            { self.min_height = other.min_height.clone(); }
        if other.max_height.is_some()            { self.max_height = other.max_height.clone(); }
        if other.left.is_some()                  { self.left = other.left.clone(); }
        if other.top.is_some()                   { self.top = other.top.clone(); }
        if other.right.is_some()                 { self.right = other.right.clone(); }
        if other.bottom.is_some()                { self.bottom = other.bottom.clone(); }
        if other.padding.is_some()               { self.padding = other.padding.clone(); }
        if other.margin.is_some()                { self.margin = other.margin.clone(); }
        if other.border.is_some()                { self.border = other.border.clone(); }
        if other.overflow.is_some()              { self.overflow = other.overflow.clone(); }
        if other.color.is_some()                 { self.color = other.color.clone(); }
        if other.background.is_some()            { self.background = other.background.clone(); }
        if other.border_color.is_some()          { self.border_color = other.border_color.clone(); }
        if other.border_width.is_some()          { self.border_width = other.border_width.clone(); }
        if other.border_radius.is_some()         { self.border_radius = other.border_radius.clone(); }
        if other.font_size.is_some()             { self.font_size = other.font_size.clone(); }
        if other.box_shadow.is_some()            { self.box_shadow = other.box_shadow.clone(); }
        if other.justify_content.is_some()       { self.justify_content = other.justify_content.clone(); }
        if other.justify_items.is_some()         { self.justify_items = other.justify_items.clone(); }
        if other.justify_self.is_some()          { self.justify_self = other.justify_self.clone(); }
        if other.align_content.is_some()         { self.align_content = other.align_content.clone(); }
        if other.align_items.is_some()           { self.align_items = other.align_items.clone(); }
        if other.align_self.is_some()            { self.align_self = other.align_self.clone(); }
        if other.flex_direction.is_some()        { self.flex_direction = other.flex_direction.clone(); }
        if other.flex_grow.is_some()             { self.flex_grow = other.flex_grow.clone(); }
        if other.flex_shrink.is_some()           { self.flex_shrink = other.flex_shrink.clone(); }
        if other.flex_basis.is_some()            { self.flex_basis = other.flex_basis.clone(); }
        if other.flex_wrap.is_some()             { self.flex_wrap = other.flex_wrap.clone(); }
        if other.grid_row.is_some()              { self.grid_row = other.grid_row.clone(); }
        if other.grid_column.is_some()           { self.grid_column = other.grid_column.clone(); }
        if other.grid_auto_flow.is_some()        { self.grid_auto_flow = other.grid_auto_flow.clone(); }
        if other.grid_template_rows.is_some()    { self.grid_template_rows = other.grid_template_rows.clone(); }
        if other.grid_template_columns.is_some() { self.grid_template_columns = other.grid_template_columns.clone(); }
        if other.grid_auto_rows.is_some()        { self.grid_auto_rows = other.grid_auto_rows.clone(); }
        if other.grid_auto_columns.is_some()     { self.grid_auto_columns = other.grid_auto_columns.clone(); }
        if other.gap.is_some()                   { self.gap = other.gap.clone(); }
        if other.text_wrap.is_some()             { self.text_wrap = other.text_wrap.clone(); }
        if other.z_index.is_some()               { self.z_index = other.z_index.clone(); }
        if other.pointer_events.is_some()        { self.pointer_events = other.pointer_events.clone(); }
    }
}

/// Component representing style information resolved for a specific UI (`ui_id`).
///
/// The `styles` map should be sourced from the asset-based pipeline (e.g., `UiParsedStyles`)
/// that merges all CSS assets for the UI (with hot-reload).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct WidgetStyle {
    /// Logical UI identifier. Used to look up parsed styles from `UiParsedStyles`.
    pub ui_id: String,
    /// Full selector -> Style map for this UI at the time of creation/refresh.
    pub styles: HashMap<String, Style>,
    /// Optional currently active style (caller-defined semantics).
    pub active_style: Option<Style>,
}

impl WidgetStyle {
    /// Build from the current parsed styles of a UI.
    ///
    /// Pulls the merged selector map from `UiParsedStyles.by_ui[ui_id]`.
    pub fn from_ui(ui_id: impl Into<String>, parsed: &UiParsedStyles) -> Self {
        let ui_id = ui_id.into();
        let styles = parsed
            .by_ui
            .get(&ui_id)
            .map(|p| p.styles.clone())
            .unwrap_or_default();

        Self {
            ui_id,
            styles,
            active_style: None,
        }
    }

    /// Refresh the internal style map from `UiParsedStyles` for this component's `ui_id`.
    ///
    /// Call this when you receive `CssReady`/`HtmlReady` or after your style rebuild system ran.
    pub fn refresh_from_parsed(&mut self, parsed: &UiParsedStyles) {
        if let Some(p) = parsed.by_ui.get(&self.ui_id) {
            self.styles = p.styles.clone();
        }
    }

    /// Legacy-style constructor that mimics the old "load from path" but now maps the path
    /// to a UI id. If your UI id *is* the html/css path stem passes that here. Otherwise,
    /// prefer `from_ui()`.
    pub fn load_from_file_compat(path_as_ui_id: &str, parsed: &UiParsedStyles) -> Self {
        Self::from_ui(path_as_ui_id, parsed)
    }

    /// Filters the styles based on ID, class, and tag, preserving a simple priority:
    /// ID (highest), Class, Tag (lowest). Pseudo-classes are considered.
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
                // base, base:* and descendant selectors containing base
                if selector == base
                    || selector.starts_with(&format!("{base}:"))
                    || selector.contains(&format!("{base} "))
                {
                    let current_prio = priority_map.get(selector).copied().unwrap_or(u8::MAX);
                    if prio <= current_prio {
                        filtered.insert(selector.clone(), style.clone());
                        priority_map.insert(selector.clone(), prio);
                    }
                }

                for pseudo in &pseudo_classes {
                    let full = format!("{base}:{pseudo}");
                    if selector == &full || selector.contains(&format!("{full} ")) {
                        let current_prio = priority_map.get(selector).copied().unwrap_or(u8::MAX);
                        if prio <= current_prio {
                            filtered.insert(selector.clone(), style.clone());
                            priority_map.insert(selector.clone(), prio);
                        }
                    }
                }
            }
        };

        // Priority 3: tag
        if let Some(tag) = tag {
            insert_with_pseudo(&tag.0, 3);
        }

        // Priority 2: classes
        if let Some(classes) = classes {
            for class in &classes.0 {
                let normalized = if class.starts_with('.') {
                    class.to_string()
                } else {
                    format!(".{class}")
                };
                insert_with_pseudo(&normalized, 2);
            }
        }

        // Priority 1: id
        if let Some(id) = id {
            let selector = format!("#{}", id.0);
            insert_with_pseudo(&selector, 1);
        }

        Self {
            ui_id: self.ui_id.clone(),
            styles: filtered,
            active_style: None,
        }
    }

    /// No-op in the asset-based world. Kept for API compatibility.
    /// Use `refresh_from_parsed(parsed)` after your style rebuild system ran.
    pub fn reload(&mut self, parsed: &UiParsedStyles) {
        self.refresh_from_parsed(parsed);
    }

    /// Ensures a class is available by refreshing from parsed styles.
    /// In asset-based flow, classes arrive via hot-reload and rebuild.
    pub fn ensure_class_loaded(&mut self, class: &str, parsed: &UiParsedStyles) {
        let has_any = self.styles.keys().any(|k| k.contains(class));
        if !has_any {
            self.refresh_from_parsed(parsed);
        }
    }
}