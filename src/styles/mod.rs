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
#[derive(Reflect, Debug, Clone, PartialEq, Copy)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl FontWeight {
    /// Parses CSS-like font-weight names (case-insensitive)
    ///
    /// Examples:
    /// - "bold" -> Bold
    /// - "normal" -> Normal
    /// - "semibold" / "semi-bold" -> SemiBold
    pub fn from_name(name: &str) -> Option<Self> {
        let n = name.trim().to_ascii_lowercase();

        match n.as_str() {
            "thin" => Some(Self::Thin),
            "extralight" | "extra-light" => Some(Self::ExtraLight),
            "light" => Some(Self::Light),
            "normal" | "regular" => Some(Self::Normal),
            "medium" => Some(Self::Medium),
            "semibold" | "semi-bold" => Some(Self::SemiBold),
            "bold" => Some(Self::Bold),
            "extrabold" | "extra-bold" => Some(Self::ExtraBold),
            "black" | "heavy" => Some(Self::Black),
            _ => None,
        }
    }

    /// Parses numeric CSS font-weight values
    ///
    /// Examples:
    /// - 700 -> Bold
    /// - 650 -> SemiBold (nearest lower)
    /// - 999 -> Black
    pub fn from_number(value: u16) -> Option<Self> {
        Some(match value {
            100 => Self::Thin,
            200 => Self::ExtraLight,
            300 => Self::Light,
            400 => Self::Normal,
            500 => Self::Medium,
            600 => Self::SemiBold,
            700 => Self::Bold,
            800 => Self::ExtraBold,
            900 => Self::Black,
            _ => Self::Normal,
        })
    }

    /// Returns the numeric weight (100–900)
    pub fn as_number(self) -> u16 {
        self as u16
    }
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

#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct FontFamily(pub String);

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Copy)]
pub enum TransitionTiming {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl TransitionTiming {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            TransitionTiming::Linear => t,
            TransitionTiming::Ease => t * t * (3.0 - 2.0 * t),
            TransitionTiming::EaseIn => t * t,
            TransitionTiming::EaseOut => 1.0 - (1.0 - t).powi(2),
            TransitionTiming::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "linear" => Some(Self::Linear),
            "ease" => Some(Self::Ease),
            "ease-in" => Some(Self::EaseIn),
            "ease-out" => Some(Self::EaseOut),
            "ease-in-out" => Some(Self::EaseInOut),
            _ => None,
        }
    }
}

impl Default for TransitionTiming {
    fn default() -> Self {
        TransitionTiming::EaseInOut
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Copy)]
pub enum TransitionProperty {
    All,
    Color,
    Background,
}

impl Default for TransitionProperty {
    fn default() -> Self {
        TransitionProperty::All
    }
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct TransitionSpec {
    pub properties: Vec<TransitionProperty>,
    pub duration: f32,
    pub delay: f32,
    pub timing: TransitionTiming,
}

impl Default for TransitionSpec {
    fn default() -> Self {
        Self {
            properties: vec![TransitionProperty::All],
            duration: 0.3,
            delay: 0.0,
            timing: TransitionTiming::EaseInOut,
        }
    }
}

#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct StylePair {
    pub important: Style,
    pub normal: Style,
}

/// Comprehensive style properties for UI elements.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Style {
    pub display: Option<Display>,
    pub box_sizing: Option<BoxSizing>,
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
    pub font_family: Option<FontFamily>,
    pub font_weight: Option<FontWeight>,
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
    pub scrollbar_width: Option<f32>,
    pub transition: Option<TransitionSpec>,
}

impl Style {
    /**
     * Merges another `Style` into this one.
     * For each field, if the other style has a value set (`Some`), it overwrites this style's value.
     *
     * @param other The other style to merge from.
     */
    pub fn merge(&mut self, other: &Style) {
        merge_opt(&mut self.display, &other.display);
        merge_opt(&mut self.box_sizing, &other.box_sizing);
        merge_opt(&mut self.position_type, &other.position_type);

        merge_opt(&mut self.width, &other.width);
        merge_opt(&mut self.min_width, &other.min_width);
        merge_opt(&mut self.max_width, &other.max_width);

        merge_opt(&mut self.height, &other.height);
        merge_opt(&mut self.min_height, &other.min_height);
        merge_opt(&mut self.max_height, &other.max_height);

        merge_opt(&mut self.left, &other.left);
        merge_opt(&mut self.top, &other.top);
        merge_opt(&mut self.right, &other.right);
        merge_opt(&mut self.bottom, &other.bottom);

        merge_opt(&mut self.padding, &other.padding);
        merge_opt(&mut self.margin, &other.margin);
        merge_opt(&mut self.border, &other.border);

        merge_opt(&mut self.overflow, &other.overflow);

        merge_opt(&mut self.color, &other.color);
        merge_opt(&mut self.background, &other.background);

        merge_opt(&mut self.border_color, &other.border_color);
        merge_opt(&mut self.border_width, &other.border_width);
        merge_opt(&mut self.border_radius, &other.border_radius);

        merge_opt(&mut self.font_size, &other.font_size);
        merge_opt(&mut self.font_family, &other.font_family);
        merge_opt(&mut self.font_weight, &other.font_weight);
        merge_opt(&mut self.box_shadow, &other.box_shadow);

        merge_opt(&mut self.justify_content, &other.justify_content);
        merge_opt(&mut self.justify_items, &other.justify_items);
        merge_opt(&mut self.justify_self, &other.justify_self);

        merge_opt(&mut self.align_content, &other.align_content);
        merge_opt(&mut self.align_items, &other.align_items);
        merge_opt(&mut self.align_self, &other.align_self);

        merge_opt(&mut self.flex_direction, &other.flex_direction);
        merge_opt(&mut self.flex_wrap, &other.flex_wrap);
        merge_opt(&mut self.flex_grow, &other.flex_grow);
        merge_opt(&mut self.flex_shrink, &other.flex_shrink);
        merge_opt(&mut self.flex_basis, &other.flex_basis);

        merge_opt(&mut self.grid_row, &other.grid_row);
        merge_opt(&mut self.grid_column, &other.grid_column);
        merge_opt(&mut self.grid_auto_flow, &other.grid_auto_flow);

        merge_opt(&mut self.grid_template_rows, &other.grid_template_rows);
        merge_opt(
            &mut self.grid_template_columns,
            &other.grid_template_columns,
        );
        merge_opt(&mut self.grid_auto_rows, &other.grid_auto_rows);
        merge_opt(&mut self.grid_auto_columns, &other.grid_auto_columns);

        merge_opt(&mut self.gap, &other.gap);

        merge_opt(&mut self.text_wrap, &other.text_wrap);

        merge_opt(&mut self.z_index, &other.z_index);
        merge_opt(&mut self.pointer_events, &other.pointer_events);

        merge_opt(&mut self.scrollbar_width, &other.scrollbar_width);
        merge_opt(&mut self.transition, &other.transition);
    }
}

#[inline]
fn merge_opt<T: Clone>(dst: &mut Option<T>, src: &Option<T>) {
    if let Some(v) = src.as_ref() {
        *dst = Some(v.clone());
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
