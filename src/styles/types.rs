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
                    color: Color::srgb_u8(143, 201,  249)
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

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SliderStyle {
    pub style: Style,
    pub thumb_width: Val,
    pub thumb_height: Val,
    pub thumb_color: Color,
    pub thumb_border: UiRect,
    pub thumb_border_radius: Radius,
    pub thumb_border_color: Color,
    pub thumb_box_shadow: Option<BoxShadow>,
    pub track_color: Color,
    pub track_border: UiRect,
    pub track_border_radius: Radius,
    pub track_border_color: Color
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            style: Style {
                width: Val::Px(300.),
                min_width: Val::Px(100.),
                height: Val::Px(8.),
                display: Display::Flex,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                background: Background {
                    color: Color::srgb_u8(65, 88,  108),
                    ..default()
                },
                border: UiRect::all(Val::Px(0.)),
                border_radius: Radius::all(Val::Px(5.)),
                ..default()
            },
            thumb_width: Val::Px(20.0),
            thumb_height: Val::Px(20.0),
            thumb_border: UiRect::all(Val::Px(0.0)),
            thumb_border_radius: Radius::all(Val::Percent(50.)),
            thumb_color: Color::srgb_u8(143, 201,  249),
            thumb_box_shadow: Some(BoxShadow {
                color: Color::srgba_u8(143, 201,  249, 180),
                blur_radius: Val::Px(3.),
                spread_radius: Val::Px(3.),
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(0.0),
            }),
            track_border_radius: Radius::all(Val::Px(5.)),
            track_color: Color::srgb_u8(143, 201,  249),
            track_border_color: Color::default(),
            track_border: UiRect::all(Val::Px(0.)),
            thumb_border_color: Color::default()
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct InputStyle {
    pub style: Style,
    pub label_style: LabelStyle,
    pub placeholder_color: Color,
    pub placeholder_font_size: f32,
    pub label_color: Color,
    pub label_font_size: f32,
    pub icon_color: Option<Color>,
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            style: Style {
                width: Val::Px(250.),
                min_width: Val::Px(100.),
                height: Val::Px(50.),
                border_radius: Radius {
                    top_left: Val::Px(5.),
                    top_right: Val::Px(5.),
                    bottom_right: Val::Px(0.),
                    bottom_left: Val::Px(0.)
                },
                border: UiRect::bottom(Val::Px(2.)),
                background: Background {
                    color: Color::srgb_u8(60, 60, 70),
                    ..default()
                },
                border_color: Color::srgb_u8(210, 210, 210),
                ..default()
            },
            label_style: LabelStyle {
                color: Color::srgb(1.0, 1.0, 1.0),
                font_size: 14.,
                line_break: LineBreak::NoWrap,
                ..default()
            },
            label_color: Color::srgb_u8(210, 210, 210),
            icon_color: None,
            label_font_size: 15.,
            placeholder_color: Color::srgb_u8(150, 150, 150),
            placeholder_font_size: 14.,
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct DivStyle {
    pub style: Style
}
