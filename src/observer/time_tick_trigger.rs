use bevy::prelude::*;
use crate::widgets::WidgetId;

/// Event representing a tick targeted at a specific widget entity.
///
/// Contains the target entity and the associated widget data.
#[derive(Message, EntityEvent)]
pub struct TimeTick {
    /// The entity that is the target of this tick event.
    pub entity: Entity,
    /// The widget data identifier associated with the target.
    pub widget_data: WidgetId,
}

/// Plugin that observes and emits `TimeTick` events every update cycle.
///
/// This plugin adds the `TimeTick` event type and a system that emits
/// `TimeTick` events for all entities with a `WidgetId` component.
pub struct TimeTickTriggerObserver;

impl Plugin for TimeTickTriggerObserver {
    /// Builds the plugin by registering the `TimeTick` event
    /// and adding the `emit_time_tick` system to the update schedule.
    ///
    /// # Parameters
    /// - `app`: Mutable reference to the Bevy app builder.
    fn build(&self, app: &mut App) {
        app.add_message::<TimeTick>();
        app.add_systems(Update, emit_time_tick);
    }
}

/// System that emits `TimeTick` events for all entities with a `WidgetId`.
///
/// Iterates through all entities with a `WidgetId` component and triggers
/// a `TimeTick` event targeting each entity.
///
/// # Parameters
/// - `commands`: Bevy commands for issuing events.
/// - `query`: Query that retrieves all entities with `WidgetId`.
fn emit_time_tick(mut commands: Commands, query: Query<(Entity, &WidgetId)>) {
    for (entity, data) in query.iter() {
        commands.trigger(TimeTick { entity, widget_data: data.clone() });
    }
}