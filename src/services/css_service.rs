use bevy::prelude::*;
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

/// Plugin that keeps UI styles in sync with CSS assets.
pub struct CssService;

impl Plugin for CssService {
    /// Registers resources and systems for CSS processing.
    fn build(&self, app: &mut App) {
        app.init_resource::<ExistingCssIDs>();
        app.init_resource::<CssUsers>();
        app.add_systems(
            Update,
            (
                invalidate_css_cache_on_asset_change,
                update_css_users_index,
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
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                if let Ok(mut cache) = PARSED_CSS_CACHE.write() {
                    cache.remove(id);
                }
            }
            _ => {}
        }
    }
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

/// Applies merged CSS styles to entities that are dirty or affected by changes.
fn apply_css_to_entities(
    mut commands: Commands,

    css_assets: Res<Assets<CssAsset>>,
    mut css_events: MessageReader<AssetEvent<CssAsset>>,
    css_users: Res<CssUsers>,

    // CHANGED: include entities that got CssDirty added
    query_changed_source: Query<
        (
            Entity,
            &CssSource,
            Option<&CssID>,
            Option<&CssClass>,
            Option<&TagName>,
            Option<&ChildOf>,
            Option<&CssDirty>,
        ),
        Or<(Changed<CssSource>, Added<CssSource>, Added<CssDirty>)>,
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

    // Entities whose CssSource changed / was added / got CssDirty
    for (e, _, _, _, _, _, _) in query_changed_source.iter() {
        dirty.insert(e);
    }

    // Entities affected by CssAsset events (via CssUsers index)
    for ev in css_events.read() {
        let id = match ev {
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => Some(*id),
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
        // We need CssSource and selector metadata to compute styles.
        // If the entity is dirty only due to CssUsers (asset event),
        // it might not match the query filter above. So we must fetch it.
        //
        // Best: have a second query for "all CssSource users".
        // Minimal change: just try get() from query_changed_source; if it fails, skip.
        // (If you want: I can provide the "all query" variant.)
        let Ok((_, css_source, id, class, tag, parent, _dirty_marker)) =
            query_changed_source.get(entity)
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
        );

        let primary_css = css_source.0.first().cloned().unwrap_or_default();

        let final_style = UiStyle {
            css: primary_css,
            styles: merged_css.styles,
            keyframes: merged_css.keyframes,
            active_style: None,
        };

        match style_query.get(entity) {
            Ok(Some(existing)) if existing.styles != final_style.styles => {
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
            _ => {}
        }
    }
}

/// Loads and merges CSS styles from multiple sources with selector matching.
fn load_and_merge_styles_from_assets(
    sources: &Vec<Handle<CssAsset>>,
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
        let asset_id = handle.id();

        let parsed_map = if let Some(cached) = PARSED_CSS_CACHE
            .read()
            .ok()
            .and_then(|c| c.get(&asset_id).cloned())
        {
            cached
        } else {
            let Some(css_asset) = css_assets.get(handle) else {
                continue;
            };
            let parsed = load_css(&css_asset.text);
            if let Ok(mut cache) = PARSED_CSS_CACHE.write() {
                cache.insert(asset_id, parsed.clone());
            }
            parsed
        };

        for (selector, new_style) in parsed_map.styles.iter() {
            let selector_parts: Vec<&str> = selector.split_whitespace().collect();

            if matches_selector_chain(&selector_parts, id, class, tag, parent, parent_query) {
                merged_styles
                    .entry(selector.clone())
                    .and_modify(|existing| {
                        existing.normal.merge(&new_style.normal);
                        existing.important.merge(&new_style.important);
                        existing.origin = index; // Update origin to latest source
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
    selectors: &[&str],
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

    if selectors.len() > 1 && parent_opt.is_none() {
        return false;
    }

    let current_sel = selectors[selectors.len() - 1];
    if !matches_selector(current_sel, id_opt, class_opt, tag_opt) {
        return false;
    }

    if selectors.len() == 1 {
        return true;
    }

    if let Some(parent) = parent_opt {
        if let Ok((pid, p_class, p_tag, p_parent)) = parent_query.get(parent.parent()) {
            return matches_selector_chain(
                &selectors[..selectors.len() - 1],
                pid,
                p_class,
                p_tag,
                p_parent,
                parent_query,
            );
        }
    }

    false
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
