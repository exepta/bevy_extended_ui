#[cfg(feature = "theme-provider")]
mod theming_provider_example;
#[cfg(feature = "typed-values")]
mod typed_values_example;
#[cfg(feature = "widget-overview")]
mod widget_overview_example;

fn main() {
    #[cfg(any(
        all(feature = "theme-provider", feature = "widget-overview"),
        all(feature = "theme-provider", feature = "typed-values"),
        all(feature = "widget-overview", feature = "typed-values"),
    ))]
    compile_error!(
        "Select exactly one demo feature for local-wasm-examples: `theme-provider`, `widget-overview`, or `typed-values`."
    );

    #[cfg(not(any(feature = "theme-provider", feature = "widget-overview", feature = "typed-values")))]
    compile_error!(
        "Enable one demo feature for local-wasm-examples: `theme-provider`, `widget-overview`, or `typed-values`."
    );

    #[cfg(feature = "theme-provider")]
    theming_provider_example::run();

    #[cfg(feature = "widget-overview")]
    widget_overview_example::run();

    #[cfg(feature = "typed-values")]
    typed_values_example::run();
}
