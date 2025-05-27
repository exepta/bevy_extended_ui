use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::widgets::Slider;

#[derive(Component)]
struct SliderBase;

#[derive(Component)]
struct SliderTrack;

#[derive(Component, Reflect, Debug, Clone)]
struct SliderThumb {
    current_x: f32,
}

#[derive(Component, Deref, DerefMut)]
struct PreviousSliderValue(i32);

pub struct SliderWidget;

impl Plugin for SliderWidget {
    fn build(&self, app: &mut App) {
        app.register_type::<SliderThumb>();
        app.add_systems(Update, (
            internal_node_creation_system,
            detect_change_slider_values
        ));
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &Slider, Option<&CssSource>), (With<Slider>, Without<SliderBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, slider, source_opt) in query.iter() {
        let mut css_source = CssSource(String::from("assets/css/core.css"));
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("Slider-{}", slider.w_count)),
            Node {
                width: Val::Px(150.),
                height: Val::Px(8.),
                ..default()
            },
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            css_source.clone(),
            PreviousSliderValue(slider.value),
            TagName(String::from("slider")),
            RenderLayers::layer(*layer),
            SliderBase,
        )).observe(on_click_track)
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {

                // Slider Track child
                builder.spawn((
                    Name::new(format!("Slider-Track-{}", slider.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    RenderLayers::layer(*layer),
                    css_source.clone(),
                    CssClass(vec!["track".to_string()]),
                    Pickable::IGNORE,
                    SliderTrack,
                    BindToID(id.0)
                ));

                // Slider Thumb child
                builder.spawn((
                    Name::new(format!("Slider-Thumb-{}", slider.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    UIWidgetState::default(),
                    BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                    ZIndex::default(),
                    RenderLayers::layer(*layer),
                    css_source.clone(),
                    CssClass(vec!["thumb".to_string()]),
                    SliderThumb { current_x: 0.0 },
                    BindToID(id.0)
                )).observe(on_move_thumb);
            });
    }
}

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Slider>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}

fn on_move_thumb(
    event: Trigger<Pointer<Drag>>,
    mut query: Query<(&mut Slider, &ComputedNode, &Children), With<Slider>>,
    mut track_query: Query<(&mut Node, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &mut WidgetStyle), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = match window_query.single() {
        Ok(window) => window,
        Err(_) => return
    };
    for (mut slider, computed_node, children) in query.iter_mut() {
        let slider_width = computed_node.size().x / window.scale_factor();
        for child in children.iter() {
            if event.target.eq(&child) {
                let next_child = children.iter().next();
                if let Ok((mut thumb_node, mut slider_thumb, mut style)) = thumb_query.get_mut(child) {
                    slider_thumb.current_x += event.event.delta.x;
                    slider_thumb.current_x = slider_thumb.current_x.clamp(0.0, slider_width);

                    thumb_node.left = Val::Px(slider_thumb.current_x - 8.0);

                    for (_, styles) in style.styles.iter_mut() {
                        styles.left = Some(thumb_node.left);
                    }

                    if let Some(track_child) = next_child {
                        if let Ok((mut track_node, mut track_style)) = track_query.get_mut(track_child) {
                            track_node.width = Val::Px(slider_thumb.current_x);

                            for (_, styles) in track_style.styles.iter_mut() {
                                styles.width = Some(track_node.width);
                            }
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
    mut query: Query<(Entity, &mut Slider, &ComputedNode, &UIGenID, &Children), With<Slider>>,
    mut track_query: Query<(&mut Node, &BindToID, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut WidgetStyle), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = match window_query.single() {
        Ok(window) => window,
        Err(_) => return
    };
    for (entity, mut slider, computed_node, ui_id, children) in query.iter_mut() {
        if event.target.eq(&entity) {
            let slider_width = computed_node.size().x / window.scale_factor();
            let track_left = (window.width() - slider_width) / 2.0;
            let click_x = event.pointer_location.position.x - track_left;
            let clamped_x = click_x.clamp(0.0, slider_width);

            for child in children.iter() {
                let next_child = children.iter().next();
                if let Some(track_child) = next_child {
                    if let Ok((mut thumb_node, mut slider_thumb, bind_to_thumb, mut style)) = thumb_query.get_mut(child) {
                        if bind_to_thumb.0 != ui_id.0 {
                            continue;
                        }
                        slider_thumb.current_x = clamped_x;
                        thumb_node.left = Val::Px(clamped_x - 8.0);

                        for (_, styles) in style.styles.iter_mut() {
                            styles.left = Some(thumb_node.left);
                        }

                        update_slider_track_width(&mut track_query, &track_child, &ui_id, clamped_x);

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

fn detect_change_slider_values(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        Entity,
        &mut Slider,
        &UIWidgetState,
        &ComputedNode,
        &Children,
        &UIGenID,
        &mut PreviousSliderValue
    ), With<Slider>>,
    mut track_query: Query<(&mut Node, &BindToID, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut WidgetStyle), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = match window_query.single() {
        Ok(window) => window,
        Err(_) => return,
    };

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    for (_entity, mut slider, state, computed_node, children, ui_id, mut prev) in query.iter_mut() {
        let mut changed = false;

        // Handle keyboard change
        if state.focused {
            let step = if shift { slider.step * 10 } else { slider.step };
            if keyboard.just_pressed(KeyCode::ArrowRight) {
                slider.value = (slider.value + step).min(slider.max);
            }
            if keyboard.just_pressed(KeyCode::ArrowLeft) {
                slider.value = (slider.value - step).max(slider.min);
            }
        }

        if slider.value != **prev {
            changed = true;
            **prev = slider.value; // update stored value
        }

        if changed {
            let slider_width = computed_node.size().x / window.scale_factor();
            let percent = (slider.value - slider.min) as f32 / (slider.max - slider.min) as f32;
            let clamped_x = percent * slider_width;

            for child in children.iter() {
                if let Ok((mut thumb_node, mut thumb, bind_to_thumb, mut style)) = thumb_query.get_mut(child) {
                    if bind_to_thumb.0 != ui_id.0 {
                        continue;
                    }
                    thumb.current_x = clamped_x;
                    thumb_node.left = Val::Px(clamped_x - 8.0);
                    for (_, styles) in style.styles.iter_mut() {
                        styles.left = Some(thumb_node.left);
                    }
                }
                update_slider_track_width(&mut track_query, &child, ui_id, clamped_x);
            }
        }
    }
}


fn update_slider_track_width(
    track_query: &mut Query<(&mut Node, &BindToID, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    track_child: &Entity,
    ui_id: &UIGenID,
    clamped_x: f32,
) {
    if let Ok((mut track_node, bind_to, mut style)) = track_query.get_mut(*track_child) {
        if bind_to.0 != ui_id.0 {
            return;
        }
        track_node.width = Val::Px(clamped_x);
        for (_, styles) in style.styles.iter_mut() {
            styles.width = Some(track_node.width);
        }
    }
}
