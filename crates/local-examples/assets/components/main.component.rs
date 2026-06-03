use crate::data_structs::{DataPack, DataState};
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

#[component_init]
pub fn constructor(
    mut commands: Commands,
    player: Option<Res<Player>>,
    info: Option<Res<Info>>,
    data_pack: Option<Res<DataPack>>,
    data_state: Option<Res<DataState>>,
) {
    if player.is_none() {
        commands.insert_resource(Player::default());
    }
    if info.is_none() {
        commands.insert_resource(Info::default());
    }
    if data_pack.is_none() {
        commands.insert_resource(DataPack::default());
    }
    if data_state.is_none() {
        commands.insert_resource(DataState::default());
    }
}

/// Represents a player entity with a state, name, and a list of associated data.
///
/// # Fields
/// * `state` - A boolean representing the state of the player.
/// * `name` - A `String` representing the player's name.
/// * `list` - A vector of strings containing additional data related to the player.
#[derive(Resource, Serialize)]
pub struct Player {
    /// The state of display the test area.
    pub state: bool,
    /// The name of the player.
    pub name: String,
    pub list: Vec<String>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            state: false,
            name: "John".to_string(),
            list: vec!["Alice".to_string(), "Bob".to_string()],
        }
    }
}

/// Structure representing information for shared HTML rendering.
///
/// # Fields
///
/// * `display` - A `String` that represents the main content or data to be displayed.
/// * `see_mee` - A `String` containing additional information related to the `display`.
#[derive(Resource, Serialize)]
pub struct Info {
    pub display: String,
    pub see_mee: String,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            display: "Hello World!".to_string(),
            see_mee: "See mee!".to_string(),
        }
    }
}

#[html_fn("check_state")]
pub fn check_state(In(_): In<HtmlClick>, mut player: ResMut<Player>) {
    player.state = !player.state;
}

#[html_fn("init_main")]
pub fn init_main(In(_): In<HtmlInit>) {
    println!("init_main: hello world!");
}
