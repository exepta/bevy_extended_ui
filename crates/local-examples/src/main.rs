mod theming_provider_example;
mod typed_values_example;
mod widget_overview_example;

fn main() {
    configure_linux_window_backend();

    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "local-examples".to_string());

    match args.next().as_deref() {
        None => widget_overview_example::run(),
        Some("--help" | "-h" | "help") => print_usage(&program),
        Some(selection) if run_example(selection) => {}
        Some(selection) => {
            eprintln!("Unknown example: {selection}\n");
            print_usage(&program);
            std::process::exit(2);
        }
    }
}

fn run_example(selection: &str) -> bool {
    match selection {
        "widget_overview_example" | "widget-overview" | "widget" => {
            widget_overview_example::run();
            true
        }
        "theming_provider_example" | "theming-provider" | "theme" => {
            theming_provider_example::run();
            true
        }
        "typed_values_example" | "typed-values" | "typed" => {
            typed_values_example::run();
            true
        }
        _ => false,
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage: {program} [widget-overview|theming-provider|typed-values]");
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
