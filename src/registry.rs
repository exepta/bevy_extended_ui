use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use crate::html::HtmlSource;
use crate::widgets::{HtmlBody, WidgetId, WidgetKind};

/// A global thread-safe pool of IDs for the "body" widget.
pub static BODY_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "div" widget.
pub static DIV_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "headline" widget.
pub static HEADLINE_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "paragraph" widget.
pub static PARAGRAPH_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "button" widget.
pub static BUTTON_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "input" widget.
pub static INPUT_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "select/choice box" widget.
pub static SELECT_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "slider" widget.
pub static SLIDER_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "image" widget.
pub static IMG_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
/// A global thread-safe pool of IDs for the "check box" widget.
pub static CHECK_BOX_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));

/// A pool that manages reusable integer IDs for widgets.
/// It hands out new IDs or recycles freed IDs.
pub struct IdPool {
    /// The next ID to assign if no free ID exists.
    counter: usize,
    /// Queue of IDs that have been released and can be reused.
    free_list: VecDeque<usize>,
}

impl IdPool {
    /// Creates a new empty `IdPool`.
    pub fn new() -> Self {
        Self {
            counter: 0,
            free_list: VecDeque::new(),
        }
    }

    /// Acquires a new ID.
    /// Returns a recycled ID if available, otherwise generates a new one.
    pub fn acquire(&mut self) -> usize {
        if let Some(id) = self.free_list.pop_front() {
            id
        } else {
            let id = self.counter;
            self.counter += 1;
            id
        }
    }

    /// Releases an ID back to the pool for reuse.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to release.
    pub fn release(&mut self, id: usize) {
        self.free_list.push_back(id);
    }
}

/// Resource that registers and manages available UI HTML sources.
///
/// It stores named HTML sources and tracks the currently active UI.
#[derive(Default, Resource, Debug)]
pub struct UiRegistry {
    /// Collection mapping UI names to their HTML source data.
    pub collection: HashMap<String, HtmlSource>,
    /// The currently active UI name, if any.
    pub current: Option<String>,
}

impl UiRegistry {
    /// Creates a new empty UI registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a named UI HTML source to the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The name under which the UI source will be registered.
    /// * `source` - The HTML source data.
    pub fn add(&mut self, name: String, source: HtmlSource) {
        self.collection.insert(name.clone(), HtmlSource { source: source.source.clone(), source_id: name.clone() });
    }

    /// Adds a UI source and marks it as currently in use.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to register and use.
    /// * `source` - The HTML source data.
    pub fn add_and_use(&mut self, name: String, source: HtmlSource) {
        self.add(name.clone(), HtmlSource { source: source.source.clone(), source_id: name.clone() });
        self.use_ui(&name);
    }

    /// Retrieves a UI source by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The UI name to look up.
    ///
    /// # Returns
    ///
    /// An optional reference to the `HtmlSource` if found.
    pub fn get(&self, name: &str) -> Option<&HtmlSource> {
        self.collection.get(name)
    }

    /// Sets the currently active UI by name.
    ///
    /// Logs an error if the UI name is not found.
    ///
    /// # Arguments
    ///
    /// * `name` - The UI name to activate.
    pub fn use_ui(&mut self, name: &str) {
        if self.get(name).is_some() {
            self.current = Some(name.to_string());
        } else {
            error!("Ui {} not found", name);
        }
    }
}

/// Resource tracking the last UI usage name to detect changes.
#[derive(Resource, Debug)]
struct LastUiUsage(pub Option<String>);

/// Bevy plugin that manages the UI registry lifecycle and cleanup.
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_widget_ids,
                update_que,
            )
                .chain()
                .run_if(resource_changed::<UiRegistry>)
        );
    }
}

/// System that updates the UI entity queue.
/// Spawns UI entities if not present or reloads if a different UI is requested.
///
/// # Arguments
///
/// * `commands` - Commands for spawning/despawning entities.
/// * `ui_registry` - The UI registry resource.
/// * `query` - Query for entities with `HtmlSource`.
/// * `body_query` - Query for body entities without `HtmlSource` but with `HtmlBody`.
fn update_que(
    mut commands: Commands,
    ui_registry: Res<UiRegistry>,
    query: Query<(Entity, &HtmlSource), With<HtmlSource>>,
    body_query: Query<Entity, (Without<HtmlSource>, With<HtmlBody>)>,
) {
    if let Some(name) = ui_registry.current.clone() {
        if query.is_empty() {
            spawn_ui_source(&mut commands, &name, &ui_registry);
            return;
        }

        for (entity, html_source) in query.iter() {
            if html_source.source_id == name {
                warn!("UI {} is already loaded", name);
                continue;
            }

            // Despawn old body entities before spawning a new UI
            for body_entity in body_query.iter() {
                commands.entity(body_entity).despawn();
            }

            spawn_ui_source(&mut commands, &name, &ui_registry);

            // Despawn outdated UI entity
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns a UI entity from a registered HTML source by name.
///
/// # Arguments
///
/// * `commands` - Commands to spawn the UI entity.
/// * `name` - The name of the UI to spawn.
/// * `ui_registry` - The UI registry resource.
fn spawn_ui_source(commands: &mut Commands, name: &str, ui_registry: &UiRegistry) {
    if let Some(source) = ui_registry.get(name) {
        commands.spawn((
            Name::new(name.to_string()),
            source.clone(),
        ));
        info!("Loaded Registered UI {:?}", source);
    } else {
        warn!("UI source {} not found in registry", name);
    }
}

/// System that releases widget IDs back to their respective pools when widgets are despawned.
///
/// It avoids releasing IDs if the UI hasn't changed since the last run.
///
/// # Arguments
///
/// * `commands` - Commands to insert resources.
/// * `ui_registry` - Current UI registry resource.
/// * `last_ui_usage` - Optional resource tracking the last UI used.
/// * `query` - Query to iterate over entities with `WidgetId`.
/// * `widget_ids` - Query to access the `WidgetId` components.
fn despawn_widget_ids(
    mut commands: Commands,
    ui_registry: Res<UiRegistry>,
    last_ui_usage: Option<Res<LastUiUsage>>,
    query: Query<Entity, With<WidgetId>>,
    widget_ids: Query<&WidgetId>,
) {
    if let Some(name) = ui_registry.current.clone() {
        if let Some(last_ui) = last_ui_usage {
            if let Some(last_ui_name) = last_ui.0.clone() {
                if last_ui_name.eq(&name) {
                    // UI hasn't changed, skip releasing IDs
                    info!("UI unchanged: current = {}, last = {}", name, last_ui_name);
                    return;
                }
            }
        }
    }

    for entity in query.iter() {
        if let Ok(widget_id) = widget_ids.get(entity) {
            match widget_id.kind {
                WidgetKind::HtmlBody => BODY_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Div => DIV_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Headline => HEADLINE_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Paragraph => PARAGRAPH_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Button => BUTTON_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::CheckBox => CHECK_BOX_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Slider => SLIDER_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::InputField => INPUT_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::ChoiceBox => SELECT_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Img => IMG_ID_POOL.lock().unwrap().release(widget_id.id),
            }
        }
    }

    // Remember the current UI usage for the next run
    commands.insert_resource(LastUiUsage(ui_registry.current.clone()));
}