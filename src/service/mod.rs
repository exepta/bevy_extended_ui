mod style_service;
mod css_service;
pub mod state_service;
pub mod image_cache_service;
mod html_binding_service;

use bevy::prelude::*;
use crate::service::css_service::CssService;
use crate::service::html_binding_service::HtmlBindingService;
use crate::service::image_cache_service::ImageCacheService;
use crate::service::state_service::StateService;
use crate::service::style_service::StyleService;

pub struct ServicePlugin;

impl Plugin for ServicePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StateService, CssService, StyleService, ImageCacheService, HtmlBindingService));
    }
}