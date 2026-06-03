use crate::services::image_service::get_or_load_image;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{BindToID, SwitchButton, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Marker component for initialized switch button widgets.
#[derive(Component)]
struct SwitchButtonBase;

/// Marker component for the switch track.
#[derive(Component)]
pub struct SwitchButtonTrack;

/// Marker component for the switch dot.
#[derive(Component)]
pub struct SwitchButtonDot;

/// Marker component for the switch label node.
#[derive(Component)]
struct SwitchButtonLabel;

/// Plugin that registers switch button widget behavior.
pub struct SwitchButtonWidget;

impl Plugin for SwitchButtonWidget {
    /// Registers systems for switch button setup and interaction.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                update_switch_button_system,
                sync_switch_component_from_state,
            ),
        );
    }
}

/// Initializes UI nodes for switch button widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &UIGenID,
            &SwitchButton,
            Option<&CssSource>,
            &mut UIWidgetState,
        ),
        (With<SwitchButton>, Without<SwitchButtonBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, id, switch_button, source_opt, mut state) in query.iter_mut() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }
        state.checked = switch_button.selected;

        let dot_icon = switch_button.icon.as_ref().map(|icon_path| {
            get_or_load_image(icon_path, &mut image_cache, &mut images, &asset_server)
        });

        commands
            .entity(entity)
            .insert((
                Name::new(format!("SwitchButton-{}", switch_button.entry)),
                Node { ..default() },
                WidgetId {
                    id: switch_button.entry,
                    kind: WidgetKind::SwitchButton,
                },
                BackgroundColor::default(),
                ImageNode::default(),
                BorderColor::default(),
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
                        Node { ..default() },
                        BackgroundColor::default(),
                        BorderColor::default(),
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

fn update_switch_button_system(
    switch_q: Query<
        (&SwitchButton, &UIGenID),
        (
            With<SwitchButton>,
            With<SwitchButtonBase>,
            Changed<SwitchButton>,
        ),
    >,
    mut label_q: Query<(&BindToID, &mut Text), With<SwitchButtonLabel>>,
) {
    for (switch_button, id) in switch_q.iter() {
        set_switch_label_text_for_id(id.0, &switch_button.label, &mut label_q);
    }
}

fn sync_switch_component_from_state(
    mut switch_q: Query<
        (&UIWidgetState, &mut SwitchButton),
        (With<SwitchButton>, With<SwitchButtonBase>),
    >,
) {
    for (state, mut switch_button) in switch_q.iter_mut() {
        if switch_button.selected != state.checked {
            switch_button.selected = state.checked;
        }
    }
}

fn set_switch_label_text_for_id(
    bind_id: usize,
    value: &str,
    label_q: &mut Query<(&BindToID, &mut Text), With<SwitchButtonLabel>>,
) {
    for (bind_to, mut text) in label_q.iter_mut() {
        if bind_to.0 == bind_id {
            text.0 = value.to_string();
        }
    }
}

/// Handles click events for switch buttons and toggles state.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut switch_q: Query<
        (Entity, &mut UIWidgetState, &UIGenID, &mut SwitchButton),
        With<SwitchButton>,
    >,
    switch_tag_q: Query<(), With<SwitchButton>>,
    bind_q: Query<&BindToID>,
    parent_q: Query<&ChildOf>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let mut target_entity = trigger.entity;
    let mut resolved_entity = None;

    if switch_tag_q.get(target_entity).is_ok() {
        resolved_entity = Some(target_entity);
    } else {
        while let Ok(parent) = parent_q.get(target_entity) {
            target_entity = parent.parent();
            if switch_tag_q.get(target_entity).is_ok() {
                resolved_entity = Some(target_entity);
                break;
            }
        }
    }

    if resolved_entity.is_none() {
        if let Ok(bind_to) = bind_q.get(trigger.entity) {
            for (entity, _, gen_id, _) in switch_q.iter_mut() {
                if gen_id.0 == bind_to.0 {
                    resolved_entity = Some(entity);
                    break;
                }
            }
        }
    }

    if let Some(entity) = resolved_entity {
        let Ok((_, mut state, gen_id, mut switch_button)) = switch_q.get_mut(entity) else {
            trigger.propagate(false);
            return;
        };

        if state.disabled {
            trigger.propagate(false);
            return;
        }

        state.checked = !state.checked;
        switch_button.selected = state.checked;
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
    trigger.propagate(false);
}

/// Sets hovered state when the cursor enters a switch button.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<SwitchButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

/// Clears hovered state when the cursor leaves a switch button.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<SwitchButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}
