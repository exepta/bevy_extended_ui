use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use crate::ImageCache;

pub const DEFAULT_CHECK_MARK_ICON: &[u8] = include_bytes!("../../assets/icons/check-mark.png");
pub const DEFAULT_CHECK_MARK_KEY: &str = "__embedded/bevy_extended_ui/check-mark.png";

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
    if let Some(existing) = image_cache.map.get(path) {
        return existing.clone();
    }

    info!("Loading {}, {}", path, DEFAULT_CHECK_MARK_KEY);
    
    let handle = if path == DEFAULT_CHECK_MARK_KEY {
        let image = Image::from_buffer(
            DEFAULT_CHECK_MARK_ICON,
            ImageType::Extension("png"),
            CompressedImageFormats::all(),
            true,
            ImageSampler::default(),
            RenderAssetUsages::MAIN_WORLD,
        )
            .expect("Failed to decode embedded check-mark icon");
        
        images.add(image)
    } else {
        asset_server.load(path)
    };

    image_cache.map.insert(path.to_string(), handle.clone());
    handle
}