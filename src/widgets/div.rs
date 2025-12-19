use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{Div, Scrollbar, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};

use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::ScrollPosition;

#[derive(Component)]
struct DivBase;

#[derive(Component)]
struct DivScrollContent;

#[derive(Component, Deref)]
struct DivContentRoot(Entity);

#[derive(Component, Deref)]
struct DivScrollbar(Entity);

/// Allows the content-layer to update hover/focus state of the owning div wrapper.
#[derive(Component, Deref)]
struct DivContentOwner(Entity);

pub struct DivWidget;

impl Plugin for DivWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                ensure_div_scroll_structure,
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
            // Wrapper events (may not fire if pointer is over children; we'll mirror from content)
            .observe(on_div_click)
            .observe(on_div_cursor_entered)
            .observe(on_div_cursor_leave);
    }
}

/// Ensure wrapper + scroll-content + overlay scrollbar structure.
///
/// Rule:
/// - If the div's Node requests overflow-y: Scroll -> build structure.
/// - Once built, we KEEP it (Latch gegen Styling-Flackern).
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
    has_scrollpos_q: Query<(), With<ScrollPosition>>,
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
        // Latch: wenn Struktur schon existiert, bleibt sie aktiv
        let wants_scroll = root_opt.is_some() || div_node.overflow.y == OverflowAxis::Scroll;
        if !wants_scroll {
            continue;
        }

        // 1) Ensure scroll-content child
        let content_entity = if let Some(root) = root_opt {
            **root
        } else {
            let css_source = source_opt.cloned().unwrap_or_default();

            // Content Node: only scroller basics. CSS decides layout (flex-direction etc).
            let mut content_node = Node::default();
            content_node.width = Val::Percent(100.0);
            content_node.height = Val::Percent(100.0);
            content_node.overflow.y = OverflowAxis::Scroll;
            content_node.overflow.x = OverflowAxis::Hidden;

            // IMPORTANT: Content inherits wrapper styles:
            // - TagName "div" so all div rules apply
            // - CssClass = wrapper classes + internal marker class
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
                ))
                // Mirror hover to wrapper (so wheel works even when over children)
                .observe(on_content_cursor_entered)
                .observe(on_content_cursor_leave)
                .observe(on_content_click)
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
            let sb_entry = div.0.saturating_mul(10_000).saturating_add(1);

            let mut sb_node = Node::default();
            sb_node.position_type = PositionType::Absolute;
            sb_node.right = Val::Px(0.0);
            sb_node.top = Val::Px(0.0);
            sb_node.bottom = Val::Px(0.0);
            sb_node.width = Val::Px(12.0);

            let css_source = source_opt.cloned().unwrap_or_default();

            let sb_entity = commands
                .spawn((
                    Name::new(format!("Div-Scrollbar-{}", div.0)),
                    sb_node,
                    css_source,
                    CssClass(vec!["scrollbar".to_string(), "scrollbar-vertical".to_string()]),
                    TagName("scroll".to_string()),
                    Scrollbar {
                        entry: sb_entry,
                        vertical: true,
                        min: 0.0,
                        max: 0.0,
                        value: 0.0,
                        step: 1.0,
                        entity: Some(content_entity),
                        ..Default::default()
                    },
                    UIWidgetState::default(),
                    WidgetId {
                        id: sb_entry,
                        kind: WidgetKind::Scrollbar,
                    },
                    ImageNode::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    ZIndex(10),
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

        // 4) Wrapper clippt, but does not scroll
        div_node.overflow.y = OverflowAxis::Hidden;
        div_node.overflow.x = OverflowAxis::Hidden;

        // 5) Ensure content remains scrollable even if some system overwrote it
        if let Ok(mut node) = content_node_q.get_mut(content_entity) {
            node.overflow.y = OverflowAxis::Scroll;
            node.overflow.x = OverflowAxis::Hidden;
            node.width = Val::Percent(100.0);
            node.height = Val::Percent(100.0);
        }

        // 6) Safety: ensure ScrollPosition exists
        if has_scrollpos_q.get(content_entity).is_err() {
            commands.entity(content_entity).insert(ScrollPosition::default());
        }
    }
}

/// Scroll wheel should scroll the *content child*, not the wrapper.
fn handle_div_scroll_wheel(
    mut wheel_events: MessageReader<MouseWheel>,
    div_q: Query<(&Visibility, &UIWidgetState, Option<&DivContentRoot>), With<Div>>,
    mut content_q: Query<(&Node, &ComputedNode, &mut ScrollPosition), With<DivScrollContent>>,
    time: Res<Time>,
) {
    let smooth_factor = 30.0;

    for event in wheel_events.read() {
        let raw = match event.unit {
            MouseScrollUnit::Line => event.y * 25.0,
            MouseScrollUnit::Pixel => event.y,
        };
        let delta = -raw;

        for (vis, state, root_opt) in div_q.iter() {
            if !matches!(*vis, Visibility::Visible | Visibility::Inherited) {
                continue;
            }

            // This now works because content mirrors hover into wrapper state
            if !state.hovered {
                continue;
            }

            let Some(root) = root_opt else { continue; };
            let Ok((node, computed, mut scroll)) = content_q.get_mut(**root) else {
                continue;
            };

            if node.overflow.y != OverflowAxis::Scroll {
                continue;
            }

            let viewport_h = computed.size().y.max(1.0);
            let content_h = computed.content_size.y.max(viewport_h);
            let max_scroll = (content_h - viewport_h).max(0.0);

            let target = (scroll.y + delta).clamp(0.0, max_scroll);
            let smoothed = scroll.y + (target - scroll.y) * smooth_factor * time.delta_secs();
            scroll.y = smoothed.clamp(0.0, max_scroll);
        }
    }
}

/// Sync scrollbar range/value from the content's layout each frame.
fn sync_scrollbar_from_content(
    div_q: Query<(Option<&DivContentRoot>, Option<&DivScrollbar>), With<Div>>,
    content_q: Query<&ComputedNode, With<DivScrollContent>>,
    mut scroll_q: Query<&mut Scrollbar>,
    target_pos_q: Query<&ScrollPosition>,
) {
    for (root_opt, sb_opt) in div_q.iter() {
        let (Some(root), Some(sb)) = (root_opt, sb_opt) else { continue; };

        let Ok(content_comp) = content_q.get(**root) else { continue; };

        let viewport_h = content_comp.size().y.max(1.0);
        let content_h = content_comp.content_size.y.max(viewport_h);
        let max_scroll = (content_h - viewport_h).max(0.0);

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

// -------------------- Events: Div wrapper --------------------

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

fn on_div_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

fn on_div_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

// -------------------- Events: Content layer (mirrors to wrapper) --------------------

fn on_content_cursor_entered(
    trigger: On<Pointer<Over>>,
    owner_q: Query<&DivContentOwner, With<DivScrollContent>>,
    mut div_state_q: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(owner) = owner_q.get(trigger.entity) {
        if let Ok(mut state) = div_state_q.get_mut(**owner) {
            state.hovered = true;
        }
    }
    // don't stop propagation; we just mirror state
}

fn on_content_cursor_leave(
    trigger: On<Pointer<Out>>,
    owner_q: Query<&DivContentOwner, With<DivScrollContent>>,
    mut div_state_q: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(owner) = owner_q.get(trigger.entity) {
        if let Ok(mut state) = div_state_q.get_mut(**owner) {
            state.hovered = false;
        }
    }
}

fn on_content_click(
    trigger: On<Pointer<Click>>,
    owner_q: Query<&DivContentOwner, With<DivScrollContent>>,
    mut div_state_q: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(owner) = owner_q.get(trigger.entity) {
        if let Ok(mut state) = div_state_q.get_mut(**owner) {
            state.focused = true;
        }
    }
    // don't stop propagation
}
