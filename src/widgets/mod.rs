use bevy::prelude::*;
use crate::widgets::button::{ButtonWidget};
use crate::widgets::containers::DivWidget;
use crate::widgets::input::{InputCap, InputType, InputWidget};
use crate::widgets::slider::SliderWidget;
use crate::global::{UiGenID, UiElementState};
use crate::widgets::check_box::CheckBoxWidget;
use crate::styles::types::{ButtonStyle, DivStyle, CheckBoxStyle};

pub mod containers;
pub mod button;
pub mod input;
pub mod slider;
pub mod check_box;
mod choice_box;

/// ===============================================
///                 Div
/// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, DivStyle)]
pub struct DivContainer;

/// ===============================================
///                 Slider
/// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState)]
pub struct Slider {
    pub value: i32,
    pub step: i32,
    pub min: i32,
    pub max: i32,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: 0,
            max: 100,
        }
    }
}

/// ===============================================
///                 Button
/// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, ButtonStyle)]
pub struct Button(pub String);

impl Default for Button {
    fn default() -> Self {
        Self("Button".to_string())
    }
}

/// ===============================================
///                 InputField
/// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState)]
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

/// ===============================================
///                 CheckBox
/// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, CheckBoxStyle)]
pub struct CheckBox {
    pub label: String,
    pub checked: bool
}

impl Default for CheckBox {
    fn default() -> Self {
        Self {
            checked: false,
            label: String::from("CheckBox"),
        }
    }
}

/// ===============================================
///                 ChoiceBox
/// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState)]
pub struct ChoiceBox {
    pub value: ChoiceOption,
    pub options: Vec<ChoiceOption>
}

impl Default for ChoiceBox {
    fn default() -> Self {
        Self {
            value: ChoiceOption { selected: true, ..default() },
            options: vec![ChoiceOption { selected: true, ..default() }]
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct ChoiceOption {
    pub label: String,
    pub internal_val: String,
    pub icon: Option<Handle<Image>>,
    pub selected: bool
}

impl Default for ChoiceOption {
    fn default() -> Self {
        Self {
            label: String::from("Option 1"),
            internal_val: String::from("option1"),
            icon: None,
            selected: false,
        }
    }
}

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DivContainer>();
        app.register_type::<CheckBox>();
        app.register_type::<Button>();
        app.register_type::<InputField>();
        app.register_type::<Slider>();
        app.add_plugins((
            DivWidget,
            ButtonWidget,
            InputWidget,
            SliderWidget,
            CheckBoxWidget
        ));
    }
}

