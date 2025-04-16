use bevy::prelude::{Reflect, Val};

#[derive(Reflect, Default, Clone, PartialEq, Debug)]
pub struct Radius {
    pub top_left: Val,
    pub top_right: Val,
    pub bottom_left: Val,
    pub bottom_right: Val,
}

impl Radius {
    pub fn all(val: Val) -> Self {
        Self {
            top_left: val,
            top_right: val,
            bottom_left: val,
            bottom_right: val
        }
    }
}