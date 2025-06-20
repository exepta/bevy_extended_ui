use bevy::prelude::*;
use crate::html::{HtmlEventBindings, HtmlFunctionRegistry};

pub struct HtmlBindingService;

impl Plugin for HtmlBindingService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fetch_observers);
    }
}

/// Fetches and attaches observer systems to entities based on their HTML event bindings.
///
/// This system queries for entities that have newly added `HtmlEventBindings` components.
/// For each binding (like `onclick`, `onmouseenter`, `onmouseleave`, `onupdate`),
/// it looks up the corresponding system in the `HtmlFunctionRegistry` and attaches
/// it as an observer to the entity if found.
///
/// # Parameters
/// - `query`: Query for entities with newly added `HtmlEventBindings`.
/// - `registry`: Resource containing mappings from event names to system functions.
/// - `commands`: Commands to add observers to entities.
fn fetch_observers(
    query: Query<(Entity, &HtmlEventBindings), Added<HtmlEventBindings>>,
    registry: Res<HtmlFunctionRegistry>,
    mut commands: Commands
) {
    for (entity, bindings) in query.iter() {
        if let Some(name) = &bindings.onclick {
            if let Some(system) = registry.click.get(name) {
                commands.entity(entity).observe(system.clone());
            }
        }

        if let Some(name) = &bindings.onmouseenter {
            if let Some(system) = registry.over.get(name) {
                commands.entity(entity).observe(system.clone());
            }
        }

        if let Some(name) = &bindings.onmouseleave {
            if let Some(system) = registry.out.get(name) {
                commands.entity(entity).observe(system.clone());
            }
        }
        
        if let Some(name) = &bindings.onupdate {
            if let Some(system) = registry.update.get(name) {
                commands.entity(entity).observe(system.clone());
            }
        }
    }
}