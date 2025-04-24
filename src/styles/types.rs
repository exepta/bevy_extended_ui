use bevy::prelude::*;
use crate::styles::css_types::{Background, IconPlace};
use crate::styles::{LabelStyle, Style};

#[derive(Component)]
pub struct CheckBoxStyle;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ButtonStyle {
    pub style: Style,
    pub label_style: LabelStyle,
    pub icon_path: Option<String>,
    pub icon_place: IconPlace
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(45.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                gap_column: Val::Px(16.),
                background: Background {
                    image: None,
                    color: Color::srgb_u8(50, 168, 80)
                },
                box_shadow: BoxShadow { 
                    color: Color::srgba(0.0, 0.0, 0.0, 0.75),
                    blur_radius: Val::Px(4.),
                    spread_radius: Val::Px(4.),
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(1.0),
                },
                ..default()
            },
            label_style: LabelStyle {
                color: Color::BLACK,
                font_size: 15.,
                line_break: LineBreak::NoWrap,
                ..default()
            },
            icon_path: None,
            icon_place: Default::default()
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct DivStyle {
    pub style: Style
}