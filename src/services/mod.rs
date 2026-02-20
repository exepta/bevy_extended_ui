mod css_service;
pub mod image_service;
mod state_service;
mod style_service;
mod unit_tests;

use crate::services::css_service::CssService;
use crate::services::image_service::ImageCacheService;
use crate::services::state_service::StateService;
use crate::services::style_service::StyleService;
use bevy::prelude::*;

/// Plugin that aggregates all service-related plugins.
pub struct ExtendedServicePlugin;

impl Plugin for ExtendedServicePlugin {
    /// Registers service plugins for CSS, state, styles, and images.
    fn build(&self, app: &mut App) {
        app.add_plugins((CssService, StateService, StyleService, ImageCacheService));
    }
}
