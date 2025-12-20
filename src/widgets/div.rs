use std::collections::HashMap;

use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::ScrollPosition;

use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{Div, Scrollbar, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};

#[derive(Component)]
struct DivBase;

#[derive(Component)]
struct DivScrollContent;

#[derive(Component, Deref)]
struct DivContentRoot(Entity);

#[derive(Component, Deref)]
struct DivScrollbar(Entity);

/// Mark which Div owns this scroll-content subtree.
#[derive(Component, Deref)]
struct DivContentOwner(Entity);

/// Mark which Div owns this scrollbar overlay (so hovering scrollbar also enables wheel scrolling if desired).
#[derive(Component, Deref)]
struct DivScrollbarOwner(Entity);

/// Robust hover tracking that survives "child swap" (Out/Over firing when moving between pickable children).
///
/// We refcount hover hits per Div and keep the last active Div for wheel scrolling.
#[derive(Resource, Default)]
struct HoveredDivTracker {
    div_ref: HashMap<Entity, u32>,
    last_div: Option<Entity>,
}

pub struct DivWidget;

impl Plugin for DivWidget {
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

/// Creates the base node for <div>
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
                BorderRadius::default(),
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

fn ensure_div_scroll_structure(
    mut commands: Commands,
    mut div_q: Query<
        (
            Entity,
            &Div,
            &mut Node,
            Option<&Children>,
            Option<&CssSource>,
            Option<&CssClass>,
            Option<&RenderLayers>,
            Option<&DivContentRoot>,
            Option<&DivScrollbar>,
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
        mut div_node,
        children_opt,
        source_opt,
        wrapper_class_opt,
        layers_opt,
        root_opt,
        sb_opt,
    ) in div_q.iter_mut()
    {
        // the requested state comes from your css->node system writing Node.overflow.y
        // latch once the structure exists so it doesn't thrash
        let wants_scroll = root_opt.is_some() || div_node.overflow.y == OverflowAxis::Scroll;
        if !wants_scroll {
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
            content_node.overflow.y = OverflowAxis::Scroll;
            content_node.overflow.x = OverflowAxis::Hidden;

            // IMPORTANT: content must inherit wrapper styles so CSS keeps working
            let mut classes: Vec<String> = wrapper_class_opt
                .map(|c| c.0.clone())
                .unwrap_or_default();
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
                    ScrollPosition::default(),
                    UIWidgetState::default(),
                    // pickable so we can still get pointer hits when directly over the empty content area
                    Pickable::default(),
                ))
                .id();

            if let Some(layers) = layers_opt {
                commands.entity(content_entity).insert(layers.clone());
            }

            commands.entity(div_entity).add_child(content_entity);
            commands.entity(div_entity).insert(DivContentRoot(content_entity));

            content_entity
        };

        // 2) Ensure overlay scrollbar child (sibling of content)
        if sb_opt.is_none() {
            let css_source = source_opt.cloned().unwrap_or_default();

            // right-side absolute overlay
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
                    css_source,
                    CssClass(vec!["scrollbar".to_string(), "scrollbar-vertical".to_string()]),
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
                    BorderRadius::default(),
                    Pickable::default(),
                ))
                .id();

            if let Some(layers) = layers_opt {
                commands.entity(sb_entity).insert(layers.clone());
            }

            commands.entity(div_entity).add_child(sb_entity);
            commands.entity(div_entity).insert(DivScrollbar(sb_entity));
        }

        // 3) Reparent wrapper children under content (only if needed)
        if let Some(children) = children_opt {
            let scrollbar_entity = sb_opt.map(|s| **s);

            let list: Vec<Entity> = children.iter().clone().collect();
            for child in list {
                if child == content_entity {
                    continue;
                }
                if Some(child) == scrollbar_entity {
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

        // Wrapper clips, but does not scroll (prevents scrollbar moving with content)
        div_node.overflow.y = OverflowAxis::Clip;
        div_node.overflow.x = OverflowAxis::Clip;

        // Ensure content remains scrollable
        if let Ok(mut node) = content_node_q.get_mut(content_entity) {
            node.overflow.y = OverflowAxis::Scroll;
            node.overflow.x = OverflowAxis::Hidden;
            node.width = Val::Percent(100.0);
            node.height = Val::Percent(100.0);
        }

        // Safety: ensure ScrollPosition exists
        if has_scroll_pos_q.get(content_entity).is_err() {
            commands.entity(content_entity).insert(ScrollPosition::default());
        }
    }
}

/// Global hover routing:
/// - If pa ointer is over ANY pickable child inside the scroll-content subtree -> mark owning Div hovered
/// - If a pointer is over scrollbar -> also mark owning Div hovered (optional but usually desired)
///
/// Uses refcount to avoid flicker while moving between children in the same div.
fn route_hover_from_pointer_messages(
    mut over: MessageReader<Pointer<Over>>,
    mut out: MessageReader<Pointer<Out>>,
    parents: Query<&ChildOf>,
    scroll_owner_q: Query<&DivContentOwner, With<DivScrollContent>>,
    sb_owner_q: Query<&DivScrollbarOwner>,
    is_scroll_content_q: Query<(), With<DivScrollContent>>,
    mut div_state_q: Query<&mut UIWidgetState, With<Div>>,
    mut hovered: ResMut<HoveredDivTracker>,
) {
    fn find_owner_div(
        mut e: Entity,
        parents: &Query<&ChildOf>,
        is_scroll_content_q: &Query<(), With<DivScrollContent>>,
        scroll_owner_q: &Query<&DivContentOwner, With<DivScrollContent>>,
        sb_owner_q: &Query<&DivScrollbarOwner>,
    ) -> Option<Entity> {
        loop {
            // scrollbar overlay path
            if let Ok(owner) = sb_owner_q.get(e) {
                return Some(**owner);
            }

            // content subtree path
            if is_scroll_content_q.get(e).is_ok() {
                if let Ok(owner) = scroll_owner_q.get(e) {
                    return Some(**owner);
                }
                // should never happen, but keep walking if it does
            }

            if let Ok(p) = parents.get(e) {
                e = p.parent();
            } else {
                return None;
            }
        }
    }

    // OVER
    for msg in over.read() {
        let Some(div) = find_owner_div(
            msg.entity,
            &parents,
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

    // OUT
    for msg in out.read() {
        let Some(div) = find_owner_div(
            msg.entity,
            &parents,
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

/// Wheel scroll uses the "last hovered div" and scrolls its DivScrollContent.
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
        let raw = match event.unit {
            MouseScrollUnit::Line => event.y * 25.0,
            MouseScrollUnit::Pixel => event.y,
        };
        let delta = -raw;

        let Ok((node, computed, mut scroll)) = content_q.get_mut(**root) else {
            continue;
        };

        if node.overflow.y != OverflowAxis::Scroll {
            continue;
        }

        let viewport_h = computed.size().y.max(1.0);
        let content_h = computed.content_size.y.max(viewport_h);
        let max_scroll = (content_h - viewport_h).max(0.0);

        scroll.y = (scroll.y + delta).clamp(0.0, max_scroll);
    }
}

/// Sync scrollbar range/value + show only if it makes sense (max_scroll > 0)
fn sync_scrollbar_from_content(
    div_q: Query<(Option<&DivContentRoot>, Option<&DivScrollbar>), With<Div>>,
    content_q: Query<&ComputedNode, With<DivScrollContent>>,
    mut scroll_q: Query<&mut Scrollbar>,
    target_pos_q: Query<&ScrollPosition>,
    mut sb_node_q: Query<&mut Node, With<Scrollbar>>,
) {
    for (root_opt, sb_opt) in div_q.iter() {
        let (Some(root), Some(sb)) = (root_opt, sb_opt) else { continue; };

        let Ok(content_comp) = content_q.get(**root) else { continue; };

        let viewport_h = content_comp.size().y.max(1.0);
        let content_h = content_comp.content_size.y.max(viewport_h);
        let max_scroll = (content_h - viewport_h).max(0.0);

        // Plan change: show only if needed (even when overflow-y: scroll)
        let show = max_scroll > 0.0;
        if let Ok(mut sb_node) = sb_node_q.get_mut(**sb) {
            sb_node.display = if show { Display::Flex } else { Display::None };
        }

        let Ok(mut scrollbar) = scroll_q.get_mut(**sb) else { continue; };
        scrollbar.min = 0.0;
        scrollbar.max = max_scroll;

        if let Some(target) = scrollbar.entity {
            if let Ok(pos) = target_pos_q.get(target) {
                scrollbar.value = pos.y.clamp(0.0, max_scroll);
            }
        }
    }
}

// -------------------- Events --------------------

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
