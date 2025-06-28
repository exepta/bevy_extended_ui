use bevy::prelude::*;
use crate::html::{HtmlEventBindings, HtmlMeta, HtmlStates, HtmlStructureMap, HtmlWidgetNode};
use crate::styling::convert::{CssClass, CssID, CssSource};
use crate::UIWidgetState;
use crate::widgets::Widget;

#[derive(Event)]
struct AllWidgetsSpawned;

#[derive(Resource, Default)]
struct ShowWidgetsTimer {
    timer: Timer,
    active: bool,
}

/// A plugin that spawns Bevy UI entities from parsed HTML node structures.
pub struct HtmlBuilderSystem;

impl Plugin for HtmlBuilderSystem {
    /// Registers the HTML builder system to run whenever the HTML structure maps resource changes.
    fn build(&self, app: &mut App) {
        app.add_event::<AllWidgetsSpawned>();
        app.insert_resource(ShowWidgetsTimer::default());
        app.add_systems(Update, build_html_source.run_if(resource_changed::<HtmlStructureMap>));
        app.add_systems(Update, show_all_widgets_start.after(build_html_source));
        app.add_systems(Update, show_all_widgets_finish.after(show_all_widgets_start));
    }
}

/// System that builds the active HTML structure into Bevy UI entities.
///
/// This system is triggered when the [`HtmlStructureMap`] resource changes.
/// It looks up the active structure and recursively spawns UI entities for each node.
fn build_html_source(
    mut commands: Commands,
    structure_map: Res<HtmlStructureMap>,
    asset_server: Res<AssetServer>,
    mut event_writer: EventWriter<AllWidgetsSpawned>,
) {
    if let Some(active) = structure_map.active.clone() {
        if let Some(structure) = structure_map.html_map.get(active.as_str()) {
            for node in structure {
                spawn_widget_node(&mut commands, node, &asset_server, None);
            }
            event_writer.write(AllWidgetsSpawned);
        }
    }
}

fn show_all_widgets_start(
    mut events: EventReader<AllWidgetsSpawned>,
    mut timer: ResMut<ShowWidgetsTimer>,
) {
    for _event in events.read() {
        timer.timer = Timer::from_seconds(0.1, TimerMode::Once);
        timer.active = true;
        debug!("Starting 100ms timer before showing widgets");
    }
}

fn show_all_widgets_finish(
    time: Res<Time>,
    mut timer: ResMut<ShowWidgetsTimer>,
    mut query: Query<&mut Visibility, With<Widget>>,
) {
    if timer.active && timer.timer.tick(time.delta()).finished() {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Inherited;
        }
        timer.active = false;
        debug!("All widgets are now visible after 100ms delay");
    }
}

/// Recursively spawns Bevy entities for a given [`HtmlWidgetNode`] and its children.
///
/// Each entity is assigned UI components as well as metadata components like `CssClass` and `CssID`.
///
/// If a parent entity is provided, the new entity is added as a child of that parent.
fn spawn_widget_node(
    commands: &mut Commands,
    node: &HtmlWidgetNode,
    asset_server: &AssetServer,
    parent: Option<Entity>,
) -> Entity {
    let entity = match node {
        HtmlWidgetNode::Button(button, meta, states, functions, widget) => {
            spawn_with_meta(commands, button.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::Input(input, meta, states, functions, widget) => {
            spawn_with_meta(commands, input.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::Headline(headline, meta, states, functions, widget) => {
            spawn_with_meta(commands, headline.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::Img(img, meta, states, functions, widget) => {
            spawn_with_meta(commands, img.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::ProgressBar(progress_bar, meta, states, functions, widget) => {
            spawn_with_meta(commands, progress_bar.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::Paragraph(paragraph, meta, states, functions, widget) => {
            spawn_with_meta(commands, paragraph.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::CheckBox(checkbox, meta, states, functions, widget) => {
            spawn_with_meta(commands, checkbox.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::ChoiceBox(choice_box, meta, states, functions, widget) => {
            spawn_with_meta(commands, choice_box.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::Slider(slider, meta, states, functions, widget) => {
            spawn_with_meta(commands, slider.clone(), meta, states, functions, widget)
        }
        HtmlWidgetNode::Div(div, meta, states, children, functions,widget) => {
            let entity = spawn_with_meta(commands, div.clone(), meta, states, functions, widget);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
        HtmlWidgetNode::HtmlBody(body, meta, states, children, functions, widget) => {
            let entity = spawn_with_meta(commands, body.clone(), meta, states, functions, widget);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
    };

    if let Some(parent) = parent {
        commands.entity(parent).add_child(entity);
    }

    entity
}

/// Spawns a single UI entity with metadata components based on the provided [`HtmlMeta`].
///
/// The entity will include a [`Node`] component, as well as any CSS metadata like `class`, `id`,
/// and raw CSS source.
///
/// # Type Parameters
/// - `T`: A Bevy component representing a UI element (e.g., `Button`, `Div`, etc.).
fn spawn_with_meta<T: Component>(
    commands: &mut Commands,
    component: T,
    meta: &HtmlMeta,
    states: &HtmlStates,
    functions: &HtmlEventBindings,
    widget: &Widget
) -> Entity {
    let mut visible = Visibility::Hidden;
    let mut ui_state = UIWidgetState::default();
    
    if states.hidden {
        visible = Visibility::Hidden;
    }
    
    ui_state.readonly = states.readonly;
    ui_state.disabled = states.disabled;
    
    commands.spawn((
        component,
        functions.clone(),
        widget.clone(),
        Node::default(),
        CssSource(meta.css.clone()),
        CssClass(meta.class.clone().unwrap_or_default()),
        CssID(meta.id.clone().unwrap_or_default()),
        ui_state,
        visible
    )).id()
}
