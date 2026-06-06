use bevy::prelude::*;
use serde_json::{Number as JsonNumber, Value as JsonValue};

#[cfg(feature = "extended-framework")]
use crate::framework::UiBindingStore;
use crate::lang::UiSharedValues;
use crate::widgets::{
    CheckBox, ChoiceBox, ChoiceOption, ColorPicker, DatePicker, FieldSelectionMulti,
    FieldSelectionSingle, InputValue, ListBox, ProgressBar, RadioButton, Slider, SwitchButton,
    ToggleButton, WidgetValue,
};

/// Compiled inline functions attached directly to HTML event attributes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HtmlInlineEventBindings {
    pub onclick: Option<HtmlInlineAction>,
    pub onmousedown: Option<HtmlInlineAction>,
    pub onmouseup: Option<HtmlInlineAction>,
    pub onmouseover: Option<HtmlInlineAction>,
    pub onmouseout: Option<HtmlInlineAction>,
    pub onchange: Option<HtmlInlineAction>,
    pub oninit: Option<HtmlInlineAction>,
    pub onfoucs: Option<HtmlInlineAction>,
    pub onscroll: Option<HtmlInlineAction>,
    pub onwheel: Option<HtmlInlineAction>,
    pub onkeydown: Option<HtmlInlineAction>,
    pub onkeyup: Option<HtmlInlineAction>,
    pub ondragstart: Option<HtmlInlineAction>,
    pub ondrag: Option<HtmlInlineAction>,
    pub ondragstop: Option<HtmlInlineAction>,
    pub ontouchstart: Option<HtmlInlineAction>,
    pub ontouchmove: Option<HtmlInlineAction>,
    pub ontouchend: Option<HtmlInlineAction>,
}

/// A semicolon-separated inline action list.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlInlineAction {
    calls: Vec<HtmlInlineCall>,
}

impl HtmlInlineAction {
    pub fn calls(&self) -> &[HtmlInlineCall] {
        &self.calls
    }
}

/// Supported inline function names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInlineFunction {
    Set,
    Add,
    Min,
}

/// One parsed inline function call.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlInlineCall {
    pub function: HtmlInlineFunction,
    pub target: HtmlInlinePath,
    pub value: HtmlInlineExpr,
}

/// Dot-separated target or source path, e.g. `info.value`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInlinePath {
    segments: Vec<String>,
}

impl HtmlInlinePath {
    pub fn new(segments: Vec<String>) -> Option<Self> {
        if segments.is_empty() || segments.iter().any(|segment| segment.trim().is_empty()) {
            None
        } else {
            Some(Self { segments })
        }
    }

    pub fn root(&self) -> &str {
        &self.segments[0]
    }

    pub fn tail(&self) -> &[String] {
        &self.segments[1..]
    }

    pub fn as_dotted(&self) -> String {
        self.segments.join(".")
    }
}

/// Value expression used as function argument.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlInlineExpr {
    Event(HtmlInlinePath),
    Path(HtmlInlinePath),
    Literal(JsonValue),
}

/// Parses an HTML inline action such as `$set(info.value, $event.value)`.
pub fn parse_html_inline_action(raw: &str) -> Result<HtmlInlineAction, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("inline HTML function is empty".to_string());
    }

    let mut calls = Vec::new();
    for part in split_top_level(raw, ';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        calls.push(parse_inline_call(part)?);
    }

    if calls.is_empty() {
        return Err("inline HTML function has no calls".to_string());
    }

    Ok(HtmlInlineAction { calls })
}

/// Queues an inline action for execution with direct `World` access.
pub(crate) fn queue_html_inline_action(
    commands: &mut Commands,
    entity: Entity,
    action: HtmlInlineAction,
) {
    commands.queue(move |world: &mut World| {
        execute_html_inline_action(world, entity, &action);
    });
}

fn execute_html_inline_action(world: &mut World, entity: Entity, action: &HtmlInlineAction) {
    for call in action.calls() {
        let Some(value) = resolve_expr(world, entity, &call.value) else {
            warn!(
                "Failed to resolve inline HTML value for target '{}'",
                call.target.as_dotted()
            );
            continue;
        };

        let next = match call.function {
            HtmlInlineFunction::Set => value,
            HtmlInlineFunction::Add | HtmlInlineFunction::Min => {
                let Some(current) = read_store_path(world, &call.target) else {
                    warn!(
                        "Cannot apply inline HTML arithmetic to unknown target '{}'",
                        call.target.as_dotted()
                    );
                    continue;
                };
                let factor = if call.function == HtmlInlineFunction::Add {
                    1.0
                } else {
                    -1.0
                };
                match arithmetic_value(&current, &value, factor) {
                    Some(value) => value,
                    None => {
                        warn!(
                            "Cannot apply inline HTML arithmetic to non-numeric target '{}'",
                            call.target.as_dotted()
                        );
                        continue;
                    }
                }
            }
        };

        if !write_store_path(world, &call.target, next) {
            warn!(
                "Inline HTML function target '{}' could not be written",
                call.target.as_dotted()
            );
        }
    }
}

fn parse_inline_call(raw: &str) -> Result<HtmlInlineCall, String> {
    let raw = raw.trim();
    let Some(rest) = raw.strip_prefix('$') else {
        return Err(format!("inline call '{raw}' must start with '$'"));
    };
    let Some(open) = rest.find('(') else {
        return Err(format!("inline call '{raw}' is missing '('"));
    };
    if !rest.ends_with(')') {
        return Err(format!("inline call '{raw}' is missing ')'"));
    }

    let name = rest[..open].trim();
    let function = match name {
        "set" => HtmlInlineFunction::Set,
        "add" => HtmlInlineFunction::Add,
        "min" => HtmlInlineFunction::Min,
        _ => return Err(format!("unknown inline HTML function '${name}'")),
    };

    let args = split_top_level(&rest[open + 1..rest.len() - 1], ',');
    if args.len() != 2 {
        return Err(format!(
            "inline function '${name}' expects exactly 2 arguments"
        ));
    }

    let target = parse_path(args[0].trim())
        .ok_or_else(|| format!("invalid inline function target '{}'", args[0].trim()))?;
    let value = parse_expr(args[1].trim())?;

    Ok(HtmlInlineCall {
        function,
        target,
        value,
    })
}

fn parse_expr(raw: &str) -> Result<HtmlInlineExpr, String> {
    let raw = raw.trim();
    if let Some(event_path) = raw.strip_prefix("$event") {
        let event_path = event_path.strip_prefix('.').unwrap_or("value");
        let path = parse_path(event_path)
            .ok_or_else(|| format!("invalid $event path '{}'", raw.trim()))?;
        return Ok(HtmlInlineExpr::Event(path));
    }

    if let Some(value) = parse_literal(raw) {
        return Ok(HtmlInlineExpr::Literal(value));
    }

    let path = parse_path(raw).ok_or_else(|| format!("invalid inline expression '{raw}'"))?;
    Ok(HtmlInlineExpr::Path(path))
}

fn parse_literal(raw: &str) -> Option<JsonValue> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("true") {
        return Some(JsonValue::Bool(true));
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Some(JsonValue::Bool(false));
    }
    if trimmed.eq_ignore_ascii_case("null") {
        return Some(JsonValue::Null);
    }
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return Some(JsonValue::String(unquote(trimmed)));
    }
    if let Ok(value) = serde_json::from_str::<JsonValue>(trimmed) {
        if matches!(
            value,
            JsonValue::Number(_) | JsonValue::Array(_) | JsonValue::Object(_)
        ) {
            return Some(value);
        }
    }
    None
}

fn parse_path(raw: &str) -> Option<HtmlInlinePath> {
    let mut segments = Vec::new();
    for segment in raw.split('.') {
        let segment = segment.trim();
        if !is_identifier(segment) {
            return None;
        }
        segments.push(segment.to_string());
    }
    HtmlInlinePath::new(segments)
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn split_top_level(raw: &str, delimiter: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in raw.chars() {
        if let Some(active_quote) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            _ if ch == delimiter && depth == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    result.push(current.trim().to_string());
    result
}

fn unquote(raw: &str) -> String {
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::new();
    let mut escaped = false;
    for ch in inner.chars() {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    out
}

fn resolve_expr(world: &World, entity: Entity, expr: &HtmlInlineExpr) -> Option<JsonValue> {
    match expr {
        HtmlInlineExpr::Event(path) => event_path_value(world, entity, path),
        HtmlInlineExpr::Path(path) => read_store_path(world, path),
        HtmlInlineExpr::Literal(value) => Some(value.clone()),
    }
}

fn event_path_value(world: &World, entity: Entity, path: &HtmlInlinePath) -> Option<JsonValue> {
    match path.root() {
        "value" => widget_value(world, entity),
        "checked" | "selected" => widget_checked(world, entity),
        "text" => widget_text(world, entity),
        "red" | "green" | "blue" | "alpha" => color_channel_value(world, entity, path.root()),
        "rgb" => world
            .get::<ColorPicker>(entity)
            .map(|picker| JsonValue::String(picker.rgb_string())),
        "rgba" => world
            .get::<ColorPicker>(entity)
            .map(|picker| JsonValue::String(picker.rgba_string())),
        "hex" => world
            .get::<ColorPicker>(entity)
            .map(|picker| JsonValue::String(picker.hex())),
        _ => None,
    }
}

fn widget_value(world: &World, entity: Entity) -> Option<JsonValue> {
    if let Some(input) = world.get::<InputValue>(entity) {
        return Some(JsonValue::String(input.0.clone()));
    }
    if let Some(checkbox) = world.get::<CheckBox>(entity) {
        return Some(JsonValue::Bool(checkbox.checked));
    }
    if let Some(slider) = world.get::<Slider>(entity) {
        return number_json(slider.value as f64);
    }
    if let Some(progress) = world.get::<ProgressBar>(entity) {
        return number_json(progress.value as f64);
    }
    if let Some(picker) = world.get::<ColorPicker>(entity) {
        return Some(JsonValue::String(picker.hex()));
    }
    if let Some(choice) = world.get::<ChoiceBox>(entity) {
        return widget_value_to_json(&choice.value.value)
            .or_else(|| Some(JsonValue::String(choice.value.text.clone())));
    }
    if let Some(list) = world.get::<ListBox>(entity) {
        let values = list
            .values
            .iter()
            .map(choice_option_to_json)
            .collect::<Vec<_>>();
        return Some(JsonValue::Array(values));
    }
    if let Some(radio) = world.get::<RadioButton>(entity) {
        return widget_value_to_json(&radio.value)
            .or_else(|| Some(JsonValue::String(radio.label.clone())));
    }
    if let Some(toggle) = world.get::<ToggleButton>(entity) {
        return widget_value_to_json(&toggle.value)
            .or_else(|| Some(JsonValue::Bool(toggle.selected)));
    }
    if let Some(switch) = world.get::<SwitchButton>(entity) {
        return Some(JsonValue::Bool(switch.selected));
    }
    if let Some(_picker) = world.get::<DatePicker>(entity) {
        if let Some(value) = world.get::<InputValue>(entity) {
            return Some(JsonValue::String(value.0.clone()));
        }
    }

    fieldset_value(world, entity)
}

fn widget_checked(world: &World, entity: Entity) -> Option<JsonValue> {
    if let Some(checkbox) = world.get::<CheckBox>(entity) {
        return Some(JsonValue::Bool(checkbox.checked));
    }
    if let Some(radio) = world.get::<RadioButton>(entity) {
        return Some(JsonValue::Bool(radio.selected));
    }
    if let Some(toggle) = world.get::<ToggleButton>(entity) {
        return Some(JsonValue::Bool(toggle.selected));
    }
    if let Some(switch) = world.get::<SwitchButton>(entity) {
        return Some(JsonValue::Bool(switch.selected));
    }
    None
}

fn widget_text(world: &World, entity: Entity) -> Option<JsonValue> {
    if let Some(choice) = world.get::<ChoiceBox>(entity) {
        return Some(JsonValue::String(choice.value.text.clone()));
    }
    if let Some(radio) = world.get::<RadioButton>(entity) {
        return Some(JsonValue::String(radio.label.clone()));
    }
    if let Some(toggle) = world.get::<ToggleButton>(entity) {
        return Some(JsonValue::String(toggle.label.clone()));
    }
    if let Some(switch) = world.get::<SwitchButton>(entity) {
        return Some(JsonValue::String(switch.label.clone()));
    }
    None
}

fn color_channel_value(world: &World, entity: Entity, channel: &str) -> Option<JsonValue> {
    let picker = world.get::<ColorPicker>(entity)?;
    let value = match channel {
        "red" => picker.red,
        "green" => picker.green,
        "blue" => picker.blue,
        "alpha" => picker.alpha,
        _ => return None,
    };
    Some(JsonValue::Number(JsonNumber::from(value)))
}

fn fieldset_value(world: &World, entity: Entity) -> Option<JsonValue> {
    if let Some(selection) = world.get::<FieldSelectionSingle>(entity) {
        let selected = selection.0?;
        return widget_value(world, selected);
    }

    if let Some(selection) = world.get::<FieldSelectionMulti>(entity) {
        let values = selection
            .0
            .iter()
            .filter_map(|selected| widget_value(world, *selected))
            .collect::<Vec<_>>();
        return Some(JsonValue::Array(values));
    }

    None
}

fn choice_option_to_json(option: &ChoiceOption) -> JsonValue {
    widget_value_to_json(&option.value).unwrap_or_else(|| JsonValue::String(option.text.clone()))
}

fn widget_value_to_json(value: &WidgetValue) -> Option<JsonValue> {
    macro_rules! downcast_number {
        ($ty:ty) => {
            if let Some(value) = value.get::<$ty>() {
                return serde_json::to_value(*value).ok();
            }
        };
    }

    if let Some(value) = value.get::<String>() {
        return Some(JsonValue::String(value.clone()));
    }
    if let Some(value) = value.get::<bool>() {
        return Some(JsonValue::Bool(*value));
    }
    downcast_number!(u8);
    downcast_number!(u16);
    downcast_number!(u32);
    downcast_number!(u64);
    downcast_number!(u128);
    downcast_number!(usize);
    downcast_number!(i8);
    downcast_number!(i16);
    downcast_number!(i32);
    downcast_number!(i64);
    downcast_number!(i128);
    downcast_number!(isize);
    downcast_number!(f32);
    downcast_number!(f64);
    value.get::<JsonValue>().cloned()
}

fn read_store_path(world: &World, path: &HtmlInlinePath) -> Option<JsonValue> {
    #[cfg(feature = "extended-framework")]
    if let Some(store) = world.get_resource::<UiBindingStore>() {
        if let Some(value) = store.json_path(&path.as_dotted()) {
            return Some(value);
        }
    }

    let shared = world.get_resource::<UiSharedValues>()?;
    let value = shared.values.get(path.root())?.clone();
    resolve_json_path(value, path.tail())
}

fn write_store_path(world: &mut World, path: &HtmlInlinePath, value: JsonValue) -> bool {
    #[cfg(feature = "extended-framework")]
    if let Some(mut store) = world.get_resource_mut::<UiBindingStore>() {
        return store.set_path_json(&path.as_dotted(), value);
    }

    #[cfg(not(feature = "extended-framework"))]
    let _ = world;
    warn!(
        "Inline HTML target '{}' requires the 'extended-framework' feature",
        path.as_dotted()
    );
    let _ = value;
    false
}

fn resolve_json_path(mut value: JsonValue, tail: &[String]) -> Option<JsonValue> {
    for segment in tail {
        match value {
            JsonValue::Object(map) => {
                value = map.get(segment)?.clone();
            }
            JsonValue::Array(items) => {
                let index = segment.parse::<usize>().ok()?;
                value = items.get(index)?.clone();
            }
            _ => return None,
        }
    }
    Some(value)
}

fn arithmetic_value(current: &JsonValue, delta: &JsonValue, factor: f64) -> Option<JsonValue> {
    let left = json_to_f64(current)?;
    let right = json_to_f64(delta)? * factor;
    number_json(left + right)
}

fn json_to_f64(value: &JsonValue) -> Option<f64> {
    match value {
        JsonValue::Number(number) => number.as_f64(),
        JsonValue::String(value) => value.trim().parse::<f64>().ok(),
        JsonValue::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn number_json(value: f64) -> Option<JsonValue> {
    JsonNumber::from_f64(value).map(JsonValue::Number)
}
