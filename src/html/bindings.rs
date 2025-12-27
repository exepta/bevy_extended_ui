use bevy::log::warn;
use bevy::prelude::*;
use crate::html::{HtmlChange, HtmlClick, HtmlEvent, HtmlEventBindings, HtmlEventObject, HtmlFunctionRegistry, HtmlInit, HtmlMouseOut, HtmlMouseOver};
use crate::widgets::{CheckBox, FieldSelectionMulti, FieldSelectionSingle, UIWidgetState};

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
        app.add_systems(Update, emit_html_init_events);
        app.add_observer(on_html_init);

        // observer (change)
        app.add_systems(Update, emit_checkbox_change);
        app.add_systems(Update, emit_field_set_change);
        app.add_observer(on_html_change);
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
    q_bindings: Query<(Entity, Option<&HtmlEventBindings>), Added<HtmlEventBindings>>,
) {
    for (entity, bindings) in q_bindings.iter() {
        if let Some(bindings) = bindings {
            if bindings.oninit.is_some() {
                commands.trigger(HtmlInit { entity });
            }
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

/// FieldSet
pub(crate) fn emit_field_set_change(
    mut commands: Commands,
    query: Query<
        (Entity, &HtmlEventBindings),
        (
            Or<(
                Changed<FieldSelectionSingle>,
                Changed<FieldSelectionMulti>,
            )>,
        ),
    >,
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