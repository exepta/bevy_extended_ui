use bevy::asset::AssetEvent;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::io::CssAsset;
use crate::styles::CssSource;

#[derive(Component)]
pub struct CssDirty;

pub struct HtmlReloadPlugin;

impl Plugin for HtmlReloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mark_css_users_dirty_on_css_change);
    }
}

fn mark_css_users_dirty_on_css_change(
    mut commands: Commands,
    mut css_events: MessageReader<AssetEvent<CssAsset>>,
    query: Query<(Entity, &CssSource)>,
) {
    // Collect changed/removed CSS asset ids (fast lookup).
    let mut changed: HashSet<AssetId<CssAsset>> = HashSet::new();
    for ev in css_events.read() {
        match ev {
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                changed.insert(*id);
            }
            _ => {}
        }
    }

    if changed.is_empty() {
        return;
    }

    // Mark all entities that reference any changed CssAsset.
    for (entity, css_source) in query.iter() {
        let uses_changed = css_source
            .0
            .iter()
            .any(|h| changed.contains(&h.id()));

        if uses_changed {
            // IMPORTANT: entity might have been despawned by HTML hot reload
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.insert(CssDirty);
            }
        }
    }
}
