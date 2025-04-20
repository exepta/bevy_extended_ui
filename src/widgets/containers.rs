use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{UiGenID, UiElementState };
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::{BaseStyle, InternalStyle, Style};
use crate::widgets::DivContainer;

#[derive(Component)]
pub struct DivRoot;

pub struct DivWidget;

impl Plugin for DivWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_generate_component_system);
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, Option<&BaseStyle>), (Without<DivRoot>, With<DivContainer>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity , gen_id, option_base_style) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Div-{}", gen_id.0)),
            Node::default(),
            default_div_style(option_base_style),
            RenderLayers::layer(*layer),
            DivRoot
        ))
            .observe(on_internal_mouse_click)
            .observe(on_internal_mouse_entered)
            .observe(on_internal_mouse_leave);
    }
}

fn on_internal_mouse_click(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UiElementState, &UiGenID), With<DivContainer>>,
    mut current_element_selected: ResMut<CurrentElementSelected>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(event.target) {
        state.selected = true;
        current_element_selected.0 = gen_id.0;
    }
}

fn on_internal_mouse_entered(event: Trigger<Pointer<Over>>, mut query: Query<&mut UiElementState, With<DivContainer>>) {
    if let Ok(mut state) = query.get_mut(event.target) {
        state.hovered = true;
    }
}

fn on_internal_mouse_leave(event: Trigger<Pointer<Out>>, mut query: Query<&mut UiElementState, With<DivContainer>>) {
    if let Ok(mut state) = query.get_mut(event.target) {
        state.hovered = false;
    }
}

fn default_div_style(overwrite: Option<&BaseStyle>) -> InternalStyle {
    let mut internal_style = InternalStyle(Style {
        margin: UiRect::all(Val::Px(10.0)),
        ..default()
    });

    if let Some(style) = overwrite {
        internal_style.merge_styles(&style.0);
    }
    internal_style
}