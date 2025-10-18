use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Duration;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use thiserror::Error;

// -----------------------------------------------------------------------------
// Assets and errors
// -----------------------------------------------------------------------------
#[derive(Asset, TypePath, Deref, DerefMut, Clone)]
pub struct HtmlDocument(pub String);

#[derive(Asset, TypePath, Deref, DerefMut, Clone)]
pub struct CssStylesheet(pub String);

#[derive(Debug, Error)]
pub enum TextAssetLoaderError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

// -----------------------------------------------------------------------------
// Loaders
// -----------------------------------------------------------------------------
#[derive(Default)]
pub struct HtmlLoader;

impl AssetLoader for HtmlLoader {
    type Asset = HtmlDocument;
    type Settings = ();
    type Error = TextAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(HtmlDocument(String::from_utf8(bytes)?))
    }

    fn extensions(&self) -> &[&str] {
        &["html", "htm"]
    }
}

#[derive(Default)]
pub struct CssLoader;

impl AssetLoader for CssLoader {
    type Asset = CssStylesheet;
    type Settings = ();
    type Error = TextAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(CssStylesheet(String::from_utf8(bytes)?))
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}

// -----------------------------------------------------------------------------
// Multi-UI request and handle tracking
// -----------------------------------------------------------------------------
#[derive(Clone)]
pub struct UiSpec {
    pub id: String,
    pub html_path: String,
    pub css_paths: Vec<String>,
}

#[derive(Resource, Clone)]
pub struct UiAssetRequests {
    pub specs: Vec<UiSpec>,
    pub rebuild_debounce: Duration,
}

impl UiAssetRequests {
    pub fn single(html: impl Into<String>) -> Self {
        Self {
            specs: vec![UiSpec {
                id: "default".into(),
                html_path: html.into(),
                css_paths: Vec::new(),
            }],
            rebuild_debounce: Duration::from_millis(25),
        }
    }
}

#[derive(Clone, Reflect)]
pub struct UiHandle {
    pub id: String,
    pub html: Handle<HtmlDocument>,
    pub css: Vec<Handle<CssStylesheet>>,
    pub html_path: String,
    pub css_paths: Vec<String>,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct UiLoadedHandles {
    pub by_id: HashMap<String, UiHandle>,
    pub html_index: HashMap<AssetId<HtmlDocument>, String>,
    pub css_index: HashMap<AssetId<CssStylesheet>, String>,
}

// -----------------------------------------------------------------------------
// Ready events (carry UI id)
// -----------------------------------------------------------------------------
#[derive(Event, Message, Clone)]
pub struct HtmlReady {
    pub id: String,
    pub handle: Handle<HtmlDocument>,
    pub path: String,
}

#[derive(Event, Message, Clone)]
pub struct CssReady {
    pub id: String,
    pub handle: Handle<CssStylesheet>,
    pub path: String,
}

// -----------------------------------------------------------------------------
// Plugin
// -----------------------------------------------------------------------------
pub(crate) struct UiIoPlugin;

impl Plugin for UiIoPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HtmlDocument>()
            .init_asset::<CssStylesheet>()
            .init_asset_loader::<HtmlLoader>()
            .init_asset_loader::<CssLoader>()
            .insert_resource(UiLoadedHandles::default())
            .add_message::<HtmlReady>()
            .add_message::<CssReady>()
            .add_systems(Startup, (auto_discover_html, load_all_ui_assets).chain())
            .add_systems(Update, (on_html_asset_events, on_css_asset_events, parse_links_and_load_css));
    }
}

// -----------------------------------------------------------------------------
// Systems
// -----------------------------------------------------------------------------

fn auto_discover_html(mut commands: Commands, reqs: Option<ResMut<UiAssetRequests>>) {
    let base = Path::new("assets").join("html");
    if !base.exists() {
        return;
    }

    // 1) collect all html/htm files recursively
    let mut found: Vec<String> = Vec::new();
    let mut stack = vec![base.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(read_dir) = fs::read_dir(&dir) else { continue };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_l = ext.to_ascii_lowercase();
                if ext_l == "html" || ext_l == "htm" {
                    if let Some(asset_rel) = to_asset_path(&path) {
                        found.push(asset_rel);
                    }
                }
            }
        }
    }

    if let Some(mut reqs) = reqs {
        let existing: HashSet<_> = reqs.specs.iter().map(|s| s.html_path.clone()).collect();
        for html_path in found {
            if !existing.contains(&html_path) {
                let id = html_path
                    .trim_end_matches(".html")
                    .trim_end_matches(".htm")
                    .to_string();
                reqs.specs.push(UiSpec {
                    id,
                    html_path,
                    css_paths: Vec::new(),
                });
            }
        }
    } else {
        let specs: Vec<UiSpec> = found
            .into_iter()
            .map(|html_path| {
                let id = html_path
                    .trim_end_matches(".html")
                    .trim_end_matches(".htm")
                    .to_string();
                UiSpec {
                    id,
                    html_path,
                    css_paths: Vec::new(),
                }
            })
            .collect();

        commands.insert_resource(UiAssetRequests {
            specs,
            rebuild_debounce: Duration::from_millis(25),
        });
    }
}

fn to_asset_path(disk_path: &Path) -> Option<String> {
    // Expect "assets/<rest>", return "<rest>" with forward slashes
    let mut comps = disk_path.components();
    if comps.next()?.as_os_str() != "assets" {
        return None;
    }
    let rest = comps.as_path();
    Some(path_to_asset_string(rest))
}

fn path_to_asset_string(p: &Path) -> String {
    let s = p.to_string_lossy().to_string();
    s.replace('\\', "/")
}

fn load_all_ui_assets(
    asset_server: Res<AssetServer>,
    reqs: Option<Res<UiAssetRequests>>,
    mut loaded: ResMut<UiLoadedHandles>,
) {
    let Some(reqs) = reqs else { return };

    for spec in &reqs.specs {
        let html: Handle<HtmlDocument> = asset_server.load(spec.html_path.clone());
        let entry = UiHandle {
            id: spec.id.clone(),
            html: html.clone(),
            css: Vec::new(),
            html_path: spec.html_path.clone(),
            css_paths: Vec::new(),
        };
        loaded.html_index.insert(html.id(), spec.id.clone());
        loaded.by_id.insert(spec.id.clone(), entry);
    }
}

fn on_html_asset_events(
    mut ev_asset: MessageReader<AssetEvent<HtmlDocument>>,
    assets: Res<Assets<HtmlDocument>>,
    loaded: Res<UiLoadedHandles>,
    mut ev_ready: MessageWriter<HtmlReady>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                if let Some(ui_id) = loaded.html_index.get(id) {
                    if let Some(entry) = loaded.by_id.get(ui_id) {
                        if assets.get(&entry.html).is_some() {
                            ev_ready.write(HtmlReady {
                                id: ui_id.clone(),
                                handle: entry.html.clone(),
                                path: entry.html_path.clone(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn parse_links_and_load_css(
    mut ev_html_ready: MessageReader<HtmlReady>,
    assets_html: Res<Assets<HtmlDocument>>,
    asset_server: Res<AssetServer>,
    mut loaded: ResMut<UiLoadedHandles>,
) {
    for ev in ev_html_ready.read() {
        let Some(handle) = loaded.by_id.get(&ev.id).map(|h| h.html.clone()) else { continue };
        let Some(doc) = assets_html.get(&handle) else { continue };

        let css_hrefs = extract_stylesheet_hrefs(&doc.0);

        // Resolve and load
        let mut css_handles = Vec::new();
        let mut css_paths = Vec::new();
        for href in css_hrefs {
            let resolved = resolve_href(&ev.path, &href);
            let handle: Handle<CssStylesheet> = asset_server.load(resolved.clone());
            loaded.css_index.insert(handle.id(), ev.id.clone());
            css_handles.push(handle);
            css_paths.push(resolved);
        }

        if let Some(entry) = loaded.by_id.get_mut(&ev.id) {
            entry.css = css_handles;
            entry.css_paths = css_paths;
        }
    }
}

// Very small, permissive extractor for <link rel="stylesheet" href="...">
fn extract_stylesheet_hrefs(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = html.to_lowercase();

    let mut i = 0;
    while let Some(pos) = lower[i..].find("<link") {
        i += pos;
        if let Some(end) = lower[i..].find('>') {
            let tag = &html[i..i + end + 1];
            let tag_l = &lower[i..i + end + 1];

            if tag_l.contains("rel=\"stylesheet\"") || tag_l.contains("rel='stylesheet'") || tag_l.contains("rel=stylesheet") {
                if let Some(href_val) = extract_attr(tag, "href") {
                    out.push(href_val);
                }
            }
            i += end + 1;
        } else {
            break;
        }
    }
    out
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let tl = tag.to_lowercase();
    let key = format!("{attr}=");
    let p = tl.find(&key)?;
    let rest = &tag[p + key.len()..].trim_start();
    if rest.starts_with('"') {
        let rest = &rest[1..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    } else if rest.starts_with('\'') {
        let rest = &rest[1..];
        let end = rest.find('\'')?;
        Some(rest[..end].to_string())
    } else {
        // unquoted until space or '>'
        let end = rest.find(|c: char| c.is_whitespace() || c == '>').unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

fn resolve_href(html_asset_path: &str, href: &str) -> String {
    if href.starts_with('/') {
        return href.trim_start_matches('/').to_string();
    }
    let html_path = Path::new(html_asset_path);
    let base = html_path.parent().unwrap_or_else(|| Path::new("html"));
    let joined = base.join(href);
    path_to_asset_string(&joined)
}

fn on_css_asset_events(
    mut ev_asset: MessageReader<AssetEvent<CssStylesheet>>,
    assets: Res<Assets<CssStylesheet>>,
    loaded: Res<UiLoadedHandles>,
    mut ev_ready: MessageWriter<CssReady>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                if let Some(ui_id) = loaded.css_index.get(id) {
                    if let Some(entry) = loaded.by_id.get(ui_id) {
                        for css_handle in &entry.css {
                            if &css_handle.id() == id && assets.get(css_handle).is_some() {
                                let idx = entry.css.iter().position(|h| h.id() == *id).unwrap();
                                let path = entry
                                    .css_paths
                                    .get(idx)
                                    .cloned()
                                    .unwrap_or_else(|| "<unknown>".into());
                                ev_ready.write(CssReady {
                                    id: ui_id.clone(),
                                    handle: css_handle.clone(),
                                    path,
                                });
                                info!("CSS loaded");
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}