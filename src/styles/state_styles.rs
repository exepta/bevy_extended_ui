use bevy::prelude::*;
use crate::styles::types::{ButtonStyle, CheckBoxStyle, DivStyle, InputStyle, SliderStyle};

#[derive(Debug, Clone, Reflect)]
pub enum Styling {
    Button(ButtonStyle),
    Div(DivStyle),
    CheckBox(CheckBoxStyle),
    Slider(SliderStyle),
    InputField(InputStyle),
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Base(pub Styling);

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