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

#[derive(Component)]
struct SliderNeedInit;

pub struct SliderWidget;

impl Plugin for SliderWidget {
    fn build(&self, app: &mut App) {
        app.register_type::<SliderThumb>();
        app.add_systems(Update, (
            internal_node_creation_system,
            detect_change_slider_values,
            initialize_slider_visual_state
        ).chain());
    }
}

/// Creates and initializes UI nodes for slider entities that have not yet been initialized.
///
/// This system queries all entities with a `Slider` component but without a `SliderBase` component.
/// For each such entity, it inserts necessary UI components including the main slider node,
/// background, border, box shadow, and associates CSS sources.
/// It also creates child nodes for the slider track and the slider thumb, each with their own components
/// and event observers.
///
/// # Parameters
/// - `commands`: ECS commands used to insert components and spawn children entities.
/// - `query`: Query fetching entities with `Slider` and `UIGenID` but without `SliderBase`, optionally with a `CssSource`.
/// - `config`: Shared UI configuration resource, providing render layer information.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &Slider, Option<&CssSource>), (With<Slider>, Without<SliderBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, slider, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
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
            ImageNode::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            css_source.clone(),
            PreviousSliderValue(0),
            TagName(String::from("slider")),
            RenderLayers::layer(*layer),
            SliderNeedInit,
            SliderBase,
        )).observe(on_click_track)
            .observe(on_drag_track)
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {

                // Slider Track child
                builder.spawn((
                    Name::new(format!("Slider-Track-{}", slider.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    ImageNode::default(),
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
                    ImageNode::default(),
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

/// Event handler for click events on slider entities.
///
/// Marks the clicked slider as focused and updates the global currently focused widget ID.
///
/// # Parameters
/// - `trigger`: The pointer click event trigger containing the target entity.
/// - `query`: Query to access and mutate UI widget state and generation ID for sliders.
/// - `current_widget_state`: Mutable resource tracking the currently focused widget.
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

/// Event handler for pointer cursor entering slider entities.
///
/// Sets the hovered state of the slider to true.
///
/// # Parameters
/// - `trigger`: The pointer over event trigger containing the target entity.
/// - `query`: Query to mutate UI widget state for sliders.
fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

/// Event handler for pointer cursor leaving slider entities.
///
/// Sets the hovered state of the slider to false.
///
/// # Parameters
/// - `trigger`: The pointer out event trigger containing the target entity.
/// - `query`: Query to mutate UI widget state for sliders.
fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}

/// Handles dragging of the slider thumb, updating its position and the slider value.
///
/// This system responds to `Pointer<Drag>` events targeting the slider thumb.
/// It calculates the new thumb position relative to the slider width,
/// clamps it within valid bounds, updates the thumb's UI node position,
/// adjusts the track width accordingly, and calculates the corresponding slider value
/// based on the slider's min, max, and step values.
///
/// # Parameters
/// - `event`: The drag event containing the pointer movement delta and target entity.
/// - `query`: Query for sliders along with their computed node and children entities.
/// - `track_query`: Query to mutate the slider track node and style.
/// - `thumb_query`: Query to mutate the slider thumb node, component, and style.
/// - `window_query`: Query to access the primary window for a scale factor.
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

/// Handles click events on the slider track to update the slider's value and UI.
///
/// This system listens for `Pointer<Click>` events on slider tracks.
/// When triggered, it delegates the logic to `handle_track_event` to update
/// the slider value and the UI accordingly.
///
/// # Parameters
/// - `event`: The click event containing pointer location and target entity.
/// - `query`: Query for sliders and their children.
/// - `track_query`: Query for slider track nodes and styles.
/// - `thumb_query`: Query for slider thumb nodes, components, and styles.
/// - `window_query`: Query for the primary window to get a scale factor.
fn on_click_track(
    event: Trigger<Pointer<Click>>,
    query: Query<(Entity, &mut Slider, &ComputedNode, &UIGenID, &Children), With<Slider>>,
    track_query: Query<(&mut Node, &BindToID, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut WidgetStyle), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = window_query.single() {
        handle_track_event(
            event.pointer_location.position,
            event.target,
            query,
            track_query,
            thumb_query,
            window,
        );
    }
}

/// Handles drag events on the slider track to update the slider's value and UI.
///
/// This system listens for `Pointer<Drag>` events on slider tracks.
/// When triggered, it delegates the logic to `handle_track_event` to update
/// the slider value and the UI accordingly.
///
/// # Parameters
/// - `event`: The drag event containing pointer location and target entity.
/// - `query`: Query for sliders and their children.
/// - `track_query`: Query for slider track nodes and styles.
/// - `thumb_query`: Query for slider thumb nodes, components, and styles.
/// - `window_query`: Query for the primary window to get a scale factor.
fn on_drag_track(
    event: Trigger<Pointer<Drag>>,
    query: Query<(Entity, &mut Slider, &ComputedNode, &UIGenID, &Children), With<Slider>>,
    track_query: Query<(&mut Node, &BindToID, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut WidgetStyle), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = window_query.single() {
        handle_track_event(
            event.pointer_location.position,
            event.target,
            query,
            track_query,
            thumb_query,
            window,
        );
    }
}

/// Internal helper that applies the pointer position to update the slider's thumb and track UI,
/// and calculates the new slider value.
///
/// This function finds the slider entity matching the given target,
/// then updates the thumb's position, track width, and slider value accordingly.
///
/// # Parameters
/// - `pointer_position`: The current pointer position in window coordinates.
/// - `target`: The entity which received the pointer event.
/// - `query`: Query for sliders, their computed nodes, UI IDs, and children.
/// - `track_query`: Mutable query for slider track nodes, binding IDs, and styles.
/// - `thumb_query`: Mutable query for slider thumb nodes, components, binding IDs, and styles.
/// - `window`: Reference to the primary window for scale factor and coordinate conversion.
fn handle_track_event(
    pointer_position: Vec2,
    target: Entity,
    mut query: Query<(Entity, &mut Slider, &ComputedNode, &UIGenID, &Children), With<Slider>>,
    mut track_query: Query<(&mut Node, &BindToID, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut WidgetStyle), (With<SliderThumb>, Without<SliderTrack>)>,
    window: &Window,
) {
    for (entity, mut slider, computed_node, ui_id, children) in query.iter_mut() {
        if target == entity {
            track_internal_logic(
                &pointer_position,
                &mut slider,
                computed_node,
                ui_id,
                children,
                &mut track_query,
                &mut thumb_query,
                window,
            );
        }
    }
}

/// Internal logic to update slider UI and value based on pointer position.
///
/// This function updates the slider thumb's horizontal position, the slider track's width,
/// and calculates the corresponding slider value based on the pointer position relative to
/// the slider's visual width. It also clamps values to valid ranges and respects the slider's
/// min, max, and step values.
///
/// # Arguments
/// - `pointer_position`: The pointer (mouse or touch) position in window coordinates.
/// - `slider`: Mutable reference to the slider component to update its value.
/// - `computed_node`: The computed layout node of the slider, used to get its size.
/// - `ui_id`: Unique UI identifier for binding slider parts.
/// - `children`: Child entities of the slider, typically including thumb and track entities.
/// - `track_query`: Query to access and mutate slider track nodes and styles.
/// - `thumb_query`: Query to access and mutate slider thumb nodes, components, and styles.
/// - `window`: Reference to the primary window for coordinate and scale factor calculations.
#[allow(clippy::too_many_arguments)]
fn track_internal_logic(
    pointer_position: &Vec2,
    slider: &mut Slider,
    computed_node: &ComputedNode,
    ui_id: &UIGenID,
    children: &Children,
    track_query: &mut Query<(&mut Node, &BindToID, &mut WidgetStyle), (With<SliderTrack>, Without<SliderThumb>)>,
    thumb_query: &mut Query<(&mut Node, &mut SliderThumb, &BindToID, &mut WidgetStyle), (With<SliderThumb>, Without<SliderTrack>)>,
    window: &Window,
) {
    let slider_width = computed_node.size().x / window.scale_factor();
    let track_left = (window.width() - slider_width) / 2.0;
    let click_x = pointer_position.x - track_left;
    let clamped_x = click_x.clamp(0.0, slider_width);

    for child in children.iter() {
        if let Ok((mut thumb_node, mut slider_thumb, bind_to_thumb, mut thumb_style)) = thumb_query.get_mut(child) {
            if bind_to_thumb.0 != ui_id.0 {
                continue;
            }

            slider_thumb.current_x = clamped_x;
            thumb_node.left = Val::Px(clamped_x - 8.0);
            for (_, styles) in thumb_style.styles.iter_mut() {
                styles.left = Some(thumb_node.left);
            }

            update_slider_track_width(track_query, &child, ui_id, clamped_x);

            let percent = clamped_x / slider_width;
            let range = (slider.max - slider.min) as f32;
            let raw_value = slider.min as f32 + percent * range;
            let stepped_value = ((raw_value / slider.step as f32).round() * slider.step as f32) as i32;
            slider.value = stepped_value;
        }
    }
}

/// Detects keyboard input to adjust slider values and updates the slider UI accordingly.
///
/// This system listens for left/right arrow key presses to increment or decrement in the slider's value.
/// Holding Shift increases the step size by a factor of 10 for faster adjustment.
/// After updating the slider's value, the system updates the thumb's position and tracks width visually.
///
/// # Parameters
/// - `keyboard`: Resource providing current keyboard input state.
/// - `query`: Query for sliders along with UI state, computed layout, children, UI ID, and previous value.
/// - `track_query`: Query to mutate slider track nodes and styles.
/// - `thumb_query`: Query to mutate slider thumb nodes, components, and styles.
/// - `window_query`: Query to access the primary window for scaling.
fn detect_change_slider_values(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
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

    for (mut slider, state, computed_node, children, ui_id, mut prev) in query.iter_mut() {
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
            **prev = slider.value;
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

/// Initializes the visual state of sliders that require setup.
///
/// This system runs once for sliders marked with `SliderNeedInit`.
/// It calculates the initial position of the thumb and the width of the track
/// based on the slider's current value and removes the initialization marker.
/// Ensures the slider UI matches the slider value on startup.
///
/// # Parameters
/// - `commands`: Commands to remove the initialization marker component.
/// - `query`: Query for sliders, their computed layout, children, UI ID, and initialization marker.
/// - `track_query`: Query to mutate slider track nodes, styles, and binding IDs.
/// - `thumb_query`: Query to mutate slider thumb nodes, components, styles, and binding IDs.
/// - `window_query`: Query to access the primary window for a scale factor and dimensions.
fn initialize_slider_visual_state(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Slider,
        &ComputedNode,
        &Children,
        &UIGenID,
        Option<&SliderNeedInit>,
    )>,
    mut track_query: Query<(&mut Node, &mut WidgetStyle, &BindToID), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &mut WidgetStyle, &BindToID), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else { return };

    for (entity, slider, computed_node, children, ui_id, needs_init) in query.iter_mut() {
        if needs_init.is_none() {
            continue;
        }

        let slider_width = computed_node.size().x / window.scale_factor();
        if slider_width <= 1.0 {
            continue;
        }

        let percent = (slider.value - slider.min) as f32 / (slider.max - slider.min) as f32;
        let clamped_x = percent * slider_width;

        for child in children.iter() {
            if let Ok((mut thumb_node, mut thumb, mut style, bind_to)) = thumb_query.get_mut(child) {
                if bind_to.0 != ui_id.0 {
                    continue;
                }

                thumb.current_x = clamped_x;
                thumb_node.left = Val::Px(clamped_x - 8.0);

                for (_, styles) in style.styles.iter_mut() {
                    styles.left = Some(thumb_node.left);
                }
            }

            if let Ok((mut track_node, mut style, bind_to)) = track_query.get_mut(child) {
                if bind_to.0 != ui_id.0 {
                    continue;
                }

                track_node.width = Val::Px(clamped_x);
                for (_, styles) in style.styles.iter_mut() {
                    styles.width = Some(track_node.width);
                }
            }
        }

        commands.entity(entity).remove::<SliderNeedInit>();
    }
}

/// Helper function to update the width of the slider track visually.
///
/// This function finds the track entity associated with the slider UI ID and sets its width
/// to match the current position of the slider thumb.
///
/// # Parameters
/// - `track_query`: Mutable query to access and modify slider track nodes and styles.
/// - `track_child`: The child entity representing the slider track.
/// - `ui_id`: The unique UI ID for identifying the correct slider components.
/// - `clamped_x`: The horizontal position to set the track width to.
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
