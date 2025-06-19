use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::convert::{CssSource, TagName};
use crate::widgets::Paragraph;

#[derive(Component)]
struct ParagraphBase;

pub struct ParagraphWidget;

impl Plugin for ParagraphWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (internal_node_creation_system, update_text));
    }
}

/// Creates UI nodes for paragraphs that don't have a `ParagraphBase` yet.
///
/// For each `Paragraph` entity without a `ParagraphBase`, this system inserts necessary UI parts,
/// sets the render layer from the configuration, applies CSS source if available, and attaches event observers.
///
/// # Parameters
/// - `commands`: Commands to modify the ECS world.
/// - `query`: Query for entities with `Paragraph` but without `ParagraphBase`, optionally with a `CssSource`.
/// - `config`: UI configuration resource containing render layers.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Paragraph, Option<&CssSource>), (With<Paragraph>, Without<ParagraphBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, paragraph, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("Paragraph-{}", paragraph.w_count)),
            Node::default(),
            Text::new(paragraph.text.clone()),
            TextColor::default(),
            TextFont::default(),
            TextLayout::default(),
            ZIndex::default(),
            css_source,
            TagName("p".to_string()),
            RenderLayers::layer(*layer),
            ParagraphBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Updates the `Text` components of all entities with a `Paragraph` component.
///
/// This system iterates over all entities that have both a `Text` and `Paragraph` component,
/// and sets the content of the `Text` to match the `text` field of the `Paragraph`.
///
/// # Parameters
/// - `query`: A mutable query for entities that have both `Text` and `Paragraph` components.
///
/// # Behavior
/// The system clones the `Paragraph::text` value into the corresponding `Text` component,
/// effectively synchronizing the displayed text with the paragraph content.
///
/// # Example
/// If a `Paragraph` contains `"Hello, world!"`, the associated `Text` will be updated to show it.
fn update_text(mut query: Query<(&mut Text, &Paragraph), With<Paragraph>>) {
    for (mut text, p) in query.iter_mut() {
        text.0 = p.text.clone();
    }
}

/// Handles click events on paragraph nodes.
///
/// Sets the focused state of the paragraph to `true` and updates the globally tracked current widget ID.
///
/// # Parameters
/// - `trigger`: The pointer click trigger containing the target entity.
/// - `query`: Query to access mutable UI widget state and generation ID of paragraphs.
/// - `current_widget_state`: Mutable resource tracking the currently focused widget ID.
fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Paragraph>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

/// Handles the pointer cursor entering paragraph nodes.
///
/// Sets the hovered state of the paragraph to `true`.
///
/// # Parameters
/// - `trigger`: The pointer over trigger containing the target entity.
/// - `query`: Query to access mutable UI widget state of paragraphs.
fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Paragraph>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

/// Handles pointer cursor leaving paragraph nodes.
///
/// Sets the hovered state of the paragraph to `false`.
///
/// # Parameters
/// - `trigger`: The pointer out trigger containing the target entity.
/// - `query`: Query to access mutable UI widget state of paragraphs.
fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Paragraph>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}