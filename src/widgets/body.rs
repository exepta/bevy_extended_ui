use std::collections::HashMap;

use crate::html::HtmlStyle;
use crate::registry::UiRegistry;
use crate::styles::{CssClass, CssSource, Style, TagName};
use crate::widgets::controls::choice_box::ChoiceLayoutBoxBase;
use crate::widgets::div::DivContentRoot;
use crate::widgets::widget_util::wheel_delta_y;
use crate::widgets::{
    BindToID, Body, Div, Scrollbar, UIGenID, UIWidgetState, WidgetId, WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ScrollPosition;

/// Marker component for the internal body node.
#[derive(Component)]
struct BodyBase;

/// Marker for the scroll content node inside a body.
#[derive(Component)]
struct BodyScrollContent;

/// Component storing the root content entity for a body.
#[derive(Component, Deref)]
struct BodyContentRoot(Entity);

/// Component storing the vertical scrollbar entity for a body.
#[derive(Component, Deref)]
struct BodyScrollbar(Entity);

/// Component storing the horizontal scrollbar entity for a body.
#[derive(Component, Deref)]
struct BodyScrollbarH(Entity);

/// Marks which body owns a scroll-content subtree.
#[derive(Component, Deref)]
struct BodyContentOwner(Entity);

/// Marks which body owns a scrollbar overlay.
#[derive(Component, Deref)]
struct BodyScrollbarOwner(Entity);

/// Tracks hover counts for body widgets.
#[derive(Resource, Default)]
struct HoveredBodyTracker {
    body_ref: HashMap<Entity, u32>,
    last_body: Option<Entity>,
}

/// Returns true for dialog overlays/panels that must not be nested into scroll-content.
fn is_dialog_overlay_node(tag_opt: Option<&TagName>, class_opt: Option<&CssClass>) -> bool {
    if let Some(tag) = tag_opt {
        let tag_name = tag.0.trim();
        if tag_name.eq_ignore_ascii_case("dialog")
            || tag_name.eq_ignore_ascii_case("dialog-overlay")
        {
            return true;
        }
    }

    class_opt.is_some_and(|classes| {
        classes.0.iter().any(|class_name| {
            class_name == "dialog-widget"
                || class_name == "dialog-renderer-bevy-app"
                || class_name == "dialog-overlay"
        })
    })
}

/// Plugin that wires up body widget behavior.
pub struct BodyWidget;

impl Plugin for BodyWidget {
    /// Registers systems for body widget setup.
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredBodyTracker>();
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                ensure_body_scroll_structure,
                route_hover_from_pointer_messages,
                sync_body_content_from_scrollbar,
                handle_body_scroll_wheel,
                sync_body_scrollbar_from_content,
            )
                .chain(),
        );
    }
}

/// Spawns internal Bevy UI nodes for body widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Body, Option<&CssSource>, Option<&mut HtmlStyle>),
        (With<Body>, Without<BodyBase>),
    >,
    existing_bodies: Query<&ZIndex, With<BodyBase>>,
    config: Res<ExtendedUiConfiguration>,
    ui_registry: Res<UiRegistry>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    let ui_order = ui_registry.current.as_ref();
    let max_z_index = existing_bodies
        .iter()
        .map(|z_index| z_index.0)
        .max()
        .unwrap_or(-1);
    let mut next_z_index = max_z_index + 1;

    for (entity, body, source_opt, html_style) in query.iter_mut() {
        let css_source = source_opt.cloned().unwrap_or_default();

        let mut html_id = String::new();
        if let Some(id) = body.html_key.clone() {
            html_id = id;
        }

        let z_index = if html_id.is_empty() {
            0
        } else if ui_order
            .and_then(|names| names.iter().position(|name| name == &html_id))
            .is_some()
        {
            let assigned = next_z_index;
            next_z_index += 1;
            assigned
        } else {
            0
        };

        if let Some(mut inline_style) = html_style {
            inline_style.0.z_index = Some(z_index);
        } else {
            let mut style = Style::default();
            style.z_index = Some(z_index);
            commands.entity(entity).insert(HtmlStyle(style));
        }

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Body-{}-{}", html_id, body.entry)),
                Node::default(),
                WidgetId {
                    id: body.entry,
                    kind: WidgetKind::Body,
                },
                BackgroundColor::default(),
                ImageNode::default(),
                ZIndex(z_index),
                Pickable::default(),
                css_source,
                TagName("body".to_string()),
                RenderLayers::layer(*layer),
                BodyBase,
            ))
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Ensures the body has scroll content and scrollbar nodes when needed.
fn ensure_body_scroll_structure(
    mut commands: Commands,
    mut body_q: Query<
        (
            Entity,
            &Body,
            &UIGenID,
            &mut Node,
            Option<&mut HtmlStyle>,
            Option<&Children>,
            Option<&CssSource>,
            Option<&CssClass>,
            Option<&RenderLayers>,
            Option<&BodyContentRoot>,
            Option<&BodyScrollbar>,
            Option<&BodyScrollbarH>,
        ),
        (With<Body>, With<BodyBase>, Without<BodyScrollContent>),
    >,
    parent_q: Query<&ChildOf>,
    mut content_node_q: Query<&mut Node, (With<BodyScrollContent>, Without<BodyBase>)>,
    mut content_style_q: Query<&mut HtmlStyle, (With<BodyScrollContent>, Without<Body>)>,
    content_children_q: Query<&Children, With<BodyScrollContent>>,
    child_meta_q: Query<(Option<&TagName>, Option<&CssClass>)>,
    has_scroll_pos_q: Query<(), With<ScrollPosition>>,
) {
    for (
        body_entity,
        body,
        ui_id,
        mut body_node,
        body_inline_style_opt,
        children_opt,
        source_opt,
        class_opt,
        layers_opt,
        root_opt,
        sb_y_opt,
        sb_x_opt,
    ) in body_q.iter_mut()
    {
        let mut wants_scroll_y = body_node.overflow.y == OverflowAxis::Scroll || sb_y_opt.is_some();
        let mut wants_scroll_x = body_node.overflow.x == OverflowAxis::Scroll || sb_x_opt.is_some();
        let body_layout_template = body_node.clone();

        if let Some(root) = root_opt {
            if let Ok(content_node) = content_node_q.get(**root) {
                wants_scroll_y |= content_node.overflow.y == OverflowAxis::Scroll;
                wants_scroll_x |= content_node.overflow.x == OverflowAxis::Scroll;
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

            let mut content_node = body_layout_template.clone();
            content_node.width = Val::Percent(100.0);
            content_node.height = Val::Percent(100.0);
            content_node.position_type = PositionType::Relative;
            content_node.left = Val::Auto;
            content_node.right = Val::Auto;
            content_node.top = Val::Auto;
            content_node.bottom = Val::Auto;
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

            let mut classes = class_opt.map(|c| c.0.clone()).unwrap_or_default();
            classes.push("body-scroll-content".to_string());

            let content_entity = commands
                .spawn((
                    Name::new(format!("Body-ScrollContent-{}", body.entry)),
                    content_node,
                    css_source,
                    CssClass(classes),
                    TagName("div".to_string()),
                    BodyScrollContent,
                    BodyContentOwner(body_entity),
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

            commands.entity(body_entity).add_child(content_entity);
            commands
                .entity(body_entity)
                .insert(BodyContentRoot(content_entity));

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
                    Name::new(format!("Body-Scrollbar-{}", body.entry)),
                    sb_node,
                    css_source.clone(),
                    CssClass(vec![
                        "scrollbar".to_string(),
                        "scrollbar-vertical".to_string(),
                    ]),
                    TagName("scroll".to_string()),
                    BodyScrollbarOwner(body_entity),
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

            commands.entity(body_entity).add_child(sb_entity);
            commands
                .entity(body_entity)
                .insert(BodyScrollbar(sb_entity));
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
                    Name::new(format!("Body-Scrollbar-H-{}", body.entry)),
                    sb_node,
                    css_source,
                    CssClass(vec![
                        "scrollbar".to_string(),
                        "scrollbar-horizontal".to_string(),
                        "scroll-horizontal".to_string(),
                    ]),
                    TagName("scroll".to_string()),
                    BodyScrollbarOwner(body_entity),
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

            commands.entity(body_entity).add_child(sb_entity);
            commands
                .entity(body_entity)
                .insert(BodyScrollbarH(sb_entity));
            sb_x_entity = Some(sb_entity);
        }

        // 3a) Move dialog overlays back to body root if they were previously nested in content.
        if let Ok(content_children) = content_children_q.get(content_entity) {
            for child in content_children.iter() {
                if let Ok((tag_opt, class_opt)) = child_meta_q.get(child)
                    && is_dialog_overlay_node(tag_opt, class_opt)
                {
                    commands.entity(child).set_parent_in_place(body_entity);
                }
            }
        }

        // 3b) Reparent regular body children under content (only if needed)
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
                if let Ok((tag_opt, class_opt)) = child_meta_q.get(child)
                    && is_dialog_overlay_node(tag_opt, class_opt)
                {
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

        // 3c) Ensure the body's direct children order keeps scroll-content first.
        if let Some(children) = children_opt {
            let mut desired = Vec::with_capacity(children.len());
            desired.push(content_entity);
            if let Some(sb) = sb_y_entity {
                desired.push(sb);
            }
            if let Some(sb) = sb_x_entity {
                desired.push(sb);
            }
            for child in children.iter().clone() {
                if child == content_entity
                    || Some(child) == sb_y_opt.map(|s| **s)
                    || Some(child) == sb_x_opt.map(|s| **s)
                {
                    continue;
                }
                if let Ok((tag_opt, class_opt)) = child_meta_q.get(child)
                    && is_dialog_overlay_node(tag_opt, class_opt)
                {
                    desired.push(child);
                }
            }

            if !children.iter().clone().eq(desired.iter().copied()) {
                commands.entity(body_entity).add_children(&desired);
            }
        }

        // Wrapper clips but does not scroll.
        body_node.overflow.y = OverflowAxis::Clip;
        body_node.overflow.x = OverflowAxis::Clip;

        // Ensure content scroll flags stay aligned.
        if let Ok(mut node) = content_node_q.get_mut(content_entity) {
            *node = body_layout_template.clone();
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
            node.position_type = PositionType::Relative;
            node.left = Val::Auto;
            node.right = Val::Auto;
            node.top = Val::Auto;
            node.bottom = Val::Auto;
        }

        if has_scroll_pos_q.get(content_entity).is_err() {
            commands
                .entity(content_entity)
                .insert(ScrollPosition::default());
        }

        // Keep the outer body as a clipping wrapper; the inner content node is the scroll target.
        let body_clip_overflow = Overflow {
            x: OverflowAxis::Clip,
            y: OverflowAxis::Clip,
        };
        if let Some(mut inline_style) = body_inline_style_opt {
            inline_style.0.overflow = Some(body_clip_overflow);
        } else {
            let mut style = Style::default();
            style.overflow = Some(body_clip_overflow);
            commands.entity(body_entity).insert(HtmlStyle(style));
        }

        // Mirror effective scroll axes into inline style so later CSS passes cannot unset scrolling.
        let content_overflow = Overflow {
            x: if wants_scroll_x {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            },
            y: if wants_scroll_y {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            },
        };
        if let Ok(mut inline_style) = content_style_q.get_mut(content_entity) {
            inline_style.0.overflow = Some(content_overflow);
            inline_style.0.width = Some(Val::Percent(100.0));
            inline_style.0.height = Some(Val::Percent(100.0));
            inline_style.0.position_type = Some(PositionType::Relative);
            inline_style.0.left = Some(Val::Auto);
            inline_style.0.right = Some(Val::Auto);
            inline_style.0.top = Some(Val::Auto);
            inline_style.0.bottom = Some(Val::Auto);
        } else {
            let mut style = Style::default();
            style.overflow = Some(content_overflow);
            style.width = Some(Val::Percent(100.0));
            style.height = Some(Val::Percent(100.0));
            style.position_type = Some(PositionType::Relative);
            style.left = Some(Val::Auto);
            style.right = Some(Val::Auto);
            style.top = Some(Val::Auto);
            style.bottom = Some(Val::Auto);
            commands.entity(content_entity).insert(HtmlStyle(style));
        }

        commands.entity(content_entity).insert((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Inherited,
            InheritedVisibility::default(),
        ));
    }
}

/// Routes hover state to the owning body.
fn route_hover_from_pointer_messages(
    mut over: MessageReader<Pointer<Over>>,
    mut out: MessageReader<Pointer<Out>>,
    parents: Query<&ChildOf>,
    is_body_q: Query<(), With<Body>>,
    scroll_owner_q: Query<&BodyContentOwner, With<BodyScrollContent>>,
    sb_owner_q: Query<&BodyScrollbarOwner>,
    is_scroll_content_q: Query<(), With<BodyScrollContent>>,
    mut body_state_q: Query<&mut UIWidgetState, With<Body>>,
    mut hovered: ResMut<HoveredBodyTracker>,
) {
    /// Walks up the hierarchy to find the owning body entity.
    fn find_owner_body(
        mut e: Entity,
        parents: &Query<&ChildOf>,
        is_body_q: &Query<(), With<Body>>,
        is_scroll_content_q: &Query<(), With<BodyScrollContent>>,
        scroll_owner_q: &Query<&BodyContentOwner, With<BodyScrollContent>>,
        sb_owner_q: &Query<&BodyScrollbarOwner>,
    ) -> Option<Entity> {
        loop {
            if let Ok(owner) = sb_owner_q.get(e) {
                return Some(**owner);
            }

            if is_body_q.get(e).is_ok() {
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
        let Some(body) = find_owner_body(
            msg.entity,
            &parents,
            &is_body_q,
            &is_scroll_content_q,
            &scroll_owner_q,
            &sb_owner_q,
        ) else {
            continue;
        };

        let d = hovered.body_ref.entry(body).or_insert(0);
        *d = d.saturating_add(1);
        hovered.last_body = Some(body);

        if let Ok(mut state) = body_state_q.get_mut(body) {
            state.hovered = true;
        }
    }

    for msg in out.read() {
        let Some(body) = find_owner_body(
            msg.entity,
            &parents,
            &is_body_q,
            &is_scroll_content_q,
            &scroll_owner_q,
            &sb_owner_q,
        ) else {
            continue;
        };

        if let Some(d) = hovered.body_ref.get_mut(&body) {
            *d = d.saturating_sub(1);
            if *d == 0 {
                hovered.body_ref.remove(&body);

                if let Ok(mut state) = body_state_q.get_mut(body) {
                    state.hovered = false;
                }

                if hovered.last_body == Some(body) {
                    hovered.last_body = hovered.body_ref.keys().next().copied();
                }
            }
        }
    }
}

/// Wheel scroll uses the "last hovered body" and scrolls its BodyScrollContent (Y only).
fn handle_body_scroll_wheel(
    mut wheel_events: MessageReader<MouseWheel>,
    hovered: Res<HoveredBodyTracker>,
    body_q: Query<(Entity, &BodyContentRoot), With<Body>>,
    mut content_q: Query<(&Node, &ComputedNode, &mut ScrollPosition), With<BodyScrollContent>>,
    div_q: Query<(&UIWidgetState, &DivContentRoot), With<Div>>,
    div_content_q: Query<
        (&Node, &ComputedNode),
        (With<ScrollPosition>, Without<BodyScrollContent>),
    >,
    choice_overlay_q: Query<
        (&UIWidgetState, &Node, &ComputedNode, &Visibility),
        With<ChoiceLayoutBoxBase>,
    >,
    dialog_overlay_q: Query<(Option<&TagName>, Option<&CssClass>, &Visibility)>,
) {
    // Scrollable div under the pointer has priority over body scrolling.
    let has_hovered_scrollable_div = div_q.iter().any(|(state, root)| {
        if !state.hovered {
            return false;
        }

        let Ok((node, computed)) = div_content_q.get(**root) else {
            return false;
        };

        if node.overflow.y != OverflowAxis::Scroll {
            return false;
        }

        let inv_sf = computed.inverse_scale_factor.max(f32::EPSILON);
        let viewport_h = (computed.size().y * inv_sf).max(1.0);
        let content_h = (computed.content_size.y * inv_sf).max(viewport_h);
        let max_scroll = (content_h - viewport_h).max(0.0);

        max_scroll > 0.5
    });
    if has_hovered_scrollable_div {
        return;
    }

    // Open ChoiceBox overlays should keep wheel focus while hovered.
    let has_hovered_scrollable_choice_overlay =
        choice_overlay_q
            .iter()
            .any(|(state, node, computed, visibility)| {
                if !state.hovered {
                    return false;
                }
                if !matches!(*visibility, Visibility::Visible | Visibility::Inherited) {
                    return false;
                }
                if node.overflow.y != OverflowAxis::Scroll {
                    return false;
                }

                let inv_sf = computed.inverse_scale_factor.max(f32::EPSILON);
                let viewport_h = (computed.size().y * inv_sf).max(1.0);
                let content_h = (computed.content_size.y * inv_sf).max(viewport_h);
                let max_scroll = (content_h - viewport_h).max(0.0);

                max_scroll > 0.5
            });
    if has_hovered_scrollable_choice_overlay {
        return;
    }

    // Block body wheel while any bevy-app dialog overlay is visible.
    let has_visible_dialog_overlay =
        dialog_overlay_q
            .iter()
            .any(|(tag_opt, class_opt, visibility)| {
                matches!(*visibility, Visibility::Visible | Visibility::Inherited)
                    && is_dialog_overlay_node(tag_opt, class_opt)
            });
    if has_visible_dialog_overlay {
        return;
    }

    let active_body = hovered
        .last_body
        .or_else(|| body_q.iter().next().map(|(entity, _)| entity));
    let Some(active_body) = active_body else {
        wheel_events.clear();
        return;
    };

    let Ok((_, root)) = body_q.get(active_body) else {
        return;
    };

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

/// Synchronizes body content scroll positions from scrollbar widget values.
fn sync_body_content_from_scrollbar(
    body_q: Query<
        (
            Option<&BodyContentRoot>,
            Option<&BodyScrollbar>,
            Option<&BodyScrollbarH>,
        ),
        With<Body>,
    >,
    content_q: Query<&ComputedNode, With<BodyScrollContent>>,
    scroll_q: Query<&Scrollbar>,
    mut target_pos_q: Query<&mut ScrollPosition, With<BodyScrollContent>>,
) {
    for (root_opt, sb_y_opt, sb_x_opt) in body_q.iter() {
        let Some(root) = root_opt else {
            continue;
        };
        let Ok(content_comp) = content_q.get(**root) else {
            continue;
        };
        let Ok(mut pos) = target_pos_q.get_mut(**root) else {
            continue;
        };

        let inv_sf = content_comp.inverse_scale_factor.max(f32::EPSILON);
        let viewport = content_comp.size() * inv_sf;
        let content = content_comp.content_size * inv_sf;

        if let Some(sb) = sb_y_opt {
            if let Ok(scrollbar) = scroll_q.get(**sb) {
                let viewport_h = viewport.y.max(1.0);
                let content_h = content.y.max(viewport_h);
                let max_scroll = (content_h - viewport_h).max(0.0);
                pos.y = scrollbar.value.clamp(0.0, max_scroll);
            }
        }

        if let Some(sb) = sb_x_opt {
            if let Ok(scrollbar) = scroll_q.get(**sb) {
                let viewport_w = viewport.x.max(1.0);
                let content_w = content.x.max(viewport_w);
                let max_scroll = (content_w - viewport_w).max(0.0);
                pos.x = scrollbar.value.clamp(0.0, max_scroll);
            }
        }
    }
}

/// Synchronizes body scrollbars from scrollable content.
fn sync_body_scrollbar_from_content(
    body_q: Query<
        (
            Option<&BodyContentRoot>,
            Option<&BodyScrollbar>,
            Option<&BodyScrollbarH>,
        ),
        With<Body>,
    >,
    content_q: Query<&ComputedNode, With<BodyScrollContent>>,
    mut scroll_q: Query<&mut Scrollbar>,
    target_pos_q: Query<&ScrollPosition, With<BodyScrollContent>>,
    mut sb_node_q: Query<&mut Node, With<Scrollbar>>,
    mut sb_vis_q: Query<&mut Visibility, With<Scrollbar>>,
) {
    for (root_opt, sb_y_opt, sb_x_opt) in body_q.iter() {
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
                scrollbar.entity = Some(**root);
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
                scrollbar.entity = Some(**root);
                scrollbar.min = 0.0;
                scrollbar.max = max_scroll;
                scrollbar.viewport_extent = viewport_w;
                scrollbar.content_extent = content_w;
                scrollbar.value = scroll_pos.x.clamp(0.0, max_scroll);
            }
        }
    }
}

/// Updates scrollbar visibility if content dimensions have changed.
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

/// Event handler that activates when an internal body node is clicked.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Body>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.event_target()) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }

    trigger.propagate(false);
}

/// Event handler for when the cursor enters an internal body node.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Body>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.event_target()) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Event handler for when the cursor leaves an internal body node.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Body>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.event_target()) {
        state.hovered = false;
    }

    trigger.propagate(false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::message::Messages;
    use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

    fn computed_node(viewport: Vec2, content: Vec2) -> ComputedNode {
        let mut computed = ComputedNode::default();
        computed.inverse_scale_factor = 1.0;
        computed.size = viewport;
        computed.content_size = content;
        computed
    }

    fn line_wheel(y: f32) -> MouseWheel {
        MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y,
            window: Entity::PLACEHOLDER,
        }
    }

    fn spawn_body_scroll_content(world: &mut World, viewport_h: f32, content_h: f32) -> Entity {
        let mut node = Node::default();
        node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        world
            .spawn((
                BodyScrollContent,
                node,
                computed_node(Vec2::new(200.0, viewport_h), Vec2::new(200.0, content_h)),
                ScrollPosition::default(),
            ))
            .id()
    }

    fn spawn_div_scroll_content(world: &mut World, viewport_h: f32, content_h: f32) -> Entity {
        let mut node = Node::default();
        node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        world
            .spawn((
                node,
                computed_node(Vec2::new(180.0, viewport_h), Vec2::new(180.0, content_h)),
                ScrollPosition::default(),
            ))
            .id()
    }

    fn spawn_choice_overlay(
        world: &mut World,
        viewport_h: f32,
        content_h: f32,
        hovered: bool,
    ) -> Entity {
        let mut node = Node::default();
        node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        world
            .spawn((
                ChoiceLayoutBoxBase,
                UIWidgetState {
                    hovered,
                    ..default()
                },
                node,
                computed_node(Vec2::new(220.0, viewport_h), Vec2::new(220.0, content_h)),
                ScrollPosition::default(),
                Visibility::Inherited,
            ))
            .id()
    }

    fn spawn_dialog_overlay_marker(world: &mut World, visible: bool) -> Entity {
        world
            .spawn((
                TagName("dialog-overlay".to_string()),
                CssClass(vec!["dialog-overlay".to_string()]),
                if visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ))
            .id()
    }

    #[test]
    fn body_wheel_scroll_moves_body_content_when_no_div_is_hovered() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-1.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 25.0);
    }

    #[test]
    fn body_wheel_scroll_is_blocked_by_hovered_scrollable_div() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        let div_content_entity = spawn_div_scroll_content(app.world_mut(), 120.0, 260.0);
        app.world_mut().spawn((
            Div::default(),
            UIWidgetState {
                hovered: true,
                ..default()
            },
            DivContentRoot(div_content_entity),
        ));

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-2.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn body_wheel_scroll_ignores_hovered_div_without_scroll_range() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        // Content equals viewport -> max scroll is zero, so this div should not block body scrolling.
        let div_content_entity = spawn_div_scroll_content(app.world_mut(), 160.0, 160.0);
        app.world_mut().spawn((
            Div::default(),
            UIWidgetState {
                hovered: true,
                ..default()
            },
            DivContentRoot(div_content_entity),
        ));

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-1.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 25.0);
    }

    #[test]
    fn ensure_body_scroll_structure_creates_content_and_vertical_scrollbar() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, ensure_body_scroll_structure);

        let mut body_node = Node::default();
        body_node.width = Val::Px(640.0);
        body_node.height = Val::Px(360.0);
        body_node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyBase, UIGenID::default(), body_node))
            .id();

        app.update();

        let body_root = app
            .world()
            .get::<BodyContentRoot>(body_entity)
            .expect("body should have a content root after setup");
        let content_entity = **body_root;

        let body_node_after = app
            .world()
            .get::<Node>(body_entity)
            .expect("body should still have a node");
        assert_eq!(body_node_after.overflow.y, OverflowAxis::Clip);
        assert_eq!(body_node_after.overflow.x, OverflowAxis::Clip);

        assert!(
            app.world()
                .get::<BodyScrollContent>(content_entity)
                .is_some(),
            "content root should be marked as BodyScrollContent"
        );
        assert!(
            app.world().get::<ScrollPosition>(content_entity).is_some(),
            "content root should be scrollable"
        );

        let content_node = app
            .world()
            .get::<Node>(content_entity)
            .expect("content root should have a node");
        assert_eq!(content_node.overflow.y, OverflowAxis::Scroll);
        assert_eq!(content_node.overflow.x, OverflowAxis::Hidden);
        assert_eq!(content_node.width, Val::Percent(100.0));
        assert_eq!(content_node.height, Val::Percent(100.0));

        let scrollbar_ref = app
            .world()
            .get::<BodyScrollbar>(body_entity)
            .expect("vertical scrollbar should exist");
        let scrollbar = app
            .world()
            .get::<Scrollbar>(**scrollbar_ref)
            .expect("scrollbar entity should have Scrollbar component");
        assert!(scrollbar.vertical);
        assert_eq!(scrollbar.entity, Some(content_entity));
    }

    #[test]
    fn body_wheel_scroll_is_blocked_by_hovered_scrollable_choice_overlay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        let _overlay = spawn_choice_overlay(app.world_mut(), 90.0, 240.0, true);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-2.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn body_wheel_scroll_is_blocked_by_visible_dialog_overlay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        let _overlay = spawn_dialog_overlay_marker(app.world_mut(), true);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-2.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 0.0);
    }
}
