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
    pub font: Option<Handle<Font>>,
    pub font_size: Option<f32>,
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

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct BaseStyle(pub Style);

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