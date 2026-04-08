use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::io::CssAsset;
use crate::styles::components::UiStyle;
use crate::styles::parser::{collect_root_css_vars, load_css, load_css_with_root_vars};
use crate::styles::{
    AnimationKeyframe, CssClass, CssID, CssSource, ExistingCssIDs, ParsedCss, StylePair, TagName,
};

// Marks entities as needing CSS re-apply on hot reload
use crate::html::reload::CssDirty;

static PARSED_CSS_CACHE: Lazy<RwLock<HashMap<AssetId<CssAsset>, ParsedCss>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static ROOT_CSS_VARS_CACHE: Lazy<RwLock<HashMap<AssetId<CssAsset>, HashMap<String, String>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static PARSED_CSS_WITH_VARS_CACHE: Lazy<RwLock<HashMap<ParsedCssWithVarsKey, ParsedCss>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct ParsedCssWithVarsKey {
    asset_id: AssetId<CssAsset>,
    vars_hash: u64,
}

/// Tracks which entities reference which CSS assets.
#[derive(Resource, Default)]
pub struct CssUsers {
    pub users: HashMap<AssetId<CssAsset>, HashSet<Entity>>,
    entity_assets: HashMap<Entity, Vec<AssetId<CssAsset>>>,
}

/// Stores the last known primary-window size to detect breakpoint changes.
#[derive(Resource, Debug, Clone, Copy)]
struct CssViewportTracker {
    width: f32,
    height: f32,
}

impl Default for CssViewportTracker {
    fn default() -> Self {
        Self {
            width: -1.0,
            height: -1.0,
        }
    }
}

/// Plugin that keeps UI styles in sync with CSS assets.
pub struct CssService;

impl Plugin for CssService {
    /// Registers resources and systems for CSS processing.
    fn build(&self, app: &mut App) {
        app.init_resource::<ExistingCssIDs>();
        app.init_resource::<CssUsers>();
        #[cfg(not(all(feature = "wasm-default", target_arch = "wasm32")))]
        app.init_resource::<CssViewportTracker>();
        #[cfg(all(feature = "wasm-default", target_arch = "wasm32"))]
        app.add_systems(
            Update,
            (
                invalidate_css_cache_on_asset_change,
                update_css_users_index,
                apply_css_to_entities_legacy,
            )
                .chain(),
        );
        #[cfg(not(all(feature = "wasm-default", target_arch = "wasm32")))]
        app.add_systems(
            Update,
            (
                invalidate_css_cache_on_asset_change,
                update_css_users_index,
                mark_css_users_dirty_on_viewport_change,
                apply_css_to_entities,
            )
                .chain(),
        );
    }
}

/// Invalidates cached parsed CSS when assets change.
fn invalidate_css_cache_on_asset_change(mut ev: MessageReader<AssetEvent<CssAsset>>) {
    for e in ev.read() {
        match e {
            AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                if let Ok(mut cache) = PARSED_CSS_CACHE.write() {
                    cache.remove(id);
                }
                if let Ok(mut cache) = ROOT_CSS_VARS_CACHE.write() {
                    cache.remove(id);
                }
                if let Ok(mut cache) = PARSED_CSS_WITH_VARS_CACHE.write() {
                    cache.retain(|key, _| key.asset_id != *id);
                }
            }
            _ => {}
        }
    }
}

fn get_or_parse_css_by_id(
    asset_id: AssetId<CssAsset>,
    css_assets: &Assets<CssAsset>,
) -> Option<ParsedCss> {
    if let Some(cached) = PARSED_CSS_CACHE
        .read()
        .ok()
        .and_then(|cache| cache.get(&asset_id).cloned())
    {
        return Some(cached);
    }

    let css_asset = css_assets.get(asset_id)?;
    let parsed = load_css(&css_asset.text);
    if let Ok(mut cache) = PARSED_CSS_CACHE.write() {
        cache.insert(asset_id, parsed.clone());
    }
    Some(parsed)
}

fn get_or_parse_css(handle: &Handle<CssAsset>, css_assets: &Assets<CssAsset>) -> Option<ParsedCss> {
    get_or_parse_css_by_id(handle.id(), css_assets)
}

fn hash_root_vars(root_vars: &HashMap<String, String>) -> u64 {
    let mut entries: Vec<(&String, &String)> = root_vars.iter().collect();
    entries.sort_unstable_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));

    let mut hasher = DefaultHasher::new();
    for (key, value) in entries {
        key.hash(&mut hasher);
        value.hash(&mut hasher);
    }
    hasher.finish()
}

fn get_or_collect_root_vars(
    handle: &Handle<CssAsset>,
    css_assets: &Assets<CssAsset>,
) -> Option<HashMap<String, String>> {
    let asset_id = handle.id();

    if let Some(cached) = ROOT_CSS_VARS_CACHE
        .read()
        .ok()
        .and_then(|cache| cache.get(&asset_id).cloned())
    {
        return Some(cached);
    }

    let css_asset = css_assets.get(handle)?;
    let vars = collect_root_css_vars(&css_asset.text);

    if let Ok(mut cache) = ROOT_CSS_VARS_CACHE.write() {
        cache.insert(asset_id, vars.clone());
    }

    Some(vars)
}

fn get_or_parse_css_with_root_vars(
    handle: &Handle<CssAsset>,
    css_assets: &Assets<CssAsset>,
    root_vars: &HashMap<String, String>,
) -> Option<ParsedCss> {
    if root_vars.is_empty() {
        return get_or_parse_css(handle, css_assets);
    }

    let vars_hash = hash_root_vars(root_vars);
    let cache_key = ParsedCssWithVarsKey {
        asset_id: handle.id(),
        vars_hash,
    };

    if let Some(cached) = PARSED_CSS_WITH_VARS_CACHE
        .read()
        .ok()
        .and_then(|cache| cache.get(&cache_key).cloned())
    {
        return Some(cached);
    }

    let css_asset = css_assets.get(handle)?;
    let parsed = load_css_with_root_vars(&css_asset.text, root_vars);

    if let Ok(mut cache) = PARSED_CSS_WITH_VARS_CACHE.write() {
        cache.insert(cache_key, parsed.clone());
    }

    Some(parsed)
}

fn collect_global_root_vars_for_sources(
    sources: &[Handle<CssAsset>],
    css_assets: &Assets<CssAsset>,
) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    for handle in sources {
        let Some(extracted) = get_or_collect_root_vars(handle, css_assets) else {
            continue;
        };
        for (name, value) in extracted {
            vars.insert(name, value);
        }
    }

    vars
}

fn remove_entity_from_css_users(css_users: &mut CssUsers, entity: Entity) {
    let Some(previous_assets) = css_users.entity_assets.remove(&entity) else {
        return;
    };

    for asset_id in previous_assets {
        let should_remove = if let Some(set) = css_users.users.get_mut(&asset_id) {
            set.remove(&entity);
            set.is_empty()
        } else {
            false
        };

        if should_remove {
            css_users.users.remove(&asset_id);
        }
    }
}

/// Updates the reverse index of entities using each CSS asset.
fn update_css_users_index(
    mut css_users: ResMut<CssUsers>,
    query_changed: Query<(Entity, &CssSource), Or<(Added<CssSource>, Changed<CssSource>)>>,
    mut removed_sources: RemovedComponents<CssSource>,
) {
    for entity in removed_sources.read() {
        remove_entity_from_css_users(&mut css_users, entity);
    }

    for (entity, css_source) in query_changed.iter() {
        remove_entity_from_css_users(&mut css_users, entity);

        let mut current_assets = Vec::with_capacity(css_source.0.len());

        for handle in &css_source.0 {
            let asset_id = handle.id();
            css_users.users.entry(asset_id).or_default().insert(entity);
            if !current_assets.contains(&asset_id) {
                current_assets.push(asset_id);
            }
        }

        if !current_assets.is_empty() {
            css_users.entity_assets.insert(entity, current_assets);
        }
    }
}

/// Marks only affected CSS users dirty when breakpoint/media-query matches change.
fn mark_css_users_dirty_on_viewport_change(
    mut commands: Commands,
    mut viewport_tracker: ResMut<CssViewportTracker>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    css_assets: Res<Assets<CssAsset>>,
    css_users: Res<CssUsers>,
) {
    let Some(next_viewport) = resolve_breakpoint_viewport(&window_query) else {
        return;
    };

    let viewport_changed = (viewport_tracker.width - next_viewport.x).abs() > 0.5
        || (viewport_tracker.height - next_viewport.y).abs() > 0.5;

    if !viewport_changed {
        return;
    }

    let prev_viewport = Vec2::new(viewport_tracker.width, viewport_tracker.height);

    let affected_assets = if prev_viewport.x < 0.0 || prev_viewport.y < 0.0 {
        // Initial resize tracking warm-up: startup CssSource insertion already triggers CSS apply.
        HashSet::new()
    } else {
        collect_assets_with_changed_media_matches(
            &css_users,
            &css_assets,
            prev_viewport,
            next_viewport,
        )
    };

    viewport_tracker.width = next_viewport.x;
    viewport_tracker.height = next_viewport.y;

    if affected_assets.is_empty() {
        return;
    }

    let mut affected_entities = HashSet::new();
    for asset_id in affected_assets {
        if let Some(users) = css_users.users.get(&asset_id) {
            affected_entities.extend(users.iter().copied());
        }
    }

    for entity in affected_entities {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(CssDirty);
        }
    }
}

/// Returns the viewport used for media-query breakpoints.
///
/// Feature behavior:
/// - `wasm-breakpoints` overrides `css-breakpoints` when enabled.
/// - `css-breakpoints` reads from Bevy's primary window (desktop/default).
/// - no active breakpoint feature returns `None`.
#[cfg(all(feature = "wasm-breakpoints", target_arch = "wasm32"))]
fn resolve_breakpoint_viewport(
    _window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let window = web_sys::window()?;
    let width = window.inner_width().ok()?.as_f64()? as f32;
    let height = window.inner_height().ok()?.as_f64()? as f32;
    Some(Vec2::new(width, height))
}

#[cfg(all(feature = "wasm-breakpoints", not(target_arch = "wasm32")))]
fn resolve_breakpoint_viewport(window_query: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    Some(Vec2::new(
        window.resolution.width(),
        window.resolution.height(),
    ))
}

#[cfg(all(not(feature = "wasm-breakpoints"), feature = "css-breakpoints"))]
fn resolve_breakpoint_viewport(window_query: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    Some(Vec2::new(
        window.resolution.width(),
        window.resolution.height(),
    ))
}

#[cfg(all(not(feature = "wasm-breakpoints"), not(feature = "css-breakpoints")))]
fn resolve_breakpoint_viewport(
    _window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    None
}

/// Returns the CSS asset ids whose media rules change match state between two viewports.
pub(crate) fn collect_assets_with_changed_media_matches(
    css_users: &CssUsers,
    css_assets: &Assets<CssAsset>,
    prev_viewport: Vec2,
    next_viewport: Vec2,
) -> HashSet<AssetId<CssAsset>> {
    let mut affected_assets = HashSet::new();

    for asset_id in css_users.users.keys().copied() {
        let Some(parsed) = get_or_parse_css_by_id(asset_id, css_assets) else {
            continue;
        };

        let media_changed = parsed.styles.values().any(|style| {
            let Some(media) = style.media.as_ref() else {
                return false;
            };

            media.matches_viewport(prev_viewport) != media.matches_viewport(next_viewport)
        });

        if media_changed {
            affected_assets.insert(asset_id);
        }
    }

    affected_assets
}

/// Applies merged CSS styles to entities that are dirty or affected by changes.
fn apply_css_to_entities(
    mut commands: Commands,

    css_assets: Res<Assets<CssAsset>>,
    mut css_events: MessageReader<AssetEvent<CssAsset>>,
    css_users: Res<CssUsers>,

    // CHANGED: include entities that got CssDirty added
    query_changed_source: Query<
        (Entity, Option<&CssDirty>),
        Or<(
            Changed<CssSource>,
            Added<CssSource>,
            Added<CssDirty>,
            Changed<CssClass>,
            Changed<CssID>,
            Changed<TagName>,
            Changed<ChildOf>,
        )>,
    >,
    query_all_source: Query<
        (
            Entity,
            &CssSource,
            Option<&CssID>,
            Option<&CssClass>,
            Option<&TagName>,
            Option<&ChildOf>,
            Option<&CssDirty>,
        ),
        With<CssSource>,
    >,
    window_query: Query<&Window, With<PrimaryWindow>>,

    parent_query: Query<(
        Option<&CssID>,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&ChildOf>,
    )>,

    style_query: Query<Option<&UiStyle>>,
) {
    let mut dirty: HashSet<Entity> = HashSet::new();

    // Entities whose CssSource changed / was added / got CssDirty
    for (e, _) in query_changed_source.iter() {
        dirty.insert(e);
    }

    // Entities affected by CssAsset events (via CssUsers index)
    for ev in css_events.read() {
        let id = match ev {
            AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                Some(*id)
            }
            _ => None,
        };
        let Some(id) = id else { continue };

        if let Some(users) = css_users.users.get(&id) {
            dirty.extend(users.iter().copied());
        }
    }

    if dirty.is_empty() {
        return;
    }

    let viewport = resolve_breakpoint_viewport(&window_query).unwrap_or(Vec2::ZERO);

    for entity in dirty {
        let Ok((_, css_source, id, class, tag, parent, dirty_marker)) =
            query_all_source.get(entity)
        else {
            continue;
        };

        let merged_css = load_and_merge_styles_from_assets(
            &css_source.0,
            &css_assets,
            id,
            class,
            tag,
            parent,
            &parent_query,
            viewport,
        );

        let primary_css = css_source.0.first().cloned().unwrap_or_default();

        let final_style = UiStyle {
            css: primary_css,
            styles: merged_css.styles,
            keyframes: merged_css.keyframes,
            active_style: None,
        };

        match style_query.get(entity) {
            Ok(Some(existing))
                if existing.styles != final_style.styles
                    || existing.keyframes != final_style.keyframes =>
            {
                commands
                    .entity(entity)
                    .queue_silenced(move |mut ew: EntityWorldMut| {
                        ew.insert(final_style);
                        ew.remove::<CssDirty>();
                    });
            }
            Ok(None) => {
                commands
                    .entity(entity)
                    .queue_silenced(move |mut ew: EntityWorldMut| {
                        ew.insert(final_style);
                        ew.remove::<CssDirty>();
                    });
            }
            _ => {
                if dirty_marker.is_some() {
                    commands
                        .entity(entity)
                        .queue_silenced(|mut ew: EntityWorldMut| {
                            ew.remove::<CssDirty>();
                        });
                }
            }
        }
    }
}

/// Legacy CSS apply path used for WASM compatibility mode.
///
/// This intentionally mirrors the pre-breakpoint-refresh behavior.
#[cfg(all(feature = "wasm-default", target_arch = "wasm32"))]
fn apply_css_to_entities_legacy(
    mut commands: Commands,
    css_assets: Res<Assets<CssAsset>>,
    mut css_events: MessageReader<AssetEvent<CssAsset>>,
    css_users: Res<CssUsers>,
    query_changed_source: Query<
        (Entity, Option<&CssDirty>),
        Or<(
            Changed<CssSource>,
            Added<CssSource>,
            Added<CssDirty>,
            Changed<CssClass>,
            Changed<CssID>,
            Changed<TagName>,
            Changed<ChildOf>,
        )>,
    >,
    query_all_source: Query<
        (
            Entity,
            &CssSource,
            Option<&CssID>,
            Option<&CssClass>,
            Option<&TagName>,
            Option<&ChildOf>,
            Option<&CssDirty>,
        ),
        With<CssSource>,
    >,
    parent_query: Query<(
        Option<&CssID>,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&ChildOf>,
    )>,
    style_query: Query<Option<&UiStyle>>,
) {
    let mut dirty: HashSet<Entity> = HashSet::new();

    for (e, _) in query_changed_source.iter() {
        dirty.insert(e);
    }

    for ev in css_events.read() {
        let id = match ev {
            AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                Some(*id)
            }
            _ => None,
        };
        let Some(id) = id else { continue };

        if let Some(users) = css_users.users.get(&id) {
            dirty.extend(users.iter().copied());
        }
    }

    if dirty.is_empty() {
        return;
    }

    for entity in dirty {
        let Ok((_, css_source, id, class, tag, parent, dirty_marker)) =
            query_all_source.get(entity)
        else {
            continue;
        };

        let merged_css = load_and_merge_styles_from_assets_legacy(
            &css_source.0,
            &css_assets,
            id,
            class,
            tag,
            parent,
            &parent_query,
        );

        let primary_css = css_source.0.first().cloned().unwrap_or_default();
        let final_style = UiStyle {
            css: primary_css,
            styles: merged_css.styles,
            keyframes: merged_css.keyframes,
            active_style: None,
        };

        match style_query.get(entity) {
            Ok(Some(existing))
                if existing.styles != final_style.styles
                    || existing.keyframes != final_style.keyframes =>
            {
                commands
                    .entity(entity)
                    .queue_silenced(move |mut ew: EntityWorldMut| {
                        ew.insert(final_style);
                        ew.remove::<CssDirty>();
                    });
            }
            Ok(None) => {
                commands
                    .entity(entity)
                    .queue_silenced(move |mut ew: EntityWorldMut| {
                        ew.insert(final_style);
                        ew.remove::<CssDirty>();
                    });
            }
            _ => {
                if dirty_marker.is_some() {
                    commands
                        .entity(entity)
                        .queue_silenced(|mut ew: EntityWorldMut| {
                            ew.remove::<CssDirty>();
                        });
                }
            }
        }
    }
}

/// Loads and merges CSS styles from multiple sources with selector matching.
fn load_and_merge_styles_from_assets(
    sources: &[Handle<CssAsset>],
    css_assets: &Assets<CssAsset>,
    id: Option<&CssID>,
    class: Option<&CssClass>,
    tag: Option<&TagName>,
    parent: Option<&ChildOf>,
    parent_query: &Query<(
        Option<&CssID>,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&ChildOf>,
    )>,
    viewport: Vec2,
) -> ParsedCss {
    let mut merged_styles: HashMap<String, StylePair> = HashMap::new();
    let mut merged_keyframes: HashMap<String, Vec<AnimationKeyframe>> = HashMap::new();
    let global_root_vars = collect_global_root_vars_for_sources(sources, css_assets);

    for (index, handle) in sources.iter().enumerate() {
        let Some(parsed_map) =
            get_or_parse_css_with_root_vars(handle, css_assets, &global_root_vars)
        else {
            continue;
        };

        for (selector_key, new_style) in parsed_map.styles.iter() {
            if let Some(media) = &new_style.media {
                if !media.matches_viewport(viewport) {
                    continue;
                }
            }

            let selector = if new_style.selector.is_empty() {
                selector_key.as_str()
            } else {
                new_style.selector.as_str()
            };
            let selector_parts = parse_selector_steps(selector);

            if matches_selector_chain(&selector_parts, id, class, tag, parent, parent_query) {
                merged_styles
                    .entry(selector_key.clone())
                    .and_modify(|existing| {
                        existing.normal.merge(&new_style.normal);
                        existing.important.merge(&new_style.important);
                        existing.origin = index; // Update origin to the latest source
                    })
                    .or_insert_with(|| {
                        let mut s = new_style.clone();
                        s.origin = index;
                        s
                    });
            }
        }

        for (name, keyframes) in parsed_map.keyframes.iter() {
            merged_keyframes.insert(name.clone(), keyframes.clone());
        }
    }

    ParsedCss {
        styles: merged_styles,
        keyframes: merged_keyframes,
    }
}

#[cfg(all(feature = "wasm-default", target_arch = "wasm32"))]
fn load_and_merge_styles_from_assets_legacy(
    sources: &[Handle<CssAsset>],
    css_assets: &Assets<CssAsset>,
    id: Option<&CssID>,
    class: Option<&CssClass>,
    tag: Option<&TagName>,
    parent: Option<&ChildOf>,
    parent_query: &Query<(
        Option<&CssID>,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&ChildOf>,
    )>,
) -> ParsedCss {
    let mut merged_styles: HashMap<String, StylePair> = HashMap::new();
    let mut merged_keyframes: HashMap<String, Vec<AnimationKeyframe>> = HashMap::new();
    let global_root_vars = collect_global_root_vars_for_sources(sources, css_assets);

    for (index, handle) in sources.iter().enumerate() {
        let Some(parsed_map) =
            get_or_parse_css_with_root_vars(handle, css_assets, &global_root_vars)
        else {
            continue;
        };

        for (selector_key, new_style) in parsed_map.styles.iter() {
            let selector = if new_style.selector.is_empty() {
                selector_key.as_str()
            } else {
                new_style.selector.as_str()
            };
            let selector_parts = parse_selector_steps(selector);

            if matches_selector_chain(&selector_parts, id, class, tag, parent, parent_query) {
                merged_styles
                    .entry(selector_key.clone())
                    .and_modify(|existing| {
                        existing.normal.merge(&new_style.normal);
                        existing.important.merge(&new_style.important);
                        existing.origin = index;
                    })
                    .or_insert_with(|| {
                        let mut s = new_style.clone();
                        s.origin = index;
                        s
                    });
            }
        }

        for (name, keyframes) in parsed_map.keyframes.iter() {
            merged_keyframes.insert(name.clone(), keyframes.clone());
        }
    }

    ParsedCss {
        styles: merged_styles,
        keyframes: merged_keyframes,
    }
}

/// Recursively matches a selector chain against an element and its parents.
fn matches_selector_chain(
    selectors: &[SelectorStep],
    id_opt: Option<&CssID>,
    class_opt: Option<&CssClass>,
    tag_opt: Option<&TagName>,
    parent_opt: Option<&ChildOf>,
    parent_query: &Query<(
        Option<&CssID>,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&ChildOf>,
    )>,
) -> bool {
    if selectors.is_empty() {
        return true;
    }

    let mut current_parent = parent_opt;

    let current_sel = &selectors[selectors.len() - 1].selector;
    if !matches_selector(current_sel, id_opt, class_opt, tag_opt) {
        return false;
    }

    if selectors.len() == 1 {
        return true;
    }

    let mut index = selectors.len() - 1;
    while index > 0 {
        let relation = selectors[index].combinator;
        let target = &selectors[index - 1].selector;

        match relation {
            SelectorCombinator::Child => {
                let Some(parent) = current_parent else {
                    return false;
                };
                let Ok((pid, p_class, p_tag, p_parent)) = parent_query.get(parent.parent()) else {
                    return false;
                };
                if !matches_selector(target, pid, p_class, p_tag) {
                    return false;
                }
                current_parent = p_parent;
            }
            SelectorCombinator::Descendant => {
                let mut parent = current_parent;
                let mut found = false;
                while let Some(parent_entity) = parent {
                    let Ok((pid, p_class, p_tag, p_parent)) =
                        parent_query.get(parent_entity.parent())
                    else {
                        return false;
                    };
                    if matches_selector(target, pid, p_class, p_tag) {
                        current_parent = p_parent;
                        found = true;
                        break;
                    }
                    parent = p_parent;
                }
                if !found {
                    return false;
                }
            }
            SelectorCombinator::Root => {
                return false;
            }
        }

        index -= 1;
    }

    true
}

/// Matches a single selector against an element's id, class, and tag.
fn matches_selector(
    selector: &str,
    id_opt: Option<&CssID>,
    class_opt: Option<&CssClass>,
    tag_opt: Option<&TagName>,
) -> bool {
    let base = selector.split(':').next().unwrap_or(selector);

    if base == "*" {
        return true;
    }

    if let (Some(id), Some(rest)) = (id_opt, base.strip_prefix('#')) {
        if rest == id.0 {
            return true;
        }
    }

    if let Some(classes) = class_opt {
        if let Some(rest) = base.strip_prefix('.') {
            for c in &classes.0 {
                if rest == c {
                    return true;
                }
            }
        }
    }

    if let Some(tag) = tag_opt {
        if base == tag.0 {
            return true;
        }
    }

    false
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectorCombinator {
    Root,
    Descendant,
    Child,
}

#[derive(Clone, Debug)]
struct SelectorStep {
    selector: String,
    combinator: SelectorCombinator,
}

fn parse_selector_steps(selector: &str) -> Vec<SelectorStep> {
    let mut steps = Vec::new();
    let mut next_relation = SelectorCombinator::Descendant;

    for part in selector.replace('>', " > ").split_whitespace() {
        if part == ">" {
            next_relation = SelectorCombinator::Child;
            continue;
        }

        let relation = if steps.is_empty() {
            SelectorCombinator::Root
        } else {
            next_relation
        };

        steps.push(SelectorStep {
            selector: part.to_string(),
            combinator: relation,
        });

        next_relation = SelectorCombinator::Descendant;
    }

    steps
}
