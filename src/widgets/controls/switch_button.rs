use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{
    BindToID, SwitchButton, UIGenID, UIWidgetState, WidgetId, WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

#[derive(Component)]
struct SwitchButtonBase;

#[derive(Component)]
pub struct SwitchButtonTrack;

#[derive(Component)]
pub struct SwitchButtonDot;

#[derive(Component)]
struct SwitchButtonLabel;

pub struct SwitchButtonWidget;

impl Plugin for SwitchButtonWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &SwitchButton, Option<&CssSource>),
        (With<SwitchButton>, Without<SwitchButtonBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, id, switch_button, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        let dot_icon = switch_button.icon.as_ref().map(|icon_path| {
            image_cache
                .map
                .entry(icon_path.clone())
                .or_insert_with(|| asset_server.load(icon_path.clone())).clone()
        });

        commands
            .entity(entity)
            .insert((
                Name::new(format!("SwitchButton-{}", switch_button.entry)),
                Node {
                    ..default()
                },
                WidgetId {
                    id: switch_button.entry,
                    kind: WidgetKind::SwitchButton,
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
                TagName(String::from("switch")),
                RenderLayers::layer(*layer),
                SwitchButtonBase,
            ))
            .with_children(|builder| {
                builder
                    .spawn((
                        Name::new(format!("Switch-Track-{}", switch_button.entry)),
                        Node {
                            ..default()
                        },
                        BackgroundColor::default(),
                        BorderColor::default(),
                        BorderRadius::default(),
                        css_source.clone(),
                        UIWidgetState::default(),
                        CssClass(vec!["switch-track".to_string()]),
                        Pickable::IGNORE,
                        BindToID(id.0),
                        RenderLayers::layer(*layer),
                        SwitchButtonTrack,
                    ))
                    .with_children(|track| {
                        track
                            .spawn((
                                Name::new(format!("Switch-Dot-{}", switch_button.entry)),
                                Node::default(),
                                BackgroundColor::default(),
                                BorderColor::default(),
                                BorderRadius::default(),
                                css_source.clone(),
                                UIWidgetState::default(),
                                CssClass(vec!["switch-dot".to_string()]),
                                Pickable::IGNORE,
                                BindToID(id.0),
                                RenderLayers::layer(*layer),
                                SwitchButtonDot,
                            ))
                            .with_children(|dot| {
                                if let Some(handle) = dot_icon.clone() {
                                    dot.spawn((
                                        Name::new(format!(
                                            "Switch-Dot-Icon-{}",
                                            switch_button.entry
                                        )),
                                        ImageNode::new(handle),
                                        ZIndex::default(),
                                        UIWidgetState::default(),
                                        css_source.clone(),
                                        CssClass(vec!["icon-dot".to_string()]),
                                        Pickable::IGNORE,
                                        BindToID(id.0),
                                        RenderLayers::layer(*layer),
                                    ));
                                }
                            });
                    });

                builder.spawn((
                    Name::new(format!("Switch-Label-{}", switch_button.entry)),
                    Text::new(switch_button.label.clone()),
                    TextColor::default(),
                    TextFont::default(),
                    TextLayout::default(),
                    css_source.clone(),
                    UIWidgetState::default(),
                    CssClass(vec!["switch-text".to_string()]),
                    Pickable::IGNORE,
                    BindToID(id.0),
                    RenderLayers::layer(*layer),
                    SwitchButtonLabel,
                ));
            })
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut switch_q: Query<(&mut UIWidgetState, &UIGenID), With<SwitchButton>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = switch_q.get_mut(trigger.entity) {
        if state.disabled {
            trigger.propagate(false);
            return;
        }
        state.checked = !state.checked;
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
    trigger.propagate(false);
}

fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<SwitchButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<SwitchButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}