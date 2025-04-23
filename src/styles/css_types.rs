use bevy::prelude::*;

#[derive(Reflect, Component, Debug, Clone)]
#[reflect(Component)]
pub struct Background {
    pub color: Color,
    pub image: Option<String>,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            image: None,
        }
    }
}

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

#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum IconPlace {
    Left,
    Right
}

impl Default for IconPlace {
    fn default() -> Self {
        IconPlace::Right
    }
}