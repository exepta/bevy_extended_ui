use std::collections::HashMap;

use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{BindToID, Div, Scrollbar, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::ScrollPosition;

/// Marker component for initialized div widgets.
#[derive(Component)]
struct DivBase;

/// Marker for the scroll content node inside a div.
#[derive(Component)]
struct DivScrollContent;

/// Component storing the root content entity for a div.
#[derive(Component, Deref)]
struct DivContentRoot(Entity);

/// Component storing the vertical scrollbar entity for a div.
#[derive(Component, Deref)]
struct DivScrollbar(Entity);

/// Component storing the horizontal scrollbar entity for a div.
#[derive(Component, Deref)]
struct DivScrollbarH(Entity);

/// Marks which div owns a scroll-content subtree.
#[derive(Component, Deref)]
struct DivContentOwner(Entity);

/// Marks which div owns a scrollbar overlay.
#[derive(Component, Deref)]
struct DivScrollbarOwner(Entity);

/// Tracks hover counts for div widgets.
#[derive(Resource, Default)]
struct HoveredDivTracker {
    div_ref: HashMap<Entity, u32>,
    last_div: Option<Entity>,
}

/// Plugin that wires up div widget behavior.
pub struct DivWidget;

impl Plugin for DivWidget {
    /// Registers systems for div widget setup and scrolling.
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredDivTracker>();

        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                ensure_div_scroll_structure,
                route_hover_from_pointer_messages,
                handle_div_scroll_wheel,
                sync_scrollbar_from_content,
            )
                .chain(),
        );
    }
}

/// Creates the base node for div widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Div, Option<&CssSource>), (With<Div>, Without<DivBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, div, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Div-{}", div.0)),
                Node::default(),
                WidgetId {
                    id: div.0,
                    kind: WidgetKind::Div,
                },
                ImageNode::default(),
                BackgroundColor::default(),
                BorderColor::default(),
                BoxShadow::new(
                    Colored::TRANSPARENT,
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                ),
                ZIndex::default(),
                Pickable::default(),
                css_source,
                TagName("div".to_string()),
                RenderLayers::layer(*layer),
                DivBase,
                UIWidgetState::default(),
            ))
            .observe(on_div_click);
    }
}

/// Ensures the div has scroll content and scrollbar nodes when needed.
fn ensure_div_scroll_structure(
    mut commands: Commands,
    mut div_q: Query<
        (
            Entity,
            &Div,
            &UIGenID,
            &mut Node,
            Option<&Children>,
            Option<&CssSource>,
            Option<&CssClass>,
            Option<&RenderLayers>,
            Option<&DivContentRoot>,
            Option<&DivScrollbar>,
            Option<&DivScrollbarH>,
        ),
        (With<Div>, With<DivBase>),
    >,
    parent_q: Query<&ChildOf>,
    mut content_node_q: Query<&mut Node, (With<DivScrollContent>, Without<DivBase>)>,
    has_scroll_pos_q: Query<(), With<ScrollPosition>>,
) {
    for (
        div_entity,
        div,
        ui_id,
        mut div_node,
        children_opt,
        source_opt,
        wrapper_class_opt,
        layers_opt,
        root_opt,
        sb_y_opt,
        sb_x_opt,
    ) in div_q.iter_mut()
    {
        let mut wants_scroll_y = div_node.overflow.y == OverflowAxis::Scroll;
        let mut wants_scroll_x = div_node.overflow.x == OverflowAxis::Scroll;

        if let Some(root) = root_opt {
            if let Ok(content_node) = content_node_q.get(**root) {
                wants_scroll_y = content_node.overflow.y == OverflowAxis::Scroll;
                wants_scroll_x = content_node.overflow.x == OverflowAxis::Scroll;
            }
        }

        if !(wants_scroll_y || wants_scroll_x || root_opt.is_some()) {
            continue;
        }

        // 1) Ensure scroll-content child
        let content_entity = if let Some(root) = root_opt {
            **root
        } else {
            let css_source = source_opt.cloned().unwrap_or_default();

            let mut content_node = Node::default();
            content_node.width = Val::Percent(100.0);
            content_node.height = Val::Percent(100.0);
            content_node.overflow.y = if wants_scroll_y {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            };
            content_node.overflow.x = if wants_scroll_x {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            };

            let mut classes: Vec<String> =
                wrapper_class_opt.map(|c| c.0.clone()).unwrap_or_default();
            classes.push("div-scroll-content".to_string());

            let content_entity = commands
                .spawn((
                    Name::new(format!("Div-ScrollContent-{}", div.0)),
                    content_node,
                    css_source,
                    CssClass(classes),
                    TagName("div".to_string()),
                    DivScrollContent,
                    DivContentOwner(div_entity),
                    BindToID(ui_id.get()),
                    Visibility::Inherited,
                    InheritedVisibility::default(),
                    Transform::default(),
                    GlobalTransform::default(),
                    ScrollPosition::default(),
                    UIWidgetState::default(),
                    Pickable::default(),
                ))
                .id();

            if let Some(layers) = layers_opt {
                commands.entity(content_entity).insert(layers.clone());
            }

            commands.entity(div_entity).add_child(content_entity);
            commands
                .entity(div_entity)
                .insert(DivContentRoot(content_entity));

            content_entity
        };
        commands
            .entity(content_entity)
            .insert(BindToID(ui_id.get()));

        let mut sb_y_entity = sb_y_opt.map(|s| **s);
        let mut sb_x_entity = sb_x_opt.map(|s| **s);

        // 2a) Ensure vertical scrollbar
        if wants_scroll_y && sb_y_opt.is_none() {
            let css_source = source_opt.cloned().unwrap_or_default();

            let mut sb_node = Node::default();
            sb_node.position_type = PositionType::Absolute;
            sb_node.left = Val::Auto;
            sb_node.right = Val::Px(0.0);
            sb_node.top = Val::Px(0.0);
            sb_node.bottom = Val::Px(0.0);
            sb_node.width = Val::Px(12.0);

            let sb_entity = commands
                .spawn((
                    Name::new(format!("Div-Scrollbar-{}", div.0)),
                    sb_node,
                    css_source.clone(),
                    CssClass(vec![
                        "scrollbar".to_string(),
                        "scrollbar-vertical".to_string(),
                    ]),
                    TagName("scroll".to_string()),
                    DivScrollbarOwner(div_entity),
                    Scrollbar {
                        vertical: true,
                        min: 0.0,
                        max: 0.0,
                        value: 0.0,
                        step: 1.0,
                        entity: Some(content_entity),
                        ..default()
                    },
                    UIWidgetState::default(),
                    ZIndex(10),
                    ImageNode::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    Pickable::default(),
                ))
                .id();

            if let Some(layers) = layers_opt {
                commands.entity(sb_entity).insert(layers.clone());
            }

            commands.entity(div_entity).add_child(sb_entity);
            commands.entity(div_entity).insert(DivScrollbar(sb_entity));
            sb_y_entity = Some(sb_entity);
        }

        // 2b) Ensure horizontal scrollbar
        if wants_scroll_x && sb_x_opt.is_none() {
            let css_source = source_opt.cloned().unwrap_or_default();

            let mut sb_node = Node::default();
            sb_node.position_type = PositionType::Absolute;
            sb_node.left = Val::Px(0.0);
            sb_node.right = Val::Px(0.0);
            sb_node.bottom = Val::Px(0.0);
            sb_node.top = Val::Auto;
            sb_node.height = Val::Px(12.0);

            let sb_entity = commands
                .spawn((
                    Name::new(format!("Div-Scrollbar-H-{}", div.0)),
                    sb_node,
                    css_source,
                    CssClass(vec![
                        "scrollbar".to_string(),
                        "scrollbar-horizontal".to_string(),
                        "scroll-horizontal".to_string(),
                    ]),
                    TagName("scroll".to_string()),
                    DivScrollbarOwner(div_entity),
                    Scrollbar {
                        vertical: false,
                        min: 0.0,
                        max: 0.0,
                        value: 0.0,
                        step: 1.0,
                        entity: Some(content_entity),
                        ..default()
                    },
                    UIWidgetState::default(),
                    ZIndex(10),
                    ImageNode::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    Pickable::default(),
                ))
                .id();

            if let Some(layers) = layers_opt {
                commands.entity(sb_entity).insert(layers.clone());
            }

            commands.entity(div_entity).add_child(sb_entity);
            commands.entity(div_entity).insert(DivScrollbarH(sb_entity));
            sb_x_entity = Some(sb_entity);
        }

        // 3) Reparent wrapper children under content (only if needed)
        if let Some(children) = children_opt {
            let sb_y = sb_y_opt.map(|s| **s);
            let sb_x = sb_x_opt.map(|s| **s);

            let list: Vec<Entity> = children.iter().clone().collect();
            for child in list {
                if child == content_entity {
                    continue;
                }
                if Some(child) == sb_y || Some(child) == sb_x {
                    continue;
                }

                if let Ok(parent) = parent_q.get(child) {
                    if parent.parent() == content_entity {
                        continue;
                    }
                }

                commands.entity(child).set_parent_in_place(content_entity);
            }
        }

        // 3b) Ensure the div's direct children order keeps scroll content first
        // This mirrors the manual reordering workaround observed in the inspector.
        if let Some(children) = children_opt {
            let mut rest: Vec<Entity> = children
                .iter()
                .clone()
                .filter(|c| {
                    *c != content_entity
                        && Some(*c) != sb_y_opt.map(|s| **s)
                        && Some(*c) != sb_x_opt.map(|s| **s)
                })
                .collect();

            let mut desired = Vec::with_capacity(children.len());
            desired.push(content_entity);
            if let Some(sb) = sb_y_entity {
                desired.push(sb);
            }
            if let Some(sb) = sb_x_entity {
                desired.push(sb);
            }
            desired.append(&mut rest);

            if !children.iter().clone().eq(desired.iter().copied()) {
                commands.entity(div_entity).add_children(&desired);
            }
        }

        // Wrapper clips but does not scroll
        div_node.overflow.y = OverflowAxis::Clip;
        div_node.overflow.x = OverflowAxis::Clip;

        // Ensure content scroll flags stay aligned
        if let Ok(mut node) = content_node_q.get_mut(content_entity) {
            node.overflow.y = if wants_scroll_y {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            };
            node.overflow.x = if wants_scroll_x {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            };
            node.width = Val::Percent(100.0);
            node.height = Val::Percent(100.0);
        }

        if has_scroll_pos_q.get(content_entity).is_err() {
            commands
                .entity(content_entity)
                .insert(ScrollPosition::default());
        }

        commands.entity(content_entity).insert((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Inherited,
            InheritedVisibility::default(),
        ));
    }
}

/// Routes hover state to the owning div.
fn route_hover_from_pointer_messages(
    mut over: MessageReader<Pointer<Over>>,
    mut out: MessageReader<Pointer<Out>>,
    parents: Query<&ChildOf>,
    is_div_q: Query<(), With<Div>>,
    scroll_owner_q: Query<&DivContentOwner, With<DivScrollContent>>,
    sb_owner_q: Query<&DivScrollbarOwner>,
    is_scroll_content_q: Query<(), With<DivScrollContent>>,
    mut div_state_q: Query<&mut UIWidgetState, With<Div>>,
    mut hovered: ResMut<HoveredDivTracker>,
) {
    /// Walks up the hierarchy to find the owning div entity.
    fn find_owner_div(
        mut e: Entity,
        parents: &Query<&ChildOf>,
        is_div_q: &Query<(), With<Div>>,
        is_scroll_content_q: &Query<(), With<DivScrollContent>>,
        scroll_owner_q: &Query<&DivContentOwner, With<DivScrollContent>>,
        sb_owner_q: &Query<&DivScrollbarOwner>,
    ) -> Option<Entity> {
        loop {
            if let Ok(owner) = sb_owner_q.get(e) {
                return Some(**owner);
            }

            if is_div_q.get(e).is_ok() {
                return Some(e);
            }

            if is_scroll_content_q.get(e).is_ok() {
                if let Ok(owner) = scroll_owner_q.get(e) {
                    return Some(**owner);
                }
            }

            if let Ok(p) = parents.get(e) {
                e = p.parent();
            } else {
                return None;
            }
        }
    }

    for msg in over.read() {
        let Some(div) = find_owner_div(
            msg.entity,
            &parents,
            &is_div_q,
            &is_scroll_content_q,
            &scroll_owner_q,
            &sb_owner_q,
        ) else {
            continue;
        };

        let d = hovered.div_ref.entry(div).or_insert(0);
        *d = d.saturating_add(1);
        hovered.last_div = Some(div);

        if let Ok(mut state) = div_state_q.get_mut(div) {
            state.hovered = true;
        }
    }

    for msg in out.read() {
        let Some(div) = find_owner_div(
            msg.entity,
            &parents,
            &is_div_q,
            &is_scroll_content_q,
            &scroll_owner_q,
            &sb_owner_q,
        ) else {
            continue;
        };

        if let Some(d) = hovered.div_ref.get_mut(&div) {
            *d = d.saturating_sub(1);
            if *d == 0 {
                hovered.div_ref.remove(&div);

                if let Ok(mut state) = div_state_q.get_mut(div) {
                    state.hovered = false;
                }

                if hovered.last_div == Some(div) {
                    hovered.last_div = hovered.div_ref.keys().next().copied();
                }
            }
        }
    }
}

/// Wheel scroll uses the "last hovered div" and scrolls its DivScrollContent (Y only).
/// Handles mouse wheel scrolling for div content.
fn handle_div_scroll_wheel(
    mut wheel_events: MessageReader<MouseWheel>,
    hovered: Res<HoveredDivTracker>,
    div_q: Query<(&DivContentRoot, &Visibility), With<Div>>,
    mut content_q: Query<(&Node, &ComputedNode, &mut ScrollPosition), With<DivScrollContent>>,
) {
    let Some(active_div) = hovered.last_div else {
        wheel_events.clear();
        return;
    };

    let Ok((root, vis)) = div_q.get(active_div) else {
        return;
    };

    if !matches!(*vis, Visibility::Visible | Visibility::Inherited) {
        return;
    }

    for event in wheel_events.read() {
        let Ok((node, computed, mut scroll)) = content_q.get_mut(**root) else {
            continue;
        };

        if node.overflow.y != OverflowAxis::Scroll {
            continue;
        }

        let inv_sf = computed.inverse_scale_factor.max(f32::EPSILON);
        let delta = -wheel_delta_y(event, inv_sf);

        let viewport_h = (computed.size().y * inv_sf).max(1.0);
        let content_h = (computed.content_size.y * inv_sf).max(viewport_h);
        let max_scroll = (content_h - viewport_h).max(0.0);

        scroll.y = (scroll.y + delta).clamp(0.0, max_scroll);
    }
}

fn wheel_delta_y(event: &MouseWheel, inv_scale_factor: f32) -> f32 {
    match event.unit {
        MouseScrollUnit::Line => {
            let line_delta = event.y;
            if line_delta.abs() > 10.0 {
                line_delta * inv_scale_factor
            } else {
                line_delta * 25.0
            }
        }
        MouseScrollUnit::Pixel => event.y * inv_scale_factor,
    }
}

/// Synchronizes scrollbar values from scrollable content.
fn sync_scrollbar_from_content(
    div_q: Query<
        (
            Option<&DivContentRoot>,
            Option<&DivScrollbar>,
            Option<&DivScrollbarH>,
        ),
        With<Div>,
    >,
    content_q: Query<&ComputedNode, With<DivScrollContent>>,
    mut scroll_q: Query<&mut Scrollbar>,
    target_pos_q: Query<&ScrollPosition, With<DivScrollContent>>,
    mut sb_node_q: Query<&mut Node, With<Scrollbar>>,
    mut sb_vis_q: Query<&mut Visibility, With<Scrollbar>>,
) {
    for (root_opt, sb_y_opt, sb_x_opt) in div_q.iter() {
        let Some(root) = root_opt else {
            continue;
        };

        let Ok(content_comp) = content_q.get(**root) else {
            continue;
        };
        let Ok(scroll_pos) = target_pos_q.get(**root) else {
            continue;
        };

        let inv_sf = content_comp.inverse_scale_factor.max(f32::EPSILON);
        let viewport = content_comp.size() * inv_sf;
        let content = content_comp.content_size * inv_sf;

        // -------- Y --------
        if let Some(sb) = sb_y_opt {
            let viewport_h = viewport.y.max(1.0);
            let content_h = content.y.max(viewport_h);
            let max_scroll = (content_h - viewport_h).max(0.0);

            check_scroll_bar_state(&mut sb_node_q, &mut sb_vis_q, sb, max_scroll);

            if let Ok(mut scrollbar) = scroll_q.get_mut(**sb) {
                scrollbar.min = 0.0;
                scrollbar.max = max_scroll;
                scrollbar.viewport_extent = viewport_h;
                scrollbar.content_extent = content_h;

                scrollbar.value = scroll_pos.y.clamp(0.0, max_scroll);
            }
        }

        // -------- X --------
        if let Some(sb) = sb_x_opt {
            let viewport_w = viewport.x.max(1.0);
            let content_w = content.x.max(viewport_w);
            let max_scroll = (content_w - viewport_w).max(0.0);

            check_scroll_bar_state(&mut sb_node_q, &mut sb_vis_q, sb, max_scroll);

            if let Ok(mut scrollbar) = scroll_q.get_mut(**sb) {
                scrollbar.min = 0.0;
                scrollbar.max = max_scroll;
                scrollbar.viewport_extent = viewport_w;
                scrollbar.content_extent = content_w;

                scrollbar.value = scroll_pos.x.clamp(0.0, max_scroll);
            }
        }
    }
}

/// Updates scrollbar state if content dimensions have changed.
fn check_scroll_bar_state<T>(
    sb_node_q: &mut Query<&mut Node, With<Scrollbar>>,
    sb_vis_q: &mut Query<&mut Visibility, With<Scrollbar>>,
    sb: &T,
    max_scroll: f32,
) where
    T: std::ops::Deref<Target = Entity>,
{
    if let Ok(mut sb_node) = sb_node_q.get_mut(**sb) {
        if sb_node.display != Display::Flex {
            sb_node.display = Display::Flex;
        }
    }

    if let Ok(mut vis) = sb_vis_q.get_mut(**sb) {
        let want_visible = max_scroll > 0.5;
        let new_vis = if want_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != new_vis {
            *vis = new_vis;
        }
    }
}
// -------------------- Events --------------------

/// Sets focus on div click and updates the current widget state.
fn on_div_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Div>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
    trigger.propagate(false);
}
