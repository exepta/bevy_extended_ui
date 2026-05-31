use std::collections::HashMap;

use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::widget_util::{
    apply_wheel_scroll_events_for_root, find_owner_with_scroll_and_bar, max_scroll_for_extents,
    set_scrollbar_display_and_visibility,
};
use crate::widgets::{
    ActiveScrollTarget, BindToID, Div, Scrollbar, UIGenID, UIWidgetState, WidgetId, WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::MouseWheel;
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
pub struct DivContentRoot(pub Entity);

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
    div_with_scroll_q: Query<&DivContentRoot, With<Div>>,
    mut hovered: ResMut<HoveredDivTracker>,
) {
    /// Finds all ancestor Div entities up the parent chain.
    fn find_all_ancestor_divs(
        mut e: Entity,
        parents: &Query<&ChildOf>,
        is_div_q: &Query<(), With<Div>>,
    ) -> Vec<Entity> {
        let mut ancestors = Vec::new();
        while let Ok(p) = parents.get(e) {
            e = p.parent();
            if is_div_q.get(e).is_ok() {
                ancestors.push(e);
            }
        }
        ancestors
    }

    /// Decrements hover ref-count and updates state when it reaches zero.
    fn decrement_div_hover(
        target: Entity,
        div_ref: &mut HashMap<Entity, u32>,
        last_div: &mut Option<Entity>,
        div_state_q: &mut Query<&mut UIWidgetState, With<Div>>,
    ) {
        if let Some(depth) = div_ref.get_mut(&target) {
            *depth = depth.saturating_sub(1);
            if *depth == 0 {
                div_ref.remove(&target);

                if let Ok(mut state) = div_state_q.get_mut(target) {
                    state.hovered = false;
                }

                if *last_div == Some(target) {
                    *last_div = div_ref.keys().next().copied();
                }
            }
        }
    }

    for msg in over.read() {
        let Some(div) = find_owner_with_scroll_and_bar(
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

        if let Ok(mut state) = div_state_q.get_mut(div) {
            state.hovered = true;
        }

        // Also mark all ancestor divs as hovered for nested div support
        let ancestors = find_all_ancestor_divs(div, &parents, &is_div_q);

        // Find the closest (deepest/most recent) scrollable div in the hierarchy for last_div.
        // Prefer scrollable ancestors over non-scrollable children for wheel event handling.
        let mut scrollable_div = None;
        if div_with_scroll_q.get(div).is_ok() {
            scrollable_div = Some(div);
        }
        for ancestor_div in &ancestors {
            let d = hovered.div_ref.entry(*ancestor_div).or_insert(0);
            *d = d.saturating_add(1);

            if let Ok(mut state) = div_state_q.get_mut(*ancestor_div) {
                state.hovered = true;
            }

            // Use the first scrollable ancestor found (closest parent)
            if scrollable_div.is_none() && div_with_scroll_q.get(*ancestor_div).is_ok() {
                scrollable_div = Some(*ancestor_div);
            }
        }

        hovered.last_div = scrollable_div.or(Some(div));
    }

    for msg in out.read() {
        let Some(div) = find_owner_with_scroll_and_bar(
            msg.entity,
            &parents,
            &is_div_q,
            &is_scroll_content_q,
            &scroll_owner_q,
            &sb_owner_q,
        ) else {
            continue;
        };

        let HoveredDivTracker { div_ref, last_div } = &mut *hovered;
        decrement_div_hover(div, div_ref, last_div, &mut div_state_q);

        // Also unmark ancestor divs
        let ancestors = find_all_ancestor_divs(div, &parents, &is_div_q);
        for ancestor_div in ancestors {
            decrement_div_hover(ancestor_div, div_ref, last_div, &mut div_state_q);
        }
    }
}

/// Wheel scroll uses the "last hovered div" and scrolls its DivScrollContent on both axes.
/// Handles mouse wheel scrolling for div content.
fn handle_div_scroll_wheel(
    mut wheel_events: MessageReader<MouseWheel>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    active_scroll_target: Res<ActiveScrollTarget>,
    hovered: Res<HoveredDivTracker>,
    div_q: Query<(&DivContentRoot, &Visibility), With<Div>>,
    mut content_q: Query<(&Node, &ComputedNode, &mut ScrollPosition), With<DivScrollContent>>,
) {
    if active_scroll_target.entity.is_some() {
        wheel_events.clear();
        return;
    }

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

    apply_wheel_scroll_events_for_root(
        **root,
        &mut wheel_events,
        keyboard.as_deref(),
        &mut content_q,
    );
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
            let max_scroll = max_scroll_for_extents(viewport_h, content_h);

            set_scrollbar_display_and_visibility(&mut sb_node_q, &mut sb_vis_q, sb, max_scroll);

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
            let max_scroll = max_scroll_for_extents(viewport_w, content_w);

            set_scrollbar_display_and_visibility(&mut sb_node_q, &mut sb_vis_q, sb, max_scroll);

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
