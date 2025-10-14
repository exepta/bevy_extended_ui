use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, ImageCache, UIGenID, UIWidgetState};
use crate::service::image_cache_service::{get_or_load_image, DEFAULT_CHECK_MARK_KEY};
use crate::styling::paint::Colored;
use crate::widgets::{CheckBox, WidgetId, WidgetKind};

#[derive(Component)]
struct CheckBoxBase;

#[derive(Component)]
struct CheckBoxLabel;

#[derive(Component)]
pub struct CheckBoxMark;

pub struct CheckBoxWidget;

impl Plugin for CheckBoxWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

/// A system that initializes and spawns internal UI nodes for all entities with a [`CheckBox`]
/// component that do not yet have a [`CheckBoxBase`] marker.
///
/// This system generates a visual checkbox node consisting of:
/// - A container node (`CheckBoxBase`)
/// - A child "mark" box used to show a checkmark (`CheckBoxMark`)
/// - A child text label for the checkbox (`CheckBoxLabel`)
///
/// The generated nodes are styled using CSS-like metadata (e.g., [`CssClass`], [`CssSource`]),
/// and interactive behavior is hooked via `.observe(...)` calls.
///
/// # Parameters
/// - `commands`: Bevy's command buffer used to spawn and modify entities.
/// - `query`: A query for all UI checkbox entities needing visual generation.
/// - `config`: A resource containing UI configuration like `render_layers`.
///
/// # Behavior
/// - Inserts layout and style components (e.g., `Node`, `BackgroundColor`, `ZIndex`) on the main entity.
/// - Spawns two children:
///   - The visual mark box
///   - The label text
/// - Sets up input handlers with `.observe(...)` for click and hover behavior.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &CheckBox, Option<&CssSource>), (With<CheckBox>, Without<CheckBoxBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, checkbox, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("CheckBox-{}", checkbox.w_count)),
            Node {
                width: Val::Px(200.0),
                height: Val::Px(40.0),
                display: Display::Flex,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                ..default()
            },
            WidgetId {
                id: checkbox.w_count,
                kind: WidgetKind::CheckBox
            },
            BackgroundColor::default(),
            ImageNode::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            Pickable::default(),
            css_source.clone(),
            TagName(String::from("checkbox")),
            RenderLayers::layer(*layer),
            CheckBoxBase,
            children![
                (
                    Name::new(format!("Check-Mark-{}", checkbox.w_count)),
                    Node {
                      display: Display::Flex,
                      justify_content: JustifyContent::Center,
                      align_items: AlignItems::Center,
                      ..default()
                    },
                    BackgroundColor::default(),
                    ImageNode::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                    ZIndex::default(),
                    css_source.clone(),
                    UIWidgetState::default(),
                    CssClass(vec!["mark-box".to_string()]),
                    Pickable::IGNORE,
                    BindToID(id.0),
                    RenderLayers::layer(*layer),
                    CheckBoxMark,
                ),
                (
                    Name::new(format!("Check-Label-{}", checkbox.w_count)),
                    Text::new(checkbox.label.clone()),
                    TextColor::default(),
                    TextFont::default(),
                    TextLayout::default(),
                    ZIndex::default(),
                    css_source.clone(),
                    UIWidgetState::default(),
                    CssClass(vec!["check-text".to_string()]),
                    Pickable::IGNORE,
                    BindToID(id.0),
                    RenderLayers::layer(*layer),
                    CheckBoxLabel
                )
            ]
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Handles click interactions on UI checkboxes.
///
/// Toggles the `checked` state of the target [`CheckBox`] and visually updates the checkmark node
/// inside the [`CheckBoxMark`] entity. The system also updates [`UIWidgetState`] to reflect focus
/// and manages the dynamic creation or removal of a child mark node with an image icon.
///
/// # Parameters
/// - `trigger`: A [`On<Pointer<Click>>`] event generated by a user click.
/// - `commands`: Bevy's command buffer for entity modifications.
/// - `query`: The checkbox component and its state on the clicked entity.
/// - `inner_query`: All entities marked with [`CheckBoxMark`] to find the visual mark box for updating.
/// - `current_widget_state`: Global resource to store the ID of the currently active (clicked) widget.
/// - `config`: UI configuration, including render layer info.
/// - `asset_server`: Asset server used to load image assets.
/// - `image_cache`: Caches loaded image handles.
/// - `images`: The actual asset storage for image data.
///
/// # Behavior
/// - If a checkbox becomes checked, inserts a new child node under the `CheckBoxMark` with the checkmark image.
/// - If unchecked, despawns any child nodes under the `CheckBoxMark`.
/// - Sets `focused = true` and toggles `checked`.
fn on_internal_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    mut query: Query<(&mut UIWidgetState, &UIGenID, &CheckBox, &CssSource), With<CheckBox>>,
    inner_query: Query<(Entity, &BindToID, Option<&Children>, &ComputedNode), With<CheckBoxMark>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    if let Ok((mut state, gen_id, checkbox, css_source)) = query.get_mut(trigger.entity) {
        state.focused = true;
        state.checked = !state.checked;
        current_widget_state.widget_id = gen_id.0;

        for (entity, id, children_opt, computed_node) in inner_query.iter() {
            if gen_id.0 != id.0 {
                continue;
            }
            
            let width = computed_node.size.x / 1.5;
            let height = computed_node.size.y / 1.5;

            if state.checked {
                let mut child = None;
                commands.entity(entity).with_children(|builder| {
                    let in_child = builder.spawn((
                        Name::new(format!("Mark-{}", checkbox.w_count)),
                        Node {
                            width: Val::Px(width),
                            height: Val::Px(height),
                            ..default()
                        },
                        Pickable::IGNORE,
                        css_source.clone(),
                        UIWidgetState::default(),
                        CssClass(vec!["mark".to_string()]),
                        RenderLayers::layer(*layer),
                    )).id();
                    child = Some(in_child);
                });

                if let Some(child) = child {
                    let handle = get_or_load_image(
                        checkbox.icon_path.as_deref().unwrap_or(DEFAULT_CHECK_MARK_KEY),
                        &mut image_cache,
                        &mut images,
                        &asset_server,
                    );
                    
                    commands.entity(child).insert(ImageNode::new(handle.clone()));
                }
            } else {
                if let Some(children) = children_opt {
                    for child in children.iter() {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}

/// Handles cursor-entered events on checkboxes.
///
/// Sets the `hovered` flag of the corresponding [`UIWidgetState`] to `true`.
/// This enables hover styles (e.g., `:hover`) to apply.
///
/// # Parameters
/// - `trigger`: A [`On<Pointer<Over>>`] when the pointer enters the checkbox area.
/// - `query`: Query for the UI widget state to be modified.
fn on_internal_cursor_entered(
    trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<CheckBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
}

/// Handles cursor-leave events on checkboxes.
///
/// Sets the `hovered` flag of the corresponding [`UIWidgetState`] to `false`,
/// disabling hover styles (e.g., `:hover`).
///
/// # Parameters
/// - `trigger`: A [`On<Pointer<Out>>`] when the pointer leaves the checkbox area.
/// - `query`: Query for the UI widget state to be modified.
fn on_internal_cursor_leave(
    trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<CheckBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
}