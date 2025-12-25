use std::collections::HashMap;
use std::time::Duration;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::styles::{Background, CssClass, CssSource, FontVal, TagName};
use crate::utils::keycode_to_char;
use crate::widgets::{
    BindToID, InputCap, InputField, InputType, UIGenID, UIWidgetState, WidgetId, WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};

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
            timer: Timer::from_seconds(0.95, TimerMode::Repeating),
        }
    }
}

pub struct InputWidget;

impl Plugin for InputWidget {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyRepeatTimers::default());
        app.insert_resource(CursorBlinkTimer::default());
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                update_cursor_visibility,
                update_cursor_position,
                handle_typing,
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
                BorderRadius::default(),
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
            ))
            .with_children(|builder| {
                if let Some(icon_path) = field.icon_path.clone() {
                    let owned_icon = icon_path.to_string();
                    let handle = image_cache
                        .map
                        .entry(icon_path.clone())
                        .or_insert_with(|| asset_server.load(owned_icon))
                        .clone();

                    // Icon left
                    builder.spawn((
                        Name::new(format!("Input-Icon-{}", field.entry)),
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
                        BorderRadius::default(),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["in-text-container".to_string()]),
                        Pickable::IGNORE,
                        OriginalWidth(-1.),
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
                                BorderRadius::default(),
                                ZIndex::default(),
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
                                ZIndex::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["input-text".to_string()]),
                                Pickable::IGNORE,
                                RenderLayers::layer(layer),
                                InputFieldText,
                                BindToID(id.0),
                            )
                        ],
                    ))
                    .insert(ImageNode::default());
            })
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
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
    mut text_query: Query<(&mut Text, &BindToID), With<InputFieldText>>,
) {
    cursor_blink_timer.timer.tick(time.delta());

    // Build a compact lookup map by UI id to avoid nested loops.
    #[derive(Clone)]
    struct FieldView {
        focused: bool,
        input_type: InputType,
        text: String,
        placeholder: String,
    }

    let mut fields: HashMap<usize, FieldView> = HashMap::new();
    for (field, state, ui_id) in input_field_query.iter() {
        fields.insert(
            ui_id.0,
            FieldView {
                focused: state.focused,
                input_type: field.input_type.clone(),
                text: field.text.clone(),
                placeholder: field.placeholder.clone(),
            },
        );
    }

    for (mut visibility, mut background, mut styles, bind_cursor_id) in cursor_query.iter_mut() {
        let Some(field) = fields.get(&bind_cursor_id.0) else {
            continue;
        };

        if field.focused {
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

                for (mut text, bind_id) in text_query.iter_mut() {
                    if bind_id.0 != bind_cursor_id.0 {
                        continue;
                    }

                    let shown = if field.input_type == InputType::Password {
                        if field.text.is_empty() {
                            field.placeholder.clone()
                        } else {
                            "*".repeat(field.text.chars().count())
                        }
                    } else if field.text.is_empty() {
                        field.placeholder.clone()
                    } else {
                        field.text.clone()
                    };

                    text.0 = shown;
                }
            }
        } else {
            if !matches!(*visibility, Visibility::Hidden) {
                *visibility = Visibility::Hidden;

                for (mut text, bind_id) in text_query.iter_mut() {
                    if bind_id.0 != bind_cursor_id.0 {
                        continue;
                    }
                    if field.text.is_empty() {
                        text.0.clear();
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
fn update_cursor_position(
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut cursor_query: Query<(&mut Node, &mut UiStyle, &BindToID), With<InputCursor>>,
    mut text_field_query: Query<
        (&mut InputField, &UIGenID),
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
        let Some((mut text_field, _ui_id)) = text_field_query
            .iter_mut()
            .find(|(_, ui_id)| ui_id.0 == bind_id.0)
        else {
            continue;
        };

        // Arrow left
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            text_field.cursor_position = text_field.cursor_position.saturating_sub(1);
            key_repeat.timers.insert(
                KeyCode::ArrowLeft,
                Timer::from_seconds(initial_delay, TimerMode::Once),
            );
        }

        // Arrow right
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            text_field.cursor_position =
                (text_field.cursor_position + 1).min(text_field.text.len());
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
                            }
                            KeyCode::ArrowRight => {
                                text_field.cursor_position =
                                    (text_field.cursor_position + 1).min(text_field.text.len());
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
        if !state.focused {
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
/// - Handles Enter key to lose focus and optionally clear input field text.
/// - Updates the visible text and cursor position accordingly.
/// - Masks input with `*` characters if an input type is `Password`.
/// - Updates text color on changes.
fn handle_typing(
    time: Res<Time>,
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut query: Query<(&mut InputField, &mut UIWidgetState, &UiStyle, &UIGenID)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut text_query: Query<
        (
            &mut Text,
            &mut TextColor,
            &UiStyle,
            &ComputedNode,
            &BindToID,
        ),
        (With<InputFieldText>, With<BindToID>),
    >,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    // Cache pressed keys to avoid repeated iterator calls.
    let pressed: Vec<KeyCode> = keyboard.get_pressed().copied().collect();

    for (mut in_field, mut state, style, ui_id) in query.iter_mut() {
        if !state.focused {
            continue;
        }

        for (mut text, mut text_color, widget_style, computed_node, bind_id) in
            text_query.iter_mut()
        {
            if bind_id.0 != ui_id.0 {
                continue;
            }

            // Enter: lose focus and optionally clear text.
            if keyboard.just_pressed(KeyCode::Enter) {
                state.focused = false;
                if in_field.clear_after_focus_lost {
                    in_field.text.clear();
                    text.0 = in_field.text.clone();
                }
                continue;
            }

            // Backspace (single press).
            if keyboard.just_pressed(KeyCode::Backspace) {
                if in_field.cursor_position > 0 && !in_field.text.is_empty() {
                    let remove_at = in_field.cursor_position - 1;
                    in_field.cursor_position = remove_at;
                    in_field.text.remove(remove_at);

                    set_visible_text(&in_field, &mut text);
                }

                if in_field.text.is_empty() {
                    text_color.0 = get_active_text_color(widget_style);
                    text.0 = in_field.placeholder.clone();
                }

                key_repeat.timers.insert(
                    KeyCode::Backspace,
                    Timer::from_seconds(initial_delay, TimerMode::Once),
                );
                continue;
            }

            // Character insertion + repeat.
            for key in &pressed {
                let Some(ch) = keycode_to_char(*key, shift, alt) else {
                    continue;
                };

                if !in_field.input_type.is_valid_char(ch) {
                    // Do not abort the entire system; just skip this character.
                    continue;
                }

                if keyboard.just_pressed(*key) {
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

                    set_visible_text(&in_field, &mut text);

                    text_color.0 = get_active_text_color(widget_style);
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

                        set_visible_text(&in_field, &mut text);

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
                        if in_field.cursor_position > 0 && !in_field.text.is_empty() {
                            let remove_at = in_field.cursor_position - 1;
                            in_field.cursor_position = remove_at;
                            in_field.text.remove(remove_at);

                            set_visible_text(&in_field, &mut text);

                            timer.set_duration(Duration::from_secs_f32(repeat_rate));
                            timer.reset();
                        }
                    }
                }
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

            if let Some(w) = icon_widths.get(&gen_id.0).copied() {
                let expected_left = 5.0 + w;
                let left_now = match node.left {
                    Val::Px(px) => px,
                    _ => 10.,
                };

                if (left_now - expected_left).abs() > f32::EPSILON {
                    node.left = Val::Px(expected_left);
                }
            }

            for (_, style) in styles.styles.iter_mut() {
                style.normal.top = Some(node.top);
                style.normal.left = Some(node.left);
                style.normal.font_size = Some(FontVal::Px(text_font.font_size));
            }
        }
    }
}

// ===============================================
//             Internal Helper Functions
// ===============================================

/// Calculates the horizontal pixel position of the cursor within the input text.
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
fn calculate_text_width(text: &str, style: &TextFont) -> f32 {
    text.len() as f32 * style.font_size * 0.6
}

/// Retrieves the active text color from a widget style, falling back to white.
fn get_active_text_color(style: &UiStyle) -> Color {
    style
        .active_style
        .as_ref()
        .and_then(|s| s.color)
        .unwrap_or(Color::WHITE)
}

fn set_visible_text(in_field: &InputField, out: &mut Text) {
    if in_field.input_type == InputType::Password {
        out.0 = "*".repeat(in_field.text.chars().count());
    } else if in_field.text.is_empty() {
        out.0 = in_field.placeholder.clone();
    } else {
        out.0 = in_field.text.clone();
    }
}

// ===============================================
//                   Internal Events
// ===============================================

/// Handles click events on input fields.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<InputField>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.event_target()) {
        if !state.disabled {
            state.focused = true;
            current_widget_state.widget_id = gen_id.0;
        }
    }

    trigger.propagate(false);
}

/// Handles pointer cursor entering input fields.
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
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<InputField>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.event_target()) {
        state.hovered = false;
    }

    trigger.propagate(false);
}
