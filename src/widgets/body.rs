use crate::html::HtmlStyle;
use crate::registry::UiRegistry;
use crate::styles::{CssSource, Style, TagName};
use crate::widgets::{Body, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Marker component for the internal body node.
#[derive(Component)]
struct BodyBase;

/// Plugin that wires up body widget behavior.
pub struct BodyWidget;

impl Plugin for BodyWidget {
    /// Registers systems for body widget setup.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
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
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

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
