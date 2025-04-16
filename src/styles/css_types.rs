use bevy::prelude::*;

#[derive(Reflect, Component, Debug, Clone)]
#[reflect(Component)]
pub struct Background {
    pub color: Color,
    pub image: Option<Handle<Image>>,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            image: None,
        }
    }
}