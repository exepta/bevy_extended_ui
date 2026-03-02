use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use crate::io::CssAsset;
use crate::styles::components::UiStyle;
use crate::styles::parser::load_css;
use crate::styles::{
    AnimationKeyframe, CssClass, CssID, CssSource, ExistingCssIDs, ParsedCss, StylePair, TagName,
};

// Marks entities as needing CSS re-apply on hot reload
use crate::html::reload::CssDirty;

static PARSED_CSS_CACHE: Lazy<RwLock<HashMap<AssetId<CssAsset>, ParsedCss>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Tracks which entities reference which CSS assets.
#[derive(Resource, Default)]
pub struct CssUsers {
    pub users: HashMap<AssetId<CssAsset>, HashSet<Entity>>,
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
            }
            _ => {}
        }
    }
}

fn get_or_parse_css(handle: &Handle<CssAsset>, css_assets: &Assets<CssAsset>) -> Option<ParsedCss> {
    let asset_id = handle.id();

    if let Some(cached) = PARSED_CSS_CACHE
        .read()
        .ok()
        .and_then(|cache| cache.get(&asset_id).cloned())
    {
        return Some(cached);
    }

    let css_asset = css_assets.get(handle)?;
    let parsed = load_css(&css_asset.text);
    if let Ok(mut cache) = PARSED_CSS_CACHE.write() {
        cache.insert(asset_id, parsed.clone());
    }
    Some(parsed)
}

/// Updates the reverse index of entities using each CSS asset.
fn update_css_users_index(
    mut css_users: ResMut<CssUsers>,
    query_changed: Query<(Entity, &CssSource), Or<(Added<CssSource>, Changed<CssSource>)>>,
) {
    for (entity, css_source) in query_changed.iter() {
        // Remove entity from all previous sets
        for set in css_users.users.values_mut() {
            set.remove(&entity);
        }

        // Add entity to new CSS handles
        for h in &css_source.0 {
            css_users.users.entry(h.id()).or_default().insert(entity);
        }
    }
}

/// Marks all CSS users dirty when the active breakpoint viewport changes.
fn mark_css_users_dirty_on_viewport_change(
    mut commands: Commands,
    mut viewport_tracker: ResMut<CssViewportTracker>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    css_assets: Res<Assets<CssAsset>>,
    css_query: Query<(Entity, &CssSource)>,
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

    let breakpoint_changed = if prev_viewport.x < 0.0 || prev_viewport.y < 0.0 {
        // Initial resize tracking warm-up: startup CssSource insertion already triggers CSS apply.
        false
    } else {
        media_match_changed_between_viewports(&css_query, &css_assets, prev_viewport, next_viewport)
    };

    viewport_tracker.width = next_viewport.x;
    viewport_tracker.height = next_viewport.y;

    if !breakpoint_changed {
        return;
    }

    for (entity, _) in css_query.iter() {
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

/// Returns true when at least one media rule changes the match state between two viewports.
fn media_match_changed_between_viewports(
    css_query: &Query<(Entity, &CssSource)>,
    css_assets: &Assets<CssAsset>,
    prev_viewport: Vec2,
    next_viewport: Vec2,
) -> bool {
    let mut seen_assets = HashSet::new();

    for (_, css_source) in css_query.iter() {
        for handle in &css_source.0 {
            let asset_id = handle.id();
            if !seen_assets.insert(asset_id) {
                continue;
            }

            let Some(parsed) = get_or_parse_css(handle, css_assets) else {
                continue;
            };

            for style in parsed.styles.values() {
                let Some(media) = style.media.as_ref() else {
                    continue;
                };

                let old_match = media.matches_viewport(prev_viewport);
                let new_match = media.matches_viewport(next_viewport);
                if old_match != new_match {
                    return true;
                }
            }
        }
    }

    false
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

    for (index, handle) in sources.iter().enumerate() {
        let Some(parsed_map) = get_or_parse_css(handle, css_assets) else {
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

    for (index, handle) in sources.iter().enumerate() {
        let Some(parsed_map) = get_or_parse_css(handle, css_assets) else {
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
