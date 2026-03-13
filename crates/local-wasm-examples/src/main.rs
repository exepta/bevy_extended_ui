#[cfg(feature = "theme-provider")]
mod theming_provider_example;
#[cfg(feature = "widget-overview")]
mod widget_overview_example;

fn main() {
    #[cfg(all(feature = "theme-provider", feature = "widget-overview"))]
    compile_error!(
        "Select exactly one demo feature for local-wasm-examples: `theme-provider` or `widget-overview`."
    );

    #[cfg(not(any(feature = "theme-provider", feature = "widget-overview")))]
    compile_error!(
        "Enable one demo feature for local-wasm-examples: `theme-provider` or `widget-overview`."
    );

    #[cfg(feature = "theme-provider")]
    theming_provider_example::run();

    #[cfg(feature = "widget-overview")]
    widget_overview_example::run();
}
