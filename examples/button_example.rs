use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::UIWidgetState;
use bevy_extended_ui_macros::html_fn;

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/button.html");
        reg.add_and_use("button_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}

#[html_fn("click_me_btn")]
fn click_me_btn(mut query: Query<(&mut UIWidgetState, &CssID, &mut bevy_extended_ui::widgets::Button), With<CssID>>) {
    for (mut state, id, mut button) in query.iter_mut() {
        if id.0.eq("fn_q_key") {
            state.disabled = !state.disabled;
            if state.disabled { button.text = "Im Disabled".to_string(); } else { button.text = "Im Enabled".to_string();}
        }
    }
}