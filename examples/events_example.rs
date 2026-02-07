use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{
    HtmlChange, HtmlClick, HtmlDrag, HtmlDragStart, HtmlDragStop, HtmlFocus, HtmlInit, HtmlKeyDown,
    HtmlKeyUp, HtmlMouseOut, HtmlMouseOver, HtmlScroll, HtmlSource,
};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui_macros::html_fn;

fn main() {
    let mut app = make_app("Debug Html UI - events");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/events.html");
        reg.add_and_use("events_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}

#[html_fn("ev_init")]
fn ev_init(In(event): In<HtmlInit>) {
    info!("init target: {:?}", event.entity);
}

#[html_fn("ev_click")]
fn ev_click(In(event): In<HtmlClick>) {
    info!(
        "click target: {:?} position: {:?} inner: {:?}",
        event.entity, event.position, event.inner_position
    );
}

#[html_fn("ev_over")]
fn ev_over(In(event): In<HtmlMouseOver>) {
    info!("mouseover target: {:?}", event.entity);
}

#[html_fn("ev_out")]
fn ev_out(In(event): In<HtmlMouseOut>) {
    info!("mouseout target: {:?}", event.entity);
}

#[html_fn("ev_change")]
fn ev_change(In(event): In<HtmlChange>) {
    info!("change target: {:?} action: {:?}", event.entity, event.action);
}

#[html_fn("ev_focus")]
fn ev_focus(In(event): In<HtmlFocus>) {
    info!("focus target: {:?} state: {:?}", event.entity, event.state);
}

#[html_fn("ev_scroll")]
fn ev_scroll(In(event): In<HtmlScroll>) {
    info!(
        "scroll target: {:?} delta: {:?} x: {:.2} y: {:.2}",
        event.entity, event.delta, event.x, event.y
    );
}

#[html_fn("ev_key_down")]
fn ev_key_down(In(event): In<HtmlKeyDown>) {
    info!("key down target: {:?} key: {:?}", event.entity, event.key);
}

#[html_fn("ev_key_up")]
fn ev_key_up(In(event): In<HtmlKeyUp>) {
    info!("key up target: {:?} key: {:?}", event.entity, event.key);
}

#[html_fn("ev_drag_start")]
fn ev_drag_start(In(event): In<HtmlDragStart>) {
    info!("drag start target: {:?} position: {:?}", event.entity, event.position);
}

#[html_fn("ev_drag")]
fn ev_drag(In(event): In<HtmlDrag>) {
    info!("drag target: {:?} position: {:?}", event.entity, event.position);
}

#[html_fn("ev_drag_stop")]
fn ev_drag_stop(In(event): In<HtmlDragStop>) {
    info!("drag stop target: {:?} position: {:?}", event.entity, event.position);
}
