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

/// Resource storing a one-time initialization timer for UI widgets.
///
/// # Fields
/// - `timer`: A [`Timer`] that runs once, set to 0.3 seconds.
#[derive(Resource)]
struct UiInitTimer {
    timer: Timer,
}

pub struct WidgetInitTrigger;

impl Plugin for WidgetInitTrigger {
    fn build(&self, app: &mut App) {
        app.add_event::<WidgetInit>();
        app.insert_resource(UiInitTimer {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        });
        app.add_systems(Startup, pre_init);
        app.add_systems(Update, init_widget);
    }
}

/// System that triggers widget initialization events immediately during startup.
///
/// Iterates over all entities with a [`WidgetId`] component and sends a [`WidgetInit`] event.
///
/// # Parameters
/// - `commands`: The [`Commands`] to issue events.
/// - `query`: Query over entities with a [`WidgetId`] component.
fn pre_init(
    mut commands: Commands,
    query: Query<(Entity, &WidgetId)>
) {
    for (entity, data) in query.iter() {
        commands.trigger_targets(WidgetInit { target: entity, widget_data: data.clone() }, entity);
        debug!("Pre-Init widget successfully");
    }
}

/// System that triggers widget initialization events after a delay (300 ms) if required.
///
/// Controlled by the `UiInitResource` flag and uses [`UiInitTimer`] for timing.
///
/// # Parameters
/// - `commands`: The [`Commands`] to issue events.
/// - `query`: Query over entities with a [`WidgetId`] component.
/// - `ui_init`: Mutable reference to `UiInitResource` controlling whether to trigger initialization.
/// - `time`: Time resource to track frame delta.
/// - `timer_res`: Mutable reference to [`UiInitTimer`] controlling the delay.
fn init_widget(
    mut commands: Commands,
    query: Query<(Entity, &WidgetId)>,
    mut ui_init: ResMut<UiInitResource>,
    time: Res<Time>,
    mut timer_res: ResMut<UiInitTimer>,
) {
    if ui_init.0 {
        if timer_res.timer.tick(time.delta()).finished() {
            for (entity, data) in query.iter() {
                commands.trigger_targets(WidgetInit {
                    target: entity,
                    widget_data: data.clone(),
                }, entity);
            }
            debug!("Init widget successfully after (300ms)");
            ui_init.0 = false;
        }
    }
}