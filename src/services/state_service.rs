use crate::CurrentWidgetState;
use crate::widgets::{BindToID, IgnoreParentState, UIGenID, UIWidgetState};
use bevy::prelude::*;

/// Plugin that manages widget focus and state propagation.
pub struct StateService;

impl Plugin for StateService {
    /// Registers widget state systems.
    fn build(&self, app: &mut App) {
        app.register_type::<Pickable>();
        app.add_systems(PostUpdate, update_widget_states);
        app.add_systems(
            Update,
            (
                internal_state_check.run_if(resource_changed::<CurrentWidgetState>),
                handle_tab_focus,
                unfocus_disabled,
            ),
        );
    }
}

/// Synchronizes the widget state from parent UI elements to child elements linked via [`BindToID`].
///
/// This system propagates UI states such as `hovered`, `focused`, `readonly`, `disabled`, and `checked`
/// from widgets that have a [`UIGenID`] to other UI elements bound to the same ID.
///
/// # Parameters
/// - `main_query`: Retrieves all UI widgets with a [`UIGenID`] whose [`UIWidgetState`] has changed.
/// - `inner_query`: Finds all UI elements that are bound via [`BindToID`], excluding those with their
///   own `UIGenID` or an explicit [`IgnoreParentState`].
///
/// # Purpose
/// Enables state propagation for compound widgets like checkbox containers, input groups, or radio button groups.
///
/// # Example
/// If a container with ID `#group` is focused, and an internal widget is bound to it, the inner widget
/// will also be marked as focused.
pub fn update_widget_states(
    main_query: Query<(&UIGenID, &UIWidgetState), (Changed<UIWidgetState>, With<UIGenID>)>,
    mut inner_query: Query<
        (&BindToID, &mut UIWidgetState),
        (Without<UIGenID>, Without<IgnoreParentState>),
    >,
) {
    for (id, state) in main_query.iter() {
        for (bind_to, mut inner_state) in inner_query.iter_mut() {
            if bind_to.0 != id.get() {
                continue;
            }

            inner_state.hovered = state.hovered;
            inner_state.focused = state.focused;
            inner_state.readonly = state.readonly;
            inner_state.disabled = state.disabled;
            inner_state.checked = state.checked;
        }
    }
}

/// Clears the `focused` state from all widgets except the currently focused one.
///
/// Ensures that only a single UI widget is marked as focused at any given time.
/// The focused widget ID is tracked in the [`CurrentWidgetState`] resource.
///
/// # Parameters
/// - `current_state_element`: The current global widget focus state.
/// - `query`: All UI widgets with a [`UIGenID`] and a mutable [`UIWidgetState`].
///
/// # Behavior
/// If the current widget ID is `0` (none), the system does nothing.
/// Otherwise, it clears `focused = false` on all widgets except the one with the matching ID.
fn internal_state_check(
    current_state_element: Res<CurrentWidgetState>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<UIGenID>>,
) {
    for (mut state, gen_id) in query.iter_mut() {
        if gen_id.get() == current_state_element.widget_id {
            continue;
        }
        state.focused = false;
    }
}

/// Handles keyboard tab navigation between focusable UI widgets.
///
/// This system detects when the Tab key is pressed and moves the focus to the next available widget,
/// based on sorted [`UIGenID`] values. Shift+Tab navigates in reverse.
///
/// # Parameters
/// - `keys`: The current keyboard input state.
/// - `mut current_state`: The global [`CurrentWidgetState`] resource tracking focused widget ID.
/// - `widgets`: A list of all focusable widgets that can receive focus.
///
/// # Behavior
/// - Widgets are sorted by `UIGenID.0`.
/// - Pressing `Tab` sets focus to the next widget in order.
/// - Pressing `Shift+Tab` sets focus to the previous widget in order.
/// - The focus wraps around if reaching the end or beginning.
///
/// # Requirements
/// All focusable widgets must have unique, non-zero `UIGenID` values.
///
/// # Example
/// When the user presses Tab while focused on widget `#2`, focus will move to widget `#3`.
fn handle_tab_focus(
    mut query: Query<(Entity, &mut UIWidgetState, &UIGenID)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_state: ResMut<CurrentWidgetState>,
) {
    if !keyboard.just_pressed(KeyCode::Tab) {
        return;
    }
    let reverse = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let mut elems: Vec<_> = query
        .iter_mut()
        .filter(|(_, state, _)| !state.disabled)
        .collect();

    elems.sort_by_key(|(_, _, id)| id.get());

    let len = elems.len();
    if len == 0 {
        return;
    }

    let mut focused_idx = None;
    for (i, (_, state, _)) in elems.iter().enumerate() {
        if state.focused {
            focused_idx = Some(i);
            break;
        }
    }

    match focused_idx {
        Some(i) => {
            elems[i].1.focused = false;
            let next = if reverse {
                (i + len - 1) % len
            } else {
                (i + 1) % len
            };
            elems[next].1.focused = true;
            current_state.widget_id = elems[next].2.get();
        }
        None => {
            let next = if reverse { len - 1 } else { 0 };
            elems[next].1.focused = true;
            current_state.widget_id = elems[next].2.get();
        }
    }
}

/// Clears focus from widgets that became disabled.
fn unfocus_disabled(mut q: Query<&mut UIWidgetState, Changed<UIWidgetState>>) {
    for mut s in &mut q {
        if s.disabled && s.focused {
            s.focused = false;
        }
    }
}
