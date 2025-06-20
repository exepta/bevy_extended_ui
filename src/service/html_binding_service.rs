use bevy::prelude::*;
use crate::html::{HtmlEventBindings, HtmlFunctionRegistry};

pub struct HtmlBindingService;

impl Plugin for HtmlBindingService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fetch_observers);
    }
}

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
    }
}