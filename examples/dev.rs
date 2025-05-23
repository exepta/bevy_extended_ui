use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::styling::convert::{CssClass, CssID, CssSource};
use bevy_extended_ui::styling::IconPlace;
use bevy_extended_ui::widgets::{Button, CheckBox, ChoiceBox, ChoiceOption, Div, InputCap, InputField, InputType, Slider};

fn main() {
    let _ = App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Debug UI".to_string(),
                    resolution: WindowResolution::new(1270.0, 720.0),
                    ..default()
                }),
                ..default()
            }
        ))
        .add_plugins(ExtendedUiPlugin)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_plugins(WorldInspectorPlugin::default()
            .run_if(input_toggle_active(false, KeyCode::F1)))
        .add_systems(Startup, test_button)
        .run();
}

fn test_button(mut commands: Commands) {
    commands.spawn((
        Div::default(),
        CssSource(String::from("examples/css/dev.css")),
        CssClass(vec!["div-test".to_string(), "div-override".to_string()]),
        CssID("container".to_string()),
        children![
            (
                InputField {
                    input_type: InputType::Text,
                    cap_text_at: InputCap::NoCap,
                    ..default()
                },
                CssID("input-id".to_string()),
            ),
            (
                ChoiceBox::default(),
            ),
            Button {
                icon_path: Some(String::from("icons/pass-icon.png")),
                icon_place: IconPlace::Left,
                ..default()
            },
                // This generates a normal button without any custom styling
            (
                Button::default(),
            ),
            (
                CheckBox::default(),
            ),
            (
                Slider::default(),
            ),
            (
                ChoiceBox {
                    options: vec![
                        ChoiceOption::default(),
                        ChoiceOption::new("Test 2"),
                        ChoiceOption::new("Test 3"), 
                        ChoiceOption::new("Test 4"), 
                        ChoiceOption::new("Test 5"),
                        ChoiceOption::new("Test 6")
                    ],
                    ..default()
                },
            ),
        ]
    ));
}