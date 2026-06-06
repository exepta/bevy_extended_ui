#[cfg(test)]
mod tests {
    use bevy_extended_ui::load;
    use bevy_extended_ui::routing::{Router, Routes};

    #[test]
    fn routes_resolve_registered_path() {
        let routes = Routes::new().route("/help", "app-help");
        assert_eq!(routes.resolve_component("/help"), Some("app-help"));
    }

    #[test]
    fn load_macro_marks_route_keep_alive() {
        let routes = Routes::new().route("/", load!("app-home"));
        assert_eq!(routes.resolve_component("/"), Some("app-home"));
        assert!(routes.routes()[0].keep_alive);
    }

    #[test]
    fn routes_merge_another_route_table() {
        let routes = Routes::new()
            .route("/", "app-home")
            .merge(Routes::new().route("/help", "app-help"));

        assert_eq!(routes.resolve_component("/"), Some("app-home"));
        assert_eq!(routes.resolve_component("/help"), Some("app-help"));
    }

    #[test]
    fn routes_merge_route_function() {
        fn secondary_routes() -> Routes {
            Routes::new().route("/settings", "app-settings")
        }

        let routes = Routes::new().route("/", "app-home").merge(secondary_routes);

        assert_eq!(routes.resolve_component("/settings"), Some("app-settings"));
    }

    #[test]
    fn routes_merge_overrides_paths_and_keeps_right_fallback() {
        let routes = Routes::new()
            .route("/settings", "app-settings-old")
            .fallback("app-old-fallback")
            .merge(
                Routes::new()
                    .route("/settings", load!("app-settings-new"))
                    .fallback("app-new-fallback"),
            );

        let settings = routes
            .routes()
            .iter()
            .find(|route| route.path == "/settings")
            .expect("settings route");
        assert_eq!(settings.component, "app-settings-new");
        assert!(settings.keep_alive);
        assert_eq!(routes.fallback_component(), Some("app-new-fallback"));
    }

    #[test]
    fn routes_merge_appends_redirects() {
        let routes = Routes::new()
            .route("/", "app-home")
            .redirect("", "/")
            .merge(Routes::new().redirect("/home", "/"));

        assert_eq!(routes.redirects().len(), 2);
        assert_eq!(routes.resolve_component("/home"), Some("app-home"));
    }

    #[test]
    fn routes_apply_redirects() {
        let routes = Routes::new().route("/", "app-home").redirect("", "/");
        assert_eq!(routes.resolve_component(""), Some("app-home"));
    }

    #[test]
    fn routes_apply_redirect_chains() {
        let routes = Routes::new()
            .route("/", "app-home")
            .redirect("/start", "/home")
            .redirect("/home", "/");

        assert_eq!(routes.resolve_component("/start"), Some("app-home"));
    }

    #[test]
    fn routes_stop_redirect_loops() {
        let routes = Routes::new()
            .route("/", "app-home")
            .redirect("/a", "/b")
            .redirect("/b", "/a");

        assert_eq!(routes.resolve_component("/a"), None);
    }

    #[test]
    fn routes_normalize_paths() {
        let routes = Routes::new().route("help\\", "app-help");
        assert_eq!(routes.resolve_component("/help/"), Some("app-help"));
    }

    #[test]
    fn routes_use_fallback_for_unknown_path() {
        let routes = Routes::new().route("/", "app-home").fallback("app-home");
        assert_eq!(routes.resolve_component("/missing"), Some("app-home"));
    }

    #[test]
    fn router_revision_changes_on_navigation() {
        let mut router = Router::default();
        let first = router.revision();
        router.navigate("/help");
        assert!(router.revision() > first);
    }

    #[test]
    fn router_revision_does_not_change_for_same_config_or_path() {
        let routes = Routes::new().route("/", "app-home");
        let mut router = Router::default();
        router.configure(routes.clone());
        let configured = router.revision();
        router.configure(routes);
        assert_eq!(router.revision(), configured);

        router.navigate("/");
        assert_eq!(router.revision(), configured);
        assert_eq!(router.current_path(), "/");
        assert_eq!(router.routes().resolve_component("/"), Some("app-home"));
    }
}
