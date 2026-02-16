use crate::ExtendedUiConfiguration;
use crate::styles::{CssClass, CssID, CssSource, TagName};
use crate::widgets::{
    BindToID, Body, ToolTip, ToolTipAlignment, ToolTipPriority, ToolTipTrigger, ToolTipVariant,
    UIGenID, UIWidgetState, WidgetId, WidgetKind,
};
use bevy::camera::visibility::RenderLayers;
use bevy::color::Srgba;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform, UiScale};
use bevy::window::PrimaryWindow;
use std::collections::{HashMap, HashSet};

/// Marker component for initialized tooltip widgets.
#[derive(Component)]
struct ToolTipBase;

/// Marker component for tooltip text nodes.
#[derive(Component)]
struct ToolTipText;

/// Marker component for tooltip nose nodes (point variant).
#[derive(Component)]
struct ToolTipNose;

/// Runtime state for tooltip target tracking and visibility animation.
#[derive(Component, Default)]
struct ToolTipRuntime {
    target: Option<Entity>,
    alpha: f32,
}

/// Cached base colors used for fade animation.
#[derive(Component)]
struct ToolTipVisual {
    base_bg: Srgba,
    base_text: Srgba,
    base_nose: Srgba,
    captured: bool,
}

/// Tracks active click/drag trigger states for tooltip targets.
#[derive(Resource, Default)]
struct ToolTipTriggerState {
    clicked: HashSet<Entity>,
    dragging: HashSet<Entity>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ToolTipSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// Plugin that wires tooltip widget behavior.
pub struct ToolTipWidget;

impl Plugin for ToolTipWidget {
    /// Registers systems for tooltip setup and runtime updates.
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolTipTriggerState>();
        app.add_observer(track_click_trigger);
        app.add_observer(track_drag_start_trigger);
        app.add_observer(track_drag_end_trigger);

        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                resolve_tooltip_targets,
                update_tooltip_text,
            )
                .chain(),
        );
        app.add_systems(PostUpdate, update_tooltip_visuals);
    }
}

/// Initializes internal nodes for tooltips.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &ToolTip, Option<&CssSource>),
        (With<ToolTip>, Without<ToolTipBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, ui_id, tooltip, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        let mut node = Node::default();
        node.position_type = PositionType::Absolute;
        node.left = Val::Px(0.0);
        node.top = Val::Px(0.0);
        node.padding = UiRect::axes(Val::Px(10.0), Val::Px(6.0));

        commands
            .entity(entity)
            .insert((
                Name::new(format!("ToolTip-{}", tooltip.entry)),
                node,
                WidgetId {
                    id: tooltip.entry,
                    kind: WidgetKind::ToolTip,
                },
                BackgroundColor(Color::srgba_u8(24, 24, 30, 235)),
                ZIndex(10_000),
                Pickable::IGNORE,
                UiTransform::default(),
                css_source.clone(),
                TagName("tool-tip".to_string()),
                CssClass(vec!["tooltip".to_string()]),
                RenderLayers::layer(*layer),
                ToolTipBase,
                ToolTipRuntime::default(),
                ToolTipVisual {
                    base_bg: Color::srgba_u8(24, 24, 30, 235).to_srgba(),
                    base_text: Color::WHITE.to_srgba(),
                    base_nose: Color::srgba_u8(24, 24, 30, 235).to_srgba(),
                    captured: false,
                },
            ))
            .with_children(|builder| {
                builder.spawn((
                    Name::new(format!("ToolTip-Text-{}", tooltip.entry)),
                    Text::new(tooltip.text.clone()),
                    TextColor(Color::WHITE),
                    TextFont::default(),
                    TextLayout::default(),
                    Pickable::IGNORE,
                    css_source.clone(),
                    CssClass(vec!["tooltip-text".to_string()]),
                    BindToID(ui_id.get()),
                    UIWidgetState::default(),
                    RenderLayers::layer(*layer),
                    ToolTipText,
                ));

                let mut nose_node = Node::default();
                nose_node.position_type = PositionType::Absolute;
                nose_node.width = Val::Px(10.0);
                nose_node.height = Val::Px(10.0);
                nose_node.left = Val::Px(0.0);
                nose_node.top = Val::Px(0.0);

                builder.spawn((
                    Name::new(format!("ToolTip-Nose-{}", tooltip.entry)),
                    nose_node,
                    BackgroundColor(Color::srgba_u8(24, 24, 30, 235)),
                    UiTransform {
                        rotation: Rot2::radians(std::f32::consts::FRAC_PI_4),
                        ..default()
                    },
                    Pickable::IGNORE,
                    css_source.clone(),
                    CssClass(vec!["tooltip-nose".to_string()]),
                    BindToID(ui_id.get()),
                    UIWidgetState::default(),
                    RenderLayers::layer(*layer),
                    Visibility::Hidden,
                    ToolTipNose,
                ));
            });
    }
}

/// Tracks click state used for trigger="click".
fn track_click_trigger(
    ev: On<Pointer<Click>>,
    mut trigger_state: ResMut<ToolTipTriggerState>,
    parents_q: Query<&ChildOf>,
    widget_q: Query<(), With<WidgetId>>,
    body_q: Query<(), With<Body>>,
) {
    let Some(owner) = resolve_owner_widget(ev.event_target(), &parents_q, &widget_q, &body_q) else {
        trigger_state.clicked.clear();
        return;
    };

    if trigger_state.clicked.contains(&owner) {
        trigger_state.clicked.remove(&owner);
    } else {
        trigger_state.clicked.clear();
        trigger_state.clicked.insert(owner);
    }
}

/// Tracks drag start state used for trigger="drag".
fn track_drag_start_trigger(
    ev: On<Pointer<DragStart>>,
    mut trigger_state: ResMut<ToolTipTriggerState>,
    parents_q: Query<&ChildOf>,
    widget_q: Query<(), With<WidgetId>>,
    body_q: Query<(), With<Body>>,
) {
    if let Some(owner) = resolve_owner_widget(ev.event_target(), &parents_q, &widget_q, &body_q) {
        trigger_state.dragging.insert(owner);
    }
}

/// Tracks drag end state used for trigger="drag".
fn track_drag_end_trigger(
    ev: On<Pointer<DragEnd>>,
    mut trigger_state: ResMut<ToolTipTriggerState>,
    parents_q: Query<&ChildOf>,
    widget_q: Query<(), With<WidgetId>>,
    body_q: Query<(), With<Body>>,
) {
    if let Some(owner) = resolve_owner_widget(ev.event_target(), &parents_q, &widget_q, &body_q) {
        trigger_state.dragging.remove(&owner);
    }
}

/// Resolves the nearest widget ancestor for an entity, excluding body.
fn resolve_owner_widget(
    mut current: Entity,
    parents_q: &Query<&ChildOf>,
    widget_q: &Query<(), With<WidgetId>>,
    body_q: &Query<(), With<Body>>,
) -> Option<Entity> {
    loop {
        if body_q.get(current).is_ok() {
            return None;
        }

        if widget_q.get(current).is_ok() {
            return Some(current);
        }

        if let Ok(parent) = parents_q.get(current) {
            current = parent.parent();
        } else {
            return None;
        }
    }
}

/// Resolves tooltip targets from either `for="id"` or the direct parent widget.
fn resolve_tooltip_targets(
    mut tooltip_q: Query<
        (&ToolTip, Option<&ChildOf>, &mut ToolTipRuntime),
        (With<ToolTip>, With<ToolTipBase>),
    >,
    parents_q: Query<&ChildOf>,
    id_q: Query<(Entity, &CssID), Without<ToolTip>>,
    widget_q: Query<(), With<WidgetId>>,
    body_q: Query<(), With<Body>>,
) {
    let mut by_id: HashMap<&str, Entity> = HashMap::new();
    for (entity, css_id) in id_q.iter() {
        if !css_id.0.is_empty() {
            by_id.entry(css_id.0.as_str()).or_insert(entity);
        }
    }

    for (tooltip, parent_opt, mut runtime) in tooltip_q.iter_mut() {
        let target = if let Some(for_id) = tooltip.for_id.as_deref() {
            let normalized = for_id.trim().trim_start_matches('#');
            by_id.get(normalized).copied()
        } else if let Some(parent) = parent_opt {
            let mut current = parent.parent();
            let mut resolved = None;

            loop {
                if body_q.get(current).is_ok() {
                    break;
                }

                if widget_q.get(current).is_ok() {
                    resolved = Some(current);
                    break;
                }

                if let Ok(next) = parents_q.get(current) {
                    current = next.parent();
                } else {
                    break;
                }
            }

            resolved
        } else {
            None
        };

        runtime.target = target;
    }
}

/// Updates tooltip text content when the component changes.
fn update_tooltip_text(
    tooltip_q: Query<(&ToolTip, &Children), (With<ToolTip>, With<ToolTipBase>, Changed<ToolTip>)>,
    mut text_q: Query<&mut Text, With<ToolTipText>>,
) {
    for (tooltip, children) in tooltip_q.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_q.get_mut(child) {
                text.0 = tooltip.text.clone();
            }
        }
    }
}

/// Returns true when any configured trigger mode is active.
fn is_trigger_active(tooltip: &ToolTip, hovered: bool, clicked: bool, dragging: bool) -> bool {
    if tooltip.trigger.is_empty() {
        return hovered;
    }

    tooltip.trigger.iter().any(|trigger| match trigger {
        ToolTipTrigger::Hover => hovered,
        ToolTipTrigger::Click => clicked,
        ToolTipTrigger::Drag => dragging,
    })
}

/// Resolves preferred side from prio + alignment.
fn preferred_side(alignment: ToolTipAlignment, prio: ToolTipPriority) -> ToolTipSide {
    match alignment {
        ToolTipAlignment::Horizontal => match prio {
            ToolTipPriority::Left => ToolTipSide::Left,
            ToolTipPriority::Right => ToolTipSide::Right,
            _ => ToolTipSide::Right,
        },
        ToolTipAlignment::Vertical => match prio {
            ToolTipPriority::Bottom => ToolTipSide::Bottom,
            ToolTipPriority::Top => ToolTipSide::Top,
            _ => ToolTipSide::Top,
        },
    }
}

/// Returns the opposite placement side.
fn opposite_side(side: ToolTipSide) -> ToolTipSide {
    match side {
        ToolTipSide::Top => ToolTipSide::Bottom,
        ToolTipSide::Bottom => ToolTipSide::Top,
        ToolTipSide::Left => ToolTipSide::Right,
        ToolTipSide::Right => ToolTipSide::Left,
    }
}

/// Computes follow placement (cursor-based).
fn place_follow(
    cursor: Vec2,
    tip_w: f32,
    tip_h: f32,
    window_w: f32,
    window_h: f32,
    margin: f32,
    gap: f32,
) -> (f32, f32, ToolTipSide) {
    let mut x = cursor.x + gap;
    let mut y = cursor.y + gap;
    let mut side = ToolTipSide::Right;

    if x + tip_w > window_w - margin {
        x = cursor.x - tip_w - gap;
        side = ToolTipSide::Left;
    }

    x = x.clamp(margin, (window_w - tip_w - margin).max(margin));
    y = y.clamp(margin, (window_h - tip_h - margin).max(margin));

    (x, y, side)
}

/// Computes point placement (target-based, centered at target).
fn place_point(
    target_top_left: Vec2,
    target_size: Vec2,
    tip_w: f32,
    tip_h: f32,
    alignment: ToolTipAlignment,
    prio: ToolTipPriority,
    window_w: f32,
    window_h: f32,
    margin: f32,
    gap: f32,
) -> (f32, f32, ToolTipSide) {
    let center = target_top_left + target_size * 0.5;
    let mut side = preferred_side(alignment, prio);

    let calc = |s: ToolTipSide| -> (f32, f32) {
        match s {
            ToolTipSide::Right => (target_top_left.x + target_size.x + gap, center.y - tip_h * 0.5),
            ToolTipSide::Left => (target_top_left.x - tip_w - gap, center.y - tip_h * 0.5),
            ToolTipSide::Top => (center.x - tip_w * 0.5, target_top_left.y - tip_h - gap),
            ToolTipSide::Bottom => {
                (center.x - tip_w * 0.5, target_top_left.y + target_size.y + gap)
            }
        }
    };

    let (mut x, mut y) = calc(side);

    let side_overflows = match side {
        ToolTipSide::Right => x + tip_w > window_w - margin,
        ToolTipSide::Left => x < margin,
        ToolTipSide::Top => y < margin,
        ToolTipSide::Bottom => y + tip_h > window_h - margin,
    };

    if side_overflows {
        side = opposite_side(side);
        (x, y) = calc(side);
    }

    x = x.clamp(margin, (window_w - tip_w - margin).max(margin));
    y = y.clamp(margin, (window_h - tip_h - margin).max(margin));

    (x, y, side)
}

/// Updates a tooltip nose node so it points towards the target side.
fn place_nose(node: &mut Node, side: ToolTipSide, tip_w: f32, tip_h: f32) {
    let size = 10.0;
    let half = size * 0.5;
    node.position_type = PositionType::Absolute;
    node.width = Val::Px(size);
    node.height = Val::Px(size);

    match side {
        ToolTipSide::Right => {
            node.left = Val::Px(-half);
            node.top = Val::Px((tip_h - size) * 0.5);
        }
        ToolTipSide::Left => {
            node.left = Val::Px(tip_w - half);
            node.top = Val::Px((tip_h - size) * 0.5);
        }
        ToolTipSide::Top => {
            node.left = Val::Px((tip_w - size) * 0.5);
            node.top = Val::Px(tip_h - half);
        }
        ToolTipSide::Bottom => {
            node.left = Val::Px((tip_w - size) * 0.5);
            node.top = Val::Px(-half);
        }
    }
}

/// Positions and animates visible tooltips near the mouse cursor.
fn update_tooltip_visuals(
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    target_state_q: Query<&UIWidgetState>,
    target_geometry_q: Query<(&ComputedNode, &UiGlobalTransform)>,
    trigger_state: Res<ToolTipTriggerState>,
    parent_q: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut tooltip_q: Query<
        (
            &ToolTip,
            &mut Node,
            &mut Visibility,
            &mut BackgroundColor,
            &mut UiTransform,
            Option<&ChildOf>,
            &Children,
            &mut ToolTipRuntime,
            &mut ToolTipVisual,
            Option<&ComputedNode>,
        ),
        (
            With<ToolTip>,
            With<ToolTipBase>,
            Without<ToolTipNose>,
            Without<ToolTipText>,
        ),
    >,
    mut text_q: Query<&mut TextColor, (With<ToolTipText>, Without<ToolTip>, Without<ToolTipNose>)>,
    mut nose_q: Query<
        (&mut Node, &mut BackgroundColor, &mut Visibility),
        (With<ToolTipNose>, Without<ToolTip>, Without<ToolTipText>),
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let cursor_position = window.cursor_position();
    let sf = window.scale_factor() * ui_scale.0;
    let window_width = window.width();
    let window_height = window.height();

    for (
        tooltip,
        mut node,
        mut visibility,
        mut bg_color,
        mut transform,
        parent_opt,
        children,
        mut runtime,
        mut visual,
        computed,
    ) in tooltip_q.iter_mut()
    {
        let Some(target) = runtime.target else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let Ok(target_state) = target_state_q.get(target) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let target_hovered = target_state.hovered && !target_state.disabled;
        let target_clicked = trigger_state.clicked.contains(&target) && !target_state.disabled;
        let target_dragging = trigger_state.dragging.contains(&target) && !target_state.disabled;

        let trigger_active = is_trigger_active(tooltip, target_hovered, target_clicked, target_dragging);
        let needs_cursor = tooltip.variant == ToolTipVariant::Follow;
        let should_show = trigger_active && (!needs_cursor || cursor_position.is_some());

        let speed = 12.0;
        let dt = time.delta_secs();

        if should_show {
            runtime.alpha = (runtime.alpha + dt * speed).min(1.0);
        } else {
            runtime.alpha = (runtime.alpha - dt * speed).max(0.0);
        }

        if !visual.captured {
            visual.base_bg = bg_color.0.to_srgba();
            for child in children.iter() {
                if let Ok(text_color) = text_q.get_mut(child) {
                    visual.base_text = text_color.0.to_srgba();
                }
                if let Ok((_, nose_bg, _)) = nose_q.get_mut(child) {
                    visual.base_nose = nose_bg.0.to_srgba();
                }
            }
            visual.captured = true;
        }

        if runtime.alpha <= 0.01 && !should_show {
            *visibility = Visibility::Hidden;
            for child in children.iter() {
                if let Ok((_, _, mut nose_visibility)) = nose_q.get_mut(child) {
                    *nose_visibility = Visibility::Hidden;
                }
            }
            continue;
        }

        *visibility = Visibility::Inherited;

        let mut parent_top_left = Vec2::ZERO;
        if let Some(parent) = parent_opt {
            if let Ok((parent_node, parent_transform)) = parent_q.get(parent.parent()) {
                let half = parent_node.size() * 0.5;
                parent_top_left = parent_transform.affine().transform_point2(-half) / sf;
            }
        }

        let (tip_w, tip_h) = if let Some(computed) = computed {
            (
                (computed.size().x / sf).max(32.0),
                (computed.size().y / sf).max(18.0),
            )
        } else {
            (180.0, 32.0)
        };

        let margin = 8.0;
        let gap = 12.0;

        let placement = match tooltip.variant {
            ToolTipVariant::Follow => cursor_position.map(|cursor| {
                place_follow(
                    cursor,
                    tip_w,
                    tip_h,
                    window_width,
                    window_height,
                    margin,
                    gap,
                )
            }),
            ToolTipVariant::Point => target_geometry_q.get(target).ok().map(|(target_node, target_transform)| {
                let target_size = target_node.size() / sf;
                let target_top_left = target_transform
                    .affine()
                    .transform_point2(-(target_node.size() * 0.5))
                    / sf;

                place_point(
                    target_top_left,
                    target_size,
                    tip_w,
                    tip_h,
                    tooltip.alignment,
                    tooltip.prio,
                    window_width,
                    window_height,
                    margin,
                    gap,
                )
            }),
        };

        let Some((x, y, side)) = placement else {
            continue;
        };

        node.position_type = PositionType::Absolute;
        node.left = Val::Px(x - parent_top_left.x);
        node.top = Val::Px(y - parent_top_left.y);

        for child in children.iter() {
            if let Ok((mut nose_node, mut nose_bg, mut nose_visibility)) = nose_q.get_mut(child) {
                if tooltip.variant == ToolTipVariant::Point {
                    place_nose(&mut nose_node, side, tip_w, tip_h);
                    *nose_visibility = Visibility::Inherited;
                    nose_bg.0 = Color::srgba(
                        visual.base_nose.red,
                        visual.base_nose.green,
                        visual.base_nose.blue,
                        visual.base_nose.alpha * runtime.alpha,
                    );
                } else {
                    *nose_visibility = Visibility::Hidden;
                }
            }
        }

        let eased = runtime.alpha * runtime.alpha * (3.0 - 2.0 * runtime.alpha);
        transform.scale = Vec2::splat(0.94 + 0.06 * eased);

        bg_color.0 = Color::srgba(
            visual.base_bg.red,
            visual.base_bg.green,
            visual.base_bg.blue,
            visual.base_bg.alpha * eased,
        );

        for child in children.iter() {
            if let Ok(mut text_color) = text_q.get_mut(child) {
                text_color.0 = Color::srgba(
                    visual.base_text.red,
                    visual.base_text.green,
                    visual.base_text.blue,
                    visual.base_text.alpha * eased,
                );
            }
            if let Ok((_, mut nose_bg, _)) = nose_q.get_mut(child) {
                nose_bg.0 = Color::srgba(
                    visual.base_nose.red,
                    visual.base_nose.green,
                    visual.base_nose.blue,
                    visual.base_nose.alpha * eased,
                );
            }
        }
    }
}
