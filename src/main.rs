use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::styles::{BaseStyle, PartialStyle};
use bevy_extended_ui::styles::css_types::Background;
use bevy_extended_ui::widgets::{DivContainer, Button, InputField, Slider, CheckBox};
use bevy_extended_ui::widgets::input::{InputCap, InputType};

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
        BaseStyle(PartialStyle {
            width: Some(Val::Percent(100.0)),
            height: Some(Val::Percent(100.0)),
            display: Some(Display::Flex),
            flex_direction: Some(FlexDirection::Column),
            gap_column: Some(Val::Px(20.0)),
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            border: Some(UiRect::all(Val::Px(5.0))),
            margin: Some(UiRect::all(Val::Px(0.0))),
            border_color: Some(Color::srgb(0.0, 0.0, 1.0)),
            background: Some(Background { /*image: Some(asset_server.load("backgrounds/test.jpg")), */ ..default() }),
            ..default()
        }),
    )).with_children(| builder | {
        builder.spawn(
            Button::default()
        );

        builder.spawn(
            Button {
                icon: Some(asset_server.load("icons/drop-arrow.png")),
                ..default()
            }
        );

        builder.spawn(
            InputField {
                placeholder_text: "Username".to_string(),
                icon: Some(asset_server.load("icons/user-icon.png")),
                input_type: InputType::Text,
                cap_text_at: InputCap::NoCap,
                ..default()
            }
        );

        builder.spawn(
            InputField {
                placeholder_text: "Password".to_string(),
                icon: Some(asset_server.load("icons/pass-icon.png")),
                input_type: InputType::Password,
                cap_text_at: InputCap::NoCap,
                ..default()
            }
        );

        builder.spawn(Slider::default());
        builder.spawn(Slider::default());
        
        builder.spawn(CheckBox::default());
    });
}