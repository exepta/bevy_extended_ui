use bevy::log::warn;
use bevy::prelude::*;
use crate::html::*;
use crate::widgets::{CheckBox, FieldSelectionMulti, FieldSelectionSingle, InputValue, UIWidgetState};

pub struct HtmlEventBindingsPlugin;

impl Plugin for HtmlEventBindingsPlugin {
    fn build(&self, app: &mut App) {
        // observer (click)
        app.add_observer(emit_html_click_events);
        app.add_observer(on_html_click);

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
        app.add_systems(Update, emit_choice_box_change.in_set(HtmlSystemSet::Bindings));
        app.add_systems(Update, emit_field_set_change.in_set(HtmlSystemSet::Bindings));
        app.add_systems(Update, emit_input_change.in_set(HtmlSystemSet::Bindings));
        app.add_systems(Update, emit_slider_change.in_set(HtmlSystemSet::Bindings));
        app.add_observer(on_html_change);

        // observer (focus)
        app.add_systems(Update, emit_html_focus_events.in_set(HtmlSystemSet::Bindings));
        app.add_observer(on_html_focus);
    }
}

// =================================================
//                        Click
// =================================================

pub(crate) fn emit_html_click_events(
    ev: On<Pointer<Click>>,
    mut commands: Commands,
    q_bindings: Query<(&HtmlEventBindings, Option<&UIWidgetState>)>,
) {
    let entity = ev.event().entity;

    let Ok((bindings, state_opt)) = q_bindings.get(entity) else { return };
    if let Some(state) = state_opt {
        if state.disabled { return; }
    }
    if bindings.onclick.is_some() {
        commands.trigger(HtmlClick { entity });
    }
}

pub(crate) fn on_html_click(
    click: On<HtmlClick>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = click.entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };
    let Some(name) = bindings.onclick.as_deref() else { return };

    if let Some(&sys_id) = reg.click.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity, object: HtmlEventObject::Click(HtmlClick { entity: entity.clone()}) });
    } else {
        warn!("onclick binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Over
// =================================================

pub(crate) fn emit_html_mouse_over_events(
    ev: On<Pointer<Over>>,
    mut commands: Commands,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = ev.event().entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };
    if bindings.onmouseover.is_some() {
        commands.trigger(HtmlMouseOver { entity });
    }
}

pub(crate) fn on_html_mouse_over(
    over: On<HtmlMouseOver>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = over.entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };
    let Some(name) = bindings.onmouseover.as_deref() else { return };

    if let Some(&sys_id) = reg.over.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity, object: HtmlEventObject::MouseOver(HtmlMouseOver { entity: entity.clone()}) });
    } else {
        warn!("onmouseover binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Out
// =================================================

pub(crate) fn emit_html_mouse_out_events(
    ev: On<Pointer<Out>>,
    mut commands: Commands,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = ev.event().entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };
    if bindings.onmouseout.is_some() {
        commands.trigger(HtmlMouseOut { entity });
    }
}

pub(crate) fn on_html_mouse_out(
    out: On<HtmlMouseOut>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = out.entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };
    let Some(name) = bindings.onmouseout.as_deref() else { return };

    if let Some(&sys_id) = reg.out.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity, object: HtmlEventObject::MouseOut(HtmlMouseOut { entity: entity.clone()}) });
    } else {
        warn!("onmouseout binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Init
// =================================================

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

fn track_html_init_visibility(
    mut events: MessageReader<HtmlAllWidgetsVisible>,
    mut pending: ResMut<HtmlInitDelay>,
) {
    if events.read().next().is_some() {
        pending.0 = Some(10);
    }
}

fn advance_html_init_delay(mut pending: ResMut<HtmlInitDelay>) {
    if let Some(steps) = pending.0.as_mut() {
        if *steps > 0 {
            *steps -= 1;
        }
    }
}

pub(crate) fn on_html_init(
    init: On<HtmlInit>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = init.entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };

    let Some(name) = bindings.oninit.as_deref() else { return };

    if let Some(&sys_id) = reg.init.get(name) {

        commands.run_system_with(sys_id, HtmlEvent { entity, object: HtmlEventObject::Init(HtmlInit { entity: entity.clone()}) });
    } else {
        warn!("oninit binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Change
// =================================================

/// CheckBox
pub(crate) fn emit_checkbox_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<CheckBox>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity);
    }
}

/// ChoiceBox
pub(crate) fn emit_choice_box_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<ChoiceBox>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity);
    }
}

/// FieldSet
pub(crate) fn emit_field_set_change(
    mut commands: Commands,
    query: Query<
        (Entity, &HtmlEventBindings),
        Or<(
            Changed<FieldSelectionSingle>,
            Changed<FieldSelectionMulti>,
        )>,

    >,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity);
    }
}

/// Slider
pub(crate) fn emit_slider_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<Slider>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity);
    }
}

pub(crate) fn emit_input_change(
    mut commands: Commands,
    query: Query<(Entity, &HtmlEventBindings), Changed<InputValue>>,
) {
    for (entity, binding) in &query {
        emit_change_if_bound(&mut commands, binding, entity);
    }
}

fn emit_change_if_bound(
    commands: &mut Commands,
    bindings: &HtmlEventBindings,
    entity: Entity,
) {
    if bindings.onchange.is_some() {
        commands.trigger(HtmlChange { entity });
    }
}

pub(crate) fn on_html_change(
    init: On<HtmlChange>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = init.entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };

    let Some(name) = bindings.onchange.as_deref() else { return };

    if let Some(&sys_id) = reg.change.get(name) {

        commands.run_system_with(sys_id, HtmlEvent { entity, object: HtmlEventObject::Change(HtmlChange { entity: entity.clone()}) });
    } else {
        warn!("onchange binding '{name}' not registered via #[html_fn(...)]");
    }
}

// =================================================
//                        Focus
// =================================================

pub(crate) fn emit_html_focus_events(
    mut commands: Commands,
    mut query: Query<
        (Entity, &HtmlEventBindings, &UIWidgetState, Option<&mut HtmlFocusState>),
        Changed<UIWidgetState>,
    >,
) {
    for (entity, bindings, state, focus_state) in &mut query {
        let should_track = bindings.onfoucs.is_some();
        let was_focused = focus_state.as_ref().map(|s| s.focused).unwrap_or(false);

        if let Some(mut focus_state) = focus_state {
            focus_state.focused = state.focused;
        } else if should_track {
            commands.entity(entity).insert(HtmlFocusState { focused: state.focused });
        }

        if !should_track || state.disabled {
            continue;
        }

        if state.focused && !was_focused {
            commands.trigger(HtmlFocus { entity });
        }
    }
}

pub(crate) fn on_html_focus(
    focus: On<HtmlFocus>,
    mut commands: Commands,
    reg: Res<HtmlFunctionRegistry>,
    q_bindings: Query<&HtmlEventBindings>,
) {
    let entity = focus.entity;

    let Ok(bindings) = q_bindings.get(entity) else { return };
    let Some(name) = bindings.onfoucs.as_deref() else { return };

    if let Some(&sys_id) = reg.focus.get(name) {
        commands.run_system_with(sys_id, HtmlEvent { entity, object: HtmlEventObject::Focus(HtmlFocus { entity: entity.clone()}) });
    } else {
        warn!("onfoucs binding '{name}' not registered via #[html_fn(...)]");
    }
}
