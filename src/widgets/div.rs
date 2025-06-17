use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::convert::{CssSource, TagName};
use crate::styling::paint::Colored;
use crate::widgets::Div;

#[derive(Component)]
struct DivBase;

pub struct DivWidget;

impl Plugin for DivWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
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
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, div, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }
        
        commands.entity(entity).insert((
            Name::new(format!("Div-{}", div.0)),
            Node::default(),
            ImageNode::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            css_source,
            TagName("div".to_string()),
            RenderLayers::layer(*layer),
            DivBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
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
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Div>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
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
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

/// Sets `hovered = false` when the cursor leaves a [`Div`] widget's bounds.
///
/// # Triggered By:
/// - `Trigger<Pointer<Out>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}