pub mod image_cache_service;
pub mod state_service;
mod style_service;

use bevy::prelude::*;
use crate::services::image_cache_service::ImageCacheService;
use crate::services::state_service::StateService;
use crate::services::style_service::StyleService;

pub struct ServicePlugin;

impl Plugin for ServicePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StateService, StyleService, ImageCacheService));
    }
}