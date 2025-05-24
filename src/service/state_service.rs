use bevy::prelude::*;
use crate::{BindToID, CurrentWidgetState, IgnoreParentState, UIGenID, UIWidgetState};

pub struct StateService;

impl Plugin for StateService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_widget_states);
        app.add_systems(Update, (
            internal_state_check.run_if(resource_changed::<CurrentWidgetState>), 
            handle_tab_focus
        ));
    }
}

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

fn internal_state_check(
    current_state_element: Res<CurrentWidgetState>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<UIGenID>>
) {
    if current_state_element.widget_id == 0 {
        return;
    }

    for (mut state, gen_id) in query.iter_mut() {
        if gen_id.0 == current_state_element.widget_id {
            continue;
        }
        state.focused = false;
    }
}

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