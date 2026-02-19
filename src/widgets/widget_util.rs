use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

pub fn wheel_delta_y(event: &MouseWheel, inv_scale_factor: f32) -> f32 {
    match event.unit {
        MouseScrollUnit::Line => {
            let line_delta = event.y;
            if line_delta.abs() > 10.0 {
                line_delta * inv_scale_factor
            } else {
                line_delta * 25.0
            }
        }
        MouseScrollUnit::Pixel => event.y * inv_scale_factor,
    }
}
