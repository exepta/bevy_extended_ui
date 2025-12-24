use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui_macros::html_fn;

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/check_box.html");
        reg.add_and_use("check_box_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}

#[html_fn("show_h1")]
fn show_h1(In(_target): In<Entity>, mut query: Query<(&mut Visibility, &CssID), With<CssID>>) {
    for (mut visibility, id) in query.iter_mut() {
        if id.0.eq("h1") {
            change_visibility(&mut visibility);
        }
    }
}

#[html_fn("show_h2")]
fn show_h2(In(_target): In<Entity>, mut query: Query<(&mut Visibility, &CssID), With<CssID>>) {
    for (mut visibility, id) in query.iter_mut() {
        if id.0.eq("h2") {
            change_visibility(&mut visibility);
        }
    }
}

fn change_visibility(visibility: &mut Visibility) {
    if *visibility == Visibility::Visible || *visibility == Visibility::Inherited {
        *visibility = Visibility::Hidden;
    } else {
        *visibility = Visibility::Visible;
    }
}
