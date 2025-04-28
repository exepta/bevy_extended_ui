use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{BindToID, UiElementState, UiGenID};
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::css_types::Colored;
use crate::styles::state_styles::{Disabled, Hover, Selected, Styling};
use crate::styles::types::ChoiceBoxStyle;
use crate::styles::utils::{apply_base_component_style, apply_design_styles, resolve_style_by_state};
use crate::widgets::ChoiceBox;

#[derive(Component)]
struct ChoiceRoot;

#[derive(Component)]
struct ChoiceOptionRoot;

#[derive(Component)]
struct SelectedOptionRoot;

#[derive(Component)]
struct ChoiceLayoutBoxRoot;

pub struct ChoiceBoxWidget;

impl Plugin for ChoiceBoxWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            internal_generate_component_system,
            internal_style_update_que
                .after(internal_generate_component_system),
            handle_scroll_events
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &ChoiceBox, &ChoiceBoxStyle), (Without<ChoiceRoot>, With<ChoiceBox>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, gen_id, choice_box, style) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("ChoiceBox-{}", gen_id.0)),
            Node::default(),
            BorderRadius::default(),
            BorderColor::default(),
            BoxShadow::default(),
            BackgroundColor::default(),
            RenderLayers::layer(*layer),
            ChoiceRoot
        ))
            .observe(on_click_main_root)
            .with_children(|builder| {

                // Choice Option Fields
                generate_child_selected_option(builder, &style, &choice_box, gen_id.0, &asset_server);

                // Option Layout Content
                builder.spawn((
                    Name::new(format!("Layout-Option-Root-{}", gen_id.0)),
                    Node::default(),
                    BackgroundColor::default(),
                    BorderRadius::default(),
                    BorderColor::default(),
                    BoxShadow::default(),
                    RenderLayers::layer(*layer),
                    Visibility::Hidden,
                    ChoiceLayoutBoxRoot,
                    BindToID(gen_id.0)
                )).with_children(|builder| {

                    for option in choice_box.options.iter() {
                        builder.spawn((
                            Name::new(format!("Option-{}", gen_id.0)),
                            Node {
                                width: Val::Percent(100.),
                                height: Val::Px(50.),
                                display: Display::Flex,
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(10.),
                                flex_grow: 1.0,
                                ..default()
                            },
                            BackgroundColor::default(),
                            RenderLayers::layer(*layer),
                            option.clone(),
                            ChoiceOptionRoot,
                            BindToID(gen_id.0)
                        )).with_children(|builder| {
                            if let Some(icon) = option.icon_path.clone() {
                                builder.spawn((
                                    Name::new(format!("Option-Icon-{}", gen_id.0)),
                                    ImageNode {
                                        image: asset_server.load(icon),
                                        ..default()
                                    },
                                    RenderLayers::layer(*layer),
                                    PickingBehavior::IGNORE,
                                    BindToID(gen_id.0)
                                ));
                            }

                            builder.spawn((
                                Name::new(format!("Option-Text-{}", gen_id.0)),
                                Text::new(option.label.clone()),
                                TextColor::default(),
                                TextFont::default(),
                                RenderLayers::layer(*layer),
                                PickingBehavior::IGNORE,
                                BindToID(gen_id.0)
                            ));
                        });
                    }

                });

            });
    }
}

fn handle_scroll_events(
    mut scroll_events: EventReader<MouseWheel>,
    mut layout_query: Query<(Entity, &Visibility, &Children), With<ChoiceLayoutBoxRoot>>,
    mut option_query: Query<(&mut Node, &Parent), With<ChoiceOptionRoot>>,
    time: Res<Time>,
) {
    let mut max_scroll = -0.0;
    let min_scroll = 0.0;

    let smooth_factor = 20.;

    for event in scroll_events.read() {
        for (layout_entity, visibility, children) in layout_query.iter_mut() {
            if *visibility != Visibility::Visible {
                continue;
            }

            if children.len() > 3 {
                max_scroll = -50.0 * (children.len() - 3) as f32;
            }

            let scroll_amount = match event.unit {
                MouseScrollUnit::Line => event.y * 25.0,
                MouseScrollUnit::Pixel => event.y,
            };

            let inverted_scroll_amount = scroll_amount;

            for (mut style, parent) in option_query.iter_mut() {
                if parent.get() != layout_entity {
                    continue;
                }

                let current_offset = match style.top {
                    Val::Px(val) => val,
                    _ => 0.0,
                };

                let target_offset = (current_offset + inverted_scroll_amount)
                    .clamp(max_scroll, min_scroll);

                let smoothed_offset = current_offset + (target_offset - current_offset) * smooth_factor * time.delta_secs();

                style.top = Val::Px(smoothed_offset);
            }
        }
    }
}

fn on_click_main_root(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(&mut Visibility, &Parent), With<ChoiceLayoutBoxRoot>>,
    mut check_query: Query<(&mut UiElementState, &UiGenID), With<ChoiceBox>>,
    mut current_element_selected: ResMut<CurrentElementSelected>
) {
    let target = event.target;

    if let Ok((mut state, gen_id)) = check_query.get_mut(event.target) {
        state.selected = true;
        current_element_selected.0 = gen_id.0;
    }
    
    for (mut visibility, parent) in query.iter_mut() {
        if target.eq(&parent.get()) {
            if *visibility == Visibility::Hidden {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}



fn generate_child_selected_option(builder: &mut ChildBuilder, style: &ChoiceBoxStyle,  choice_box: &ChoiceBox, id: usize, asset_server: &AssetServer) {
    builder.spawn((                                    
        Name::new(format!("Selected-Option-{}", id)),
        Node {
            flex_grow: 1.0,
            display: Display::Flex,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.),
            align_items: AlignItems::Center,
            padding: UiRect::left(Val::Px(15.)),
            ..default()
        },
        BorderRadius {
            top_right: Val::Px(0.),
            bottom_right: Val::Px(0.),
            top_left: style.style.border_radius.top_left,
            bottom_left: style.style.border_radius.bottom_left,
        },
        BackgroundColor(style.style.background.color),
        RenderLayers::layer(1),
        PickingBehavior::IGNORE,
        SelectedOptionRoot
    )).with_children(|builder| {

        if let Some(icon) = choice_box.value.icon_path.clone() {
            builder.spawn((
                ImageNode {
                    image: asset_server.load(icon),
                    ..default()
                },
                RenderLayers::layer(1),
                PickingBehavior::IGNORE,
            ));
        }

        builder.spawn((
            Text::new(choice_box.value.label.clone()),
            TextColor(Color::BLACK),
            TextFont {
                font_size: 15.,
                ..default()
            },
            RenderLayers::layer(1),
            PickingBehavior::IGNORE,
        ));

    });

    // Icon for drop down
    builder.spawn((
        Node {
            width: Val::Px(50.),
            min_width: Val::Px(25.),
            max_width: Val::Px(50.),
            height: Val::Percent(100.),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Colored::TRANSPARENT),
        BorderRadius {
            top_right: style.style.border_radius.top_right,
            bottom_right: style.style.border_radius.bottom_right,
            top_left: Val::Px(0.),
            bottom_left: Val::Px(0.),
        },
        RenderLayers::layer(1),
        PickingBehavior::IGNORE,
        SelectedOptionRoot
    )).with_children(|builder| {
        if let Some(icon) = choice_box.icon.clone() {
            builder.spawn((
                ImageNode {
                    image: asset_server.load(icon),
                    ..default()
                },
                RenderLayers::layer(1),
                PickingBehavior::IGNORE
            ));
        }
    });
}

fn internal_style_update_que(
    mut query: Query<(&UiElementState, &UiGenID, &ChoiceBoxStyle, Option<&Hover>, Option<&Selected>, Option<&Disabled>,
                      &mut Node,
                      &mut BackgroundColor,
                      &mut BoxShadow,
                      &mut BorderRadius,
                      &mut BorderColor
    ), (With<ChoiceBox>, Without<ChoiceLayoutBoxRoot>, Without<ChoiceOptionRoot>)>,
    mut layout_query: Query<(&BindToID,
                             &mut Node,
                             &mut BackgroundColor,
                             &mut BoxShadow,
                             &mut BorderRadius,
                             &mut BorderColor, 
                             &Children
    ), (With<ChoiceLayoutBoxRoot>, Without<ChoiceBox>, Without<ChoiceOptionRoot>)>,
    mut option_con_query: Query<(&BindToID, &mut BackgroundColor, &Children), (With<ChoiceOptionRoot>, Without<ChoiceLayoutBoxRoot>, Without<ChoiceBox>)>,
    mut option_label_text: Query<(&BindToID, &mut TextColor, &mut TextFont), (Without<ChoiceOptionRoot>, Without<ChoiceLayoutBoxRoot>)>,
) {
    for (state, ui_id, style, hover_style, selected_style, disabled_style,
        mut node,
        mut background_color,
        mut box_shadow,
        mut border_radius,
        mut border_color) in query.iter_mut() {
        let internal_style = resolve_style_by_state(
            &Styling::ChoiceBox(style.clone()),
            state,
            hover_style,
            selected_style,
            disabled_style,
        );

        if let Styling::ChoiceBox(choice_box_style) = internal_style {
            apply_base_component_style(&choice_box_style.style, &mut node);
            apply_design_styles(&choice_box_style.style, &mut background_color, &mut border_color, &mut border_radius, &mut box_shadow);
            
            for (bind_to, mut layout_node, mut layout_background_color, 
                mut layout_box_shadow, mut layout_border_radius, mut layout_border_color, children) 
            in layout_query.iter_mut() {
                if bind_to.0 != ui_id.0 {
                    continue;
                }
                
                apply_base_component_style(&choice_box_style.layout, &mut layout_node);
                apply_design_styles(&choice_box_style.layout, &mut layout_background_color, &mut layout_border_color
                                    , &mut layout_border_radius, &mut layout_box_shadow);
                
                for child in children.iter() {
                    if let Ok((bind_to, mut option_background, inner_children)) = option_con_query.get_mut(*child) {
                        if bind_to.0 != ui_id.0 {
                            continue;
                        }

                        if let Some(background) = choice_box_style.option_style.background_color {
                            option_background.0 = background;
                        } else {
                            option_background.0 = choice_box_style.layout.background.color;
                        }
                        
                        for inner_child in inner_children.iter() {
                            if let Ok((bind_to, mut text_color, mut text_font)) = option_label_text.get_mut(*inner_child) {
                                if bind_to.0 != ui_id.0 {
                                    continue;
                                }
                                
                                text_color.0 = choice_box_style.label_style.color;
                                text_font.font_size = choice_box_style.label_style.font_size;
                                text_font.font_smoothing = choice_box_style.label_style.smoothing;
                            }
                        }
                    }
                }
            }
        }
    }
}
