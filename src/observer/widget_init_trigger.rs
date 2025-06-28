use bevy::prelude::*;
use crate::registry::UiInitResource;
use crate::widgets::WidgetId;

/// Event fired to initialize a widget.
///
/// # Fields
/// - `target`: The [`Entity`] representing the widget to initialize.
/// - `widget_data`: The [`WidgetId`] containing initialization data for the widget.
#[derive(Event)]
pub struct WidgetInit {
    pub target: Entity,
    pub widget_data: WidgetId,
}

pub struct WidgetInitTrigger;

impl Plugin for WidgetInitTrigger {
    fn build(&self, app: &mut App) {
        app.add_event::<WidgetInit>();
        app.add_systems(Update, init_widget);
    }
}

fn init_widget(
    mut commands: Commands,
    query: Query<(Entity, &WidgetId, &Visibility)>,
    mut ui_init: ResMut<UiInitResource>,
) {
    if ui_init.0 {
        for (entity, data, vis) in query.iter() {
            commands.trigger_targets(WidgetInit {
                target: entity,
                widget_data: data.clone(),
            }, entity);
            if !vis.eq(&Visibility::Hidden) {
                ui_init.0 = false;
            }
        }
        debug!("Init widget successfully");
    }
}