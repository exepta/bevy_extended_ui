pub mod components;
pub mod paint;
pub mod parser;

use crate::io::CssAsset;
use crate::styles::components::UiStyle;
use bevy::prelude::*;
use std::cmp::PartialEq;
use std::collections::HashSet;
// ==================================================
//                     Css Styling
// ==================================================

/// Resource that tracks existing CSS IDs to ensure uniqueness.
#[derive(Resource, Default)]
pub struct ExistingCssIDs(pub HashSet<String>);

/// Component representing the tag name of an element (e.g., "div", "span").
#[derive(Component, Reflect, Debug, Clone, Deref, DerefMut)]
#[reflect(Component)]
pub struct TagName(pub String);

/// Component representing one or more CSS classes applied to an element.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssClass(pub Vec<String>);

/// Component representing the CSS ID of an element.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssID(pub String);

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct CssSource(pub Vec<Handle<CssAsset>>);

impl CssSource {
    pub fn from_path(asset_server: &AssetServer, path: &str) -> Self {
        Self(vec![asset_server.load::<CssAsset>(path.to_string())])
    }

    pub fn push_path(&mut self, asset_server: &AssetServer, path: &str) {
        self.0.push(asset_server.load::<CssAsset>(path.to_string()));
    }

    pub fn from_paths(
        asset_server: &AssetServer,
        paths: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self(
            paths
                .into_iter()
                .map(|p| asset_server.load::<CssAsset>(p.into()))
                .collect(),
        )
    }
}

/// Represents the border-radius of a rectangle with individual corner values.
#[derive(Reflect, Default, Clone, PartialEq, Debug)]
pub struct Radius {
    pub top_left: Val,
    pub top_right: Val,
    pub bottom_left: Val,
    pub bottom_right: Val,
}

impl Radius {
    /**
     * Creates a `Radius` where all corners have the same radius value.
     *
     * @param val The radius value to use for all four corners.
     * @return A new `Radius` with uniform corner radii.
     */
    pub fn all(val: Val) -> Self {
        Self {
            top_left: val,
            top_right: val,
            bottom_left: val,
            bottom_right: val,
        }
    }
}

/// Defines the background style including color and optional image.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Background {
    pub color: Color,
    pub image: Option<String>,
}

impl Default for Background {
    /**
     * Creates a default `Background` with transparent color and no image.
     */
    fn default() -> Self {
        Self {
            color: Color::NONE,
            image: None,
        }
    }
}

/// Constants for common font weight values.
#[derive(Reflect, Debug, Clone)]
pub struct FontWeight;

impl FontWeight {
    pub const THIN: u16 = 100;
    pub const EXTRA_LIGHT: u16 = 200;
    pub const LIGHT: u16 = 300;
    pub const NORMAL: u16 = 400;
    pub const MEDIUM: u16 = 500;
    pub const SEMI_BOLD: u16 = 600;
    pub const BOLD: u16 = 700;
    pub const EXTRA_BOLD: u16 = 800;
    pub const BLACK: u16 = 900;
}

/// Placement of an icon relative to text.
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum IconPlace {
    Left,
    Right,
}

impl Default for IconPlace {
    /**
     * Returns the default icon placement (`Right`).
     */
    fn default() -> Self {
        IconPlace::Right
    }
}

/// Represents a font size value, either in pixels or rem units.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum FontVal {
    Px(f32),
    Rem(f32),
}

impl Default for FontVal {
    /**
     * Returns the default font size of 12 pixels.
     */
    fn default() -> Self {
        FontVal::Px(12.0)
    }
}

impl FontVal {
    /**
     * Computes the absolute font size in pixels, resolving rem units using a base size.
     *
     * @param base Optional base font size in pixels for rem calculations. Defaults to 1.0 if not provided.
     * @return The computed font size in pixels.
     */
    pub fn get(&self, base: Option<f32>) -> f32 {
        match self {
            FontVal::Px(x) => x.clone(),
            FontVal::Rem(x) => x * base.unwrap_or(1.0),
        }
    }
}

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
    pub pointer_events: Option<Pickable>,
}

impl Style {
    /**
     * Merges another `Style` into this one.
     * For each field, if the other style has a value set (`Some`), it overwrites this style's value.
     *
     * @param other The other style to merge from.
     */
    pub fn merge(&mut self, other: &Style) {
        if other.display.is_some() {
            self.display = other.display.clone();
        }
        if other.position_type.is_some() {
            self.position_type = other.position_type.clone();
        }
        if other.width.is_some() {
            self.width = other.width.clone();
        }
        if other.min_width.is_some() {
            self.min_width = other.min_width.clone();
        }
        if other.max_width.is_some() {
            self.max_width = other.max_width.clone();
        }
        if other.height.is_some() {
            self.height = other.height.clone();
        }
        if other.min_height.is_some() {
            self.min_height = other.min_height.clone();
        }
        if other.max_height.is_some() {
            self.max_height = other.max_height.clone();
        }
        if other.left.is_some() {
            self.left = other.left.clone();
        }
        if other.top.is_some() {
            self.top = other.top.clone();
        }
        if other.right.is_some() {
            self.right = other.right.clone();
        }
        if other.bottom.is_some() {
            self.bottom = other.bottom.clone();
        }
        if other.padding.is_some() {
            self.padding = other.padding.clone();
        }
        if other.margin.is_some() {
            self.margin = other.margin.clone();
        }
        if other.border.is_some() {
            self.border = other.border.clone();
        }
        if other.overflow.is_some() {
            self.overflow = other.overflow.clone();
        }
        if other.color.is_some() {
            self.color = other.color.clone();
        }
        if other.background.is_some() {
            self.background = other.background.clone();
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color.clone();
        }
        if other.border_width.is_some() {
            self.border_width = other.border_width.clone();
        }
        if other.border_radius.is_some() {
            self.border_radius = other.border_radius.clone();
        }
        if other.font_size.is_some() {
            self.font_size = other.font_size.clone();
        }
        if other.box_shadow.is_some() {
            self.box_shadow = other.box_shadow.clone();
        }
        if other.justify_content.is_some() {
            self.justify_content = other.justify_content.clone();
        }
        if other.justify_items.is_some() {
            self.justify_items = other.justify_items.clone();
        }
        if other.justify_self.is_some() {
            self.justify_self = other.justify_self.clone();
        }
        if other.align_content.is_some() {
            self.align_content = other.align_content.clone();
        }
        if other.align_items.is_some() {
            self.align_items = other.align_items.clone();
        }
        if other.align_self.is_some() {
            self.align_self = other.align_self.clone();
        }
        if other.flex_direction.is_some() {
            self.flex_direction = other.flex_direction.clone();
        }
        if other.flex_grow.is_some() {
            self.flex_grow = other.flex_grow.clone();
        }
        if other.flex_shrink.is_some() {
            self.flex_shrink = other.flex_shrink.clone();
        }
        if other.flex_basis.is_some() {
            self.flex_basis = other.flex_basis.clone();
        }
        if other.flex_wrap.is_some() {
            self.flex_wrap = other.flex_wrap.clone();
        }
        if other.grid_row.is_some() {
            self.grid_row = other.grid_row.clone();
        }
        if other.grid_column.is_some() {
            self.grid_column = other.grid_column.clone();
        }
        if other.grid_auto_flow.is_some() {
            self.grid_auto_flow = other.grid_auto_flow.clone();
        }
        if other.grid_template_rows.is_some() {
            self.grid_template_rows = other.grid_template_rows.clone();
        }
        if other.grid_template_columns.is_some() {
            self.grid_template_columns = other.grid_template_columns.clone();
        }
        if other.grid_auto_rows.is_some() {
            self.grid_auto_rows = other.grid_auto_rows.clone();
        }
        if other.grid_auto_columns.is_some() {
            self.grid_auto_columns = other.grid_auto_columns.clone();
        }
        if other.gap.is_some() {
            self.gap = other.gap.clone();
        }
        if other.text_wrap.is_some() {
            self.text_wrap = other.text_wrap.clone();
        }
        if other.z_index.is_some() {
            self.z_index = other.z_index.clone();
        }
        if other.pointer_events.is_some() {
            self.pointer_events = other.pointer_events.clone();
        }
    }
}

pub struct ExtendedStylingPlugin;

impl Plugin for ExtendedStylingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UiStyle>();
        app.register_type::<CssClass>();
        app.register_type::<CssID>();
    }
}
