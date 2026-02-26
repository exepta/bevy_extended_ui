use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlFocus, HtmlFocusState, HtmlMouseOut, HtmlMouseOver, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{Paragraph, ToggleButton};
use bevy_extended_ui_macros::html_fn;

/// Runs a showcase for recently fixed bugs.
fn main() {
    let mut app = make_app("Debug Html UI - bugfix showcase");
    app.add_systems(Startup, load_ui);
    app.add_systems(Update, sync_showcase_labels);
    app.run();
}

fn load_ui(mut registry: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/bugfix_showcase.html");
    registry.add_and_use(
        "bugfix_showcase".to_string(),
        HtmlSource::from_handle(handle),
    );
}

#[html_fn("hover_std_over")]
fn hover_std_over(In(_event): In<HtmlMouseOver>, mut labels: Query<(&CssID, &mut Paragraph)>) {
    for (id, mut p) in &mut labels {
        if id.0 == "hover-status" {
            p.text = "Now: onmouseover fired (previously ignored)".to_string();
        }
    }
}

#[html_fn("hover_std_out")]
fn hover_std_out(In(_event): In<HtmlMouseOut>, mut labels: Query<(&CssID, &mut Paragraph)>) {
    for (id, mut p) in &mut labels {
        if id.0 == "hover-status" {
            p.text = "Hover the button using onmouseover/onmouseout".to_string();
        }
    }
}

#[html_fn("focus_probe")]
fn focus_probe(
    In(event): In<HtmlFocus>,
    ids: Query<&CssID>,
    mut labels: Query<(&CssID, &mut Paragraph)>,
) {
    if event.state != HtmlFocusState::Gained {
        return;
    }

    let focused_label = ids
        .get(event.entity)
        .ok()
        .map(|id| id.0.clone())
        .unwrap_or_else(|| "unknown".to_string());

    for (id, mut p) in &mut labels {
        if id.0 == "focus-status" {
            p.text =
                format!("Focus: {focused_label} (Test Tab + Shift+Tab for forward/backward)");
        }
    }
}

fn sync_showcase_labels(
    window_query: Query<&Window, With<PrimaryWindow>>,
    toggle_query: Query<(&CssID, &ToggleButton)>,
    mut labels: Query<(&CssID, &mut Paragraph)>,
) {
    let mut free_toggle = None;
    let mut disabled_toggle = None;

    for (id, toggle) in &toggle_query {
        match id.0.as_str() {
            "free-toggle" => free_toggle = Some(toggle.selected),
            "locked-toggle" => disabled_toggle = Some(toggle.selected),
            _ => {}
        }
    }

    let window_text = window_query.single().ok().map(|window| {
        format!(
            "Window: {} x {} px",
            window.resolution.width().round() as u32,
            window.resolution.height().round() as u32
        )
    });

    for (id, mut p) in &mut labels {
        match id.0.as_str() {
            "window-size-live" => {
                if let Some(text) = &window_text {
                    if p.text != *text {
                        p.text = text.clone();
                    }
                }
            }
            "disabled-toggle-status" => {
                let text = match disabled_toggle {
                    Some(true) => "Locked Toggle: selected=true (should remain unchanged on click)",
                    Some(false) => "Locked Toggle: selected=false (used to toggle incorrectly)",
                    None => "Locked Toggle: not found",
                };
                if p.text != text {
                    p.text = text.to_string();
                }
            }
            "free-toggle-status" => {
                let text = match free_toggle {
                    Some(true) => "Free Toggle: selected=true",
                    Some(false) => "Free Toggle: selected=false",
                    None => "Free Toggle: not found",
                };
                if p.text != text {
                    p.text = text.to_string();
                }
            }
            _ => {}
        }
    }
}
