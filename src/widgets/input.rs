use std::collections::HashMap;
use std::time::Duration;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::{Background, FontVal};
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::utils::keycode_to_char;
use crate::widgets::{InputCap, InputField, InputType};

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
            handle_overlay_label
        ));
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &InputField, Option<&CssSource>), (With<InputField>, Without<InputFieldBase>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, field, source_opt) in query.iter() {
        let mut css_source = CssSource(String::from("assets/css/core.css"));
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("Input-{}", field.w_count)),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            css_source.clone(),
            TagName(String::from("input")),
            RenderLayers::layer(*layer),
            InputFieldBase
        )).with_children(|builder| {
            if let Some(icon_path) = field.icon_path.clone() {
                // Icon left
                builder.spawn((
                    Name::new(format!("Input-Icon-{}", field.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
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
                                image: asset_server.load(icon_path),
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
                RenderLayers::layer(*layer),
                InputContainer,
                BindToID(id.0),
                children![
                    // Input Cursor
                    (
                        Name::new(format!("Cursor-{}", field.w_count)),
                        Node::default(),
                        BackgroundColor::default(),
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
            ));
        }).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

// ===============================================
//             Intern Functions
// ===============================================

fn update_cursor_visibility(
    time: Res<Time>,
    mut cursor_blink_timer: ResMut<CursorBlinkTimer>,
    mut cursor_query: Query<(&mut Visibility, &mut BackgroundColor, &mut WidgetStyle, &BindToID), With<InputCursor>>,
    mut input_field_query: Query<(&InputField, &mut UIWidgetState, &UIGenID), With<InputFieldBase>>, // Assuming Focus component indicates if field is focused
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

                if !visibility.eq(&Visibility::Visible) {

                    *visibility = Visibility::Visible;
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
                        if timer.finished() {
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
                        let visible_left = scroll.offset_x;
                        let visible_right = scroll.offset_x + available_width;

                        if cursor_x > visible_right {
                            scroll.offset_x = cursor_x - available_width + char_width;
                        }
                        else if cursor_x < visible_left {
                            scroll.offset_x = cursor_x;
                        }

                        let total_text_width = input_field.text.len() as f32 * char_width;
                        if total_text_width < available_width {
                            scroll.offset_x = 0.0;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

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
                                if timer.finished() {
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
                            if timer.finished() {
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

fn handle_overlay_label(
    query: Query<(&UIWidgetState, &UIGenID, &InputField, &Children), With<InputField>>,
    mut label_query: Query<(&BindToID, &mut Node, &mut TextFont, &mut WidgetStyle), With<OverlayLabel>>,
) {
    for (state, gen_id, in_field, children) in query.iter() {
        for child in children.iter() {
            if let Ok((bind_to, mut node, mut text_font, mut styles)) = label_query.get_mut(child) {
                if bind_to.0 != gen_id.0 {
                    continue;
                }

                if state.focused {
                    node.top = Val::Px(5.);
                    text_font.font_size = 10.;
                } else {
                    if in_field.text.is_empty() {
                        node.top = Val::Px(19.5);
                        text_font.font_size = 13.;
                    }
                }

                for (_, style) in styles.styles.iter_mut() {
                    style.top = Some(node.top);
                    style.font_size = Some(FontVal::Px(text_font.font_size));
                }
            }
        }
    }
}

// ===============================================
//             Intern Helper Functions
// ===============================================

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

fn calculate_text_width(text: &str, style: &TextFont) -> f32 {
    // Calculate text width based on font size
    text.len() as f32 * style.font_size * 0.6 // Adjust factor based on font characteristics
}

fn get_active_text_color(style: &WidgetStyle) -> Color {
    style
        .active_style
        .as_ref()
        .and_then(|s| s.color)
        .unwrap_or(Color::BLACK)
}

// ===============================================
//                   Intern Events
// ===============================================

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<InputField>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<InputField>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<InputField>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}