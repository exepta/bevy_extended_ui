use bevy::prelude::*;
use bevy_extended_ui::html::{
    HtmlMouseDown, HtmlMouseUp, HtmlSource, HtmlTouchEnd, HtmlTouchMove, HtmlTouchStart, HtmlWheel,
};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{Badge, Paragraph, Slider, SliderType};
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};
use bevy_extended_ui_macros::html_fn;

#[derive(Resource)]
struct BadgeTickTimer(Timer);

const EVENT_DEBUG_ID: &str = "event-debug-log";

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .insert_resource(BadgeTickTimer(Timer::from_seconds(
            0.8,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (configure_ui, load_ui))
        .add_systems(Update, (animate_badges, update_range_slider_debug))
        .run();
}

fn configure_ui(mut config: ResMut<ExtendedUiConfiguration>) {
    config.camera = ExtendedCam::Default;
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/widgets_overview.html");
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

fn update_range_slider_debug(
    slider_q: Query<(&Slider, &CssID), With<Slider>>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
) {
    let Some(slider) = slider_q
        .iter()
        .find(|(_, id)| id.0 == "range-slider-demo")
        .map(|(slider, _)| slider)
    else {
        return;
    };

    let text = match slider.slider_type {
        SliderType::Range => format!(
            "Range: {} - {}",
            format_debug_value(slider.range_start),
            format_debug_value(slider.range_end)
        ),
        SliderType::Default => format!("Value: {}", format_debug_value(slider.value)),
    };

    for (mut paragraph, id) in &mut paragraph_q {
        if id.0 != "range-slider-debug" {
            continue;
        }
        paragraph.text = text.clone();
    }
}

fn format_debug_value(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract().abs() < 0.0001 {
        return format!("{}", rounded as i64);
    }

    let mut txt = format!("{rounded:.2}");
    while txt.ends_with('0') {
        txt.pop();
    }
    if txt.ends_with('.') {
        txt.pop();
    }
    txt
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
