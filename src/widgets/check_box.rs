use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{BindToID, UiGenID};
use crate::resources::ExtendedUiConfiguration;
use crate::widgets::{CheckBox};

#[derive(Component)]
struct CheckBoxRoot;

#[derive(Component)]
struct CheckBoxLabel;

#[derive(Component)]
pub struct CheckBoxMark;

pub struct CheckBoxWidget;

impl Plugin for CheckBoxWidget {
    fn build(&self, app: &mut App) {
/*        app.add_systems(Update, internal_generate_component_system);*/
    }
}

/*fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &CheckBox), (Without<CheckBoxRoot>, With<CheckBox>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, gen_id, checkbox) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("CheckBox-{}", gen_id.0)),
            Node::default(),
            RenderLayers::layer(*layer),
            CheckBoxRoot,
            CheckBoxStyle
        ))
            .observe(on_internal_click)
            .with_children(|builder| {
            builder.spawn((
                Name::new(format!("Check-Mark-{}", gen_id.0)),
                Node {
                    width: Val::Px(default_style.0.check_size),
                    height: Val::Px(default_style.0.check_size),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: default_style.0.check_border,
                    ..default()
                },
                RenderLayers::layer(*layer),
                BorderRadius {
                    top_left: default_style.0.check_border_radius.top_left,
                    top_right: default_style.0.check_border_radius.top_right,
                    bottom_left: default_style.0.check_border_radius.bottom_left,
                    bottom_right: default_style.0.check_border_radius.bottom_right,
                },
                BorderColor(default_style.0.check_border_color),
                BackgroundColor(if checkbox.checked { default_style.0.check_background_color } else { Color::srgba(0.0, 0.0, 0.0, 0.0) }),
                CheckBoxMark,
                PickingBehavior::IGNORE,
                BindToID(gen_id.0)
            ));

            builder.spawn((
                Name::new(format!("Check-Label-{}", gen_id.0)),
                Text::new(checkbox.label.clone()),
                TextFont {
                    font_size: default_style.0.font_size,
                    ..default()
                },
                TextColor(default_style.0.color),
                RenderLayers::layer(*layer),
                CheckBoxLabel,
                PickingBehavior::IGNORE,
                BindToID(gen_id.0),
            ));
        });
    }
}*/

fn on_internal_click(
    event: Trigger<Pointer<Click>>, 
    mut query: Query<(Entity, &mut CheckBox, &UiGenID), With<CheckBox>>,
    inner_query: Query<(Entity, &BindToID, Option<&Children>), With<CheckBoxMark>>,
    mut commands: Commands,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
) {
    let target = event.target;
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, mut checkbox, ui_id) in query.iter_mut() {
        if target.eq(&entity) {
            checkbox.checked = !checkbox.checked;

            for (inner_entity, bind_to, children) in inner_query.iter() {
                if ui_id.0 != bind_to.0 {
                    continue;
                }

                if checkbox.checked {
                    commands.entity(inner_entity).with_children(|builder| {
                        builder.spawn((
                            Name::new(format!("Mark-{}", ui_id.0)),
/*                            Node {
                                width: Val::Px(style.0.check_mark_size),
                                height: Val::Px(style.0.check_mark_size),
                                ..default()
                            },*/
/*                            ImageNode {
                                color: if let Some(icon_color) = style.0.icon_color { icon_color } else { style.0.color },
                                image: asset_server.load("icons/check-mark.png"),
                                ..default()
                            },*/
                            PickingBehavior::IGNORE,
                            RenderLayers::layer(*layer),
                        ));
                    });
                } else {
                    if let Some(children) = children {
                        for child in children.iter() {
                            commands.entity(*child).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}

