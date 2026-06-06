use crate::styles::CssClass;
use crate::widgets::{ActiveScrollTarget, BindToID, Body, Scrollbar, WidgetId};
use bevy::ecs::query::QueryFilter;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, OverflowAxis, ScrollPosition, UiGlobalTransform};
use std::ops::Deref;

/// Handles `wheel_delta_axis` in the extended UI workflow.
fn wheel_delta_axis(value: f32, unit: MouseScrollUnit, inv_scale_factor: f32) -> f32 {
    match unit {
        MouseScrollUnit::Line => {
            if value.abs() > 10.0 {
                value * inv_scale_factor
            } else {
                value * 25.0
            }
        }
        MouseScrollUnit::Pixel => value * inv_scale_factor,
    }
}

#[inline]
fn parent_entity(entity: Entity, parents: &Query<&ChildOf>) -> Option<Entity> {
    parents.get(entity).ok().map(|parent| parent.parent())
}

/// Resolves the nearest widget ancestor for an entity, excluding body.
pub(crate) fn resolve_owner_widget(
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

        current = parent_entity(current, parents_q)?;
    }
}

/// Resolves the nearest widget ancestor from an optional parent entity.
pub(crate) fn resolve_owner_widget_from_parent(
    parent_opt: Option<&ChildOf>,
    parents_q: &Query<&ChildOf>,
    widget_q: &Query<(), With<WidgetId>>,
    body_q: &Query<(), With<Body>>,
) -> Option<Entity> {
    parent_opt.and_then(|parent| resolve_owner_widget(parent.parent(), parents_q, widget_q, body_q))
}

/// Applies base + dynamic CSS classes while avoiding duplicates.
pub(crate) fn set_css_classes(classes: &mut CssClass, base: &[String], dynamic: &[&str]) {
    let mut next = base.to_vec();
    for class in dynamic {
        if !next.iter().any(|existing| existing == class) {
            next.push((*class).to_string());
        }
    }

    if classes.0 != next {
        classes.0 = next;
    }
}

/// Returns the top-left world position of a badge/tooltip parent in UI units.
pub(crate) fn resolve_parent_top_left(
    parent_opt: Option<&ChildOf>,
    parent_q: &Query<(&ComputedNode, &UiGlobalTransform)>,
    scale_factor: f32,
) -> Vec2 {
    let Some(parent) = parent_opt else {
        return Vec2::ZERO;
    };

    let Ok((parent_node, parent_transform)) = parent_q.get(parent.parent()) else {
        return Vec2::ZERO;
    };

    let half = parent_node.size() * 0.5;
    parent_transform.affine().transform_point2(-half) / scale_factor
}

/// Resolves a root owner by climbing parents while supporting scroll-content and scrollbar owners.
pub(crate) fn find_owner_with_scroll_and_bar<
    RootMarker: Component,
    ScrollContentMarker: Component,
    ScrollOwner: Component + Deref<Target = Entity>,
    ScrollbarOwner: Component + Deref<Target = Entity>,
>(
    mut entity: Entity,
    parents_q: &Query<&ChildOf>,
    is_root_q: &Query<(), With<RootMarker>>,
    is_scroll_content_q: &Query<(), With<ScrollContentMarker>>,
    scroll_owner_q: &Query<&ScrollOwner, With<ScrollContentMarker>>,
    scrollbar_owner_q: &Query<&ScrollbarOwner>,
) -> Option<Entity> {
    loop {
        if let Ok(owner) = scrollbar_owner_q.get(entity) {
            return Some(**owner);
        }

        if is_root_q.get(entity).is_ok() {
            return Some(entity);
        }

        if is_scroll_content_q.get(entity).is_ok() {
            if let Ok(owner) = scroll_owner_q.get(entity) {
                return Some(**owner);
            }
        }

        if let Ok(parent) = parents_q.get(entity) {
            entity = parent.parent();
        } else {
            return None;
        }
    }
}

/// Computes max scroll distance for viewport/content extents in UI units.
pub(crate) fn max_scroll_for_extents(viewport_extent: f32, content_extent: f32) -> f32 {
    let viewport = viewport_extent.max(1.0);
    let content = content_extent.max(viewport);
    (content - viewport).max(0.0)
}

/// Computes max scroll distance from pixel extents and inverse scale factor.
pub(crate) fn max_scroll_for_axis(
    viewport_extent_px: f32,
    content_extent_px: f32,
    inverse_scale_factor: f32,
) -> f32 {
    let viewport = viewport_extent_px * inverse_scale_factor;
    let content = content_extent_px * inverse_scale_factor;
    max_scroll_for_extents(viewport, content)
}

/// Returns true when an axis is scrollable and has effective scroll range.
pub(crate) fn can_scroll_axis(
    overflow_axis: OverflowAxis,
    viewport_extent_px: f32,
    content_extent_px: f32,
    inverse_scale_factor: f32,
) -> bool {
    overflow_axis == OverflowAxis::Scroll
        && max_scroll_for_axis(viewport_extent_px, content_extent_px, inverse_scale_factor) > 0.5
}

/// Applies a scroll delta to one axis with clamping based on viewport/content extents.
pub(crate) fn apply_scroll_delta(
    current: &mut f32,
    delta: f32,
    viewport_extent_px: f32,
    content_extent_px: f32,
    inverse_scale_factor: f32,
) {
    if delta.abs() <= f32::EPSILON {
        return;
    }

    let max_scroll =
        max_scroll_for_axis(viewport_extent_px, content_extent_px, inverse_scale_factor);
    *current = (*current + delta).clamp(0.0, max_scroll);
}

/// Applies one wheel event to a scrollable node on both axes (including Shift->X behavior).
pub(crate) fn apply_wheel_scroll_event(
    event: &MouseWheel,
    keyboard: Option<&ButtonInput<KeyCode>>,
    node: &Node,
    computed: &ComputedNode,
    scroll: &mut ScrollPosition,
) {
    let can_scroll_y = node.overflow.y == OverflowAxis::Scroll;
    let can_scroll_x = node.overflow.x == OverflowAxis::Scroll;
    if !can_scroll_x && !can_scroll_y {
        return;
    }

    let inv_sf = computed.inverse_scale_factor.max(f32::EPSILON);
    let shift = keyboard
        .is_some_and(|keys| keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight));

    let mut delta_x = -wheel_delta_x(event, inv_sf);
    let mut delta_y = -wheel_delta_y(event, inv_sf);

    if shift && delta_x.abs() <= f32::EPSILON && can_scroll_x {
        delta_x = delta_y;
        delta_y = 0.0;
    }

    if can_scroll_y {
        apply_scroll_delta(
            &mut scroll.y,
            delta_y,
            computed.size().y,
            computed.content_size.y,
            inv_sf,
        );
    }

    if can_scroll_x {
        apply_scroll_delta(
            &mut scroll.x,
            delta_x,
            computed.size().x,
            computed.content_size.x,
            inv_sf,
        );
    }
}

/// Applies all queued wheel events to one specific scroll root entity.
pub(crate) fn apply_wheel_scroll_events_for_root<F: QueryFilter>(
    root: Entity,
    wheel_events: &mut MessageReader<MouseWheel>,
    keyboard: Option<&ButtonInput<KeyCode>>,
    content_q: &mut Query<(&Node, &ComputedNode, &mut ScrollPosition), F>,
) {
    for event in wheel_events.read() {
        let Ok((node, computed, mut scroll)) = content_q.get_mut(root) else {
            continue;
        };

        apply_wheel_scroll_event(event, keyboard, node, computed, &mut scroll);
    }
}

/// Finds the nearest ancestor that contains the given marker component.
pub(crate) fn find_ancestor_with_component<Marker: Component>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    markers: &Query<(), With<Marker>>,
) -> Option<Entity> {
    while let Some(ancestor) = parent_entity(entity, parents) {
        if markers.get(ancestor).is_ok() {
            return Some(ancestor);
        }
        entity = ancestor;
    }
    None
}

/// Ensures scrollbar node display and visibility match current scroll range.
pub(crate) fn set_scrollbar_display_and_visibility<T>(
    scrollbar_node_query: &mut Query<&mut Node, With<Scrollbar>>,
    scrollbar_visibility_query: &mut Query<&mut Visibility, With<Scrollbar>>,
    scrollbar_owner: &T,
    max_scroll: f32,
) where
    T: Deref<Target = Entity>,
{
    if let Ok(mut scrollbar_node) = scrollbar_node_query.get_mut(**scrollbar_owner) {
        if scrollbar_node.display != Display::Flex {
            scrollbar_node.display = Display::Flex;
        }
    }

    if let Ok(mut visibility) = scrollbar_visibility_query.get_mut(**scrollbar_owner) {
        let want_visible = max_scroll > 0.5;
        let next_visibility = if want_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next_visibility {
            *visibility = next_visibility;
        }
    }
}

/// Returns the first child's logical height for a given parent, if present.
pub(crate) fn first_child_logical_height<F: QueryFilter>(
    parent: Entity,
    query: &Query<(&ComputedNode, &ChildOf), F>,
    min_height: f32,
) -> Option<f32> {
    for (computed, child_of) in query.iter() {
        if child_of.parent() == parent {
            let inverse_scale_factor = computed.inverse_scale_factor.max(f32::EPSILON);
            return Some((computed.size().y * inverse_scale_factor).max(min_height));
        }
    }
    None
}

/// Sets a z-index pair based on open/closed state.
pub(crate) fn set_z_index_pair(
    z_index: &mut ZIndex,
    global_z_index: &mut GlobalZIndex,
    is_open: bool,
    open_z: i32,
) {
    let value = if is_open { open_z } else { 0 };
    z_index.0 = value;
    global_z_index.0 = value;
}

/// Sets visibility and z-index pair based on open/closed state.
pub(crate) fn set_overlay_visibility_and_z(
    visibility: &mut Visibility,
    z_index: &mut ZIndex,
    global_z_index: &mut GlobalZIndex,
    is_open: bool,
    open_z: i32,
) {
    *visibility = if is_open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    set_z_index_pair(z_index, global_z_index, is_open, open_z);
}

/// Applies overlay visibility/z-index state for all nodes bound to a widget id.
pub(crate) fn apply_overlay_state_for_bind<F: QueryFilter>(
    bind_id: usize,
    is_open: bool,
    open_z: i32,
    query: &mut Query<(&mut Visibility, &mut ZIndex, &mut GlobalZIndex, &BindToID), F>,
) {
    for (mut visibility, mut z_index, mut global_z_index, bind_to_id) in query.iter_mut() {
        if bind_to_id.0 != bind_id {
            continue;
        }
        set_overlay_visibility_and_z(
            &mut visibility,
            &mut z_index,
            &mut global_z_index,
            is_open,
            open_z,
        );
    }
}

/// Clears the active scroll target when it matches the given entity.
pub(crate) fn clear_active_scroll_target_for_entity(
    active_scroll_target: &mut ActiveScrollTarget,
    entity: Entity,
) {
    if active_scroll_target.entity == Some(entity) {
        active_scroll_target.entity = None;
    }
}

/// Handles `wheel_delta_x` in the extended UI workflow.
///
/// # Examples
///
/// ```rust
/// // Call `wheel_delta_x` with values from your app state and world context.
/// ```
pub fn wheel_delta_x(event: &MouseWheel, inv_scale_factor: f32) -> f32 {
    wheel_delta_axis(event.x, event.unit, inv_scale_factor)
}

/// Handles `wheel_delta_y` in the extended UI workflow.
///
/// # Examples
///
/// ```rust
/// // Call `wheel_delta_y` with values from your app state and world context.
/// ```
pub fn wheel_delta_y(event: &MouseWheel, inv_scale_factor: f32) -> f32 {
    wheel_delta_axis(event.y, event.unit, inv_scale_factor)
}
