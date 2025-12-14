use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use crate::styles::{CssSource, TagName};
use crate::widgets::{Headline, UIGenID, UIWidgetState, WidgetId, WidgetKind};

#[derive(Component)]
struct HeadlineBase;

pub struct HeadlineWidget;

impl Plugin for HeadlineWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (internal_node_creation_system, update_text));
    }
}

/// Initializes internal UI nodes for all [`Headline`] components not yet marked with [`HeadlineBase`].
///
/// This system transforms semantic headline elements (like `<h1>`, `<h2>`, etc.)
/// into Bevy `Text` UI nodes. It sets up font styling, text layout, rendering order,
/// and default CSS styling.
///
/// The entity will be tagged with:
/// - [`HeadlineBase`] to prevent duplicate initialization
/// - [`Text`], [`TextFont`], [`TextColor`], [`TextLayout`] for rendering
/// - [`CssSource`] (either provided or default)
/// - [`RenderLayers`] for controlling UI rendering layer
/// - [`TagName`] for semantic tagging (`"h1"`, `"h2"`, etc.)
///
/// Pointer event handlers are also attached:
/// - [`on_internal_click`] → sets the node as focused
/// - [`on_internal_cursor_entered`] → sets hover state to true
/// - [`on_internal_cursor_leave`] → sets hover state to false
///
/// # Parameters
/// - `commands`: Command buffer for inserting components
/// - `query`: Finds [`Headline`] nodes not yet initialized
/// - `config`: UI render configuration for layering
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Headline, Option<&CssSource>), (With<Headline>, Without<HeadlineBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, headline, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("{}-{}", headline.h_type.to_string().to_uppercase(), headline.entry)),
            Node::default(),
            WidgetId {
                id: headline.entry,
                kind: WidgetKind::Headline
            },
            Text::new(headline.text.clone()),
            TextColor::default(),
            TextFont::default(),
            TextLayout::default(),
            ZIndex::default(),
            Pickable::default(),
            css_source,
            TagName(format!("{}", headline.h_type.to_string())),
            RenderLayers::layer(*layer),
            HeadlineBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Synchronizes the `Text` component of entities with their `Headline` component.
///
/// This system runs on all entities that have both a `Text` and a `Headline` component.
/// It updates the `Text` to reflect the current value of the `Headline::text` field.
///
/// # Parameters
/// - `query`: A mutable query that fetches entities with both `Text` and `Headline` components.
///
/// # Behavior
/// For each matching entity, the system copies the headline's text into the `Text` component,
/// ensuring that the UI displays the latest headline content.
///
/// # Example
/// If a `Headline` contains `"Breaking News!"`, the associated `Text` will be updated to show `"Breaking News!"`.
fn update_text(mut query: Query<(&mut Text, &Headline), With<Headline>>) {
    for (mut text, headline) in query.iter_mut() {
        text.0 = headline.text.clone();
    }
}

/// Handles pointer click events on a [`Headline`] node.
///
/// Focuses the clicked widget and stores its ID in the global [`CurrentWidgetState`].
/// This can be used to support keyboard input focus or highlight states.
///
/// # Triggered By:
/// - `Trigger<Pointer<Click>>`
///
/// # Affects:
/// - `UIWidgetState::focused`
/// - `CurrentWidgetState::widget_id`
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Headline>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }

    trigger.propagate(false);
}

/// Sets the hover state to `true` when the cursor enters a [`Headline`] node's bound.
///
/// This is used for visual hover feedback, e.g., style highlighting.
///
/// # Triggered By:
/// - `Trigger<Pointer<Over>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Headline>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Sets the hover state to `false` when the cursor leaves a [`Headline`] node's bound.
///
/// Used to remove hover effects from the node.
///
/// # Triggered By:
/// - `Trigger<Pointer<Out>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Headline>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}