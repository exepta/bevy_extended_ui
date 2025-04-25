use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::global::UiElementState;
use bevy_extended_ui::styles::css_types::Background;
use bevy_extended_ui::styles::Style;
use bevy_extended_ui::styles::types::DivStyle;
use bevy_extended_ui::widgets::{DivContainer, Button, CheckBox, Slider, InputField};

fn main() {
    let _ = App::new()
        .add_plugins((DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Debug UI".to_string(),
                    resolution: WindowResolution::new(1270.0, 720.0),
                    ..default()
                }),
                ..default()
            }
        ), ExtendedUiPlugin))
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)))
        .add_systems(Startup, example_widget)
        .run();
}

fn example_widget(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DivContainer, 
        DivStyle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }
        }
    )).with_children(| builder | {
        builder.spawn((
            DivContainer,
            DivStyle {
                style: Style {
                    width: Val::Percent(50.),
                    height: Val::Percent(50.),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    gap_column: Val::Px(20.),
                    background: Background { 
                        color: Color::srgba(0.99, 0.99, 0.99, 1.0),
                        image: None,
                    },
                    ..default()
                },
            }
        )).with_children(|builder| {
            builder.spawn(
                InputField {
                    label: String::from("Username"),
                    icon: Some(asset_server.load("icons/user-icon.png")),
                    ..default()
                }
            );

            builder.spawn(
                Button::default()
            );

            builder.spawn((
                Button::default(),
                UiElementState {
                    disabled: true,
                    ..default()
                }
            ));

            builder.spawn(
                CheckBox::default()
            );

            builder.spawn(
                Slider::default()
            );
        });
    });
}