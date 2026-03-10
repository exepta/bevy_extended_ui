use bevy::prelude::*;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Badge;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};

#[derive(Resource)]
struct BadgeTickTimer(Timer);

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .insert_resource(BadgeTickTimer(Timer::from_seconds(
            0.8,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (configure_ui, load_ui))
        .add_systems(Update, animate_badges)
        .run();
}

fn configure_ui(mut config: ResMut<ExtendedUiConfiguration>) {
    config.camera = ExtendedCam::Default;
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/badge.html");
    reg.add_and_use("badge-demo".to_string(), HtmlSource::from_handle(handle));
}

fn animate_badges(
    time: Res<Time>,
    mut timer: ResMut<BadgeTickTimer>,
    mut badges: Query<&mut Badge>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for mut badge in &mut badges {
        badge.value = if badge.value >= 130 {
            0
        } else {
            badge.value + 7
        };
    }
}
