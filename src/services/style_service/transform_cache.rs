use bevy::prelude::*;
use bevy::ui::UiTransform;

/// Component caching the last computed UI transform.
#[derive(Component, Debug, Clone, Copy)]
pub struct LastUiTransform(pub UiTransform);

/// Updates the cached UI transform after styles are applied.
pub fn sync_last_ui_transform(
    mut commands: Commands,
    mut query: Query<
        (Entity, &UiTransform, Option<&mut LastUiTransform>),
        Or<(
            Added<UiTransform>,
            Changed<UiTransform>,
            Without<LastUiTransform>,
        )>,
    >,
) {
    for (entity, transform, last_opt) in query.iter_mut() {
        if let Some(mut last) = last_opt {
            last.0 = *transform;
        } else {
            commands.entity(entity).insert(LastUiTransform(*transform));
        }
    }
}
