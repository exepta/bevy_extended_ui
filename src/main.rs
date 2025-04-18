use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::styles::{BaseStyle, PartialStyle};
use bevy_extended_ui::styles::css_types::Background;
use bevy_extended_ui::widgets::containers::{ DivContainer };
use bevy_extended_ui::widgets::button::Button;
use bevy_extended_ui::widgets::input::{InputCap, InputField, InputType};

fn main() {
    let _ = App::new()
        .add_plugins((DefaultPlugins, ExtendedUiPlugin))
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
            gap_column: Some(Val::Px(10.0)),
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
                placeholder_text: "Placeholder".to_string(),
                icon: Some(asset_server.load("icons/user-icon.png")),
                input_type: InputType::Text,
                cap_text_at: InputCap::NoCap,
                ..default()
            }
        );
    });
}