use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

/// Handles `wheel_delta_axis` in the extended UI workflow.
fn wheel_delta_axis(value: f32, unit: MouseScrollUnit, inv_scale_factor: f32) -> f32 {
    match unit {
        MouseScrollUnit::Line => {
            if value.abs() > 10.0 {
                value * inv_scale_factor
            } else {
                value * 25.0
            }
        }
        MouseScrollUnit::Pixel => value * inv_scale_factor,
    }
}

/// Handles `wheel_delta_x` in the extended UI workflow.
///
/// # Examples
///
/// ```rust
/// // Call `wheel_delta_x` with values from your app state and world context.
/// ```
pub fn wheel_delta_x(event: &MouseWheel, inv_scale_factor: f32) -> f32 {
    wheel_delta_axis(event.x, event.unit, inv_scale_factor)
}

/// Handles `wheel_delta_y` in the extended UI workflow.
///
/// # Examples
///
/// ```rust
/// // Call `wheel_delta_y` with values from your app state and world context.
/// ```
pub fn wheel_delta_y(event: &MouseWheel, inv_scale_factor: f32) -> f32 {
    wheel_delta_axis(event.y, event.unit, inv_scale_factor)
}
