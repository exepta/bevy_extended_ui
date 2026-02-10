use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::io::{CssAsset, DefaultCssHandle};
use bevy_extended_ui::styles::{CssClass, CssSource};
use bevy_extended_ui::widgets::{Body, Button, CheckBox, Div, InputField, InputType};

/// Runs the contact form example without HTML parsing.
fn main() {
    let mut app = make_app("Contact Form (CSS only)");
    app.add_systems(PostStartup, setup);
    app.run();
}

/// Builds the contact form UI tree directly in code.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    default_css: Res<DefaultCssHandle>,
) {
    let custom_css: Handle<CssAsset> = asset_server.load("examples/contact_form.css");
    let css = CssSource(vec![default_css.0.clone(), custom_css]);

    let body = commands.spawn((Body::default(), css.clone())).id();

    commands.entity(body).with_children(|builder| {
        builder
            .spawn((
                Div::default(),
                css.clone(),
                CssClass(vec!["contact-card".to_string()]),
            ))
            .with_children(|builder| {
                builder.spawn((
                    InputField {
                        label: "Name".to_string(),
                        placeholder: "Dein Name".to_string(),
                        ..default()
                    },
                    css.clone(),
                    CssClass(vec!["contact-input".to_string()]),
                ));

                builder.spawn((
                    InputField {
                        label: "E-Mail".to_string(),
                        placeholder: "name@example.com".to_string(),
                        input_type: InputType::Email,
                        ..default()
                    },
                    css.clone(),
                    CssClass(vec!["contact-input".to_string()]),
                ));

                builder.spawn((
                    CheckBox {
                        label: "Ich stimme zu".to_string(),
                        ..default()
                    },
                    css.clone(),
                    CssClass(vec!["contact-check".to_string()]),
                ));

                builder.spawn((
                    Button {
                        text: "Nachricht senden".to_string(),
                        ..default()
                    },
                    css.clone(),
                    CssClass(vec!["primary-button".to_string()]),
                ));
            });
    });
}
