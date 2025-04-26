use bevy::prelude::*;
use crate::services::state_service::StateService;

mod state_service;

pub struct ServicesPlugin;

impl Plugin for ServicesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StateService);
    }
}