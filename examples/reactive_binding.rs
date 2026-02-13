use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlClick, HtmlInnerContent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::{Button, Headline, Paragraph};
use bevy_extended_ui_macros::html_fn;

/// Runtime data used by the reactive binding example.
#[derive(Resource, Debug, Clone)]
struct ReactiveModel {
    user_name: String,
    user_level: u32,
    clicks: u32,
    name_index: usize,
}

impl Default for ReactiveModel {
    fn default() -> Self {
        Self {
            user_name: "Alice".to_string(),
            user_level: 1,
            clicks: 0,
            name_index: 0,
        }
    }
}

/// Runs the reactive binding example app.
fn main() {
    let mut app = make_app("Debug Html UI - reactive binding");

    app.init_resource::<ReactiveModel>();
    app.add_systems(Startup, load_ui);
    app.add_systems(
        Update,
        (
            retitle_headline_template,
            apply_reactive_bindings,
            log_detected_bindings,
        )
            .chain(),
    );

    app.run();
}

/// Loads the HTML for this example.
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/reactive_binding.html");
    reg.add_and_use(
        "reactive-binding".to_string(),
        HtmlSource::from_handle(handle),
    );
}

/// Shows how setters can override templates at runtime.
fn retitle_headline_template(
    model: Res<ReactiveModel>,
    mut query: Query<&mut HtmlInnerContent, With<Headline>>,
) {
    if !model.is_changed() {
        return;
    }

    for mut content in &mut query {
        if model.clicks >= 3 {
            content.set_inner_text("Reactive mode: {{user.name}} clicked {{stats.clicks}} times");
            content.set_inner_bindings(vec![
                "{{user.name}}".to_string(),
                "{{stats.clicks}}".to_string(),
            ]);
        } else {
            content.set_inner_text("Welcome {{user.name}}");
            content.set_inner_bindings(vec!["{{user.name}}".to_string()]);
        }
    }
}

/// Applies `innerBindings` to current model data and updates text widgets.
fn apply_reactive_bindings(
    model: Res<ReactiveModel>,
    mut headline_q: Query<(&HtmlInnerContent, &mut Headline)>,
    mut paragraph_q: Query<(&HtmlInnerContent, &mut Paragraph)>,
    mut button_q: Query<(&HtmlInnerContent, &mut Button)>,
) {
    if !model.is_changed() {
        return;
    }

    for (content, mut headline) in &mut headline_q {
        let rendered = render_from_bindings(content, &model);
        if headline.text != rendered {
            headline.text = rendered;
        }
    }

    for (content, mut paragraph) in &mut paragraph_q {
        let rendered = render_from_bindings(content, &model);
        if paragraph.text != rendered {
            paragraph.text = rendered;
        }
    }

    for (content, mut button) in &mut button_q {
        let rendered = render_from_bindings(content, &model);
        if button.text != rendered {
            button.text = rendered;
        }
    }
}

/// Logs discovered bindings once per widget spawn.
fn log_detected_bindings(query: Query<(Entity, &HtmlInnerContent), Added<HtmlInnerContent>>) {
    for (entity, content) in &query {
        if !content.inner_bindings().is_empty() {
            info!(
                "Reactive bindings on {entity:?}: {:?}",
                content.inner_bindings()
            );
        }
    }
}

/// Converts an inner template into final text by replacing `{{...}}` tokens.
fn render_from_bindings(content: &HtmlInnerContent, model: &ReactiveModel) -> String {
    let mut rendered = content.inner_text().to_string();

    for raw_binding in content.inner_bindings() {
        let expression = raw_binding
            .trim()
            .trim_start_matches("{{")
            .trim_end_matches("}}")
            .trim();

        let Some(value) = resolve_binding(expression, model) else {
            continue;
        };

        rendered = rendered.replace(raw_binding, &value);
    }

    rendered
}

/// Maps a binding expression to the corresponding model value.
fn resolve_binding(expression: &str, model: &ReactiveModel) -> Option<String> {
    match expression {
        "user.name" => Some(model.user_name.clone()),
        "user.level" => Some(model.user_level.to_string()),
        "stats.clicks" => Some(model.clicks.to_string()),
        _ => None,
    }
}

/// Increments the click counter and derived level.
#[html_fn("rb_click")]
fn rb_click(In(_event): In<HtmlClick>, mut model: ResMut<ReactiveModel>) {
    model.clicks = model.clicks.saturating_add(1);
    model.user_level = 1 + (model.clicks / 3);
}

/// Rotates the user name to demonstrate reactive updates.
#[html_fn("rb_rename")]
fn rb_rename(In(_event): In<HtmlClick>, mut model: ResMut<ReactiveModel>) {
    const NAMES: [&str; 4] = ["Alice", "Bob", "Charlie", "Dana"];
    model.name_index = (model.name_index + 1) % NAMES.len();
    model.user_name = NAMES[model.name_index].to_string();
}
