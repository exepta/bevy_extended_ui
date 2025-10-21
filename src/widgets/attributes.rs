use bevy::prelude::*;
use crate::registry::UI_ID_GENERATE;
use crate::styles::css_watcher::apply_property_to_style;
use crate::styles::Style;

/// Unique identifier for UI elements.
///
/// Each UI element should have a unique `UIGenID` generated atomically.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UIGenID(usize);

impl Default for UIGenID {

    /// Generates a new unique `UIGenID` using a global atomic counter.
    fn default() -> Self {
        Self(UI_ID_GENERATE.lock().unwrap().acquire())
    }
}

impl UIGenID {
    pub fn get(&self) -> usize {
        self.0
    }
}

/// Associates a UI child entity with a parent widget by ID.
///
/// Used for binding UI components to their logical parent.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BindToID(pub usize);

/// Stores the interaction and UI state flags for a widget.
///
/// Contains boolean flags for common widget states such as focused, hovered, disabled, etc.
#[derive(Component, Reflect, Default, PartialEq, Eq, Debug, Clone)]
#[reflect(Component)]
pub struct UIWidgetState {
    pub focused: bool,
    pub hovered: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub checked: bool,
    pub open: bool,
}

/// Tracks the currently focused or active widget by its ID.
///
/// This resource holds the ID of the widget that currently has focus.
#[derive(Resource, Debug, Clone)]
pub struct CurrentWidgetState {
    pub widget_id: usize,
}

impl Default for CurrentWidgetState {

    /// Returns a default `CurrentWidgetState` with `widget_id` set to 0
    /// (meaning no widget currently focused).
    fn default() -> Self {
        Self {
            widget_id: 0,
        }
    }
}

/// Marker component for UI elements that should ignore the parent widget state.
///
/// Used to mark UI nodes that do not inherit state like `focused`, `hovered`, etc.
#[derive(Component)]
pub struct IgnoreParentState;

/// A component that stores parsed CSS style data using Bevy's `Style` struct.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlStyle(pub Style);

impl HtmlStyle {
    /// Parses a raw CSS style string and converts it into an `HtmlStyle`.
    ///
    /// The input string should be a semicolon-separated list of CSS properties.
    ///
    /// # Example
    /// ```rust
    /// use bevy_extended_ui::html::HtmlStyle;
    /// let style = HtmlStyle::from_str("display: flex; justify-content: center;");
    /// ```
    pub fn from_str(style_code: &str) -> HtmlStyle {
        let mut style = Style::default();

        for part in style_code.split(';') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (name, value) = if let Some((k, v)) = trimmed.split_once(':') {
                (k.trim(), v.trim())
            } else if let Some((k, v)) = trimmed.split_once(' ') {
                (k.trim(), v.trim())
            } else {
                continue;
            };

            apply_property_to_style(&mut style, name, value);
        }

        HtmlStyle(style)
    }
}