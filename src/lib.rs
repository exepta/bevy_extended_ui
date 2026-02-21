use crate::html::ExtendedUiHtmlPlugin;
use crate::io::ExtendedIoPlugin;
use crate::registry::ExtendedRegistryPlugin;
use crate::services::ExtendedServicePlugin;
use crate::styles::ExtendedStylingPlugin;
use crate::widgets::ExtendedWidgetPlugin;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use std::collections::HashMap;

pub mod example_utils;
pub mod html;
pub mod io;
pub mod lang;
pub mod registry;
pub mod services;
pub mod styles;
mod unit_tests;
pub mod utils;
pub mod widgets;

pub use lang::{UILang, UiLangVariables};

/// A cache mapping image paths to their loaded handles,
/// preventing duplicate loads and allowing cleanup of unused images.
#[derive(Resource, Default)]
pub struct ImageCache {
    pub map: HashMap<String, Handle<Image>>,
}

/// Global UI configuration resource.
///
/// Controls UI camera order, HDR support, whether the default UI camera is enabled,
/// and which render layers to use.
#[derive(Resource, Debug, Clone)]
pub struct ExtendedUiConfiguration {
    pub order: isize,
    pub hdr_support: bool,
    pub camera: ExtendedCam,
    pub render_layers: Vec<usize>,
    pub assets_path: String,
    pub language_path: String,
}

impl Default for ExtendedUiConfiguration {
    /// Returns a default `ExtendedUiConfiguration` with:
    /// - `order` = 2
    /// - `hdr_support` disabled (Currently Buggy!)
    /// - `camera` default of [`ExtendedCam`]
    /// - `render_layers` set to layers 1 and 2
    /// - `assets_path`: for preload images. Default `assets/extended_ui/`
    /// - `language_path`: for translations. Default `assets/lang`
    fn default() -> Self {
        Self {
            order: 2,
            hdr_support: false,
            camera: ExtendedCam::default(),
            render_layers: vec![1, 2],
            assets_path: String::from("assets/extended_ui/"),
            language_path: String::from("assets/lang"),
        }
    }
}

/// Defines which camera setup should be used by the extended UI rendering pipeline.
///
/// This enum is typically used as a configuration option to select a specific camera mode:
/// - [`ExtendedCam::Default`] uses the recommended default camera configuration.
/// - [`ExtendedCam::Simple`] uses a minimal camera setup (useful for lightweight scenes or testing).
/// - [`ExtendedCam::None`] disables automatic camera spawning/handling completely.
#[derive(Debug, Clone, Default)]
pub enum ExtendedCam {
    /// Use the recommended default camera setup.
    #[default]
    Default,
    /// Use a minimal / lightweight camera setup (useful for tests and small demos).
    Simple,
    /// Disable camera spawning/handling completely.
    None,
}

/// Tracks the currently focused or active widget by its ID.
///
/// This resource holds the ID of the widget that currently has focus.
#[derive(Resource, Debug, Clone)]
pub struct CurrentWidgetState {
    pub widget_id: usize,
}

impl Default for CurrentWidgetState {
    /// Returns a default `CurrentWidgetState` with `widget_id` set to 0
    /// (meaning no widget currently focused).
    fn default() -> Self {
        Self { widget_id: 0 }
    }
}

/// Marker component for the UI camera entity.
#[derive(Component)]
struct UiCamera;

/// Bevy plugin that wires up all extended UI subsystems.
pub struct ExtendedUiPlugin;

impl Plugin for ExtendedUiPlugin {
    /// Registers resources, plugins, and systems required by the extended UI.
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtendedUiConfiguration>();
        app.init_resource::<ImageCache>();
        app.init_resource::<CurrentWidgetState>();
        app.init_resource::<UILang>();
        app.init_resource::<UiLangVariables>();
        app.register_type::<Camera>();
        app.add_plugins((
            ExtendedRegistryPlugin,
            ExtendedWidgetPlugin,
            ExtendedServicePlugin,
            ExtendedStylingPlugin,
            ExtendedIoPlugin,
            ExtendedUiHtmlPlugin,
        ));
        app.add_systems(
            Update,
            load_ui_camera_system.run_if(resource_changed::<ExtendedUiConfiguration>),
        );
    }
}

/// Manages the lifecycle and configuration of the UI camera.
///
/// Uses `ExtendedUiConfiguration.camera` to decide which camera setup is active.
fn load_ui_camera_system(
    mut commands: Commands,
    configuration: Res<ExtendedUiConfiguration>,
    mut query: Query<(Entity, &mut Camera, &mut RenderLayers), With<UiCamera>>,
) {
    match configuration.camera {
        ExtendedCam::Default => {
            if let Some((cam_entity, mut camera, mut layers)) = query.iter_mut().next() {
                camera.order = configuration.order;
                *layers = RenderLayers::from_layers(configuration.render_layers.as_slice());

                if configuration.hdr_support {
                    commands.entity(cam_entity).insert(Hdr::default());
                } else {
                    commands.entity(cam_entity).remove::<Hdr>();
                }

                debug!("Ui Camera updated (Default)!");
            } else {
                let cam_entity = commands
                    .spawn((
                        Name::new("Extended Ui Camera"),
                        Camera2d,
                        Camera {
                            order: configuration.order,
                            clear_color: ClearColorConfig::None,
                            ..default()
                        },
                        Msaa::Sample4,
                        RenderLayers::from_layers(configuration.render_layers.as_slice()),
                        UiCamera,
                        IsDefaultUiCamera,
                    ))
                    .id();

                if configuration.hdr_support {
                    commands.entity(cam_entity).insert(Hdr::default());
                }

                debug!("Ui Camera created (Default)!");
            }
        }

        ExtendedCam::Simple => {
            // remove existing UI cameras first (to avoid duplicate cameras)
            for (entity, _, _) in query.iter() {
                commands.entity(entity).despawn();
            }

            commands.spawn((
                Name::new("Extended UI Camera"),
                Camera2d,
                Camera {
                    order: configuration.order,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                UiCamera,
            ));

            debug!("Ui Camera created (Simple)!");
        }

        ExtendedCam::None => {
            for (entity, _, _) in query.iter() {
                commands.entity(entity).despawn();
                debug!("Ui Camera removed!");
            }
        }
    }
}
