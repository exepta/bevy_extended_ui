mod button;
mod div;
mod check_box;
mod slider;
mod input;
mod choice_box;
mod body;
mod headline;
mod paragraph;
mod image;
mod progress_bar;

use std::fmt;
use bevy::prelude::*;
use crate::{UIGenID, UIWidgetState};
use crate::html::{HtmlSource, HtmlEventBindings};
use crate::registry::*;
use crate::styling::IconPlace;
use crate::widgets::body::HtmlBodyWidget;
use crate::widgets::button::ButtonWidget;
use crate::widgets::check_box::CheckBoxWidget;
use crate::widgets::choice_box::ChoiceBoxWidget;
use crate::widgets::div::DivWidget;
use crate::widgets::headline::HeadlineWidget;
use crate::widgets::image::ImageWidget;
use crate::widgets::input::InputWidget;
use crate::widgets::paragraph::ParagraphWidget;
use crate::widgets::progress_bar::ProgressBarWidget;
use crate::widgets::slider::SliderWidget;

#[derive(Component, Default)]
pub struct Widget;

#[derive(Component, Clone, Copy, Debug)]
pub struct WidgetId {
    pub id: usize,
    pub kind: WidgetKind,
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetKind {
    HtmlBody,
    Div,
    Headline,
    Paragraph,
    Button,
    CheckBox,
    Slider,
    InputField,
    ChoiceBox,
    Img,
    ProgressBar,
}

// ===============================================
//                       Body
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget, HtmlEventBindings)]
pub struct HtmlBody {
    pub w_count: usize,
    pub bind_to_html: Option<String>,
    pub fn_controller: Option<String>,
    pub source: Option<HtmlSource>,
}

impl Default for HtmlBody {
    fn default() -> Self {
        let w_count = BODY_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            bind_to_html: None,
            fn_controller: None,
            source: None
        }
    }
}

// ===============================================
//                       Div
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget, HtmlEventBindings)]
pub struct Div(pub usize);

impl Default for Div {
    fn default() -> Self {
        let w_count = DIV_ID_POOL.lock().unwrap().acquire();
        Self(w_count)
    }
}

// ===============================================
//                       Headline
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget, HtmlEventBindings)]
pub struct Headline {
    pub w_count: usize,
    pub text: String,
    pub h_type: HeadlineType
}

impl Default for Headline {

    fn default() -> Self {
        let w_count = HEADLINE_ID_POOL.lock().unwrap().acquire();
        Self {
            w_count,
            text: String::from("Headline"),
            h_type: HeadlineType::H3
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
//                     Paragraph
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget, HtmlEventBindings)]
pub struct Paragraph {
    pub w_count: usize,
    pub text: String,
}

impl Default for Paragraph {
    fn default() -> Self {
        let w_count = PARAGRAPH_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            text: String::from(""),
        }
    }
}

// ===============================================
//                     Button
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget, HtmlEventBindings)]
pub struct Button {
    pub w_count: usize,
    pub text: String,
    pub icon_place: IconPlace,
    pub icon_path: Option<String>,
}

impl Default for Button {
    fn default() -> Self {
        let w_count = BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
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
#[require(UIGenID, UIWidgetState, Widget, HtmlEventBindings)]
pub struct CheckBox {
    pub w_count: usize,
    pub label: String,
    pub icon_path: Option<String>,
}

impl Default for CheckBox {
    fn default() -> Self {
        let w_count = CHECK_BOX_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            label: String::from("label"),
            icon_path: Some(String::from("extended_ui/icons/check-mark.png")),
        }
    }
}

// ===============================================
//                      Slider
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget, HtmlEventBindings)]
pub struct Slider {
    pub w_count: usize,
    pub value: f32,
    pub step: f32,
    pub min: f32,
    pub max: f32
}

impl Default for Slider {
    fn default() -> Self {
        let w_count = SLIDER_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            value: 0.0,
            step: 1.0,
            min: 0.0,
            max: 100.0,
        }
    }
}

// ===============================================
//                   InputField
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget, HtmlEventBindings)]
pub struct InputField {
    pub w_count: usize,
    pub text: String,
    pub label: String,
    pub placeholder: String,
    pub cursor_position: usize,
    pub clear_after_focus_lost: bool,
    pub icon_path: Option<String>,
    pub input_type: InputType,
    pub cap_text_at: InputCap
}

impl Default for InputField {
    fn default() -> Self {
        let w_count = INPUT_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
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
    
    pub fn from_str(value: &str) -> Option<InputType> {
        match value.to_lowercase().as_str() {
            "text" => Some(InputType::Text),
            "password" => Some(InputType::Password),
            "number" => Some(InputType::Number),
            _ => None
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

// ===============================================
//                   ChoiceBox
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget, HtmlEventBindings)]
pub struct ChoiceBox {
    pub w_count: usize,
    pub label: String,
    pub value: ChoiceOption,
    pub options: Vec<ChoiceOption>,
    pub icon_path: Option<String>,
}

impl Default for ChoiceBox {
    fn default() -> Self {
        let w_count = SELECT_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
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
//                       Image
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget, HtmlEventBindings)]
pub struct Img {
    pub w_count: usize,
    pub src: Option<String>,
    pub alt: String,
}

impl Default for Img {
    fn default() -> Self {
        let w_count = IMG_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            src: None,
            alt: String::from(""),
        }
    }
}

// ===============================================
//                   ProgressBar
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget, HtmlEventBindings)]
pub struct ProgressBar {
    pub w_count: usize,
    pub value: f32,
    pub max: f32,
    pub min: f32
}

impl Default for ProgressBar {
    fn default() -> Self {
        let w_count = PROGRESS_BAR_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            value: 0.0,
            max: 100.0,
            min: 0.0,
        }
    }
}

pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HtmlBody>();
        app.register_type::<Div>();
        app.register_type::<Button>();
        app.register_type::<CheckBox>();
        app.register_type::<Slider>();
        app.register_type::<ProgressBar>();
        app.register_type::<InputField>();
        app.register_type::<ChoiceBox>();
        app.register_type::<Headline>();
        app.register_type::<Paragraph>();
        app.register_type::<Img>();
        app.add_plugins((
            HtmlBodyWidget,
            DivWidget,
            ImageWidget,
            HeadlineWidget,
            ParagraphWidget,
            ButtonWidget,
            CheckBoxWidget,
            SliderWidget,
            ProgressBarWidget,
            InputWidget,
            ChoiceBoxWidget
        ));
    }
}