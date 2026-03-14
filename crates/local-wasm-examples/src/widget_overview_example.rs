use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy_extended_ui::html::{
    HtmlMouseDown, HtmlMouseUp, HtmlSource, HtmlTouchEnd, HtmlTouchMove, HtmlTouchStart, HtmlWheel,
};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{Badge, Paragraph};
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};
use bevy_extended_ui_macros::html_fn;

#[derive(Resource)]
struct BadgeTickTimer(Timer);

const EVENT_DEBUG_ID: &str = "event-debug-log";

pub fn run() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Extended UI WASM Widget Gallery".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Extended UI WASM Widget Gallery (Native Fallback)".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        );
    }

    app.add_plugins(ExtendedUiPlugin)
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
    let handle: Handle<HtmlAsset> = asset_server.load("examples/widgets_overview.html");
    reg.add_and_use("widgets-demo".to_string(), HtmlSource::from_handle(handle));
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

fn set_event_debug_message(
    text: String,
    paragraph_q: &mut Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    for (mut paragraph, id) in paragraph_q.iter_mut() {
        if id.0 != EVENT_DEBUG_ID {
            continue;
        }
        paragraph.text = text;
        return;
    }
}

#[html_fn("event_mouse_down")]
fn event_mouse_down(
    In(event): In<HtmlMouseDown>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    set_event_debug_message(
        format!(
            "Event: mousedown ({:?}) @ {:.0}, {:.0}",
            event.button, event.inner_position.x, event.inner_position.y
        ),
        &mut paragraph_q,
    );
}

#[html_fn("event_mouse_up")]
fn event_mouse_up(
    In(event): In<HtmlMouseUp>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    set_event_debug_message(
        format!(
            "Event: mouseup ({:?}) @ {:.0}, {:.0}",
            event.button, event.inner_position.x, event.inner_position.y
        ),
        &mut paragraph_q,
    );
}

#[html_fn("event_wheel")]
fn event_wheel(
    In(event): In<HtmlWheel>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let unit = match event.unit {
        bevy::input::mouse::MouseScrollUnit::Line => "line",
        bevy::input::mouse::MouseScrollUnit::Pixel => "pixel",
    };
    set_event_debug_message(
        format!(
            "Event: wheel [{}] dx={:.2} dy={:.2}",
            unit, event.delta.x, event.delta.y
        ),
        &mut paragraph_q,
    );
}

#[html_fn("event_touch_start")]
fn event_touch_start(
    In(event): In<HtmlTouchStart>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    set_event_debug_message(
        format!(
            "Event: touchstart #{} @ {:.0}, {:.0}",
            event.touch_id, event.inner_position.x, event.inner_position.y
        ),
        &mut paragraph_q,
    );
}

#[html_fn("event_touch_move")]
fn event_touch_move(
    In(event): In<HtmlTouchMove>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    set_event_debug_message(
        format!(
            "Event: touchmove #{} dx={:.1} dy={:.1}",
            event.touch_id, event.delta.x, event.delta.y
        ),
        &mut paragraph_q,
    );
}

#[html_fn("event_touch_end")]
fn event_touch_end(
    In(event): In<HtmlTouchEnd>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    set_event_debug_message(
        format!(
            "Event: touchend #{} @ {:.0}, {:.0}",
            event.touch_id, event.inner_position.x, event.inner_position.y
        ),
        &mut paragraph_q,
    );
}
