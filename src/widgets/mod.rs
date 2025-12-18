mod body;
mod content;
mod controls;
mod div;

use crate::registry::*;
use crate::styles::IconPlace;
use crate::widgets::body::BodyWidget;
use crate::widgets::content::ExtendedContentWidgets;
use crate::widgets::controls::ExtendedControlWidgets;
use crate::widgets::div::DivWidget;
use bevy::prelude::*;
use std::fmt;

/// Marker component for UI elements that should ignore the parent widget state.
///
/// Used to mark UI nodes that do not inherit state like `focused`, `hovered`, etc.
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
}

#[derive(Component, Default, Clone, Debug)]
pub struct Widget(pub Option<String>);

#[derive(Component, Clone, Copy, Debug)]
pub struct WidgetId {
    pub id: usize,
    pub kind: WidgetKind,
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetKind {
    Body,
    Button,
    CheckBox,
    ChoiceBox,
    Div,
    Divider,
    FieldSet,
    Headline,
    Img,
    InputField,
    Paragraph,
    ProgressBar,
    RadioButton,
    Slider,
    SwitchButton,
    ToggleButton,
}

pub struct ExtendedWidgetPlugin;

impl Plugin for ExtendedWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UIGenID>();
        app.register_type::<BindToID>();
        app.register_type::<UIWidgetState>();
        app.register_type::<Body>();
        app.add_plugins((
            ExtendedControlWidgets,
            ExtendedContentWidgets,
            BodyWidget,
            DivWidget,
        ));
    }
}

// ===============================================
//                       Body
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Body {
    pub entry: usize,
    pub html_key: Option<String>,
}

impl Default for Body {
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

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Div(pub usize);

impl Default for Div {
    fn default() -> Self {
        let entry = DIV_ID_POOL.lock().unwrap().acquire();
        Self(entry)
    }
}

// ===============================================
//                     Button
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Button {
    pub entry: usize,
    pub text: String,
    pub icon_place: IconPlace,
    pub icon_path: Option<String>,
}

impl Default for Button {
    fn default() -> Self {
        let entry = BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            text: String::from("Button"),
            icon_path: None,
            icon_place: IconPlace::default(),
        }
    }
}

// ===============================================
//                     CheckBox
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct CheckBox {
    pub entry: usize,
    pub label: String,
    pub icon_path: Option<String>,
}

impl Default for CheckBox {
    fn default() -> Self {
        let entry = CHECK_BOX_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            label: String::from("label"),
            icon_path: Some(String::from("extended_ui/icons/check-mark.png")),
        }
    }
}

// ===============================================
//                   ChoiceBox
// ===============================================

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

#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct ChoiceOption {
    pub text: String,
    pub internal_value: String,
    pub icon_path: Option<String>,
}

impl Default for ChoiceOption {
    fn default() -> Self {
        Self {
            text: String::from("Please Select"),
            internal_value: String::from("default"),
            icon_path: None,
        }
    }
}

impl ChoiceOption {
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

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Divider {
    pub entry: usize,
    pub alignment: DividerAlignment,
}

impl Default for Divider {
    fn default() -> Self {
        let entry = DIVIDER_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            alignment: DividerAlignment::default(),
        }
    }
}

#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum DividerAlignment {
    #[default]
    Vertical,
    Horizontal,
}

impl DividerAlignment {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "vertical" | "vert" | "v" => Some(Self::Vertical),
            "horizontal" | "horiz" | "h" => Some(Self::Horizontal),
            _ => None,
        }
    }
}

impl fmt::Display for DividerAlignment {
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

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Radio,
    Toggle,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldMode {
    Multi,
    Single,
    Count(u8),
}

impl FieldMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "single" | "solo" | "one" => Some(Self::Single),
            "multi" | "more" => Some(Self::Multi),
            "count" => Some(Self::Count(0)),
            _ => None,
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct InFieldSet(pub Entity);

#[derive(Component, Reflect, Debug, Default)]
pub struct FiledSelectionSingle(pub Option<Entity>);

#[derive(Component, Reflect, Debug, Default)]
pub struct FieldSelectionMulti(pub Vec<Entity>);

// ===============================================
//                   Headline
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Headline {
    pub entry: usize,
    pub text: String,
    pub h_type: HeadlineType,
}

impl Default for Headline {
    fn default() -> Self {
        let entry = HEADLINE_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            text: String::from("Headline"),
            h_type: HeadlineType::H3,
        }
    }
}

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

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Img {
    pub entry: usize,
    pub src: Option<String>,
    pub alt: String,
}

impl Default for Img {
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

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct InputField {
    pub entry: usize,
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
    fn default() -> Self {
        let entry = INPUT_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
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
    pub fn is_valid_char(&self, c: char) -> bool {
        match self {
            InputType::Text | InputType::Password => true,
            InputType::Number => c.is_ascii_digit() || "+-*/() ".contains(c),
            InputType::Email => c.is_ascii_alphanumeric() || c == '@' || c == '.' || c == '-',
            InputType::Date => c.is_ascii_digit() || c == '/' || c == '-' || c == '.',
        }
    }

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
            Self::CapAtNodeSize => 0,
        }
    }
}

// ===============================================
//                     Paragraph
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Paragraph {
    pub entry: usize,
    pub text: String,
}

impl Default for Paragraph {
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
//                      Slider
// ===============================================

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
//                   Switch Button
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct SwitchButton {
    pub entry: usize,
    pub label: String,
    pub icon: Option<String>,
}

impl Default for SwitchButton {
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