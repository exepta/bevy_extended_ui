use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::html::HtmlPlugin;
use crate::observer::ObserverRegistryPlugin;
use crate::registry::{UiInitResource, RegistryPlugin, UiRegistry, UI_ID_GENERATE};
use crate::service::ServicePlugin;
use crate::styling::StylingPlugin;
use crate::widgets::WidgetPlugin;

pub mod widgets;
pub mod styling;
pub mod html;
pub mod registry;
pub mod utils;
pub mod service;
pub mod observer;

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
    pub enable_default_camera: bool,
    pub render_layers: Vec<usize>,
    pub assets_path: String,
}

impl Default for ExtendedUiConfiguration {

    /// Returns a default `ExtendedUiConfiguration` with:
    /// - `order` = 2
    /// - `hdr_support` enabled
    /// - `enable_default_camera` enabled
    /// - `render_layers` set to layers 1 and 2
    fn default() -> Self {
        Self {
            order: 2,
            hdr_support: true,
            enable_default_camera: true,
            render_layers: vec![1, 2],
            assets_path: String::from("assets/extended_ui/"),
        }
    }
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
        Self {
            widget_id: 0,
        }
    }
}

/// Marker component for the UI camera entity.
///
/// This component tags the camera entity used for rendering the UI.
#[derive(Component)]
struct UiCamera;

/// Marker component for UI elements that should ignore the parent widget state.
///
/// Used to mark UI nodes that do not inherit state like `focused`, `hovered`, etc.
#[derive(Component)]
pub struct IgnoreParentState;

/// Unique identifier for UI elements.
///
/// Each UI element should have a unique `UIGenID` generated atomically.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UIGenID(usize);

impl Default for UIGenID {

    /// Generates a new unique `UIGenID` using a global atomic counter.
    fn default() -> Self {
        Self(UI_ID_GENERATE.lock().unwrap().acquire())
    }
}

/// Associates a UI child entity with a parent widget by ID.
///
/// Used for binding UI components to their logical parent.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BindToID(pub usize);

/// Stores the interaction and UI state flags for a widget.
///
/// Contains boolean flags for common widget states such as focused, hovered, disabled, etc.
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
        app.init_resource::<UiInitResource>();
        app.init_resource::<UiRegistry>();
        app.init_resource::<CurrentWidgetState>();
        app.register_type::<UIGenID>();
        app.register_type::<BindToID>();
        app.register_type::<UIWidgetState>();
        app.add_plugins((RegistryPlugin, ObserverRegistryPlugin, HtmlPlugin, StylingPlugin, ServicePlugin, WidgetPlugin));
        app.add_systems(Update, load_ui_camera_system
            .run_if(resource_changed::<ExtendedUiConfiguration>));
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
