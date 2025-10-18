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

/// Represents an HTML document asset.
///
/// This asset type holds the raw HTML content as a UTF-8 encoded string.
/// It is typically used to load and represent `.html` or `.htm` files from
/// the asset system (e.g., under `assets/html/`).
///
/// The HTML content is not parsed automatically — this struct simply stores
/// the full text for further processing or parsing within the engine.
///
/// # Example
/// ```ignore
/// use bevy::prelude::*;
/// use bevy_extended_ui::HtmlDocument;
///
/// fn handle_html(documents: Res<Assets<HtmlDocument>>) {
///     for (handle, html) in documents.iter() {
///         println!("Loaded HTML document with {} bytes", html.len());
///     }
/// }
/// ```
///
/// # See also
/// * [`CssStylesheet`] — companion asset type for CSS files.
#[derive(Asset, TypePath, Deref, DerefMut, Clone)]
pub struct HtmlDocument(pub String);

/// Represents a CSS stylesheet asset.
///
/// This asset type holds the raw CSS content as a UTF-8 encoded string.
/// It is typically used to load `.css` files from the asset system
/// and can be dynamically hot-reloaded when changes are detected on disk.
///
/// The CSS data is not parsed automatically — it serves as a low-level
/// container for the stylesheet text used by the UI system or custom parsers.
///
/// # Example
/// ```ignore
/// use bevy::prelude::*;
/// use bevy_extended_ui::CssStylesheet;
///
/// fn apply_styles(styles: Res<Assets<CssStylesheet>>) {
///     for (handle, css) in styles.iter() {
///         println!("Loaded CSS stylesheet:\n{}", &css[..100.min(css.len())]);
///     }
/// }
/// ```
///
/// # See also
/// * [`HtmlDocument`] — companion asset type for HTML documents.
#[derive(Asset, TypePath, Deref, DerefMut, Clone)]
pub struct CssStylesheet(pub String);

/// Represents all possible errors that can occur while loading text-based assets.
///
/// This error type is shared between [`HtmlLoader`] and [`CssLoader`] and covers:
/// * I/O errors that happen when reading a file from disk.
/// * UTF-8 conversion errors when the file contains invalid text data.
///
/// These errors are non-fatal; Bevy’s asset system will log them and retry
/// loading as needed when the file changes.
///
/// # Variants
/// * `Io(std::io::Error)` — an underlying file or read error.
/// * `Utf8(std::string::FromUtf8Error)` — failed to interpret bytes as valid UTF-8 text.
///
/// # Example
/// ```ignore
/// use thiserror::Error;
/// use bevy_extended_ui::TextAssetLoaderError;
///
/// fn handle_error(err: TextAssetLoaderError) {
///     match err {
///         TextAssetLoaderError::Io(e) => eprintln!("File I/O error: {e}"),
///         TextAssetLoaderError::Utf8(e) => eprintln!("Invalid UTF-8: {e}"),
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum TextAssetLoaderError {
    /// Occurs when the underlying I/O operation fails (e.g., file not found, permission denied).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Occurs when the loaded bytes cannot be converted into valid UTF-8 text.
    #[error("Invalid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

// -----------------------------------------------------------------------------
// Loaders
// -----------------------------------------------------------------------------

/// An [`AssetLoader`] implementation for loading [`HtmlDocument`] assets.
///
/// This loader reads `.html` and `.htm` files from the Bevy asset system,
/// converts their byte contents into a UTF-8 string, and wraps them inside
/// an [`HtmlDocument`] asset for later use by the UI layer or other systems.
///
/// The loader performs no parsing or validation of the HTML itself — it simply
/// reads and stores the file contents as text.
///
/// # Supported extensions
/// * `.html`
/// * `.htm`
///
/// # Errors
/// Return a [`TextAssetLoaderError`] if:
/// * The file cannot be read from the asset source (I/O error).
/// * The file contains invalid UTF-8 bytes and cannot be converted to a string.
///
/// Once registered, `.html` and `.htm` files can be loaded using:
/// ```ignore
/// let handle: Handle<HtmlDocument> = asset_server.load("html/example.html");
/// ```
#[derive(Default)]
pub struct HtmlLoader;

impl AssetLoader for HtmlLoader {
    type Asset = HtmlDocument;
    type Settings = ();
    type Error = TextAssetLoaderError;

    /// Asynchronously loads an HTML asset from the given reader.
    ///
    /// Reads all bytes from the provided [`Reader`], converts them to UTF-8,
    /// and returns an [`HtmlDocument`] asset.
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

    /// Returns the list of file extensions handled by this loader.
    fn extensions(&self) -> &[&str] {
        &["html", "htm"]
    }
}

/// An [`AssetLoader`] implementation for loading [`CssStylesheet`] assets.
///
/// This loader is responsible for reading `.css` files from Bevy’s asset system,
/// decoding them as UTF-8 text, and wrapping the result in a [`CssStylesheet`] asset.
///
/// It is typically used together with [`HtmlLoader`] and [`PreloadedCss`] to support
/// HTML + CSS–based UI systems with hot-reloadable stylesheets.
///
/// The loader itself does **not** parse or validate CSS; it simply provides the raw
/// text content for later use by UI renderers, parsers, or style managers.
///
/// # Supported extensions
/// * `.css`
///
/// # Errors
/// Return a [`TextAssetLoaderError`] if:
/// * The file cannot be read from disk or another source (I/O error).
/// * The bytes are not valid UTF-8 and cannot be converted into a `String`.
///
/// Once registered, a CSS file can be loaded with:
/// ```ignore
/// let style: Handle<CssStylesheet> = asset_server.load("css/main.css");
/// ```
///
/// # See also
/// * [`PreloadedCss`] — helper resource for preloading and tracking CSS handles.
/// * [`HtmlLoader`] — companion loader for HTML documents.
#[derive(Default)]
pub struct CssLoader;

/// Stores handles to all preloaded CSS assets.
///
/// This resource is used to ensure that all discovered `.css` files are
/// registered with Bevy’s asset system early in the app lifecycle so that
/// hot-reloading (file watching) works reliably for all stylesheets.
///
/// The key is the asset path relative to the `assets/` directory
/// (for example, `"css/main.css"`), and the value is the corresponding
/// [`Handle<CssStylesheet>`].
#[derive(Resource, Default)]
pub struct PreloadedCss {
    /// Mapping of asset paths (`"css/file.css"`) to their preloaded handles.
    pub handle_by_path: HashMap<String, Handle<CssStylesheet>>,
}

impl AssetLoader for CssLoader {
    type Asset = CssStylesheet;
    type Settings = ();
    type Error = TextAssetLoaderError;

    /// Asynchronously loads a CSS stylesheet asset from the given reader.
    ///
    /// The file contents are read into memory, interpreted as UTF-8, and
    /// returned as a [`CssStylesheet`] asset.
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

    /// Returns the list of file extensions handled by this loader.
    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}

// -----------------------------------------------------------------------------
// Multi-UI request and handle tracking
// -----------------------------------------------------------------------------

/// Describes a single UI definition consisting of one HTML file and optional CSS files.
///
/// A `UiSpec` defines how a specific UI instance should be loaded — it contains
/// a unique identifier (`id`), the path to its HTML source, and a list of CSS
/// asset paths discovered either at startup or dynamically through `<link>` tags.
///
/// The `html_path` and `css_paths` are relative to the Bevy `assets/` directory,
/// for example, `"html/menu/main.html"` or `"css/theme.css"`.
#[derive(Clone)]
pub struct UiSpec {
    /// Unique identifier for this UI.
    pub id: String,
    /// Asset path to the associated HTML document (relative to `assets/`).
    pub html_path: String,
    /// Asset paths to one or more associated CSS files (relative to `assets/`).
    pub css_paths: Vec<String>,
}

/// Represents a collection of UI load requests used during startup or rebuilds.
///
/// This resource defines which UIs should be loaded and tracked by the system.
/// It can be built manually or discovered automatically by scanning the
/// `assets/html` directory. Each entry corresponds to a [`UiSpec`].
///
/// The `rebuild_debounce` defines how long to wait (in milliseconds) between
/// consecutive rebuild triggers — useful when watching files during development.
#[derive(Resource, Clone)]
pub struct UiAssetRequests {
    /// List of UI specifications to be loaded.
    pub specs: Vec<UiSpec>,
    /// Minimum delay between automatic rebuilds when assets change.
    pub rebuild_debounce: Duration,
}

impl UiAssetRequests {
    /// Creates a [`UiAssetRequests`] resource for a single HTML file with no CSS.
    ///
    /// This is a convenience constructor for simple UIs that consist of only
    /// one HTML document (for example, loading `"html/main.html"`).
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

/// Holds the runtime asset handles for a loaded UI instance.
///
/// Each [`UiHandle`] corresponds to a single [`UiSpec`] and stores the loaded
/// asset handles for both HTML and CSS resources. It also retains the original
/// asset paths for use in reloading or debugging.
///
/// This structure is typically stored in [`UiLoadedHandles`] and indexed by
/// its `id` or asset IDs.
#[derive(Clone, Reflect)]
pub struct UiHandle {
    /// Unique identifier for this UI instance.
    pub id: String,
    /// Handle to the loaded [`HtmlDocument`] asset.
    pub html: Handle<HtmlDocument>,
    /// Handles to the loaded [`CssStylesheet`] assets associated with this UI.
    pub css: Vec<Handle<CssStylesheet>>,
    /// Original HTML file path relative to the `assets/` folder.
    pub html_path: String,
    /// Original CSS file paths relative to the `assets/` folder.
    pub css_paths: Vec<String>,
}

/// Central resource for tracking all loaded UI handles and their asset mappings.
///
/// This resource keeps:
/// * A map from UI IDs (`by_id`) to [`UiHandle`] structures.
/// * A reverse lookup from [`HtmlDocument`] asset IDs to their corresponding UI IDs (`html_index`).
/// * A reverse lookup from [`CssStylesheet`] asset IDs to their corresponding UI IDs (`css_index`).
///
/// This makes it easy to resolve which UI should be reloaded or rebuilt when
/// an asset changes (for example, when Bevy’s file watcher emits a reload event).
///
/// # Example
/// ```ignore
/// fn debug_loaded_handles(loaded: Res<UiLoadedHandles>) {
///     for (id, handle) in &loaded.by_id {
///         println!("UI {} -> HTML: {}", id, handle.html_path);
///     }
/// }
/// ```
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct UiLoadedHandles {
    /// Mapping from UI IDs to their corresponding loaded handles.
    pub by_id: HashMap<String, UiHandle>,
    /// Reverse index mapping from HTML asset IDs to UI IDs.
    pub html_index: HashMap<AssetId<HtmlDocument>, String>,
    /// Reverse index mapping from CSS asset IDs to UI IDs.
    pub css_index: HashMap<AssetId<CssStylesheet>, String>,
}

// -----------------------------------------------------------------------------
// Ready events (carry UI id)
// -----------------------------------------------------------------------------

/// Emitted when an [`HtmlDocument`] asset has been successfully loaded or reloaded.
///
/// This message/event is used to notify systems that a specific UI’s HTML file
/// is now available in memory and ready to be parsed or built into UI elements.
///
/// It carries the UI’s logical `id`, the [`Handle<HtmlDocument>`], and the
/// asset path relative to the `assets/` directory.
///
/// These events are triggered by Bevy’s asset system when the underlying
/// HTML file is first loaded or modified on disk (hot reload).
///
/// # Example
/// ```ignore
/// fn on_html_ready(mut reader: MessageReader<HtmlReady>, assets: Res<Assets<HtmlDocument>>) {
///     for ev in reader.read() {
///         if let Some(doc) = assets.get(&ev.handle) {
///             println!("UI '{}' HTML reloaded ({} bytes)", ev.id, doc.len());
///         }
///     }
/// }
/// ```
///
/// # See also
/// * [`CssReady`] — emitted for corresponding CSS files.
#[derive(Event, Message, Clone)]
pub struct HtmlReady {
    /// Unique identifier of the UI that owns this HTML document.
    pub id: String,
    /// Handle to the loaded or reloaded [`HtmlDocument`] asset.
    pub handle: Handle<HtmlDocument>,
    /// The asset path of the HTML file (relative to `assets/`).
    pub path: String,
}

/// Emitted when a [`CssStylesheet`] asset has been successfully loaded or reloaded.
///
/// This message/event indicates that a stylesheet associated with a specific UI
/// has been loaded and is available for use. It is emitted both when the file is
/// first read and when Bevy’s file watcher detects a modification.
///
/// Each `CssReady` event contains the UI’s logical `id`, the handle to the
/// stylesheet, and the asset path relative to the `assets/` directory.
///
/// # Example
/// ```ignore
/// fn on_css_ready(mut reader: MessageReader<CssReady>, assets: Res<Assets<CssStylesheet>>) {
///     for ev in reader.read() {
///         if let Some(css) = assets.get(&ev.handle) {
///             println!("UI '{}' stylesheet updated: {}", ev.id, ev.path);
///             println!("First 100 chars:\n{}", &css[..css.len().min(100)]);
///         }
///     }
/// }
/// ```
///
/// # See also
/// * [`HtmlReady`] — emitted for HTML files belonging to the same UI.
#[derive(Event, Message, Clone)]
pub struct CssReady {
    /// Unique identifier of the UI that owns this CSS stylesheet.
    pub id: String,
    /// Handle to the loaded or reloaded [`CssStylesheet`] asset.
    pub handle: Handle<CssStylesheet>,
    /// The asset path of the CSS file (relative to `assets/`).
    pub path: String,
}


// -----------------------------------------------------------------------------
// Plugin
// -----------------------------------------------------------------------------

/// Bevy plugin that wires up HTML/CSS asset loading and hot-reload for UI files.
///
/// `UiIoPlugin` registers custom asset types (`HtmlDocument`, `CssStylesheet`)
/// and their loaders, sets up runtime resources to track loaded handles, and adds
/// systems that:
///
/// - Discover UI files on the disk (`assets/html/**/*.html|htm`) at startup.
/// - Optionally pre-warm CSS assets so file watching works reliably.
/// - Load all discovered HTML (and CSS) via `AssetServer`.
/// - Listen for `AssetEvent<HtmlDocument>` / `AssetEvent<CssStylesheet>` and emit
///   high-level messages (`HtmlReady`, `CssReady`) that your UI pipeline can consume.
/// - Parse `<link rel="stylesheet" href="...">` from loaded HTML and dynamically
///   load any referenced stylesheets (with de-duplication per UI).
///
/// ### Requirements
/// - Enable Bevy’s file watching to get hot-reload on disk changes:
///   ```toml
///   bevy = { version = "0.17.2", features = ["file_watcher"] }
///   ```
/// After adding the plugin, listen for `HtmlReady`/`CssReady` messages to (re)build
/// your UI or re-apply styles.
pub(crate) struct UiIoPlugin;

impl Plugin for UiIoPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HtmlDocument>()
            .init_asset::<CssStylesheet>()
            .init_asset_loader::<HtmlLoader>()
            .init_asset_loader::<CssLoader>()
            .insert_resource(UiLoadedHandles::default())
            .insert_resource(PreloadedCss::default())
            .add_message::<HtmlReady>()
            .add_message::<CssReady>()
            .add_systems(Startup, (auto_discover_html, auto_discover_css, load_all_ui_assets).chain())
            .add_systems(Update, (on_html_asset_events, on_css_asset_events, parse_links_and_load_css));
    }
}

// -----------------------------------------------------------------------------
// Systems
// -----------------------------------------------------------------------------

/// Discovers all HTML/HTM UI entry files under `assets/html` and records them in [`UiAssetRequests`].
///
/// This startup system scans the on-disk directory `assets/html` recursively and collects
/// every file with the extension `.html` or `.htm`. For each discovered asset path
/// (e.g. `"html/menu/main.html"`), it creates a corresponding [`UiSpec`] using:
///
/// - `id`: derived from the asset path with the `.html`/`.htm` suffix removed
/// - `html_path`: the asset path relative to the `assets/` root
/// - `css_paths`: initially empty; these are typically filled later by parsing `<link>` tags
///
/// If a [`UiAssetRequests`] resource already exists, newly discovered specs are appended
/// without duplicating entries that are already present. If the resource does not exist,
/// it is created and inserted with all discovered specs and a default `rebuild_debounce`.
///
/// This system does not load any assets by itself; it only prepares the request list.
/// Actual loading should be performed by a later system (e.g. [`load_all_ui_assets`]).
///
/// # Scheduling
/// Should run early at startup and before any systems that consume [`UiAssetRequests`].
///
/// # Parameters
/// - `commands`: used to insert [`UiAssetRequests`] if it does not already exist
/// - `reqs`: an optional mutable reference to the existing [`UiAssetRequests`] resource
///
/// # See also
/// - [`UiAssetRequests`], [`UiSpec`], [`load_all_ui_assets`]
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
                    if let Some(asset_rel) = helper::to_asset_path(&path) {
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

/// Preloads all `.css` files so the file watcher can emit hot-reload events.
///
/// This startup system scans common roots (`assets/html`, `assets/css`, and `assets`)
/// recursively, finds every file with the `.css` extension, converts its disk path
/// (`assets/<rest>`) into an asset path (`<rest>`), and calls `AssetServer::load`
/// once per path. Preloading ensures Bevy’s file watcher is aware of these files
/// early in the app lifecycle, allowing later edits to trigger
/// `AssetEvent::Modified<CssStylesheet>`.
///
/// Discovered handles are stored in the [`PreloadedCss`] resource, so other systems
/// (e.g., the HTML `<link>` parser) can reuse existing handles instead of creating
/// duplicates.
///
/// # Scheduling
/// Run this in `Startup`, ideally before systems that link CSS to UIs (so the
/// watcher already tracks all stylesheets).
///
/// # Notes
/// - The scan is recursive and liberal: it looks under `assets/html`, `assets/css`,
///   and finally `assets` as a fallback, filtering by the `.css` extension only.
/// - Paths are de-duplicated by keying the map with the asset-relative path.
/// - This system does **not** associate stylesheets with specific UIs; it only
///   pre-warms the handles and the watcher. Association happens later (e.g., by
///   parsing `<link rel="stylesheet" href="...">` from HTML).
fn auto_discover_css(mut commands: Commands, asset_server: Res<AssetServer>) {
    // scan both "assets/html" and "assets/css" if they exist
    let roots = ["assets/html", "assets/css", "assets"]; // be liberal; filter by ext
    let mut pre = PreloadedCss::default();

    for root in roots {
        let path = Path::new(root);
        if !path.exists() { continue; }

        let mut stack = vec![path.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(read_dir) = fs::read_dir(&dir) else { continue };
            for entry in read_dir.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("css")).unwrap_or(false) {
                    // convert "assets/<rest>" -> "<rest>"
                    if let Some(asset_rel) = helper::to_asset_path(&p) {
                        // load once so the file watcher tracks it
                        let h: Handle<CssStylesheet> = asset_server.load(asset_rel.clone());
                        pre.handle_by_path.insert(asset_rel, h);
                    }
                }
            }
        }
    }

    commands.insert_resource(pre);
}

/// Loads all scheduled UI HTML assets and initializes handle tracking.
///
/// This startup system iterates over [`UiAssetRequests::specs`] and:
/// - Calls `AssetServer::load` for each `html_path`.
/// - Creates a [`UiHandle`] entry with the HTML handle and empty CSS lists.
/// - Populates the reverse index `html_index` so future asset events can be
///   mapped back to the corresponding UI `id`.
///
/// This system does **not** load CSS files. CSS association is expected to be
/// handled by:
/// - a preloading pass (e.g. `auto_discover_css`) to warm up the file watcher, and/or
/// - a link-parsing pass (e.g. `parse_links_and_load_css`) reacting to `HtmlReady`.
///
/// # Scheduling
/// Run after HTML discovery (e.g. `auto_discover_html`), and before any systems
/// that depend on valid `UiLoadedHandles` entries.
///
/// # Parameters
/// - `asset_server`: Bevy asset server used to request asset loads.
/// - `reqs`: Optional UI request list; if absent, this system is a no-op.
/// - `loaded`: Runtime registry for UI handles and reverse indices.
///
/// # See also
/// - [`auto_discover_html`]
/// - [`auto_discover_css`]
/// - [`parse_links_and_load_css`]
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

/// Parses `<link rel="stylesheet" href="...">` tags from loaded HTML and ensures CSS is loaded and tracked.
///
/// This system reacts to [`HtmlReady`] messages. For each ready HTML document, it:
/// 1. Extracts all stylesheet `href` values via [`helper::extract_stylesheet_hrefs`].
/// 2. Resolves each `href` into an asset path relative to the HTML’s location using
///    [`helper::resolve_href`].
/// 3. For every resolved CSS path not yet known for the UI:
///    - Reuses a preloaded handle from [`PreloadedCss`] if available, otherwise loads it via
///      [`AssetServer::load`].
///    - Registers the CSS handle-path pair in the UI’s entry within [`UiLoadedHandles`].
///    - Updates the reverse index (`css_index`) so future `AssetEvent<CssStylesheet>`
///      can be mapped back to the UI `id`.
///
/// This function is designed to avoid conflicting mutable borrows by splitting the work
/// into phases:
/// - Phase 1 computes missing CSS additions without mutating [`UiLoadedHandles`].
/// - Phase 2 updates the UI’s handle lists (`by_id`) only.
/// - Phase 3 updates the reverse index (`css_index`) after the mutable borrowed from Phase 2 ends.
///
/// Hot-reload behavior:
/// - If Bevy’s file watcher is enabled and the CSS files were preloaded (see `auto_discover_css`),
///   edits to those files will trigger `AssetEvent::Modified<CssStylesheet>`, which should be
///   handled by your `on_css_asset_events` system to emit [`CssReady`].
///
/// # Scheduling
/// Should run in `Update`, after systems that emit [`HtmlReady`] and before systems that depend
/// on CSS being attached to the UI.
///
/// # Parameters
/// - `ev_html_ready`: stream of HTML readiness messages
/// - `assets_html`: access to loaded [`HtmlDocument`] assets
/// - `asset_server`: used to load missing stylesheets
/// - `loaded`: runtime registry that maps UI ids to their handles and indices
/// - `pre`: store of preloaded CSS handles to avoid duplicate loads
fn parse_links_and_load_css(
    mut ev_html_ready: MessageReader<HtmlReady>,
    assets_html: Res<Assets<HtmlDocument>>,
    asset_server: Res<AssetServer>,
    mut loaded: ResMut<UiLoadedHandles>,
    mut pre: ResMut<PreloadedCss>,
) {
    for ev in ev_html_ready.read() {
        let ui_id = ev.id.clone();

        let Some(handle) = loaded.by_id.get(&ui_id).map(|h| h.html.clone()) else { continue };
        let Some(doc) = assets_html.get(&handle) else { continue };

        let css_paths: Vec<String> = helper::extract_stylesheet_hrefs(&doc.0)
            .into_iter()
            .map(|href| helper::resolve_href(&ev.path, &href))
            .collect();

        // Phase 1: determine additions (no mutable borrows on `loaded`)
        let mut to_add: Vec<(Handle<CssStylesheet>, String)> = Vec::new();
        if let Some(entry) = loaded.by_id.get(&ui_id) {
            for path in css_paths {
                if !entry.css_paths.iter().any(|p| p == &path) {
                    // use a preloaded handle if present, otherwise load and store
                    let h = if let Some(h) = pre.handle_by_path.get(&path) {
                        h.clone()
                    } else {
                        let h: Handle<CssStylesheet> = asset_server.load(path.clone());
                        pre.handle_by_path.insert(path.clone(), h.clone());
                        h
                    };
                    to_add.push((h, path));
                }
            }
        } else {
            continue;
        }

        // Phase 2: update UiLoadedHandles.by_id
        if let Some(entry) = loaded.by_id.get_mut(&ui_id) {
            for (h, path) in &to_add {
                entry.css_paths.push(path.clone());
                entry.css.push(h.clone());
            }
        }

        // Phase 3: update css_index
        for (h, _) in to_add {
            loaded.css_index.insert(h.id(), ui_id.clone());
        }
    }
}

/// Emits a high-level [`HtmlReady`] message when an [`HtmlDocument`] asset loads or changes.
///
/// This system listens to low-level asset messages for `HtmlDocument` and translates
/// them into `HtmlReady` messages that downstream systems can consume to (re)parse
/// and rebuild UI. It handles both the initial load (`LoadedWithDependencies`) and
/// later hot-reloads (`Modified`).
///
/// Processing steps:
/// 1. Read incoming `AssetEvent<HtmlDocument>` messages.
/// 2. Map the asset `id` back to a UI via `UiLoadedHandles.html_index`.
/// 3. Ensure the asset data is currently available in `Assets<HtmlDocument>`.
/// 4. Emit [`HtmlReady`] with the UI `id`, the HTML handle, and the asset path.
///
/// Notes:
/// - Uses Bevy's message API (`MessageReader` / `MessageWriter`) introduced in 0.17.
/// - This function is side effect free beyond emitting `HtmlReady`.
/// - Pair with [`parse_links_and_load_css`] to load any linked stylesheets when HTML changes.
///
/// Scheduling:
/// - Run in `Update`.
/// - Should execute before systems that react to `HtmlReady`
///   (e.g., DOM parsing, UI patching).
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

/// Emits a high-level [`CssReady`] message when a [`CssStylesheet`] asset loads or changes.
///
/// This system translates low-level `AssetEvent<CssStylesheet>` messages into `CssReady`
/// messages that downstream systems can react to (e.g., reapplying styles to a UI).
/// It handles both the initial load (`LoadedWithDependencies`) and later hot-reloads
/// (`Modified`).
///
/// Processing steps:
/// 1. Read incoming `AssetEvent<CssStylesheet>` messages.
/// 2. Use `UiLoadedHandles.css_index` to map the asset `id` back to the owning UI `id`.
/// 3. Look up the UI’s `UiHandle` and find the matching CSS handle by `AssetId`.
/// 4. Ensure the asset is currently available in `Assets<CssStylesheet>`.
/// 5. Emit [`CssReady`] with the UI `id`, the CSS handle, and its asset path.
///
/// Notes:
/// - Uses Bevy’s message API (`MessageReader` / `MessageWriter`) introduced in 0.17.
/// - Pair this with `on_html_asset_events` and `parse_links_and_load_css` for a complete
///   HTML/CSS hot-reload pipeline.
/// - The emitted `CssReady` includes the resolved asset path for convenience.
///
/// Scheduling:
/// - Run in `Update`.
/// - Should execute before systems that consume `CssReady` (e.g., style application).
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
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

mod helper {
    use std::path::Path;

    /// Converts a disk path like `assets/<rest>` into an asset-relative path `<rest>`.
    ///
    /// Returns `None` if the path does not start with the `assets` directory.
    ///
    /// The returned path uses forward slashes on all platforms.
    pub(crate) fn to_asset_path(disk_path: &Path) -> Option<String> {
        // Expect "assets/<rest>", return "<rest>" with forward slashes
        let mut comps = disk_path.components();
        if comps.next()?.as_os_str() != "assets" {
            return None;
        }
        let rest = comps.as_path();
        Some(path_to_asset_string(rest))
    }

    /// Resolves an `href` value from an HTML document to an asset path.
    ///
    /// Behavior:
    /// - If `href` starts with `/`, it is treated as an asset-root-relative path
    ///   (e.g., `"/css/main.css"` → `"css/main.css"`).
    /// - Otherwise, `href` is resolved relative to the directory containing `html_asset_path`.
    ///
    /// The result uses forward slashes on all platforms.
    pub(crate) fn resolve_href(html_asset_path: &str, href: &str) -> String {
        if href.starts_with('/') {
            return href.trim_start_matches('/').to_string();
        }
        let html_path = Path::new(html_asset_path);
        let base = html_path.parent().unwrap_or_else(|| Path::new("html"));
        let joined = base.join(href);
        path_to_asset_string(&joined)
    }

    /// Extracts all `<link rel="stylesheet" href="...">` `href` values from an HTML string.
    ///
    /// This is a lightweight, permissive extractor:
    /// - Matches `<link ...>` tags based on substring search.
    /// - Accepts `rel="stylesheet"`, `rel='stylesheet'`, or unquoted `rel=stylesheet`.
    /// - Reads the `href` attribute with `"..."`, `'...'`, or unquoted forms.
    ///
    /// It does **not** perform full HTML parsing and may miss edge cases in malformed markup.
    pub(crate) fn extract_stylesheet_hrefs(html: &str) -> Vec<String> {
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

    /// Converts a `Path` into a forward-slash-separated string.
    fn path_to_asset_string(p: &Path) -> String {
        let s = p.to_string_lossy().to_string();
        s.replace('\\', "/")
    }

    /// Extracts an attribute value from a single tag string.
    ///
    /// Supports `attr="..."`, `attr='...'`, and unquoted `attr=...` forms.
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
}