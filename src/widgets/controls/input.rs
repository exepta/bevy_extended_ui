use std::collections::HashMap;
use std::time::Duration;

use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::styles::{Background, CssClass, CssSource, FontVal, Style, TagName};
use crate::utils::keycode_to_char;
use crate::widgets::{
    BindToID, InputCap, InputField, InputType, InputValue, UIGenID, UIWidgetState, WidgetId,
    WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
#[cfg(not(target_arch = "wasm32"))]
use arboard::Clipboard;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::text::{TextBackgroundColor, TextLayoutInfo, TextSpan};
use bevy::ui::{RelativeCursorPosition, ScrollPosition, UiScale};
use bevy::window::PrimaryWindow;
#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
use std::sync::{Arc, Mutex};
#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
use wasm_bindgen_futures::{JsFuture, spawn_local};

/// Marker component for initialized input fields.
#[derive(Component)]
struct InputFieldBase;

/// Marker component for the input text node.
#[derive(Component)]
struct InputFieldText;

/// Marker component for the input icon container.
#[derive(Component)]
struct InputFieldIcon;

/// Marker component for the input icon image.
#[derive(Component)]
struct InputFieldIconImage;

/// Marker component for the blinking cursor node.
#[derive(Component)]
struct InputCursor;

/// Marker component for the text container node.
#[derive(Component)]
struct InputContainer;

/// Tracks text selection within an input field.
#[derive(Component, Default, Clone)]
struct InputSelection {
    anchor: usize,
    focus: usize,
    dragging: bool,
}

/// Marker component for the selection text span.
#[derive(Component)]
struct InputSelectionSpan;

/// Marker component for the suffix text span.
#[derive(Component)]
struct InputSuffixSpan;

/// Marker component for the overlay label node.
#[derive(Component)]
struct OverlayLabel;

/// Tracks key repeat timers for continuous input.
#[derive(Resource, Default)]
struct KeyRepeatTimers {
    timers: HashMap<KeyCode, Timer>,
}

/// Clipboard access for input shortcuts.
#[derive(Resource)]
struct InputClipboard {
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: Option<Clipboard>,
    fallback: String,
    #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
    pending: Arc<Mutex<PendingPaste>>,
}

#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
struct PendingPaste {
    target: Option<usize>,
    text: Option<String>,
}

impl Default for InputClipboard {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return Self {
                clipboard: Clipboard::new().ok(),
                fallback: String::new(),
            };
        }

        #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
        {
            return Self {
                fallback: String::new(),
                pending: Arc::new(Mutex::new(PendingPaste {
                    target: None,
                    text: None,
                })),
            };
        }

        #[cfg(all(target_arch = "wasm32", not(feature = "clipboard-wasm")))]
        {
            return Self {
                fallback: String::new(),
            };
        }
    }
}

impl InputClipboard {
    fn set_text(&mut self, text: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(clipboard) = self.clipboard.as_mut() {
            if clipboard.set_text(text.to_string()).is_ok() {
                return;
            }
        }

        #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
        {
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                let text_owned = text.to_string();
                spawn_local(async move {
                    let _ = JsFuture::from(clipboard.write_text(&text_owned)).await;
                });
                return;
            }
        }

        self.fallback = text.to_string();
    }

    fn get_text(&mut self) -> Option<String> {
        #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
        {
            return None;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(clipboard) = self.clipboard.as_mut() {
            if let Ok(text) = clipboard.get_text() {
                return Some(text);
            }
        }

        if self.fallback.is_empty() {
            None
        } else {
            Some(self.fallback.clone())
        }
    }

    #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
    fn request_paste(&mut self, target: usize) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.target = Some(target);
            pending.text = None;
        }

        let pending = self.pending.clone();

        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            spawn_local(async move {
                let text = JsFuture::from(clipboard.read_text())
                    .await
                    .ok()
                    .and_then(|v| v.as_string());
                if let Some(text) = text {
                    if let Ok(mut pending) = pending.lock() {
                        pending.text = Some(text);
                    }
                }
            });
            return;
        }
    }

    #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
    fn take_pending(&mut self) -> Option<(usize, String)> {
        let mut pending = self.pending.lock().ok()?;
        let target = pending.target?;
        let text = pending.text.take()?;
        pending.target = None;
        Some((target, text))
    }
}

/// Stores the original width of an input text container.
#[derive(Component)]
struct OriginalWidth(pub f32);

/// Resource controlling cursor blink timing.
#[derive(Resource)]
pub struct CursorBlinkTimer {
    pub timer: Timer,
}

impl Default for CursorBlinkTimer {
    /// Creates the default cursor blink timer.
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.95, TimerMode::Repeating),
        }
    }
}

/// Plugin that registers input field widget behavior.
pub struct InputWidget;

impl Plugin for InputWidget {
    /// Registers systems for input field setup and interaction.
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyRepeatTimers::default());
        app.insert_resource(CursorBlinkTimer::default());
        app.insert_resource(InputClipboard::default());
        #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
        let typing_chain = (handle_typing, sync_input_text_spans, apply_pending_paste).chain();

        #[cfg(not(all(target_arch = "wasm32", feature = "clipboard-wasm")))]
        let typing_chain = (handle_typing, sync_input_text_spans).chain();

        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                sync_input_field_updates,
                update_cursor_visibility,
                update_cursor_position,
                typing_chain,
                handle_input_horizontal_scroll,
                calculate_correct_text_container_width,
                handle_overlay_label,
            ),
        );
    }
}

/// Creates UI nodes for each `InputField` component entity that does not yet have an `InputFieldBase` marker.
///
/// This system:
/// - Queries all entities with `InputField` but without `InputFieldBase`.
/// - Sets the cursor position to the end of the text if text is present.
/// - Inserts the base components required for the input field, including background, border, shadow, and rendering layer.
/// - Spawns child entities representing:
///   - An optional icon on the left side, loaded via `AssetServer` and cached in `ImageCache`.
///   - An overlay label with the input's descriptive text.
///   - A text container holding:
///     - A hidden input cursor node.
///     - The actual input text node.
/// - Adds event observers for click, cursor enter, and cursor leave events to handle interactivity.
fn internal_node_creation_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &UIGenID, &mut InputField, Option<&CssSource>),
        (With<InputField>, Without<InputFieldBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for (entity, id, mut field, source_opt) in query.iter_mut() {
        let css_source = source_opt.cloned().unwrap_or_default();

        if !field.text.is_empty() {
            field.cursor_position = field.text.len();
        }

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Input-{}", field.entry)),
                Node::default(),
                WidgetId {
                    id: field.entry,
                    kind: WidgetKind::InputField,
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
                TagName(String::from("input")),
                RenderLayers::layer(layer),
                InputFieldBase,
                InputValue(field.text.clone()),
                InputSelection {
                    anchor: field.cursor_position,
                    focus: field.cursor_position,
                    dragging: false,
                },
            ))
            .with_children(|builder| {
                let icon_path = field
                    .icon_path
                    .as_deref()
                    .map(str::trim)
                    .filter(|path| !path.is_empty());
                if let Some(icon_path) = icon_path {
                    let owned_icon = icon_path.to_string();
                    let handle = image_cache
                        .map
                        .entry(owned_icon.clone())
                        .or_insert_with(|| asset_server.load(owned_icon))
                        .clone();

                    // Icon left
                    builder.spawn((
                        Name::new(format!("Input-Icon-{}", field.entry)),
                        Node::default(),
                        BackgroundColor::default(),
                        ImageNode::default(),
                        BorderColor::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["in-icon-container".to_string()]),
                        Pickable::IGNORE,
                        RenderLayers::layer(layer),
                        InputFieldIcon,
                        BindToID(id.0),
                        children![(
                            Name::new(format!("Icon-{}", field.entry)),
                            ImageNode {
                                image: handle,
                                ..default()
                            },
                            ZIndex::default(),
                            UIWidgetState::default(),
                            css_source.clone(),
                            CssClass(vec!["in-icon".to_string()]),
                            Pickable::IGNORE,
                            RenderLayers::layer(layer),
                            InputFieldIconImage,
                            BindToID(id.0),
                        )],
                    ));
                }

                // Overlay label
                builder.spawn((
                    Name::new(format!("Input-Label-{}", field.entry)),
                    Node::default(),
                    Text::new(field.label.clone()),
                    TextColor::default(),
                    TextLayout::default(),
                    TextFont::default(),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    css_source.clone(),
                    CssClass(vec!["input-label".to_string()]),
                    Pickable::IGNORE,
                    RenderLayers::layer(layer),
                    OverlayLabel,
                    BindToID(id.0),
                ));

                // Text content children
                builder
                    .spawn((
                        Name::new(format!("Input-Text-Container-{}", field.entry)),
                        Node::default(),
                        BackgroundColor::default(),
                        BorderColor::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["in-text-container".to_string()]),
                        Pickable::IGNORE,
                        OriginalWidth(-1.),
                        RelativeCursorPosition::default(),
                        RenderLayers::layer(layer),
                        InputContainer,
                        BindToID(id.0),
                        children![
                            // Input Cursor
                            (
                                Name::new(format!("Cursor-{}", field.entry)),
                                Node::default(),
                                BackgroundColor::default(),
                                ImageNode::default(),
                                BorderColor::default(),
                                ZIndex(3),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["input-cursor".to_string()]),
                                Visibility::Hidden,
                                Pickable::IGNORE,
                                RenderLayers::layer(layer),
                                InputCursor,
                                BindToID(id.0),
                            ),
                            // Input Text
                            (
                                Name::new(format!("Text-{}", field.entry)),
                                Node::default(),
                                Text::new(field.text.clone()),
                                TextColor::default(),
                                TextLayout::default(),
                                TextFont::default(),
                                ZIndex(0),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["input-text".to_string()]),
                                Pickable::IGNORE,
                                RenderLayers::layer(layer),
                                InputFieldText,
                                BindToID(id.0),
                                children![
                                    (
                                        Name::new(format!("Selection-Span-{}", field.entry)),
                                        TextSpan::new(String::new()),
                                        TextColor::default(),
                                        TextBackgroundColor(Color::NONE),
                                        TextFont::default(),
                                        InputSelectionSpan,
                                        BindToID(id.0),
                                    ),
                                    (
                                        Name::new(format!("Suffix-Span-{}", field.entry)),
                                        TextSpan::new(String::new()),
                                        TextColor::default(),
                                        TextFont::default(),
                                        InputSuffixSpan,
                                        BindToID(id.0),
                                    ),
                                ],
                            )
                        ],
                    ))
                    .insert(ImageNode::default());
            })
            .observe(on_internal_press)
            .observe(on_internal_drag)
            .observe(on_internal_release)
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Syncs input field text and state to UI components.
fn sync_input_field_updates(
    mut commands: Commands,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut query: Query<
        (
            Entity,
            &mut InputField,
            &mut InputValue,
            &mut InputSelection,
            &UIGenID,
            Option<&CssSource>,
        ),
        (With<InputFieldBase>, Changed<InputField>),
    >,
    children_query: Query<&Children, With<InputFieldBase>>,
    // text is synced in `sync_input_text_spans`
    mut label_query: Query<(&mut Text, &BindToID), (With<OverlayLabel>, Without<InputFieldText>)>,
    icon_query: Query<(Entity, &BindToID), With<InputFieldIcon>>,
    mut icon_image_query: Query<(&mut ImageNode, &BindToID), With<InputFieldIconImage>>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    let mut icon_entities: HashMap<usize, Entity> = HashMap::new();
    for (entity, bind) in icon_query.iter() {
        icon_entities.insert(bind.0, entity);
    }

    for (entity, mut field, mut input_value, mut selection, ui_id, source_opt) in query.iter_mut() {
        let css_source = source_opt.cloned().unwrap_or_default();

        field.cursor_position = field.cursor_position.min(field.text.len());
        selection.anchor = selection.anchor.min(field.text.len());
        selection.focus = selection.focus.min(field.text.len());
        if input_value.0 != field.text {
            input_value.0 = field.text.clone();
        }

        for (mut label_text, bind_id) in label_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            label_text.0 = field.label.clone();
        }

        let icon_path = field
            .icon_path
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty());
        if let Some(icon_path) = icon_path {
            let owned_icon = icon_path.to_string();
            let handle = image_cache
                .map
                .entry(owned_icon.clone())
                .or_insert_with(|| asset_server.load(owned_icon))
                .clone();

            if let Some(icon_entity) = icon_entities.get(&ui_id.0).copied() {
                for (mut image_node, bind_id) in icon_image_query.iter_mut() {
                    if bind_id.0 != ui_id.0 {
                        continue;
                    }
                    image_node.image = handle.clone();
                }

                commands.entity(icon_entity).insert(Visibility::Inherited);
            } else {
                let icon_entity = commands
                    .spawn((
                        Name::new(format!("Input-Icon-{}", field.entry)),
                        Node::default(),
                        BackgroundColor::default(),
                        ImageNode::default(),
                        BorderColor::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["in-icon-container".to_string()]),
                        Pickable::IGNORE,
                        RenderLayers::layer(layer),
                        InputFieldIcon,
                        BindToID(ui_id.0),
                        children![(
                            Name::new(format!("Icon-{}", field.entry)),
                            ImageNode {
                                image: handle,
                                ..default()
                            },
                            ZIndex::default(),
                            UIWidgetState::default(),
                            css_source.clone(),
                            CssClass(vec!["in-icon".to_string()]),
                            Pickable::IGNORE,
                            RenderLayers::layer(layer),
                            InputFieldIconImage,
                            BindToID(ui_id.0),
                        )],
                    ))
                    .id();

                if children_query.get(entity).is_ok() {
                    commands.entity(entity).insert_children(0, &[icon_entity]);
                } else {
                    commands.entity(entity).add_child(icon_entity);
                }
            }
        } else if let Some(icon_entity) = icon_entities.get(&ui_id.0).copied() {
            commands.entity(icon_entity).despawn();
        }
    }
}

// ===============================================
//             Internal Functions
// ===============================================

/// Updates the visibility and blinking animation of the input cursor.
///
/// - Blinks the cursor by adjusting its alpha transparency using a sine wave based on elapsed time.
/// - Shows the cursor only if the associated `InputField` is focused.
/// - Updates the displayed text for the input field:
///   - Masks the text with `*` characters if the input type is `Password`.
///   - Shows placeholder text if the input is empty.
/// - Hides the cursor and clears the text display when the input field is unfocused.
/// Updates cursor visibility based on focus and blink timer.
fn update_cursor_visibility(
    time: Res<Time>,
    mut cursor_blink_timer: ResMut<CursorBlinkTimer>,
    mut cursor_query: Query<
        (
            &mut Visibility,
            &mut BackgroundColor,
            &mut UiStyle,
            &BindToID,
        ),
        With<InputCursor>,
    >,
    input_field_query: Query<(&InputField, &UIWidgetState, &UIGenID), With<InputFieldBase>>,
) {
    cursor_blink_timer.timer.tick(time.delta());

    // Build a compact lookup map by UI id to avoid nested loops.
    /// Lightweight view of input field state for cursor rendering.
    #[derive(Clone)]
    struct FieldView {
        focused: bool,
        disabled: bool,
    }

    let mut fields: HashMap<usize, FieldView> = HashMap::new();
    for (_field, state, ui_id) in input_field_query.iter() {
        fields.insert(
            ui_id.0,
            FieldView {
                focused: state.focused,
                disabled: state.disabled,
            },
        );
    }

    for (mut visibility, mut background, mut styles, bind_cursor_id) in cursor_query.iter_mut() {
        let Some(field) = fields.get(&bind_cursor_id.0) else {
            continue;
        };

        if field.focused && !field.disabled {
            let alpha =
                (cursor_blink_timer.timer.elapsed_secs() * 2.0 * std::f32::consts::PI).sin() * 0.5
                    + 0.5;
            background.0.set_alpha(alpha);

            for (_, style) in styles.styles.iter_mut() {
                style.normal.background = Some(Background {
                    color: background.0,
                    ..default()
                });
            }

            // Fix: this condition was always true due to `||`.
            let needs_show = !matches!(*visibility, Visibility::Inherited | Visibility::Visible);
            if needs_show {
                *visibility = Visibility::Inherited;
            }
        } else {
            if !matches!(*visibility, Visibility::Hidden) {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

/// Updates the cursor position within the input text based on keyboard input.
///
/// - Handles left and right arrow keys with initial delay and repeat rate timers.
/// - Calculates the horizontal pixel position of the cursor based on text font metrics.
/// - Updates the cursor node's CSS left position to reflect the cursor's position in the text.
/// Positions the cursor within the input text container.
fn update_cursor_position(
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut cursor_query: Query<(&mut Node, &mut UiStyle, &BindToID), With<InputCursor>>,
    mut text_field_query: Query<
        (
            &mut InputField,
            &mut InputSelection,
            &UIGenID,
            &UIWidgetState,
        ),
        (With<InputField>, Without<InputCursor>),
    >,
    text_query: Query<(&TextFont, &BindToID), (With<InputFieldText>, Without<InputCursor>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    // Cache fonts by UI id to avoid per-cursor linear search.
    let mut fonts: HashMap<usize, TextFont> = HashMap::new();
    for (font, bind) in text_query.iter() {
        fonts.insert(bind.0, font.clone());
    }

    for (mut cursor_node, mut styles, bind_id) in cursor_query.iter_mut() {
        let Some((mut text_field, mut selection, _ui_id, state)) = text_field_query
            .iter_mut()
            .find(|(_, _, ui_id, _)| ui_id.0 == bind_id.0)
        else {
            continue;
        };

        if !state.focused || state.disabled {
            continue;
        }

        // Arrow left
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            text_field.cursor_position = text_field.cursor_position.saturating_sub(1);
            selection.anchor = text_field.cursor_position;
            selection.focus = text_field.cursor_position;
            key_repeat.timers.insert(
                KeyCode::ArrowLeft,
                Timer::from_seconds(initial_delay, TimerMode::Once),
            );
        }

        // Arrow right
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            text_field.cursor_position =
                (text_field.cursor_position + 1).min(text_field.text.len());
            selection.anchor = text_field.cursor_position;
            selection.focus = text_field.cursor_position;
            key_repeat.timers.insert(
                KeyCode::ArrowRight,
                Timer::from_seconds(initial_delay, TimerMode::Once),
            );
        }

        for key in [KeyCode::ArrowLeft, KeyCode::ArrowRight] {
            if keyboard_input.pressed(key) {
                if let Some(timer) = key_repeat.timers.get_mut(&key) {
                    timer.tick(time.delta());
                    if timer.is_finished() {
                        match key {
                            KeyCode::ArrowLeft => {
                                text_field.cursor_position =
                                    text_field.cursor_position.saturating_sub(1);
                                selection.anchor = text_field.cursor_position;
                                selection.focus = text_field.cursor_position;
                            }
                            KeyCode::ArrowRight => {
                                text_field.cursor_position =
                                    (text_field.cursor_position + 1).min(text_field.text.len());
                                selection.anchor = text_field.cursor_position;
                                selection.focus = text_field.cursor_position;
                            }
                            _ => {}
                        }

                        timer.set_duration(Duration::from_secs_f32(repeat_rate));
                        timer.reset();
                    }
                }
            }
        }

        let Some(text_font) = fonts.get(&bind_id.0) else {
            continue;
        };

        let cursor_x_position =
            calculate_cursor_x_position(&text_field, text_field.cursor_position, text_font);
        cursor_node.left = Val::Px(cursor_x_position);

        for (_, style) in styles.styles.iter_mut() {
            style.normal.left = Some(cursor_node.left);
        }
    }

    key_repeat
        .timers
        .retain(|key, _| keyboard_input.pressed(*key));
}

/// Adjusts the width of the input text container when the input field or style changes.
///
/// - If the input field has an icon, reduces the text container width by a fixed percentage to accommodate the icon.
/// - Caches the original width to avoid repeated adjustments.
/// Adjusts the text container width based on content and cap rules.
fn calculate_correct_text_container_width(
    query: Query<
        (&InputField, &UIGenID),
        (
            With<InputField>,
            Without<InputContainer>, // <-- add this to make queries disjoint
            Or<(Added<InputField>, Changed<UiStyle>, Changed<InputField>)>,
        ),
    >,
    mut container_query: Query<(&mut UiStyle, &mut OriginalWidth, &BindToID), With<InputContainer>>,
) {
    for (input_field, ui_id) in query.iter() {
        if input_field.icon_path.is_none() {
            continue;
        }

        for (mut style, mut original_width, bind_id) in container_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            let Some(active) = style.active_style.clone() else {
                continue;
            };

            let current = match active.width.unwrap_or_default() {
                Val::Percent(percent) => percent,
                _ => 100.,
            };

            if original_width.0 == -1.0 {
                original_width.0 = current;
            }

            if original_width.0 > current {
                continue;
            }

            for (_, value) in style.styles.iter_mut() {
                value.normal.width = Some(Val::Percent((current - 15.0).max(0.0)));
            }
        }
    }
}

/// Handles horizontal scrolling of an input field's text container to keep the cursor visible.
///
/// - Only operates on the currently focused input field.
/// - Calculates the cursor's horizontal pixel position based on the cursor index and character width.
/// - Adjusts the scroll offset of the input container to ensure the cursor is within the visible bounds.
/// - Resets scroll to zero if the total text width fits inside the visible container.
/// Handles horizontal scrolling when input text exceeds available width.
fn handle_input_horizontal_scroll(
    query: Query<(&InputField, &UIGenID, &UIWidgetState), With<InputFieldBase>>,
    mut scroll_query: Query<(&mut ScrollPosition, &BindToID), With<InputContainer>>,
    text_node_query: Query<(&ComputedNode, &BindToID, &TextFont), With<InputFieldText>>,
) {
    // Cache computed node/font by UI id for faster lookup.
    let mut text_meta: HashMap<usize, (Vec2, f32)> = HashMap::new();
    for (node, bind_id, font) in text_node_query.iter() {
        text_meta.insert(bind_id.0, (node.size(), font.font_size));
    }

    for (input_field, ui_id, state) in query.iter() {
        if !state.focused || state.disabled {
            continue;
        }

        let Some((text_size, font_size)) = text_meta.get(&ui_id.0).copied() else {
            continue;
        };

        let char_width = font_size;
        let cursor_x = input_field.cursor_position as f32 * char_width;

        let available_width = text_size.x - 10.0;

        for (mut scroll, bind_id) in scroll_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            match input_field.cap_text_at {
                InputCap::NoCap => {
                    let visible_left = scroll.x;
                    let visible_right = scroll.x + available_width;

                    if cursor_x > visible_right {
                        scroll.x = cursor_x - available_width + char_width;
                    } else if cursor_x < visible_left {
                        scroll.x = cursor_x;
                    }

                    let total_text_width = input_field.text.len() as f32 * char_width;
                    if total_text_width < available_width {
                        scroll.x = 0.0;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Processes typing input for text fields, including key repeat and special keys.
///
/// - Handles insertion of characters respecting input caps and input type validations.
/// - Supports backspace with key repeat functionality.
/// - Handles an Enter key to lose focus and optionally clear input field text.
/// - Updates the visible text and cursor position accordingly.
/// - Masks input with `*` characters if an input type is `Password`.
/// - Updates text color on changes.
/// Processes keyboard input for focused input fields.
fn handle_typing(
    time: Res<Time>,
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut query: Query<(
        &mut InputField,
        &mut InputValue,
        &mut InputSelection,
        &mut UIWidgetState,
        &UiStyle,
        &UIGenID,
    )>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<InputClipboard>,
    text_query: Query<(&ComputedNode, &BindToID), With<InputFieldText>>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    // Cache pressed keys to avoid repeated iterator calls.
    let pressed: Vec<KeyCode> = keyboard.get_pressed().copied().collect();

    for (mut in_field, mut input_value, mut selection, mut state, style, ui_id) in query.iter_mut()
    {
        if state.disabled {
            state.focused = false;
            continue;
        }

        if !state.focused {
            continue;
        }

        for (computed_node, bind_id) in text_query.iter() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            if ctrl {
                if keyboard.just_pressed(KeyCode::KeyA) {
                    selection.anchor = 0;
                    selection.focus = in_field.text.len();
                    in_field.cursor_position = in_field.text.len();
                    continue;
                }

                if keyboard.just_pressed(KeyCode::KeyC) {
                    if let Some((start, end)) = selection_range(&selection) {
                        if let Some(selected) = in_field.text.get(start..end) {
                            clipboard.set_text(selected);
                        }
                    }
                    continue;
                }

                if keyboard.just_pressed(KeyCode::KeyX) {
                    if let Some((start, end)) = selection_range(&selection) {
                        if let Some(selected) = in_field.text.get(start..end) {
                            clipboard.set_text(selected);
                        }
                        if delete_selection(&mut in_field, &mut selection) {
                            input_value.0 = in_field.text.clone();
                        }
                    }
                    continue;
                }

                if keyboard.just_pressed(KeyCode::KeyV) {
                    #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
                    {
                        clipboard.request_paste(ui_id.0);
                    }

                    #[cfg(not(all(target_arch = "wasm32", feature = "clipboard-wasm")))]
                    {
                        if let Some(paste) = clipboard.get_text() {
                            let deleted = delete_selection(&mut in_field, &mut selection);
                            let inserted =
                                insert_text_filtered(&mut in_field, style, computed_node, &paste);
                            if deleted || inserted {
                                input_value.0 = in_field.text.clone();
                            }
                        }
                    }
                    continue;
                }
            }

            // Enter: lose focus and optionally clear text.
            if keyboard.just_pressed(KeyCode::Enter) {
                state.focused = false;
                if in_field.clear_after_focus_lost {
                    in_field.text.clear();
                    if input_value.0 != in_field.text {
                        input_value.0 = in_field.text.clone();
                    }
                }
                clear_selection(&mut selection, in_field.cursor_position);
                continue;
            }

            // Backspace (single press).
            if keyboard.just_pressed(KeyCode::Backspace) {
                if delete_selection(&mut in_field, &mut selection) {
                    input_value.0 = in_field.text.clone();
                    continue;
                }

                if in_field.cursor_position > 0 && !in_field.text.is_empty() {
                    let remove_at = in_field.cursor_position - 1;
                    in_field.cursor_position = remove_at;
                    in_field.text.remove(remove_at);

                    if input_value.0 != in_field.text {
                        input_value.0 = in_field.text.clone();
                    }
                }

                key_repeat.timers.insert(
                    KeyCode::Backspace,
                    Timer::from_seconds(initial_delay, TimerMode::Once),
                );
                continue;
            }

            // Character insertion + repeat.
            for key in &pressed {
                if ctrl {
                    continue;
                }

                let Some(ch) = keycode_to_char(*key, shift, alt) else {
                    continue;
                };

                if !in_field.input_type.is_valid_char(ch) {
                    // Do not abort the entire system; just skip this character.
                    continue;
                }

                if keyboard.just_pressed(*key) {
                    delete_selection(&mut in_field, &mut selection);
                    let pos = in_field.cursor_position;

                    if in_field.cap_text_at.get_value() > 0 {
                        let cap = in_field.cap_text_at.clone();
                        if pos >= cap.get_value() {
                            continue;
                        }
                    }

                    if in_field.cap_text_at == InputCap::CapAtNodeSize {
                        let font_px = style
                            .active_style
                            .as_ref()
                            .and_then(|s| s.font_size.as_ref())
                            .cloned()
                            .unwrap_or(FontVal::Px(13.))
                            .get(None);

                        let allowed_char_len = (computed_node.size().x / font_px).round() as usize;
                        if pos >= allowed_char_len {
                            continue;
                        }
                    }

                    in_field.text.insert(pos, ch);
                    in_field.cursor_position += 1;

                    if input_value.0 != in_field.text {
                        input_value.0 = in_field.text.clone();
                    }
                    key_repeat
                        .timers
                        .insert(*key, Timer::from_seconds(initial_delay, TimerMode::Once));
                    continue;
                }

                if let Some(timer) = key_repeat.timers.get_mut(key) {
                    timer.tick(time.delta());
                    if timer.is_finished() {
                        // Repeat should also insert at the cursor position (not push at the end).
                        let pos = in_field.cursor_position;
                        in_field.text.insert(pos, ch);
                        in_field.cursor_position += 1;

                        if input_value.0 != in_field.text {
                            input_value.0 = in_field.text.clone();
                        }

                        timer.set_duration(Duration::from_secs_f32(repeat_rate));
                        timer.reset();
                    }
                }
            }

            // Backspace repeat (hold).
            if keyboard.pressed(KeyCode::Backspace) {
                if let Some(timer) = key_repeat.timers.get_mut(&KeyCode::Backspace) {
                    timer.tick(time.delta());
                    if timer.is_finished() {
                        if delete_selection(&mut in_field, &mut selection) {
                            input_value.0 = in_field.text.clone();
                            timer.set_duration(Duration::from_secs_f32(repeat_rate));
                            timer.reset();
                            continue;
                        }

                        if in_field.cursor_position > 0 && !in_field.text.is_empty() {
                            let remove_at = in_field.cursor_position - 1;
                            in_field.cursor_position = remove_at;
                            in_field.text.remove(remove_at);

                            input_value.0 = in_field.text.clone();

                            timer.set_duration(Duration::from_secs_f32(repeat_rate));
                            timer.reset();
                        }
                    }
                }
            }

            if selection.anchor == selection.focus {
                selection.anchor = in_field.cursor_position;
                selection.focus = in_field.cursor_position;
            }

            in_field.cursor_position = in_field.cursor_position.min(in_field.text.len());
        }
    }

    key_repeat.timers.retain(|key, _| keyboard.pressed(*key));
}

/// Updates the position and font size of overlay labels associated with input fields.
///
/// This function manages the floating label behavior:
/// - When the input field is focused, the label shrinks and moves upwards.
/// - When the input field is unfocused and empty, the label is centered and larger.
/// - When the input field is unfocused but contains text, the label stays small and on top.
///
/// It also adjusts the label's horizontal position if the input field has an icon,
/// shifting the label right to avoid overlapping the icon.
/// Shows or hides overlay labels based on input focus and content.
fn handle_overlay_label(
    query: Query<
        (&UIWidgetState, &UIGenID, &InputField, &UiStyle, &Children),
        (With<InputField>, Without<OverlayLabel>),
    >,
    mut label_query: Query<
        (&BindToID, &mut Node, &mut TextFont, &mut UiStyle),
        (With<OverlayLabel>, Without<InputField>),
    >,
    icon_container_query: Query<
        (&UiStyle, &BindToID),
        (With<InputFieldIcon>, Without<OverlayLabel>),
    >,
) {
    // Cache icon widths per UI id.
    let mut icon_widths: HashMap<usize, f32> = HashMap::new();
    for (icon_style, bind) in icon_container_query.iter() {
        if let Some(active) = icon_style.active_style.clone() {
            if let Some(Val::Px(w)) = active.width {
                icon_widths.insert(bind.0, w);
            }
        }
    }

    for (state, gen_id, in_field, in_style, children) in query.iter() {
        let Some(active_style) = in_style.active_style.clone() else {
            continue;
        };

        let height = match active_style.height.unwrap_or_default() {
            Val::Px(px) => px,
            _ => 55.,
        };

        for child in children.iter() {
            let Ok((bind_to, mut node, mut text_font, mut styles)) = label_query.get_mut(child)
            else {
                continue;
            };

            if bind_to.0 != gen_id.0 {
                continue;
            }

            let center = (height / 2.0) - text_font.font_size / 1.5;
            let on_top = text_font.font_size / 2.0;

            if state.focused {
                node.top = Val::Px(on_top);
                text_font.font_size = 10.;
            } else if in_field.text.is_empty() {
                node.top = Val::Px(center);
                text_font.font_size = 14.;
            } else {
                node.top = Val::Px(on_top);
                text_font.font_size = 10.;
            }

            let expected_left = if let Some(w) = icon_widths.get(&gen_id.0).copied() {
                5.0 + w
            } else {
                10.0
            };
            let left_now = match node.left {
                Val::Px(px) => px,
                _ => 10.0,
            };

            if (left_now - expected_left).abs() > f32::EPSILON {
                node.left = Val::Px(expected_left);
            }

            for (_, style) in styles.styles.iter_mut() {
                style.normal.top = Some(node.top);
                style.normal.left = Some(node.left);
                style.normal.font_size = Some(FontVal::Px(text_font.font_size));
            }
        }
    }
}

/// Synchronizes input text with selection spans.
fn sync_input_text_spans(
    input_query: Query<
        (
            &InputField,
            &InputSelection,
            &UIWidgetState,
            &UIGenID,
            &UiStyle,
        ),
        With<InputFieldBase>,
    >,
    mut text_query: Query<
        (&mut Text, &mut TextColor, &TextFont, &BindToID),
        (
            With<InputFieldText>,
            Without<InputSelectionSpan>,
            Without<InputSuffixSpan>,
        ),
    >,
    mut selection_span_query: Query<
        (
            &mut TextSpan,
            &mut TextColor,
            &mut TextBackgroundColor,
            &mut TextFont,
            &BindToID,
        ),
        (
            With<InputSelectionSpan>,
            Without<InputFieldText>,
            Without<InputSuffixSpan>,
        ),
    >,
    mut suffix_span_query: Query<
        (&mut TextSpan, &mut TextColor, &mut TextFont, &BindToID),
        (
            With<InputSuffixSpan>,
            Without<InputFieldText>,
            Without<InputSelectionSpan>,
        ),
    >,
) {
    for (field, selection, state, ui_id, ui_style) in input_query.iter() {
        let base_color = get_active_text_color(ui_style);

        let visible_text = if field.text.is_empty() {
            if state.focused {
                field.placeholder.clone()
            } else {
                String::new()
            }
        } else if field.input_type == InputType::Password {
            "*".repeat(field.text.chars().count())
        } else {
            field.text.clone()
        };

        let selection_active = state.focused && !state.disabled && !field.text.is_empty();
        let selection_range = if selection_active {
            selection_range(&selection)
        } else {
            None
        };

        let (prefix, selected, suffix) = if let Some((start, end)) = selection_range {
            let prefix = visible_text.get(0..start).unwrap_or("");
            let selected = visible_text.get(start..end).unwrap_or("");
            let suffix = visible_text.get(end..).unwrap_or("");
            (prefix, selected, suffix)
        } else {
            (visible_text.as_str(), "", "")
        };

        let (selection_text, selection_bg) = resolve_selection_colors(ui_style, state, base_color);

        for (mut text, mut text_color, text_font, bind_id) in text_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            text.0 = prefix.to_string();
            text_color.0 = base_color;

            for (mut span, mut span_color, mut span_bg, mut span_font, span_bind) in
                selection_span_query.iter_mut()
            {
                if span_bind.0 != ui_id.0 {
                    continue;
                }

                span.0 = selected.to_string();
                span_color.0 = if selected.is_empty() {
                    base_color
                } else {
                    selection_text
                };
                span_bg.0 = if selected.is_empty() {
                    Color::NONE
                } else {
                    selection_bg
                };
                *span_font = text_font.clone();
            }

            for (mut span, mut span_color, mut span_font, span_bind) in suffix_span_query.iter_mut()
            {
                if span_bind.0 != ui_id.0 {
                    continue;
                }

                span.0 = suffix.to_string();
                span_color.0 = base_color;
                *span_font = text_font.clone();
            }
        }
    }
}

#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
fn apply_pending_paste(
    mut clipboard: ResMut<InputClipboard>,
    mut input_query: Query<(
        &mut InputField,
        &mut InputSelection,
        &mut InputValue,
        &UiStyle,
        &UIGenID,
        &UIWidgetState,
    )>,
    text_query: Query<(&ComputedNode, &BindToID), With<InputFieldText>>,
) {
    let Some((target, text)) = clipboard.take_pending() else {
        return;
    };

    let mut computed = None;
    for (node, bind) in text_query.iter() {
        if bind.0 == target {
            computed = Some(node.clone());
            break;
        }
    }

    let Some(computed_node) = computed else {
        return;
    };

    for (mut field, mut selection, mut input_value, style, ui_id, state) in input_query.iter_mut() {
        if ui_id.0 != target || state.disabled || !state.focused {
            continue;
        }

        delete_selection(&mut field, &mut selection);
        if insert_text_filtered(&mut field, style, &computed_node, &text) {
            input_value.0 = field.text.clone();
        }
        break;
    }
}

// ===============================================
//             Internal Helper Functions
// ===============================================

/// Calculates the horizontal pixel position of the cursor within the input text.
/// Calculates the cursor X position from a text index.
fn calculate_cursor_x_position(
    text_field: &InputField,
    cursor_pos: usize,
    style: &TextFont,
) -> f32 {
    if text_field.text.is_empty() || cursor_pos == 0 {
        return 0.0;
    }

    let cursor_pos = cursor_pos.min(text_field.text.len());
    let text_substr = &text_field.text[..cursor_pos];
    let text_width = calculate_text_width(text_substr, style);

    text_width + 1.0
}

/// Estimates the pixel width of a given text string based on font size.
/// Estimates the width of a text string for cursor placement.
fn calculate_text_width(text: &str, style: &TextFont) -> f32 {
    text.len() as f32 * style.font_size * 0.6
}

/// Retrieves the active text color from a widget style, falling back to white.
/// Resolves the active text color from the UI style.
fn get_active_text_color(style: &UiStyle) -> Color {
    style
        .active_style
        .as_ref()
        .and_then(|s| s.color)
        .unwrap_or(Color::WHITE)
}

/// Applies input text to the visible `Text` component.
fn selection_range(selection: &InputSelection) -> Option<(usize, usize)> {
    if selection.anchor == selection.focus {
        return None;
    }

    let start = selection.anchor.min(selection.focus);
    let end = selection.anchor.max(selection.focus);
    Some((start, end))
}

fn clear_selection(selection: &mut InputSelection, cursor: usize) {
    selection.anchor = cursor;
    selection.focus = cursor;
    selection.dragging = false;
}

fn delete_selection(in_field: &mut InputField, selection: &mut InputSelection) -> bool {
    let Some((start, end)) = selection_range(selection) else {
        return false;
    };

    if start >= end || end > in_field.text.len() {
        clear_selection(selection, in_field.cursor_position);
        return false;
    }

    in_field.text.replace_range(start..end, "");
    in_field.cursor_position = start;
    clear_selection(selection, start);
    true
}

fn insert_text_filtered(
    in_field: &mut InputField,
    style: &UiStyle,
    computed_node: &ComputedNode,
    text: &str,
) -> bool {
    let mut inserted = false;

    for ch in text.chars() {
        if ch == '\n' || ch == '\r' {
            continue;
        }

        if !in_field.input_type.is_valid_char(ch) {
            continue;
        }

        let pos = in_field.cursor_position;

        if in_field.cap_text_at.get_value() > 0 {
            let cap = in_field.cap_text_at.get_value();
            if pos >= cap {
                break;
            }
        }

        if in_field.cap_text_at == InputCap::CapAtNodeSize {
            let font_px = style
                .active_style
                .as_ref()
                .and_then(|s| s.font_size.as_ref())
                .cloned()
                .unwrap_or(FontVal::Px(13.))
                .get(None);

            let allowed_char_len = (computed_node.size().x / font_px).round() as usize;
            if pos >= allowed_char_len {
                break;
            }
        }

        in_field.text.insert(pos, ch);
        in_field.cursor_position += 1;
        inserted = true;
    }

    inserted
}

fn default_selection_background() -> Color {
    Color::srgba(0.2, 0.45, 1.0, 0.35)
}

fn resolve_selection_colors(
    ui_style: &UiStyle,
    state: &UIWidgetState,
    fallback_text: Color,
) -> (Color, Color) {
    let mut candidates: Vec<(&String, u32, usize)> = Vec::new();

    for (key, style_pair) in &ui_style.styles {
        let selector = if style_pair.selector.is_empty() {
            key.as_str()
        } else {
            style_pair.selector.as_str()
        };

        if !selector.contains("::selection") {
            continue;
        }
        if !selector_matches_state_for_selection(selector, state) {
            continue;
        }
        let spec = selector_specificity_for_selection(selector);
        candidates.push((key, spec, style_pair.origin));
    }

    if candidates.is_empty() {
        return (fallback_text, default_selection_background());
    }

    candidates.sort_by(|a, b| match a.2.cmp(&b.2) {
        std::cmp::Ordering::Equal => a.1.cmp(&b.1),
        other => other,
    });

    let mut final_style = Style::default();
    for (sel, _, _) in &candidates {
        if let Some(pair) = ui_style.styles.get(*sel) {
            final_style.merge(&pair.normal);
        }
    }
    for (sel, _, _) in &candidates {
        if let Some(pair) = ui_style.styles.get(*sel) {
            final_style.merge(&pair.important);
        }
    }

    let text_color = final_style.color.unwrap_or(fallback_text);
    let background = final_style
        .background
        .as_ref()
        .map(|b| b.color)
        .unwrap_or_else(default_selection_background);

    (text_color, background)
}

fn selector_matches_state_for_selection(selector: &str, state: &UIWidgetState) -> bool {
    for part in selector.replace('>', " > ").split_whitespace() {
        if part == ">" {
            continue;
        }

        for pseudo in part.split(':').skip(1) {
            if pseudo.is_empty() || pseudo == "selection" {
                continue;
            }

            match pseudo {
                "read-only" if !state.readonly => return false,
                "disabled" if !state.disabled => return false,
                "checked" if state.disabled || !state.checked => return false,
                "focus" if state.disabled || !state.focused => return false,
                "hover" if state.disabled || !state.hovered => return false,
                "invalid" if !state.invalid => return false,
                _ => {}
            }
        }
    }
    true
}

fn selector_specificity_for_selection(selector: &str) -> u32 {
    let mut spec = 0;
    for part in selector.replace('>', " > ").split_whitespace() {
        if part == ">" {
            continue;
        }

        let segments: Vec<&str> = part
            .split(':')
            .filter(|segment| !segment.is_empty())
            .collect();

        let base = segments.first().copied().unwrap_or("");

        spec += if base.starts_with('#') {
            100
        } else if base.starts_with('.') {
            10
        } else if base == "*" || base.is_empty() {
            0
        } else {
            1
        };

        let pseudo_count = segments
            .iter()
            .skip(1)
            .filter(|segment| **segment != "selection")
            .count();
        spec += pseudo_count as u32;
    }
    spec
}

fn cursor_position_from_pointer(
    ui_id: usize,
    text_len: usize,
    selection: &InputSelection,
    container_query: &Query<
        (
            &ComputedNode,
            &RelativeCursorPosition,
            Option<&ScrollPosition>,
            Option<&UiStyle>,
            &BindToID,
        ),
        With<InputContainer>,
    >,
    text_query: &Query<(&TextFont, &BindToID), With<InputFieldText>>,
    layout_query: &Query<(&TextLayoutInfo, &BindToID), With<InputFieldText>>,
    window_q: &Query<&Window, With<PrimaryWindow>>,
    ui_scale: &UiScale,
) -> Option<usize> {
    if text_len == 0 {
        return Some(0);
    }
    let Ok(window) = window_q.single() else {
        return None;
    };
    let sf = window.scale_factor() * ui_scale.0;

    let mut cursor_x = None;
    let mut padding_left = 0.0;
    let mut scroll_x = 0.0;

    for (node, rel, scroll, style, bind) in container_query.iter() {
        if bind.0 != ui_id {
            continue;
        }

        let Some(normalized) = rel.normalized else {
            return None;
        };

        let width = (node.size().x / sf).max(1.0);
        let clamped = (normalized.x + 0.5).clamp(0.0, 1.0);
        cursor_x = Some(clamped * width);

        if let Some(scroll) = scroll {
            scroll_x = scroll.x;
        }

        if let Some(style) = style
            .and_then(|s| s.active_style.as_ref())
            .and_then(|s| s.padding)
        {
            if let Val::Px(px) = style.left {
                padding_left = px;
            }
        }

        break;
    }

    let Some(mut cursor_x) = cursor_x else {
        return None;
    };

    cursor_x = (cursor_x - padding_left).max(0.0) + scroll_x;

    let selection_range = selection_range(selection);
    let (prefix_len, selection_len) = if let Some((start, end)) = selection_range {
        (start, end.saturating_sub(start))
    } else {
        (text_len, 0)
    };
    let selection_offset = prefix_len;
    let suffix_offset = prefix_len.saturating_add(selection_len);

    for (layout, bind) in layout_query.iter() {
        if bind.0 != ui_id {
            continue;
        }

        if layout.glyphs.is_empty() {
            return Some(0);
        }

        let scale = layout.scale_factor.max(1.0);
        let mut last_end = 0usize;
        let mut last_right = 0.0;

        for glyph in layout.glyphs.iter().filter(|g| g.line_index == 0) {
            let g_left = glyph.position.x / scale;
            let g_right = (glyph.position.x + glyph.size.x) / scale;
            let mid = (g_left + g_right) * 0.5;
            let span_offset = match glyph.span_index {
                0 => 0,
                1 => selection_offset,
                2 => suffix_offset,
                _ => text_len,
            };
            let g_start = span_offset.saturating_add(glyph.byte_index);
            let g_end = span_offset.saturating_add(glyph.byte_index + glyph.byte_length);

            if cursor_x <= mid {
                return Some(g_start.min(text_len));
            }

            if cursor_x <= g_right {
                return Some(g_end.min(text_len));
            }

            last_end = g_end;
            last_right = g_right;
        }

        if cursor_x >= last_right {
            return Some(last_end.min(text_len));
        }

        return Some(last_end.min(text_len));
    }

    let mut font_size = None;
    for (font, bind) in text_query.iter() {
        if bind.0 == ui_id {
            font_size = Some(font.font_size);
            break;
        }
    }

    let Some(font_size) = font_size else {
        return None;
    };

    let char_width = (font_size * 0.6).max(1.0);
    let pos = (cursor_x / char_width).floor() as usize;
    Some(pos.min(text_len))
}

// ===============================================
//                   Internal Events
// ===============================================

/// Handles pointer press events on input fields.
fn on_internal_press(
    mut trigger: On<Pointer<Press>>,
    mut query: Query<
        (
            &mut UIWidgetState,
            &UIGenID,
            &mut InputField,
            &mut InputSelection,
        ),
        With<InputField>,
    >,
    bind_query: Query<&BindToID>,
    container_query: Query<
        (
            &ComputedNode,
            &RelativeCursorPosition,
            Option<&ScrollPosition>,
            Option<&UiStyle>,
            &BindToID,
        ),
        With<InputContainer>,
    >,
    text_query: Query<(&TextFont, &BindToID), With<InputFieldText>>,
    layout_query: Query<(&TextLayoutInfo, &BindToID), With<InputFieldText>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let target = trigger.event_target();

    let mut apply_press = |mut state: Mut<UIWidgetState>,
                           gen_id: &UIGenID,
                           mut field: Mut<InputField>,
                           mut selection: Mut<InputSelection>| {
        if state.disabled {
            return;
        }

        state.focused = true;
        current_widget_state.widget_id = gen_id.0;

        if let Some(pos) = cursor_position_from_pointer(
            gen_id.0,
            field.text.len(),
            &selection,
            &container_query,
            &text_query,
            &layout_query,
            &window_q,
            &ui_scale,
        ) {
            field.cursor_position = pos;
            selection.anchor = pos;
            selection.focus = pos;
            selection.dragging = true;
        }
    };

    if let Ok((state, gen_id, field, selection)) = query.get_mut(target) {
        apply_press(state, gen_id, field, selection);
    } else if let Ok(bind) = bind_query.get(target) {
        if let Some((state, gen_id, field, selection)) =
            query.iter_mut().find(|(_, id, _, _)| id.0 == bind.0)
        {
            apply_press(state, gen_id, field, selection);
        }
    }

    trigger.propagate(false);
}

/// Handles drag events to update text selection.
fn on_internal_drag(
    event: On<Pointer<Drag>>,
    mut query: Query<
        (
            &mut InputField,
            &mut InputSelection,
            &UIWidgetState,
            &UIGenID,
        ),
        With<InputField>,
    >,
    bind_query: Query<&BindToID>,
    container_query: Query<
        (
            &ComputedNode,
            &RelativeCursorPosition,
            Option<&ScrollPosition>,
            Option<&UiStyle>,
            &BindToID,
        ),
        With<InputContainer>,
    >,
    text_query: Query<(&TextFont, &BindToID), With<InputFieldText>>,
    layout_query: Query<(&TextLayoutInfo, &BindToID), With<InputFieldText>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    if event.button != PointerButton::Primary {
        return;
    }

    let target = event.event_target();

    let apply_drag = |mut field: Mut<InputField>,
                      mut selection: Mut<InputSelection>,
                      state: &UIWidgetState,
                      gen_id: &UIGenID| {
        if state.disabled || !state.focused {
            return;
        }

        if let Some(pos) = cursor_position_from_pointer(
            gen_id.0,
            field.text.len(),
            &selection,
            &container_query,
            &text_query,
            &layout_query,
            &window_q,
            &ui_scale,
        ) {
            field.cursor_position = pos;
            selection.focus = pos;
            selection.dragging = true;
        }
    };

    if let Ok((field, selection, state, gen_id)) = query.get_mut(target) {
        apply_drag(field, selection, state, gen_id);
    } else if let Ok(bind) = bind_query.get(target) {
        if let Some((field, selection, state, gen_id)) =
            query.iter_mut().find(|(_, _, _, id)| id.0 == bind.0)
        {
            apply_drag(field, selection, state, gen_id);
        }
    }
}

/// Handles pointer release events for input selection.
fn on_internal_release(
    mut trigger: On<Pointer<Release>>,
    mut query: Query<(&mut InputSelection, &UIGenID), With<InputField>>,
    bind_query: Query<&BindToID>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let target = trigger.event_target();
    if let Ok((mut selection, _)) = query.get_mut(target) {
        selection.dragging = false;
    } else if let Ok(bind) = bind_query.get(target) {
        if let Some((mut selection, _)) = query.iter_mut().find(|(_, id)| id.0 == bind.0) {
            selection.dragging = false;
        }
    }

    trigger.propagate(false);
}

/// Handles click events on input fields.
/// Focuses the input field on click and updates the current widget state.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<InputField>>,
    bind_query: Query<&BindToID>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let target = trigger.event_target();
    if let Ok((mut state, gen_id)) = query.get_mut(target) {
        if !state.disabled {
            state.focused = true;
            current_widget_state.widget_id = gen_id.0;
        }
    } else if let Ok(bind) = bind_query.get(target) {
        if let Some((mut state, gen_id)) = query.iter_mut().find(|(_, id)| id.0 == bind.0) {
            if !state.disabled {
                state.focused = true;
                current_widget_state.widget_id = gen_id.0;
            }
        }
    }

    trigger.propagate(false);
}

/// Handles pointer cursor entering input fields.
/// Sets hovered state when the cursor enters an input field.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<InputField>>,
    bind_query: Query<&BindToID>,
) {
    let target = trigger.event_target();
    if let Ok((mut state, _)) = query.get_mut(target) {
        state.hovered = true;
    } else if let Ok(bind) = bind_query.get(target) {
        if let Some((mut state, _)) = query.iter_mut().find(|(_, id)| id.0 == bind.0) {
            state.hovered = true;
        }
    }

    trigger.propagate(false);
}

/// Handles pointer cursor leaving input fields.
/// Clears hovered state when the cursor leaves an input field.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<InputField>>,
    bind_query: Query<&BindToID>,
) {
    let target = trigger.event_target();
    if let Ok((mut state, _)) = query.get_mut(target) {
        state.hovered = false;
    } else if let Ok(bind) = bind_query.get(target) {
        if let Some((mut state, _)) = query.iter_mut().find(|(_, id)| id.0 == bind.0) {
            state.hovered = false;
        }
    }

    trigger.propagate(false);
}
