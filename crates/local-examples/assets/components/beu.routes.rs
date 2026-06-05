use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
        .route("/test", "app-test")
        .redirect("", "/")
        .fallback("app-main")
}
