use bevy::camera::visibility::RenderLayers;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use crate::styles::{CssClass, CssSource, TagName};
use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::widgets::{BindToID, Slider, UIGenID, UIWidgetState, WidgetId, WidgetKind};

#[derive(Component)]
struct SliderBase;

#[derive(Component)]
struct SliderTrackContainer;

#[derive(Component)]
struct SliderTrackFill;

#[derive(Component, Reflect, Debug, Clone)]
struct SliderThumb {
    current_center_x: f32,
}

#[derive(Component, Deref, DerefMut)]
struct PreviousSliderValue(f32);

#[derive(Component)]
struct SliderNeedInit;

pub struct SliderWidget;

impl Plugin for SliderWidget {
    fn build(&self, app: &mut App) {
        app.register_type::<SliderThumb>();
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                detect_change_slider_values,
                initialize_slider_visual_state,
            )
                .chain(),
        );
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &Slider, Option<&CssSource>), (With<Slider>, Without<SliderBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for (entity, id, slider, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        commands
            .entity(entity)
            .insert((
                Node::default(),
                WidgetId { id: slider.w_count, kind: WidgetKind::Slider },
                BackgroundColor::default(),
                BorderColor::default(),
                BorderRadius::default(),
                ZIndex::default(),
                css_source.clone(),
                PreviousSliderValue(slider.value),
                TagName(String::from("slider")),
                RenderLayers::layer(layer),
                SliderNeedInit,
                SliderBase,
                Pickable::default(),
            ))
            .insert((
                ImageNode::default(),
                BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ))
            .insert(Name::new(format!("Slider-{}", slider.w_count)))
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {
                builder
                    .spawn((
                        Name::new(format!("Slider-Track-Box-{}", slider.w_count)),
                        Node::default(),
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        BorderColor::default(),
                        BorderRadius::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["slider-track".to_string()]),
                        RenderLayers::layer(layer),
                        Pickable::default(),
                        SliderTrackContainer,
                        BindToID(id.0),
                    ))
                    .insert(ImageNode::default())
                    .observe(on_track_click)
                    .with_children(|builder| {
                        builder
                            .spawn((
                                Name::new(format!("Slider-Fill-{}", slider.w_count)),
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
                                BorderRadius::default(),
                                ZIndex::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["track-fill".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::IGNORE,
                                SliderTrackFill,
                                BindToID(id.0),
                            ))
                            .insert(ImageNode::default());

                        builder
                            .spawn((
                                Name::new(format!("Slider-Thumb-{}", slider.w_count)),
                                Node::default(),
                                BackgroundColor::default(),
                                BorderColor::default(),
                                BorderRadius::default(),
                                ZIndex::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["thumb".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::default(),
                                SliderThumb { current_center_x: 0.0 },
                                BindToID(id.0),
                            ))
                            .insert((
                                ImageNode::default(),
                                BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                            ))
                            .observe(on_thumb_drag);
                    });
            });
    }
}

fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Slider>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
    trigger.propagate(false);
}

fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Slider>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

fn on_track_click(
    mut trigger: On<Pointer<Click>>,
    mut slider_query: Query<(&mut Slider, &UIGenID), With<Slider>>,
    track_q: Query<(&ComputedNode, &GlobalTransform, &BindToID), With<SliderTrackContainer>>,
    thumb_size_q: Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    mut fill_q: Query<(&mut Node, &BindToID, &mut UiStyle), (With<SliderTrackFill>, Without<SliderThumb>)>,
    mut thumb_q: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut UiStyle), (With<SliderThumb>, Without<SliderTrackFill>)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.single() else { return; };
    let sf = window.scale_factor();

    info!("1");

    // IMPORTANT: use the listener entity (the one you attached `.observe` to)
    let track_entity = trigger.entity;

    let Ok((track_node, track_gt, bind)) = track_q.get(track_entity) else { return; };
    let Some(hit_pos) = trigger.hit.position else { return; };

    info!("2");

    let track_width = (track_node.size().x / sf).max(1.0);
    let Some(thumb_width) = find_bound_width(bind.0, &thumb_size_q, sf) else { return; };

    info!("3");

    let click_x = world_to_local_left_x(hit_pos, track_gt, track_width);
    let desired_left = click_x - thumb_width * 0.5;

    apply_from_track_left_x(
        bind.0,                // <-- this is your usize BindToID
        desired_left,
        track_width,
        thumb_width,
        &mut slider_query,
        &mut fill_q,
        &mut thumb_q,
    );

    // so the click doesn't bubble up and get re-handled / swallowed elsewhere
    trigger.propagate(false);
}

fn on_thumb_drag(
    event: On<Pointer<Drag>>,
    parent_q: Query<&ChildOf>,
    track_q: Query<(&ComputedNode, &BindToID), With<SliderTrackContainer>>,
    thumb_node_q: Query<&ComputedNode, With<SliderThumb>>,
    mut slider_q: Query<(&mut Slider, &UIGenID), With<Slider>>,
    mut fill_q: Query<(&mut Node, &BindToID, &mut UiStyle), (With<SliderTrackFill>, Without<SliderThumb>)>,
    mut thumb_q: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut UiStyle), (With<SliderThumb>, Without<SliderTrackFill>)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.single() else { return };
    let sf = window.scale_factor();

    let Ok(parent) = parent_q.get(event.entity) else { return; };
    let Ok((track_node, bind)) = track_q.get(parent.parent()) else { return; };
    let track_width = (track_node.size().x / sf).max(1.0);

    let Ok(thumb_node) = thumb_node_q.get(event.entity) else { return; };
    let thumb_width = (thumb_node.size().x / sf).max(1.0);
    let half = thumb_width * 0.5;

    let dx = event.event.delta.x / sf;

    let Ok((_, thumb, _, _)) = thumb_q.get(event.entity) else { return; };
    let current_left = thumb.current_center_x - half;

    apply_from_track_left_x(
        bind.0,
        current_left + dx,
        track_width,
        thumb_width,
        &mut slider_q,
        &mut fill_q,
        &mut thumb_q,
    );
}

fn world_to_local_left_x(hit_world: Vec3, track_gt: &GlobalTransform, track_width: f32) -> f32 {
    let inv = track_gt.to_matrix().inverse();
    let local = inv.transform_point3(hit_world);
    local.x + track_width * 0.5
}

fn apply_from_track_left_x(
    target_ui_id: usize,
    desired_left_x: f32,
    track_width: f32,
    thumb_width: f32,
    slider_q: &mut Query<(&mut Slider, &UIGenID), With<Slider>>,
    fill_q: &mut Query<(&mut Node, &BindToID, &mut UiStyle), (With<SliderTrackFill>, Without<SliderThumb>)>,
    thumb_q: &mut Query<(&mut Node, &mut SliderThumb, &BindToID, &mut UiStyle), (With<SliderThumb>, Without<SliderTrackFill>)>,
) {
    for (mut slider, ui_id) in slider_q.iter_mut() {
        if ui_id.0 != target_ui_id {
            continue;
        }
        apply_slider_from_thumb_left(
            desired_left_x,
            &mut slider,
            track_width,
            thumb_width,
            ui_id,
            fill_q,
            thumb_q,
        );
    }
}

fn apply_slider_from_thumb_left(
    desired_left: f32,
    slider: &mut Slider,
    track_width: f32,
    thumb_width: f32,
    ui_id: &UIGenID,
    fill_q: &mut Query<(&mut Node, &BindToID, &mut UiStyle), (With<SliderTrackFill>, Without<SliderThumb>)>,
    thumb_q: &mut Query<(&mut Node, &mut SliderThumb, &BindToID, &mut UiStyle), (With<SliderThumb>, Without<SliderTrackFill>)>,
) {
    let track_width = track_width.max(1.0);
    let thumb_width = thumb_width.max(1.0);
    let half = thumb_width * 0.5;

    let max_left = (track_width - thumb_width).max(0.0);
    let left = desired_left.clamp(0.0, max_left);

    let percent = if max_left > 0.0 { left / max_left } else { 0.0 };
    let percent = percent.clamp(0.0, 1.0);

    let center_x = left + half;
    let fill_width = center_x.clamp(0.0, track_width);

    for (mut node, mut thumb, bind, mut style) in thumb_q.iter_mut() {
        if bind.0 != ui_id.0 {
            continue;
        }
        thumb.current_center_x = center_x;
        node.left = Val::Px(left);

        for (_, s) in style.styles.iter_mut() {
            s.left = Some(node.left);
            s.top = Some(node.top);
        }
    }

    for (mut node, bind, mut style) in fill_q.iter_mut() {
        if bind.0 != ui_id.0 {
            continue;
        }
        node.width = Val::Px(fill_width);
        for (_, s) in style.styles.iter_mut() {
            s.width = Some(node.width);
        }
    }

    let range = slider.max - slider.min;
    let raw = slider.min + percent * range;

    let step = slider.step.max(f32::EPSILON);
    slider.value = (raw / step).round() * step;
}

fn detect_change_slider_values(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut slider_q: Query<(&mut Slider, &UIWidgetState, &UIGenID, &mut PreviousSliderValue), With<Slider>>,
    track_q: Query<(&ComputedNode, &BindToID), With<SliderTrackContainer>>,
    thumb_size_q: Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    mut fill_q: Query<(&mut Node, &BindToID, &mut UiStyle), (With<SliderTrackFill>, Without<SliderThumb>)>,
    mut thumb_q: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut UiStyle), (With<SliderThumb>, Without<SliderTrackFill>)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.single() else { return };
    let sf = window.scale_factor();

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    for (mut slider, state, ui_id, mut prev) in slider_q.iter_mut() {
        if state.focused {
            let step = if shift { slider.step * 10.0 } else { slider.step };
            if keyboard.just_pressed(KeyCode::ArrowRight) {
                slider.value = (slider.value + step).min(slider.max);
            }
            if keyboard.just_pressed(KeyCode::ArrowLeft) {
                slider.value = (slider.value - step).max(slider.min);
            }
        }

        if slider.value == **prev {
            continue;
        }
        **prev = slider.value;

        let track_width = find_bound_width(ui_id.0, &track_q, sf).unwrap_or(1.0);
        let Some(thumb_width) = find_bound_width(ui_id.0, &thumb_size_q, sf) else { continue; };

        let max_left = (track_width - thumb_width).max(0.0);
        let percent = ((slider.value - slider.min) / (slider.max - slider.min)).clamp(0.0, 1.0);
        let left = percent * max_left;

        apply_slider_from_thumb_left(left, &mut slider, track_width, thumb_width, ui_id, &mut fill_q, &mut thumb_q);
    }
}

fn initialize_slider_visual_state(
    mut commands: Commands,
    mut slider_q: Query<(Entity, &mut Slider, &UIGenID, Option<&SliderNeedInit>), With<Slider>>,
    track_q: Query<(&ComputedNode, &BindToID), With<SliderTrackContainer>>,
    thumb_size_q: Query<(&ComputedNode, &BindToID), With<SliderThumb>>,
    mut fill_q: Query<(&mut Node, &BindToID, &mut UiStyle), (With<SliderTrackFill>, Without<SliderThumb>)>,
    mut thumb_q: Query<(&mut Node, &mut SliderThumb, &BindToID, &mut UiStyle), (With<SliderThumb>, Without<SliderTrackFill>)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.single() else { return };
    let sf = window.scale_factor();

    for (entity, mut slider, ui_id, needs) in slider_q.iter_mut() {
        if needs.is_none() {
            continue;
        }

        let track_width = find_bound_width(ui_id.0, &track_q, sf).unwrap_or(1.0);
        let Some(thumb_width) = find_bound_width(ui_id.0, &thumb_size_q, sf) else { continue; };

        let max_left = (track_width - thumb_width).max(0.0);
        let percent = ((slider.value - slider.min) / (slider.max - slider.min)).clamp(0.0, 1.0);
        let left = percent * max_left;

        apply_slider_from_thumb_left(left, &mut slider, track_width, thumb_width, ui_id, &mut fill_q, &mut thumb_q);

        commands.entity(entity).remove::<SliderNeedInit>();
    }
}

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
        .map(|(computed, _)| (computed.size().x / scale_factor).max(1.0))
}
