use std::collections::HashMap;
use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{ProgressBar, UIWidgetState};

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/progress_bar.html");
        reg.add_and_use("progress_bar_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.add_systems(Update, update_progress_bar);

    app.run();
}

pub fn update_progress_bar(
    time: Res<Time>,
    container_q: Query<(&CssID, &UIWidgetState)>,
    mut progress_bar_q: Query<(&CssID, &mut ProgressBar)>,
    mut raw_by_id: Local<HashMap<String, f32>>,
) {
    let mut trigger_hovered = false;
    for (id, state) in &container_q {
        if id.0.to_string() == "trigger" {
            trigger_hovered = state.hovered;
            break;
        }
    }

    let dir = if trigger_hovered { 1.0 } else { -1.0 };
    let speed = 10.0;
    let dt = time.delta_secs();

    for (id, mut bar) in &mut progress_bar_q {
        if id.0.to_string() != "progress-bar" {
            continue;
        }

        let key = "progress-bar".to_string();
        let raw = raw_by_id.entry(key).or_insert(bar.value);

        *raw = (*raw + dir * speed * dt).clamp(bar.min, bar.max);

        bar.value = if dir > 0.0 {
            raw.floor()
        } else {
            raw.ceil()
        };
    }
}

