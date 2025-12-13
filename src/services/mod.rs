mod css_service;
mod style_service;
mod state_service;
mod image_service;

use bevy::prelude::*;
use crate::services::css_service::CssService;
use crate::services::image_service::ImageCacheService;
use crate::services::state_service::StateService;
use crate::services::style_service::StyleService;

pub struct ExtendedServicePlugin;

impl Plugin for ExtendedServicePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CssService, StateService, StyleService, ImageCacheService));
    }
}