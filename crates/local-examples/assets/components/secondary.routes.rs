use bevy_extended_ui::routing::Routes;
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn secondary_routes() -> Routes {
    Routes::new()
        .route("/settings", "app-settings")
        .route("/info", "app-infopage")
}
