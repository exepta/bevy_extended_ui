use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::{CssID, IconPlace};
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

#[html_fn("init_me_btn")]
fn init_me_btn(In(target): In<HtmlEvent>) {
    info!("init_me_btn called for entity {:?}", target.entity);
}

#[html_fn("click_me_btn")]
fn click_me_btn(In(_target): In<HtmlEvent>, mut query: Query<(&mut UIWidgetState, &CssID, &mut bevy_extended_ui::widgets::Button), With<CssID>>) {
    for (mut state, id, mut button) in query.iter_mut() {
        if id.0.eq("fn_q_key") {
            state.disabled = !state.disabled;
            if state.disabled { button.text = "Im Disabled".to_string(); } else { button.text = "Im Enabled".to_string();}
        }
    }
}

#[html_fn("enter_me_btn")]
fn enter_me_btn(In(target): In<HtmlEvent>, mut query: Query<(Entity, &mut bevy_extended_ui::widgets::Button), With<CssID>>) {
    for (entity, mut button) in query.iter_mut() {
        if entity.eq(&target.entity) {
            button.icon_place = IconPlace::Right;
        }
    }
}

#[html_fn("leave_me_btn")]
fn leave_me_btn(In(target): In<HtmlEvent>, mut query: Query<(Entity, &mut bevy_extended_ui::widgets::Button), With<CssID>>) {
    for (entity, mut button) in query.iter_mut() {
        if entity.eq(&target.entity) {
            button.icon_place = IconPlace::Left;
        }
    }
}