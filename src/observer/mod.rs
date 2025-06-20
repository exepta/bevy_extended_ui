pub mod time_tick_trigger;

use bevy::prelude::*;
use crate::observer::time_tick_trigger::TimeTickTriggerObserver;

pub struct ObserverRegistryPlugin;

impl Plugin for ObserverRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TimeTickTriggerObserver);
    }
}