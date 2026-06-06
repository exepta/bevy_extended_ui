use crate::services::image_service::get_or_load_image;
use crate::services::style_service::apply_calc_styles_system;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{BindToID, SwitchButton, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::ui::ComputedNode;
use std::collections::HashMap;

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
                update_switch_button_system.before(sync_switch_component_from_state),
                sync_switch_component_from_state,
            ),
        );
        app.add_systems(
            PostUpdate,
            sync_switch_visual_layout.after(apply_calc_styles_system),
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
    mut switch_q: Query<
        (&SwitchButton, &UIGenID, &mut UIWidgetState),
        (
            With<SwitchButton>,
            With<SwitchButtonBase>,
            Changed<SwitchButton>,
        ),
    >,
    mut bound_state_q: Query<(&BindToID, &mut UIWidgetState), Without<UIGenID>>,
    mut track_q: Query<(&BindToID, &mut Node), (With<SwitchButtonTrack>, Without<SwitchButtonDot>)>,
    mut dot_q: Query<(&BindToID, &mut Node), (With<SwitchButtonDot>, Without<SwitchButtonTrack>)>,
    mut label_q: Query<(&BindToID, &mut Text), With<SwitchButtonLabel>>,
) {
    for (switch_button, id, mut state) in switch_q.iter_mut() {
        state.checked = switch_button.selected;
        sync_bound_switch_state(id.0, switch_button.selected, &mut bound_state_q);
        sync_switch_track_layout(id.0, switch_button.selected, &mut track_q);
        sync_switch_dot_layout(id.0, switch_button.selected, &mut dot_q);
        set_switch_label_text_for_id(id.0, &switch_button.label, &mut label_q);
    }
}

fn sync_bound_switch_state(
    bind_id: usize,
    checked: bool,
    bound_state_q: &mut Query<(&BindToID, &mut UIWidgetState), Without<UIGenID>>,
) {
    for (bind_to, mut state) in bound_state_q.iter_mut() {
        if bind_to.0 == bind_id {
            state.checked = checked;
        }
    }
}

fn sync_switch_track_layout(
    bind_id: usize,
    checked: bool,
    track_q: &mut Query<
        (&BindToID, &mut Node),
        (With<SwitchButtonTrack>, Without<SwitchButtonDot>),
    >,
) {
    let justify_content = if checked {
        JustifyContent::End
    } else {
        JustifyContent::Start
    };

    for (bind_to, mut node) in track_q.iter_mut() {
        if bind_to.0 == bind_id {
            node.justify_content = justify_content;
        }
    }
}

fn sync_switch_dot_layout(
    bind_id: usize,
    checked: bool,
    dot_q: &mut Query<(&BindToID, &mut Node), (With<SwitchButtonDot>, Without<SwitchButtonTrack>)>,
) {
    for (bind_to, mut node) in dot_q.iter_mut() {
        if bind_to.0 != bind_id {
            continue;
        }

        node.left = Val::Px(if checked { 27.0 } else { 0.0 });
        node.right = Val::Auto;
    }
}

fn sync_switch_visual_layout(
    switch_q: Query<(&UIGenID, &UIWidgetState), (With<SwitchButton>, With<SwitchButtonBase>)>,
    mut track_q: Query<
        (&BindToID, &mut Node, Option<&ComputedNode>),
        (With<SwitchButtonTrack>, Without<SwitchButtonDot>),
    >,
    mut dot_q: Query<
        (&BindToID, &mut Node, Option<&ComputedNode>),
        (With<SwitchButtonDot>, Without<SwitchButtonTrack>),
    >,
) {
    let mut checked_by_id = HashMap::new();
    for (id, state) in switch_q.iter() {
        checked_by_id.insert(id.0, state.checked);
    }

    let mut track_width_by_id = HashMap::new();
    for (bind_to, mut node, computed) in track_q.iter_mut() {
        let Some(checked) = checked_by_id.get(&bind_to.0).copied() else {
            continue;
        };

        node.justify_content = if checked {
            JustifyContent::End
        } else {
            JustifyContent::Start
        };

        let width = computed
            .map(logical_computed_width)
            .filter(|width| *width > 0.0)
            .unwrap_or(50.0);
        track_width_by_id.insert(bind_to.0, width);
    }

    for (bind_to, mut node, computed) in dot_q.iter_mut() {
        let Some(checked) = checked_by_id.get(&bind_to.0).copied() else {
            continue;
        };

        let track_width = track_width_by_id.get(&bind_to.0).copied().unwrap_or(50.0);
        let dot_width = computed
            .map(logical_computed_width)
            .filter(|width| *width > 0.0)
            .unwrap_or(23.0);
        let left = if checked {
            (track_width - dot_width).max(0.0)
        } else {
            0.0
        };

        node.left = Val::Px(left);
        node.right = Val::Auto;
    }
}

fn logical_computed_width(computed: &ComputedNode) -> f32 {
    computed.size.x * computed.inverse_scale_factor.max(f32::EPSILON)
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
