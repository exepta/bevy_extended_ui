use std::time::Duration;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use crate::global::{UiGenID, UiElementState};
use crate::styles::{BaseStyle, HoverStyle, SelectedStyle, InternalStyle, Style};
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::css_types::Background;
use crate::utils::Radius;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, BaseStyle, HoverStyle, SelectedStyle, InternalStyle)]
pub struct InputField {
    pub text: String,
    pub placeholder_text: String,
    pub cap_text_at: InputCap,
    pub cursor_position: usize,
    pub input_type: InputType,
    pub clear_after_focus_lost: bool,
    pub icon: Option<Handle<Image>>,
}

impl Default for InputField {
    fn default() -> Self {
        Self {
            text: String::from(""),
            placeholder_text: String::from(""),
            cap_text_at: InputCap::default(),
            input_type: InputType::default(),
            cursor_position: 0,
            clear_after_focus_lost: false,
            icon: None,
        }
    }
}

impl InputField {
    pub fn new(text: &str, placeholder_text: &str, input_type: InputType) -> Self {
        Self {
            text: text.to_string(),
            placeholder_text: placeholder_text.to_string(),
            input_type,
            ..default()
        }
    }
}

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
    CapAt(usize), // 0 means nothing to print!
}

#[derive(Component)]
struct InputFieldRoot;

#[derive(Component)]
struct InputFieldText;

#[derive(Component)]
struct InputFieldIcon;

#[derive(Component)]
struct InputCursor;

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
            timer: Timer::from_seconds(0.85, TimerMode::Repeating)
        }
    }
}

pub struct InputWidget;

impl Plugin for InputWidget {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyRepeatTimers::default());
        app.insert_resource(CursorBlinkTimer::default());
        app.register_type::<InputField>();
        app.add_systems(Update, (
            internal_generate_component_system,
            update_cursor_visibility,
            update_cursor_position,
            handle_typing
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &InputField, Option<&BaseStyle>), (Without<InputFieldRoot>, With<InputField>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity , gen_id, in_field, option_base_style) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("InputField-{}", gen_id.0)),
            Node::default(),
            default_style(option_base_style),
            RenderLayers::layer(*layer),
            InputFieldRoot,
        )).with_children(|builder| {

            if let Some(icon) = in_field.icon.clone() {
                builder.spawn((
                    Name::new(format!("InputIcon-{}", gen_id.0)),
                    Node {
                        width: Val::Px(32.0),
                        min_width: Val::Px(25.0),
                        height: Val::Percent(100.0),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    RenderLayers::layer(*layer),
                    PickingBehavior::IGNORE,
                    InputFieldIcon,
                )).with_children(|builder| {
                    builder.spawn((
                        Name::new(format!("InputIconNode-{}", gen_id.0)),
                        ImageNode {
                            image: icon,
                            color: Color::BLACK,
                            ..default()
                        },
                        RenderLayers::layer(*layer),
                        PickingBehavior::IGNORE,
                    ));
                });
            }

            builder.spawn((
                Name::new(format!("InputCursor-{}", gen_id.0)),
                Node {
                    width: Val::Px(1.5),
                    height: Val::Px(18.0),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                RenderLayers::layer(*layer),
                Visibility::Hidden,
                PickingBehavior::IGNORE,
                InputCursor,
            ));

            let mut text = in_field.placeholder_text.clone();
            if text.is_empty() {
                text = in_field.text.clone();
            }
            builder.spawn((
                Name::new(format!("Input-Text-{}", gen_id.0)),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                Text::new(text),
                PickingBehavior::IGNORE,
                RenderLayers::layer(*layer),
                InputFieldText
            ));

        })
            .observe(on_internal_mouse_click)
            .observe(on_internal_mouse_entered)
            .observe(on_internal_mouse_leave);
    }
}

fn update_cursor_visibility(
    time: Res<Time>,
    mut cursor_blink_timer: ResMut<CursorBlinkTimer>,
    mut cursor_query: Query<(&mut Visibility, &mut BackgroundColor, &Parent), With<InputCursor>>,
    mut input_field_query: Query<(&InputField, &mut UiElementState, &Children), With<InputFieldRoot>>, // Assuming Focus component indicates if field is focused
    mut text_query: Query<&mut Text, With<InputFieldText>>,
) {
    cursor_blink_timer.timer.tick(time.delta());

    for (mut visibility, mut background, parent) in cursor_query.iter_mut() {
        if let Ok((in_field, state, children)) = input_field_query.get_mut(parent.get()) {
            // Show the cursor if the input field is focused
            if state.selected {
                let alpha = (cursor_blink_timer.timer.elapsed_secs() * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;
                background.0.set_alpha(alpha);

                if !visibility.eq(&Visibility::Visible) {

                    *visibility = Visibility::Visible;
                    for child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(*child) {
                            if in_field.input_type.eq(&InputType::Password) {
                                let masked_text: String = "*".repeat(in_field.text.chars().count());
                                text.0 = masked_text;
                            } else {
                                text.0 = in_field.text.clone();
                            }
                        }
                    }
                }
            } else {
                if !visibility.eq(&Visibility::Hidden) {
                    *visibility = Visibility::Hidden;
                    for child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(*child) {
                            if in_field.text.is_empty() {
                                text.0 = in_field.placeholder_text.clone();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_cursor_position(
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut cursor_query: Query<(&mut Node, &Parent), With<InputCursor>>,
    mut text_field_query: Query<(&mut InputField, &InternalStyle, &Parent)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    for (mut cursor_node, parent) in cursor_query.iter_mut() {
        if let Ok((mut text_field, internal_style, _)) = text_field_query.get_mut(parent.get()) {
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

            let cursor_x_position = calculate_cursor_x_position(&text_field, text_field.cursor_position, &internal_style.0);
            cursor_node.left = Val::Px(cursor_x_position);
        }
    }

    key_repeat
        .timers
        .retain(|key, _| keyboard_input.pressed(*key));
}

fn handle_typing(
    time: Res<Time>,
    mut key_repeat: ResMut<KeyRepeatTimers>,
    mut query: Query<(&mut InputField, &mut UiElementState, &InternalStyle, &Children)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<InputFieldText>>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    let initial_delay = 0.3;
    let repeat_rate = 0.07;

    for (mut in_field, mut state, style, children) in query.iter_mut() {
        if state.selected {
            for child in children.iter() {
                if let Ok((mut text, mut text_color)) = text_query.get_mut(*child) {
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
                            text_color.0 = style.0.placeholder_color;
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

                                if in_field.input_type.eq(&InputType::Password) {
                                    in_field.text.insert(pos, char);
                                    in_field.cursor_position += 1;
                                    text.0.insert(pos, '*');
                                } else {
                                    in_field.text.insert(pos, char);
                                    in_field.cursor_position += 1;
                                    text.0 = in_field.text.clone();
                                }
                                text_color.0 = style.0.color;
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

fn calculate_cursor_x_position(text_field: &InputField, cursor_pos: usize, style: &Style) -> f32 {
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

fn calculate_text_width(text: &str, style: &Style) -> f32 {
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

fn default_style(overwrite: Option<&BaseStyle>) -> InternalStyle {
    let mut internal_style = InternalStyle(Style {
        width: Val::Px(250.),
        height: Val::Px(50.),
        display: Display::Flex,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        background: Background { color: Color::srgba(1.0, 1.0, 1.0, 1.0), ..default() },
        border: UiRect::all(Val::Px(2.)),
        border_radius: Radius::all(Val::Px(5.)),
        padding: UiRect::all(Val::Px(5.)),
        ..default()
    });

    if let Some(style) = overwrite {
        internal_style.merge_styles(&style.0);
    }
    internal_style
}