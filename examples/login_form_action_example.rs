use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlSource, HtmlSubmit};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui_macros::html_fn;

/// Runs a form-action login example.
fn main() {
    let mut app = make_app("Debug Html UI - login form action");

    app.add_systems(
        Startup,
        |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
            let handle: Handle<HtmlAsset> = asset_server.load("examples/login_form.html");
            reg.add_and_use("login_form".to_string(), HtmlSource::from_handle(handle));
        },
    );

    app.run();
}

/// Handles <form action="login_action"> submits and prints collected data.
#[html_fn("login_action")]
fn login_action(In(event): In<HtmlSubmit>) {
    info!(
        "login action='{}' submitter={:?} form={:?}",
        event.action, event.submitter, event.entity
    );

    let username = event.data.get("username").cloned().unwrap_or_default();
    let email = event.data.get("email").cloned().unwrap_or_default();
    let password = event.data.get("password").cloned().unwrap_or_default();

    info!(
        "login data => username='{}', email='{}', password='{}'",
        username, email, password
    );
}
