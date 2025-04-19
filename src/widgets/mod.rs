use bevy::prelude::*;
use crate::widgets::button::ButtonWidget;
use crate::widgets::containers::DivWidget;
use crate::widgets::input::InputWidget;
use crate::widgets::slider::SliderWidget;

pub mod containers;
pub mod button;
pub mod input;
pub mod slider;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DivWidget,
            ButtonWidget,
            InputWidget,
            SliderWidget,
        ));
    }
}

