use bevy::prelude::*;
#[cfg(feature = "extended-framework")]
use regex::Regex;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::any::{Any, TypeId, type_name};
#[cfg(feature = "extended-framework")]
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "extended-framework")]
use crate::component::{
    load_component_definitions, load_component_template_html, validate_component_assets,
};
use crate::lang::UiSharedValues;
#[cfg(feature = "extended-framework")]
use crate::routing::Router;

pub use inventory;

/// Configuration for the experimental extended framework mode.
///
/// - `assets_component_root`: root folder (inside `assets/`) for Angular-like components.
/// - `rust_component_root`: root folder (inside project `src/`) for component logic files.
#[cfg(feature = "extended-framework")]
#[derive(Resource, Debug, Clone)]
pub struct ExtendedFrameworkConfiguration {
    pub assets_component_root: String,
    pub rust_component_root: String,
    pub asset_root_fs_path: String,
    pub index_html_file: String,
}

#[cfg(feature = "extended-framework")]
impl Default for ExtendedFrameworkConfiguration {
    /// Handles `default` in the extended UI workflow.
    fn default() -> Self {
        Self {
            assets_component_root: "ui/bevy_ang".to_string(),
            rust_component_root: "src/packages".to_string(),
            asset_root_fs_path: "assets".to_string(),
            index_html_file: "index.html".to_string(),
        }
    }
}

/// Result of the framework pre-compile phase.
///
/// For the base implementation this is intentionally a no-op transform for HTML,
/// plus an optional inferred component-controller path.
#[cfg(feature = "extended-framework")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameworkCompileResult {
    pub html: String,
    pub inferred_controller: Option<String>,
    pub component_controllers: Vec<String>,
}

/// Plugin for initializing resources used by the extended framework.
#[cfg(feature = "extended-framework")]
pub struct ExtendedFrameworkPlugin;

#[cfg(feature = "extended-framework")]
impl Plugin for ExtendedFrameworkPlugin {
    /// Handles `build` in the extended UI workflow.
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtendedFrameworkConfiguration>();
        app.init_resource::<UiBindingStore>();
        app.add_systems(Startup, register_beu_stores);
    }
}

/// Trait implemented by `#[derive(BeuStore)]`.
///
/// A store type is registered automatically when [`ExtendedFrameworkPlugin`]
/// starts. Values can then be written with [`UiBindingStore::set_store`] and
/// read by templates through the shared template value pipeline.
pub trait BeuStore: Send + Sync + 'static {
    /// Short key used in [`UiBindingStore::data`].
    const STORE_KEY: &'static str;
    /// Fully-qualified Rust path used for disambiguated template imports.
    const STORE_PATH: &'static str;
}

/// Runtime registration emitted by `#[derive(BeuStore)]`.
pub struct UiBindingStoreRegistration {
    pub key: &'static str,
    pub path: &'static str,
    pub register: fn(&mut UiBindingStore),
}

inventory::collect!(UiBindingStoreRegistration);

/// Type-erased value storage for [`UiBindingStore`].
pub trait UiBindingStoredValue: Any + Send + Sync {
    /// Returns the value as [`Any`] for typed downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Compares two erased values without marking the store as changed.
    fn equals(&self, other: &dyn UiBindingStoredValue) -> bool;
}

impl<T> UiBindingStoredValue for T
where
    T: PartialEq + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, other: &dyn UiBindingStoredValue) -> bool {
        other.as_any().downcast_ref::<T>() == Some(self)
    }
}

/// A single typed value stored by [`UiBindingStore`].
pub struct UiBindingValue {
    type_id: TypeId,
    type_name: &'static str,
    value: Box<dyn UiBindingStoredValue>,
    json: Option<JsonValue>,
}

impl UiBindingValue {
    /// Returns the Rust [`TypeId`] of the stored value.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the Rust type name of the stored value.
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Returns the template-visible JSON value, when the value was inserted
    /// through a serializing setter.
    pub fn json(&self) -> Option<&JsonValue> {
        self.json.as_ref()
    }

    /// Returns the stored value as a typed reference.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.value.as_any().downcast_ref::<T>()
    }
}

/// Metadata and value slot for a registered UI binding key.
pub struct UiBindingEntry {
    type_id: TypeId,
    type_name: &'static str,
    type_path: &'static str,
    value: Option<UiBindingValue>,
    revision: u64,
}

impl UiBindingEntry {
    /// Returns the registered Rust [`TypeId`].
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the short Rust type name for this entry.
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Returns the fully-qualified Rust type path for this entry.
    pub fn type_path(&self) -> &'static str {
        self.type_path
    }

    /// Returns the per-entry revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns whether this key has a concrete value.
    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }

    /// Returns the template-visible JSON value for this entry.
    pub fn json(&self) -> Option<&JsonValue> {
        self.value.as_ref().and_then(UiBindingValue::json)
    }

    /// Returns the stored value as a typed reference.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.value.as_ref().and_then(UiBindingValue::get::<T>)
    }
}

/// Reactive store used by the extended framework.
///
/// Values are addressed by string keys and are stored type-erased so primitive
/// values, strings, booleans, structs, enums, and other serializable Rust values
/// can share the same resource. The store maintains its own revision counter;
/// the revision only changes when a setter stores a different value.
#[derive(Resource, Default)]
pub struct UiBindingStore {
    /// Keyed store data. Use the provided setter methods so revisions stay
    /// accurate and the UI only rebuilds after real value changes.
    pub data: HashMap<String, UiBindingEntry>,
    known_types: HashSet<String>,
    revision: u64,
}

impl UiBindingStore {
    /// Registers a `#[derive(BeuStore)]` type without requiring a value.
    pub fn register<T: BeuStore>(&mut self) -> bool {
        self.register_type::<T>(T::STORE_KEY, T::STORE_PATH)
    }

    /// Registers a type under an explicit key and path.
    pub fn register_type<T: Send + Sync + 'static>(
        &mut self,
        key: &'static str,
        path: &'static str,
    ) -> bool {
        let changed_known = self.known_types.insert(key.to_string())
            | self.known_types.insert(path.to_string())
            | self
                .known_types
                .insert(simple_type_name(type_name::<T>()).to_string());

        if self.data.contains_key(key) {
            if changed_known {
                self.bump_revision();
            }
            return changed_known;
        }

        self.data.insert(
            key.to_string(),
            UiBindingEntry {
                type_id: TypeId::of::<T>(),
                type_name: simple_type_name(type_name::<T>()),
                type_path: path,
                value: None,
                revision: self.revision + 1,
            },
        );
        self.bump_revision();
        true
    }

    /// Sets a serializable value under a string key.
    ///
    /// Returns `true` only when the stored value actually changed.
    pub fn set<T>(&mut self, key: impl Into<String>, value: T) -> bool
    where
        T: UiBindingStoredValue + Serialize,
    {
        let key = key.into();
        let type_path = type_name::<T>();
        let json = serde_json::to_value(&value).ok();
        self.set_internal::<T>(&key, simple_type_name(type_path), type_path, value, json)
    }

    /// Sets a serializable value for a `#[derive(BeuStore)]` type.
    ///
    /// Returns `true` only when the stored value actually changed.
    pub fn set_store<T>(&mut self, value: T) -> bool
    where
        T: BeuStore + UiBindingStoredValue + Serialize,
    {
        let json = serde_json::to_value(&value).ok();
        self.set_internal::<T>(T::STORE_KEY, T::STORE_KEY, T::STORE_PATH, value, json)
    }

    /// Sets a non-serializable value under a string key.
    ///
    /// Raw values can be read back through typed getters but are not visible to
    /// template interpolation because no JSON representation is available.
    pub fn set_raw<T>(&mut self, key: impl Into<String>, value: T) -> bool
    where
        T: UiBindingStoredValue,
    {
        let key = key.into();
        let type_path = type_name::<T>();
        self.set_internal::<T>(&key, simple_type_name(type_path), type_path, value, None)
    }

    /// Returns a typed value stored under a key.
    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(UiBindingEntry::get::<T>)
    }

    /// Returns a typed value stored for a `#[derive(BeuStore)]` type.
    pub fn get_store<T: BeuStore>(&self) -> Option<&T> {
        self.get::<T>(T::STORE_KEY)
    }

    /// Returns a template-visible JSON value from a dot-separated binding path.
    pub fn json_path(&self, path: &str) -> Option<JsonValue> {
        let (root, tail) = split_binding_path(path)?;
        let root = self.resolve_binding_root(root);
        let entry = self.data.get(root.as_str())?;
        let value = entry.json()?.clone();
        resolve_json_tail(value, &tail)
    }

    /// Sets a template-visible value through a dot-separated binding path.
    ///
    /// Direct primitive keys keep their registered Rust type (`bool`, strings,
    /// all common integer widths, `usize`/`isize`, `f32`, `f64`). Nested paths
    /// update the stored JSON projection so templates can react immediately.
    pub fn set_path_json(&mut self, path: &str, value: JsonValue) -> bool {
        let Some((root, tail)) = split_binding_path(path) else {
            return false;
        };
        let root = self.resolve_binding_root(root);
        let root = root.as_str();

        if tail.is_empty() {
            return self.set_direct_json(root, value);
        }

        let mut root_json = self
            .data
            .get(root)
            .and_then(UiBindingEntry::json)
            .cloned()
            .unwrap_or_else(|| JsonValue::Object(Default::default()));

        if !set_json_tail(&mut root_json, &tail, value) {
            return false;
        }

        self.set_json_projection(root, root_json)
    }

    /// Returns whether a key is registered in the store.
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns the current store revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns all known type names and paths registered with the store.
    pub fn known_types(&self) -> impl Iterator<Item = &str> {
        self.known_types.iter().map(String::as_str)
    }

    /// Returns serializable store values for the template shared-value pipeline.
    pub fn template_values(&self) -> Vec<(String, JsonValue)> {
        let mut values = Vec::new();

        for (key, entry) in &self.data {
            let Some(json) = entry.json() else {
                continue;
            };

            values.push((key.clone(), json.clone()));
            if entry.type_path() != key {
                values.push((entry.type_path().to_string(), json.clone()));
            }
            if entry.type_name() != key {
                values.push((entry.type_name().to_string(), json.clone()));
            }
        }

        values
    }

    fn set_internal<T>(
        &mut self,
        key: &str,
        type_name: &'static str,
        type_path: &'static str,
        value: T,
        json: Option<JsonValue>,
    ) -> bool
    where
        T: UiBindingStoredValue,
    {
        let changed = self
            .data
            .get(key)
            .and_then(|entry| entry.value.as_ref())
            .is_none_or(|current| !current.value.equals(&value));

        self.known_types.insert(key.to_string());
        self.known_types.insert(type_name.to_string());
        self.known_types.insert(type_path.to_string());

        if !changed {
            return false;
        }

        let next_revision = self.revision + 1;
        self.data.insert(
            key.to_string(),
            UiBindingEntry {
                type_id: TypeId::of::<T>(),
                type_name,
                type_path,
                value: Some(UiBindingValue {
                    type_id: TypeId::of::<T>(),
                    type_name,
                    value: Box::new(value),
                    json,
                }),
                revision: next_revision,
            },
        );
        self.revision = next_revision;
        true
    }

    fn set_direct_json(&mut self, key: &str, value: JsonValue) -> bool {
        if let Some(changed) = self.set_known_primitive_json(key, &value) {
            return changed;
        }

        self.set_json_projection(key, value)
    }

    fn set_known_primitive_json(&mut self, key: &str, value: &JsonValue) -> Option<bool> {
        let entry = self.data.get(key)?;
        let type_id = entry.type_id();

        macro_rules! set_unsigned {
            ($ty:ty) => {
                if type_id == TypeId::of::<$ty>() {
                    return Some(json_to_u128(value).map_or(false, |value| {
                        <$ty>::try_from(value)
                            .ok()
                            .is_some_and(|value| self.set::<$ty>(key, value))
                    }));
                }
            };
        }

        macro_rules! set_signed {
            ($ty:ty) => {
                if type_id == TypeId::of::<$ty>() {
                    return Some(json_to_i128(value).map_or(false, |value| {
                        <$ty>::try_from(value)
                            .ok()
                            .is_some_and(|value| self.set::<$ty>(key, value))
                    }));
                }
            };
        }

        set_unsigned!(u8);
        set_unsigned!(u16);
        set_unsigned!(u32);
        set_unsigned!(u64);
        set_unsigned!(u128);
        set_unsigned!(usize);
        set_signed!(i8);
        set_signed!(i16);
        set_signed!(i32);
        set_signed!(i64);
        set_signed!(i128);
        set_signed!(isize);

        if type_id == TypeId::of::<bool>() {
            return Some(json_to_bool(value).is_some_and(|value| self.set::<bool>(key, value)));
        }
        if type_id == TypeId::of::<String>() {
            return Some(self.set::<String>(key, json_to_string(value)));
        }
        if type_id == TypeId::of::<f32>() {
            return Some(
                json_to_f64(value).is_some_and(|value| self.set::<f32>(key, value as f32)),
            );
        }
        if type_id == TypeId::of::<f64>() {
            return Some(json_to_f64(value).is_some_and(|value| self.set::<f64>(key, value)));
        }
        if type_id == TypeId::of::<JsonValue>() {
            return Some(self.set::<JsonValue>(key, value.clone()));
        }

        None
    }

    fn resolve_binding_root(&self, root: &str) -> String {
        if self.data.contains_key(root) {
            return root.to_string();
        }

        let mut matches = self
            .data
            .iter()
            .filter(|(key, entry)| binding_root_matches(root, key, entry))
            .map(|(key, _entry)| key.as_str())
            .collect::<Vec<_>>();
        matches.sort_unstable();

        matches.first().copied().unwrap_or(root).to_string()
    }

    fn set_json_projection(&mut self, key: &str, value: JsonValue) -> bool {
        if let Some(entry) = self.data.get_mut(key) {
            let Some(stored) = entry.value.as_mut() else {
                return self.set::<JsonValue>(key, value);
            };

            if stored.json.as_ref() == Some(&value) {
                return false;
            }

            stored.json = Some(value);
            let next_revision = self.revision + 1;
            entry.revision = next_revision;
            self.revision = next_revision;
            self.known_types.insert(key.to_string());
            return true;
        }

        self.set::<JsonValue>(key, value)
    }

    fn bump_revision(&mut self) {
        self.revision += 1;
    }
}

fn split_binding_path(path: &str) -> Option<(&str, Vec<String>)> {
    let mut parts = path.split('.');
    let root = parts.next()?.trim();
    if root.is_empty() {
        return None;
    }

    let mut tail = Vec::new();
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }
        tail.push(part.to_string());
    }

    Some((root, tail))
}

fn binding_root_matches(root: &str, key: &str, entry: &UiBindingEntry) -> bool {
    to_template_alias(key) == root
        || to_template_alias(entry.type_name()) == root
        || to_template_alias(simple_type_name(entry.type_path())) == root
}

fn resolve_json_tail(mut value: JsonValue, tail: &[String]) -> Option<JsonValue> {
    for segment in tail {
        match value {
            JsonValue::Object(map) => {
                value = map.get(segment)?.clone();
            }
            JsonValue::Array(items) => {
                let index = segment.parse::<usize>().ok()?;
                value = items.get(index)?.clone();
            }
            _ => return None,
        }
    }
    Some(value)
}

fn set_json_tail(value: &mut JsonValue, tail: &[String], next: JsonValue) -> bool {
    if tail.is_empty() {
        let changed = *value != next;
        *value = next;
        return changed;
    }

    if !value.is_object() {
        *value = JsonValue::Object(Default::default());
    }

    let JsonValue::Object(map) = value else {
        return false;
    };

    let entry = map
        .entry(tail[0].clone())
        .or_insert_with(|| JsonValue::Object(Default::default()));
    set_json_tail(entry, &tail[1..], next)
}

fn json_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::String(value) => value.clone(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::Null => String::new(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}

fn json_to_bool(value: &JsonValue) -> Option<bool> {
    match value {
        JsonValue::Bool(value) => Some(*value),
        JsonValue::Number(value) => Some(value.as_f64()? != 0.0),
        JsonValue::String(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn json_to_u128(value: &JsonValue) -> Option<u128> {
    match value {
        JsonValue::Number(value) => value
            .as_u64()
            .map(u128::from)
            .or_else(|| value.as_i64().and_then(|value| u128::try_from(value).ok())),
        JsonValue::String(value) => value.trim().parse::<u128>().ok(),
        JsonValue::Bool(value) => Some(if *value { 1 } else { 0 }),
        _ => None,
    }
}

fn json_to_i128(value: &JsonValue) -> Option<i128> {
    match value {
        JsonValue::Number(value) => value
            .as_i64()
            .map(i128::from)
            .or_else(|| value.as_u64().and_then(|value| i128::try_from(value).ok())),
        JsonValue::String(value) => value.trim().parse::<i128>().ok(),
        JsonValue::Bool(value) => Some(if *value { 1 } else { 0 }),
        _ => None,
    }
}

fn json_to_f64(value: &JsonValue) -> Option<f64> {
    match value {
        JsonValue::Number(value) => value.as_f64(),
        JsonValue::String(value) => value.trim().parse::<f64>().ok(),
        JsonValue::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Registers all `#[derive(BeuStore)]` types collected by `inventory`.
pub fn register_beu_stores(world: &mut World) {
    let mut store = world.resource_mut::<UiBindingStore>();

    for registration in inventory::iter::<UiBindingStoreRegistration> {
        (registration.register)(&mut store);
    }
}

/// Mirrors serializable store values into the template shared-value resource.
///
/// This keeps framework stores visible to template interpolation without
/// coupling store behavior to language or localization systems.
pub fn sync_ui_binding_store_values(world: &mut World) {
    let Some(store) = world.get_resource::<UiBindingStore>() else {
        return;
    };

    let known_types = store
        .known_types()
        .map(str::to_string)
        .collect::<Vec<String>>();
    let values = store.template_values();

    let mut shared = world.resource_mut::<UiSharedValues>();
    let mut changed = false;

    for known in known_types {
        changed |= shared.known_types.insert(known);
    }

    for (key, value) in values {
        if shared.values.get(&key) != Some(&value) {
            shared.values.insert(key, value);
            changed = true;
        }
    }

    if changed {
        debug!("UiBindingStore values synced to template shared values");
    }
}

/// Run condition that is true only when [`UiBindingStore::revision`] changed.
///
/// This is stricter than Bevy's resource-change check because it ignores
/// `ResMut<UiBindingStore>` accesses that do not store a different value.
pub fn ui_binding_store_changed(
    store: Option<Res<UiBindingStore>>,
    mut last_revision: Local<u64>,
) -> bool {
    let Some(store) = store else {
        return false;
    };

    if store.revision() == *last_revision {
        return false;
    }

    *last_revision = store.revision();
    true
}

/// Compiles an HTML template in extended framework mode.
///
/// Current base behavior:
/// - HTML passes through unchanged.
/// - If the file looks like `*.component.html` under `assets_component_root`,
///   an inferred Rust component path is returned.
#[cfg(feature = "extended-framework")]
pub fn compile_framework_template(
    template_html: &str,
    source_path: &str,
    config: &ExtendedFrameworkConfiguration,
) -> FrameworkCompileResult {
    compile_framework_template_with_router(template_html, source_path, config, None)
}

/// Compiles an HTML template and resolves `<router-outlet>` against the active route.
#[cfg(feature = "extended-framework")]
pub fn compile_framework_template_with_router(
    template_html: &str,
    source_path: &str,
    config: &ExtendedFrameworkConfiguration,
    router: Option<&Router>,
) -> FrameworkCompileResult {
    let source = normalize_source_path(source_path);
    let mut html = template_html.to_string();
    let mut component_controllers = Vec::new();
    if source == normalize_source_path(&config.index_html_file) {
        let route_components = router.map(registered_route_components).unwrap_or_default();
        component_controllers =
            compile_index_template(&mut html, config, router, route_components.as_slice())
                .unwrap_or_else(|err| {
                    panic!("extended-framework compile failed for index.html: {err}")
                });
    }

    FrameworkCompileResult {
        html,
        inferred_controller: infer_component_controller_path(source_path, config),
        component_controllers,
    }
}

/// Infers a Rust component source path from an HTML component source path.
///
/// Example:
/// - HTML: `assets/ui/bevy_ang/hud/hud.component.html`
/// - Rust: `src/packages/hud.component.rs`
#[cfg(feature = "extended-framework")]
pub fn infer_component_controller_path(
    source_path: &str,
    config: &ExtendedFrameworkConfiguration,
) -> Option<String> {
    let source = normalize_source_path(source_path);
    let root = normalize_root(&config.assets_component_root);

    if root.is_empty() {
        return None;
    }

    let expected_prefix = format!("{root}/");
    if !source.starts_with(&expected_prefix) {
        return None;
    }

    let file_name = source.rsplit('/').next()?;
    if !file_name.ends_with(".component.html") {
        return None;
    }

    let rust_file = file_name
        .strip_suffix(".html")
        .map(|name| format!("{name}.rs"))?;
    let rust_root = normalize_root(&config.rust_component_root);
    if rust_root.is_empty() {
        return Some(rust_file);
    }

    Some(format!("{rust_root}/{rust_file}"))
}

/// Handles `normalize_source_path` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn normalize_source_path(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    if let Some(rest) = normalized.strip_prefix("assets/") {
        normalized = rest.to_string();
    }
    normalized.trim_matches('/').to_string()
}

/// Handles `normalize_root` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn normalize_root(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    if let Some(rest) = normalized.strip_prefix("assets/") {
        normalized = rest.to_string();
    }
    normalized.trim_matches('/').to_string()
}

/// Handles `compile_index_template` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn compile_index_template(
    index_html: &mut String,
    config: &ExtendedFrameworkConfiguration,
    router: Option<&Router>,
    route_components: &[String],
) -> Result<Vec<String>, String> {
    let defs = load_component_definitions(config)?;
    validate_component_assets(&defs, config)?;
    let has_router_outlet = has_router_outlet(index_html)?;

    if let Some(router) = router {
        replace_router_outlets(index_html, router, &defs)?;
    }

    let mut used_style_hrefs = if has_router_outlet {
        collect_route_component_style_hrefs(route_components, &defs, config)
    } else {
        BTreeSet::new()
    };
    let mut used_component_controllers: BTreeSet<String> = BTreeSet::new();

    for _ in 0..16 {
        let mut replaced = false;

        for def in &defs {
            let component_html = load_component_template_html(def, config)?;
            let tag_name = regex::escape(&def.template_name);

            let full_tag_re = Regex::new(&format!(
                r"(?is)<\s*{tag}\b[^>]*>.*?</\s*{tag}\s*>",
                tag = tag_name
            ))
            .map_err(|err| format!("invalid regex for tag `{}`: {err}", def.template_name))?;
            let self_closing_re =
                Regex::new(&format!(r"(?is)<\s*{tag}\b[^>]*/\s*>", tag = tag_name)).map_err(
                    |err| format!("invalid regex for tag `{}`: {err}", def.template_name),
                )?;

            if full_tag_re.is_match(index_html) || self_closing_re.is_match(index_html) {
                *index_html = full_tag_re
                    .replace_all(index_html, regex::NoExpand(component_html.as_str()))
                    .to_string();
                *index_html = self_closing_re
                    .replace_all(index_html, regex::NoExpand(component_html.as_str()))
                    .to_string();
                for style in &def.styles {
                    used_style_hrefs.insert(build_component_style_href(
                        &config.assets_component_root,
                        &def.source_dir_rel,
                        style,
                    ));
                }
                used_component_controllers.insert(build_component_controller_path(
                    &config.rust_component_root,
                    &def.source_dir_rel,
                    &def.template_file,
                ));
                replaced = true;
            }
        }

        if !replaced {
            break;
        }
    }

    inject_component_styles(index_html, used_style_hrefs);
    Ok(used_component_controllers.into_iter().collect())
}

/// Handles `registered_route_components` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn registered_route_components(router: &Router) -> Vec<String> {
    let mut components = router
        .routes()
        .routes()
        .iter()
        .map(|route| route.component.clone())
        .collect::<BTreeSet<_>>();

    if let Some(fallback) = router.routes().fallback_component() {
        components.insert(fallback.to_string());
    }

    components.into_iter().collect()
}

/// Handles `has_router_outlet` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn has_router_outlet(html: &str) -> Result<bool, String> {
    let full_outlet_re = Regex::new(r"(?is)<\s*router-outlet\b[^>]*>.*?</\s*router-outlet\s*>")
        .map_err(|err| format!("invalid router-outlet regex: {err}"))?;
    let self_closing_outlet_re = Regex::new(r"(?is)<\s*router-outlet\b[^>]*/\s*>")
        .map_err(|err| format!("invalid router-outlet regex: {err}"))?;

    Ok(full_outlet_re.is_match(html) || self_closing_outlet_re.is_match(html))
}

/// Handles `collect_route_component_style_hrefs` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn collect_route_component_style_hrefs(
    route_components: &[String],
    defs: &[crate::component::ComponentDefinition],
    config: &ExtendedFrameworkConfiguration,
) -> BTreeSet<String> {
    let route_components = route_components
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut hrefs = BTreeSet::new();

    for def in defs {
        if !route_components.contains(def.template_name.as_str()) {
            continue;
        }

        for style in &def.styles {
            hrefs.insert(build_component_style_href(
                &config.assets_component_root,
                &def.source_dir_rel,
                style,
            ));
        }
    }

    hrefs
}

/// Handles `replace_router_outlets` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn replace_router_outlets(
    html: &mut String,
    router: &Router,
    defs: &[crate::component::ComponentDefinition],
) -> Result<(), String> {
    let full_outlet_re = Regex::new(r"(?is)<\s*router-outlet\b[^>]*>.*?</\s*router-outlet\s*>")
        .map_err(|err| format!("invalid router-outlet regex: {err}"))?;
    let self_closing_outlet_re = Regex::new(r"(?is)<\s*router-outlet\b[^>]*/\s*>")
        .map_err(|err| format!("invalid router-outlet regex: {err}"))?;

    if !full_outlet_re.is_match(html) && !self_closing_outlet_re.is_match(html) {
        return Ok(());
    }

    let Some(active_route_component) = router.active_component() else {
        return Ok(());
    };

    if !component_is_registered(active_route_component, defs) {
        return Err(format!(
            "Route component `{active_route_component}` is not registered as a component template_name."
        ));
    }

    let keep_alive_routes = router
        .routes()
        .routes()
        .iter()
        .filter(|route| route.keep_alive)
        .collect::<Vec<_>>();

    let routed_component = if keep_alive_routes.is_empty() {
        format!("<{active_route_component}></{active_route_component}>")
    } else {
        build_keep_alive_router_outlet(active_route_component, keep_alive_routes.as_slice(), defs)?
    };
    *html = full_outlet_re
        .replace_all(html, regex::NoExpand(routed_component.as_str()))
        .to_string();
    *html = self_closing_outlet_re
        .replace_all(html, regex::NoExpand(routed_component.as_str()))
        .to_string();
    Ok(())
}

#[cfg(feature = "extended-framework")]
fn component_is_registered(
    component: &str,
    defs: &[crate::component::ComponentDefinition],
) -> bool {
    defs.iter()
        .any(|definition| definition.template_name == component)
}

#[cfg(feature = "extended-framework")]
fn build_keep_alive_router_outlet(
    active_route_component: &str,
    keep_alive_routes: &[&crate::routing::Route],
    defs: &[crate::component::ComponentDefinition],
) -> Result<String, String> {
    let mut html = String::new();
    let mut active_is_keep_alive = false;

    for route in keep_alive_routes {
        if !component_is_registered(&route.component, defs) {
            return Err(format!(
                "Keep-alive route component `{}` is not registered as a component template_name.",
                route.component
            ));
        }

        let active = route.component == active_route_component;
        active_is_keep_alive |= active;
        html.push_str(&route_component_wrapper(&route.component, active));
    }

    if !active_is_keep_alive {
        html.push_str(&route_component_wrapper(active_route_component, true));
    }

    Ok(html)
}

#[cfg(feature = "extended-framework")]
fn route_component_wrapper(component: &str, active: bool) -> String {
    let style = if active {
        "display: flex; width: 100%; height: 100%; flex-direction: column;"
    } else {
        "display: none;"
    };
    let class = if active {
        "beu-route beu-route-active"
    } else {
        "beu-route beu-route-cached"
    };

    format!(
        r#"<div class="{class}" style="{style}">
<{component}></{component}>
</div>"#,
    )
}

/// Handles `inject_component_styles` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn inject_component_styles(html: &mut String, style_hrefs: BTreeSet<String>) {
    if style_hrefs.is_empty() {
        return;
    }

    let mut links = String::new();

    for href in style_hrefs {
        let marker = format!("href=\"{href}\"");
        if html.contains(&marker) {
            continue;
        }
        links.push_str(&format!("<link rel=\"stylesheet\" href=\"{href}\">\n"));
    }

    if links.is_empty() {
        return;
    }

    let lower = html.to_ascii_lowercase();
    if let Some(pos) = lower.find("</head>") {
        html.insert_str(pos, &links);
    } else {
        html.insert_str(0, &links);
    }
}

/// Handles `build_component_style_href` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn build_component_style_href(component_root: &str, source_dir_rel: &str, style: &str) -> String {
    let root = normalize_root(component_root);
    let style = normalize_root(style);
    let source_dir_rel = normalize_root(source_dir_rel);

    let relative = if style.contains('/') {
        style
    } else if source_dir_rel.is_empty() {
        style
    } else {
        format!("{source_dir_rel}/{style}")
    };

    if root.is_empty() {
        relative
    } else {
        format!("{root}/{relative}")
    }
}

#[cfg(feature = "extended-framework")]
fn build_component_controller_path(
    rust_root: &str,
    source_dir_rel: &str,
    template_file: &str,
) -> String {
    let rust_root = normalize_filesystem_root(rust_root);
    let source_dir_rel = normalize_root(source_dir_rel);
    let template_file = normalize_root(template_file);
    let rust_file = template_file
        .strip_suffix(".html")
        .map(|base| format!("{base}.rs"))
        .unwrap_or(template_file);

    let relative = if rust_file.contains('/') {
        rust_file
    } else if source_dir_rel.is_empty() {
        rust_file
    } else {
        format!("{source_dir_rel}/{rust_file}")
    };

    if rust_root.is_empty() {
        relative
    } else {
        format!("{rust_root}/{relative}")
    }
}

#[cfg(feature = "extended-framework")]
fn normalize_filesystem_root(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    normalized.trim_end_matches('/').to_string()
}

fn simple_type_name(path: &'static str) -> &'static str {
    path.rsplit("::").next().unwrap_or(path)
}

fn to_template_alias(type_name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in type_name.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                out.push('_');
            }
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else {
            out.push(ch);
        }
    }
    out
}
