use super::*;
use std::sync::Arc;

/// Each world owns its cache: asset indices can be identical in different Apps.
/// Ordered source lists also capture the scope and precedence of root variables.
#[derive(Resource, Default)]
pub(super) struct CssCache {
    sources: HashMap<Vec<AssetId<CssAsset>>, Arc<Vec<PreparedCss>>>,
}

pub(super) struct PreparedCss {
    pub parsed: ParsedCss,
    selectors: HashMap<String, Vec<SelectorStep>>,
    ids: HashMap<String, Vec<String>>,
    classes: HashMap<String, Vec<String>>,
    tags: HashMap<String, Vec<String>>,
    universal: Vec<String>,
}

impl PreparedCss {
    fn new(parsed: ParsedCss) -> Self {
        let mut prepared = Self {
            parsed,
            selectors: HashMap::new(),
            ids: HashMap::new(),
            classes: HashMap::new(),
            tags: HashMap::new(),
            universal: Vec::new(),
        };
        for (key, rule) in &prepared.parsed.styles {
            let selector = if rule.selector.is_empty() {
                key
            } else {
                &rule.selector
            };
            let steps = parse_selector_steps(selector);
            // A rule has exactly one bucket, selected from its rightmost token.
            // Full matching below still checks compound selectors and ancestors.
            let requirements = steps
                .last()
                .and_then(|step| parse_simple_selector(&step.selector));
            let bucket = if let Some(requirements) = requirements {
                if let Some(id) = requirements.id {
                    prepared.ids.entry(id.to_owned()).or_default()
                } else if let Some(class) = requirements.classes.first() {
                    prepared.classes.entry((*class).to_owned()).or_default()
                } else if let Some(tag) = requirements.tag {
                    prepared.tags.entry(tag.to_ascii_lowercase()).or_default()
                } else {
                    &mut prepared.universal
                }
            } else {
                &mut prepared.universal
            };
            bucket.push(key.clone());
            prepared.selectors.insert(key.clone(), steps);
        }
        prepared
    }

    pub(super) fn candidates(
        &self,
        id: Option<&CssID>,
        class: Option<&CssClass>,
        tag: Option<&TagName>,
    ) -> Vec<(&String, &StylePair, &[SelectorStep])> {
        let mut keys: Vec<&String> = self.universal.iter().collect();
        if let Some(bucket) = id.and_then(|id| self.ids.get(&id.0)) {
            keys.extend(bucket);
        }
        if let Some(class) = class {
            for name in &class.0 {
                if let Some(bucket) = self.classes.get(name) {
                    keys.extend(bucket);
                }
            }
        }
        if let Some(bucket) = tag.and_then(|tag| self.tags.get(&tag.0.to_ascii_lowercase())) {
            keys.extend(bucket);
        }
        // CssClass is public and can contain duplicate names.
        keys.sort_unstable();
        keys.dedup();
        keys.into_iter()
            .map(|key| {
                (
                    key,
                    &self.parsed.styles[key],
                    self.selectors[key].as_slice(),
                )
            })
            .collect()
    }
}

impl CssCache {
    pub(super) fn invalidate(&mut self, asset: AssetId<CssAsset>) {
        // A changed variable declaration can affect every sheet in the source list.
        self.sources.retain(|sources, _| !sources.contains(&asset));
    }

    pub(super) fn prepare(
        &mut self,
        sources: &[Handle<CssAsset>],
        assets: &Assets<CssAsset>,
    ) -> Arc<Vec<PreparedCss>> {
        let key: Vec<_> = sources.iter().map(Handle::id).collect();
        self.prepare_ids(key, assets)
    }

    pub(super) fn prepare_ids(
        &mut self,
        key: Vec<AssetId<CssAsset>>,
        assets: &Assets<CssAsset>,
    ) -> Arc<Vec<PreparedCss>> {
        if let Some(prepared) = self.sources.get(&key) {
            return Arc::clone(prepared);
        }
        let mut vars = HashMap::new();
        for id in &key {
            if let Some(asset) = assets.get(*id) {
                vars.extend(collect_root_css_vars(&asset.text));
            }
        }
        let prepared = Arc::new(
            key.iter()
                .map(|id| {
                    let parsed = assets
                        .get(*id)
                        .map(|asset| {
                            if vars.is_empty() {
                                load_css(&asset.text)
                            } else {
                                load_css_with_root_vars(&asset.text, &vars)
                            }
                        })
                        .unwrap_or_default();
                    PreparedCss::new(parsed)
                })
                .collect(),
        );
        // Do not retain partial results while asynchronous assets are still loading.
        if key.iter().all(|id| assets.contains(*id)) {
            self.sources.insert(key, Arc::clone(&prepared));
        }
        prepared
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_preserve_compound_pseudo_and_ancestor_selectors() {
        let sheet = PreparedCss::new(load_css(
            "* { color: red; } div { width: 1px; } .item.active { height: 2px; } #main.item:hover { padding: 3px; } .parent > .item { margin: 4px; } .missing { width: 5px; } @media (max-width: 500px) { .item { width: 6px; } }",
        ));
        let id = CssID("main".into());
        let class = CssClass(vec!["item".into(), "active".into(), "item".into()]);
        let tag = TagName("DIV".into());
        let candidates = sheet.candidates(Some(&id), Some(&class), Some(&tag));
        let actual: HashSet<_> = candidates.iter().map(|(key, _, _)| key.as_str()).collect();
        let expected: HashSet<_> = sheet
            .parsed
            .styles
            .iter()
            .filter(|(_, rule)| rule.selector != ".missing")
            .map(|(key, _)| key.as_str())
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(
            actual.len(),
            candidates.len(),
            "duplicate classes must not duplicate candidates"
        );
    }

    #[test]
    fn prepared_sources_are_shared_and_invalidated_as_a_unit() {
        let mut assets = Assets::<CssAsset>::default();
        let vars = assets.add(CssAsset {
            text: ":root { --width: 10px; }".into(),
        });
        let rules = assets.add(CssAsset {
            text: ".item { width: var(--width); }".into(),
        });
        let mut cache = CssCache::default();
        let sources = [vars.clone(), rules];
        let first = cache.prepare(&sources, &assets);
        let second = cache.prepare(&sources, &assets);
        assert!(
            Arc::ptr_eq(&first, &second),
            "cache hits must not clone stylesheets"
        );
        cache.invalidate(vars.id());
        assert!(!Arc::ptr_eq(&first, &cache.prepare(&sources, &assets)));
    }

    #[test]
    fn root_variable_scope_respects_ordered_source_lists() {
        let mut assets = Assets::<CssAsset>::default();
        let first = assets.add(CssAsset {
            text: ":root { --size: 10px; }".into(),
        });
        let second = assets.add(CssAsset {
            text: ":root { --size: 20px; }".into(),
        });
        let rules = assets.add(CssAsset {
            text: ".item { width: var(--size); }".into(),
        });
        let mut cache = CssCache::default();
        let forward = cache.prepare(&[first.clone(), second.clone(), rules.clone()], &assets);
        let reverse = cache.prepare(&[second, first, rules], &assets);
        assert_eq!(
            forward[2].parsed.styles[".item"].normal.width,
            Some(Val::Px(20.0))
        );
        assert_eq!(
            reverse[2].parsed.styles[".item"].normal.width,
            Some(Val::Px(10.0))
        );
    }

    #[test]
    fn pending_assets_do_not_leave_stale_partial_styles() {
        let mut assets = Assets::<CssAsset>::default();
        let pending = assets.reserve_handle();
        let mut cache = CssCache::default();
        let sources = [pending.clone()];
        assert!(cache.prepare(&sources, &assets)[0].parsed.styles.is_empty());
        assets
            .insert(
                pending.id(),
                CssAsset {
                    text: ".item { width: 42px; }".into(),
                },
            )
            .unwrap();
        assert_eq!(
            cache.prepare(&sources, &assets)[0].parsed.styles[".item"]
                .normal
                .width,
            Some(Val::Px(42.0))
        );
    }
}
