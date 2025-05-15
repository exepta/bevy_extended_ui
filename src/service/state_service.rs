use bevy::prelude::*;
use crate::{BindToID, CurrentWidgetState, UIGenID, UIWidgetState};

pub struct StateService;

impl Plugin for StateService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_widget_states);
        app.add_systems(Update, internal_state_check.run_if(resource_changed::<CurrentWidgetState>));
    }
}

pub fn update_widget_states(
    main_query: Query<(&UIGenID, &UIWidgetState), (Changed<UIWidgetState>, With<UIGenID>)>,
    mut inner_query: Query<(&BindToID, &mut UIWidgetState), Without<UIGenID>>,
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