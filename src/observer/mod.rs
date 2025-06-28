pub mod time_tick_trigger;
pub mod widget_init_trigger;

use bevy::prelude::*;
use crate::observer::time_tick_trigger::TimeTickTriggerObserver;
use crate::observer::widget_init_trigger::WidgetInitTrigger;

pub struct ObserverRegistryPlugin;

impl Plugin for ObserverRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TimeTickTriggerObserver,
            WidgetInitTrigger
        ));
    }
}