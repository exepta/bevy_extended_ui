use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::paint::Colored;
use crate::widgets::InputField;

#[derive(Component)]
struct InputFieldBase;

#[derive(Component)]
struct InputFieldText;

#[derive(Component)]
struct InputFieldIcon;

#[derive(Component)]
struct InputCursor;

#[derive(Component, Clone)]
struct CursorColor(pub Color);

#[derive(Component)]
struct InputContainer;

#[derive(Component)]
struct OverlayLabel;

#[derive(Resource, Default)]
struct KeyRepeatTimers {
    timers: HashMap<KeyCode, Timer>,
}

#[derive(Resource)]
pub struct CursorBlinkTimer {
    pub timer: Timer,
}

impl Default for CursorBlinkTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.95, TimerMode::Repeating)
        }
    }
}

pub struct InputWidget;

impl Plugin for InputWidget {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyRepeatTimers::default());
        app.insert_resource(CursorBlinkTimer::default());
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &InputField, Option<&CssSource>), (With<InputField>, Without<InputFieldBase>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, field, source_opt) in query.iter() {
        let mut css_source = CssSource(String::from("assets/css/core.css"));
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("Input-{}", field.w_count)),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            css_source.clone(),
            TagName(String::from("input")),
            RenderLayers::layer(*layer),
            InputFieldBase
        )).with_children(|builder| {
            if let Some(icon_path) = field.icon_path.clone() {
                // Icon left
                builder.spawn((
                    Name::new(format!("Input-Icon-{}", field.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    UIWidgetState::default(),
                    css_source.clone(),
                    CssClass(vec!["in-icon-container".to_string()]),
                    Pickable::IGNORE,
                    RenderLayers::layer(*layer),
                    InputFieldIcon,
                    BindToID(id.0),
                    children![
                        (
                            Name::new(format!("Icon-{}", field.w_count)),
                            ImageNode {
                                image: asset_server.load(icon_path),
                                ..default()
                            },
                            UIWidgetState::default(),
                            css_source.clone(),
                            CssClass(vec!["in-icon".to_string()]),
                            Pickable::IGNORE,
                            RenderLayers::layer(*layer),
                            BindToID(id.0),
                        )
                    ]
                ));
            }
            
            // Overlay label
            builder.spawn((
                Name::new(format!("Input-Label-{}", field.w_count)),
                Node::default(),
                Text::new(field.label.clone()),
                TextColor::default(),
                TextLayout::default(),
                TextFont::default(),
                UIWidgetState::default(),
                css_source.clone(),
                CssClass(vec!["input-label".to_string()]),
                Pickable::IGNORE,
                RenderLayers::layer(*layer),
                OverlayLabel,
                BindToID(id.0)
            ));
            
            // Text content children
            builder.spawn((
                Name::new(format!("Input-Text-Container-{}", field.w_count)),
                Node::default(),
                BackgroundColor::default(),
                BorderColor::default(),
                BorderRadius::default(),
                UIWidgetState::default(),
                css_source.clone(),
                CssClass(vec!["input-label".to_string()]),
                Pickable::IGNORE,
                RenderLayers::layer(*layer),
                InputContainer,
                BindToID(id.0),
                children![
                    // Input Cursor
                    (
                        Name::new(format!("Cursor-{}", field.w_count)),
                        Node::default(),
                        BackgroundColor::default(),
                        BorderColor::default(),
                        BorderRadius::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["input-cursor".to_string()]),
                        Pickable::IGNORE,
                        RenderLayers::layer(*layer),
                        InputCursor,
                        BindToID(id.0),
                    ),
                    // Input Text
                    (
                        Name::new(format!("Text-{}", field.w_count)),
                        Node::default(),
                        Text::new(field.text.clone()),
                        TextColor::default(),
                        TextLayout::default(),
                        TextFont::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["input-text".to_string()]),
                        Pickable::IGNORE,
                        RenderLayers::layer(*layer),
                        InputCursor,
                        BindToID(id.0),
                    )
                ]
            ));
        });
    }
}