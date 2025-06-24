use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache, UIGenID, UIWidgetState};
use crate::service::image_cache_service::get_or_load_image;
use crate::styling::convert::{CssSource, TagName};
use crate::styling::paint::Colored;
use crate::widgets::{Img, WidgetId, WidgetKind};

#[derive(Component)]
struct ImageBase;

pub struct ImageWidget;

impl Plugin for ImageWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (internal_node_creation_system, update_src));
    }
}

/// Initializes internal UI nodes for all [`Img`] components not yet marked with [`ImageBase`].
///
/// This system creates image-rendering UI nodes from `<img>` elements, applying
/// default styling, image loading via [`AssetServer`], and caching via [`ImageCache`].
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

        let mut image = ImageNode::default();
        if let Some(path) = img.src.clone() {
            let handle = get_or_load_image(
                path.as_str(),
                &mut image_cache,
                &mut images,
                &asset_server,
            );
            
            image = ImageNode {
                image: handle,
                ..default()
            };
        }

        commands.entity(entity).insert((
            Name::new(format!("Img-{}", img.w_count)),
            Node::default(),
            WidgetId {
                id: img.w_count,
                kind: WidgetKind::Img
            },
            image,
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            Pickable::default(),
            css_source,
            TagName("img".to_string()),
            RenderLayers::layer(*layer),
            ImageBase,
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
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
    mut query: Query<(&mut ImageNode, &mut UIWidgetState, &Img), (With<Img>, Changed<Img>)>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for (image, mut state, img) in query.iter_mut() {
        let mut current_img = image;
        if let Some(path) = img.src.clone() {
            let handle = get_or_load_image(
                path.as_str(),
                &mut image_cache,
                &mut images,
                &asset_server,
            );

            *current_img = ImageNode {
                image: handle,
                ..default()
            };
            
            state.checked = state.checked;
        }
    }
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
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Img>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

/// Marks an [`Img`] node as hovered when the cursor enters.
///
/// # Triggered By:
/// - `Trigger<Pointer<Over>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Img>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

/// Unsets the hover state of an [`Img`] node when the cursor exits.
///
/// # Triggered By:
/// - `Trigger<Pointer<Out>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Img>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}