use bevy::prelude::{In, ResMut};
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui::routing::Router;
use bevy_extended_ui_macros::*;

#[ui_component]
pub struct HelpComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const HELP_COMPONENT: HelpComponent = HelpComponent {
    template_name: "app-help",
    template_file: "help.component.html",
    styles: &["help.component.css"],
};

#[html_fn("go_home")]
pub fn go_home(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/");
}
