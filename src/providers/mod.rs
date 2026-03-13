mod theme_provider;

use bevy::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

pub use theme_provider::{ThemeProvider, ThemeProviderState};

/// Controls which child tags are valid inside a provider element.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ProviderChildPolicy {
    /// Any child tag is accepted.
    #[default]
    Any,
    /// Only the given direct child tags are accepted.
    Only(Vec<String>),
}

/// Structural rules for an HTML provider node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderRules {
    /// Require a direct `<body>` child.
    pub requires_body_child: bool,
    /// Restrict which direct child tags are allowed.
    pub child_policy: ProviderChildPolicy,
    /// Whether this provider is allowed inside `<head>`.
    pub allow_in_head: bool,
}

impl Default for ProviderRules {
    fn default() -> Self {
        Self {
            requires_body_child: false,
            child_policy: ProviderChildPolicy::Any,
            allow_in_head: true,
        }
    }
}

/// Effects returned by a provider for HTML parsing.
#[derive(Clone, Debug, Default)]
pub struct ProviderEffect {
    /// Extra CSS asset paths that should be applied to the provider scope.
    pub extra_css_paths: Vec<String>,
}

/// Context passed to provider resolution.
#[derive(Clone, Copy)]
pub struct ProviderResolveContext<'a> {
    attributes: &'a HashMap<String, String>,
    source_path: &'a str,
    theme_asset_dir: Option<&'a str>,
    known_themes: Option<&'a HashSet<String>>,
    fallback_theme: Option<&'a str>,
    active_theme: Option<&'a str>,
}

impl<'a> ProviderResolveContext<'a> {
    /// Creates a new provider resolve context.
    pub fn new(attributes: &'a HashMap<String, String>, source_path: &'a str) -> Self {
        Self {
            attributes,
            source_path,
            theme_asset_dir: None,
            known_themes: None,
            fallback_theme: None,
            active_theme: None,
        }
    }

    /// Enriches the context with theme-provider specific data.
    pub fn with_theme_scope(
        mut self,
        theme_asset_dir: Option<&'a str>,
        known_themes: Option<&'a HashSet<String>>,
        fallback_theme: Option<&'a str>,
        active_theme: Option<&'a str>,
    ) -> Self {
        self.theme_asset_dir = theme_asset_dir;
        self.known_themes = known_themes;
        self.fallback_theme = fallback_theme;
        self.active_theme = active_theme;
        self
    }

    /// Returns an attribute value.
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    /// Returns the current HTML asset source path.
    pub fn source_path(&self) -> &str {
        self.source_path
    }

    /// Returns the configured theme asset directory (relative to asset root).
    pub fn theme_asset_dir(&self) -> Option<&str> {
        self.theme_asset_dir
    }

    /// Returns known discovered theme names.
    pub fn known_themes(&self) -> Option<&HashSet<String>> {
        self.known_themes
    }

    /// Returns the currently configured fallback/default theme.
    pub fn fallback_theme(&self) -> Option<&str> {
        self.fallback_theme
    }

    /// Returns the currently active runtime theme override.
    pub fn active_theme(&self) -> Option<&str> {
        self.active_theme
    }
}

/// User-implemented provider trait for extending HTML parsing behavior.
pub trait UiProvider: Send + Sync + 'static {
    /// Returns the HTML tag name handled by this provider.
    fn tag(&self) -> &'static str;

    /// Returns structural rules for this provider.
    fn rules(&self) -> ProviderRules {
        ProviderRules::default()
    }

    /// Resolves provider effects from the current node context.
    fn resolve(&self, ctx: ProviderResolveContext<'_>) -> Result<ProviderEffect, String>;
}

/// Registry of active UI providers.
#[derive(Resource, Default)]
pub struct UiProviderRegistry {
    providers: Vec<Arc<dyn UiProvider>>,
}

impl UiProviderRegistry {
    /// Registers (or replaces) a provider by tag.
    pub fn register<P: UiProvider>(&mut self, provider: P) {
        let tag = provider.tag();
        self.providers.retain(|current| current.tag() != tag);
        self.providers.push(Arc::new(provider));
    }

    /// Returns an iterator over all registered providers.
    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn UiProvider>> {
        self.providers.iter()
    }
}

/// App extension for provider registration.
pub trait UiProviderAppExt {
    /// Registers a custom provider.
    fn register_ui_provider<P: UiProvider>(&mut self, provider: P) -> &mut Self;
}

impl UiProviderAppExt for App {
    fn register_ui_provider<P: UiProvider>(&mut self, provider: P) -> &mut Self {
        self.init_resource::<UiProviderRegistry>();
        self.world_mut()
            .resource_mut::<UiProviderRegistry>()
            .register(provider);
        self
    }
}

/// Plugin that wires the provider registry and built-in providers.
pub struct ExtendedUiProviderPlugin;

impl Plugin for ExtendedUiProviderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiProviderRegistry>();
        app.init_resource::<ThemeProviderState>();
        app.world_mut()
            .resource_mut::<UiProviderRegistry>()
            .register(ThemeProvider);
        app.add_systems(Startup, theme_provider::refresh_theme_provider_state);
        app.add_systems(
            Update,
            (
                theme_provider::refresh_theme_provider_state
                    .run_if(resource_changed::<crate::ExtendedUiConfiguration>),
                theme_provider::apply_theme_switch_requests,
            ),
        );
    }
}
