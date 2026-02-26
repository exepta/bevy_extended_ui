use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlClick, HtmlInnerContent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::{Headline, Paragraph};
use bevy_extended_ui_macros::html_fn;

/// Runtime data for the template iteration example.
#[derive(Resource, Debug, Clone)]
struct InventoryModel {
    items: Vec<String>,
    next_id: usize,
}

impl Default for InventoryModel {
    fn default() -> Self {
        Self {
            items: vec![
                "Rope".to_string(),
                "Torch".to_string(),
                "Potion".to_string(),
            ],
            next_id: 4,
        }
    }
}

fn main() {
    let mut app = make_app("Template iteration example");

    app.init_resource::<InventoryModel>();
    app.add_systems(Startup, load_ui);
    app.add_systems(Update, apply_inventory_bindings);

    app.run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/template_iteration.html");
    reg.add_and_use(
        "template-iteration".to_string(),
        HtmlSource::from_handle(handle),
    );
}

/// Applies `{{...}}` bindings to headline and paragraph widgets.
fn apply_inventory_bindings(
    model: Res<InventoryModel>,
    added: Query<(), Added<HtmlInnerContent>>,
    mut headline_q: Query<(&HtmlInnerContent, &mut Headline)>,
    mut paragraph_q: Query<(&HtmlInnerContent, &mut Paragraph)>,
) {
    let should_refresh = model.is_changed() || !added.is_empty();
    if !should_refresh {
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
}

fn render_from_bindings(content: &HtmlInnerContent, model: &InventoryModel) -> String {
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

fn resolve_binding(expression: &str, model: &InventoryModel) -> Option<String> {
    match expression {
        "inventory.count" => Some(model.items.len().to_string()),
        "inventory.list" => Some(format_inventory_list(&model.items)),
        _ => None,
    }
}

fn format_inventory_list(items: &[String]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| format!("{}. {}", i + 1, item))
        .collect::<Vec<_>>()
        .join("\n")
}

#[html_fn("inv_add")]
fn inv_add(In(_event): In<HtmlClick>, mut model: ResMut<InventoryModel>) {
    let item = format!("Item {}", model.next_id);
    model.items.push(item);
    model.next_id += 1;
}

#[html_fn("inv_clear")]
fn inv_clear(In(_event): In<HtmlClick>, mut model: ResMut<InventoryModel>) {
    model.items.clear();
}
