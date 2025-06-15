use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use crate::ImageCache;
pub const DEFAULT_CHECK_MARK_KEY: &str = "extended_ui/icons/check-mark.png";
pub const DEFAULT_CHOICE_BOX_KEY: &str = "extended_ui/icons/drop-arrow.png";

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

pub fn get_or_load_image(
    path: &str,
    image_cache: &mut ResMut<ImageCache>,
    images: &mut ResMut<Assets<Image>>,
    asset_server: &Res<AssetServer>,
) -> Handle<Image> {
    if let Some(handle) = image_cache.map.get(path) {
        return handle.clone();
    }
    
    let handle: Handle<Image> = asset_server.load(path);
    
    if handle.path().is_none() {
        warn!("Image not found at '{}', using embedded fallback.", path);

        let embedded_png = include_bytes!("../../assets/extended_ui/icons/check-mark.png");
        let image = Image::from_buffer(
            embedded_png,
            ImageType::Extension("png"),
            CompressedImageFormats::empty(),
            true,
            ImageSampler::default(),
            RenderAssetUsages::MAIN_WORLD,
        ).expect("Failed to create image from embedded PNG");

        let fallback_handle = images.add(image);
        image_cache.map.insert(path.to_string(), fallback_handle.clone());
        return fallback_handle;
    }

    image_cache.map.insert(path.to_string(), handle.clone());
    handle
}