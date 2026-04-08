/// Example demonstrating typed values on ChoiceBox and RadioButton widgets.
///
/// Shows how integer, boolean, enum-like string, and struct/f32 values are
/// stored inside `Arc<dyn Any + Send + Sync>` and recovered at runtime via
/// `get_value::<T>()` / `value_as_str()`.
use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlChange, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{ChoiceBox, FieldSelectionSingle, Paragraph, RadioButton};
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};
use bevy_extended_ui_macros::html_fn;

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, (configure_ui, load_ui))
        .run();
}

fn configure_ui(mut config: ResMut<ExtendedUiConfiguration>) {
    config.camera = ExtendedCam::Default;
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/typed_values.html");
    reg.add_and_use(
        "typed-values-demo".to_string(),
        HtmlSource::from_handle(handle),
    );
}

// ---------------------------------------------------------------------------
// Helper – write a string to a result label by CssID
// ---------------------------------------------------------------------------

fn set_result(
    id: &str,
    text: String,
    paragraph_q: &mut Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    for (mut p, css_id) in paragraph_q.iter_mut() {
        if css_id.0 == id {
            p.text = text;
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// ChoiceBox - integer (i32)
// ---------------------------------------------------------------------------

#[html_fn("on_choice_int_change")]
fn on_choice_int_change(
    In(event): In<HtmlChange>,
    choice_q: Query<&ChoiceBox>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(cb) = choice_q.get(event.entity) else {
        return;
    };

    let text = match cb.value.get_value::<i32>() {
        Some(n) => format!("Selected: {} (i32)", n),
        None => format!("Selected: {}", cb.value.text),
    };
    set_result("choice-int-result", text, &mut paragraph_q);
}

// ---------------------------------------------------------------------------
// ChoiceBox - boolean
// ---------------------------------------------------------------------------

#[html_fn("on_choice_bool_change")]
fn on_choice_bool_change(
    In(event): In<HtmlChange>,
    choice_q: Query<&ChoiceBox>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(cb) = choice_q.get(event.entity) else {
        return;
    };

    let text = match cb.value.get_value::<bool>() {
        Some(b) => format!("Selected: {} (bool)", b),
        None => format!("Selected: {}", cb.value.text),
    };
    set_result("choice-bool-result", text, &mut paragraph_q);
}

// ---------------------------------------------------------------------------
// ChoiceBox - enum-like (plain String, mapped to Direction on Rust side)
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "North" => Some(Self::North),
            "South" => Some(Self::South),
            "East" => Some(Self::East),
            "West" => Some(Self::West),
            _ => None,
        }
    }
}

#[html_fn("on_choice_enum_change")]
fn on_choice_enum_change(
    In(event): In<HtmlChange>,
    choice_q: Query<&ChoiceBox>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(cb) = choice_q.get(event.entity) else {
        return;
    };

    let text = match cb.value.value_as_str().and_then(Direction::from_str) {
        Some(dir) => format!("Selected: {:?} (Direction)", dir),
        None => format!("Selected: {}", cb.value.text),
    };
    set_result("choice-enum-result", text, &mut paragraph_q);
}

// ---------------------------------------------------------------------------
// ChoiceBox - object (resolution stored as plain String, parsed to struct)
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    fn from_str(s: &str) -> Option<Self> {
        let (w, h) = s.split_once('x')?;
        Some(Self {
            width: w.parse().ok()?,
            height: h.parse().ok()?,
        })
    }
}

#[html_fn("on_choice_obj_change")]
fn on_choice_obj_change(
    In(event): In<HtmlChange>,
    choice_q: Query<&ChoiceBox>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(cb) = choice_q.get(event.entity) else {
        return;
    };

    let text = match cb.value.value_as_str().and_then(Resolution::from_str) {
        Some(res) => format!("Selected: {}x{} (Resolution)", res.width, res.height),
        None => format!("Selected: {}", cb.value.text),
    };
    set_result("choice-obj-result", text, &mut paragraph_q);
}

// ---------------------------------------------------------------------------
// RadioButton - integer (u32)  - fieldset emits HtmlChange on the FieldSet entity
// ---------------------------------------------------------------------------

#[html_fn("on_radio_int_change")]
fn on_radio_int_change(
    In(event): In<HtmlChange>,
    selection_q: Query<&FieldSelectionSingle>,
    radio_q: Query<&RadioButton>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(sel) = selection_q.get(event.entity) else {
        return;
    };
    let Some(radio_entity) = sel.0 else { return };
    let Ok(rb) = radio_q.get(radio_entity) else {
        return;
    };

    let text = match rb.get_value::<u32>() {
        Some(n) => format!("Selected: {} (u32)", n),
        None => format!("Selected: {}", rb.label),
    };
    set_result("radio-int-result", text, &mut paragraph_q);
}

// ---------------------------------------------------------------------------
// RadioButton - boolean
// ---------------------------------------------------------------------------

#[html_fn("on_radio_bool_change")]
fn on_radio_bool_change(
    In(event): In<HtmlChange>,
    selection_q: Query<&FieldSelectionSingle>,
    radio_q: Query<&RadioButton>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(sel) = selection_q.get(event.entity) else {
        return;
    };
    let Some(radio_entity) = sel.0 else { return };
    let Ok(rb) = radio_q.get(radio_entity) else {
        return;
    };

    let text = match rb.get_value::<bool>() {
        Some(b) => format!("Selected: {} (bool)", b),
        None => format!("Selected: {}", rb.label),
    };
    set_result("radio-bool-result", text, &mut paragraph_q);
}

// ---------------------------------------------------------------------------
// RadioButton - enum-like (Theme, mapped from String)
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Light" => Some(Self::Light),
            "Dark" => Some(Self::Dark),
            "System" => Some(Self::System),
            _ => None,
        }
    }
}

#[html_fn("on_radio_enum_change")]
fn on_radio_enum_change(
    In(event): In<HtmlChange>,
    selection_q: Query<&FieldSelectionSingle>,
    radio_q: Query<&RadioButton>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(sel) = selection_q.get(event.entity) else {
        return;
    };
    let Some(radio_entity) = sel.0 else { return };
    let Ok(rb) = radio_q.get(radio_entity) else {
        return;
    };

    let text = match rb.value_as_str().and_then(Theme::from_str) {
        Some(theme) => format!("Selected: {:?} (Theme)", theme),
        None => format!("Selected: {}", rb.label),
    };
    set_result("radio-enum-result", text, &mut paragraph_q);
}

// ---------------------------------------------------------------------------
// RadioButton - object (f32 scale factor)
// ---------------------------------------------------------------------------

#[html_fn("on_radio_obj_change")]
fn on_radio_obj_change(
    In(event): In<HtmlChange>,
    selection_q: Query<&FieldSelectionSingle>,
    radio_q: Query<&RadioButton>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Ok(sel) = selection_q.get(event.entity) else {
        return;
    };
    let Some(radio_entity) = sel.0 else { return };
    let Ok(rb) = radio_q.get(radio_entity) else {
        return;
    };

    let text = match rb.get_value::<f32>() {
        Some(scale) => format!("Selected: {:.2}x (f32)", scale),
        None => format!("Selected: {}", rb.label),
    };
    set_result("radio-obj-result", text, &mut paragraph_q);
}
