use bevy::prelude::*;
use crate::widgets::containers::DivWidget;

pub mod containers;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DivWidget);
    }
}

