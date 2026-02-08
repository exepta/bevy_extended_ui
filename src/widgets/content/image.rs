use bevy::asset::LoadState;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::services::image_service::get_or_load_image;
use crate::styles::paint::Colored;
use crate::styles::{CssSource, TagName};
use crate::widgets::{Img, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};

#[derive(Component)]
struct ImageBase;

#[derive(Component)]
struct AltTextNode;

/// Stores the spawned alt-text child entity so we can update/remove it without scanning Children.
#[derive(Component, Copy, Clone)]
struct AltTextChild(Entity);

/// Tracks what we *last applied* so we don't spam updates/logs every frame.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
enum ImgFallbackState {
    None,
    AltShown,
}

/// Caches the last alt text we wrote into the child.
/// Prevents re-inserting Text every frame.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
struct AltTextCached(String);

pub struct ImageWidget;

impl Plugin for ImageWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                update_src,
                sync_alt_text_with_image_state, // needed for "missing file => alt text"
            ),
        );
    }
}

/// Initializes internal UI nodes for all [`Img`] components not yet marked with [`ImageBase`].
///
/// This system creates image-rendering UI nodes from `<img>` elements, applying
/// default styling, image loading via [`AssetServer`], and caches via [`ImageCache`].
///
/// It sets up:
/// - [`ImageNode`] with an optional image handle from `img.src`
/// - CSS styling components (`BackgroundColor`, `BorderColor`, `BoxShadow`, etc.)
/// - [`CssSource`] if provided
/// - [`RenderLayers`] for UI layer control
/// - [`TagName`] set to `"img"`
/// - [`Name`] using the image's internal counter (`w_count`)
///
/// Also attaches pointer event observers:
/// - [`on_internal_click`] → focuses the image widget
/// - [`on_internal_cursor_entered`] → sets hover state to true
/// - [`on_internal_cursor_leave`] → sets hover state to false
///
/// # Parameters
/// - `commands`: To insert components onto entities
/// - `query`: Finds [`Img`] entities missing [`ImageBase`]
/// - `config`: Provides render layer configuration
/// - `asset_server`: Loads assets if not already cached
/// - `image_cache`: Caches loaded image handles to avoid reloading
/// - `images`: Asset container for `Image` handles
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Img, Option<&CssSource>), (With<Img>, Without<ImageBase>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, img, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        let mut image_node = ImageNode::default();

        assign_image_from_src(
            &mut image_node,
            img,
            &asset_server,
            &mut image_cache,
            &mut images,
        );

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Img-{}", img.entry)),
                Node::default(),
                WidgetId {
                    id: img.entry,
                    kind: WidgetKind::Img,
                },
                image_node,
                BackgroundColor::default(),
                BorderColor::default(),
                BoxShadow::new(
                    Colored::TRANSPARENT,
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                ),
                ZIndex::default(),
                Pickable::default(),
                css_source,
                TagName("img".to_string()),
                RenderLayers::layer(*layer),
                ImageBase,
            ))
            .insert(ImgFallbackState::None)
            .insert(AltTextCached(String::new()))
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);

        // If src is empty, show alt immediately (no need to wait for any load state).
        let src_empty = is_src_empty(img);
        if src_empty {
            let child = spawn_or_update_alt_text_child(&mut commands, entity, None, &img.alt);
            if let Some(child) = child {
                commands.entity(entity).insert(AltTextChild(child));
            }
            // Mark as applied so the sync system won't spam.
            commands.entity(entity).insert(ImgFallbackState::AltShown);
        }
    }
}

/// Updates the `ImageNode` texture for UI widgets when the associated `Img` component changes.
///
/// <p>
/// This system listens for changes to the `Img` component and updates the corresponding
/// `ImageNode` by loading the image from the specified `src` path. If the path is already
/// cached, the existing handle is reused. The `UIWidgetState` is also accessed to allow future
/// extensions (e.g., reacting to image changes).
/// </p>
///
/// # Parameters
/// - `query`: A query that retrieves all entities with mutable access to `ImageNode`,
///   `UIWidgetState`, and an `Img` component, filtered by the `Changed<Img>` condition.
/// - `asset_server`: A handle to Bevy's asset server for loading images from disk.
/// - `image_cache`: A mutable reference to an image cache used to avoid reloading assets.
/// - `images`: A mutable reference to the global asset collection of loaded `Image` assets.
///
/// # Behavior
/// For each changed `Img`:
/// - If the `src` field is `Some`, the image is loaded or reused from cache.
/// - The `ImageNode` is updated with the new image handle.
/// - The `UIWidgetState` is accessed (currently unchanged, but ready for future use).
///
/// # See Also
/// - [`get_or_load_image`]: Utility function to cache or load images from a path.
/// - [`ImageNode`]: Component that defines image appearance in the UI.
/// - [`Img`]: Component holding the `src` image path for UI image widgets.
fn update_src(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut ImageNode,
            &mut UIWidgetState,
            &Img,
            Option<&AltTextChild>,
            &mut ImgFallbackState,
            &mut AltTextCached,
        ),
        (With<Img>, Changed<Img>),
    >,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, mut image_node, _state, img, alt_child, mut fb_state, mut cached) in
        query.iter_mut()
    {
        let existing_child = alt_child.map(|c| c.0);

        // Always update the image handle if src is non-empty.
        assign_image_from_src(
            &mut image_node,
            img,
            &asset_server,
            &mut image_cache,
            &mut images,
        );

        // If src is empty -> show alt immediately (and mark state).
        if is_src_empty(img) {
            let child =
                spawn_or_update_alt_text_child(&mut commands, entity, existing_child, &img.alt);
            if let Some(child) = child {
                commands.entity(entity).insert(AltTextChild(child));
                set_cached_alt_if_changed(&mut commands, entity, &mut cached, &img.alt);
            }
            *fb_state = ImgFallbackState::AltShown;
        } else {
            // src changed to something non-empty:
            // don't remove alt *here* (async load). The sync system will remove it on Loaded.
            // But we should reset the fallback state so we can log on the next real transition.
            *fb_state = ImgFallbackState::None;
        }
    }
}

/// Keeps alt-text in sync with the actual load state.
/// This is required because asset loading is async and does NOT trigger Changed<Img>.
fn sync_alt_text_with_image_state(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<
        (
            Entity,
            &Img,
            &ImageNode,
            Option<&AltTextChild>,
            &ImgFallbackState,
            &AltTextCached,
        ),
        With<Img>,
    >,
) {
    for (entity, img, image_node, alt_child, fb_state, cached) in query.iter() {
        let existing_child = alt_child.map(|c| c.0);

        // src empty -> alt is handled by creation/update_src already; avoid per-frame work.
        if is_src_empty(img) {
            continue;
        }

        // Decide based on the *actual* current load state.
        match asset_server.get_load_state(image_node.image.id()) {
            Some(LoadState::Loaded) => {
                // Only do work if alt is currently shown.
                if *fb_state == ImgFallbackState::AltShown {
                    remove_alt_text_children(&mut commands, entity, existing_child);
                    commands.entity(entity).insert(ImgFallbackState::None);
                    // Log only on transition/action.
                    debug!("Image loaded again, removing alt text: {:?}", entity);
                }
            }
            Some(LoadState::Failed(_)) => {
                // Only do work if alt is NOT currently shown.
                if *fb_state != ImgFallbackState::AltShown {
                    let child = spawn_or_update_alt_text_child(
                        &mut commands,
                        entity,
                        existing_child,
                        &img.alt,
                    );
                    if let Some(child) = child {
                        commands.entity(entity).insert(AltTextChild(child));
                        // Cache alt to avoid pointless text inserts later.
                        let mut cached_local = cached.clone();
                        set_cached_alt_if_changed(
                            &mut commands,
                            entity,
                            &mut cached_local,
                            &img.alt,
                        );
                    }
                    commands.entity(entity).insert(ImgFallbackState::AltShown);
                    // Log only on transition/action.
                    debug!("[WARN] Image failed to load, using alt text: {:?}", img.alt);
                } else {
                    // Alt already shown. Only update text if alt actually changed.
                    if cached.0 != img.alt.trim() {
                        if let Some(child) = existing_child {
                            commands
                                .entity(child)
                                .insert(Text::new(img.alt.trim().to_string()));
                            commands
                                .entity(entity)
                                .insert(AltTextCached(img.alt.trim().to_string()));
                            debug!("Alt text changed, updating child for Img: {:?}", entity);
                        }
                    }
                }
            }
            _ => {
                // Loading / NotLoaded / None -> do nothing to avoid flicker and spam.
            }
        }
    }
}

/// Loads an image handle from img.src if non-empty and assigns it to ImageNode.
/// This avoids duplicating the get_or_load_image logic.
fn assign_image_from_src(
    image_node: &mut ImageNode,
    img: &Img,
    asset_server: &Res<AssetServer>,
    image_cache: &mut ImageCache,
    images: &mut ResMut<Assets<Image>>,
) {
    if let Some(path) = img.src.clone().filter(|s| !s.trim().is_empty()) {
        let handle = get_or_load_image(path.as_str(), image_cache, images, asset_server);
        image_node.image = handle;
    }
}

fn is_src_empty(img: &Img) -> bool {
    img.src
        .as_ref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true)
}

fn set_cached_alt_if_changed(
    commands: &mut Commands,
    parent: Entity,
    cached: &mut AltTextCached,
    alt: &str,
) {
    let alt = alt.trim().to_string();
    if cached.0 != alt {
        cached.0 = alt.clone();
        commands.entity(parent).insert(AltTextCached(alt));
    }
}

fn spawn_or_update_alt_text_child(
    commands: &mut Commands,
    parent: Entity,
    existing_child: Option<Entity>,
    alt: &str,
) -> Option<Entity> {
    let alt = alt.trim();
    if alt.is_empty() {
        warn!("Alt text is empty for Img: {:?}", parent);
        return existing_child;
    }

    // Update instead of spawning duplicates.
    if let Some(child) = existing_child {
        // Don't spam inserts if it's the same text; we now gate updates elsewhere,
        // but keeping this safe is fine.
        commands.entity(child).insert(Text::new(alt.to_string()));
        return Some(child);
    }

    let child = commands
        .spawn((
            AltTextNode,
            Node::default(),
            Text::new(alt.to_string()),
            TextColor(Color::WHITE),
            TextFont::default(),
            TextLayout::default(),
        ))
        .id();

    commands.entity(parent).add_child(child);
    Some(child)
}

fn remove_alt_text_children(
    commands: &mut Commands,
    parent: Entity,
    existing_child: Option<Entity>,
) {
    let Some(child) = existing_child else { return };

    // Despawn alt child and clear tracking component.
    commands.entity(child).despawn();
    commands.entity(parent).remove::<AltTextChild>();
}

/// Handles pointer click on an [`Img`] element.
///
/// Sets the image's `UIWidgetState::focused` flag and updates the
/// `CurrentWidgetState` to track the selected widget ID.
///
/// # Triggered By:
/// - `Trigger<Pointer<Click>>`
///
/// # Affects:
/// - `UIWidgetState::focused`
/// - `CurrentWidgetState::widget_id`
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Img>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }

    trigger.propagate(false);
}

/// Marks an [`Img`] node as hovered when the cursor enters.
///
/// # Triggered By:
/// - `Trigger<Pointer<Over>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Img>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Unsets the hover state of an [`Img`] node when the cursor exits.
///
/// # Triggered By:
/// - `Trigger<Pointer<Out>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Img>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}
