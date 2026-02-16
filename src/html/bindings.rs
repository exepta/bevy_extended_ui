use crate::CurrentWidgetState;
use crate::html::*;
use crate::widgets::{
    BindToID, Button, ButtonType, CheckBox, ChoiceBox, ColorPicker, FieldSelectionMulti,
    FieldSelectionSingle, Form, FormValidationMode, InputField, InputValue, RadioButton, Scrollbar,
    Slider, SwitchButton, ToggleButton, UIGenID, UIWidgetState, ValidationRules,
    evaluate_validation_state,
};
use bevy::log::warn;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, RelativeCursorPosition, ScrollPosition};
use std::collections::{HashMap, HashSet};

/// Component tracking focus state for HTML widgets.
#[derive(Component, Default, Clone, Copy)]
pub(crate) struct HtmlFocusTracker {
    focused: bool,
}

/// Resource tracking scroll positions for HTML widgets.
#[derive(Resource, Default)]
pub(crate) struct HtmlScrollTracker {
    positions: HashMap<usize, Vec2>,
    scrollbar_values: HashMap<Entity, f32>,
}

/// Plugin that wires HTML event bindings into Bevy observers and systems.
pub struct HtmlEventBindingsPlugin;

impl Plugin for HtmlEventBindingsPlugin {
    /// Registers observers and systems for HTML events.
    fn build(&self, app: &mut App) {
        app.init_resource::<HtmlScrollTracker>();

        // observer (click)
        app.add_observer(emit_html_click_events);
        app.add_observer(on_html_click);
        app.add_observer(emit_html_submit_events);
        app.add_observer(on_html_submit);

        // observer (over)
        app.add_observer(emit_html_mouse_over_events);
        app.add_observer(on_html_mouse_over);

        // observer (out)
        app.add_observer(emit_html_mouse_out_events);
        app.add_observer(on_html_mouse_out);

        // observer (init)
        app.add_systems(
            Update,
            (
                track_html_init_visibility,
                advance_html_init_delay.after(track_html_init_visibility),
            )
                .in_set(HtmlSystemSet::Bindings),
        );
        app.add_systems(Last, emit_html_init_events);
        app.add_observer(on_html_init);

        // observer (change)
        app.add_systems(Update, emit_checkbox_change.in_set(HtmlSystemSet::Bindings));
        app.add_systems(
            Update,
            emit_choice_box_change.in_set(HtmlSystemSet::Bindings),
        );
        app.add_systems(
            Update,
            emit_field_set_change.in_set(HtmlSystemSet::Bindings),
        );
        app.add_systems(Update, emit_input_change.in_set(HtmlSystemSet::Bindings));
        app.add_systems(Update, emit_slider_change.in_set(HtmlSystemSet::Bindings));
        app.add_systems(
            Update,
            emit_color_picker_change.in_set(HtmlSystemSet::Bindings),
        );
        app.add_observer(on_html_change);

        // observer (focus)
        app.add_systems(
            Update,
            emit_html_focus_events.in_set(HtmlSystemSet::Bindings),
        );
        app.add_observer(on_html_focus);

        // observer (scroll)
        app.add_systems(
            Update,
            emit_html_scroll_events.in_set(HtmlSystemSet::Bindings),
        );
        app.add_systems(
            Update,
            emit_html_scrollbar_events.in_set(HtmlSystemSet::Bindings),
        );
        app.add_observer(on_html_scroll);

        // observer (keyboard)
        app.add_systems(
            Update,
            emit_html_key_down_events.in_set(HtmlSystemSet::Bindings),
        );
        app.add_systems(
            Update,
            emit_html_key_up_events.in_set(HtmlSystemSet::Bindings),
        );
        app.add_observer(on_html_key_down);
        app.add_observer(on_html_key_up);

        // observer (drag)
        app.add_observer(emit_html_drag_start_events);
        app.add_observer(on_html_drag_start);
        app.add_observer(emit_html_drag_events);
        app.add_observer(on_html_drag);
        app.add_observer(emit_html_drag_stop_events);
        app.add_observer(on_html_drag_stop);
    }
}

// =================================================
//                        Click
// =================================================

/// Emits click events for widgets with `onclick` bindings.
pub(crate) fn emit_html_click_events(
    ev: On<Pointer<Click>>,
    mut commands: Commands,
    q_bindings: Query<(
        &HtmlEventBindings,
        Option<&UIWidgetState>,
        Option<&RelativeCursorPosition>,
        Option<&ComputedNode>,
    )>,
) {
    let entity = ev.event().entity;

    let Ok((bindings, state_opt, rel_pos, node)) = q_bindings.get(entity) else {
        return;
    };
    if let Some(state) = state_opt {
        if state.disabled {
            return;
        }
    }
    if bindings.onclick.is_some() {
        let position = ev.pointer_location.position;
        let inner_position = rel_pos
            .and_then(|rel| rel.normalized)
            .map(|norm| {
                if let Some(node) = node {
                    Vec2::new(norm.x * node.size.x, norm.y * node.size.y)
                } else {
                    norm
                }
            })
            .unwrap_or(position);
        commands.trigger(HtmlClick {
            entity,
            position,
            inner_position,
        });
    }
}

/// Dispatches registered click handlers for HTML widgets.
pub(crate) fn on_html_click(
    click: On<HtmlClick>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = click.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.onclick.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.click_typed.get(name) {
        commands.run_system_with(sys_id, *click);
    } else if let Some(&sys_id) = reg.click.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onclick binding '{name}' not registered via #[html_fn(...)]");
    }
}

/// Emits submit events for form actions when submit buttons are clicked.
pub(crate) fn emit_html_submit_events(
    ev: On<Pointer<Click>>,
    mut commands: Commands,
    mut params: ParamSet<(
        Query<(&Button, Option<&UIWidgetState>)>,
        Query<(&Form, Option<&UIWidgetState>)>,
        Query<&ChildOf>,
        Query<&Children>,
        Query<(&InputField, &InputValue)>,
        Query<(
            &ValidationRules,
            &mut UIWidgetState,
            Option<&InputValue>,
            Option<&Button>,
            Option<&CheckBox>,
            Option<&RadioButton>,
            Option<&ToggleButton>,
            Option<&SwitchButton>,
        )>,
    )>,
) {
    let submitter = ev.event().entity;
    let (button_type, button_disabled) = {
        let button_q = params.p0();
        let Ok((button, button_state)) = button_q.get(submitter) else {
            return;
        };
        (
            button.button_type.clone(),
            button_state.map(|state| state.disabled).unwrap_or(false),
        )
    };

    if button_disabled {
        return;
    }

    let is_submit = matches!(button_type, ButtonType::Submit);
    if !is_submit {
        return;
    }

    let ancestor_chain = {
        let parent_q = params.p2();
        let mut current = submitter;
        let mut chain = Vec::new();
        while let Ok(parent) = parent_q.get(current) {
            let entity = parent.parent();
            chain.push(entity);
            current = entity;
        }
        chain
    };
    let form_entity = {
        let form_q = params.p1();
        let Some(form_entity) = ancestor_chain
            .into_iter()
            .find(|entity| form_q.get(*entity).is_ok())
        else {
            return;
        };
        form_entity
    };

    let (action, validate_mode) = {
        let form_q = params.p1();
        let Ok((form, form_state)) = form_q.get(form_entity) else {
            return;
        };
        if form_state.map(|state| state.disabled).unwrap_or(false) {
            return;
        }

        let Some(action) = form
            .action
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return;
        };
        (action.to_string(), form.validate_mode.clone())
    };

    let descendants = {
        let children_q = params.p3();
        let mut descendants = Vec::new();
        collect_descendants(form_entity, &children_q, &mut descendants);
        descendants
    };

    {
        let mut valid = true;
        let mut validation_q = params.p5();
        match validate_mode {
            FormValidationMode::Send => {
                for entity in &descendants {
                    if let Ok((
                        rules,
                        mut state,
                        input_value,
                        button,
                        checkbox,
                        radio,
                        toggle,
                        switch,
                    )) = validation_q.get_mut(*entity)
                    {
                        let invalid = evaluate_validation_state(
                            rules,
                            &state,
                            input_value,
                            button,
                            checkbox,
                            radio,
                            toggle,
                            switch,
                        );
                        if state.invalid != invalid {
                            state.invalid = invalid;
                        }
                        if invalid {
                            valid = false;
                        }
                    }
                }
            }
            FormValidationMode::Always | FormValidationMode::Interact => {
                for entity in &descendants {
                    if let Ok((
                        _rules,
                        state,
                        _input_value,
                        _button,
                        _checkbox,
                        _radio,
                        _toggle,
                        _switch,
                    )) = validation_q.get_mut(*entity)
                    {
                        if state.invalid {
                            valid = false;
                        }
                    }
                }
            }
        }
        if !valid {
            return;
        }
    }

    let mut data: HashMap<String, String> = HashMap::new();
    {
        let input_q = params.p4();
        for entity in descendants {
            if let Ok((input, value)) = input_q.get(entity) {
                let key = if input.name.trim().is_empty() {
                    format!("input_{}", input.entry)
                } else {
                    input.name.clone()
                };
                data.insert(key, value.0.clone());
            }
        }
    }

    commands.trigger(HtmlSubmit {
        entity: form_entity,
        submitter,
        action,
        data,
    });
}

/// Dispatches registered submit handlers for HTML form actions.
pub(crate) fn on_html_submit(
    submit: On<HtmlSubmit>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
) {
    let action = submit.action.as_str();

    if let Some(&sys_id) = reg.submit_typed.get(action) {
        commands.run_system_with(sys_id, submit.event().clone());
    } else if let Some(&sys_id) = reg.submit.get(action) {
        commands.run_system_with(
            sys_id,
            HtmlEvent {
                entity: submit.entity,
            },
        );
    } else {
        warn!("form action '{action}' not registered via #[html_fn(...)]");
    }
}

/// Collects all descendant entities of a root node.
fn collect_descendants(root: Entity, children_q: &Query<&Children>, out: &mut Vec<Entity>) {
    if let Ok(children) = children_q.get(root) {
        for child in children.iter() {
            out.push(child);
            collect_descendants(child, children_q, out);
        }
    }
}

// =================================================
//                        Over
// =================================================

/// Emits mouse-over events for widgets with `onmouseover` bindings.
pub(crate) fn emit_html_mouse_over_events(
    ev: On<Pointer<Over>>,
    mut commands: Commands,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = ev.event().entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    if bindings.onmouseover.is_some() {
        commands.trigger(HtmlMouseOver { entity });
    }
}

/// Dispatches registered mouse-over handlers for HTML widgets.
pub(crate) fn on_html_mouse_over(
    over: On<HtmlMouseOver>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = over.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.onmouseover.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.over_typed.get(name) {
        commands.run_system_with(sys_id, *over);
    } else if let Some(&sys_id) = reg.over.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onmouseover binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Out
// =================================================

/// Emits mouse-out events for widgets with `onmouseout` bindings.
pub(crate) fn emit_html_mouse_out_events(
    ev: On<Pointer<Out>>,
    mut commands: Commands,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = ev.event().entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    if bindings.onmouseout.is_some() {
        commands.trigger(HtmlMouseOut { entity });
    }
}

/// Dispatches registered mouse-out handlers for HTML widgets.
pub(crate) fn on_html_mouse_out(
    out: On<HtmlMouseOut>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = out.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.onmouseout.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.out_typed.get(name) {
        commands.run_system_with(sys_id, *out);
    } else if let Some(&sys_id) = reg.out.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onmouseout binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Init
// =================================================

/// Emits init events after widgets become visible.
pub(crate) fn emit_html_init_events(
    mut commands: Commands,
    mut pending: ResMut<HtmlInitDelay>,
    q_bindings: Query<(Entity, &HtmlEventBindings), Without<HtmlInitEmitted>>,
) {
    let Some(0) = pending.0 else { return };

    for (entity, bindings) in q_bindings.iter() {
        if bindings.oninit.is_some() {
            commands.trigger(HtmlInit { entity });
            commands.entity(entity).insert(HtmlInitEmitted);
        }
    }
    pending.0 = None;
}

/// Tracks visibility and marks widgets that are ready to emit init events.
fn track_html_init_visibility(
    mut events: MessageReader<HtmlAllWidgetsVisible>,
    mut pending: ResMut<HtmlInitDelay>,
) {
    if events.read().next().is_some() {
        pending.0 = Some(10);
    }
}

/// Advances the global init delay counter.
fn advance_html_init_delay(mut pending: ResMut<HtmlInitDelay>) {
    if let Some(steps) = pending.0.as_mut() {
        if *steps > 0 {
            *steps -= 1;
        }
    }
}

/// Dispatches registered init handlers for HTML widgets.
pub(crate) fn on_html_init(
    init: On<HtmlInit>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = init.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };

    let Some(name) = bindings.oninit.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.init_typed.get(name) {
        commands.run_system_with(sys_id, *init);
    } else if let Some(&sys_id) = reg.init.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("oninit binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Change
// =================================================

/// CheckBox
/// Emits change events for checkbox widgets.
pub(crate) fn emit_checkbox_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<CheckBox>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity, HtmlChangeAction::State);
    }
}

/// ChoiceBox
/// Emits change events for choice box widgets.
pub(crate) fn emit_choice_box_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<ChoiceBox>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity, HtmlChangeAction::State);
    }
}

/// FieldSet
/// Emits change events for field set widgets.
pub(crate) fn emit_field_set_change(
    mut commands: Commands,
    query: Query<
        (Entity, &HtmlEventBindings),
        Or<(Changed<FieldSelectionSingle>, Changed<FieldSelectionMulti>)>,
    >,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity, HtmlChangeAction::State);
    }
}

/// Slider
/// Emits change events for slider widgets.
pub(crate) fn emit_slider_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<Slider>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity, HtmlChangeAction::State);
    }
}

/// ColorPicker
/// Emits change events for color picker widgets.
pub(crate) fn emit_color_picker_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<ColorPicker>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity, HtmlChangeAction::State);
    }
}

/// Emits change events for input widgets.
pub(crate) fn emit_input_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<InputValue>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity, HtmlChangeAction::State);
    }
}

/// Emits a change event when a binding is present.
fn emit_change_if_bound(
    commands: &mut Commands,
    bindings: &HtmlEventBindings,
    entity: Entity,
    action: HtmlChangeAction,
) {
    if bindings.onchange.is_some() {
        commands.trigger(HtmlChange { entity, action });
    }
}

/// Dispatches registered change handlers for HTML widgets.
pub(crate) fn on_html_change(
    init: On<HtmlChange>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = init.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };

    let Some(name) = bindings.onchange.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.change_typed.get(name) {
        commands.run_system_with(sys_id, *init);
    } else if let Some(&sys_id) = reg.change.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onchange binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Focus
// =================================================

/// Emits focus events based on pointer focus changes.
pub(crate) fn emit_html_focus_events(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &HtmlEventBindings,
            &UIWidgetState,
            Option<&mut HtmlFocusTracker>,
        ),
        Changed<UIWidgetState>,
    >,
) {
    for (entity, bindings, state, focus_state) in &mut query {
        let should_track = bindings.onfoucs.is_some();
        let was_focused = focus_state.as_ref().map(|s| s.focused).unwrap_or(false);

        if let Some(mut focus_state) = focus_state {
            focus_state.focused = state.focused;
        } else if should_track {
            commands.entity(entity).insert(HtmlFocusTracker {
                focused: state.focused,
            });
        }

        if !should_track {
            continue;
        }

        if state.focused != was_focused {
            if state.focused && state.disabled {
                continue;
            }
            let focus_state = if state.focused {
                HtmlFocusState::Gained
            } else {
                HtmlFocusState::Lost
            };
            commands.trigger(HtmlFocus {
                entity,
                state: focus_state,
            });
        }
    }
}

/// Dispatches registered focus handlers for HTML widgets.
pub(crate) fn on_html_focus(
    focus: On<HtmlFocus>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = focus.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.onfoucs.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.focus_typed.get(name) {
        commands.run_system_with(sys_id, *focus);
    } else if let Some(&sys_id) = reg.focus.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onfoucs binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Scroll
// =================================================

/// Emits scroll events based on cursor wheel input.
pub(crate) fn emit_html_scroll_events(
    mut commands: Commands,
    scroll_q: Query<(&ScrollPosition, &BindToID), Changed<ScrollPosition>>,
    widget_q: Query<(Entity, &UIGenID, &HtmlEventBindings, Option<&UIWidgetState>)>,
    mut tracker: ResMut<HtmlScrollTracker>,
) {
    let mut bindings_by_id: HashMap<usize, (Entity, bool)> = HashMap::new();
    for (entity, id, bindings, state_opt) in &widget_q {
        if bindings.onscroll.is_some() {
            let disabled = state_opt.map(|s| s.disabled).unwrap_or(false);
            bindings_by_id.insert(id.get(), (entity, disabled));
        }
    }

    if bindings_by_id.is_empty() {
        return;
    }

    let mut fired: HashSet<Entity> = HashSet::new();
    for (scroll_pos, bind) in &scroll_q {
        let Some((entity, disabled)) = bindings_by_id.get(&bind.0) else {
            continue;
        };
        let current = Vec2::new(scroll_pos.x, scroll_pos.y);
        let last = tracker.positions.get(&bind.0).copied().unwrap_or(current);
        tracker.positions.insert(bind.0, current);
        if *disabled {
            continue;
        }
        if fired.insert(*entity) {
            commands.trigger(HtmlScroll {
                entity: *entity,
                delta: current - last,
                x: current.x,
                y: current.y,
            });
        }
    }
}

/// Emits scroll events based on scrollbar changes.
pub(crate) fn emit_html_scrollbar_events(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Scrollbar,
            &HtmlEventBindings,
            Option<&UIWidgetState>,
        ),
        Changed<Scrollbar>,
    >,
    mut tracker: ResMut<HtmlScrollTracker>,
) {
    for (entity, scroll, bindings, state_opt) in &query {
        if bindings.onscroll.is_none() {
            continue;
        }
        let last_value = tracker
            .scrollbar_values
            .get(&entity)
            .copied()
            .unwrap_or(scroll.value);
        tracker.scrollbar_values.insert(entity, scroll.value);

        if state_opt.map(|s| s.disabled).unwrap_or(false) {
            continue;
        }
        let (x, y, delta) = if scroll.vertical {
            (0.0, scroll.value, Vec2::new(0.0, scroll.value - last_value))
        } else {
            (scroll.value, 0.0, Vec2::new(scroll.value - last_value, 0.0))
        };

        commands.trigger(HtmlScroll {
            entity,
            delta,
            x,
            y,
        });
    }
}

/// Dispatches registered scroll handlers for HTML widgets.
pub(crate) fn on_html_scroll(
    scroll: On<HtmlScroll>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = scroll.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.onscroll.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.scroll_typed.get(name) {
        commands.run_system_with(sys_id, *scroll);
    } else if let Some(&sys_id) = reg.scroll.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onscroll binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                       Keyboard
// =================================================

fn find_keyboard_target_entity(
    current_widget_state: &CurrentWidgetState,
    q_bindings: &Query<(Entity, &UIGenID, &HtmlEventBindings, &UIWidgetState)>,
) -> Option<Entity> {
    if current_widget_state.widget_id != 0 {
        for (entity, id, _, _) in q_bindings {
            if id.get() == current_widget_state.widget_id {
                return Some(entity);
            }
        }
    }

    for (entity, _, _, state) in q_bindings {
        if state.focused {
            return Some(entity);
        }
    }

    None
}

/// Emits key-down events for the focused widget.
pub(crate) fn emit_html_key_down_events(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_widget_state: Res<CurrentWidgetState>,
    q_bindings: Query<(Entity, &UIGenID, &HtmlEventBindings, &UIWidgetState)>,
) {
    let keys: Vec<KeyCode> = keyboard.get_just_pressed().copied().collect();
    if keys.is_empty() {
        return;
    }

    let Some(entity) = find_keyboard_target_entity(&current_widget_state, &q_bindings) else {
        return;
    };
    let Ok((_, _, bindings, state)) = q_bindings.get(entity) else {
        return;
    };
    if state.disabled || bindings.onkeydown.is_none() {
        return;
    }

    for key in keys {
        commands.trigger(HtmlKeyDown { entity, key });
    }
}

/// Emits key-up events for the focused widget.
pub(crate) fn emit_html_key_up_events(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_widget_state: Res<CurrentWidgetState>,
    q_bindings: Query<(Entity, &UIGenID, &HtmlEventBindings, &UIWidgetState)>,
) {
    let keys: Vec<KeyCode> = keyboard.get_just_released().copied().collect();
    if keys.is_empty() {
        return;
    }

    let Some(entity) = find_keyboard_target_entity(&current_widget_state, &q_bindings) else {
        return;
    };
    let Ok((_, _, bindings, state)) = q_bindings.get(entity) else {
        return;
    };
    if state.disabled || bindings.onkeyup.is_none() {
        return;
    }

    for key in keys {
        commands.trigger(HtmlKeyUp { entity, key });
    }
}

/// Dispatches registered key-down handlers for HTML widgets.
pub(crate) fn on_html_key_down(
    keydown: On<HtmlKeyDown>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = keydown.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.onkeydown.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.keydown_typed.get(name) {
        commands.run_system_with(sys_id, *keydown);
    } else if let Some(&sys_id) = reg.keydown.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onkeydown binding '{name}' not registered via #[html_fn(...)]");
    }
}

/// Dispatches registered key-up handlers for HTML widgets.
pub(crate) fn on_html_key_up(
    keyup: On<HtmlKeyUp>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = keyup.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.onkeyup.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.keyup_typed.get(name) {
        commands.run_system_with(sys_id, *keyup);
    } else if let Some(&sys_id) = reg.keyup.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("onkeyup binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                         Drag
// =================================================

/// Emits drag-start events for widgets with drag bindings.
pub(crate) fn emit_html_drag_start_events(
    ev: On<Pointer<DragStart>>,
    mut commands: Commands,
    q_bindings: Query<(&HtmlEventBindings, Option<&UIWidgetState>)>,
) {
    let entity = ev.event().entity;

    let Ok((bindings, state_opt)) = q_bindings.get(entity) else {
        return;
    };
    if let Some(state) = state_opt {
        if state.disabled {
            return;
        }
    }
    if bindings.ondragstart.is_some() {
        commands.trigger(HtmlDragStart {
            entity,
            position: ev.pointer_location.position,
        });
    }
}

/// Dispatches registered drag-start handlers for HTML widgets.
pub(crate) fn on_html_drag_start(
    drag: On<HtmlDragStart>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = drag.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.ondragstart.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.dragstart_typed.get(name) {
        commands.run_system_with(sys_id, *drag);
    } else if let Some(&sys_id) = reg.dragstart.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("ondragstart binding '{name}' not registered via #[html_fn(...)]");
    }
}

/// Emits drag events for widgets with drag bindings.
pub(crate) fn emit_html_drag_events(
    ev: On<Pointer<Drag>>,
    mut commands: Commands,
    q_bindings: Query<(&HtmlEventBindings, Option<&UIWidgetState>)>,
) {
    let entity = ev.event().entity;

    let Ok((bindings, state_opt)) = q_bindings.get(entity) else {
        return;
    };
    if let Some(state) = state_opt {
        if state.disabled {
            return;
        }
    }
    if bindings.ondrag.is_some() {
        commands.trigger(HtmlDrag {
            entity,
            position: ev.pointer_location.position,
        });
    }
}

/// Dispatches registered drag handlers for HTML widgets.
pub(crate) fn on_html_drag(
    drag: On<HtmlDrag>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = drag.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.ondrag.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.drag_typed.get(name) {
        commands.run_system_with(sys_id, *drag);
    } else if let Some(&sys_id) = reg.drag.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("ondrag binding '{name}' not registered via #[html_fn(...)]");
    }
}

/// Emits drag-stop events for widgets with drag bindings.
pub(crate) fn emit_html_drag_stop_events(
    ev: On<Pointer<DragEnd>>,
    mut commands: Commands,
    q_bindings: Query<(&HtmlEventBindings, Option<&UIWidgetState>)>,
) {
    let entity = ev.event().entity;

    let Ok((bindings, state_opt)) = q_bindings.get(entity) else {
        return;
    };
    if let Some(state) = state_opt {
        if state.disabled {
            return;
        }
    }
    if bindings.ondragstop.is_some() {
        commands.trigger(HtmlDragStop {
            entity,
            position: ev.pointer_location.position,
        });
    }
}

/// Dispatches registered drag-stop handlers for HTML widgets.
pub(crate) fn on_html_drag_stop(
    drag: On<HtmlDragStop>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = drag.entity;

    let Ok(bindings) = q_bindings.get(entity) else {
        return;
    };
    let Some(name) = bindings.ondragstop.as_deref() else {
        return;
    };

    if let Some(&sys_id) = reg.dragstop_typed.get(name) {
        commands.run_system_with(sys_id, *drag);
    } else if let Some(&sys_id) = reg.dragstop.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity });
    } else {
        warn!("ondragstop binding '{name}' not registered via #[html_fn(...)]");
    }
}
