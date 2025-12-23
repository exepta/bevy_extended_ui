use bevy::prelude::*;
use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use crate::html::HtmlSource;
use crate::widgets::{Body, UIGenID, WidgetId, WidgetKind};

pub static UI_ID_GENERATE: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static BODY_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static DIV_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static BUTTON_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static CHECK_BOX_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static CHOICE_BOX_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static DIVIDER_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static FIELDSET_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static HEADLINE_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static IMAGE_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static INPUT_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static PARAGRAPH_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static PROGRESS_BAR_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static RADIO_BUTTON_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static SCROLL_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static SLIDER_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static SWITCH_BUTTON_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));
pub static TOGGLE_BUTTON_ID_POOL: Lazy<Mutex<IdPool>> = Lazy::new(|| Mutex::new(IdPool::new()));

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
#[derive(Default, Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct UiRegistry {
    /// Collection mapping UI names to their HTML source data.
    pub collection: HashMap<String, HtmlSource>,
    /// The currently active UI name, if any.
    pub current: Option<String>,

    pub ui_update: bool,
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
        self.collection.insert(name.clone(), HtmlSource { source_id: name.clone(), ..source });
    }

    /// Adds a UI source and marks it as currently in use.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to register and use.
    /// * `source` - The HTML source data.
    pub fn add_and_use(&mut self, name: String, source: HtmlSource) {
        self.add(name.clone(), HtmlSource { source_id: name.clone(), ..source});
        self.use_ui(&name);
    }

    /// Removes a UI source from the registry by its name.
    ///
    /// If the currently active UI (`current`) matches the given name,
    /// it will also be cleared.
    ///
    /// # Arguments
    ///
    /// * `name` - The identifier of the UI to remove from the registry.
    pub fn remove(&mut self, name: &str) {
        if let Some(current) = self.current.clone() {
            if current.eq(&name.to_string()) {
                self.current = None;
            }
        }
        self.collection.remove(name);
    }

    /// Removes a UI source and immediately switches to another one.
    ///
    /// This is useful when replacing a currently loaded UI with a new one.
    /// If the removed UI is currently active, it switches to `to_switch`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the UI to be removed.
    /// * `to_switch` - The name of the UI to activate after removal.
    pub fn remove_and_use(&mut self, name: &str, to_switch: &str) {
        self.remove(name);
        self.use_ui(to_switch);
    }

    /// Removes all UI sources from the registry and clears the current UI.
    ///
    /// This is typically used during global resets or cleanup operations.
    pub fn remove_all(&mut self) {
        self.collection.clear();
        self.current = None;
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

    pub fn get_mut(&mut self, name: &str) -> Option<&mut HtmlSource> {
        self.collection.get_mut(name)
    }

    /// Sets the currently active UI by name if it exists in the registry.
    ///
    /// If the UI with the given name exists in the internal collection, it will be marked
    /// as the currently active UI by setting the `current` field. If it does not exist,
    /// a warning will be logged and the current UI will be set to `None`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the UI to activate.
    ///
    /// # Behavior
    ///
    /// - If the named UI exists: `self.current` will be set to `Some(name.to_string())`.
    /// - If the named UI does not exist: `self.current` will be set to `None`, and a warning is logged.
    pub fn use_ui(&mut self, name: &str) {
        if self.get(name).is_some() {
            self.current = Some(name.to_string());
            self.ui_update = true;
        } else {
            warn!("Ui was empty and will removed now!");
            self.current = None;
        }
    }
}

/// Resource flag used to control whether UI widgets should be initialized.
///
/// This resource wraps a single `bool` value indicating whether the widget initialization
/// logic should run during the next update cycle.
///
/// # Fields
/// - `0`: A `bool` flag. `true` means initialization should run; `false` means no initialization.
#[derive(Default, Resource, Debug)]
pub struct UiInitResource(pub bool);

/// Resource tracking the last UI usage name to detect changes.
#[derive(Resource, Debug)]
struct LastUiUsage(pub Option<String>);

/// Bevy plugin that manages the UI registry lifecycle and cleanup.
pub struct ExtendedRegistryPlugin;

impl Plugin for ExtendedRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiInitResource>();
        app.init_resource::<UiRegistry>();
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
    mut ui_registry: ResMut<UiRegistry>,
    mut ui_init: ResMut<UiInitResource>,
    query: Query<(Entity, &HtmlSource), With<HtmlSource>>,
    mut body_query: Query<(Entity, &mut Visibility, &Body), (Without<HtmlSource>, With<Body>)>,
) {
    if let Some(name) = ui_registry.current.clone() {
        if query.is_empty() {
            spawn_ui_source(&mut commands, &name, &ui_registry, &mut ui_init);
            return;
        }

        for (entity, html_source) in query.iter() {
            if html_source.source_id == name && !ui_registry.ui_update{
                continue;
            }

            // Despawn old body entities before spawning a new UI
            for (body_entity, mut body_vis, body) in body_query.iter_mut() {
                if ui_registry.ui_update {
                    commands.entity(body_entity).despawn();
                } else {
                    if let Some(bind) = body.html_key.clone() {
                        if bind.eq(&html_source.source_id) {
                            *body_vis = Visibility::Hidden;
                        }
                    }
                }
            }

            ui_registry.ui_update = false;

            spawn_ui_source(&mut commands, &name, &ui_registry, &mut ui_init);

            // Despawn outdated UI entity
            commands.entity(entity).despawn();
        }
    } else {
        for (entity, html_source) in query.iter() {

            for (_, mut body_vis, body) in body_query.iter_mut() {
                if let Some(bind) = body.html_key.clone() {
                    if bind.eq(&html_source.source_id) {
                        *body_vis = Visibility::Hidden;
                    }
                }
            }

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
fn spawn_ui_source(commands: &mut Commands, name: &str, ui_registry: &UiRegistry, ui_init: &mut UiInitResource) {
    if let Some(source) = ui_registry.get(name) {
        ui_init.0 = true;
        commands.spawn((
            Name::new(name.to_string()),
            source.clone(),
        ));
        debug!("Loaded Registered UI {:?}", source);
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
    ui_id: Query<&UIGenID>
) {
    if let Some(name) = ui_registry.current.clone() {
        if let Some(last_ui) = last_ui_usage {
            if let Some(last_ui_name) = last_ui.0.clone() {
                if last_ui_name.eq(&name) {
                    // UI hasn't changed, skip releasing IDs
                    debug!("UI unchanged: current: {}, last: {}", name, last_ui_name);
                }
            }
        }
    }

    for id in ui_id.iter() {
        UI_ID_GENERATE.lock().unwrap().release(id.get());
    }

    for entity in query.iter() {
        if let Ok(widget_id) = widget_ids.get(entity) {
            match widget_id.kind {
                WidgetKind::Body => BODY_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Div => DIV_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Headline => HEADLINE_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Paragraph => PARAGRAPH_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Button => BUTTON_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::CheckBox => CHECK_BOX_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Slider => SLIDER_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::InputField => INPUT_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::ChoiceBox => CHOICE_BOX_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Img => IMAGE_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::ProgressBar => PROGRESS_BAR_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::RadioButton => RADIO_BUTTON_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::SwitchButton => SWITCH_BUTTON_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::ToggleButton => TOGGLE_BUTTON_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Scrollbar => SCROLL_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::Divider => DIVIDER_ID_POOL.lock().unwrap().release(widget_id.id),
                WidgetKind::FieldSet => FIELDSET_ID_POOL.lock().unwrap().release(widget_id.id),
            }
        }
    }

    // Remember the current UI usage for the next run
    commands.insert_resource(LastUiUsage(ui_registry.current.clone()));
}
