use std::fs;
use std::path::Path;
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use crate::{ExtendedUiConfiguration, ImageCache};
pub const DEFAULT_CHECK_MARK_KEY: &str = "extended_ui/icons/check-mark.png";
pub const DEFAULT_CHOICE_BOX_KEY: &str = "extended_ui/icons/drop-arrow.png";

pub struct ImageCacheService;

impl Plugin for ImageCacheService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cleanup_unused_images_full.run_if(resource_changed::<ImageCache>));
        app.add_systems(Startup, pre_load_assets);
    }
}

/// Removes all image handles from the `ImageCache` that no longer exist in the `Assets<Image>`
/// resource. Also removes the corresponding `Image` assets from memory.
///
/// This function should be called periodically to clean up unused or unloaded images
/// from the image cache to free memory.
///
/// # Parameters
/// - `image_cache`: A mutable reference to the custom `ImageCache` resource,
///   which maps image paths to their handles.
/// - `images`: A mutable reference to the Bevy `Assets<Image>` asset storage.
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

/// Loads an image from disk (via the Bevy `AssetServer`) or returns a previously cached handle.
/// If the image is missing or fails to load, a built-in fallback PNG is used instead.
///
/// This function ensures that each unique image path is only loaded once and is reused
/// from the `ImageCache` on later requests.
///
/// # Parameters
/// - `path`: The relative path to the image file (e.g., `"assets/icons/icon.png"`).
/// - `image_cache`: A mutable reference to the custom `ImageCache` to store and reuse handles.
/// - `images`: A mutable reference to the Bevy `Assets<Image>` asset storage.
/// - `asset_server`: A reference to the Bevy `AssetServer` used to load external assets.
///
/// # Returns
/// A `Handle<Image>` pointing to the loaded or fallback image.
pub fn get_or_load_image(
    path: &str,
    image_cache: &mut ImageCache,
    images: &mut ResMut<Assets<Image>>,
    asset_server: &Res<AssetServer>,
) -> Handle<Image> {
    if let Some(handle) = image_cache.map.get(path) {
        return handle.clone();
    }

    let owned_path = path.to_string();
    let handle: Handle<Image> = asset_server.load(owned_path.clone());

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

pub fn pre_load_assets(
    extended_ui_configuration: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    let folder = extended_ui_configuration.assets_path.clone();
    let folder = Path::new(&folder);
    if !folder.exists() {
        warn!("pre_load_assets: Folder '{}' does not exist", folder.display());
        return;
    }

    let supported_extensions = ["png", "jpg", "jpeg"];

    for entry in fs::read_dir(folder).expect("Failed to read asset folder") {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if supported_extensions.contains(&ext.to_lowercase().as_str()) {
                        if let Some(asset_path) = path.strip_prefix("assets").ok() {
                            if let Some(asset_str) = asset_path.to_str() {
                                let owned_path = asset_str.to_string();

                                let handle: Handle<Image> = asset_server.load(owned_path.clone());
                                image_cache.map.insert(owned_path.clone(), handle.clone());

                                debug!("Preloaded image: {}", owned_path);
                            }
                        }
                    }
                }
            }
        }
    }
}