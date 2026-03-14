mod theming_provider_example;
mod widget_overview_example;

fn main() {
    configure_linux_window_backend();
    widget_overview_example::run();
}

#[cfg(target_os = "linux")]
fn configure_linux_window_backend() {
    let backend_already_set = std::env::var_os("WINIT_UNIX_BACKEND").is_some();
    let has_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let has_x11 = std::env::var_os("DISPLAY").is_some();

    if backend_already_set || !has_wayland || !has_x11 {
        return;
    }

    // SAFETY: this runs once at process startup before Bevy spawns worker threads.
    unsafe {
        std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    }
    eprintln!(
        "bevy_extended_ui local-example: forcing WINIT_UNIX_BACKEND=x11 for linux stability."
    );
}

#[cfg(not(target_os = "linux"))]
fn configure_linux_window_backend() {}
