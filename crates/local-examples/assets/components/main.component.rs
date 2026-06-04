use crate::data_structs::{DataPack, DataState};
use bevy::prelude::*;
use bevy_extended_ui::BeuStore;
use bevy_extended_ui::framework::UiBindingStore;
use bevy_extended_ui::html::{HtmlChange, HtmlClick, HtmlInit};
use bevy_extended_ui::widgets::{InputField, InputValue, Slider};
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
    pub image_file: String,
    pub date: String,
    pub date_range: String,
    pub dates: Vec<String>,
    pub dates_in_range: Vec<String>,
}

impl Default for Info {
    fn default() -> Self {
        let dates = vec![
            "01.01.2022".to_string(),
            "01.07.2022".to_string(),
            "01.01.2023".to_string(),
            "01.07.2023".to_string(),
            "01.01.2024".to_string(),
            "01.07.2024".to_string(),
            "01.01.2025".to_string(),
            "01.07.2025".to_string(),
            "01.01.2026".to_string(),
            "04.06.2026".to_string(),
        ];

        Self {
            display: "Hello World!".to_string(),
            see_mee: "See mee!".to_string(),
            value: 10.0,
            image_file: String::new(),
            date: "01.01.2022".to_string(),
            date_range: "01.01.2022 - 04.06.2026".to_string(),
            dates_in_range: dates.clone(),
            dates,
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

#[html_fn("set_name")]
pub fn set_name(
    In(event): In<HtmlChange>,
    mut store: ResMut<UiBindingStore>,
    inputs: Query<&InputField>,
) {
    let Ok(input) = inputs.get(event.entity) else {
        return;
    };

    let mut player = store.get_store::<Player>().cloned().unwrap_or_default();
    player.name = input.text.clone();
    store.set_store(player);
}

#[html_fn("set_value")]
pub fn set_value(
    In(event): In<HtmlChange>,
    mut store: ResMut<UiBindingStore>,
    inputs: Query<&InputField>,
) {
    let Ok(input) = inputs.get(event.entity) else {
        return;
    };

    let mut info = store.get_store::<Info>().cloned().unwrap_or_default();
    info.value = input.text.clone().parse::<f32>().unwrap_or(0.0);
    store.set_store(info);
}

#[html_fn("set_image_file")]
pub fn set_image_file(
    In(event): In<HtmlChange>,
    mut store: ResMut<UiBindingStore>,
    inputs: Query<&InputValue>,
) {
    let Ok(input_value) = inputs.get(event.entity) else {
        return;
    };

    let mut info = store.get_store::<Info>().cloned().unwrap_or_default();
    info.image_file = input_value.0.clone();
    store.set_store(info);
}

#[html_fn("set_info_date")]
pub fn set_info_date(
    In(event): In<HtmlChange>,
    mut store: ResMut<UiBindingStore>,
    inputs: Query<&InputField>,
) {
    let Ok(input) = inputs.get(event.entity) else {
        return;
    };

    let mut info = store.get_store::<Info>().cloned().unwrap_or_default();
    info.date = input.text.clone();
    store.set_store(info);
}

#[html_fn("set_info_date_range")]
pub fn set_info_date_range(
    In(event): In<HtmlChange>,
    mut store: ResMut<UiBindingStore>,
    inputs: Query<&InputField>,
) {
    let Ok(input) = inputs.get(event.entity) else {
        return;
    };

    let mut info = store.get_store::<Info>().cloned().unwrap_or_default();
    info.date_range = input.text.clone();
    info.dates_in_range = filter_dates_by_range(&info.dates, &info.date_range);
    store.set_store(info);
}

fn filter_dates_by_range(dates: &[String], range: &str) -> Vec<String> {
    let Some((start, end)) = parse_date_range(range) else {
        return dates.to_vec();
    };

    dates
        .iter()
        .filter(|date| {
            parse_german_date_key(date)
                .map(|key| key >= start && key <= end)
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

fn parse_date_range(range: &str) -> Option<((u16, u8, u8), (u16, u8, u8))> {
    let (left, right) = range.split_once(" - ")?;
    let start = parse_german_date_key(left.trim())?;
    let end = parse_german_date_key(right.trim())?;

    if start <= end {
        Some((start, end))
    } else {
        Some((end, start))
    }
}

fn parse_german_date_key(value: &str) -> Option<(u16, u8, u8)> {
    let mut parts = value.trim().split('.');
    let day = parts.next()?.parse::<u8>().ok()?;
    let month = parts.next()?.parse::<u8>().ok()?;
    let year = parts.next()?.parse::<u16>().ok()?;

    if parts.next().is_some() || day == 0 || month == 0 || month > 12 {
        return None;
    }

    Some((year, month, day))
}

#[html_fn("init_main")]
pub fn init_main(In(_): In<HtmlInit>) {
    println!("init_main: hello world!");
}
