use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{UiGenID, UiElementState };
use crate::resources::ExtendedUiConfiguration;
use crate::styles::{BaseStyle, HoverStyle, SelectedStyle};

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, BaseStyle, HoverStyle, SelectedStyle)]
pub struct DivContainer;

#[derive(Component)]
pub struct DivRoot;

pub struct DivWidget;

impl Plugin for DivWidget {
    fn build(&self, app: &mut App) {
        app.register_type::<DivContainer>();
        app.add_systems(Update, internal_generate_component_system);
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID), Without<DivRoot>>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity , gen_id) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Div-{}", gen_id.0)),
            Node {
                ..default()
            },
            RenderLayers::layer(*layer),
            DivRoot
        ))
            .observe(on_internal_mouse_click)
            .observe(on_internal_mouse_entered)
            .observe(on_internal_mouse_leave);
    }
}

fn on_internal_mouse_click(event: Trigger<Pointer<Click>>, mut query: Query<(Entity, &mut UiElementState), With<DivContainer>>) {
    for (entity, mut state) in query.iter_mut() {
        if event.target.eq(&entity) {
            state.selected = !state.selected;
        }
    }
}

fn on_internal_mouse_entered(event: Trigger<Pointer<Over>>, mut query: Query<(Entity, &mut UiElementState), With<DivContainer>>) {
    for (entity, mut state) in query.iter_mut() {
        if event.target.eq(&entity) {
            state.hovered = true;
        }
    }
}

fn on_internal_mouse_leave(event: Trigger<Pointer<Out>>, mut query: Query<(Entity, &mut UiElementState), With<DivRoot>>) {
    for (entity, mut state) in query.iter_mut() {
        if event.target.eq(&entity) {
            state.hovered = false;
        }
    }
}