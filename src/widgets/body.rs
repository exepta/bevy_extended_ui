use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::convert::{CssSource, TagName};
use crate::widgets::{HtmlBody, WidgetId, WidgetKind};

#[derive(Component)]
struct HtmlBodyBase;

pub struct HtmlBodyWidget;

impl Plugin for HtmlBodyWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

/// System that initializes internal UI nodes for HTML body elements.
///
/// It sets up a visual and interactive node for each [`HtmlBody`] that hasn't already
/// been processed (i.e., doesn't have an [`HtmlBodyBase`] tag yet). The node is styled,
/// observed for interaction, and tagged for future queries.
///
/// # Parameters
/// - `commands`: Commands to spawn or modify entities.
/// - `query`: A query to find all [`HtmlBody`] entities that haven't been set up yet.
/// - `config`: Configuration resource providing rendering layers.
///
/// # Behavior
/// Each matching entity gets the following inserted:
/// - [`Name`] for debugging
/// - [`Node`], [`BackgroundColor`], [`ImageNode`], [`ZIndex`] for UI display
/// - [`CssSource`], [`TagName`] for styling
/// - [`RenderLayers`] to define which camera layer renders it
/// - [`HtmlBodyBase`] marker to avoid re-processing
/// - Observers for pointer click and hover events
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &HtmlBody, Option<&CssSource>), (With<HtmlBody>, Without<HtmlBodyBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, body, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }
        
        let mut html_id = String::new();
        if let Some(id) = body.bind_to_html.clone() {
            html_id = id;
        }

        commands.entity(entity).insert((
            Name::new(format!("Body-{}-{}", html_id, body.w_count)),
            Node::default(),
            WidgetId {
                id: body.w_count,
                kind: WidgetKind::HtmlBody
            },
            BackgroundColor::default(),
            ImageNode::default(),
            ZIndex::default(),
            Pickable::default(),
            css_source,
            TagName("body".to_string()),
            RenderLayers::layer(*layer),
            HtmlBodyBase,
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Event handler that activates when an internal body node is clicked.
///
/// Sets the widget's state to "focused" and updates the [`CurrentWidgetState`] resource
/// to reflect the clicked widget's ID.
///
/// # Parameters
/// - `trigger`: Contains the pointer event and target entity.
/// - `query`: Allows updating the focused state on matching widgets.
/// - `current_widget_state`: Global resource tracking currently focused widget.
fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<HtmlBody>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

/// Event handler for when the cursor enters an internal body node.
///
/// Sets the `hovered` state to `true`, triggering any hover-related UI feedback.
fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<HtmlBody>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

/// Event handler for when the cursor leaves an internal body node.
///
/// Resets the `hovered` state to `false`.
fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<HtmlBody>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}