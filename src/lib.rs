mod utils;
pub mod io;

use bevy::app::PluginGroupBuilder;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use crate::io::UiIoPlugin;

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct ExtendedUiConfiguration {
    pub internal_asset_path: String,
    pub used_render_layers: Vec<usize>,
    pub enable_default_camera: bool,
    pub default_camera_configuration: DefaultUiCameraConfiguration
}


impl Default for ExtendedUiConfiguration {
    fn default() -> Self {
        Self {
            internal_asset_path: "assets/extended_ui/".to_string(),
            used_render_layers: vec![1, 2],
            enable_default_camera: true,
            default_camera_configuration: DefaultUiCameraConfiguration::default()
        }
    }

}

#[derive(Reflect, Debug)]
pub struct DefaultUiCameraConfiguration {
    pub order: isize,
    pub hdr_enabled: bool,
    pub camera_name: String
}

impl Default for DefaultUiCameraConfiguration {
    fn default() -> Self {
        DefaultUiCameraConfiguration {
            order: 2,
            hdr_enabled: true,
            camera_name: "ExtendedUiCamera".to_string()
        }
    }
}

/// Marker component for the UI camera entity.
///
/// This component tags the camera entity used for rendering the UI.
#[derive(Component)]
struct UiCamera;

struct InternalPlugin;

impl Plugin for InternalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiIoPlugin);
        app.init_resource::<ExtendedUiConfiguration>();
        app.add_systems(Update, load_ui_camera_system
            .run_if(resource_changed::<ExtendedUiConfiguration>));
    }
}

pub struct ExtendedUiPlugin;

impl Plugin for ExtendedUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InternalPlugin);
    }
}

pub trait HotReloadExt {
    fn with_asset_reload(self, enabled: bool) -> PluginGroupBuilder;
}

impl HotReloadExt for PluginGroupBuilder {
    fn with_asset_reload(self, enabled: bool) -> PluginGroupBuilder {
        self.set(AssetPlugin {
            watch_for_changes_override: Some(enabled),
            ..default()
        })
    }
}

/// System that manages the lifecycle and configuration of the UI camera.
///
/// This system checks the `ExtendedUiConfiguration` resource to determine whether
/// a default UI camera should be enabled. If enabled, it either updates an existing
/// UI camera's settings or spawns a new one with the configured parameters.
///
/// If the configuration disables the default UI camera, it despawns any existing UI cameras.
///
/// # Parameters
/// - `commands`: To spawn or despawn entities.
/// - `configuration`: Resource containing UI camera settings.
/// - `query`: Query to find existing UI cameras for update or removal.
fn load_ui_camera_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Camera, &mut RenderLayers), With<UiCamera>>,
    configuration: Res<ExtendedUiConfiguration>,
) {
    if configuration.enable_default_camera {
        if let Some((camera_entity, mut camera, mut render_layers)) = query.iter_mut().next() {
            camera.order = configuration.default_camera_configuration.order;
            *render_layers = RenderLayers::from_layers(configuration.used_render_layers.as_slice());

            if configuration.default_camera_configuration.hdr_enabled {
                commands.entity(camera_entity).insert(Hdr::default());
            }

            info!("ExtendedUI Camera updated!")
        } else {
            let camera_entity = commands.spawn((
                Name::new(configuration.default_camera_configuration.camera_name.clone()),
                Camera2d,
                Camera {
                    order: configuration.default_camera_configuration.order,
                    ..Default::default()
                },
                Msaa::Sample4,
                RenderLayers::from_layers(configuration.used_render_layers.as_slice()),
                Transform::from_translation(Vec3::Z * 1000.0),
                UiCamera,
            )).id();

            if configuration.default_camera_configuration.hdr_enabled {
                commands.entity(camera_entity).insert(Hdr::default());
            }

            info!("ExtendedUI Camera created!")
        }
    } else {
        for (camera_entity, _, _) in query.iter() {
            commands.entity(camera_entity).despawn();
            info!("ExtendedUI Camera removed!");
        }
    }
}