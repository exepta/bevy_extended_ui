use crate::data_structs::{DataPack, DataState};
use bevy::prelude::*;
use bevy_extended_ui::BeuStore;
use bevy_extended_ui::framework::UiBindingStore;
use bevy_extended_ui::html::{HtmlChange, HtmlClick, HtmlInit};
use bevy_extended_ui::widgets::Slider;
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
    data_pack: Option<Res<DataPack>>,
    data_state: Option<Res<DataState>>,
) {
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
#[derive(BeuStore, Clone, PartialEq, Serialize)]
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
#[derive(BeuStore, PartialEq, Clone, Serialize)]
pub struct Info {
    pub display: String,
    pub see_mee: String,
    pub value: f32,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            display: "Hello World!".to_string(),
            see_mee: "See mee!".to_string(),
            value: 10.0,
        }
    }
}

#[html_fn("check_state")]
pub fn check_state(In(_): In<HtmlClick>, mut store: ResMut<UiBindingStore>) {
    let mut player = store.get_store::<Player>().cloned().unwrap_or_default();
    player.state = !player.state;
    store.set_store(player);
}

#[html_fn("increase_value")]
pub fn increase_value(In(_): In<HtmlClick>, mut store: ResMut<UiBindingStore>) {
    let mut info = store.get_store::<Info>().cloned().unwrap_or_default();
    info.value = info.value + 1.0;
    store.set_store(info);
}

#[html_fn("on_slider_change")]
pub fn on_slider_change(
    In(event): In<HtmlChange>,
    mut store: ResMut<UiBindingStore>,
    sliders: Query<&Slider>,
) {
    let Ok(slider) = sliders.get(event.entity) else {
        return;
    };

    let mut info = store.get_store::<Info>().cloned().unwrap_or_default();
    info.value = slider.value;
    store.set_store(info);
}

#[html_fn("init_main")]
pub fn init_main(In(_): In<HtmlInit>) {
    println!("init_main: hello world!");
}
