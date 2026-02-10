use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::lang::UILang;
use bevy_extended_ui::registry::UiRegistry;

#[derive(Resource, Default)]
struct LangLogged(bool);

fn main() {
    let mut app = make_app("Debug Html UI - language");

    app.init_resource::<LangLogged>();
    app.add_systems(Startup, (set_lang, load_ui).chain());
    app.add_systems(Update, log_lang_once);

    app.run();
}

fn set_lang(mut ui_lang: ResMut<UILang>) {
    // Overrides the system language when <html lang="auto"> or missing.
    // For translation files see assets/lang (enable "fluent" or "properties-lang").
    ui_lang.set_selected(Some("de"));
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/lang.html");
    reg.add_and_use("lang-example".to_string(), HtmlSource::from_handle(handle));
}

fn log_lang_once(mut logged: ResMut<LangLogged>, ui_lang: Res<UILang>) {
    if !logged.0 {
        let resolved = ui_lang.resolved().unwrap_or("none");
        info!("UILang resolved: {resolved}");
        logged.0 = true;
    }
}
