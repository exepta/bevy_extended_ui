use std::collections::HashMap;
use std::time::Duration;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, ImageCache, UIGenID, UIWidgetState};
use crate::styling::{Background, FontVal};
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::utils::keycode_to_char;
use crate::widgets::{InputCap, InputField, InputType, WidgetId, WidgetKind};

#[derive(Component)]
struct InputFieldBase;

#[derive(Component)]
struct InputFieldText;

#[derive(Component)]
struct InputFieldIcon;

#[derive(Component)]
struct InputCursor;

#[derive(Component)]
struct InputContainer;

#[derive(Component)]
struct OverlayLabel;

#[derive(Resource, Default)]
struct KeyRepeatTimers {
    timers: HashMap<KeyCode, Timer>,
}

#[derive(Component)]
struct OriginalWidth(pub f32);

#[derive(Resource)]
pub struct CursorBlinkTimer {
    pub timer: Timer,
}

impl Default for CursorBlinkTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.95, TimerMode::Repeating)
        }
    }
}

pub struct InputWidget;

impl Plugin for InputWidget {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyRepeatTimers::default());
        app.insert_resource(CursorBlinkTimer::default());
        app.add_systems(Update, (
            internal_node_creation_system,
            update_cursor_visibility,
            update_cursor_position,
            handle_typing,
            handle_input_horizontal_scroll,
            calculate_correct_text_container_width,
            handle_overlay_label
        ));
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
///
/// # Parameters
/// - `commands`: Commands to insert or modify entities and components.
/// - `query`: Query for entities with `InputField` and without `InputFieldBase`, also getting the unique `UIGenID` and mutable `InputField`.
/// - `config`: Resource containing UI configuration, including render layers.
/// - `asset_server`: Resource used to load images (icons).
/// - `image_cache`: Mutable resource caching loaded images to avoid duplicate loads.
///
/// # Behavior
/// - If an `InputField` has an `icon_path`, loads or reuses the icon image and attaches it as a child node.
/// - Creates a label node overlaying the input field with the label text.
/// - Creates a container for the text content, with cursor and text nodes as children.
/// - Sets the render layer to the first configured render layer or defaults to layer 1.
fn internal_node_creation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &UIGenID, &mut InputField, Option<&CssSource>), (With<InputField>, Without<InputFieldBase>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, mut field, source_opt) in query.iter_mut() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }
        
        if !field.text.is_empty() {
            field.cursor_position = field.text.len();
        }

        commands.entity(entity).insert((
            Name::new(format!("Input-{}", field.w_count)),
            Node::default(),
            WidgetId {
                id: field.w_count,
                kind: WidgetKind::InputField
            },
            BackgroundColor::default(),
            ImageNode::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            Pickable::default(),
            css_source.clone(),
            TagName(String::from("input")),
            RenderLayers::layer(*layer),
            InputFieldBase
        )).with_children(|builder| {
            if let Some(icon_path) = field.icon_path.clone() {
                let owned_icon = icon_path.to_string();
                let handle = image_cache.map.entry(icon_path.clone())
                    .or_insert_with(|| asset_server.load(owned_icon.clone()))
                    .clone();

                // Icon left
                builder.spawn((
                    Name::new(format!("Input-Icon-{}", field.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    ImageNode::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    css_source.clone(),
                    CssClass(vec!["in-icon-container".to_string()]),
                    Pickable::IGNORE,
                    RenderLayers::layer(*layer),
                    InputFieldIcon,
                    BindToID(id.0),
                    children![
                        (
                            Name::new(format!("Icon-{}", field.w_count)),
                            ImageNode {
                                image: handle,
                                ..default()
                            },
                            ZIndex::default(),
                            UIWidgetState::default(),
                            css_source.clone(),
                            CssClass(vec!["in-icon".to_string()]),
                            Pickable::IGNORE,
                            RenderLayers::layer(*layer),
                            BindToID(id.0),
                        )
                    ]
                ));
            }
            
            // Overlay label
            builder.spawn((
                Name::new(format!("Input-Label-{}", field.w_count)),
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
                RenderLayers::layer(*layer),
                OverlayLabel,
                BindToID(id.0)
            ));
            
            // Text content children
            builder.spawn((
                Name::new(format!("Input-Text-Container-{}", field.w_count)),
                Node::default(),
                BackgroundColor::default(),
                BorderColor::default(),
                BorderRadius::default(),
                ZIndex::default(),
                UIWidgetState::default(),
                css_source.clone(),
                CssClass(vec!["in-text-container".to_string()]),
                Pickable::IGNORE,
                OriginalWidth(-1.),
                RenderLayers::layer(*layer),
                InputContainer,
                BindToID(id.0),
                children![
                    // Input Cursor
                    (
                        Name::new(format!("Cursor-{}", field.w_count)),
                        Node::default(),
                        BackgroundColor::default(),
                        ImageNode::default(),
                        BorderColor::default(),
                        BorderRadius::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["input-cursor".to_string()]),
                        Visibility::Hidden,
                        Pickable::IGNORE,
                        RenderLayers::layer(*layer),
                        InputCursor,
                        BindToID(id.0),
                    ),
                    // Input Text
                    (
                        Name::new(format!("Text-{}", field.w_count)),
                        Node::default(),
                        Text::new(field.text.clone()),
                        TextColor::default(),
                        TextLayout::default(),
                        TextFont::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["input-text".to_string()]),
                        Pickable::IGNORE,
                        RenderLayers::layer(*layer),
                        InputFieldText,
                        BindToID(id.0),
                    )
                ]
            )).insert(ImageNode::default());
        }).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

// ===============================================
//             Intern Functions
// ===============================================

/// Updates the visibility and blinking animation of the input cursor.
///
/// - Blinks the cursor by adjusting its alpha transparency using a sine wave based on elapsed time.
/// - Shows the cursor only if the associated `InputField` is focused.
/// - Updates the displayed text for the input field:
///   - Masks the text with `*` characters if the input type is `Password`.
///   - Shows placeholder text if the input is empty.
/// - Hides the cursor and clears the text display when the input field is unfocused.
///
/// # Parameters
/// - `time`: Provides delta time for timer updates.
/// - `cursor_blink_timer`: Timer resource managing the blinking rate.
/// - `cursor_query`: Query to access cursor entities with their visibility, background color, style, and bound UI ID.
/// - `input_field_query`: Query for input fields with their state and unique UI ID.
/// - `text_query`: Query for the text entities associated with input fields.
///
/// # Behavior
/// This system synchronizes the visual cursor and input text representation with the focus and text content of the input field.
fn update_cursor_visibility(
    time: Res<Time>,
    mut cursor_blink_timer: ResMut<CursorBlinkTimer>,
    mut cursor_query: Query<(&mut Visibility, &mut BackgroundColor, &mut WidgetStyle, &BindToID), With<InputCursor>>,
    mut input_field_query: Query<(&InputField, &mut UIWidgetState, &UIGenID), With<InputFieldBase>>,
    mut text_query: Query<(&mut Text, &BindToID), With<InputFieldText>>,
) {
    cursor_blink_timer.timer.tick(time.delta());

    for (mut visibility, mut background, mut styles, bind_cursor_id) in cursor_query.iter_mut() {
        for (in_field, state, ui_id) in input_field_query.iter_mut() {
            if bind_cursor_id.0 != ui_id.0 {
                continue;
            }
            // Show the cursor if the input field is focused
            if state.focused {
                let alpha = (cursor_blink_timer.timer.elapsed_secs() * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;
                background.0.set_alpha(alpha);

                for (_, style)  in styles.styles.iter_mut() {
                    style.background = Some(Background { color: background.0, ..default() });
                }

                if !visibility.eq(&Visibility::Inherited) || !visibility.eq(&Visibility::Visible) {

                    *visibility = Visibility::Inherited;
                    for (mut text, bind_id) in text_query.iter_mut() {
                        if bind_id.0 != ui_id.0 {
                            continue;
                        }
                        if in_field.input_type.eq(&InputType::Password) {
                            if in_field.text.is_empty() {
                                text.0 = in_field.placeholder.clone();
                            } else {
                                let masked_text: String = "*".repeat(in_field.text.chars().count());
                                text.0 = masked_text;
                            }
                        } else {
                            let mut show_text = in_field.text.clone();
                            if show_text.is_empty() {
                                show_text = in_field.placeholder.clone();
                            }
                            text.0 = show_text;
                        }
                    }
                }
            } else {
                if !visibility.eq(&Visibility::Hidden) {
                    *visibility = Visibility::Hidden;

                    for (mut text, bind_id) in text_query.iter_mut() {
                        if bind_id.0 != ui_id.0 {
                            continue;
                        }
                        if in_field.text.is_empty() {
                            text.0 = String::from("");
                        }
                    }
                }
            }
        }
    }
}

/// Updates the cursor position within the input text based on keyboard input.
///
/// - Handles left and right arrow keys with initial delay and repeat rate timers.
/// - Calculates the horizontal pixel position of the cursor based on text font metrics.
/// - Updates the cursor node's CSS left position to reflect the cursor's position in the text.
///
/// # Parameters
/// - `key_repeat`: Resource tracking timers for repeated key presses.
/// - `cursor_query`: Query for cursor node and its styling information.
/// - `text_field_query`: Query for input fields (excluding cursor entities).
/// - `text_query`: Query for text font info linked to input fields.
/// - `keyboard_input`: Provides current keyboard input state.
/// - `time`: Provides delta time for timer updates.
///
/// # Behavior
/// Enables smooth, timed cursor movement on arrow key presses with continuous key hold support.
fn update_cursor_position(
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut cursor_query: Query<(&mut Node, &mut WidgetStyle, &BindToID), With<InputCursor>>,
    mut text_field_query: Query<(&mut InputField, &UIGenID), (With<InputField>, Without<InputCursor>)>,
    text_query: Query<(&TextFont, &BindToID), (With<InputFieldText>, Without<InputCursor>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    for (mut cursor_node, mut styles, bind_id) in cursor_query.iter_mut() {
        for (mut text_field, ui_id) in text_field_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }
            // ARROW LEFT
            if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
                text_field.cursor_position = text_field.cursor_position.saturating_sub(1);
                key_repeat.timers.insert(
                    KeyCode::ArrowLeft,
                    Timer::from_seconds(initial_delay, TimerMode::Once),
                );
            }

            // ARROW RIGHT
            if keyboard_input.just_pressed(KeyCode::ArrowRight) {
                text_field.cursor_position = (text_field.cursor_position + 1).min(text_field.text.len());
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
                                    text_field.cursor_position = text_field.cursor_position.saturating_sub(1);
                                }
                                KeyCode::ArrowRight => {
                                    text_field.cursor_position = (text_field.cursor_position + 1).min(text_field.text.len());
                                }
                                _ => {}
                            }

                            timer.set_duration(Duration::from_secs_f32(repeat_rate));
                            timer.reset();
                        }
                    }
                }
            }

            let Some((text_font, _)) = text_query
                .iter()
                .find(|(_, bind_id)| bind_id.0 == ui_id.0)
            else {
                continue;
            };

            let cursor_x_position = calculate_cursor_x_position(&text_field, text_field.cursor_position, text_font);
            cursor_node.left = Val::Px(cursor_x_position);

            for (_, style) in styles.styles.iter_mut() {
                style.left = Some(cursor_node.left);
            }
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
///
/// # Parameters
/// - `query`: Query for input fields with changes or additions and without an existing text container.
/// - `container_query`: Query for the input text container widget styles and widths.
///
/// # Behavior
/// Ensures the text container does not overlap with the icon by shrinking appropriately on style or input field changes.

fn calculate_correct_text_container_width(
    query: Query<(
        &InputField,
        &UIGenID,
    ), (With<InputField>, Without<InputContainer>, Or<(Added<InputField>, Changed<WidgetStyle>, Changed<InputField>)>)>,
    mut container_query: Query<(&mut WidgetStyle, &mut OriginalWidth, &BindToID), (With<InputContainer>, Without<InputField>)>,
) {
    for (input_field, ui_id) in query.iter() {
        for (mut style, mut original_width, bind_id) in container_query.iter_mut() {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            if input_field.icon_path.is_some() {
                if let Some(active) = style.active_style.clone() {
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
                        value.width = Some(Val::Percent(current - 15.0));
                    }
                }
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
///
/// # Parameters
/// - `query`: Query to get all input fields, their UI IDs, and widget states.
/// - `scroll_query`: Query to get scroll position and UI binding for input containers.
/// - `text_node_query`: Query for the computed size of the text node and its font.
///
/// # Behavior
/// Supports only `InputCap::NoCap` currently; other caps are ignored.
fn handle_input_horizontal_scroll(
    mut query: Query<(
        &InputField,
        &UIGenID,
        &UIWidgetState
    ), With<InputFieldBase>>,
    mut scroll_query: Query<(&mut ScrollPosition, &BindToID), With<InputContainer>>,
    text_node_query: Query<(&ComputedNode, &BindToID, &TextFont), With<InputFieldText>>
) {
    for (input_field, ui_id, state) in &mut query {
        if !state.focused {
            continue;
        }

        let Some((text_node, _, text_font)) = text_node_query
            .iter()
            .find(|(_, bind_id, _)| bind_id.0 == ui_id.0)
        else {
            continue;
        };

        let char_width = text_font.font_size;
        let cursor_x = input_field.cursor_position as f32 * char_width;

        let available_width = text_node.size().x - 10.0;

        for (mut scroll, bind_id) in scroll_query.iter_mut() {
            if bind_id.0 == ui_id.0 {
                match input_field.cap_text_at {
                    InputCap::NoCap => {
                        let visible_left = scroll.x;
                        let visible_right = scroll.x + available_width;

                        if cursor_x > visible_right {
                            scroll.x = cursor_x - available_width + char_width;
                        }
                        else if cursor_x < visible_left {
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
}

/// Processes typing input for text fields, including key repeat and special keys.
///
/// - Handles insertion of characters respecting input caps and input type validations.
/// - Supports backspace with key repeat functionality.
/// - Handles Enter key to lose focus and optionally clear input field text.
/// - Updates the visible text and cursor position accordingly.
/// - Masks input with `*` characters if an input type is `Password`.
/// - Updates text color on changes.
///
/// # Parameters
/// - `time`: Provides delta time for timer updates.
/// - `key_repeat`: Resource managing timers for repeated key presses.
/// - `query`: Query for mutable access to input fields, their UI state, styles, and IDs.
/// - `keyboard`: Provides keyboard input states.
/// - `text_query`: Query for mutable access to text entities linked to input fields.
///
/// # Behavior
/// Implements initial delay and repeat rate for key holding behavior?
/// Maintains synchronization between input field state and displayed text.
fn handle_typing(
    time: Res<Time>,
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut query: Query<(&mut InputField, &mut UIWidgetState, &WidgetStyle, &UIGenID)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut text_query: Query<(&mut Text, &mut TextColor, &WidgetStyle, &ComputedNode, &BindToID), (With<InputFieldText>, With<BindToID>)>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    for (mut in_field, mut state, style, ui_id) in query.iter_mut() {
        if state.focused {
            for (mut text, mut text_color, widget_style, computed_node, bind_id) in text_query.iter_mut() {
                if bind_id.0 == ui_id.0 {
                    // ENTER
                    if keyboard.just_pressed(KeyCode::Enter) {
                        state.focused = false;
                        if in_field.clear_after_focus_lost {
                            in_field.text.clear();
                            text.0 = in_field.text.clone();
                        }
                        continue;
                    }

                    // BACKSPACE
                    if keyboard.just_pressed(KeyCode::Backspace) {
                        if in_field.cursor_position > 0 && !in_field.text.is_empty() {
                            let pos = in_field.cursor_position - 1;
                            in_field.cursor_position = pos;
                            in_field.text.remove(pos);
                            if in_field.input_type.eq(&InputType::Password) {
                                text.0.remove(pos);
                            } else {
                                text.0 = in_field.text.clone();
                            }
                        }
                        if text.0.is_empty() {
                            text_color.0 = get_active_text_color(widget_style);
                            text.0 = in_field.placeholder.clone();
                        }
                        key_repeat.timers.insert(
                            KeyCode::Backspace,
                            Timer::from_seconds(initial_delay, TimerMode::Once),
                        );
                        continue;
                    }

                    for key in keyboard.get_pressed() {
                        if let Some(char) = keycode_to_char(*key, shift, alt) {
                            if !in_field.input_type.is_valid_char(char) {
                                return;
                            }
                            if keyboard.just_pressed(*key) {
                                let pos = in_field.cursor_position;

                                if in_field.cap_text_at.get_value() > 0 {
                                    let cap = in_field.cap_text_at.clone();
                                    if pos >= cap.get_value() {
                                        return;
                                    }
                                }

                                if in_field.cap_text_at.eq(&InputCap::CapAtNodeSize) {
                                    let allowed_char_len = (computed_node.size().x / (
                                        if let Some(active_style) = style.active_style.clone() { 
                                            active_style.font_size.unwrap_or(FontVal::Px(13.)).get(None) 
                                        } else { 13. }
                                        )).round() as usize;
                                    if pos >= allowed_char_len {
                                        return;
                                    }
                                }

                                if in_field.input_type.eq(&InputType::Password) {
                                    in_field.text.insert(pos, char);
                                    in_field.cursor_position += 1;
                                    let masked_text: String = "*".repeat(in_field.text.chars().count());
                                    text.0 = masked_text;
                                } else {
                                    in_field.text.insert(pos, char);
                                    in_field.cursor_position += 1;
                                    text.0 = in_field.text.clone();
                                }
                                text_color.0 = get_active_text_color(widget_style);
                                key_repeat.timers.insert(
                                    *key,
                                    Timer::from_seconds(initial_delay, TimerMode::Once),
                                );
                                continue;
                            }

                            if let Some(timer) = key_repeat.timers.get_mut(key) {
                                timer.tick(time.delta());
                                if timer.is_finished() {
                                    in_field.text.push(char);
                                    in_field.cursor_position += 1;
                                    if in_field.input_type.eq(&InputType::Password) {
                                        text.0.push('*');
                                    } else {
                                        text.0 = in_field.text.clone();
                                    }
                                    timer.set_duration(Duration::from_secs_f32(repeat_rate));
                                    timer.reset();
                                }
                            }
                        }
                    }

                    if keyboard.pressed(KeyCode::Backspace) {
                        if let Some(timer) = key_repeat.timers.get_mut(&KeyCode::Backspace) {
                            timer.tick(time.delta());
                            if timer.is_finished() {
                                if in_field.cursor_position > 0 && !in_field.text.is_empty() {
                                    in_field.text.pop();
                                    in_field.cursor_position -= 1;
                                    if in_field.input_type.eq(&InputType::Password) {
                                        text.0.pop();
                                    } else {
                                        text.0 = in_field.text.clone();
                                    }
                                    timer.set_duration(Duration::from_secs_f32(repeat_rate));
                                    timer.reset();
                                }
                            }
                        }
                    }

                    in_field.cursor_position = in_field.cursor_position.min(in_field.text.len());
                }
            }
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
///
/// # Parameters
/// - `query`: Queries all input fields with their UI state, styles, unique IDs, and child entities (labels).
/// - `label_query`: Queries the label entities to update their node position, font size, and styles.
/// - `icon_container_query`: Queries any icon containers to calculate offset for label positioning.
///
/// # Behavior
/// The function matches labels to input fields by `UIGenID`.
/// It respects the active widget style height for vertical positioning.
/// The horizontal offset is adjusted if an icon is present, based on the icon's active width.
///
/// # Notes
/// This system requires that each input field's label is a child entity with the `OverlayLabel` component,
/// and that input icons have the `InputFieldIcon` component.
///
/// # Example
/// ```ignore
/// // When input is focused:
/// // label font size = 10, position near top.
/// // When input unfocused and empty:
/// // label font size = 14, position centered.
/// ```
fn handle_overlay_label(
    query: Query<(&UIWidgetState, &UIGenID, &InputField, &WidgetStyle, &Children), (With<InputField>, Without<OverlayLabel>)>,
    mut label_query: Query<(&BindToID, &mut Node, &mut TextFont, &mut WidgetStyle), (With<OverlayLabel>, Without<InputField>)>,
    icon_container_query: Query<(&WidgetStyle, &BindToID), (With<InputFieldIcon>, Without<OverlayLabel>)>,
) {
    for (state, gen_id, in_field, in_style, children) in query.iter() {
        for child in children.iter() {
            if let Ok((bind_to, mut node, mut text_font, mut styles)) = label_query.get_mut(child) {
                if bind_to.0 != gen_id.0 {
                    continue;
                }

                if let Some(active_style) = in_style.active_style.clone() {
                    let height = match active_style.height.unwrap_or_default() {
                        Val::Px(px) => px,
                        _ => 55.,
                    };

                    let center = (height / 2.0) - text_font.font_size / 1.5;
                    let on_top = text_font.font_size / 2.0;

                    if state.focused {
                        node.top = Val::Px(on_top);
                        text_font.font_size = 10.;
                    } else {
                        if in_field.text.is_empty() {
                            node.top = Val::Px(center);
                            text_font.font_size = 14.;
                        } else {
                            node.top = Val::Px(on_top);
                            text_font.font_size = 10.;
                        }
                    }

                    if let Some((icon_style, _)) = icon_container_query.iter().find(|(_, icon_bind_to)| icon_bind_to.0 == gen_id.0) {
                        if let Some(active_style) = icon_style.active_style.clone() {
                            if let Some(Val::Px(new_width)) = active_style.width {
                                let left_now = match node.left {
                                    Val::Px(px) => px,
                                    _ => 10.,
                                };

                                let expected_left = 5.0 + new_width;
                                if (left_now - expected_left).abs() > f32::EPSILON {
                                    node.left = Val::Px(expected_left);
                                }
                            }
                        }
                    }

                    for (_, style) in styles.styles.iter_mut() {
                        style.top = Some(node.top);
                        style.left = Some(node.left);
                        style.font_size = Some(FontVal::Px(text_font.font_size));
                    }
                }
            }
        }
    }
}

// ===============================================
//             Intern Helper Functions
// ===============================================

/// Calculates the horizontal pixel position of the cursor within the input text.
///
/// The position is computed as the width of the substring from the start-up to the cursor index,
/// plus a small padding to avoid the cursor overlapping the last character.
///
/// # Parameters
/// - `text_field`: The `InputField` containing the full text.
/// - `cursor_pos`: The current cursor position (character index) within the text.
/// - `style`: The font style used to determine character width.
///
/// # Returns
/// The x position in pixels where the cursor should be drawn, relative to the input container's left edge.
fn calculate_cursor_x_position(text_field: &InputField, cursor_pos: usize, style: &TextFont) -> f32 {
    // Ensure the cursor position is within the bounds of the text
    if text_field.text.is_empty() || cursor_pos == 0 {
        return 0.0; // No text or cursor at the start
    }

    // Ensure the cursor position doesn't exceed the text length
    let cursor_pos = cursor_pos.min(text_field.text.len());

    // Calculate the width of the text up to the cursor position
    let text_substr = &text_field.text[..cursor_pos];
    let text_width = calculate_text_width(text_substr, style);

    text_width + 1.0 // Add some padding so the cursor isn't directly on the text
}

/// Estimates the pixel width of a given text string based on font size.
///
/// This simplistic calculation assumes a fixed width per character multiplied by the font size,
/// scaled by a factor to approximate the actual rendered width.
///
/// # Parameters
/// - `text`: The text string to measure.
/// - `style`: The font style (primarily font size).
///
/// # Returns
/// The estimated width in pixels of the rendered text.
fn calculate_text_width(text: &str, style: &TextFont) -> f32 {
    // Calculate text width based on font size
    text.len() as f32 * style.font_size * 0.6 // Adjust a factor based on font characteristics
}

/// Retrieves the active text color from a widget style, falling back to white.
///
/// # Parameters
/// - `style`: The widget style which may contain an active color.
///
/// # Returns
/// The active `Color` if defined, otherwise defaults to white.
fn get_active_text_color(style: &WidgetStyle) -> Color {
    style
        .active_style
        .as_ref()
        .and_then(|s| s.color)
        .unwrap_or(Color::WHITE)
}

// ===============================================
//                   Intern Events
// ===============================================

/// Handles click events on input fields.
///
/// When an input field receives a click event, this function sets its focused state to true
/// and updates the globally tracked current widget ID.
///
/// # Parameters
/// - `trigger`: The pointer click trigger containing the target entity.
/// - `query`: Query to access mutable UI widget state and generation ID of input fields.
/// - `current_widget_state`: Mutable resource tracking the currently focused widget ID.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<InputField>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.event_target()) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }

    trigger.propagate(false);
}

/// Handles pointer cursor entering input fields.
///
/// When the cursor enters an input field's area, this sets its hovered state to true.
///
/// # Parameters
/// - `trigger`: The pointer over trigger containing the target entity.
/// - `query`: Query to access mutable UI widget state of input fields.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<InputField>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.event_target()) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Handles pointer cursor leaving input fields.
///
/// When the cursor leaves an input field's area, this sets its hovered state to false.
///
/// # Parameters
/// - `trigger`: The pointer out trigger containing the target entity.
/// - `query`: Query to access mutable UI widget state of input fields.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<InputField>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.event_target()) {
        state.hovered = false;
    }

    trigger.propagate(false);
}