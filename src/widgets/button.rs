use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::widgets::Button;

#[derive(Component)]
struct ButtonBase;

#[derive(Component)]
struct ButtonText;

#[derive(Component)]
struct ButtonImage;

pub struct ButtonWidget;

impl Plugin for ButtonWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &Button, Option<&CssSource>), (With<Button>, Without<ButtonBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, button, source_opt) in query.iter() {
        let mut css_internal = "assets/css/core.css";
        if let Some(source) = source_opt {
            css_internal = source.0.as_str();
        }
        info!("Spawn with {}", css_internal);
        commands.entity(entity).insert((
            Name::new(format!("Button-{}", button.w_count)),
            Node {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            WidgetStyle::load_from_file(css_internal),
            TagName("button".to_string()),
            RenderLayers::layer(*layer),
            ButtonBase,
            children![
                (
                    Name::new(format!("Button-Text-{}", button.w_count)),
                    Text::new(button.text.clone()),
                    TextColor::default(),
                    TextFont::default(),
                    TextLayout::default(),
                    WidgetStyle::load_from_file(css_internal),
                    UIWidgetState::default(),
                    CssClass(vec![".button-text".to_string()]),
                    Pickable::IGNORE,
                    BindToID(id.0),
                    RenderLayers::layer(*layer),
                    ButtonText
                ),
            ]
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Button>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Button>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Button>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}