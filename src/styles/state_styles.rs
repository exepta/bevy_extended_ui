use bevy::prelude::*;
use crate::styles::types::{ButtonStyle, CheckBoxStyle, DivStyle, SliderStyle};

#[derive(Debug, Clone, Reflect)]
pub enum Styling {
    Button(ButtonStyle),
    Div(DivStyle),
    CheckBox(CheckBoxStyle),
    Slider(SliderStyle),
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Hover(pub Styling);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Selected(pub Styling);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Disabled(pub Styling);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Checked(pub Styling);