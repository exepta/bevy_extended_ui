use std::time::Duration;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use crate::global::{UiGenID, UiElementState, BindToID};
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::{LabelStyle, Style};
use crate::styles::state_styles::{Base, Disabled, Hover, Selected, Styling};
use crate::styles::types::InputStyle;
use crate::styles::utils::{apply_base_component_style, apply_design_styles, apply_label_styles_to_child, resolve_style_by_state};
use crate::widgets::InputField;

#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Number
}

impl InputType {
    pub fn is_valid_char(&self, c: char) -> bool {
        match self {
            InputType::Text | InputType::Password => true,
            InputType::Number => c.is_ascii_digit() || "+-*/() ".contains(c),
        }
    }
}

#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum InputCap {
    #[default]
    NoCap,
    CapAtNodeSize,
    CapAt(usize), // 0 means no cap!
}

impl InputCap {
    pub fn get_value(&self) -> usize {
        match self {
            Self::CapAt(value) => *value,
            Self::NoCap => 0,
            Self::CapAtNodeSize => 0
        }
    }
}

#[derive(Component)]
struct InputFieldRoot;

#[derive(Component)]
struct InputFieldText;

#[derive(Component)]
struct InputFieldIcon;

#[derive(Component)]
struct InputCursor;

#[derive(Component, Clone)]
pub struct CursorColor(pub Color);

#[derive(Component)]
struct InputContainer;

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
            internal_generate_component_system,
            internal_style_update_que
                .after(internal_generate_component_system),
            update_cursor_visibility,
            update_cursor_position,
            handle_typing,
            handle_input_horizontal_scroll
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &InputField), (Without<InputFieldRoot>, With<InputField>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    let default_input_style = InputStyle::default();
    for (entity , gen_id, in_field) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("InputField-{}", gen_id.0)),
            Node::default(),
            BorderRadius::default(),
            BorderColor::default(),
            BackgroundColor::default(),
            BoxShadow::default(),
            Base(Styling::InputField(default_input_style.clone())),
            Selected(Styling::InputField(InputStyle {
                style: Style {
                    border_color: Color::srgb_u8(111, 162, 205),
                    ..default_input_style.style.clone()
                },
                ..default_input_style.clone()
            })),
            RenderLayers::layer(*layer),
            InputFieldRoot,
        )).with_children(|builder| {

            if let Some(icon) = in_field.icon.clone() {
                builder.spawn((
                    Name::new(format!("InputIcon-{}", gen_id.0)),
                    Node {
                        width: Val::Px(40.0),
                        min_width: Val::Px(40.0),
                        height: Val::Percent(100.0),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor::default(),
                    BorderRadius::default(),
                    RenderLayers::layer(*layer),
                    PickingBehavior::IGNORE,
                    InputFieldIcon,
                    BindToID(gen_id.0),
                )).with_children(|builder| {
                    builder.spawn((
                        Name::new(format!("InputIconNode-{}", gen_id.0)),
                        ImageNode {
                            image: icon,
                            ..default()
                        },
                        RenderLayers::layer(*layer),
                        PickingBehavior::IGNORE,
                        BindToID(gen_id.0),
                    ));
                });
            }

            builder.spawn((
                Name::new(format!("TextContainer-{}", gen_id.0)),
                Node {
                    height: Val::Percent(100.),
                    width: if in_field.icon.is_some() { Val::Percent(80.) } else { Val::Percent(95.) },
                    max_width: if in_field.icon.is_some() { Val::Percent(80.) } else { Val::Percent(95.) },
                    display: Display::Flex,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    overflow: Overflow {
                        y: OverflowAxis::Hidden,
                        x: OverflowAxis::Scroll,
                    },
                    padding: if in_field.icon.is_some() { UiRect::left(Val::Px(0.0)) } else { UiRect::left(Val::Px(10.0)) },
                    ..default()
                },
                RenderLayers::layer(*layer),
                PickingBehavior::IGNORE,
                BindToID(gen_id.0),
                InputContainer
            )).with_children(|builder| {

                builder.spawn((
                    Name::new(format!("InputCursor-{}", gen_id.0)),
                    Node {
                        width: Val::Px(1.5),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    RenderLayers::layer(*layer),
                    Visibility::Hidden,
                    PickingBehavior::IGNORE,
                    InputCursor,
                    BindToID(gen_id.0),
                ));

                let mut text = in_field.placeholder_text.clone();
                if text.is_empty() {
                    text = in_field.text.clone();
                }
                builder.spawn((
                    Name::new(format!("Input-Text-{}", gen_id.0)),
                    Node {
                        width: Val::Percent(90.0),
                        ..default()
                    },
                    Text::new(text),
                    TextColor::default(),
                    TextFont::default(),
                    TextLayout::default(),
                    PickingBehavior::IGNORE,
                    RenderLayers::layer(*layer),
                    InputFieldText,
                    BindToID(gen_id.0),
                ));

            });

        })
            .observe(on_internal_mouse_click)
            .observe(on_internal_mouse_entered)
            .observe(on_internal_mouse_leave);
    }
}

fn update_cursor_visibility(
    time: Res<Time>,
    mut cursor_blink_timer: ResMut<CursorBlinkTimer>,
    mut cursor_query: Query<(&mut Visibility, &mut BackgroundColor, &BindToID), With<InputCursor>>,
    mut input_field_query: Query<(&InputField, &mut UiElementState, &UiGenID), With<InputFieldRoot>>, // Assuming Focus component indicates if field is focused
    mut text_query: Query<(&mut Text, &BindToID), With<InputFieldText>>,
) {
    cursor_blink_timer.timer.tick(time.delta());

    for (mut visibility, mut background, bind_cursor_id) in cursor_query.iter_mut() {
        for (in_field, state, ui_id) in input_field_query.iter_mut() {
            if bind_cursor_id.0 != ui_id.0 {
                continue;
            }
            // Show the cursor if the input field is focused
            if state.selected {
                let alpha = (cursor_blink_timer.timer.elapsed_secs() * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;
                background.0.set_alpha(alpha);

                if !visibility.eq(&Visibility::Visible) {

                    *visibility = Visibility::Visible;
                    for (mut text, bind_id) in text_query.iter_mut() {
                        if bind_id.0 != ui_id.0 {
                            continue;
                        }
                        if in_field.input_type.eq(&InputType::Password) {
                            let masked_text: String = "*".repeat(in_field.text.chars().count());
                            text.0 = masked_text;
                        } else {
                            text.0 = in_field.text.clone();
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
                            text.0 = in_field.placeholder_text.clone();
                        }
                    }
                }
            }
        }
    }
}

fn update_cursor_position(
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut cursor_query: Query<(&mut Node, &BindToID), With<InputCursor>>,
    mut text_field_query: Query<(&mut InputField, &InputStyle, &UiGenID)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    for (mut cursor_node, bind_id) in cursor_query.iter_mut() {
        for (mut text_field, internal_style, ui_id) in text_field_query.iter_mut() {
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

            let cursor_x_position = calculate_cursor_x_position(&text_field, text_field.cursor_position, &internal_style.label_style);
            cursor_node.left = Val::Px(cursor_x_position);
        }
    }

    key_repeat
        .timers
        .retain(|key, _| keyboard_input.pressed(*key));
}

fn handle_input_horizontal_scroll(
    mut query: Query<(
        &InputField,
        &InputStyle,
        &UiGenID
    ), With<InputFieldRoot>>,
    mut scroll_query: Query<(&mut ScrollPosition, &BindToID), With<BindToID>>,
    text_node_query: Query<(&ComputedNode, &BindToID), With<InputFieldText>>
) {
    for (input_field, internal_style, ui_id) in &mut query {
        let char_width = internal_style.label_style.font_size;
        let cursor_x = input_field.cursor_position as f32 * char_width;

        let Some((text_node, _)) = text_node_query
            .iter()
            .find(|(_, bind_id)| bind_id.0 == ui_id.0)
        else {
            continue;
        };

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
    mut query: Query<(&mut InputField, &mut UiElementState, &InputStyle, &UiGenID)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut text_query: Query<(&mut Text, &mut TextColor, &ComputedNode, &BindToID), (With<InputFieldText>, With<BindToID>)>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    for (mut in_field, mut state, style, ui_id) in query.iter_mut() {
        if state.selected {
            for (mut text, mut text_color, computed_node, bind_id) in text_query.iter_mut() {
                if bind_id.0 == ui_id.0 {
                    // ENTER
                    if keyboard.just_pressed(KeyCode::Enter) {
                        state.selected = false;
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
                            text_color.0 = style.placeholder_color;
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
                                    let allowed_char_len = (computed_node.size().x / (style.label_style.font_size)).round() as usize;
                                    if pos >= allowed_char_len {
                                        return;
                                    }
                                }

                                if in_field.input_type.eq(&InputType::Password) {
                                    in_field.text.insert(pos, char);
                                    in_field.cursor_position += 1;
                                    text.0.insert(pos, '*');
                                } else {
                                    in_field.text.insert(pos, char);
                                    in_field.cursor_position += 1;
                                    text.0 = in_field.text.clone();
                                }
                                text_color.0 = style.label_style.color;
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

fn keycode_to_char(key: KeyCode, shift: bool, alt: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else if alt { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else if alt { '@' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::KeyZ => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::Digit0 => Some(if shift { '=' } else if alt { '}' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else if alt { '1' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '"' } else if alt { '2' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '3' } else if alt { '3' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else if alt { '4' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else if alt { '5' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '&' } else if alt { '6' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '/' } else if alt { '{' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '(' } else if alt { '[' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { ')' } else if alt { ']' } else { '9' }),
        KeyCode::NumpadMultiply => Some('*'),
        KeyCode::NumpadAdd => Some('+'),
        KeyCode::NumpadSubtract => Some('-'),
        KeyCode::NumpadDivide => Some('/'),
        KeyCode::NumpadDecimal => Some(','),
        KeyCode::Numpad0 => Some('0'),
        KeyCode::Numpad1 => Some('1'),
        KeyCode::Numpad2 => Some('2'),
        KeyCode::Numpad3 => Some('3'),
        KeyCode::Numpad4 => Some('4'),
        KeyCode::Numpad5 => Some('5'),
        KeyCode::Numpad6 => Some('6'),
        KeyCode::Numpad7 => Some('7'),
        KeyCode::Numpad8 => Some('8'),
        KeyCode::Numpad9 => Some('9'),
        KeyCode::Comma => Some(if shift {';'} else {','}),
        KeyCode::Period => Some(if shift {':'} else {'.'}),
        KeyCode::Slash => Some(if shift {'_'} else {'-'}),
        KeyCode::IntlBackslash => Some(if shift {'>'} else if alt {'|'} else {'<'}),
        KeyCode::Backquote => Some(if shift {'?'} else {'^'}),
        KeyCode::Minus => Some(if shift {'?'} else if alt {'\\'} else {'?'}),
        KeyCode::BracketRight => Some(if shift {'*'} else if alt {'~'} else {'+'}),
        KeyCode::Backslash => Some(if shift {'\''} else {'#'}),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

fn calculate_cursor_x_position(text_field: &InputField, cursor_pos: usize, style: &LabelStyle) -> f32 {
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

fn calculate_text_width(text: &str, style: &LabelStyle) -> f32 {
    // Calculate text width based on font size
    let font_size = style.font_size; // Default font size if none is provided
    text.len() as f32 * font_size * 0.6 // Adjust factor based on font characteristics
}

fn on_internal_mouse_click(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UiElementState, &UiGenID), With<InputField>>,
    mut current_element_selected: ResMut<CurrentElementSelected>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(event.target) {
        state.selected = true;
        current_element_selected.0 = gen_id.0;
    }
}

fn on_internal_mouse_entered(event: Trigger<Pointer<Over>>, mut query: Query<&mut UiElementState, With<InputField>>) {
    if let Ok(mut state) = query.get_mut(event.target) {
        state.hovered = true;
    }
}

fn on_internal_mouse_leave(event: Trigger<Pointer<Out>>, mut query: Query<&mut UiElementState, With<InputField>>) {
    if let Ok(mut state) = query.get_mut(event.target) {
        state.hovered = false;
    }
}

fn internal_style_update_que(
    mut query: Query<(&UiElementState, &UiGenID, &Children, &InputStyle, Option<&Base>, Option<&Hover>, Option<&Selected>, Option<&Disabled>,
                      &mut Node,
                      &mut BackgroundColor,
                      &mut BoxShadow,
                      &mut BorderRadius,
                      &mut BorderColor
    ), (With<InputField>, Without<InputCursor>)>,
    cursor_blink_timer: Res<CursorBlinkTimer>,
    mut label_query: Query<(&BindToID, &mut TextColor, &mut TextFont, &mut TextLayout)>,
    mut icon_container: Query<(&BindToID, &mut BackgroundColor, &mut BorderRadius), (With<InputFieldIcon>, Without<InputField>, Without<InputCursor>)>,
    mut text_cursor_query: Query<(&mut BackgroundColor, &BindToID), With<InputCursor>>,
    container_query: Query<&Children, With<InputContainer>>,
) {
    for (state, ui_id, children, style, base_style, hover_style, selected_style, disabled_style,
        mut node,
        mut background_color,
        mut box_shadow,
        mut border_radius,
        mut border_color) in query.iter_mut() {
        let mut manipulated = style.clone();
        if let Some(Base(style)) = base_style {
            if let Styling::InputField(input_style) = style {
                manipulated = input_style.clone();
            }
        }

        let internal_style = resolve_style_by_state(
            &Styling::InputField(manipulated.clone()),
            state,
            hover_style,
            selected_style,
            disabled_style,
        );

        if let Styling::InputField(input_style) = internal_style {
            apply_base_component_style(&input_style.style, &mut node);
            apply_design_styles(&input_style.style, &mut background_color, &mut border_color, &mut border_radius, &mut box_shadow);

            for child in children.iter() {
                if let Ok((bind_to,
                              mut icon_background,
                              mut icon_border_radius)) =
                    icon_container.get_mut(*child) {
                    if bind_to.0 != ui_id.0 {
                        continue;
                    }

                    icon_background.0 = input_style.style.background.color;
                    icon_border_radius.top_left = input_style.style.border_radius.top_left;
                    icon_border_radius.bottom_left = input_style.style.border_radius.top_left;
                    icon_border_radius.top_right = Val::Px(0.);
                    icon_border_radius.bottom_right = Val::Px(0.);
                }

                if let Ok(children) = container_query.get(*child) {
                    for inner_child in children.iter() {
                        apply_label_styles_to_child(*inner_child, ui_id, &input_style.label_style, &mut label_query);

                        if let Ok((mut cursor_color, bind_to)) = text_cursor_query.get_mut(*inner_child) {
                            if bind_to.0 != ui_id.0 {
                                continue;
                            }

                            if cursor_blink_timer.timer.finished() {
                                cursor_color.0 = input_style.label_style.color;
                            }
                        }
                    }
                }
            }
        }
    }
}