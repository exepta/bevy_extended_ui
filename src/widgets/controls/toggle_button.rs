use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use crate::styles::{CssClass, CssSource, IconPlace, TagName};
use crate::styles::paint::Colored;
use crate::widgets::{BindToID, ToggleButton, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::widgets::controls::place_icon_if;

#[derive(Component)]
struct ToggleButtonBase;

#[derive(Component)]
struct ToggleButtonText;

pub struct ToggleButtonWidget;

impl Plugin for ToggleButtonWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &ToggleButton, Option<&CssSource>),
        (With<ToggleButton>, Without<ToggleButtonBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, toggle_button, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands
            .entity(entity)
            .insert((
                Name::new(format!("ToggleButton-{}", toggle_button.entry)),
                Node::default(),
                WidgetId {
                    id: toggle_button.entry,
                    kind: WidgetKind::ToggleButton,
                },
                BackgroundColor::default(),
                ImageNode::default(),
                BorderColor::default(),
                BorderRadius::default(),
                BoxShadow::new(
                    Colored::TRANSPARENT,
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                ),
                ZIndex::default(),
                Pickable::default(),
                css_source.clone(),
                TagName("button".to_string()),
                RenderLayers::layer(*layer),
                ToggleButtonBase,
            ))
            .with_children(|builder| {
                place_icon_if(
                    builder,
                    toggle_button.icon_place,
                    IconPlace::Left,
                    &toggle_button.icon_path,
                    toggle_button.entry,
                    &asset_server,
                    &mut image_cache,
                    vec!["button-text".to_string()],
                    id.0,
                    *layer,
                    css_source.clone(),
                );

                builder.spawn((
                    Name::new(format!("ToggleButton-Text-{}", toggle_button.entry)),
                    Text::new(toggle_button.label.clone()),
                    TextColor::default(),
                    TextFont::default(),
                    TextLayout::default(),
                    css_source.clone(),
                    UIWidgetState::default(),
                    ZIndex::default(),
                    CssClass(vec!["button-text".to_string()]),
                    Pickable::IGNORE,
                    BindToID(id.0),
                    RenderLayers::layer(*layer),
                    ToggleButtonText,
                ));

                place_icon_if(
                    builder,
                    toggle_button.icon_place,
                    IconPlace::Right,
                    &toggle_button.icon_path,
                    toggle_button.entry,
                    &asset_server,
                    &mut image_cache,
                    vec!["button-text".to_string()],
                    id.0,
                    *layer,
                    css_source.clone(),
                );
            })
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<ToggleButton>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }

    trigger.propagate(false);
}

fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<ToggleButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<ToggleButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}
