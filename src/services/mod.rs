mod css_service;
pub mod image_service;
mod state_service;
mod style_service;

use crate::services::css_service::CssService;
use crate::services::image_service::ImageCacheService;
use crate::services::state_service::StateService;
use crate::services::style_service::StyleService;
use bevy::prelude::*;

pub struct ExtendedServicePlugin;

impl Plugin for ExtendedServicePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CssService, StateService, StyleService, ImageCacheService));
    }
}
