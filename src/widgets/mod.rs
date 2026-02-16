mod body;
mod content;
mod controls;
mod div;
mod form;
mod validation;
mod widget_util;

use crate::registry::*;
use crate::styles::IconPlace;
use crate::widgets::body::BodyWidget;
use crate::widgets::content::ExtendedContentWidgets;
use crate::widgets::controls::ExtendedControlWidgets;
use crate::widgets::div::DivWidget;
use crate::widgets::form::FormWidget;
use bevy::prelude::*;
use std::fmt;

pub(crate) use validation::evaluate_validation_state;

/// Marker component for UI elements that should ignore the parent widget state.
///
/// Used to mark UI nodes that do not inherit state like `focused` or `hovered`.
#[derive(Component)]
pub struct IgnoreParentState;

/// Unique identifier for UI elements.
///
/// Each UI element should have a unique `UIGenID` generated atomically.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UIGenID(usize);

impl Default for UIGenID {
    /// Generates a new unique `UIGenID` using a global atomic counter.
    fn default() -> Self {
        Self(UI_ID_GENERATE.lock().unwrap().acquire())
    }
}

impl UIGenID {
    /// Returns the underlying numeric ID.
    pub fn get(&self) -> usize {
        self.0
    }
}

/// Associates a UI child entity with a parent widget by ID.
///
/// Used for binding UI components to their logical parent.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BindToID(pub usize);

impl BindToID {
    /// Returns the bound widget ID.
    pub fn get(&self) -> usize {
        self.0
    }
}

/// Stores the interaction and UI state flags for a widget.
///
/// Contains boolean flags for common widget states such as focused, hovered, disabled, etc.
#[derive(Component, Reflect, Default, PartialEq, Eq, Debug, Clone)]
#[reflect(Component)]
pub struct UIWidgetState {
    pub focused: bool,
    pub hovered: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub checked: bool,
    pub open: bool,
    pub invalid: bool,
}

/// Component storing an optional widget controller name.
#[derive(Component, Default, Clone, Debug)]
pub struct Widget(pub Option<String>);

/// Validation rules parsed from HTML attributes.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct ValidationRules {
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

impl ValidationRules {
    /// Parses validation rules from a `validation` attribute string.
    pub fn from_attribute(value: &str) -> Option<Self> {
        let mut rules = ValidationRules::default();

        for part in value.split('&') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let lower = trimmed.to_ascii_lowercase();
            if lower == "required" {
                rules.required = true;
                continue;
            }

            if let Some((name, args)) = trimmed.split_once('(') {
                let name = name.trim().to_ascii_lowercase();
                let args = args.trim_end_matches(')').trim();
                match name.as_str() {
                    "length" => apply_length_rules(args, &mut rules),
                    "pattern" => apply_pattern_rule(args, &mut rules),
                    _ => {}
                }
            }
        }

        if rules.is_empty() { None } else { Some(rules) }
    }

    /// Returns true when no rules are configured.
    fn is_empty(&self) -> bool {
        !self.required
            && self.min_length.is_none()
            && self.max_length.is_none()
            && self.pattern.is_none()
    }
}

/// Applies length-related validation rules from arguments.
fn apply_length_rules(args: &str, rules: &mut ValidationRules) {
    let parts: Vec<&str> = args.split(',').map(|part| part.trim()).collect();
    if parts.is_empty() {
        return;
    }

    let parse_part = |part: &str| part.parse::<usize>().ok();

    match parts.as_slice() {
        [single] => {
            if let Some(value) = parse_part(single) {
                rules.min_length = Some(value);
                rules.max_length = Some(value);
            }
        }
        [min, max, ..] => {
            if let Some(value) = parse_part(min) {
                rules.min_length = Some(value);
            }
            if let Some(value) = parse_part(max) {
                rules.max_length = Some(value);
            }
        }
        &[] => todo!(),
    }
}

/// Applies a pattern rule from arguments.
fn apply_pattern_rule(args: &str, rules: &mut ValidationRules) {
    let trimmed = args.trim();
    let stripped = trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            trimmed
                .strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        })
        .unwrap_or(trimmed);

    if stripped.is_empty() {
        return;
    }

    rules.pattern = Some(stripped.to_string());
}

/// Component carrying a widget ID and it's kind.
#[derive(Component, Clone, Copy, Debug)]
pub struct WidgetId {
    pub id: usize,
    pub kind: WidgetKind,
}

/// Enumerates the supported widget kinds.
#[derive(Debug, Clone, Copy)]
pub enum WidgetKind {
    Body,
    Button,
    ColorPicker,
    CheckBox,
    ChoiceBox,
    Div,
    Divider,
    Form,
    FieldSet,
    Headline,
    Img,
    InputField,
    Paragraph,
    ProgressBar,
    RadioButton,
    Scrollbar,
    Slider,
    SwitchButton,
    ToggleButton,
}

/// Plugin that registers all built-in widget types.
pub struct ExtendedWidgetPlugin;

impl Plugin for ExtendedWidgetPlugin {
    /// Registers widget components and systems.
    fn build(&self, app: &mut App) {
        app.register_type::<UIGenID>();
        app.register_type::<BindToID>();
        app.register_type::<UIWidgetState>();
        app.register_type::<ValidationRules>();
        app.register_type::<Body>();
        app.register_type::<Form>();
        app.add_plugins((
            ExtendedControlWidgets,
            ExtendedContentWidgets,
            BodyWidget,
            DivWidget,
            FormWidget,
        ));
        app.add_systems(Update, validation::update_validation_states);
    }
}

// ===============================================
//                       Body
// ===============================================

/// Root widget representing the HTML `<body>` element.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Body {
    pub entry: usize,
    pub html_key: Option<String>,
}

impl Default for Body {
    /// Creates a default body widget with a unique entry ID.
    fn default() -> Self {
        let entry = BODY_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            html_key: None,
        }
    }
}

// ===============================================
//                       Div
// ===============================================

/// Container widget representing a `<div>` element.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Div(pub usize);

impl Default for Div {
    /// Creates a default div widget with a unique entry ID.
    fn default() -> Self {
        let entry = DIV_ID_POOL.lock().unwrap().acquire();
        Self(entry)
    }
}

// ===============================================
//                       Form
// ===============================================

/// Form the container widget with an optional submit action handler name.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Form {
    pub entry: usize,
    pub action: Option<String>,
    pub validate_mode: FormValidationMode,
}

impl Default for Form {
    /// Creates a default form widget with a unique entry ID.
    fn default() -> Self {
        let entry = FORM_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            action: None,
            validate_mode: FormValidationMode::default(),
        }
    }
}

/// Defines when form validation should be evaluated.
#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum FormValidationMode {
    /// Validate continuously (e.g. focus/hover/input changes).
    Always,
    /// Validate only on submit.
    #[default]
    Send,
    /// Validate on direct interaction (e.g. input text changes).
    Interact,
}

impl FormValidationMode {
    /// Parses a validation mode from the form `validate` attribute.
    pub fn from_str(value: &str) -> Option<FormValidationMode> {
        match value.trim().to_ascii_lowercase().as_str() {
            "always" | "all" => Some(FormValidationMode::Always),
            "send" => Some(FormValidationMode::Send),
            "interact" => Some(FormValidationMode::Interact),
            _ => None,
        }
    }
}

// ===============================================
//                     Button
// ===============================================

/// Supported button behavior modes.
#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum ButtonType {
    /// No explicit button type set.
    #[default]
    Auto,
    /// Regular clickable button that does not submit a form.
    Button,
    /// Form submit button.
    Submit,
    /// Form reset button.
    Reset,
}

impl ButtonType {
    /// Parses a button type from an HTML `type` attribute.
    pub fn from_str(value: &str) -> Option<ButtonType> {
        match value.to_ascii_lowercase().as_str() {
            "button" => Some(ButtonType::Button),
            "submit" => Some(ButtonType::Submit),
            "reset" => Some(ButtonType::Reset),
            _ => None,
        }
    }
}

/// Button widget with optional icon.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Button {
    pub entry: usize,
    pub text: String,
    pub icon_place: IconPlace,
    pub icon_path: Option<String>,
    pub button_type: ButtonType,
}

impl Default for Button {
    /// Creates a default button widget.
    fn default() -> Self {
        let entry = BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            text: String::from("Button"),
            icon_path: None,
            icon_place: IconPlace::default(),
            button_type: ButtonType::default(),
        }
    }
}

// ===============================================
//                     CheckBox
// ===============================================

/// Checkbox widget with label and checked state.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct CheckBox {
    pub entry: usize,
    pub label: String,
    pub icon_path: Option<String>,
    pub checked: bool,
}

impl Default for CheckBox {
    /// Creates a default checkbox widget.
    fn default() -> Self {
        let entry = CHECK_BOX_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            label: String::from("label"),
            icon_path: Some(String::from("extended_ui/icons/check-mark.png")),
            checked: false,
        }
    }
}

// ===============================================
//                   ChoiceBox
// ===============================================

/// Choice box widget with selectable options.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct ChoiceBox {
    pub entry: usize,
    pub label: String,
    pub value: ChoiceOption,
    pub options: Vec<ChoiceOption>,
    pub icon_path: Option<String>,
}

impl Default for ChoiceBox {
    /// Creates a default choice box widget.
    fn default() -> Self {
        let entry = CHOICE_BOX_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            label: String::from("select"),
            value: ChoiceOption::default(),
            options: vec![ChoiceOption::default()],
            icon_path: Some(String::from("extended_ui/icons/drop-arrow.png")),
        }
    }
}

/// Single option entry used by choice boxes.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct ChoiceOption {
    pub text: String,
    pub internal_value: String,
    pub icon_path: Option<String>,
}

impl Default for ChoiceOption {
    /// Creates a default option labeled "Please Select".
    fn default() -> Self {
        Self {
            text: String::from("Please Select"),
            internal_value: String::from("default"),
            icon_path: None,
        }
    }
}

impl ChoiceOption {
    /// Creates an option using the provided text.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            internal_value: text.trim().to_string(),
            icon_path: None,
        }
    }
}

// ===============================================
//                   Divider
// ===============================================

/// Divider widget with an alignment direction.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Divider {
    pub entry: usize,
    pub alignment: DividerAlignment,
}

impl Default for Divider {
    /// Creates a default divider widget.
    fn default() -> Self {
        let entry = DIVIDER_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            alignment: DividerAlignment::default(),
        }
    }
}

/// Orientation of a divider widget.
#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum DividerAlignment {
    #[default]
    Vertical,
    Horizontal,
}

impl DividerAlignment {
    /// Parses a divider alignment from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "vertical" | "vert" | "v" => Some(Self::Vertical),
            "horizontal" | "horiz" | "h" => Some(Self::Horizontal),
            _ => None,
        }
    }
}

impl fmt::Display for DividerAlignment {
    /// Formats the alignment as a lowercase string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DividerAlignment::Horizontal => "horizontal",
            DividerAlignment::Vertical => "vertical",
        };
        write!(f, "{}", s)
    }
}

// ===============================================
//                   FieldSet
// ===============================================

/// Field set widget grouping selectable children.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct FieldSet {
    pub entry: usize,
    pub kind: Option<FieldKind>,
    pub field_mode: FieldMode,
    pub allow_none: bool,
}

impl Default for FieldSet {
    /// Creates a default field set widget.
    fn default() -> Self {
        let entry = FIELDSET_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            kind: None,
            field_mode: FieldMode::Single,
            allow_none: false,
        }
    }
}

/// Field set content kind.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Radio,
    Toggle,
}

/// Selection mode for field sets.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldMode {
    Multi,
    Single,
    Count(u8),
}

impl FieldMode {
    /// Parses a field mode from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "single" | "solo" | "one" => Some(Self::Single),
            "multi" | "more" => Some(Self::Multi),
            "count" => Some(Self::Count(0)),
            _ => None,
        }
    }
}

/// Component marking that an entity belongs to a field set.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct InFieldSet(pub Entity);

/// Tracks a single selected entity within a field set.
#[derive(Component, Reflect, Debug, Default)]
pub struct FieldSelectionSingle(pub Option<Entity>);

/// Tracks multiple selected entities within a field set.
#[derive(Component, Reflect, Debug, Default)]
pub struct FieldSelectionMulti(pub Vec<Entity>);

// ===============================================
//                   Headline
// ===============================================

/// Headline widget with a selectable heading level.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Headline {
    pub entry: usize,
    pub text: String,
    pub h_type: HeadlineType,
}

impl Default for Headline {
    /// Creates a default headline widget.
    fn default() -> Self {
        let entry = HEADLINE_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            text: String::from("Headline"),
            h_type: HeadlineType::H3,
        }
    }
}

/// Heading level for headline widgets.
#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum HeadlineType {
    #[default]
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl fmt::Display for HeadlineType {
    /// Formats the heading level as a lowercase string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HeadlineType::H1 => "h1",
            HeadlineType::H2 => "h2",
            HeadlineType::H3 => "h3",
            HeadlineType::H4 => "h4",
            HeadlineType::H5 => "h5",
            HeadlineType::H6 => "h6",
        };
        write!(f, "{}", s)
    }
}

// ===============================================
//                       Image
// ===============================================

/// Image widget referencing an optional source path.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Img {
    pub entry: usize,
    pub src: Option<String>,
    pub alt: String,
}

impl Default for Img {
    /// Creates a default image widget.
    fn default() -> Self {
        let entry = IMAGE_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            src: None,
            alt: String::from(""),
        }
    }
}

// ===============================================
//                   InputField
// ===============================================

/// Input field widget with text and validation settings.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget, InputValue)]
pub struct InputField {
    pub entry: usize,
    pub name: String,
    pub text: String,
    pub label: String,
    pub placeholder: String,
    pub cursor_position: usize,
    pub clear_after_focus_lost: bool,
    pub icon_path: Option<String>,
    pub input_type: InputType,
    pub cap_text_at: InputCap,
}

impl Default for InputField {
    /// Creates a default input field widget.
    fn default() -> Self {
        let entry = INPUT_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            name: String::new(),
            text: String::from(""),
            label: String::from("Label"),
            placeholder: String::from(""),
            clear_after_focus_lost: false,
            cursor_position: 0,
            icon_path: None,
            cap_text_at: InputCap::default(),
            input_type: InputType::default(),
        }
    }
}

/// Supported input types for input fields.
#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum InputType {
    #[default]
    Text,
    Email,
    Date,
    Password,
    Number,
}

impl InputType {
    /// Returns true if the character is allowed for this input type.
    pub fn is_valid_char(&self, c: char) -> bool {
        match self {
            InputType::Text | InputType::Password => true,
            InputType::Number => c.is_ascii_digit() || "+-*/() ".contains(c),
            InputType::Email => c.is_ascii_alphanumeric() || c == '@' || c == '.' || c == '-',
            InputType::Date => c.is_ascii_digit() || c == '/' || c == '-' || c == '.',
        }
    }

    /// Parses an input type from a string.
    pub fn from_str(value: &str) -> Option<InputType> {
        match value.to_lowercase().as_str() {
            "text" => Some(InputType::Text),
            "password" => Some(InputType::Password),
            "number" => Some(InputType::Number),
            "email" => Some(InputType::Email),
            "date" => Some(InputType::Date),
            _ => None,
        }
    }
}

/// Input length capping configuration.
#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum InputCap {
    #[default]
    NoCap,
    CapAtNodeSize,
    CapAt(usize), // 0 means no cap!
}

impl InputCap {
    /// Returns the configured maximum length or zero for no cap.
    pub fn get_value(&self) -> usize {
        match self {
            Self::CapAt(value) => *value,
            Self::NoCap => 0,
            Self::CapAtNodeSize => 0,
        }
    }
}

/// Component storing the raw input value for a widget.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct InputValue(pub String);

// ===============================================
//                     Paragraph
// ===============================================

/// Paragraph widget for body text.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Paragraph {
    pub entry: usize,
    pub text: String,
}

impl Default for Paragraph {
    /// Creates a default paragraph widget.
    fn default() -> Self {
        let entry = PARAGRAPH_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            text: String::from(""),
        }
    }
}

// ===============================================
//                    ProgressBar
// ===============================================

/// Progress bar widget with numeric range.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, InheritedVisibility, Widget)]
pub struct ProgressBar {
    pub entry: usize,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl Default for ProgressBar {
    /// Creates a default progress bar widget.
    fn default() -> Self {
        let entry = PROGRESS_BAR_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            value: 0.0,
            max: 100.0,
            min: 0.0,
        }
    }
}

// ===============================================
//                   Radio Button
// ===============================================

/// Radio button widget with a selectable value.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct RadioButton {
    pub entry: usize,
    pub label: String,
    pub value: String,
    pub selected: bool,
}

impl Default for RadioButton {
    /// Creates a default radio button widget.
    fn default() -> Self {
        let entry = RADIO_BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            label: String::from("label"),
            value: String::from(""),
            selected: false,
        }
    }
}

// ===============================================
//                     Scrollbar
// ===============================================

/// Scrollbar widget for scrollable containers.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Scrollbar {
    pub entry: usize,
    pub entity: Option<Entity>,
    pub value: f32, // 3.146675432...............
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub vertical: bool,
    pub viewport_extent: f32,
    pub content_extent: f32,
}

impl Default for Scrollbar {
    /// Creates a default scrollbar widget.
    fn default() -> Self {
        let entry = SCROLL_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            entity: None,
            value: 0.0,
            min: 0.0,
            max: 1000.0,
            step: 10.0,
            vertical: true,
            viewport_extent: 0.0,
            content_extent: 0.0,
        }
    }
}

// ===============================================
//                      Slider
// ===============================================

/// Slider widget with numeric range.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Slider {
    pub entry: usize,
    pub value: f32,
    pub step: f32,
    pub min: f32,
    pub max: f32,
}

impl Default for Slider {
    /// Creates a default slider widget.
    fn default() -> Self {
        let entry = SLIDER_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            value: 0.0,
            step: 1.0,
            min: 0.0,
            max: 100.0,
        }
    }
}

// ===============================================
//                    Color Picker
// ===============================================

/// Color picker widget with HSV interaction and RGB/RGBA/HEX output values.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct ColorPicker {
    pub entry: usize,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

impl Default for ColorPicker {
    /// Creates a default color picker set to Google blue.
    fn default() -> Self {
        let entry = COLOR_PICKER_ID_POOL.lock().unwrap().acquire();
        Self::from_rgba_u8_with_entry(entry, 0x42, 0x85, 0xF4, 255)
    }
}

impl ColorPicker {
    /// Creates a color picker from RGBA bytes.
    pub fn from_rgba_u8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        let entry = COLOR_PICKER_ID_POOL.lock().unwrap().acquire();
        Self::from_rgba_u8_with_entry(entry, red, green, blue, alpha)
    }

    fn from_rgba_u8_with_entry(entry: usize, red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        let (hue, saturation, value) = rgb_u8_to_hsv(red, green, blue);
        Self {
            entry,
            red,
            green,
            blue,
            alpha,
            hue,
            saturation,
            value,
        }
    }

    /// Updates RGB values from HSV while preserving alpha.
    pub fn set_hsv(&mut self, hue: f32, saturation: f32, value: f32) {
        self.hue = hue.rem_euclid(360.0);
        self.saturation = saturation.clamp(0.0, 1.0);
        self.value = value.clamp(0.0, 1.0);
        let (r, g, b) = hsv_to_rgb_u8(self.hue, self.saturation, self.value);
        self.red = r;
        self.green = g;
        self.blue = b;
    }

    /// Updates HSV values from RGB while preserving alpha.
    pub fn set_rgb(&mut self, red: u8, green: u8, blue: u8) {
        self.red = red;
        self.green = green;
        self.blue = blue;
        let (hue, saturation, value) = rgb_u8_to_hsv(red, green, blue);
        self.hue = hue;
        self.saturation = saturation;
        self.value = value;
    }

    /// Returns the current color as a HEX string (`#RRGGBB`).
    pub fn hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }

    /// Returns the current color as an `rgb(r, g, b)` string.
    pub fn rgb_string(&self) -> String {
        format!("rgb({}, {}, {})", self.red, self.green, self.blue)
    }

    /// Returns the current color as an `rgba(r, g, b, a)` string (alpha in `0..255`).
    pub fn rgba_string(&self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

pub fn hsv_to_rgb_u8(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let h = hue.rem_euclid(360.0);
    let s = saturation.clamp(0.0, 1.0);
    let v = value.clamp(0.0, 1.0);

    if s <= f32::EPSILON {
        let gray = (v * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h as i32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let to_u8 = |f: f32| ((f + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    (to_u8(r1), to_u8(g1), to_u8(b1))
}

fn rgb_u8_to_hsv(red: u8, green: u8, blue: u8) -> (f32, f32, f32) {
    let r = red as f32 / 255.0;
    let g = green as f32 / 255.0;
    let b = blue as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let hue = if delta <= f32::EPSILON {
        0.0
    } else if (max - r).abs() <= f32::EPSILON {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if (max - g).abs() <= f32::EPSILON {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let saturation = if max <= f32::EPSILON {
        0.0
    } else {
        delta / max
    };
    (hue.rem_euclid(360.0), saturation, max)
}

// ===============================================
//                   Switch Button
// ===============================================

/// Switch button widget with a label and optional icon.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct SwitchButton {
    pub entry: usize,
    pub label: String,
    pub icon: Option<String>,
}

impl Default for SwitchButton {
    /// Creates a default switch button widget.
    fn default() -> Self {
        let entry = SWITCH_BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            label: String::from(""),
            icon: None,
        }
    }
}

// ===============================================
//                   Toggle Button
// ===============================================

/// Toggle button widget with selectable state.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct ToggleButton {
    pub entry: usize,
    pub label: String,
    pub value: String,
    pub icon_place: IconPlace,
    pub icon_path: Option<String>,
    pub selected: bool,
}

impl Default for ToggleButton {
    /// Creates a default toggle button widget.
    fn default() -> Self {
        let entry = TOGGLE_BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            label: String::from("label"),
            value: String::from(""),
            icon_path: None,
            icon_place: IconPlace::default(),
            selected: false,
        }
    }
}
