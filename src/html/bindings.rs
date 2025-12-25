use bevy::log::warn;
use bevy::prelude::*;
use crate::html::{HtmlClick, HtmlEventBindings, HtmlFunctionRegistry, HtmlMouseOut, HtmlMouseOver};
use crate::widgets::UIWidgetState;
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
        commands.run_system_with(sys_id, entity);
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
        commands.run_system_with(sys_id, entity);
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
        commands.run_system_with(sys_id, entity);
    } else {
        warn!("onmouseout binding '{name}' not registered via #[html_fn(...)]");
    }
}