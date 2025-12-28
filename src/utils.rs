use bevy::prelude::*;

/// Converts a `KeyCode` into a corresponding character, taking into account
/// modifier keys like Shift and Alt.
///
/// This function maps keyboard keys to characters, optionally modifying the output
/// if `shift` or `alt` is pressed. For example, letters will be uppercase if `shift` is true,
/// and certain keys may produce alternative symbols if `alt` is pressed.
///
/// # Parameters
///
/// - `key`: The `KeyCode` representing the key pressed.
/// - `shift`: `true` if the Shift key is held down.
/// - `alt`: `true` if the Alt key is held down.
///
/// # Returns
///
/// - `Some(char)` representing the character corresponding to the key and modifier keys.
/// - `None` if the key does not correspond to a character (e.g., function keys).
///
/// # Examples
///
/// ```
/// use bevy::prelude::KeyCode;
/// use bevy_extended_ui::utils::keycode_to_char;
/// assert_eq!(keycode_to_char(KeyCode::KeyA, false, false), Some('a'));
/// assert_eq!(keycode_to_char(KeyCode::KeyA, true, false), Some('A'));
/// assert_eq!(keycode_to_char(KeyCode::Digit1, true, false), Some('!'));
/// assert_eq!(keycode_to_char(KeyCode::Space, false, false), Some(' '));
/// assert_eq!(keycode_to_char(KeyCode::F1, false, false), None);
/// ```
///
pub fn keycode_to_char(key: KeyCode, shift: bool, alt: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift {
            'E'
        } else if alt {
            'E'
        } else {
            'e'
        }),
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
        KeyCode::KeyQ => Some(if shift {
            'Q'
        } else if alt {
            '@'
        } else {
            'q'
        }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::KeyZ => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::Digit0 => Some(if shift {
            '='
        } else if alt {
            '}'
        } else {
            '0'
        }),
        KeyCode::Digit1 => Some(if shift {
            '!'
        } else if alt {
            '1'
        } else {
            '1'
        }),
        KeyCode::Digit2 => Some(if shift {
            '"'
        } else if alt {
            '2'
        } else {
            '2'
        }),
        KeyCode::Digit3 => Some(if shift {
            '3'
        } else if alt {
            '3'
        } else {
            '3'
        }),
        KeyCode::Digit4 => Some(if shift {
            '$'
        } else if alt {
            '4'
        } else {
            '4'
        }),
        KeyCode::Digit5 => Some(if shift {
            '%'
        } else if alt {
            '5'
        } else {
            '5'
        }),
        KeyCode::Digit6 => Some(if shift {
            '&'
        } else if alt {
            '6'
        } else {
            '6'
        }),
        KeyCode::Digit7 => Some(if shift {
            '/'
        } else if alt {
            '{'
        } else {
            '7'
        }),
        KeyCode::Digit8 => Some(if shift {
            '('
        } else if alt {
            '['
        } else {
            '8'
        }),
        KeyCode::Digit9 => Some(if shift {
            ')'
        } else if alt {
            ']'
        } else {
            '9'
        }),
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
        KeyCode::Comma => Some(if shift { ';' } else { ',' }),
        KeyCode::Period => Some(if shift { ':' } else { '.' }),
        KeyCode::Slash => Some(if shift { '_' } else { '-' }),
        KeyCode::IntlBackslash => Some(if shift {
            '>'
        } else if alt {
            '|'
        } else {
            '<'
        }),
        KeyCode::Backquote => Some(if shift { '?' } else { '^' }),
        KeyCode::Minus => Some(if shift {
            '?'
        } else if alt {
            '\\'
        } else {
            '?'
        }),
        KeyCode::BracketRight => Some(if shift {
            '*'
        } else if alt {
            '~'
        } else {
            '+'
        }),
        KeyCode::Backslash => Some(if shift { '\'' } else { '#' }),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}
