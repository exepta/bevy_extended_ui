use bevy::prelude::*;
use std::collections::HashMap;

pub use crate::load;
pub use inventory;

/// Marks a route component as keep-alive.
///
/// Keep-alive route components are rendered inside the router outlet even when
/// inactive. The inactive wrapper uses `display: none`, so it does not consume
/// layout space, but the already spawned widget tree can be reused instantly
/// when the route becomes active again.
#[macro_export]
macro_rules! load {
    ($component:expr) => {
        $crate::routing::RouteTarget::load($component)
    };
}

/// Declarative route table used by the extended framework router.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Routes {
    routes: Vec<Route>,
    redirects: Vec<RouteRedirect>,
    fallback_component: Option<String>,
}

impl Routes {
    /// Creates an empty route table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a path that renders the component with the given template name.
    pub fn route(mut self, path: impl Into<String>, component: impl Into<RouteTarget>) -> Self {
        let target = component.into();
        self.routes.push(Route {
            path: normalize_route_path(path.into()),
            component: target.component,
            keep_alive: target.keep_alive,
        });
        self
    }

    /// Merges another route table into this one.
    ///
    /// The input can be a [`Routes`] value or a function that returns [`Routes`].
    /// This supports Angular-like route file composition:
    ///
    /// ```rust
    /// # use bevy_extended_ui::routing::Routes;
    /// fn secondary_routes() -> Routes {
    ///     Routes::new().route("/settings", "app-settings")
    /// }
    ///
    /// let routes = Routes::new()
    ///     .route("/", "app-main")
    ///     .merge(secondary_routes);
    /// ```
    pub fn merge(self, routes: impl IntoRoutes) -> Self {
        merge_routes(self, routes.into_routes())
    }

    /// Registers a redirect from one path to another.
    pub fn redirect(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.redirects.push(RouteRedirect {
            from: normalize_route_path(from.into()),
            to: normalize_route_path(to.into()),
        });
        self
    }

    /// Registers the component rendered when no route matches.
    pub fn fallback(mut self, component: impl Into<String>) -> Self {
        self.fallback_component = Some(component.into());
        self
    }

    /// Returns the component template name for a requested path.
    pub fn resolve_component(&self, path: &str) -> Option<&str> {
        let resolved_path = self.resolve_redirect(path);
        self.routes
            .iter()
            .find(|route| route.path == resolved_path)
            .map(|route| route.component.as_str())
            .or(self.fallback_component.as_deref())
    }

    /// Returns all registered routes.
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Returns all registered redirects.
    pub fn redirects(&self) -> &[RouteRedirect] {
        &self.redirects
    }

    /// Returns the fallback component template name, when configured.
    pub fn fallback_component(&self) -> Option<&str> {
        self.fallback_component.as_deref()
    }

    fn resolve_redirect(&self, path: &str) -> String {
        let mut current = normalize_route_path(path);

        for _ in 0..16 {
            let Some(redirect) = self
                .redirects
                .iter()
                .find(|redirect| redirect.from == current)
            else {
                return current;
            };
            if redirect.to == current {
                return current;
            }
            current = redirect.to.clone();
        }

        warn!("Route redirect loop detected for path `{}`", path);
        current
    }
}

/// A single path-to-component mapping.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub path: String,
    pub component: String,
    pub keep_alive: bool,
}

/// Route component target configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteTarget {
    component: String,
    keep_alive: bool,
}

impl RouteTarget {
    /// Creates a normal route target. The component can be despawned when
    /// another route becomes active.
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            keep_alive: false,
        }
    }

    /// Creates a keep-alive route target. Use through `load!("app-main")` in
    /// route files.
    pub fn load(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            keep_alive: true,
        }
    }
}

impl From<&str> for RouteTarget {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RouteTarget {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Converts a route source into a concrete route table.
pub trait IntoRoutes {
    /// Returns a concrete route table.
    fn into_routes(self) -> Routes;
}

impl IntoRoutes for Routes {
    fn into_routes(self) -> Routes {
        self
    }
}

impl<F> IntoRoutes for F
where
    F: FnOnce() -> Routes,
{
    fn into_routes(self) -> Routes {
        self()
    }
}

/// A single path redirect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteRedirect {
    pub from: String,
    pub to: String,
}

/// Runtime router state.
#[derive(Resource, Clone, Debug)]
pub struct Router {
    routes: Routes,
    current_path: String,
    revision: u64,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            routes: Routes::default(),
            current_path: "/".to_string(),
            revision: 0,
        }
    }
}

impl Router {
    /// Replaces the route table.
    pub fn configure(&mut self, routes: Routes) {
        if self.routes == routes {
            return;
        }
        self.routes = routes;
        self.bump_revision();
    }

    /// Navigates to a path and marks the router as changed when the path differs.
    pub fn navigate(&mut self, path: impl Into<String>) {
        let next = normalize_route_path(path.into());
        if self.current_path == next {
            return;
        }
        self.current_path = next;
        self.bump_revision();
    }

    /// Returns the current path.
    pub fn current_path(&self) -> &str {
        &self.current_path
    }

    /// Returns the component template name for the current route.
    pub fn active_component(&self) -> Option<&str> {
        self.routes.resolve_component(&self.current_path)
    }

    /// Returns the route table.
    pub fn routes(&self) -> &Routes {
        &self.routes
    }

    /// Returns a monotonic counter that changes after route-table or path changes.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

/// Inventory entry emitted by `#[beu_routes]`.
pub struct RoutesRegistration {
    pub name: &'static str,
    pub build: fn() -> Routes,
}

inventory::collect!(RoutesRegistration);

/// Plugin that loads route tables registered via `#[beu_routes]`.
pub struct ExtendedRoutingPlugin;

impl Plugin for ExtendedRoutingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Router>();
        app.add_systems(Startup, register_beu_routes);
    }
}

/// Registers route tables collected through inventory.
pub fn register_beu_routes(world: &mut World) {
    let mut merged = Routes::new();
    let mut has_registrations = false;
    for registration in inventory::iter::<RoutesRegistration> {
        has_registrations = true;
        let routes = (registration.build)();
        merged = merge_routes(merged, routes);
        debug!("Registered UI routes from `{}`", registration.name);
    }

    if has_registrations {
        world.resource_mut::<Router>().configure(merged);
    }
}

fn merge_routes(mut left: Routes, right: Routes) -> Routes {
    let mut by_path = left
        .routes
        .iter()
        .map(|route| (route.path.clone(), route.clone()))
        .collect::<HashMap<_, _>>();

    for route in right.routes {
        by_path.insert(route.path.clone(), route);
    }

    left.routes = by_path.into_values().collect();
    left.routes.sort_by(|a, b| a.path.cmp(&b.path));
    left.redirects.extend(right.redirects);
    if right.fallback_component.is_some() {
        left.fallback_component = right.fallback_component;
    }
    left
}

fn normalize_route_path(path: impl AsRef<str>) -> String {
    let path = path.as_ref().trim();
    if path.is_empty() {
        return "/".to_string();
    }

    let mut normalized = path.replace('\\', "/");
    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }

    while normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }

    normalized
}
