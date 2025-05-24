use bevy::prelude::*;
use crate::ImageCache;

pub struct ImageCacheService;

impl Plugin for ImageCacheService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cleanup_unused_images_full.run_if(resource_changed::<ImageCache>));
    }
}

fn cleanup_unused_images_full(
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    image_cache.map.retain(|_path, handle| {
        let still_loaded = images.get(handle.clone().id()).is_some();
        if !still_loaded {
            images.remove(handle);
        }
        still_loaded
    });
}