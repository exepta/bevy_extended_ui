use bevy::prelude::*;
use bevy_extended_ui::html::HtmlInit;
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

#[html_fn("init_main")]
pub fn init_main(In(_): In<HtmlInit>) {
    println!("init_main: hello world!");
}
