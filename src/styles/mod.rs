pub mod css_types;
pub mod types;
pub mod state_styles;
pub mod utils;

use bevy::prelude::*;
use bevy::text::FontSmoothing;
use crate::styles::css_types::{Background};
use crate::styles::state_styles::{Base, Hover};
use crate::styles::types::{ButtonStyle, CheckBoxStyle, DivStyle, InputStyle, SliderStyle};
use crate::utils::Radius;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Style {
    pub width: Val,
    pub height: Val,
    pub min_width: Val,
    pub min_height: Val,
    pub max_width: Val,
    pub max_height: Val,
    pub top: Val,
    pub left: Val,
    pub right: Val,
    pub bottom: Val,
    pub padding: UiRect,
    pub margin: UiRect,
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
    pub box_shadow: Option<BoxShadow>,
    pub background: Background,
    pub border: UiRect,
    pub border_radius: Radius,
    pub border_color: Color,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct LabelStyle {
    pub font_path: Option<String>,
    pub font_weight: u16,
    pub font_size: f32,
    pub color: Color,
    pub line_break: LineBreak,
    pub justify: JustifyText,
    pub smoothing: FontSmoothing
}

pub struct StylesPlugin;

impl Plugin for StylesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Background>();
        app.register_type::<Style>();
        app.register_type::<LabelStyle>();
        app.register_type::<Base>();
        app.register_type::<Hover>();
        app.register_type::<ButtonStyle>();
        app.register_type::<DivStyle>();
        app.register_type::<CheckBoxStyle>();
        app.register_type::<SliderStyle>();
        app.register_type::<InputStyle>();
    }
}