use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::styling::convert::{CssID, CssSource};
use bevy_extended_ui::widgets::{Button, Div};

fn main() {
    let _ = App::new().add_plugins((DefaultPlugins, ExtendedUiPlugin))
        .add_systems(Startup, example_button)
        .run();
}

fn example_button(mut commands: Commands) {
    commands.spawn((
        Div::default(),
        children![
                // This generates a custom button which have other css styles.
                // Note that the style can be overridden by other css files or any button tag
                // This is a known bug!
            (
                Button::default(),
                CssSource(String::from("examples/css/button.css")),
                CssID(String::from("example-id")),
            ),
                // This generates a normal button without any custom styling
            (
                Button::default(),
                CssSource(String::from("examples/css/button.css")),
                CssID(String::from("example-id-2"))
            ),
        ]
    ));
}