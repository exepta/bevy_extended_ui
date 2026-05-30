use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlClick, HtmlInit};
use bevy_extended_ui_macros::*;
use serde::Serialize;

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

#[html_use]
#[derive(Serialize)]
pub struct Player {
    pub state: bool,
    pub name: String,
    pub list: Vec<String>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            state: true,
            name: "John".to_string(),
            list: vec!["Alice".to_string(), "Bob".to_string()],
        }
    }
}

#[html_shared]
#[derive(Serialize)]
pub struct Info {
    pub display: String,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            display: "Hello World!".to_string(),
        }
    }
}

#[html_fn("check_state")]
pub fn check_state(In(_): In<HtmlClick>, mut player: ResMut<Player>) {
    player.state = !player.state;
}

#[html_fn("init_main")]
pub fn init_main(
    In(_): In<HtmlInit>,
    mut commands: Commands,
    player: Option<Res<Player>>,
    info: Option<Res<Info>>,
) {
    if player.is_none() {
        commands.insert_resource(Player::default());
    }
    if info.is_none() {
        commands.insert_resource(Info::default());
    }

    println!("init_main: hello world!");
}
