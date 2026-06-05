use bevy_extended_ui::routing::Routes;

pub fn secondary_routes() -> Routes {
    Routes::new()
        .route("/settings", "app-settings")
        .route("/info", "app-infopage")
}
