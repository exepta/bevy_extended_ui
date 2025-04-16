use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct ExtendedUiConfiguration {
    pub order: isize,
    pub hdr_support: bool,
    pub enable_default_camera: bool,
    pub render_layers: Vec<usize>,
}

impl Default for ExtendedUiConfiguration {
    fn default() -> Self {
        Self {
            order: 2,
            hdr_support: true,
            enable_default_camera: true,
            render_layers: vec![1, 2],
        }
    }
}