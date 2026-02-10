use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, IconPlace, TagName};
use crate::widgets::{BindToID, Button, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use crate::widgets::controls::place_icon_if;

/// Marker component for initialized button widgets.
#[derive(Component)]
struct ButtonBase;

/// Marker component for button text nodes.
#[derive(Component)]
struct ButtonText;

/// Plugin that registers button widget behavior.
pub struct ButtonWidget;

impl Plugin for ButtonWidget {
    /// Registers systems for button widget setup and updates.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (internal_node_creation_system, update_button_system));
    }
}

/// System that initializes internal UI nodes for [`Button`] components.
///
/// This system is responsible for spawning Bevy UI elements for every [`Button`] entity
/// that hasn't been initialized yet (i.e., does not contain [`ButtonBase`]).
///
/// Each button gets:
/// - A styled node container with a background, border, shadow, etc.
/// - A text child node
/// - An optional icon node (on left or right)
/// - Observers for pointer interaction
///
/// # Parameters
/// - `commands`: Used to insert components and spawn children.
/// - `query`: Finds all `Button` entities that haven't been initialized.
/// - `config`: UI configuration including rendering layer setup.
/// - `asset_server`: Used to load icons for buttons.
/// - `image_cache`: Caches icon handles to avoid reloading assets.
///
/// # Inserted Components
/// - [`Node`], [`ImageNode`], [`BackgroundColor`], [`BorderColor`], [`BorderRadius`], [`BoxShadow`]
/// - [`CssSource`], [`TagName`], [`CssClass`], [`RenderLayers`], [`ZIndex`], [`ButtonBase`]
/// - Observers for click and hover events
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &Button, Option<&CssSource>),
        (With<Button>, Without<ButtonBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, button, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Button-{}", button.entry)),
                Node::default(),
                WidgetId {
                    id: button.entry,
                    kind: WidgetKind::Button,
                },
                BackgroundColor::default(),
                ImageNode::default(),
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
                css_source.clone(),
                TagName("button".to_string()),
                RenderLayers::layer(*layer),
                ButtonBase,
            ))
            .with_children(|builder| {
                spawn_button_children(builder, button, id, *layer, css_source.clone(), &asset_server, &mut image_cache);
            })
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Rebuilds button children when the `Button` component changes.
fn update_button_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &Button, Option<&CssSource>, Option<&Children>),
        (With<Button>, With<ButtonBase>, Changed<Button>),
    >,
    text_q: Query<(), With<ButtonText>>,
    image_q: Query<(), With<crate::widgets::controls::ButtonImage>>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, button, source_opt, children_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        if let Some(children) = children_opt {
            for child in children.iter() {
                if commands.get_entity(child).is_ok()
                    && (text_q.get(child).is_ok() || image_q.get(child).is_ok())
                {
                    commands.entity(child).despawn();
                }
            }
        }

        commands.entity(entity).with_children(|builder| {
            spawn_button_children(
                builder,
                button,
                id,
                *layer,
                css_source.clone(),
                &asset_server,
                &mut image_cache,
            );
        });
    }
}

/// Spawns icon and text children for a button entity.
fn spawn_button_children(
    builder: &mut RelatedSpawnerCommands<ChildOf>,
    button: &Button,
    id: &UIGenID,
    layer: usize,
    css_source: CssSource,
    asset_server: &Res<AssetServer>,
    image_cache: &mut ResMut<ImageCache>,
) {
    place_icon_if(
        builder,
        button.icon_place,
        IconPlace::Left,
        &button.icon_path,
        button.entry,
        asset_server,
        image_cache,
        vec!["button-text".to_string()],
        id.0,
        layer,
        css_source.clone(),
    );

    builder.spawn((
        Name::new(format!("Button-Text-{}", button.entry)),
        Text::new(button.text.clone()),
        TextColor::default(),
        TextFont::default(),
        TextLayout::default(),
        css_source.clone(),
        UIWidgetState::default(),
        ZIndex::default(),
        CssClass(vec!["button-text".to_string()]),
        Pickable::IGNORE,
        BindToID(id.0),
        RenderLayers::layer(layer),
        ButtonText,
    ));

    place_icon_if(
        builder,
        button.icon_place,
        IconPlace::Right,
        &button.icon_path,
        button.entry,
        asset_server,
        image_cache,
        vec!["button-text".to_string()],
        id.0,
        layer,
        css_source.clone(),
    );
}

/// Handles click events for button widgets and updates focus state.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Button>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        if state.disabled {
            trigger.propagate(false);
            return;
        }
        state.focused = true;
        state.checked = true;
        current_widget_state.widget_id = gen_id.0;
    }

    trigger.propagate(false);
}

/// Sets hovered state when the cursor enters a button.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Button>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Clears hovered state when the cursor leaves a button.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Button>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}
