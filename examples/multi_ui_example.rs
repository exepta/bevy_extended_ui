use bevy::prelude::*;
use bevy::asset::{AssetServer, Handle};
use bevy::prelude::{In, Res, ResMut};
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Body;
use bevy_extended_ui_macros::html_fn;

#[derive(Resource)]
struct OverlayState {
    visible: bool,
    applied_initial_hide: bool,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            visible: false,
            applied_initial_hide: false,
        }
    }
}

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.init_resource::<OverlayState>();

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let main_test_handle: Handle<HtmlAsset> = asset_server.load("examples/main_ui.html");
        let overlay_test_handle: Handle<HtmlAsset> = asset_server.load("examples/overlay_ui.html");

        reg.add("main_ui".to_string(), HtmlSource::from_handle(main_test_handle));
        reg.add("overlay_ui".to_string(), HtmlSource::from_handle(overlay_test_handle));
        reg.use_uis(vec!["main_ui".to_string(), "overlay_ui".to_string()]);
    });

    app.add_systems(Update, apply_initial_overlay_state);
    app.add_systems(PostUpdate, enforce_overlay_visibility);

    app.run();
}

#[html_fn("change_ui_state")]
fn change_ui(
    In(_target): In<HtmlEvent>,
    mut state: ResMut<OverlayState>,
    mut q: Query<(&Body, &mut Visibility)>,
) {
    state.visible = !state.visible;

    let target_key = "overlay_ui";

    for (body, mut vis) in q.iter_mut() {
        if body.html_key.as_deref() == Some(target_key) {
            *vis = if state.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn apply_initial_overlay_state(
    mut state: ResMut<OverlayState>,
    mut q: Query<(&Body, &mut Visibility)>,
) {
    if state.applied_initial_hide {
        return;
    }

    let target_key = "overlay_ui";

    let mut found = false;
    for (body, mut vis) in q.iter_mut() {
        if body.html_key.as_deref() == Some(target_key) {
            found = true;
            *vis = Visibility::Hidden; // default AUS
        }
    }

    if found {
        state.applied_initial_hide = true;
    }
}

fn enforce_overlay_visibility(
    state: Res<OverlayState>,
    mut q: Query<(&Body, &mut Visibility)>,
) {
    let target_key = "overlay_ui";

    for (body, mut vis) in q.iter_mut() {
        if body.html_key.as_deref() == Some(target_key) {
            *vis = if state.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}