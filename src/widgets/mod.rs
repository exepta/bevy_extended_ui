use bevy::prelude::*;
use crate::widgets::button::ButtonWidget;
use crate::widgets::containers::DivWidget;

pub mod containers;
pub mod button;
mod input;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DivWidget, ButtonWidget));
    }
}

