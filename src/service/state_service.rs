use bevy::prelude::*;
use crate::{BindToID, UIGenID, UIWidgetState};

pub struct StateService;

impl Plugin for StateService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_widget_states);
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
        }
    }
}