mod button;
mod div;
mod check_box;
mod slider;
mod input;
mod choice_box;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use bevy::prelude::*;
use crate::{UIGenID, UIWidgetState};
use crate::styling::IconPlace;
use crate::widgets::button::ButtonWidget;
use crate::widgets::check_box::CheckBoxWidget;
use crate::widgets::choice_box::ChoiceBoxWidget;
use crate::widgets::div::DivWidget;
use crate::widgets::input::InputWidget;
use crate::widgets::slider::SliderWidget;

static BUTTON_COUNT: AtomicUsize = AtomicUsize::new(1);
static CHECK_BOX_COUNT: AtomicUsize = AtomicUsize::new(1);
static DIV_COUNT: AtomicUsize = AtomicUsize::new(1);
static SLIDER_COUNT: AtomicUsize = AtomicUsize::new(1);
static INPUT_COUNT: AtomicUsize = AtomicUsize::new(1);
static CHOICE_BOX_COUNT: AtomicUsize = AtomicUsize::new(1);

#[derive(Component, Default)]
pub struct Widget;

// ===============================================
//                       Div
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Div(usize);

impl Default for Div {
    fn default() -> Self {
        Self(DIV_COUNT.fetch_add(1, Relaxed))
    }
}

// ===============================================
//                     Button
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Button {
    pub w_count: usize,
    pub text: String,
    pub icon_place: IconPlace,
    pub icon_path: Option<String>,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            w_count: BUTTON_COUNT.fetch_add(1, Relaxed),
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
    pub w_count: usize,
    pub label: String,
    pub icon_path: Option<String>,
}

impl Default for CheckBox {
    fn default() -> Self {
        Self {
            w_count: CHECK_BOX_COUNT.fetch_add(1, Relaxed),
            label: String::from("label"),
            icon_path: Some(String::from("icons/check-mark.png")),
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
    pub w_count: usize,
    pub value: i32,
    pub step: i32,
    pub min: i32,
    pub max: i32
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            w_count: SLIDER_COUNT.fetch_add(1, Relaxed),
            value: 0,
            step: 1,
            min: 0,
            max: 100,
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
        Self {
            w_count: INPUT_COUNT.fetch_add(1, Relaxed),
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
#[require(UIGenID, UIWidgetState, Widget)]
pub struct ChoiceBox {
    pub w_count: usize,
    pub value: ChoiceOption,
    pub options: Vec<ChoiceOption>
}

impl Default for ChoiceBox {
    fn default() -> Self {
        Self {
            w_count: CHOICE_BOX_COUNT.fetch_add(1, Relaxed),
            value: ChoiceOption::default(),
            options: vec![ChoiceOption::default()]
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

pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Div>();
        app.register_type::<Button>();
        app.register_type::<CheckBox>();
        app.register_type::<Slider>();
        app.register_type::<InputField>();
        app.register_type::<ChoiceBox>();
        app.add_plugins((
            DivWidget, 
            ButtonWidget,
            CheckBoxWidget,
            SliderWidget,
            InputWidget,
            ChoiceBoxWidget
        ));
    }
}