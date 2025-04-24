use bevy::prelude::*;
use crate::styles::css_types::{Background, IconPlace};
use crate::styles::{LabelStyle, Style};
use crate::utils::Radius;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CheckBoxStyle {
    pub style: Style,
    pub label_style: LabelStyle,
    pub check_size: f32,
    pub check_background: Color,
    pub check_color: Color,
    pub check_border: UiRect,
    pub check_border_radius: Radius,
    pub check_border_color: Color,
    pub check_box_shadow: Option<BoxShadow>,
    pub icon_path: Option<String>,
}

impl Default for CheckBoxStyle {
    fn default() -> Self {
        Self {
            style: Style {
                width: Val::Px(150.),
                min_height: Val::Px(35.),
                display: Display::Flex,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                gap_row: Val::Px(10.),
                background: Background {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                    ..default()
                },
                ..default()
            },
            label_style: LabelStyle {
                font_size: 15.,
                color: Color::BLACK,
                line_break: LineBreak::NoWrap,
                ..default()
            },
            check_size: 20.,
            check_background: Color::srgba(0.0, 0.0, 0.0, 0.0),
            check_border: UiRect::all(Val::Px(2.)),
            check_border_color: Color::BLACK,
            check_color: Color::BLACK,
            check_border_radius: Radius::all(Val::Px(2.)),
            icon_path: Some("icons/check-mark.png".to_string()),
            check_box_shadow: None
        }
    }
}

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
                border_radius: Radius::all(Val::Px(5.)),
                background: Background {
                    image: None,
                    color: Color::srgb_u8(74, 207, 108)
                },
                box_shadow: Some(BoxShadow {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.75),
                    blur_radius: Val::Px(4.),
                    spread_radius: Val::Px(4.),
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(1.0),
                }),
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
