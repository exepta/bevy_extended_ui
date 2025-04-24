use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use crate::global::{UiGenID, UiElementState, BindToID};
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::state_styles::{Disabled, Hover, Selected, Styling};
use crate::styles::types::SliderStyle;
use crate::styles::utils::{apply_base_component_style, apply_design_styles, resolve_style_by_state};
use crate::widgets::Slider;

#[derive(Component)]
struct SliderRoot;

#[derive(Component)]
struct SliderTrack;

#[derive(Component, Reflect, Debug, Clone)]
struct SliderThumb {
    current_x: f32,
}

pub struct SliderWidget;

impl Plugin for SliderWidget {
    fn build(&self, app: &mut App) {
        app.register_type::<SliderThumb>();
        app.add_systems(Update, (
            internal_generate_component_system,
            internal_style_update_que
                .after(internal_generate_component_system),
            detect_change_slider_values
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID), (Without<SliderRoot>, With<Slider>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, gen_id) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Slider-{}", gen_id.0)),
            Node::default(),
            BoxShadow::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            RenderLayers::layer(*layer),
            SliderRoot
        ))
            .observe(on_click_track)
            .observe(on_internal_mouse_click)
            .with_children(|builder| {
            // Slider Track
            builder.spawn((
                Name::new(format!("Slider-Track-{}", gen_id.0)),
                Node {
                    width: Val::Px(0.),
                    height: Val::Percent(100.),
                    ..default()
                },
                BorderRadius::default(),
                BackgroundColor::default(),
                BorderColor::default(),
                RenderLayers::layer(*layer),
                PickingBehavior::IGNORE,
                SliderTrack,
                BindToID(gen_id.0)
            ));

            // Slider Thumb
            builder.spawn((
                Name::new(format!("Slider-Thumb-{}", gen_id.0)),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BorderRadius::default(),
                BorderColor::default(),
                BackgroundColor::default(),
                BoxShadow::default(),
                RenderLayers::layer(*layer),
                SliderThumb { current_x: 0. },
                BindToID(gen_id.0)
            )).observe(on_move_thumb);

        });
    }
}

fn detect_change_slider_values(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Slider, &UiElementState, &ComputedNode, &Children, &UiGenID), With<Slider>>,
    mut track_query: Query<(&mut Node, &BindToID), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID), (With<SliderThumb>, Without<SliderTrack>)>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    for (mut slider, state, computed_node, children, ui_id) in query.iter_mut() {
        // Skip unfocused sliders
        if !state.selected {
            continue;
        }
        
        let old_value = slider.value;

        // Value change via arrow keys
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            slider.value = (slider.value + if shift {slider.step * 10} else {slider.step}).min(slider.max);
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            slider.value = (slider.value - if shift {slider.step * 10} else {slider.step}).max(slider.min);
        }

        if slider.value != old_value {
            let slider_width = computed_node.size().x / 1.5;
            let percent = (slider.value - slider.min) as f32 / (slider.max - slider.min) as f32;
            let clamped_x = percent * slider_width;

            for child in children.iter() {
                if let Ok((mut thumb_node, mut thumb, bind_to_thumb)) = thumb_query.get_mut(*child) {
                    if bind_to_thumb.0 != ui_id.0 {
                        continue;
                    }
                    thumb.current_x = clamped_x;
                    thumb_node.left = Val::Px(clamped_x - 8.0);
                }
                update_slider_track_width(&mut track_query, child, &ui_id, clamped_x);
            }
        }
    }
}

fn on_move_thumb(
    event: Trigger<Pointer<Drag>>,
    mut query: Query<(&mut Slider, &ComputedNode, &Children), With<Slider>>,
    mut track_query: Query<&mut Node, (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb), (With<SliderThumb>, Without<SliderTrack>)>,
) {
    for (mut slider, computed_node, children) in query.iter_mut() {
        let slider_width = computed_node.size().x / 1.5;
        for child in children.iter() {
            if event.target.eq(child) {
                let next_child = children.iter().next();
                if let Ok((mut thumb_node, mut slider_thumb)) = thumb_query.get_mut(*child) {
                    slider_thumb.current_x += event.event.delta.x;
                    slider_thumb.current_x = slider_thumb.current_x.clamp(0.0, slider_width);

                    thumb_node.left = Val::Px(slider_thumb.current_x - 8.0);

                    if let Some(track_child) = next_child {
                        if let Ok(mut track_node) = track_query.get_mut(*track_child) {
                            track_node.width = Val::Px(slider_thumb.current_x);
                        }
                    }

                    let percent = slider_thumb.current_x / slider_width;
                    let range = (slider.max - slider.min) as f32;
                    let raw_value = slider.min as f32 + percent * range;
                    let stepped_value = ((raw_value / slider.step as f32).round() * slider.step as f32) as i32;
                    slider.value = stepped_value;
                }
            }
        }
    }
}

fn on_click_track(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(Entity, &mut Slider, &ComputedNode, &UiGenID, &Children), With<Slider>>,
    mut track_query: Query<(&mut Node, &BindToID), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    for (entity, mut slider, computed_node, ui_id, children) in query.iter_mut() {
        if event.target.eq(&entity) {
            let slider_width = computed_node.size().x / 1.5;
            let track_left = (window.width() - slider_width) / 2.0;
            let click_x = event.pointer_location.position.x - track_left;
            let clamped_x = click_x.clamp(0.0, slider_width);

            for child in children.iter() {
                let next_child = children.iter().next();
                if let Some(track_child) = next_child {
                    if let Ok((mut thumb_node, mut slider_thumb, bind_to_thumb)) = thumb_query.get_mut(*child) {
                        if bind_to_thumb.0 != ui_id.0 {
                            continue;
                        }
                        slider_thumb.current_x = clamped_x;
                        thumb_node.left = Val::Px(clamped_x - 8.0);

                        update_slider_track_width(&mut track_query, track_child, &ui_id, clamped_x);

                        let percent = slider_thumb.current_x / slider_width;
                        let range = (slider.max - slider.min) as f32;
                        let raw_value = slider.min as f32 + percent * range;
                        let stepped_value = ((raw_value / slider.step as f32).round() * slider.step as f32) as i32;
                        slider.value = stepped_value;
                    }
                }
            }
        }
    }
}

fn on_internal_mouse_click(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UiElementState, &UiGenID), With<Slider>>,
    mut current_element_selected: ResMut<CurrentElementSelected>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(event.target) {
        state.selected = true;
        current_element_selected.0 = gen_id.0;
    }
}

fn update_slider_track_width(
    track_query: &mut Query<(&mut Node, &BindToID), (With<SliderTrack>, Without<SliderThumb>)>,
    track_child: &Entity,
    ui_id: &UiGenID,
    clamped_x: f32,
) {
    if let Ok((mut track_node, bind_to)) = track_query.get_mut(*track_child) {
        if bind_to.0 != ui_id.0 {
            return;
        }
        track_node.width = Val::Px(clamped_x);
    }
}

fn internal_style_update_que(
    mut query: Query<(&UiElementState, &UiGenID, &Children, &SliderStyle, Option<&Hover>, Option<&Selected>, Option<&Disabled>,
                      &mut Node,
                      &mut BackgroundColor,
                      &mut BoxShadow,
                      &mut BorderRadius,
                      &mut BorderColor
    ), With<Slider>>,
    mut thumb_query: Query<(&BindToID, &mut Node, &mut BackgroundColor, &mut BorderRadius, &mut BorderColor, &mut BoxShadow), (With<SliderThumb>, Without<SliderTrack>, Without<Slider>)>,
    mut track_query: Query<(&BindToID, &mut Node, &mut BackgroundColor, &mut BorderRadius, &mut BorderColor), (With<SliderTrack>, Without<SliderThumb>, Without<Slider>)>,
) {
    for (state, ui_id, children, style, hover_style, selected_style, disabled_style,
        mut node,
        mut background_color,
        mut box_shadow,
        mut border_radius,
        mut border_color) in query.iter_mut() {
        let internal_style = resolve_style_by_state(
            &Styling::Slider(style.clone()),
            state,
            hover_style,
            selected_style,
            disabled_style,
        );

        if let Styling::Slider(slider_style) = internal_style {
            apply_base_component_style(&slider_style.style, &mut node);
            apply_design_styles(&slider_style.style, &mut background_color, &mut border_color, &mut border_radius, &mut box_shadow);

            for child in children.iter() {
                if let Ok((bind_to, mut track_node, 
                              mut track_background_color, mut track_border_radius, mut track_border_color)) 
                    = track_query.get_mut(*child) {
                    if bind_to.0 != ui_id.0 {
                        continue;
                    }
                    
                    track_node.border = slider_style.track_border;
                    track_border_radius.top_left = slider_style.track_border_radius.top_left;
                    track_border_radius.top_right = slider_style.track_border_radius.top_right;
                    track_border_radius.bottom_left = slider_style.track_border_radius.bottom_left;
                    track_border_radius.bottom_right = slider_style.track_border_radius.bottom_right;
                    track_border_color.0 = slider_style.track_border_color;
                    track_background_color.0 = slider_style.track_color;
                }

                if let Ok((bind_to, mut thumb_node,
                              mut thumb_background_color, mut thumb_border_radius, mut thumb_border_color, mut thumb_box_shadow))
                    = thumb_query.get_mut(*child) {
                    if bind_to.0 != ui_id.0 {
                        continue;
                    }

                    thumb_node.width = slider_style.thumb_width;
                    thumb_node.height = slider_style.thumb_height;
                    thumb_node.border = slider_style.thumb_border;
                    if let Some(box_shadow) = slider_style.thumb_box_shadow {
                        thumb_box_shadow.color = box_shadow.color;
                        thumb_box_shadow.blur_radius = box_shadow.blur_radius;
                        thumb_box_shadow.spread_radius = box_shadow.spread_radius;
                        thumb_box_shadow.x_offset = box_shadow.x_offset;
                        thumb_box_shadow.y_offset = box_shadow.y_offset;
                    }
                    thumb_border_radius.top_left = slider_style.thumb_border_radius.top_left;
                    thumb_border_radius.top_right = slider_style.thumb_border_radius.top_right;
                    thumb_border_radius.bottom_left = slider_style.thumb_border_radius.bottom_left;
                    thumb_border_radius.bottom_right = slider_style.thumb_border_radius.bottom_right;
                    thumb_border_color.0 = slider_style.thumb_border_color;
                    thumb_background_color.0 = slider_style.thumb_color;
                }
            }
        }
    }
}
