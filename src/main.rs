use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::styles::{BaseStyle, Style};
use bevy_extended_ui::widgets::containers::{ DivContainer };
use bevy_extended_ui::widgets::button::Button;

fn main() {
    let _ = App::new()
        .add_plugins((DefaultPlugins, ExtendedUiPlugin))
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)))
        .add_systems(Startup, example_widget)
        .run();
}

fn example_widget(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load::<Image>("icons/drop-arrow.png");
    commands.spawn((
        DivContainer,
        BaseStyle(Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            gap_column: Val::Px(10.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(5.0)),
            ..default()
        }),
    )).with_children(| builder | {
        builder.spawn(
            Button::default()
        );

        builder.spawn(
            Button {
                icon: Some(icon),
                ..default()
            }
        );
    });
}