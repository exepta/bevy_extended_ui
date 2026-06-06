use crate::{ExtendedUiConfiguration, ImageCache};
use base64::Engine;
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
#[cfg(all(feature = "svg", not(target_arch = "wasm32")))]
use resvg::{tiny_skia::Pixmap, usvg::Options};
use std::borrow::Cow;
use std::fs;
use std::path::{Component, Path, PathBuf};
pub const DEFAULT_CHECK_MARK_KEY: &str = "extended_ui/icons/check-mark.png";
pub const DEFAULT_CHOICE_BOX_KEY: &str = "extended_ui/icons/drop-arrow.png";
pub const DEFAULT_COLOR_KEY: &str = "extended_ui/icons/color.png";
const EMBEDDED_CHECK_MARK: &[u8] = include_bytes!("../../assets/extended_ui/icons/check-mark.png");
const EMBEDDED_DROP_ARROW: &[u8] = include_bytes!("../../assets/extended_ui/icons/drop-arrow.png");
const EMBEDDED_COLOR: &[u8] = include_bytes!("../../assets/extended_ui/icons/color.png");

/// Plugin that manages image caching and preload.
pub struct ImageCacheService;

impl Plugin for ImageCacheService {
    /// Registers image cleanup and preload systems.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            cleanup_unused_images_full.run_if(resource_changed::<ImageCache>),
        );
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
    images: &mut Assets<Image>,
    asset_server: &AssetServer,
) -> Handle<Image> {
    let path = normalize_asset_path(path);
    let path = path.as_ref();

    if path.is_empty() {
        return Handle::default();
    }

    if let Some(handle) = image_cache.map.get(path) {
        return handle.clone();
    }

    if let Some(handle) = load_image_from_data_url(path, images) {
        image_cache.map.insert(path.to_string(), handle.clone());
        return handle;
    }

    if let Some(handle) = load_image_from_filesystem(path, images) {
        image_cache.map.insert(path.to_string(), handle.clone());
        return handle;
    }

    #[cfg(all(feature = "svg", not(target_arch = "wasm32")))]
    if path_is_svg(path) {
        if let Some(handle) = load_svg_image_from_project(path, images) {
            image_cache.map.insert(path.to_string(), handle.clone());
            return handle;
        }

        warn!(
            "Failed to rasterize SVG at '{}', falling back to AssetServer load.",
            path
        );
    }

    if Path::new(path).is_absolute() {
        warn!(
            "Image file '{}' could not be loaded from the filesystem.",
            path
        );
        return Handle::default();
    }

    if let Some(embedded_png) = embedded_icon_bytes(path) {
        if !asset_exists_in_project(path) {
            warn!("Image not found at '{}', using embedded fallback.", path);

            let image = Image::from_buffer(
                embedded_png,
                ImageType::Extension("png"),
                CompressedImageFormats::empty(),
                true,
                ImageSampler::default(),
                RenderAssetUsages::default(),
            )
            .expect("Failed to create image from embedded PNG");

            let fallback_handle = images.add(image);
            image_cache
                .map
                .insert(path.to_string(), fallback_handle.clone());
            return fallback_handle;
        }
    }

    let owned_path = path.to_string();
    let handle: Handle<Image> = asset_server.load(owned_path.clone());

    image_cache.map.insert(path.to_string(), handle.clone());
    handle
}

/// Handles `load_image_from_filesystem` in the extended UI workflow.
fn load_image_from_filesystem(path: &str, images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let fs_path = Path::new(path);
    if !fs_path.is_file() {
        return None;
    }

    let ext = fs_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)?;

    if !matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "ico"
    ) {
        return None;
    }

    let bytes = fs::read(fs_path).ok()?;
    let image = Image::from_buffer(
        &bytes,
        ImageType::Extension(ext.as_str()),
        CompressedImageFormats::empty(),
        true,
        ImageSampler::default(),
        RenderAssetUsages::default(),
    )
    .ok()?;

    Some(images.add(image))
}

/// Handles `load_image_from_data_url` in the extended UI workflow.
fn load_image_from_data_url(path: &str, images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let raw = path.trim();
    if !raw.starts_with("data:") {
        return None;
    }

    let (meta, payload) = raw.split_once(',')?;
    if !meta.contains(";base64") {
        return None;
    }

    let mime = meta
        .strip_prefix("data:")
        .and_then(|value| value.split(';').next())
        .unwrap_or_default();
    if !mime.starts_with("image/") {
        return None;
    }

    let extension = match mime {
        "image/png" => "png",
        "image/jpeg" => "jpeg",
        "image/jpg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        _ => return None,
    };

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.as_bytes())
        .ok()?;

    let image = Image::from_buffer(
        &bytes,
        ImageType::Extension(extension),
        CompressedImageFormats::empty(),
        true,
        ImageSampler::default(),
        RenderAssetUsages::default(),
    )
    .ok()?;

    Some(images.add(image))
}

/// Handles `embedded_icon_bytes` in the extended UI workflow.
fn embedded_icon_bytes(path: &str) -> Option<&'static [u8]> {
    match path {
        DEFAULT_CHECK_MARK_KEY => Some(EMBEDDED_CHECK_MARK),
        DEFAULT_CHOICE_BOX_KEY => Some(EMBEDDED_DROP_ARROW),
        DEFAULT_COLOR_KEY => Some(EMBEDDED_COLOR),
        _ => None,
    }
}

/// Handles `asset_exists_in_project` in the extended UI workflow.
fn asset_exists_in_project(path: &str) -> bool {
    resolve_asset_fs_path(path).exists()
}

/// Handles `resolve_asset_fs_path` in the extended UI workflow.
fn resolve_asset_fs_path(path: &str) -> PathBuf {
    let raw = Path::new(path);
    if raw.is_absolute() || raw.starts_with("assets") {
        return raw.to_path_buf();
    }

    Path::new("assets").join(raw)
}

/// Builds an sRGB RGBA texture with linear sampling from raw pixel bytes.
fn rgba8_srgb_linear_image(width: u32, height: u32, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}

/// Handles `path_is_svg` in the extended UI workflow.
#[cfg(all(feature = "svg", not(target_arch = "wasm32")))]
fn path_is_svg(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

/// Handles `load_svg_image_from_project` in the extended UI workflow.
#[cfg(all(feature = "svg", not(target_arch = "wasm32")))]
fn load_svg_image_from_project(path: &str, images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let fs_path = resolve_asset_fs_path(path);
    let bytes = fs::read(&fs_path).ok()?;

    let tree = resvg::usvg::Tree::from_data(&bytes, &Options::default()).ok()?;
    let size = tree.size().to_int_size();

    let mut pixmap = Pixmap::new(size.width(), size.height())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let image = rgba8_srgb_linear_image(size.width(), size.height(), pixmap.take());
    Some(images.add(image))
}

/// Handles `supported_image_extensions` in the extended UI workflow.
fn supported_image_extensions() -> Vec<&'static str> {
    #[cfg(feature = "svg")]
    {
        vec!["png", "jpg", "jpeg", "webp", "svg"]
    }

    #[cfg(not(feature = "svg"))]
    {
        vec!["png", "jpg", "jpeg", "webp"]
    }
}

/// Handles `normalize_asset_path` in the extended UI workflow.
fn normalize_asset_path(path: &str) -> Cow<'_, str> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Cow::Borrowed(trimmed);
    }

    let absolute = Path::new(trimmed);
    if absolute.is_absolute() && !should_treat_leading_slash_as_asset_path(absolute) {
        return Cow::Borrowed(trimmed);
    }

    if let Some(stripped) = trimmed.strip_prefix('/') {
        return Cow::Owned(stripped.to_string());
    }

    Cow::Borrowed(trimmed)
}

fn should_treat_leading_slash_as_asset_path(path: &Path) -> bool {
    if path.exists() || path.parent().is_some_and(Path::exists) {
        return false;
    }

    true
}

/// Handles `to_assets_relative_path` in the extended UI workflow.
fn to_assets_relative_path(path: &Path) -> Option<String> {
    let mut found_assets = false;
    let mut relative = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) if found_assets => relative.push(part),
            Component::Normal(part) if part == std::ffi::OsStr::new("assets") => {
                found_assets = true
            }
            _ => {}
        }
    }

    if !found_assets || relative.as_os_str().is_empty() {
        return None;
    }

    Some(relative.to_string_lossy().replace('\\', "/"))
}

/// Preloads images from the configured assets folder into the cache.
pub fn pre_load_assets(
    extended_ui_configuration: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    #[cfg(all(feature = "svg", not(target_arch = "wasm32")))] mut images: ResMut<Assets<Image>>,
) {
    let folder = extended_ui_configuration.assets_path.clone();
    let folder = Path::new(&folder);
    if !folder.exists() {
        warn!(
            "pre_load_assets: Folder '{}' does not exist",
            folder.display()
        );
        return;
    }

    let supported_extensions = supported_image_extensions();

    let read_dir = match fs::read_dir(folder) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            warn!(
                "pre_load_assets: Failed to read asset folder '{}': {}",
                folder.display(),
                error
            );
            return;
        }
    };

    for entry in read_dir {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if supported_extensions.contains(&ext_lower.as_str()) {
                        if let Some(asset_str) = to_assets_relative_path(&path) {
                            let owned_path = normalize_asset_path(asset_str.as_str()).into_owned();

                            #[cfg(all(feature = "svg", not(target_arch = "wasm32")))]
                            if ext_lower == "svg" {
                                if let Some(handle) =
                                    load_svg_image_from_project(owned_path.as_str(), &mut images)
                                {
                                    image_cache.map.insert(owned_path.clone(), handle);
                                    debug!("Preloaded svg image: {}", owned_path);
                                } else {
                                    warn!(
                                        "Failed to preload SVG image '{}': rasterization failed",
                                        owned_path
                                    );
                                }
                                continue;
                            }

                            let handle: Handle<Image> = asset_server.load(owned_path.clone());
                            image_cache.map.insert(owned_path.clone(), handle.clone());
                            debug!("Preloaded image: {}", owned_path);
                        } else {
                            warn!(
                                "pre_load_assets: Skipping '{}' because no 'assets/' relative path could be derived.",
                                path.display()
                            );
                        }
                    }
                }
            }
        }
    }
}
