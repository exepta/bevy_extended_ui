use bevy::prelude::*;
use crate::styles::css_types::Background;
use crate::utils::Radius;

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
    pub line_break: Option<LineBreak>
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