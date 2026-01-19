use bevy::prelude::*;
use regex::Regex;

use crate::widgets::{
    Button, CheckBox, InputValue, RadioButton, SwitchButton, ToggleButton, UIWidgetState,
    ValidationRules,
};

pub fn update_validation_states(
    mut query: Query<
        (
            &ValidationRules,
            &mut UIWidgetState,
            Option<&InputValue>,
            Option<&Button>,
            Option<&CheckBox>,
            Option<&RadioButton>,
            Option<&ToggleButton>,
            Option<&SwitchButton>,
        ),
    >,
) {
    for (rules, mut state, input_value, button, checkbox, radio, toggle, switch) in query.iter_mut()
    {
        let mut invalid = false;

        if rules.required {
            let satisfied = if let Some(value) = input_value {
                !value.0.trim().is_empty()
            } else if button.is_some() || checkbox.is_some() || radio.is_some() || toggle.is_some() || switch.is_some() {
                state.checked
            } else {
                true
            };

            if !satisfied {
                invalid = true;
            }
        }

        if let Some(value) = input_value {
            let len = value.0.chars().count();

            if let Some(min) = rules.min_length {
                if len < min {
                    invalid = true;
                }
            }

            if let Some(max) = rules.max_length {
                if len > max {
                    invalid = true;
                }
            }

            if let Some(pattern) = &rules.pattern {
                if !value.0.is_empty() {
                    if let Ok(regex) = Regex::new(pattern) {
                        if !regex.is_match(&value.0) {
                            invalid = true;
                        }
                    } else {
                        invalid = true;
                    }
                }
            }
        }

        if state.invalid != invalid {
            state.invalid = invalid;
        }
    }
}