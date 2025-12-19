use bevy::camera::visibility::RenderLayers;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, ScrollPosition};
use bevy::window::PrimaryWindow;

use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{BindToID, Scrollbar, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};

#[derive(Component)]
struct ScrollBase;
#[derive(Component)]
struct ScrollTrack;
#[derive(Component)]
struct ScrollThumb {
    current_center: f32,
}
#[derive(Component, Deref, DerefMut)]
struct PreviousScrollValue(f32);
#[derive(Component)]
struct ScrollNeedInit;

pub struct ScrollWidget;

impl Plugin for ScrollWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                detect_change_scroll_values,
                initialize_scroll_visual_state,
            )
                .chain(),
        );
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &Scrollbar, Option<&CssSource>),
        (With<Scrollbar>, Without<ScrollBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for (entity, id, scroll, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Scroll-{}", scroll.entry)),
                Node::default(),
                WidgetId {
                    id: scroll.entry,
                    kind: WidgetKind::Scrollbar,
                },
                BackgroundColor::default(),
                BorderColor::default(),
                BorderRadius::default(),
                ZIndex::default(),
                css_source.clone(),
                PreviousScrollValue(scroll.value),
                TagName(String::from("scroll")),
                RenderLayers::layer(layer),
                ScrollNeedInit,
                ScrollBase,
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
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {
                builder
                    .spawn((
                        Name::new(format!("Scroll-Track-{}", scroll.entry)),
                        Node::default(),
                        BackgroundColor::default(),
                        BorderColor::default(),
                        BorderRadius::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        RelativeCursorPosition::default(),
                        css_source.clone(),
                        CssClass(vec!["scroll-track".to_string()]),
                        RenderLayers::layer(layer),
                        Pickable::default(),
                        ScrollTrack,
                        BindToID(id.0),
                    ))
                    .insert(ImageNode::default())
                    .observe(on_track_click)
                    .with_children(|builder| {
                        builder
                            .spawn((
                                Name::new(format!("Scroll-Thumb-{}", scroll.entry)),
                                Node::default(),
                                BackgroundColor::default(),
                                BorderColor::default(),
                                BorderRadius::default(),
                                ZIndex::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["scroll-thumb".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::default(),
                                ScrollThumb {
                                    current_center: 0.0,
                                },
                                BindToID(id.0),
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
                            .observe(on_thumb_drag);
                    });
            });
    }
}

fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Scrollbar>>,
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
    mut query: Query<&mut UIWidgetState, With<Scrollbar>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Scrollbar>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

fn on_track_click(
    mut trigger: On<Pointer<Click>>,
    ui_scale: Res<UiScale>,
    window_q: Query<&Window, With<PrimaryWindow>>,

    mut scroll_q: Query<(&mut Scrollbar, &UIGenID), With<Scrollbar>>,
    track_q: Query<(&ComputedNode, &BindToID, &RelativeCursorPosition), With<ScrollTrack>>,
    thumb_q: Query<(&ComputedNode, &BindToID), With<ScrollThumb>>,

    mut thumb_node_q: Query<(&mut Node, &mut ScrollThumb, &BindToID, &mut UiStyle), With<ScrollThumb>>,
    mut target_scroll_q: Query<&mut ScrollPosition>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;

    let Ok((track_node, bind, rel)) = track_q.get(trigger.entity) else {
        return;
    };

    let track_height = (track_node.size().y / sf).max(1.0);
    let Some(thumb_h) = find_bound_height(bind.0, &thumb_q, sf) else {
        return;
    };

    let Some(n) = rel.normalized else {
        trigger.propagate(false);
        return;
    };

    let t = (n.y + 0.5).clamp(0.0, 1.0);
    let click_y = t * track_height;
    let desired_top = click_y - thumb_h * 0.5;

    apply_from_track_top_uid(
        bind.0,
        desired_top,
        track_height,
        thumb_h,
        &mut scroll_q,
        &mut thumb_node_q,
        &mut target_scroll_q,
    );

    trigger.propagate(false);
}

fn on_thumb_drag(
    event: On<Pointer<Drag>>,
    parent_q: Query<&ChildOf>,
    track_q: Query<(&ComputedNode, &BindToID), With<ScrollTrack>>,
    thumb_node_q: Query<&ComputedNode, With<ScrollThumb>>,
    mut scroll_q: Query<(&mut Scrollbar, &UIGenID), With<Scrollbar>>,
    mut thumb_q: Query<(&mut Node, &mut ScrollThumb, &BindToID, &mut UiStyle), With<ScrollThumb>>,
    mut target_scroll_q: Query<&mut ScrollPosition>,
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

    let track_height = (track_node.size().y / sf).max(1.0);

    let Ok(thumb_node) = thumb_node_q.get(event.entity) else {
        return;
    };
    let thumb_h = (thumb_node.size().y / sf).max(1.0);
    let half = thumb_h * 0.5;

    let dy = event.event.delta.y / sf;

    let Ok((_, thumb, _, _)) = thumb_q.get(event.entity) else {
        return;
    };
    let current_top = thumb.current_center - half;

    apply_from_track_top_uid(
        bind.0,
        current_top + dy,
        track_height,
        thumb_h,
        &mut scroll_q,
        &mut thumb_q,
        &mut target_scroll_q,
    );
}

fn apply_from_track_top_scroll(
    scroll: &mut Scrollbar,
    ui_id: usize,
    desired_top: f32,
    track_height: f32,
    thumb_height: f32,
    thumb_q: &mut Query<(&mut Node, &mut ScrollThumb, &BindToID, &mut UiStyle), With<ScrollThumb>>,
    target_scroll_q: &mut Query<&mut ScrollPosition>,
) {
    let track_height = track_height.max(1.0);
    let thumb_height = thumb_height.max(1.0);
    let half = thumb_height * 0.5;

    let max_top = (track_height - thumb_height).max(0.0);
    let top = desired_top.clamp(0.0, max_top);

    let percent = if max_top > 0.0 { top / max_top } else { 0.0 };
    let percent = percent.clamp(0.0, 1.0);

    let center = top + half;

    for (mut node, mut thumb, bind, mut style) in thumb_q.iter_mut() {
        if bind.0 != ui_id {
            continue;
        }
        thumb.current_center = center;
        node.top = Val::Px(top);

        for (_, s) in style.styles.iter_mut() {
            s.top = Some(node.top);
            s.left = Some(node.left);
        }
    }

    let range = scroll.max - scroll.min;
    let raw = scroll.min + percent * range;
    let step = scroll.step.max(f32::EPSILON);
    scroll.value = (raw / step).round() * step;

    if let Some(target) = scroll.entity {
        if let Ok(mut pos) = target_scroll_q.get_mut(target) {
            if scroll.vertical {
                pos.y = scroll.value;
            } else {
                pos.x = scroll.value;
            }
        }
    } else if let Some(pos) = scroll.scroll_pos.as_mut() {
        if scroll.vertical {
            pos.y = scroll.value;
        } else {
            pos.x = scroll.value;
        }
    }
}

fn apply_from_track_top_uid(
    target_ui_id: usize,
    desired_top: f32,
    track_height: f32,
    thumb_height: f32,
    scroll_q: &mut Query<(&mut Scrollbar, &UIGenID), With<Scrollbar>>,
    thumb_q: &mut Query<(&mut Node, &mut ScrollThumb, &BindToID, &mut UiStyle), With<ScrollThumb>>,
    target_scroll_q: &mut Query<&mut ScrollPosition>,
) {
    for (mut scroll, ui_id) in scroll_q.iter_mut() {
        if ui_id.0 != target_ui_id {
            continue;
        }
        apply_from_track_top_scroll(
            &mut scroll,
            ui_id.0,
            desired_top,
            track_height,
            thumb_height,
            thumb_q,
            target_scroll_q,
        );
    }
}

fn detect_change_scroll_values(
    mut scroll_q: Query<
        (
            &mut Scrollbar,
            &UIWidgetState,
            &UIGenID,
            &mut PreviousScrollValue,
        ),
        With<Scrollbar>,
    >,
    track_q: Query<(&ComputedNode, &BindToID), With<ScrollTrack>>,
    thumb_q: Query<(&ComputedNode, &BindToID), With<ScrollThumb>>,
    mut thumb_node_q: Query<(&mut Node, &mut ScrollThumb, &BindToID, &mut UiStyle), With<ScrollThumb>>,
    mut target_scroll_q: Query<&mut ScrollPosition>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    for (mut scroll, state, ui_id, mut prev) in scroll_q.iter_mut() {
        if state.focused {
            let step = if shift { scroll.step * 10.0 } else { scroll.step };
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                scroll.value = (scroll.value - step).max(scroll.min);
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                scroll.value = (scroll.value + step).min(scroll.max);
            }
        }

        if scroll.value == **prev {
            continue;
        }
        **prev = scroll.value;

        let track_h = find_bound_height(ui_id.0, &track_q, sf).unwrap_or(1.0);
        let Some(thumb_h) = find_bound_height(ui_id.0, &thumb_q, sf) else {
            continue;
        };

        let max_top = (track_h - thumb_h).max(0.0);
        let percent = ((scroll.value - scroll.min) / (scroll.max - scroll.min)).clamp(0.0, 1.0);
        let top = percent * max_top;

        apply_from_track_top_scroll(
            &mut scroll,
            ui_id.0,
            top,
            track_h,
            thumb_h,
            &mut thumb_node_q,
            &mut target_scroll_q,
        );
    }
}

fn initialize_scroll_visual_state(
    mut commands: Commands,
    mut scroll_q: Query<
        (Entity, &mut Scrollbar, &UIGenID, Option<&ScrollNeedInit>),
        With<Scrollbar>,
    >,
    track_q: Query<(&ComputedNode, &BindToID), With<ScrollTrack>>,
    thumb_q: Query<(&ComputedNode, &BindToID), With<ScrollThumb>>,
    mut thumb_node_q: Query<(&mut Node, &mut ScrollThumb, &BindToID, &mut UiStyle), With<ScrollThumb>>,
    mut target_scroll_q: Query<&mut ScrollPosition>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let sf = window.scale_factor() * ui_scale.0;

    for (entity, mut scroll, ui_id, needs) in scroll_q.iter_mut() {
        if needs.is_none() {
            continue;
        }

        let track_h = find_bound_height(ui_id.0, &track_q, sf).unwrap_or(1.0);
        let Some(thumb_h) = find_bound_height(ui_id.0, &thumb_q, sf) else {
            continue;
        };

        let max_top = (track_h - thumb_h).max(0.0);
        let percent = ((scroll.value - scroll.min) / (scroll.max - scroll.min)).clamp(0.0, 1.0);
        let top = percent * max_top;

        apply_from_track_top_scroll(
            &mut scroll,
            ui_id.0,
            top,
            track_h,
            thumb_h,
            &mut thumb_node_q,
            &mut target_scroll_q,
        );

        commands.entity(entity).remove::<ScrollNeedInit>();
    }
}

fn find_bound_height<Q>(
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
        .map(|(computed, _)| (computed.size().y / scale_factor).max(1.0))
}