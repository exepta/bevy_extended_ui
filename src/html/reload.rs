use bevy::asset::AssetEvent;
use bevy::prelude::*;

use crate::io::CssAsset;
use crate::styles::CssSource;

/// Marker component: this entity must have its CSS re-applied
/// because one of its referenced CssAsset files changed.
#[derive(Component)]
pub struct CssDirty;

/// Plugin that listens to CssAsset hot-reload events and marks
/// all entities that use the changed CssAsset as CssDirty.
///
/// NOTE:
/// - HTML hot reload is handled by HtmlConverterSystem + HtmlDirty.
/// - This plugin only marks CSS users.
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
    // Collect changed/removed CSS asset ids.
    let mut changed: Vec<AssetId<CssAsset>> = Vec::new();
    for ev in css_events.read() {
        match ev {
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                changed.push(*id);
            }
            _ => {}
        }
    }

    if changed.is_empty() {
        return;
    }

    // Mark all entities that reference any changed CssAsset.
    for (entity, css_source) in query.iter() {
        let uses_changed = css_source.0.iter().any(|h| changed.contains(&h.id()));
        if uses_changed {
            commands.entity(entity).insert(CssDirty);
        }
    }
}
