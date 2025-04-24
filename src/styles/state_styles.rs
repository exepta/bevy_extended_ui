use bevy::prelude::*;
use crate::styles::types::{ButtonStyle, CheckBoxStyle, DivStyle};

#[derive(Debug, Clone, Reflect)]
pub enum Styling {
    Button(ButtonStyle),
    Div(DivStyle),
    CheckBox(CheckBoxStyle),
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Hover(pub Styling);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Checked(pub Styling);