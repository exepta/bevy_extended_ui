use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{ UiGenID, UiElementState };
use crate::resources::ExtendedUiConfiguration;
use crate::styles::{BaseStyle, HoverStyle, SelectedStyle, Style};

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
            BaseStyle(Style {
                width: Val::Px(100.),
                height: Val::Px(100.),
                border: UiRect::all(Val::Px(5.)),
                border_color: Color::srgba(0.0, 0.0, 1.0, 1.0),
                ..default()
            }),

            HoverStyle::default(),

            SelectedStyle::default(),

            RenderLayers::layer(*layer),
            DivRoot
        ));
    }
}