use bevy::prelude::{In, ResMut};
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui::routing::Router;
use bevy_extended_ui_macros::*;

#[ui_component]
pub struct SettingsComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const SETTINGS_COMPONENT: SettingsComponent = SettingsComponent {
    template_name: "app-settings",
    template_file: "settings.component.html",
    styles: &["settings.component.css"],
};

#[html_fn("settings_go_home")]
pub fn settings_go_home(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/");
}

#[html_fn("settings_go_info")]
pub fn settings_go_info(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/info");
}
