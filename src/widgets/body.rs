use crate::styles::{CssSource, TagName};
use crate::widgets::{Body, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

#[derive(Component)]
struct BodyBase;

pub struct BodyWidget;

impl Plugin for BodyWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Body, Option<&CssSource>), (With<Body>, Without<BodyBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, body, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        let mut html_id = String::new();
        if let Some(id) = body.html_key.clone() {
            html_id = id;
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
                ZIndex::default(),
                FocusPolicy::default(),
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
///
/// Sets the `hovered` state to `true`, triggering any hover-related UI feedback.
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
///
/// Resets the `hovered` state to `false`.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Body>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.event_target()) {
        state.hovered = false;
    }

    trigger.propagate(false);
}
