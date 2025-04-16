pub mod resources;
pub mod widgets;
pub mod styles;
mod services;
pub mod global;
pub mod utils;

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::UiElementState;
use crate::resources::ExtendedUiConfiguration;
use crate::services::ServicesPlugin;
use crate::styles::StylesPlugin;
use crate::widgets::WidgetsPlugin;

#[derive(Component)]
struct UiCamera;

pub struct ExtendedUiPlugin;

impl Plugin for ExtendedUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtendedUiConfiguration>();
        app.register_type::<UiElementState>();
        app.add_plugins((ServicesPlugin, StylesPlugin, WidgetsPlugin));
        app.add_systems(Update, load_ui_camera_system
            .run_if(resource_changed::<ExtendedUiConfiguration>));
    }
}

fn load_ui_camera_system(
    mut commands: Commands,
    configuration: Res<ExtendedUiConfiguration>,
    mut query: Query<(Entity, &mut Camera, &mut RenderLayers), With<UiCamera>>,
) {
    if configuration.enable_default_camera {
        if let Some((_, mut camera, mut layers)) = query.iter_mut().next() {
            camera.hdr = configuration.hdr_support;
            camera.order = configuration.order;
            *layers = RenderLayers::from_layers(configuration.render_layers.as_slice());

            info!("Ui Camera updated!");
        } else {
            commands.spawn((
                Name::new("Extended Ui Camera"),
                Camera2d,
                Camera {
                    hdr: configuration.hdr_support,
                    order: configuration.order,
                    ..default()
                },
                RenderLayers::from_layers(configuration.render_layers.as_slice()),
                Transform::from_translation(Vec3::Z * 1000.0),
                UiCamera,
            ));

            info!("Ui Camera created!");
        }
    } else {
        for (entity, _camera, _layers) in query.iter() {
            commands.entity(entity).despawn_recursive();
            info!("Ui Camera removed!");
        }
    }
}

