use crate::styles::paint::Colored;
use crate::styles::{CssSource, TagName};
use crate::widgets::{Form, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Marker component for initialized form widgets.
#[derive(Component)]
struct FormBase;

/// Plugin that wires up form widget behavior.
pub struct FormWidget;

impl Plugin for FormWidget {
    /// Registers systems for form widget setup.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

/// Creates the base node for form widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Form, Option<&CssSource>), (With<Form>, Without<FormBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, form, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Form-{}", form.entry)),
                Node::default(),
                WidgetId {
                    id: form.entry,
                    kind: WidgetKind::Form,
                },
                ImageNode::default(),
                BackgroundColor::default(),
                BorderColor::default(),
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
                TagName("form".to_string()),
                RenderLayers::layer(*layer),
                FormBase,
                UIWidgetState::default(),
            ))
            .observe(on_form_click)
            .observe(on_form_cursor_entered)
            .observe(on_form_cursor_leave);
    }
}

/// Sets focus on form click and updates the current widget state.
fn on_form_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Form>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        if state.disabled {
            trigger.propagate(false);
            return;
        }
        state.focused = true;
        current_widget_state.widget_id = gen_id.get();
    }
    trigger.propagate(false);
}

/// Sets hovered state when the cursor enters a form.
fn on_form_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Form>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

/// Clears hovered state when the cursor leaves a form.
fn on_form_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Form>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}
