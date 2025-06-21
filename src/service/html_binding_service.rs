use std::path::Path;
use bevy::prelude::*;
use crate::html::{HtmlEventBindings, HtmlFunctionRegistry, HtmlSource};
use crate::widgets::Widget;

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
    controller_query: Query<&HtmlSource, With<HtmlSource>>,
    query: Query<(Entity, &HtmlEventBindings, &Widget), Added<HtmlEventBindings>>,
    registry: Res<HtmlFunctionRegistry>,
    mut commands: Commands
) {
    for (entity, bindings, widget) in query.iter() {
        for source in controller_query.iter() {
            if let Some(controller) = source.controller.clone() {
                if let Some(widget_con) = widget.0.clone() {
                    if !controller.eq(&widget_con) {
                        return;
                    }
                    
                    if !exist_controller(&controller) {
                        error!("Controller path {} not found", controller);
                        return;
                    }
                }
            }
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
}

fn exist_controller(controller: &str) -> bool {
    let path = Path::new(controller);
    path.exists()
}