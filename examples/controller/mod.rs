mod login_controller;

use bevy::prelude::*;
use crate::controller::login_controller::LoginController;

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoginController);
    }
}