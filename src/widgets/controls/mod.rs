use crate::widgets::controls::button::ButtonWidget;
use crate::widgets::controls::check_box::CheckBoxWidget;
use crate::widgets::controls::choice_box::ChoiceBoxWidget;
use crate::widgets::controls::fieldset::FieldSetWidget;
use crate::widgets::controls::input::InputWidget;
use crate::widgets::controls::radio_button::RadioButtonWidget;
use crate::widgets::controls::slider::SliderWidget;
use bevy::prelude::*;

pub mod button;
pub mod check_box;
pub mod choice_box;
pub mod fieldset;
pub mod input;
pub mod radio_button;
pub mod slider;

pub struct ExtendedControlWidgets;

impl Plugin for ExtendedControlWidgets {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ButtonWidget,
            CheckBoxWidget,
            ChoiceBoxWidget,
            FieldSetWidget,
            InputWidget,
            RadioButtonWidget,
            SliderWidget,
        ));
    }
}
