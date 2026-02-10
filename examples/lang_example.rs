use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::lang::UILang;
use bevy_extended_ui::UiLangVariables;
use std::env;
use bevy_extended_ui::registry::UiRegistry;

/// Tracks whether the resolved language was logged.
#[derive(Resource, Default)]
struct LangLogged(bool);

/// Runs the language example app.
fn main() {
    let mut app = make_app("Debug Html UI - language");

    app.init_resource::<LangLogged>();
    app.add_systems(Startup, (set_lang, set_placeholders, load_ui).chain());
    app.add_systems(Update, log_lang_once);

    app.run();
}

/// Sets the selected UI language for the example.
fn set_lang(mut ui_lang: ResMut<UILang>) {
    // Overrides the system language when <html lang="auto"> or missing.
    // For translation files see assets/lang (enable "fluent" or "properties-lang").
    ui_lang.set_selected(Some("de"));
}

/// Populates localization placeholders with runtime values.
fn set_placeholders(mut vars: ResMut<UiLangVariables>) {
    let name = system_username().unwrap_or_else(|| "Player".to_string());
    vars.set("player_name", name);
}

/// Attempts to read the current username from the environment.
fn system_username() -> Option<String> {
    const KEYS: [&str; 3] = ["USER", "LOGNAME", "USERNAME"];
    for key in KEYS {
        if let Ok(value) = env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Loads the localized UI HTML asset.
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/lang.html");
    reg.add_and_use("lang-example".to_string(), HtmlSource::from_handle(handle));
}

/// Logs the resolved language once after startup.
fn log_lang_once(mut logged: ResMut<LangLogged>, ui_lang: Res<UILang>) {
    if !logged.0 {
        let resolved = ui_lang.resolved().unwrap_or("none");
        info!("UILang resolved: {resolved}");
        logged.0 = true;
    }
}
