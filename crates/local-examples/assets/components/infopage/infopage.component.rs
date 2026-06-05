use bevy::prelude::{In, ResMut};
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui::routing::Router;
use bevy_extended_ui_macros::*;

#[ui_component]
pub struct InfoPageComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const INFO_PAGE_COMPONENT: InfoPageComponent = InfoPageComponent {
    template_name: "app-infopage",
    template_file: "infopage.component.html",
    styles: &["infopage.component.css"],
};

#[html_fn("infopage_go_home")]
pub fn infopage_go_home(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/");
}

#[html_fn("infopage_go_settings")]
pub fn infopage_go_settings(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/settings");
}
