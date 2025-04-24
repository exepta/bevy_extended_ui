use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{BindToID, UiElementState, UiGenID};
use crate::resources::ExtendedUiConfiguration;
use crate::styles::state_styles::{Checked, Disabled, Hover, Selected, Styling};
use crate::styles::types::CheckBoxStyle;
use crate::styles::utils::{apply_base_component_style, apply_design_styles, apply_label_styles_to_child, resolve_style_by_state};
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
        app.add_systems(Update, (
            internal_generate_component_system,
            internal_style_update_que
                .after(internal_generate_component_system)
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &CheckBox), (Without<CheckBoxRoot>, With<CheckBox>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, gen_id, checkbox) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("CheckBox-{}", gen_id.0)),
            Node::default(),
            BorderRadius::default(),
            BorderColor::default(),
            BackgroundColor::default(),
            BoxShadow::default(),
            Checked(Styling::CheckBox(CheckBoxStyle {
                check_border_color: Color::srgb_u8(143, 201,  249),
                check_background: Color::srgb_u8(143, 201,  249),
                ..default()
            })),
            RenderLayers::layer(*layer),
            CheckBoxRoot,
        ))
            .observe(on_internal_click)
            .with_children(|builder| {
            builder.spawn((
                Name::new(format!("Check-Mark-{}", gen_id.0)),
                Node {
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                RenderLayers::layer(*layer),
                BorderRadius::default(),
                BorderColor::default(),
                BackgroundColor::default(),
                BoxShadow::default(),
                CheckBoxMark,
                PickingBehavior::IGNORE,
                BindToID(gen_id.0)
            ));

            builder.spawn((
                Name::new(format!("Check-Label-{}", gen_id.0)),
                Text::new(checkbox.label.clone()),
                TextFont::default(),
                TextColor::default(),
                TextLayout::default(),
                RenderLayers::layer(*layer),
                CheckBoxLabel,
                PickingBehavior::IGNORE,
                BindToID(gen_id.0),
            ));
        });
    }
}

fn on_internal_click(
    event: Trigger<Pointer<Click>>, 
    mut query: Query<(Entity, &mut CheckBox, &CheckBoxStyle, &UiGenID), With<CheckBox>>,
    inner_query: Query<(Entity, &BindToID, Option<&Children>), With<CheckBoxMark>>,
    mut commands: Commands,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
) {
    let target = event.target;
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, mut checkbox, style, ui_id) in query.iter_mut() {
        if target.eq(&entity) {
            checkbox.checked = !checkbox.checked;

            for (inner_entity, bind_to, children) in inner_query.iter() {
                if ui_id.0 != bind_to.0 {
                    continue;
                }

                if checkbox.checked {
                    let mut child = None;
                    commands.entity(inner_entity).with_children(|builder| {
                        let in_child = builder.spawn((
                            Name::new(format!("Mark-{}", ui_id.0)),
                            Node {
                                width: Val::Px(style.check_size / 1.5),
                                height: Val::Px(style.check_size / 1.5),
                                ..default()
                            },
                            PickingBehavior::IGNORE,
                            RenderLayers::layer(*layer),
                        )).id();

                        child = Some(in_child);
                    });

                    if let Some(child) = child {
                        if let Some(path) = style.icon_path.clone() {
                            commands.entity(child).insert(ImageNode::new(asset_server.load(path)).with_color(style.check_color));
                        } else {
                            commands.entity(child).insert(BackgroundColor(style.check_color));
                        }
                    }

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

fn internal_style_update_que(
    mut query: Query<(&UiElementState, &UiGenID, &Children, &CheckBox, &CheckBoxStyle, 
                      Option<&Hover>, Option<&Checked>, Option<&Selected>, Option<&Disabled>,
                      &mut Node,
                      &mut BackgroundColor,
                      &mut BoxShadow,
                      &mut BorderRadius,
                      &mut BorderColor
    ), With<CheckBox>>,
    mut label_query: Query<(&BindToID, &mut TextColor, &mut TextFont, &mut TextLayout)>,
    mut mark_query: Query<(&BindToID,
                           &mut Node,
                           &mut BackgroundColor,
                           &mut BoxShadow,
                           &mut BorderRadius,
                           &mut BorderColor),
        Without<CheckBox>>,
) {
    for (state, ui_id, children, check_box, style, 
        hover_style, checked_style, selected_style, disabled_style,
        mut node,
        mut background_color,
        mut box_shadow,
        mut border_radius,
        mut border_color) in query.iter_mut() {
        let mut internal_style = resolve_style_by_state(
            &Styling::CheckBox(style.clone()),
            state,
            hover_style,
            selected_style,
            disabled_style,
        );
        
        if check_box.checked {
            if let Some(checked) = checked_style {
                internal_style = checked.0.clone();
            }
        }

        if let Styling::CheckBox(check_box_style) = internal_style {
            apply_base_component_style(&check_box_style.style, &mut node);
            apply_design_styles(&check_box_style.style, &mut background_color, &mut border_color, &mut border_radius, &mut box_shadow);

            for child in children.iter() {
                apply_label_styles_to_child(*child, ui_id, &check_box_style.label_style, &mut label_query);

                if let Ok((bind_to, mut node, mut check_background_color,
                              mut check_box_shadow, mut check_border_radius,
                              mut check_border_color)) = mark_query.get_mut(*child) {
                    if bind_to.0 != ui_id.0 {
                        continue;
                    }

                    node.width = Val::Px(check_box_style.check_size);
                    node.height = Val::Px(check_box_style.check_size);
                    node.border = check_box_style.check_border;
                    if let Some(apply_box_shadow) = check_box_style.check_box_shadow {
                        check_box_shadow.color = apply_box_shadow.color;
                        check_box_shadow.blur_radius = apply_box_shadow.blur_radius;
                        check_box_shadow.spread_radius = apply_box_shadow.spread_radius;
                        check_box_shadow.x_offset = apply_box_shadow.x_offset;
                        check_box_shadow.y_offset = apply_box_shadow.y_offset;
                    } else {
                        check_box_shadow.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                        check_box_shadow.blur_radius = Val::Px(0.);
                        check_box_shadow.spread_radius = Val::Px(0.);
                    }
                    check_border_radius.top_left = check_box_style.check_border_radius.top_left;
                    check_border_radius.top_right = check_box_style.check_border_radius.top_right;
                    check_border_radius.bottom_left = check_box_style.check_border_radius.bottom_left;
                    check_border_radius.bottom_right = check_box_style.check_border_radius.bottom_right;
                    check_border_color.0 = check_box_style.check_border_color;
                    check_background_color.0 = check_box_style.check_background;
                }
            }
        }
    }
}
