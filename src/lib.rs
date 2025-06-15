use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::html::HtmlPlugin;
use crate::service::ServicePlugin;
use crate::styling::StylingPlugin;
use crate::widgets::WidgetPlugin;

pub mod widgets;
pub mod styling;
pub mod html;
pub mod prelude;
pub mod utils;
pub mod service;

static UI_ID_GENERATE: AtomicUsize = AtomicUsize::new(1);

#[derive(Resource, Default)]
pub struct ImageCache {
    pub map: HashMap<String, Handle<Image>>,
}

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

#[derive(Resource, Debug, Clone)]
pub struct CurrentWidgetState {
    pub widget_id: usize,
}

impl Default for CurrentWidgetState {
    fn default() -> Self {
        Self {
            widget_id: 0,
        }
    }
}

#[derive(Component)]
struct UiCamera;

#[derive(Component)]
pub struct IgnoreParentState;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UIGenID(usize);

impl Default for UIGenID {
    fn default() -> Self {
        Self(UI_ID_GENERATE.fetch_add(1, Relaxed))
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BindToID(pub usize);

#[derive(Component, Reflect, Default, PartialEq, Eq, Debug, Clone)]
#[reflect(Component)]
pub struct UIWidgetState {
    pub focused: bool,
    pub hovered: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub checked: bool,
    pub open: bool,
}

pub struct ExtendedUiPlugin;

impl Plugin for ExtendedUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtendedUiConfiguration>();
        app.init_resource::<ImageCache>();
        app.init_resource::<CurrentWidgetState>();
        app.register_type::<UIGenID>();
        app.register_type::<BindToID>();
        app.register_type::<UIWidgetState>();
        app.add_plugins((HtmlPlugin, StylingPlugin, ServicePlugin, WidgetPlugin));
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
                Msaa::Sample4,
                RenderLayers::from_layers(configuration.render_layers.as_slice()),
                Transform::from_translation(Vec3::Z * 1000.0),
                UiCamera,
            ));

            info!("Ui Camera created!");
        }
    } else {
        for (entity, _, _) in query.iter() {
            commands.entity(entity).despawn();
            info!("Ui Camera removed!");
        }
    }
}
