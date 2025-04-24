use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{UiGenID, UiElementState };
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::types::DivStyle;
use crate::styles::utils::{apply_base_component_style, apply_design_styles};
use crate::widgets::DivContainer;

#[derive(Component)]
pub struct DivRoot;

pub struct DivWidget;

impl Plugin for DivWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            internal_generate_component_system, 
            internal_style_update_que
                .after(internal_generate_component_system)
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID), (Without<DivRoot>, With<DivContainer>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity , gen_id) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Div-{}", gen_id.0)),
            Node::default(),
            BoxShadow::default(),
            BorderRadius::default(),
            BorderColor::default(),
            BackgroundColor::default(),
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

fn internal_style_update_que(
    mut query: Query<(&UiElementState, &UiGenID, &Children, &DivStyle,
                      &mut Node,
                      &mut BackgroundColor,
                      &mut BoxShadow,
                      &mut BorderRadius,
                      &mut BorderColor
    ), With<DivContainer>>
) {
    for (state, ui_id, children, style,
        mut node,
        mut background_color,
        mut box_shadow,
        mut border_radius,
        mut border_color) in query.iter_mut() {

        let internal_style = style.style.clone();

        apply_base_component_style(&internal_style, &mut node);
        apply_design_styles(&internal_style, &mut background_color, &mut border_color, &mut border_radius, &mut box_shadow);
    }
}