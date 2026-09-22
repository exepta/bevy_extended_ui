use bevy::prelude::*;
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage, PrimaryWindow, SystemCursorIcon};

use crate::ImageCache;
use crate::services::image_service::get_or_load_image;
use crate::styles::CursorStyle;
use crate::styles::components::UiStyle;
use crate::widgets::UIWidgetState;

/// Resource tracking the currently applied CSS cursor state.
#[derive(Resource, Default)]
pub(super) struct CssCursorState {
    active: bool,
    previous: Option<CursorIcon>,
}

/// Updates the OS cursor icon based on hovered widget styles.
pub(super) fn update_css_cursor_icons(
    mut commands: Commands,
    mut cursor_state: ResMut<CssCursorState>,
    mut window_q: Query<(Entity, Option<&mut CursorIcon>), With<PrimaryWindow>>,
    hovered_q: Query<(&UiStyle, &UIWidgetState)>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok((window_entity, mut cursor_opt)) = window_q.single_mut() else {
        return;
    };

    let mut desired_cursor: Option<(CursorStyle, i32)> = None;
    let mut best_z = i32::MIN;

    for (ui_style, state) in hovered_q.iter() {
        if !state.hovered {
            continue;
        }

        let Some(active) = ui_style.active_style.as_ref() else {
            continue;
        };

        let Some(cursor) = active.cursor.clone() else {
            continue;
        };

        let z = active.z_index.unwrap_or(0);
        if desired_cursor.is_none() || z > best_z {
            desired_cursor = Some((cursor, z));
            best_z = z;
        }
    }

    if let Some((cursor_style, _)) = desired_cursor {
        let new_icon = match cursor_style {
            CursorStyle::System(system_icon) => CursorIcon::from(system_icon),
            CursorStyle::Custom(path) => {
                let handle =
                    get_or_load_image(path.as_str(), &mut image_cache, &mut images, &asset_server);
                if images.get(handle.id()).is_none() {
                    return;
                }
                CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                    handle,
                    hotspot: (0, 0),
                    ..default()
                }))
            }
        };
        if !cursor_state.active {
            cursor_state.previous = cursor_opt.as_deref().cloned();
            cursor_state.active = true;
        }

        if let Some(cursor) = cursor_opt.as_deref_mut() {
            if *cursor != new_icon {
                *cursor = new_icon;
            }
        } else {
            commands.entity(window_entity).insert(new_icon);
        }
    } else if cursor_state.active {
        cursor_state.active = false;
        let restore_icon = cursor_state
            .previous
            .take()
            .unwrap_or_else(|| CursorIcon::from(SystemCursorIcon::Default));

        if let Some(cursor) = cursor_opt.as_deref_mut() {
            if *cursor != restore_icon {
                *cursor = restore_icon;
            }
        } else {
            commands.entity(window_entity).insert(restore_icon);
        }
    }
}
