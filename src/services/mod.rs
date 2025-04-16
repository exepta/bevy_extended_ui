use bevy::prelude::*;
use crate::services::state_service::StateService;
use crate::services::style_service::StyleService;

pub mod style_service;
mod state_service;

pub struct ServicesPlugin;

impl Plugin for ServicesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StateService, StyleService));
    }
}