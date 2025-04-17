pub mod css_types;

use bevy::prelude::*;
use crate::styles::css_types::Background;
use crate::utils::Radius;

pub trait Styles {}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Style {
    pub width: Val,
    pub min_width: Val,
    pub max_width: Val,
    pub height: Val,
    pub min_height: Val,
    pub max_height: Val,
    pub top: Val,
    pub left: Val,
    pub right: Val,
    pub bottom: Val,
    pub padding: UiRect,
    pub margin: UiRect,
    pub border: UiRect,
    pub border_radius: Radius,
    pub border_color: Color,
    pub background: Background,
    pub display: Display,
    pub position_type: PositionType,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Val,
    pub gap_column: Val,
    pub gap_row: Val,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub align_self: AlignSelf,
    pub justify_items: JustifyItems,
    pub justify_content: JustifyContent,
    pub justify_self: JustifySelf,
    pub color: Color,
    pub placeholder_color: Color,
    pub font: Handle<Font>,
    pub font_size: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            width: Val::default(),
            min_width: Val::default(),
            max_width: Val::default(),
            height: Val::default(),
            min_height: Val::default(),
            max_height: Val::default(),
            top: Val::default(),
            left: Val::default(),
            right: Val::default(),
            bottom: Val::default(),
            padding: UiRect::all(Val::Px(0.0)),
            margin: UiRect::all(Val::Px(0.0)),
            border: UiRect::all(Val::Px(0.0)),
            border_radius: Radius::default(),
            border_color: Color::BLACK,
            background: Background::default(),
            display: Display::Block,
            position_type: PositionType::default(),
            flex_direction: FlexDirection::default(),
            flex_wrap: FlexWrap::default(),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_basis: Val::default(),
            gap_row: Val::Px(0.0),
            gap_column: Val::Px(0.0),
            align_self: AlignSelf::default(),
            align_items: AlignItems::default(),
            align_content: AlignContent::default(),
            justify_items: JustifyItems::default(),
            justify_self: JustifySelf::default(),
            justify_content: JustifyContent::default(),
            placeholder_color: Color::srgb(0.6, 0.6, 0.6),
            color: Color::BLACK,
            font: Default::default(),
            font_size: 12.0
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct PartialStyle {
    pub width: Option<Val>,
    pub min_width: Option<Val>,
    pub max_width: Option<Val>,
    pub height: Option<Val>,
    pub min_height: Option<Val>,
    pub max_height: Option<Val>,
    pub top: Option<Val>,
    pub left: Option<Val>,
    pub right: Option<Val>,
    pub bottom: Option<Val>,
    pub padding: Option<UiRect>,
    pub margin: Option<UiRect>,
    pub border: Option<UiRect>,
    pub border_radius: Option<Radius>,
    pub border_color: Option<Color>,
    pub background: Option<Background>,
    pub display: Option<Display>,
    pub position_type: Option<PositionType>,
    pub flex_direction: Option<FlexDirection>,
    pub flex_wrap: Option<FlexWrap>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Val>,
    pub gap_column: Option<Val>,
    pub gap_row: Option<Val>,
    pub align_items: Option<AlignItems>,
    pub align_content: Option<AlignContent>,
    pub align_self: Option<AlignSelf>,
    pub justify_items: Option<JustifyItems>,
    pub justify_content: Option<JustifyContent>,
    pub justify_self: Option<JustifySelf>,
    pub color: Option<Color>,
    pub placeholder_color: Option<Color>,
    pub font: Option<Handle<Font>>,
    pub font_size: Option<f32>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct InternalStyle(pub Style);

impl InternalStyle {
    pub fn merge_styles(&mut self, other: &PartialStyle) {
        if let Some(val) = other.width { self.0.width = val; }
        if let Some(val) = other.min_width { self.0.min_width = val; }
        if let Some(val) = other.max_width { self.0.max_width = val; }
        if let Some(val) = other.height { self.0.height = val; }
        if let Some(val) = other.min_height { self.0.min_height = val; }
        if let Some(val) = other.max_height { self.0.max_height = val; }
        if let Some(val) = other.top { self.0.top = val; }
        if let Some(val) = other.bottom { self.0.bottom = val; }
        if let Some(val) = other.left { self.0.left = val; }
        if let Some(val) = other.right { self.0.right = val; }
        if let Some(val) = other.padding { self.0.padding = val; }
        if let Some(val) = other.margin { self.0.margin = val; }
        if let Some(val) = other.align_content { self.0.align_content = val; }
        if let Some(val) = other.align_self { self.0.align_self = val; }
        if let Some(val) = other.align_items { self.0.align_items = val; }
        if let Some(val) = other.justify_content { self.0.justify_content = val; }
        if let Some(val) = other.justify_self { self.0.justify_self = val; }
        if let Some(val) = other.justify_items { self.0.justify_items = val; }
        if let Some(val) = other.display { self.0.display = val; }
        if let Some(val) = other.position_type { self.0.position_type = val; }
        if let Some(val) = other.border_radius.clone() {
            self.0.border_radius.top_left = val.top_left;
            self.0.border_radius.top_right = val.top_right;
            self.0.border_radius.bottom_left = val.bottom_left;
            self.0.border_radius.bottom_right = val.bottom_right;
        }
        if let Some(val) = other.border { self.0.border = val; }
        if let Some(val) = other.border_color { self.0.border_color = val; }
        if let Some(val) = other.background.clone() { self.0.background = val; }
        if let Some(val) = other.flex_grow { self.0.flex_grow = val; }
        if let Some(val) = other.flex_shrink { self.0.flex_shrink = val; }
        if let Some(val) = other.flex_direction { self.0.flex_direction = val; }
        if let Some(val) = other.flex_basis { self.0.flex_basis = val; }
        if let Some(val) = other.flex_wrap { self.0.flex_wrap = val; }
        if let Some(val) = other.gap_row { self.0.gap_row = val; }
        if let Some(val) = other.gap_column { self.0.gap_column = val; }
        if let Some(val) = other.color { self.0.color = val; }
        if let Some(val) = other.font.clone() { self.0.font = val; }
        if let Some(val) = other.font_size { self.0.font_size = val; }
        if let Some(val) = other.placeholder_color { self.0.placeholder_color = val; }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HoverStyle(pub PartialStyle);

impl Default for HoverStyle {
    fn default() -> Self {
        Self(
            PartialStyle {
                border_color: Some(Color::srgba(1.0, 0.0, 0.0, 1.0)),
                ..default()
            }
        )
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SelectedStyle(pub PartialStyle);

impl Default for SelectedStyle {
    fn default() -> Self {
        Self(
            PartialStyle {
                border_color: Some(Color::srgba(0.0, 1.0, 0.0, 1.0)),
                ..default()
            }
        )
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BaseStyle(pub PartialStyle);

impl Default for BaseStyle {
    fn default() -> Self {
        Self(
            PartialStyle {
                border_color: Some(Color::srgba(0.0, 0.0, 1.0, 1.0)),
                ..default()
            }
        )
    }
}


pub struct StylesPlugin;

impl Plugin for StylesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Background>();
        app.register_type::<Style>();
        app.register_type::<HoverStyle>();
        app.register_type::<SelectedStyle>();
        app.register_type::<BaseStyle>();
    }
}