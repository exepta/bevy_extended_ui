use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{UiGenID, UiElementState};
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::{BaseStyle, HoverStyle, SelectedStyle, Style};
use crate::styles::css_types::Background;
use crate::utils::Radius;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, BaseStyle, HoverStyle, SelectedStyle)]
pub struct Button {
    pub label: String,
    pub icon: Option<Handle<Image>>,
    pub icon_place: IconPlace
}

impl Default for Button {
    fn default() -> Self {
        Self {
            label: String::from("Button"),
            icon: None,
            icon_place: IconPlace::Right,
        }
    }
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum IconPlace {
    Left,
    Right
}

#[derive(Component)]
struct ButtonBase;

#[derive(Component)]
struct ButtonLabel;

#[derive(Component)]
struct ButtonImage;

pub struct ButtonWidget;

impl Plugin for ButtonWidget {
    fn build(&self, app: &mut App) {
        app.register_type::<Button>();
        app.add_systems(Update, internal_generate_component_system);
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &Button), (Without<ButtonBase>, With<Button>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity , gen_id, btn) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Button-{}", gen_id.0)),
            Node::default(),
            BaseStyle(Style {
                width: Val::Px(150.),
                height: Val::Px(50.),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                gap_row: Val::Px(15.),
                background: Background { color: Color::srgba(0.95, 0.95, 0.95, 1.0), ..default() },
                border: UiRect::all(Val::Px(2.)),
                border_radius: Radius::all(Val::Px(5.)),
                ..default()
            }),
            RenderLayers::layer(*layer),
            ButtonBase
        )).with_children(|builder| {
            if btn.icon_place == IconPlace::Left {
                place_icon(builder, btn, gen_id.0, *layer);
            }

            builder.spawn((
                Name::new(format!("Button-Label-{}", gen_id.0)),
                Text::new(btn.label.clone()),
                RenderLayers::layer(*layer),
                PickingBehavior::IGNORE,
                ButtonLabel,
            ));

            if btn.icon_place == IconPlace::Right {
                place_icon(builder, btn, gen_id.0, *layer);
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

fn place_icon(builder: &mut ChildBuilder, btn: &Button, id: usize, layer: usize) {
    if let Some(icon) = btn.icon.clone() {
        builder.spawn((
            Name::new(format!("Button-Icon-{}", id)),
            ImageNode::new(icon),
            RenderLayers::layer(layer),
            PickingBehavior::IGNORE,
            ButtonImage,
            ZIndex(1)
        ));
    }
}