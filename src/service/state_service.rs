use bevy::prelude::*;
use crate::{BindToID, CurrentWidgetState, IgnoreParentState, UIGenID, UIWidgetState};

pub struct StateService;

impl Plugin for StateService {
    fn build(&self, app: &mut App) {
        app.register_type::<Pickable>();
        app.add_systems(Update, update_widget_states);
        app.add_systems(Update, (
            internal_state_check.run_if(resource_changed::<CurrentWidgetState>), 
            handle_tab_focus
        ));
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
    mut inner_query: Query<(&BindToID, &mut UIWidgetState), (Without<UIGenID>, Without<IgnoreParentState>)>,
) {
    for (id, state) in main_query.iter() {
        for (bind_to, mut inner_state) in inner_query.iter_mut() {
            if bind_to.0 != id.0 {
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
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<UIGenID>>
) {
    for (mut state, gen_id) in query.iter_mut() {
        if gen_id.0 == current_state_element.widget_id {
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
    keyboard: Res<ButtonInput<KeyCode>>
) {
    let mut sorted_ui_elements: Vec<_> = query.iter_mut().collect();
    sorted_ui_elements.sort_by_key(|(_, _, id)| id.0);
    
    let mut any_focused = false;
    if keyboard.just_pressed(KeyCode::Tab) {
        let len = sorted_ui_elements.len();
        if len == 0 {
            return;
        }

        for i in 0..len {
            if sorted_ui_elements[i].1.focused {
                sorted_ui_elements[i].1.focused = false;

                let next = (i + 1) % len;

                if let Some(&mut (_, _, _)) = sorted_ui_elements.get_mut(next) {
                    sorted_ui_elements[next].1.focused = true;
                    any_focused = true;
                }

                break;
            }
        }

        if !any_focused && len > 0 {
            sorted_ui_elements[0].1.focused = true;
        }
    }
}