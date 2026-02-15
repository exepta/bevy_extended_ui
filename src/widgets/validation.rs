use bevy::prelude::*;
use regex::Regex;

use crate::widgets::{
    Button, CheckBox, Form, FormValidationMode, InputValue, RadioButton, SwitchButton,
    ToggleButton, UIWidgetState, ValidationRules,
};

/// Evaluates a widget's validation state for the given rules and values.
pub(crate) fn evaluate_validation_state(
    rules: &ValidationRules,
    state: &UIWidgetState,
    input_value: Option<&InputValue>,
    button: Option<&Button>,
    checkbox: Option<&CheckBox>,
    radio: Option<&RadioButton>,
    toggle: Option<&ToggleButton>,
    switch: Option<&SwitchButton>,
) -> bool {
    let mut invalid = false;

    if rules.required {
        let satisfied = if let Some(value) = input_value {
            !value.0.trim().is_empty()
        } else if button.is_some()
            || checkbox.is_some()
            || radio.is_some()
            || toggle.is_some()
            || switch.is_some()
        {
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

    invalid
}

/// Updates widget validation state based on configured rules and input values.
pub fn update_validation_states(
    parent_q: Query<&ChildOf>,
    form_q: Query<&Form>,
    mut query: Query<(
        Entity,
        Ref<ValidationRules>,
        Mut<UIWidgetState>,
        Option<Ref<InputValue>>,
        Option<&Button>,
        Option<&CheckBox>,
        Option<&RadioButton>,
        Option<&ToggleButton>,
        Option<&SwitchButton>,
    )>,
) {
    for (entity, rules, mut state, input_value, button, checkbox, radio, toggle, switch) in
        query.iter_mut()
    {
        let Some(mode) = find_ancestor_form_validation_mode(entity, &parent_q, &form_q) else {
            // Validation only applies inside a form.
            if state.invalid {
                state.invalid = false;
            }
            continue;
        };

        if mode == FormValidationMode::Send {
            // Strict submit-only mode: never auto-validate on state/input changes.
            continue;
        }

        let rules_added = rules.is_added();
        let state_changed = state.is_changed();
        let input_changed = input_value
            .as_ref()
            .map(|value| value.is_added() || value.is_changed())
            .unwrap_or(false);

        let should_validate = match mode {
            FormValidationMode::Interact => input_changed,
            FormValidationMode::Always => rules_added || state_changed || input_changed,
            FormValidationMode::Send => false,
        };

        if !should_validate {
            continue;
        }

        let input_value_ref = input_value.as_ref().map(|value| &**value);
        let invalid = evaluate_validation_state(
            &rules,
            &state,
            input_value_ref,
            button,
            checkbox,
            radio,
            toggle,
            switch,
        );

        if state.invalid != invalid {
            state.invalid = invalid;
        }
    }
}

/// Finds the nearest ancestor form and returns its validation mode.
fn find_ancestor_form_validation_mode(
    entity: Entity,
    parent_q: &Query<&ChildOf>,
    form_q: &Query<&Form>,
) -> Option<FormValidationMode> {
    let mut current = entity;

    while let Ok(parent) = parent_q.get(current) {
        let parent_entity = parent.parent();
        if let Ok(form) = form_q.get(parent_entity) {
            return Some(form.validate_mode.clone());
        }
        current = parent_entity;
    }

    None
}
