use bevy::camera::visibility::RenderLayers;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;

use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{BindToID, Slider, SliderType, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};

const MIN_DOT_VALUE_GAP: f32 = 10.0;

/// Marker component for initialized slider widgets.
#[derive(Component)]
struct SliderBase;

/// Marker component for the slider track container.
#[derive(Component)]
struct SliderTrackContainer;

/// Marker component for the slider track fill node.
#[derive(Component)]
struct SliderTrackFill;

/// Marker component for generated slider dot/label nodes.
#[derive(Component)]
struct SliderDotNode;

/// Thumb role for single/range slider handling.
#[derive(Component, Reflect, Debug, Clone, Copy, Eq, PartialEq)]
enum SliderThumbRole {
    Start,
    End,
}

/// Component storing slider thumb state.
#[derive(Component, Reflect, Debug, Clone)]
struct SliderThumb {
    role: SliderThumbRole,
    current_center_x: f32,
    hovered: bool,
}

/// Marker for thumb tooltip bubble nodes.
#[derive(Component)]
struct SliderThumbTooltip {
    role: SliderThumbRole,
}

/// Marker for thumb tooltip text nodes.
#[derive(Component)]
struct SliderThumbTooltipText {
    role: SliderThumbRole,
}

/// Stores previous slider data to detect meaningful changes.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct PreviousSliderState {
    slider_type: SliderType,
    value: f32,
    range_start: f32,
    range_end: f32,
    min: f32,
    max: f32,
    step: f32,
}

impl PreviousSliderState {
    fn from_slider(slider: &Slider) -> Self {
        Self {
            slider_type: slider.slider_type,
            value: slider.value,
            range_start: slider.range_start,
            range_end: slider.range_end,
            min: slider.min,
            max: slider.max,
            step: slider.step,
        }
    }
}

/// Marker component indicating slider needs initial layout.
#[derive(Component)]
struct SliderNeedInit;

/// Plugin that registers slider widget behavior.
pub struct SliderWidget;

impl Plugin for SliderWidget {
    /// Registers systems for slider setup and updates.
    fn build(&self, app: &mut App) {
        app.register_type::<SliderThumb>();
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                detect_change_slider_values,
                initialize_slider_visual_state,
                update_thumb_tooltips,
            )
                .chain(),
        );
    }
}

/// Initializes UI nodes for slider widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &Slider, Option<&CssSource>),
        (With<Slider>, Without<SliderBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for (entity, id, slider, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        commands
            .entity(entity)
            .insert((
                Node::default(),
                WidgetId {
                    id: slider.entry,
                    kind: WidgetKind::Slider,
                },
                BackgroundColor::default(),
                BorderColor::default(),
                ZIndex::default(),
                css_source.clone(),
                PreviousSliderState::from_slider(slider),
                TagName(String::from("slider")),
                RenderLayers::layer(layer),
                SliderNeedInit,
                SliderBase,
                Pickable::default(),
            ))
            .insert((
                ImageNode::default(),
                BoxShadow::new(
                    Colored::TRANSPARENT,
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                ),
            ))
            .insert(Name::new(format!("Slider-{}", slider.entry)))
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {
                builder
                    .spawn((
                        Name::new(format!("Slider-Track-Box-{}", slider.entry)),
                        Node::default(),
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        BorderColor::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        RelativeCursorPosition::default(),
                        css_source.clone(),
                        CssClass(vec!["slider-track".to_string()]),
                        RenderLayers::layer(layer),
                        Pickable::default(),
                        SliderTrackContainer,
                        BindToID(id.0),
                    ))
                    .insert(ImageNode::default())
                    .observe(on_track_click)
                    .observe(on_track_drag)
                    .with_children(|builder| {
                        if let Some(requested_segments) = slider.dots.filter(|v| *v > 0) {
                            let segments = effective_dot_segments(slider, requested_segments);
                            let anchor_class = match slider.dot_anchor {
                                crate::widgets::SliderDotAnchor::Top => "slider-dots-top",
                                crate::widgets::SliderDotAnchor::Bottom => "slider-dots-bottom",
                            };

                            builder
                                .spawn((
                                    Name::new(format!("Slider-Dots-{}", slider.entry)),
                                    Node::default(),
                                    BackgroundColor::default(),
                                    BorderColor::default(),
                                    ZIndex::default(),
                                    UIWidgetState::default(),
                                    css_source.clone(),
                                    CssClass(vec![
                                        "slider-dots".to_string(),
                                        anchor_class.to_string(),
                                    ]),
                                    RenderLayers::layer(layer),
                                    Pickable::IGNORE,
                                    SliderDotNode,
                                    BindToID(id.0),
                                ))
                                .with_children(|builder| {
                                    for idx in 0..=segments {
                                        let t = idx as f32 / segments as f32;
                                        let value = slider.min + t * (slider.max - slider.min);

                                        builder
                                            .spawn((
                                                Name::new(format!(
                                                    "Slider-Dot-Item-{}-{}",
                                                    slider.entry, idx
                                                )),
                                                Node::default(),
                                                BackgroundColor::default(),
                                                BorderColor::default(),
                                                ZIndex::default(),
                                                UIWidgetState::default(),
                                                css_source.clone(),
                                                CssClass(vec!["slider-dot-item".to_string()]),
                                                RenderLayers::layer(layer),
                                                Pickable::IGNORE,
                                                SliderDotNode,
                                                BindToID(id.0),
                                            ))
                                            .with_children(|builder| {
                                                if slider.show_labels {
                                                    builder.spawn((
                                                        Name::new(format!(
                                                            "Slider-Dot-Label-{}-{}",
                                                            slider.entry, idx
                                                        )),
                                                        Node::default(),
                                                        Text::new(format_slider_value(value)),
                                                        TextColor::default(),
                                                        TextFont::default(),
                                                        TextLayout::default(),
                                                        UIWidgetState::default(),
                                                        css_source.clone(),
                                                        CssClass(vec![
                                                            "slider-dot-label".to_string(),
                                                        ]),
                                                        RenderLayers::layer(layer),
                                                        Pickable::IGNORE,
                                                        SliderDotNode,
                                                        BindToID(id.0),
                                                    ));
                                                }

                                                builder.spawn((
                                                    Name::new(format!(
                                                        "Slider-Dot-{}-{}",
                                                        slider.entry, idx
                                                    )),
                                                    Node::default(),
                                                    BackgroundColor::default(),
                                                    BorderColor::default(),
                                                    ZIndex::default(),
                                                    UIWidgetState::default(),
                                                    css_source.clone(),
                                                    CssClass(vec!["slider-dot".to_string()]),
                                                    RenderLayers::layer(layer),
                                                    Pickable::IGNORE,
                                                    SliderDotNode,
                                                    BindToID(id.0),
                                                ));
                                            });
                                    }
                                });
                        }

                        builder.spawn((
                            Name::new(format!("Slider-Fill-{}", slider.entry)),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                height: Val::Percent(100.0),
                                width: Val::Px(0.0),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BorderColor::default(),
                            ZIndex::default(),
                            UIWidgetState::default(),
                            css_source.clone(),
                            CssClass(vec!["track-fill".to_string()]),
                            RenderLayers::layer(layer),
                            Pickable::IGNORE,
                            SliderTrackFill,
                            BindToID(id.0),
                        ));

                        spawn_thumb(
                            builder,
                            slider,
                            &css_source,
                            layer,
                            id.0,
                            SliderThumbRole::Start,
                        );
                        spawn_thumb(
                            builder,
                            slider,
                            &css_source,
                            layer,
                            id.0,
                            SliderThumbRole::End,
                        );
                    });
            });
    }
}

fn spawn_thumb(
    builder: &mut ChildSpawnerCommands,
    slider: &Slider,
    css_source: &CssSource,
    layer: usize,
    ui_id: usize,
    role: SliderThumbRole,
) {
    let role_name = match role {
        SliderThumbRole::Start => "start",
        SliderThumbRole::End => "end",
    };

    builder
        .spawn((
            Name::new(format!("Slider-Thumb-{}-{}", slider.entry, role_name)),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            ZIndex::default(),
            UIWidgetState::default(),
            css_source.clone(),
            CssClass(vec!["thumb".to_string(), format!("thumb-{}", role_name)]),
            RenderLayers::layer(layer),
            Pickable::default(),
            SliderThumb {
                role,
                current_center_x: 0.0,
                hovered: false,
            },
            BindToID(ui_id),
        ))
        .insert((
            ImageNode::default(),
            BoxShadow::new(
                Colored::TRANSPARENT,
                Val::Px(0.),
                Val::Px(0.),
                Val::Px(0.),
                Val::Px(0.),
            ),
        ))
        .observe(on_thumb_drag)
        .observe(on_thumb_over)
        .observe(on_thumb_out)
        .with_children(|builder| {
            builder
                .spawn((
                    Name::new(format!(
                        "Slider-Thumb-Tooltip-{}-{}",
                        slider.entry, role_name
                    )),
                    Node::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    ZIndex::default(),
                    css_source.clone(),
                    CssClass(vec!["slider-thumb-tooltip".to_string()]),
                    RenderLayers::layer(layer),
                    Pickable::IGNORE,
                    Visibility::Hidden,
                    SliderThumbTooltip { role },
                    BindToID(ui_id),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Name::new(format!(
                            "Slider-Thumb-Tooltip-Text-{}-{}",
                            slider.entry, role_name
                        )),
                        Node::default(),
                        Text::new(""),
                        TextColor::default(),
                        TextFont::default(),
                        TextLayout::default(),
                        css_source.clone(),
                        CssClass(vec!["slider-thumb-tooltip-text".to_string()]),
                        RenderLayers::layer(layer),
                        Pickable::IGNORE,
                        SliderThumbTooltipText { role },
                        BindToID(ui_id),
                    ));

                    builder.spawn((
                        Name::new(format!(
                            "Slider-Thumb-Tooltip-Nose-{}-{}",
                            slider.entry, role_name
                        )),
                        Node::default(),
                        BackgroundColor::default(),
                        BorderColor::default(),
                        ZIndex::default(),
                        css_source.clone(),
                        CssClass(vec!["slider-thumb-tooltip-nose".to_string()]),
                        RenderLayers::layer(layer),
                        Pickable::IGNORE,
                        BindToID(ui_id),
                    ));
                });
        });
}

/// Focuses the slider widget on click.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Slider>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        if state.disabled {
            trigger.propagate(false);
            return;
        }
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
    trigger.propagate(false);
}

/// Sets hovered state when the cursor enters a slider.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

/// Clears hovered state when the cursor leaves a slider.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

/// Handles clicks on the slider track to update value.
fn on_track_click(
    mut trigger: On<Pointer<Click>>,
    ui_scale: Res<UiScale>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut slider_query: Query<(&mut Slider, &UIGenID), With<Slider>>,
    slider_state_q: Query<(&UIGenID, &UIWidgetState), With<Slider>>,
    track_q: Query<(&ComputedNode, &BindToID, &RelativeCursorPosition), With<SliderTrackContainer>>,
    thumb_size_q: Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    mut fill_q: Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    mut thumb_q: Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;

    apply_from_track_pointer(
        trigger.entity,
        sf,
        &mut slider_query,
        &slider_state_q,
        &track_q,
        &thumb_size_q,
        &mut fill_q,
        &mut thumb_q,
    );

    trigger.propagate(false);
}

/// Handles dragging directly on the slider track.
fn on_track_drag(
    mut trigger: On<Pointer<Drag>>,
    ui_scale: Res<UiScale>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut slider_query: Query<(&mut Slider, &UIGenID), With<Slider>>,
    slider_state_q: Query<(&UIGenID, &UIWidgetState), With<Slider>>,
    track_q: Query<(&ComputedNode, &BindToID, &RelativeCursorPosition), With<SliderTrackContainer>>,
    thumb_size_q: Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    mut fill_q: Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    mut thumb_q: Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;

    apply_from_track_pointer(
        trigger.entity,
        sf,
        &mut slider_query,
        &slider_state_q,
        &track_q,
        &thumb_size_q,
        &mut fill_q,
        &mut thumb_q,
    );

    trigger.propagate(false);
}

fn apply_from_track_pointer(
    track_entity: Entity,
    sf: f32,
    slider_query: &mut Query<(&mut Slider, &UIGenID), With<Slider>>,
    slider_state_q: &Query<(&UIGenID, &UIWidgetState), With<Slider>>,
    track_q: &Query<
        (&ComputedNode, &BindToID, &RelativeCursorPosition),
        With<SliderTrackContainer>,
    >,
    thumb_size_q: &Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    fill_q: &mut Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    thumb_q: &mut Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
) {
    let Ok((track_node, bind, rel)) = track_q.get(track_entity) else {
        return;
    };

    if is_slider_disabled(bind.0, slider_state_q) {
        return;
    }

    let track_width = (track_node.size().x / sf).max(1.0);
    let Some(thumb_width) = find_bound_width(bind.0, thumb_size_q, sf) else {
        return;
    };
    if thumb_width <= 1.0 {
        return;
    }

    let Some(n) = rel.normalized else {
        return;
    };

    let t = (n.x + 0.5).clamp(0.0, 1.0);
    let click_x = t * track_width;
    let desired_left = click_x - thumb_width * 0.5;

    apply_from_track_left_x(
        bind.0,
        None,
        desired_left,
        track_width,
        thumb_width,
        slider_query,
        fill_q,
        thumb_q,
    );
}

/// Handles dragging the slider thumb to update value.
fn on_thumb_drag(
    event: On<Pointer<Drag>>,
    parent_q: Query<&ChildOf>,
    track_q: Query<(&ComputedNode, &BindToID), With<SliderTrackContainer>>,
    thumb_node_q: Query<&ComputedNode, With<SliderThumb>>,
    mut slider_q: Query<(&mut Slider, &UIGenID), With<Slider>>,
    slider_state_q: Query<(&UIGenID, &UIWidgetState), With<Slider>>,
    mut fill_q: Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    mut thumb_q: Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;

    let Ok(parent) = parent_q.get(event.entity) else {
        return;
    };
    let Ok((track_node, bind)) = track_q.get(parent.parent()) else {
        return;
    };

    if is_slider_disabled(bind.0, &slider_state_q) {
        return;
    }

    let track_width = (track_node.size().x / sf).max(1.0);

    let Ok(thumb_node) = thumb_node_q.get(event.entity) else {
        return;
    };
    let thumb_width = (thumb_node.size().x / sf).max(1.0);
    let half = thumb_width * 0.5;

    let dx = event.event.delta.x / sf;

    let Ok((_, thumb, _, _, _)) = thumb_q.get(event.entity) else {
        return;
    };
    let current_left = thumb.current_center_x - half;

    apply_from_track_left_x(
        bind.0,
        Some(thumb.role),
        current_left + dx,
        track_width,
        thumb_width,
        &mut slider_q,
        &mut fill_q,
        &mut thumb_q,
    );
}

/// Tracks thumb hover state.
fn on_thumb_over(mut trigger: On<Pointer<Over>>, mut thumb_q: Query<&mut SliderThumb>) {
    if let Ok(mut thumb) = thumb_q.get_mut(trigger.entity) {
        thumb.hovered = true;
    }
    trigger.propagate(false);
}

/// Clears thumb hover state.
fn on_thumb_out(mut trigger: On<Pointer<Out>>, mut thumb_q: Query<&mut SliderThumb>) {
    if let Ok(mut thumb) = thumb_q.get_mut(trigger.entity) {
        thumb.hovered = false;
    }
    trigger.propagate(false);
}

/// Applies slider value based on a track X position.
fn apply_from_track_left_x(
    target_ui_id: usize,
    preferred_role: Option<SliderThumbRole>,
    desired_left_x: f32,
    track_width: f32,
    thumb_width: f32,
    slider_q: &mut Query<(&mut Slider, &UIGenID), With<Slider>>,
    fill_q: &mut Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    thumb_q: &mut Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
) {
    for (mut slider, ui_id) in slider_q.iter_mut() {
        if ui_id.0 != target_ui_id {
            continue;
        }

        sanitize_slider(&mut slider);

        let role = resolve_target_role(
            &slider,
            preferred_role,
            desired_left_x,
            track_width,
            thumb_width,
            ui_id.0,
            thumb_q,
        );

        let next_value = value_from_left(desired_left_x, track_width, thumb_width, &slider);
        match slider.slider_type {
            SliderType::Default => {
                slider.value = next_value;
            }
            SliderType::Range => match role {
                SliderThumbRole::Start => {
                    slider.range_start = next_value.min(slider.range_end);
                }
                SliderThumbRole::End => {
                    slider.range_end = next_value.max(slider.range_start);
                }
            },
        }

        apply_slider_visual_state(
            ui_id,
            &mut slider,
            track_width,
            thumb_width,
            fill_q,
            thumb_q,
        );
        break;
    }
}

fn resolve_target_role(
    slider: &Slider,
    preferred_role: Option<SliderThumbRole>,
    desired_left_x: f32,
    _track_width: f32,
    thumb_width: f32,
    ui_id: usize,
    thumb_q: &mut Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
) -> SliderThumbRole {
    if slider.slider_type != SliderType::Range {
        return SliderThumbRole::End;
    }

    if let Some(role) = preferred_role {
        return role;
    }

    let desired_center_x = desired_left_x + thumb_width * 0.5;
    let mut start_center: Option<f32> = None;
    let mut end_center: Option<f32> = None;

    for (_, thumb, bind, _, _) in thumb_q.iter_mut() {
        if bind.0 != ui_id {
            continue;
        }

        match thumb.role {
            SliderThumbRole::Start => start_center = Some(thumb.current_center_x),
            SliderThumbRole::End => end_center = Some(thumb.current_center_x),
        }
    }

    match (start_center, end_center) {
        (Some(start), Some(end)) => {
            let start_dist = (desired_center_x - start).abs();
            let end_dist = (desired_center_x - end).abs();
            if start_dist <= end_dist {
                SliderThumbRole::Start
            } else {
                SliderThumbRole::End
            }
        }
        _ => SliderThumbRole::Start,
    }
}

/// Applies visual slider position from current slider values.
fn apply_slider_visual_state(
    ui_id: &UIGenID,
    slider: &mut Slider,
    track_width: f32,
    thumb_width: f32,
    fill_q: &mut Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    thumb_q: &mut Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
) {
    sanitize_slider(slider);

    let track_width = track_width.max(1.0);
    let thumb_width = thumb_width.max(1.0);
    let half = thumb_width * 0.5;
    let max_left = (track_width - thumb_width).max(0.0);

    let (start_left, end_left, start_visible, fill_left, fill_width) =
        if slider.slider_type == SliderType::Range {
            let start_percent = normalize_value(slider.range_start, slider.min, slider.max);
            let end_percent = normalize_value(slider.range_end, slider.min, slider.max);
            let start_left = start_percent * max_left;
            let end_left = end_percent * max_left;
            let fill_left = start_left + half;
            let fill_width = (end_left - start_left).max(0.0);
            (start_left, end_left, true, fill_left, fill_width)
        } else {
            let end_percent = normalize_value(slider.value, slider.min, slider.max);
            let end_left = end_percent * max_left;
            let fill_width = (end_left + half).clamp(0.0, track_width);
            (0.0, end_left, false, 0.0, fill_width)
        };

    for (mut node, mut thumb, bind, mut style, mut visibility) in thumb_q.iter_mut() {
        if bind.0 != ui_id.0 {
            continue;
        }

        let left = match thumb.role {
            SliderThumbRole::Start => start_left,
            SliderThumbRole::End => end_left,
        };

        node.left = Val::Px(left);
        thumb.current_center_x = left + half;

        let visible = thumb.role == SliderThumbRole::End || start_visible;
        *visibility = if visible {
            Visibility::Inherited
        } else {
            thumb.hovered = false;
            Visibility::Hidden
        };

        for (_, s) in style.styles.iter_mut() {
            s.normal.left = Some(node.left);
            s.normal.top = Some(node.top);
        }
    }

    for (mut node, bind, mut style) in fill_q.iter_mut() {
        if bind.0 != ui_id.0 {
            continue;
        }

        node.left = Val::Px(fill_left.max(0.0).min(track_width));
        node.width = Val::Px(fill_width.max(0.0).min(track_width));
        for (_, s) in style.styles.iter_mut() {
            s.normal.left = Some(node.left);
            s.normal.width = Some(node.width);
        }
    }
}

/// Detects slider value changes and updates visuals.
fn detect_change_slider_values(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut slider_q: Query<
        (
            &mut Slider,
            &UIWidgetState,
            &UIGenID,
            &mut PreviousSliderState,
        ),
        With<Slider>,
    >,
    track_q: Query<(&ComputedNode, &BindToID), With<SliderTrackContainer>>,
    thumb_size_q: Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    mut fill_q: Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    mut thumb_q: Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    for (mut slider, state, ui_id, mut prev) in slider_q.iter_mut() {
        sanitize_slider(&mut slider);

        if state.focused && !state.disabled {
            let step = if shift {
                slider.step * 10.0
            } else {
                slider.step
            };

            match slider.slider_type {
                SliderType::Default => {
                    if keyboard.just_pressed(KeyCode::ArrowRight) {
                        slider.value = (slider.value + step).min(slider.max);
                    }
                    if keyboard.just_pressed(KeyCode::ArrowLeft) {
                        slider.value = (slider.value - step).max(slider.min);
                    }
                }
                SliderType::Range => {
                    if keyboard.just_pressed(KeyCode::ArrowRight) {
                        slider.range_end = (slider.range_end + step).min(slider.max);
                        slider.range_end = slider.range_end.max(slider.range_start);
                    }
                    if keyboard.just_pressed(KeyCode::ArrowLeft) {
                        slider.range_start = (slider.range_start - step).max(slider.min);
                        slider.range_start = slider.range_start.min(slider.range_end);
                    }
                }
            }
        }

        sanitize_slider(&mut slider);
        let next_state = PreviousSliderState::from_slider(&slider);
        if next_state == *prev {
            continue;
        }
        *prev = next_state;

        let track_width = find_bound_width(ui_id.0, &track_q, sf).unwrap_or(0.0);
        let Some(thumb_width) = find_bound_width(ui_id.0, &thumb_size_q, sf) else {
            continue;
        };
        if track_width <= 1.0 || thumb_width <= 1.0 {
            continue;
        }

        apply_slider_visual_state(
            ui_id,
            &mut slider,
            track_width,
            thumb_width,
            &mut fill_q,
            &mut thumb_q,
        );
    }
}

/// Initializes slider visuals after layout is available.
fn initialize_slider_visual_state(
    mut commands: Commands,
    mut slider_q: Query<(Entity, &mut Slider, &UIGenID, Option<&SliderNeedInit>), With<Slider>>,
    track_q: Query<(&ComputedNode, &BindToID), With<SliderTrackContainer>>,
    thumb_size_q: Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    mut fill_q: Query<
        (&mut Node, &BindToID, &mut UiStyle),
        (With<SliderTrackFill>, Without<SliderThumb>),
    >,
    mut thumb_q: Query<
        (
            &mut Node,
            &mut SliderThumb,
            &BindToID,
            &mut UiStyle,
            &mut Visibility,
        ),
        (With<SliderThumb>, Without<SliderTrackFill>),
    >,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;

    for (entity, mut slider, ui_id, needs) in slider_q.iter_mut() {
        if needs.is_none() {
            continue;
        }

        sanitize_slider(&mut slider);

        let track_width = find_bound_width(ui_id.0, &track_q, sf).unwrap_or(0.0);
        let Some(thumb_width) = find_bound_width(ui_id.0, &thumb_size_q, sf) else {
            continue;
        };
        if track_width <= 1.0 || thumb_width <= 1.0 {
            continue;
        }

        apply_slider_visual_state(
            ui_id,
            &mut slider,
            track_width,
            thumb_width,
            &mut fill_q,
            &mut thumb_q,
        );

        commands.entity(entity).remove::<SliderNeedInit>();
    }
}

/// Updates tooltip visibility/text for thumbs.
fn update_thumb_tooltips(
    slider_q: Query<(&Slider, &UIGenID), With<Slider>>,
    thumb_q: Query<
        (&SliderThumb, &BindToID, &Visibility),
        (With<SliderThumb>, Without<SliderThumbTooltip>),
    >,
    mut tooltip_q: Query<
        (&mut Visibility, &SliderThumbTooltip, &BindToID),
        (With<SliderThumbTooltip>, Without<SliderThumb>),
    >,
    mut text_q: Query<
        (&mut Text, &SliderThumbTooltipText, &BindToID),
        (With<SliderThumbTooltipText>, Without<SliderThumb>),
    >,
) {
    for (mut visibility, tooltip, bind) in tooltip_q.iter_mut() {
        let Some(slider) = slider_q
            .iter()
            .find(|(_, ui_id)| ui_id.0 == bind.0)
            .map(|(slider, _)| slider)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if !slider.show_tip {
            *visibility = Visibility::Hidden;
            continue;
        }

        let mut hovered = false;
        let mut thumb_visible = false;

        for (thumb, thumb_bind, thumb_visibility) in thumb_q.iter() {
            if thumb_bind.0 != bind.0 || thumb.role != tooltip.role {
                continue;
            }
            hovered = thumb.hovered;
            thumb_visible = *thumb_visibility != Visibility::Hidden;
            break;
        }

        *visibility = if hovered && thumb_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for (mut text, tooltip_text, bind) in text_q.iter_mut() {
        if let Some((slider, _)) = slider_q.iter().find(|(_, ui_id)| ui_id.0 == bind.0) {
            let value = match tooltip_text.role {
                SliderThumbRole::Start => slider.range_start,
                SliderThumbRole::End => {
                    if slider.slider_type == SliderType::Range {
                        slider.range_end
                    } else {
                        slider.value
                    }
                }
            };
            text.0 = format_slider_value(value);
        }
    }
}

fn value_from_left(desired_left: f32, track_width: f32, thumb_width: f32, slider: &Slider) -> f32 {
    let track_width = track_width.max(1.0);
    let thumb_width = thumb_width.max(1.0).min(track_width);

    let max_left = (track_width - thumb_width).max(0.0);
    let left = desired_left.clamp(0.0, max_left);
    let percent = if max_left > 0.0 { left / max_left } else { 0.0 };

    let raw = slider.min + percent.clamp(0.0, 1.0) * (slider.max - slider.min);
    snap_to_step(raw, slider)
}

fn normalize_value(value: f32, min: f32, max: f32) -> f32 {
    let span = (max - min).abs();
    if span <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

fn snap_to_step(raw: f32, slider: &Slider) -> f32 {
    let step = slider.step.abs().max(f32::EPSILON);
    let snapped = slider.min + ((raw - slider.min) / step).round() * step;
    snapped.clamp(slider.min, slider.max)
}

fn sanitize_slider(slider: &mut Slider) {
    if slider.max < slider.min {
        std::mem::swap(&mut slider.min, &mut slider.max);
    }

    slider.step = slider.step.abs().max(f32::EPSILON);
    slider.value = slider.value.clamp(slider.min, slider.max);
    slider.range_start = slider.range_start.clamp(slider.min, slider.max);
    slider.range_end = slider.range_end.clamp(slider.min, slider.max);

    if slider.range_start > slider.range_end {
        std::mem::swap(&mut slider.range_start, &mut slider.range_end);
    }
}

fn format_slider_value(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract().abs() < 0.0001 {
        return format!("{}", rounded as i64);
    }

    let mut txt = format!("{rounded:.2}");
    while txt.ends_with('0') {
        txt.pop();
    }
    if txt.ends_with('.') {
        txt.pop();
    }
    txt
}

fn effective_dot_segments(slider: &Slider, requested: u32) -> usize {
    let requested = requested.max(1) as usize;
    let span = (slider.max - slider.min).abs();

    if span <= f32::EPSILON {
        return 1;
    }

    let by_gap = (span / MIN_DOT_VALUE_GAP).floor().max(1.0) as usize;
    requested.min(by_gap.max(1))
}

/// Finds a bound width for a given slider entity.
fn find_bound_width<Q>(
    ui_id: usize,
    query: &Query<(&ComputedNode, &BindToID), Q>,
    scale_factor: f32,
) -> Option<f32>
where
    Q: QueryFilter,
{
    query
        .iter()
        .find(|(_, bind)| bind.0 == ui_id)
        .map(|(computed, _)| computed.size().x / scale_factor)
}

/// Returns true if the slider is disabled.
fn is_slider_disabled(
    ui_id: usize,
    query: &Query<(&UIGenID, &UIWidgetState), With<Slider>>,
) -> bool {
    query
        .iter()
        .find(|(gen_id, _)| gen_id.0 == ui_id)
        .is_some_and(|(_, state)| state.disabled)
}
