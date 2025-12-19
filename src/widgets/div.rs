use crate::styles::paint::Colored;
use crate::styles::{CssSource, TagName};
use crate::widgets::{Div, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

#[derive(Component)]
struct DivBase;

pub struct DivWidget;

impl Plugin for DivWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            internal_node_creation_system,
            handle_div_scroll_wheel,
        ));
    }
}

/// Initializes internal UI nodes for all [`Div`] components without a [`DivBase`] yet.
///
/// This system creates a visual representation for HTML-like `<div>` nodes,
/// applying default or user-specified styling via [`CssSource`]. Each spawned
/// node is tagged with [`DivBase`] to prevent reinitialization and enable further
/// widget behavior.
///
/// The node is set up with common UI visual components like:
/// - [`Node`], [`ImageNode`], [`BackgroundColor`], [`BorderColor`], [`BorderRadius`], [`BoxShadow`]
/// - [`ZIndex`], [`CssSource`], [`TagName("div")`], and a [`RenderLayers`] level
///
/// It also wires up internal pointer-based event handlers:
/// - [`on_internal_click`] → focuses the node and updates the current widget state
/// - [`on_internal_cursor_entered`] → sets hover state true
/// - [`on_internal_cursor_leave`] → sets hover state false
///
/// # Parameters
/// - `commands`: [`Commands`] to mutate entities
/// - `query`: Finds all [`Div`] entities that are not yet marked as [`DivBase`]
/// - `config`: Global UI configuration, used here to determine render layer
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Div, Option<&CssSource>), (With<Div>, Without<DivBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, div, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

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
            ))
            .insert(ScrollPosition::default())
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn handle_div_scroll_wheel(
    mut wheel_events: MessageReader<MouseWheel>,
    mut div_q: Query<
        (
            &Visibility,
            &Node,
            &ComputedNode,
            &mut ScrollPosition,
            &UIWidgetState,
        ),
        With<Div>,
    >,
    time: Res<Time>,
) {
    // Optional smoothing like in your ChoiceBox
    let smooth_factor = 30.0;

    for event in wheel_events.read() {
        // Wheel down -> scroll down -> increase scroll.y
        let raw = match event.unit {
            MouseScrollUnit::Line => event.y * 25.0,
            MouseScrollUnit::Pixel => event.y,
        };
        let delta = -raw;

        for (vis, node, computed, mut scroll, _state) in div_q.iter_mut() {
            if !matches!(*vis, Visibility::Visible | Visibility::Inherited) {
                continue;
            }
            if node.overflow.y != OverflowAxis::Scroll {
                continue;
            }

            // Optional: only scroll when hovered (depends on your UI model)
            // if !state.hovered { continue; }

            // Viewport height (visible area)
            let viewport_h = computed.size().y.max(1.0);

            // Content height (what Bevy actually laid out)
            let content_h = computed.content_size.y.max(viewport_h);

            // Maximum scroll range
            let max_scroll = (content_h - viewport_h).max(0.0);

            // Clamp so 0.0 is always the very top (Hello1 must be reachable)
            let target = (scroll.y + delta).clamp(0.0, max_scroll);

            // Smooth interpolation (like your ChoiceBox)
            let smoothed = scroll.y + (target - scroll.y) * smooth_factor * time.delta_secs();
            scroll.y = smoothed.clamp(0.0, max_scroll);
        }
    }
}

/// Handles click events on a [`Div`] widget.
///
/// This system marks the widget as focused and updates the global
/// [`CurrentWidgetState`] with its widget ID for input routing or styling.
///
/// # Triggered By:
/// - `Trigger<Pointer<Click>>`
///
/// # Affects:
/// - `UIWidgetState::focused`
/// - `CurrentWidgetState::widget_id`
fn on_internal_click(
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

/// Sets `hovered = true` when the cursor enters a [`Div`] widget's bounds.
///
/// Useful for hover effects like changing background or border styles.
///
/// # Triggered By:
/// - `Trigger<Pointer<Over>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Sets `hovered = false` when the cursor leaves a [`Div`] widget's bounds.
///
/// # Triggered By:
/// - `Trigger<Pointer<Out>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}