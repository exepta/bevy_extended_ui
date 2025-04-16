use bevy::prelude::*;
use crate::global::{UiElementState, UiGenID};
use crate::resources::CurrentElementSelected;

pub struct StateService;

impl Plugin for StateService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_state_check
            .run_if(resource_changed::<CurrentElementSelected>));
    }
}

fn internal_state_check(
    mut current_state_element: ResMut<CurrentElementSelected>,
    mut query: Query<(&mut UiElementState, &UiGenID), With<UiGenID>>
) {
    if current_state_element.0 == 0 {
        return;
    }

    for (mut state, gen_id) in query.iter_mut() {
        if gen_id.0 == current_state_element.0 {
            continue;
        }
        state.selected = false;
    }
}