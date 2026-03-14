use std::collections::HashMap;
#[cfg(all(
    target_os = "linux",
    not(target_arch = "wasm32"),
    feature = "extended-dialog"
))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::services::image_service::get_or_load_image;
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
use bevy::ui::{RelativeCursorPosition, ScrollPosition};
#[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
use rfd::FileDialog;
#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
use wasm_bindgen::{JsCast, closure::Closure};
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

/// Marker component for the file-size suffix label.
#[derive(Component)]
struct InputFileSizeText;

/// Marker component for the file-size validation message.
#[derive(Component)]
struct InputFileErrorText;

/// Runtime metadata for file input selections.
#[derive(Component, Default, Clone)]
struct InputFileState {
    selected_size_bytes: Option<u64>,
    error_message: Option<String>,
}

/// Tracks key repeat timers for continuous input.
#[derive(Resource, Default)]
struct KeyRepeatTimers {
    timers: HashMap<KeyCode, Timer>,
}

#[derive(Clone, Debug)]
struct PendingFileSelection {
    target: usize,
    display_name: String,
    value: String,
    size_bytes: Option<u64>,
    error_message: Option<String>,
}

/// Bridge resource for asynchronous file picker callbacks.
#[derive(Resource, Default, Clone)]
struct InputFileDialogBridge {
    pending: Arc<Mutex<Vec<PendingFileSelection>>>,
}

#[cfg(all(
    target_os = "linux",
    not(target_arch = "wasm32"),
    feature = "extended-dialog"
))]
static FILE_DIALOG_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

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
        app.insert_resource(InputFileDialogBridge::default());
        #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
        let typing_chain = (handle_typing, sync_input_text_spans, apply_pending_paste).chain();

        #[cfg(not(all(target_arch = "wasm32", feature = "clipboard-wasm")))]
        let typing_chain = (handle_typing, sync_input_text_spans).chain();

        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                apply_pending_file_selection,
                sync_input_field_updates.after(apply_pending_file_selection),
                sync_file_input_feedback.after(apply_pending_file_selection),
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
    mut images: ResMut<Assets<Image>>,
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
            .insert(InputFileState::default())
            .with_children(|builder| {
                let icon_path = field
                    .icon_path
                    .as_deref()
                    .map(str::trim)
                    .filter(|path| !path.is_empty());
                if let Some(icon_path) = icon_path {
                    let handle =
                        get_or_load_image(icon_path, &mut image_cache, &mut images, &asset_server);

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

                if field.input_type == InputType::File && field.show_size {
                    builder.spawn((
                        Name::new(format!("Input-File-Size-{}", field.entry)),
                        Node::default(),
                        Text::new(String::new()),
                        TextColor::default(),
                        TextLayout::default(),
                        TextFont::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["input-file-size".to_string()]),
                        Pickable::IGNORE,
                        RenderLayers::layer(layer),
                        InputFileSizeText,
                        BindToID(id.0),
                    ));
                }

                if field.input_type == InputType::File {
                    builder.spawn((
                        Name::new(format!("Input-File-Error-{}", field.entry)),
                        Node::default(),
                        Text::new(String::new()),
                        TextColor::default(),
                        TextLayout::default(),
                        TextFont::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["input-file-error".to_string()]),
                        Visibility::Hidden,
                        Pickable::IGNORE,
                        RenderLayers::layer(layer),
                        InputFileErrorText,
                        BindToID(id.0),
                    ));
                }
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
    mut images: ResMut<Assets<Image>>,
    mut query: Query<
        (
            Entity,
            &mut InputField,
            &mut InputValue,
            &mut InputSelection,
            &InputFileState,
            &UIGenID,
            Option<&CssSource>,
        ),
        (With<InputFieldBase>, Changed<InputField>),
    >,
    children_query: Query<&Children, With<InputFieldBase>>,
    // text is synced in `sync_input_text_spans`
    mut label_query: Query<
        (&mut Text, &BindToID),
        (
            With<OverlayLabel>,
            Without<InputFieldText>,
            Without<InputFileSizeText>,
        ),
    >,
    mut file_size_text_query: Query<
        (&mut Text, &BindToID),
        (
            With<InputFileSizeText>,
            Without<OverlayLabel>,
            Without<InputFieldText>,
        ),
    >,
    icon_query: Query<(Entity, &BindToID), With<InputFieldIcon>>,
    mut icon_image_query: Query<(&mut ImageNode, &BindToID), With<InputFieldIconImage>>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    let mut icon_entities: HashMap<usize, Entity> = HashMap::new();
    for (entity, bind) in icon_query.iter() {
        icon_entities.insert(bind.0, entity);
    }

    for (entity, mut field, mut input_value, mut selection, file_state, ui_id, source_opt) in
        query.iter_mut()
    {
        let css_source = source_opt.cloned().unwrap_or_default();

        field.cursor_position = field.cursor_position.min(field.text.len());
        selection.anchor = selection.anchor.min(field.text.len());
        selection.focus = selection.focus.min(field.text.len());
        if field.input_type != InputType::File && input_value.0 != field.text {
            input_value.0 = field.text.clone();
        }

        for (mut label_text, bind_id) in label_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            label_text.0 = field.label.clone();
        }

        let file_size_text = if field.input_type == InputType::File && field.show_size {
            file_state
                .selected_size_bytes
                .map(format_file_size)
                .unwrap_or_default()
        } else {
            String::new()
        };
        for (mut text, bind_id) in file_size_text_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }
            text.0 = file_size_text.clone();
        }

        let icon_path = field
            .icon_path
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty());
        if let Some(icon_path) = icon_path {
            let handle = get_or_load_image(icon_path, &mut image_cache, &mut images, &asset_server);

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

fn apply_pending_file_selection(
    bridge: Res<InputFileDialogBridge>,
    mut query: Query<
        (
            &UIGenID,
            &mut InputField,
            &mut InputValue,
            &mut InputSelection,
            &mut InputFileState,
            &mut UIWidgetState,
        ),
        With<InputField>,
    >,
) {
    let pending = {
        let Ok(mut guard) = bridge.pending.lock() else {
            return;
        };
        std::mem::take(&mut *guard)
    };

    if pending.is_empty() {
        return;
    }

    for selection in pending {
        if let Some((
            _,
            mut field,
            mut input_value,
            mut text_selection,
            mut file_state,
            mut state,
        )) = query
            .iter_mut()
            .find(|(ui_id, _, _, _, _, _)| ui_id.0 == selection.target)
        {
            if let Some(error_message) = selection.error_message {
                file_state.error_message = Some(error_message);
                state.invalid = true;
                state.focused = false;
                clear_selection(&mut text_selection, field.cursor_position);
                continue;
            }

            field.text = selection.display_name;
            field.cursor_position = field.text.len();
            file_state.selected_size_bytes = selection.size_bytes;
            file_state.error_message = None;

            if input_value.0 != selection.value {
                input_value.0 = selection.value;
            }

            state.invalid = false;
            state.focused = false;
            clear_selection(&mut text_selection, field.cursor_position);
        }
    }
}

/// Syncs visual feedback message for file-input validation failures (e.g. max-size).
fn sync_file_input_feedback(
    input_query: Query<
        (&UIGenID, &InputField, &InputFileState),
        (With<InputFieldBase>, Changed<InputFileState>),
    >,
    mut error_query: Query<(&mut Text, &mut Visibility, &BindToID), With<InputFileErrorText>>,
) {
    for (ui_id, field, file_state) in input_query.iter() {
        if field.input_type != InputType::File {
            continue;
        }

        let has_error = file_state
            .error_message
            .as_ref()
            .is_some_and(|message| !message.trim().is_empty());

        for (mut text, mut visibility, bind_id) in error_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            if has_error {
                text.0 = file_state.error_message.clone().unwrap_or_default();
                *visibility = Visibility::Inherited;
            } else {
                text.0.clear();
                *visibility = Visibility::Hidden;
            }
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
        is_file: bool,
    }

    let mut fields: HashMap<usize, FieldView> = HashMap::new();
    for (field, state, ui_id) in input_field_query.iter() {
        fields.insert(
            ui_id.0,
            FieldView {
                focused: state.focused,
                disabled: state.disabled,
                is_file: field.input_type == InputType::File,
            },
        );
    }

    for (mut visibility, mut background, mut styles, bind_cursor_id) in cursor_query.iter_mut() {
        let Some(field) = fields.get(&bind_cursor_id.0) else {
            continue;
        };

        if field.focused && !field.disabled && !field.is_file {
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

        if !state.focused || state.disabled || text_field.input_type == InputType::File {
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
/// - If the input field has an icon, reduces the text container width to accommodate the icon.
/// - If file size suffix is enabled, reserves extra width on the right side.
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

            let icon_reserve = input_field
                .icon_path
                .as_deref()
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(|_| 15.0)
                .unwrap_or(0.0);
            let file_size_reserve =
                if input_field.input_type == InputType::File && input_field.show_size {
                    25.0
                } else {
                    0.0
                };
            let reserve = icon_reserve + file_size_reserve;
            let target = (original_width.0 - reserve).max(0.0);

            for (_, value) in style.styles.iter_mut() {
                value.normal.width = Some(Val::Percent(target));
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
        if !state.focused || state.disabled || input_field.input_type == InputType::File {
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

        if in_field.input_type == InputType::File {
            clear_selection(&mut selection, in_field.cursor_position);
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

fn normalized_extensions(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .map(|token| token.trim_start_matches('.'))
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = bytes as f64;
    let mut unit_idx = 0usize;
    while unit_idx < UNITS.len() - 1 && value >= 900.0 {
        value /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.0} {}", value.round(), UNITS[unit_idx])
}

fn push_pending_file_selection(
    bridge: &InputFileDialogBridge,
    target: usize,
    display_name: String,
    value: String,
    size_bytes: Option<u64>,
) {
    let Ok(mut guard) = bridge.pending.lock() else {
        return;
    };
    guard.push(PendingFileSelection {
        target,
        display_name,
        value,
        size_bytes,
        error_message: None,
    });
}

fn push_pending_file_selection_error(
    bridge: &InputFileDialogBridge,
    target: usize,
    error_message: String,
) {
    let Ok(mut guard) = bridge.pending.lock() else {
        return;
    };
    guard.push(PendingFileSelection {
        target,
        display_name: String::new(),
        value: String::new(),
        size_bytes: None,
        error_message: Some(error_message),
    });
}

#[cfg(all(
    target_os = "linux",
    not(target_arch = "wasm32"),
    feature = "extended-dialog"
))]
fn spawn_linux_file_dialog_task(task: impl FnOnce() + Send + 'static) -> bool {
    if FILE_DIALOG_IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        warn!("File dialog request ignored: another file dialog is already open (linux backend).");
        return false;
    }

    std::thread::spawn(move || {
        task();
        FILE_DIALOG_IN_FLIGHT.store(false, Ordering::Release);
    });

    true
}

#[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
fn run_native_file_picker(
    target: usize,
    folder_mode: bool,
    extensions: Vec<String>,
    include_size: bool,
    max_size_bytes: Option<u64>,
    bridge: InputFileDialogBridge,
) {
    let mut dialog = FileDialog::new();
    if !folder_mode && !extensions.is_empty() {
        let extension_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
        dialog = dialog.add_filter("Allowed", &extension_refs);
    }

    if folder_mode {
        if let Some(path) = dialog.pick_folder() {
            let display_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            let value = path.to_string_lossy().into_owned();
            push_pending_file_selection(&bridge, target, display_name, value, None);
        }
        return;
    }

    if let Some(path) = dialog.pick_file() {
        let display_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let value = path.to_string_lossy().into_owned();
        let selected_size = std::fs::metadata(&path).ok().map(|meta| meta.len());

        if let (Some(limit), Some(size)) = (max_size_bytes, selected_size)
            && size > limit
        {
            push_pending_file_selection_error(
                &bridge,
                target,
                format!("File is too large (max: {})", format_file_size(limit)),
            );
            warn!(
                "File selection rejected: '{}' is {} bytes but max-size is {} bytes.",
                display_name, size, limit
            );
            return;
        }

        let size_bytes = if include_size { selected_size } else { None };

        push_pending_file_selection(&bridge, target, display_name, value, size_bytes);
    }
}

fn open_file_picker(target: usize, field: &InputField, bridge: &InputFileDialogBridge) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
    {
        let folder_mode = field.folder;
        let extensions = normalized_extensions(&field.extensions);
        let include_size = field.show_size && !folder_mode;
        let max_size_bytes = if folder_mode {
            None
        } else {
            field.max_size_bytes
        };

        #[cfg(target_os = "linux")]
        {
            let bridge = bridge.clone();
            let _ = spawn_linux_file_dialog_task(move || {
                run_native_file_picker(
                    target,
                    folder_mode,
                    extensions,
                    include_size,
                    max_size_bytes,
                    bridge,
                );
            });
            return;
        }

        #[cfg(not(target_os = "linux"))]
        {
            run_native_file_picker(
                target,
                folder_mode,
                extensions,
                include_size,
                max_size_bytes,
                bridge.clone(),
            );
            return;
        }
    }

    #[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Ok(element) = document.create_element("input") else {
            return;
        };
        let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };

        input.set_type("file");
        let _ = input.set_attribute("style", "display: none;");

        let folder_mode = field.folder;
        if folder_mode {
            let _ = input.set_attribute("webkitdirectory", "");
            let _ = input.set_attribute("directory", "");
        } else {
            let extensions = normalized_extensions(&field.extensions);
            if !extensions.is_empty() {
                let accept = extensions
                    .iter()
                    .map(|ext| format!(".{ext}"))
                    .collect::<Vec<_>>()
                    .join(",");
                input.set_accept(&accept);
            }
        }

        let bridge = bridge.clone();
        let input_clone = input.clone();
        let include_size = field.show_size && !folder_mode;
        let max_size_bytes = if folder_mode {
            None
        } else {
            field.max_size_bytes
        };

        let onchange = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let Some(files) = input_clone.files() else {
                return;
            };

            let Some(file) = files.item(0) else {
                return;
            };

            let (display_name, size_bytes) = if folder_mode {
                let relative = js_sys::Reflect::get(
                    file.as_ref(),
                    &wasm_bindgen::JsValue::from_str("webkitRelativePath"),
                )
                .ok()
                .and_then(|value| value.as_string())
                .unwrap_or_default();

                let folder_name = relative
                    .split('/')
                    .next()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| file.name());
                (folder_name, None)
            } else {
                let selected_size = file.size() as u64;
                if let Some(limit) = max_size_bytes
                    && selected_size > limit
                {
                    push_pending_file_selection_error(
                        &bridge,
                        target,
                        format!("File is too large (max: {})", format_file_size(limit)),
                    );
                    warn!(
                        "File selection rejected: '{}' is {} bytes but max-size is {} bytes.",
                        file.name(),
                        selected_size,
                        limit
                    );
                    return;
                }

                let size = if include_size {
                    Some(selected_size)
                } else {
                    None
                };
                let display_name = file.name();
                let bridge_load = bridge.clone();
                let display_name_load = display_name.clone();
                let reader_name_fallback = display_name.clone();
                let Ok(reader) = web_sys::FileReader::new() else {
                    push_pending_file_selection(
                        &bridge,
                        target,
                        display_name.clone(),
                        display_name,
                        size,
                    );
                    return;
                };
                let reader_for_load = reader.clone();
                let onload = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    let value = reader_for_load
                        .result()
                        .ok()
                        .and_then(|data| data.as_string())
                        .unwrap_or_else(|| reader_name_fallback.clone());

                    push_pending_file_selection(
                        &bridge_load,
                        target,
                        display_name_load.clone(),
                        value,
                        size,
                    );
                }) as Box<dyn FnMut(_)>);
                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
                let _ = reader.read_as_data_url(&file);
                return;
            };

            push_pending_file_selection(
                &bridge,
                target,
                display_name.clone(),
                display_name,
                size_bytes,
            );
        }) as Box<dyn FnMut(_)>);

        input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
        onchange.forget();

        input.click();
        return;
    }
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
) -> Option<usize> {
    if text_len == 0 {
        return Some(0);
    }

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

        // Use the node-local inverse scale instead of recomputing from the window scale.
        // This avoids platform-specific mismatches (notably on Windows DPI scaling).
        let width = (node.size().x * node.inverse_scale_factor).max(1.0);
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

        if field.input_type == InputType::File {
            state.focused = false;
            selection.dragging = false;
            clear_selection(&mut selection, field.cursor_position);
            current_widget_state.widget_id = gen_id.0;
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
) {
    if event.button != PointerButton::Primary {
        return;
    }

    let target = event.event_target();

    let apply_drag = |mut field: Mut<InputField>,
                      mut selection: Mut<InputSelection>,
                      state: &UIWidgetState,
                      gen_id: &UIGenID| {
        if state.disabled || !state.focused || field.input_type == InputType::File {
            return;
        }

        if let Some(pos) = cursor_position_from_pointer(
            gen_id.0,
            field.text.len(),
            &selection,
            &container_query,
            &text_query,
            &layout_query,
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
    mut query: Query<(&mut UIWidgetState, &UIGenID, &InputField), With<InputField>>,
    bind_query: Query<&BindToID>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
    file_dialog_bridge: Res<InputFileDialogBridge>,
) {
    let target = trigger.event_target();
    if let Ok((mut state, gen_id, field)) = query.get_mut(target) {
        if !state.disabled {
            current_widget_state.widget_id = gen_id.0;
            if field.input_type == InputType::File {
                state.focused = false;
                open_file_picker(gen_id.0, field, &file_dialog_bridge);
            } else {
                state.focused = true;
            }
        }
    } else if let Ok(bind) = bind_query.get(target) {
        if let Some((mut state, gen_id, field)) = query.iter_mut().find(|(_, id, _)| id.0 == bind.0)
        {
            if !state.disabled {
                current_widget_state.widget_id = gen_id.0;
                if field.input_type == InputType::File {
                    state.focused = false;
                    open_file_picker(gen_id.0, field, &file_dialog_bridge);
                } else {
                    state.focused = true;
                }
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
