use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlClick, HtmlInit};
use bevy_extended_ui::lang::UiLangVariables;
use bevy_extended_ui_macros::*;

#[ui_component]
pub struct MainComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const MAIN_COMPONENT: MainComponent = MainComponent {
    template_name: "app-main",
    template_file: "main.component.html",
    styles: &["main.component.css"],
};

#[html_fn("check_state")]
pub fn check_state(In(_): In<HtmlClick>, mut vars: ResMut<UiLangVariables>) {
    let current_state = vars
        .vars
        .get("state")
        .map(|value| value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    vars.set("state", if current_state { "false" } else { "true" });
}

#[html_fn("init_main")]
pub fn init_main(In(_): In<HtmlInit>, mut vars: ResMut<UiLangVariables>) {
    if !vars.vars.contains_key("state") {
        vars.set("state", "false");
    }

    println!("init_main: hello world!");
}
