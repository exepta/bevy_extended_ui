pub mod components;
pub mod paint;
pub mod parser;

use crate::io::CssAsset;
use crate::styles::components::UiStyle;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};

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

/// Component that stores one or more CSS asset handles for an entity.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct CssSource(pub Vec<Handle<CssAsset>>);

impl CssSource {
    /// Creates a CSS source from a single asset path.
    pub fn from_path(asset_server: &AssetServer, path: &str) -> Self {
        Self(vec![asset_server.load::<CssAsset>(path.to_string())])
    }

    /// Appends another CSS asset path to the source list.
    pub fn push_path(&mut self, asset_server: &AssetServer, path: &str) {
        self.0.push(asset_server.load::<CssAsset>(path.to_string()));
    }

    /// Creates a CSS source from multiple asset paths.
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
    /// Creates a `Radius` where all corners have the same radius value.
    pub fn all(val: Val) -> Self {
        Self {
            top_left: val,
            top_right: val,
            bottom_left: val,
            bottom_right: val,
        }
    }
}

/// Defines the background style including color, optional image, and optional gradient.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Background {
    pub color: Color,
    pub image: Option<String>,
    pub gradient: Option<LinearGradient>,
}

impl Default for Background {
    /// Creates a default `Background` with transparent color and no image.
    fn default() -> Self {
        Self {
            color: Color::NONE,
            image: None,
            gradient: None,
        }
    }
}

/// Defines how a background image is positioned.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct BackgroundPosition {
    pub x: BackgroundPositionValue,
    pub y: BackgroundPositionValue,
}

impl Default for BackgroundPosition {
    fn default() -> Self {
        Self {
            x: BackgroundPositionValue::Percent(0.0),
            y: BackgroundPositionValue::Percent(0.0),
        }
    }
}

/// Represents a single background position axis value.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum BackgroundPositionValue {
    Percent(f32),
    Px(f32),
}

/// Defines how a background image is sized.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum BackgroundSize {
    Auto,
    Cover,
    Contain,
    Explicit(BackgroundSizeValue, BackgroundSizeValue),
}

impl Default for BackgroundSize {
    fn default() -> Self {
        Self::Auto
    }
}

/// Represents a single background size axis value.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum BackgroundSizeValue {
    Auto,
    Percent(f32),
    Px(f32),
}

/// Defines how the background image is attached.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum BackgroundAttachment {
    Scroll,
    Fixed,
    Local,
}

impl Default for BackgroundAttachment {
    fn default() -> Self {
        Self::Scroll
    }
}

/// Defines supported backdrop-filter effects.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum BackdropFilter {
    Blur(f32),
}

/// Represents a parsed CSS `linear-gradient(...)` definition.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub angle: f32,
    pub stops: Vec<GradientStop>,
}

/// Represents a single color stop in a linear gradient.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    pub position: Option<GradientStopPosition>,
}

/// Represents a gradient stop position.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum GradientStopPosition {
    Percent(f32),
    Px(f32),
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
    /// Returns the default icon placement (`Right`).
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
    /// Returns the default font size of 12 pixels.
    fn default() -> Self {
        FontVal::Px(12.0)
    }
}

impl FontVal {
    /// Computes the absolute font size in pixels, resolving rem units using a base size.
    pub fn get(&self, base: Option<f32>) -> f32 {
        match self {
            FontVal::Px(x) => x.clone(),
            FontVal::Rem(x) => x * base.unwrap_or(1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcUnit {
    None,
    Px,
    Percent,
    Rem,
    Vw,
    Vh,
    VMin,
    VMax,
    Deg,
    Rad,
    Turn,
    Fr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalcValue {
    pub value: f32,
    pub unit: CalcUnit,
}

impl CalcValue {
    pub fn new(value: f32, unit: CalcUnit) -> Self {
        Self { value, unit }
    }

    fn to_length_px(self, ctx: CalcContext) -> Option<f32> {
        match self.unit {
            CalcUnit::Px => Some(self.value),
            CalcUnit::Percent => Some(ctx.base * self.value / 100.0),
            CalcUnit::Vw => Some(ctx.viewport.x * self.value / 100.0),
            CalcUnit::Vh => Some(ctx.viewport.y * self.value / 100.0),
            CalcUnit::VMin => Some(ctx.viewport.x.min(ctx.viewport.y) * self.value / 100.0),
            CalcUnit::VMax => Some(ctx.viewport.x.max(ctx.viewport.y) * self.value / 100.0),
            CalcUnit::None if self.value == 0.0 => Some(0.0),
            _ => None,
        }
    }

    fn to_angle_radians(self) -> Option<f32> {
        match self.unit {
            CalcUnit::None => Some(self.value),
            CalcUnit::Deg => Some(self.value.to_radians()),
            CalcUnit::Rad => Some(self.value),
            CalcUnit::Turn => Some(self.value * std::f32::consts::TAU),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcExpr {
    Value(CalcValue),
    Add(Box<CalcExpr>, Box<CalcExpr>),
    Sub(Box<CalcExpr>, Box<CalcExpr>),
    Mul(Box<CalcExpr>, Box<CalcExpr>),
    Div(Box<CalcExpr>, Box<CalcExpr>),
    Min(Vec<CalcExpr>),
    Max(Vec<CalcExpr>),
    Sin(Box<CalcExpr>),
}

#[derive(Debug, Clone, Copy)]
pub struct CalcContext {
    pub base: f32,
    pub viewport: Vec2,
}

impl CalcExpr {
    pub fn eval_length(&self, ctx: CalcContext) -> Option<f32> {
        match self {
            CalcExpr::Value(value) => value.to_length_px(ctx),
            CalcExpr::Add(a, b) => Some(a.eval_length(ctx)? + b.eval_length(ctx)?),
            CalcExpr::Sub(a, b) => Some(a.eval_length(ctx)? - b.eval_length(ctx)?),
            CalcExpr::Mul(a, b) => {
                let left_len = a.eval_length(ctx);
                let right_len = b.eval_length(ctx);
                let left_num = a.eval_unitless();
                let right_num = b.eval_unitless();

                if let (Some(len), Some(num)) = (left_len, right_num) {
                    return Some(len * num);
                }
                if let (Some(num), Some(len)) = (left_num, right_len) {
                    return Some(len * num);
                }
                None
            }
            CalcExpr::Div(a, b) => {
                let denom = b.eval_unitless()?;
                if denom == 0.0 {
                    return None;
                }
                Some(a.eval_length(ctx)? / denom)
            }
            CalcExpr::Min(values) => {
                let mut best: Option<f32> = None;
                for value in values {
                    let resolved = value.eval_length(ctx)?;
                    best = Some(match best {
                        Some(current) => current.min(resolved),
                        None => resolved,
                    });
                }
                best
            }
            CalcExpr::Max(values) => {
                let mut best: Option<f32> = None;
                for value in values {
                    let resolved = value.eval_length(ctx)?;
                    best = Some(match best {
                        Some(current) => current.max(resolved),
                        None => resolved,
                    });
                }
                best
            }
            CalcExpr::Sin(inner) => {
                let angle = inner.eval_angle_radians()?;
                Some(angle.sin())
            }
        }
    }

    pub fn eval_unitless(&self) -> Option<f32> {
        match self {
            CalcExpr::Value(value) if value.unit == CalcUnit::None => Some(value.value),
            CalcExpr::Add(a, b) => Some(a.eval_unitless()? + b.eval_unitless()?),
            CalcExpr::Sub(a, b) => Some(a.eval_unitless()? - b.eval_unitless()?),
            CalcExpr::Mul(a, b) => Some(a.eval_unitless()? * b.eval_unitless()?),
            CalcExpr::Div(a, b) => {
                let denom = b.eval_unitless()?;
                if denom == 0.0 {
                    return None;
                }
                Some(a.eval_unitless()? / denom)
            }
            CalcExpr::Min(values) => {
                let mut best: Option<f32> = None;
                for value in values {
                    let resolved = value.eval_unitless()?;
                    best = Some(match best {
                        Some(current) => current.min(resolved),
                        None => resolved,
                    });
                }
                best
            }
            CalcExpr::Max(values) => {
                let mut best: Option<f32> = None;
                for value in values {
                    let resolved = value.eval_unitless()?;
                    best = Some(match best {
                        Some(current) => current.max(resolved),
                        None => resolved,
                    });
                }
                best
            }
            CalcExpr::Sin(inner) => {
                let angle = inner.eval_angle_radians()?;
                Some(angle.sin())
            }
            _ => None,
        }
    }

    fn eval_angle_radians(&self) -> Option<f32> {
        match self {
            CalcExpr::Value(value) => value.to_angle_radians(),
            CalcExpr::Add(a, b) => Some(a.eval_angle_radians()? + b.eval_angle_radians()?),
            CalcExpr::Sub(a, b) => Some(a.eval_angle_radians()? - b.eval_angle_radians()?),
            CalcExpr::Mul(a, b) => {
                let left = a.eval_angle_radians();
                let right = b.eval_angle_radians();
                let left_num = a.eval_unitless();
                let right_num = b.eval_unitless();

                if let (Some(angle), Some(num)) = (left, right_num) {
                    return Some(angle * num);
                }
                if let (Some(num), Some(angle)) = (left_num, right) {
                    return Some(angle * num);
                }
                None
            }
            CalcExpr::Div(a, b) => {
                let denom = b.eval_unitless()?;
                if denom == 0.0 {
                    return None;
                }
                Some(a.eval_angle_radians()? / denom)
            }
            CalcExpr::Min(values) => {
                let mut best: Option<f32> = None;
                for value in values {
                    let resolved = value.eval_angle_radians()?;
                    best = Some(match best {
                        Some(current) => current.min(resolved),
                        None => resolved,
                    });
                }
                best
            }
            CalcExpr::Max(values) => {
                let mut best: Option<f32> = None;
                for value in values {
                    let resolved = value.eval_angle_radians()?;
                    best = Some(match best {
                        Some(current) => current.max(resolved),
                        None => resolved,
                    });
                }
                best
            }
            CalcExpr::Sin(inner) => {
                let angle = inner.eval_angle_radians()?;
                Some(angle.sin())
            }
        }
    }
}

/// Font family name wrapper for style parsing.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct FontFamily(pub String);

/// Timing functions for transitions and animations.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Copy)]
pub enum TransitionTiming {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl TransitionTiming {
    /// Applies the timing function to a normalized progress value.
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

    /// Parses a timing function name into a variant.
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
    /// Returns the default timing function.
    fn default() -> Self {
        TransitionTiming::EaseInOut
    }
}

/// Direction modes for CSS animations.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Copy)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

impl AnimationDirection {
    /// Parses an animation-direction name into a variant.
    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "reverse" => Some(Self::Reverse),
            "alternate" => Some(Self::Alternate),
            "alternate-reverse" => Some(Self::AlternateReverse),
            _ => None,
        }
    }
}

impl Default for AnimationDirection {
    /// Returns the default animation direction.
    fn default() -> Self {
        AnimationDirection::Normal
    }
}

/// Properties that can be targeted by transitions.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Copy)]
pub enum TransitionProperty {
    All,
    Color,
    Background,
    Transform,
}

impl Default for TransitionProperty {
    /// Returns the default transition property selection.
    fn default() -> Self {
        TransitionProperty::All
    }
}

/// Parsed animation specification from CSS.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct AnimationSpec {
    pub name: String,
    pub duration: f32,
    pub delay: f32,
    pub timing: TransitionTiming,
    pub iterations: Option<f32>,
    pub direction: AnimationDirection,
}

impl Default for AnimationSpec {
    /// Creates a default animation specification.
    fn default() -> Self {
        Self {
            name: String::new(),
            duration: 0.0,
            delay: 0.0,
            timing: TransitionTiming::Ease,
            iterations: Some(1.0),
            direction: AnimationDirection::Normal,
        }
    }
}

/// Parsed transition specification from CSS.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct TransitionSpec {
    pub properties: Vec<TransitionProperty>,
    pub duration: f32,
    pub delay: f32,
    pub timing: TransitionTiming,
}

impl Default for TransitionSpec {
    /// Creates a default transition specification.
    fn default() -> Self {
        Self {
            properties: vec![TransitionProperty::All],
            duration: 0.3,
            delay: 0.0,
            timing: TransitionTiming::EaseInOut,
        }
    }
}

/// Parsed keyframe entry for a CSS animation.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct AnimationKeyframe {
    pub progress: f32,
    pub style: Style,
}

/// Parsed CSS result for styles and keyframes.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct ParsedCss {
    pub styles: HashMap<String, StylePair>,
    pub keyframes: HashMap<String, Vec<AnimationKeyframe>>,
}

/// Breakpoint/media condition expression used for CSS `@media` rules.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaQueryCondition {
    Always,
    Never,
    MinWidth(f32),
    MaxWidth(f32),
    Width(f32),
    MinHeight(f32),
    MaxHeight(f32),
    Height(f32),
    OrientationLandscape,
    OrientationPortrait,
    Not(Box<MediaQueryCondition>),
    And(Vec<MediaQueryCondition>),
    Or(Vec<MediaQueryCondition>),
}

impl MediaQueryCondition {
    /// Returns true if the condition matches the given viewport size.
    pub fn matches_viewport(&self, viewport: Vec2) -> bool {
        const EPSILON: f32 = 0.5;

        match self {
            MediaQueryCondition::Always => true,
            MediaQueryCondition::Never => false,
            MediaQueryCondition::MinWidth(value) => viewport.x + EPSILON >= *value,
            MediaQueryCondition::MaxWidth(value) => viewport.x - EPSILON <= *value,
            MediaQueryCondition::Width(value) => (viewport.x - *value).abs() <= EPSILON,
            MediaQueryCondition::MinHeight(value) => viewport.y + EPSILON >= *value,
            MediaQueryCondition::MaxHeight(value) => viewport.y - EPSILON <= *value,
            MediaQueryCondition::Height(value) => (viewport.y - *value).abs() <= EPSILON,
            MediaQueryCondition::OrientationLandscape => viewport.x >= viewport.y,
            MediaQueryCondition::OrientationPortrait => viewport.y > viewport.x,
            MediaQueryCondition::Not(inner) => !inner.matches_viewport(viewport),
            MediaQueryCondition::And(parts) => parts
                .iter()
                .all(|condition| condition.matches_viewport(viewport)),
            MediaQueryCondition::Or(parts) => parts
                .iter()
                .any(|condition| condition.matches_viewport(viewport)),
        }
    }

    /// Produces a deterministic key used to separate media-scoped selector entries.
    pub fn cache_key(&self) -> String {
        match self {
            MediaQueryCondition::Always => "always".to_string(),
            MediaQueryCondition::Never => "never".to_string(),
            MediaQueryCondition::MinWidth(value) => format!("minw:{value:.3}"),
            MediaQueryCondition::MaxWidth(value) => format!("maxw:{value:.3}"),
            MediaQueryCondition::Width(value) => format!("w:{value:.3}"),
            MediaQueryCondition::MinHeight(value) => format!("minh:{value:.3}"),
            MediaQueryCondition::MaxHeight(value) => format!("maxh:{value:.3}"),
            MediaQueryCondition::Height(value) => format!("h:{value:.3}"),
            MediaQueryCondition::OrientationLandscape => "orientation:landscape".to_string(),
            MediaQueryCondition::OrientationPortrait => "orientation:portrait".to_string(),
            MediaQueryCondition::Not(inner) => format!("not({})", inner.cache_key()),
            MediaQueryCondition::And(parts) => {
                let mut key = String::from("and(");
                for (idx, part) in parts.iter().enumerate() {
                    if idx > 0 {
                        key.push(',');
                    }
                    key.push_str(&part.cache_key());
                }
                key.push(')');
                key
            }
            MediaQueryCondition::Or(parts) => {
                let mut key = String::from("or(");
                for (idx, part) in parts.iter().enumerate() {
                    if idx > 0 {
                        key.push(',');
                    }
                    key.push_str(&part.cache_key());
                }
                key.push(')');
                key
            }
        }
    }
}

/// Pair of normal and !important styles with origin tracking.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct StylePair {
    pub important: Style,
    pub normal: Style,
    pub origin: usize,
    pub selector: String,
    #[reflect(ignore)]
    pub media: Option<MediaQueryCondition>,
}

/// Transforms parsed from CSS transform properties.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct TransformStyle {
    pub translation: Option<Val2>,
    pub translation_x: Option<Val>,
    pub translation_y: Option<Val>,
    pub scale: Option<Vec2>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub rotation: Option<f32>,
}

impl TransformStyle {
    /// Returns true when no transform values are set.
    pub fn is_empty(&self) -> bool {
        self.translation.is_none()
            && self.translation_x.is_none()
            && self.translation_y.is_none()
            && self.scale.is_none()
            && self.scale_x.is_none()
            && self.scale_y.is_none()
            && self.rotation.is_none()
    }
}

/// Cursor styling, either a system cursor or a custom asset path.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum CursorStyle {
    System(SystemCursorIcon),
    Custom(String),
}

/// Comprehensive style properties for UI elements.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Style {
    pub display: Option<Display>,
    pub box_sizing: Option<BoxSizing>,
    pub position_type: Option<PositionType>,
    pub width: Option<Val>,
    #[reflect(ignore)]
    pub width_calc: Option<CalcExpr>,
    pub min_width: Option<Val>,
    #[reflect(ignore)]
    pub min_width_calc: Option<CalcExpr>,
    pub max_width: Option<Val>,
    #[reflect(ignore)]
    pub max_width_calc: Option<CalcExpr>,
    pub height: Option<Val>,
    #[reflect(ignore)]
    pub height_calc: Option<CalcExpr>,
    pub min_height: Option<Val>,
    #[reflect(ignore)]
    pub min_height_calc: Option<CalcExpr>,
    pub max_height: Option<Val>,
    #[reflect(ignore)]
    pub max_height_calc: Option<CalcExpr>,
    pub left: Option<Val>,
    #[reflect(ignore)]
    pub left_calc: Option<CalcExpr>,
    pub top: Option<Val>,
    #[reflect(ignore)]
    pub top_calc: Option<CalcExpr>,
    pub right: Option<Val>,
    #[reflect(ignore)]
    pub right_calc: Option<CalcExpr>,
    pub bottom: Option<Val>,
    #[reflect(ignore)]
    pub bottom_calc: Option<CalcExpr>,
    pub padding: Option<UiRect>,
    pub margin: Option<UiRect>,
    pub border: Option<UiRect>,
    pub overflow: Option<Overflow>,
    pub color: Option<Color>,
    pub background: Option<Background>,
    pub backdrop_filter: Option<BackdropFilter>,
    pub background_position: Option<BackgroundPosition>,
    pub background_size: Option<BackgroundSize>,
    pub background_attachment: Option<BackgroundAttachment>,
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
    #[reflect(ignore)]
    pub flex_basis_calc: Option<CalcExpr>,
    pub flex_wrap: Option<FlexWrap>,
    pub grid_row: Option<GridPlacement>,
    pub grid_column: Option<GridPlacement>,
    pub grid_auto_flow: Option<GridAutoFlow>,
    pub grid_template_rows: Option<Vec<RepeatedGridTrack>>,
    pub grid_template_columns: Option<Vec<RepeatedGridTrack>>,
    pub grid_auto_rows: Option<Vec<GridTrack>>,
    pub grid_auto_columns: Option<Vec<GridTrack>>,
    pub gap: Option<Val>,
    #[reflect(ignore)]
    pub gap_calc: Option<CalcExpr>,
    pub row_gap: Option<Val>,
    #[reflect(ignore)]
    pub row_gap_calc: Option<CalcExpr>,
    pub column_gap: Option<Val>,
    #[reflect(ignore)]
    pub column_gap_calc: Option<CalcExpr>,
    pub text_wrap: Option<LineBreak>,
    pub z_index: Option<i32>,
    pub cursor: Option<CursorStyle>,
    pub pointer_events: Option<Pickable>,
    pub scrollbar_width: Option<f32>,
    pub transition: Option<TransitionSpec>,
    pub transform: TransformStyle,
    pub animation: Option<AnimationSpec>,
}

impl Style {
    /// Merges another `Style` into this one, overriding any set fields.
    pub fn merge(&mut self, other: &Style) {
        merge_opt(&mut self.display, &other.display);
        merge_opt(&mut self.box_sizing, &other.box_sizing);
        merge_opt(&mut self.position_type, &other.position_type);

        merge_val_with_calc(
            &mut self.width,
            &mut self.width_calc,
            &other.width,
            &other.width_calc,
        );
        merge_val_with_calc(
            &mut self.min_width,
            &mut self.min_width_calc,
            &other.min_width,
            &other.min_width_calc,
        );
        merge_val_with_calc(
            &mut self.max_width,
            &mut self.max_width_calc,
            &other.max_width,
            &other.max_width_calc,
        );

        merge_val_with_calc(
            &mut self.height,
            &mut self.height_calc,
            &other.height,
            &other.height_calc,
        );
        merge_val_with_calc(
            &mut self.min_height,
            &mut self.min_height_calc,
            &other.min_height,
            &other.min_height_calc,
        );
        merge_val_with_calc(
            &mut self.max_height,
            &mut self.max_height_calc,
            &other.max_height,
            &other.max_height_calc,
        );

        merge_val_with_calc(
            &mut self.left,
            &mut self.left_calc,
            &other.left,
            &other.left_calc,
        );
        merge_val_with_calc(
            &mut self.top,
            &mut self.top_calc,
            &other.top,
            &other.top_calc,
        );
        merge_val_with_calc(
            &mut self.right,
            &mut self.right_calc,
            &other.right,
            &other.right_calc,
        );
        merge_val_with_calc(
            &mut self.bottom,
            &mut self.bottom_calc,
            &other.bottom,
            &other.bottom_calc,
        );

        merge_opt(&mut self.padding, &other.padding);
        merge_opt(&mut self.margin, &other.margin);
        merge_opt(&mut self.border, &other.border);

        merge_opt(&mut self.overflow, &other.overflow);

        merge_opt(&mut self.color, &other.color);
        merge_opt(&mut self.background, &other.background);
        merge_opt(&mut self.backdrop_filter, &other.backdrop_filter);
        merge_opt(&mut self.background_position, &other.background_position);
        merge_opt(&mut self.background_size, &other.background_size);
        merge_opt(
            &mut self.background_attachment,
            &other.background_attachment,
        );

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
        merge_val_with_calc(
            &mut self.flex_basis,
            &mut self.flex_basis_calc,
            &other.flex_basis,
            &other.flex_basis_calc,
        );

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

        merge_val_with_calc(
            &mut self.gap,
            &mut self.gap_calc,
            &other.gap,
            &other.gap_calc,
        );
        merge_val_with_calc(
            &mut self.row_gap,
            &mut self.row_gap_calc,
            &other.row_gap,
            &other.row_gap_calc,
        );
        merge_val_with_calc(
            &mut self.column_gap,
            &mut self.column_gap_calc,
            &other.column_gap,
            &other.column_gap_calc,
        );

        merge_opt(&mut self.text_wrap, &other.text_wrap);

        merge_opt(&mut self.z_index, &other.z_index);
        merge_opt(&mut self.cursor, &other.cursor);
        merge_opt(&mut self.pointer_events, &other.pointer_events);

        merge_opt(&mut self.scrollbar_width, &other.scrollbar_width);
        merge_opt(&mut self.transition, &other.transition);

        merge_opt(
            &mut self.transform.translation,
            &other.transform.translation,
        );
        merge_opt(
            &mut self.transform.translation_x,
            &other.transform.translation_x,
        );
        merge_opt(
            &mut self.transform.translation_y,
            &other.transform.translation_y,
        );
        merge_opt(&mut self.transform.scale, &other.transform.scale);
        merge_opt(&mut self.transform.scale_x, &other.transform.scale_x);
        merge_opt(&mut self.transform.scale_y, &other.transform.scale_y);
        merge_opt(&mut self.transform.rotation, &other.transform.rotation);

        merge_opt(&mut self.animation, &other.animation);
    }
}

/// Copies a source value into a destination if the source is set.
#[inline]
fn merge_opt<T: Clone>(dst: &mut Option<T>, src: &Option<T>) {
    if let Some(v) = src.as_ref() {
        *dst = Some(v.clone());
    }
}

#[inline]
fn merge_val_with_calc<T: Clone>(
    dst_val: &mut Option<T>,
    dst_calc: &mut Option<CalcExpr>,
    src_val: &Option<T>,
    src_calc: &Option<CalcExpr>,
) {
    if let Some(calc) = src_calc.as_ref() {
        *dst_calc = Some(calc.clone());
        *dst_val = None;
    } else if let Some(val) = src_val.as_ref() {
        *dst_val = Some(val.clone());
        *dst_calc = None;
    }
}

/// Bevy plugin registering style-related reflection data.
pub struct ExtendedStylingPlugin;

impl Plugin for ExtendedStylingPlugin {
    /// Registers reflected style-related components.
    fn build(&self, app: &mut App) {
        app.register_type::<UiStyle>();
        app.register_type::<CssClass>();
        app.register_type::<CssID>();
    }
}
