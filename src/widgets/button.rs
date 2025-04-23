use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{UiGenID, UiElementState};
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::css_types::IconPlace;
use crate::styles::types::ButtonStyle;
use crate::styles::utils::{apply_base_component_style, apply_design_styles};
use crate::widgets::Button;

#[derive(Component)]
struct ButtonBase;

#[derive(Component)]
struct ButtonLabel;

#[derive(Component)]
struct ButtonImage;

pub struct ButtonWidget;

impl Plugin for ButtonWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            internal_generate_component_system,
            internal_style_update_que
                .after(internal_generate_component_system),
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &Button, &ButtonStyle), (Without<ButtonBase>, With<Button>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity , gen_id, btn, style) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Button-{}", gen_id.0)),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::all(Val::Px(0.)),
            BoxShadow::default(),
            RenderLayers::layer(*layer),
            ButtonBase
        )).with_children(|builder| {
            if style.icon_place == IconPlace::Left {
                place_icon(builder, style, &asset_server, gen_id.0, *layer);
            }

            builder.spawn((
                Name::new(format!("Button-Label-{}", gen_id.0)),
                Text::new(btn.0.clone()),
                RenderLayers::layer(*layer),
                PickingBehavior::IGNORE,
                ButtonLabel,
            ));

            if style.icon_place == IconPlace::Right {
                place_icon(builder, style, &asset_server, gen_id.0, *layer);
            }
        })
            .observe(on_internal_mouse_click)
            .observe(on_internal_mouse_entered)
            .observe(on_internal_mouse_leave);
    }
}

fn on_internal_mouse_click(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UiElementState, &UiGenID), With<Button>>,
    mut current_element_selected: ResMut<CurrentElementSelected>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(event.target) {
        state.selected = true;
        current_element_selected.0 = gen_id.0;
    }
}

fn on_internal_mouse_entered(event: Trigger<Pointer<Over>>, mut query: Query<&mut UiElementState, With<Button>>) {
    if let Ok(mut state) = query.get_mut(event.target) {
        state.hovered = true;
    }
}

fn on_internal_mouse_leave(event: Trigger<Pointer<Out>>, mut query: Query<&mut UiElementState, With<Button>>) {
    if let Ok(mut state) = query.get_mut(event.target) {
        state.hovered = false;
    }
}

fn place_icon(builder: &mut ChildBuilder, style: &ButtonStyle, asset_server: &Res<AssetServer>, id: usize, layer: usize) {
    if let Some(icon) = style.icon_path.clone() {
        builder.spawn((
            Name::new(format!("Button-Icon-{}", id)),
            ImageNode::new(asset_server.load(icon.as_str())),
            RenderLayers::layer(layer),
            PickingBehavior::IGNORE,
            ButtonImage,
            ZIndex(1)
        ));
    }
}

fn internal_style_update_que(
    mut query: Query<(&UiElementState, &UiGenID, &Children, &ButtonStyle,
                      &mut Node,
                      &mut BackgroundColor,
                      &mut BoxShadow,
                      &mut BorderRadius,
                      &mut BorderColor
    ), With<Button>>
) {
    for (state, ui_id, children, style,
        mut node,
        mut background_color,
        mut box_shadow,
        mut border_radius,
        mut border_color) in query.iter_mut() {

        let mut internal_style = style.style.clone();
        if state.hovered {
            internal_style.background.color = Color::srgb_u8(74, 207, 108);
        }

        apply_base_component_style(&internal_style, &mut node);
        apply_design_styles(&internal_style, &mut background_color, &mut border_color, &mut border_radius, &mut box_shadow);
    }
}