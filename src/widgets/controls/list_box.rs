use crate::services::image_service::get_or_load_image;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::widget_util::wheel_delta_y;
use crate::widgets::{
    BindToID, ChoiceOption, IgnoreParentState, ListBox, UIGenID, UIWidgetState, WidgetId,
    WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// Marker component for initialized list box widgets.
#[derive(Component)]
struct ListBoxBase;

/// Marker component for individual list box option entries.
#[derive(Component)]
pub(crate) struct ListBoxOptionBase;

/// Plugin that registers list box widget behavior.
pub struct ListBoxWidget;

impl Plugin for ListBoxWidget {
    /// Registers systems for list box setup and interaction.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (internal_node_creation_system, handle_scroll_events).chain(),
        );
    }
}

/// System that initializes internal UI nodes for [`ListBox`] components.
///
/// Builds a scrollable list of option entries. Each option is always visible
/// (unlike `ChoiceBox` which hides options behind a dropdown).
///
/// Supports `multiselect`: clicking an option toggles it.
/// In single-select mode clicking an option deselects any previously selected one.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &ListBox, Option<&CssSource>),
        (With<ListBox>, Without<ListBoxBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, list_box, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands
            .entity(entity)
            .insert((
                Name::new(format!("List-Box-{}", list_box.entry)),
                Node::default(),
                WidgetId {
                    id: list_box.entry,
                    kind: WidgetKind::ListBox,
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
                TagName("listbox".to_string()),
                RenderLayers::layer(*layer),
                ScrollPosition::default(),
                ListBoxBase,
            ))
            .insert(GlobalZIndex::default())
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {
                for option in list_box.options.iter() {
                    let is_selected = list_box.values.contains(option);

                    let state = UIWidgetState {
                        checked: is_selected,
                        ..default()
                    };

                    builder
                        .spawn((
                            Name::new(format!("ListBox-Option-{}", list_box.entry)),
                            Node::default(),
                            BackgroundColor::default(),
                            ImageNode::default(),
                            BorderColor::default(),
                            ZIndex::default(),
                            state.clone(),
                            IgnoreParentState,
                            option.clone(),
                            css_source.clone(),
                            CssClass(vec![String::from("listbox-option")]),
                            RenderLayers::layer(*layer),
                            ListBoxOptionBase,
                            BindToID(id.0),
                        ))
                        .observe(on_option_click)
                        .observe(on_option_cursor_entered)
                        .observe(on_option_cursor_leave)
                        .with_children(|builder| {
                            if let Some(icon_path) = option.icon_path.as_deref() {
                                let handle = get_or_load_image(
                                    icon_path,
                                    &mut image_cache,
                                    &mut images,
                                    &asset_server,
                                );

                                builder.spawn((
                                    Name::new(format!(
                                        "ListBox-Option-Icon-{}",
                                        list_box.entry
                                    )),
                                    ImageNode {
                                        image: handle,
                                        ..default()
                                    },
                                    ZIndex::default(),
                                    state.clone(),
                                    IgnoreParentState,
                                    css_source.clone(),
                                    CssClass(vec![
                                        String::from("option-icon"),
                                        String::from("option-text"),
                                    ]),
                                    Pickable::IGNORE,
                                    RenderLayers::layer(*layer),
                                    BindToID(id.0),
                                ));
                            }

                            let text = if option.text.trim().is_empty() {
                                Text::new("(empty)")
                            } else {
                                Text::new(option.text.clone())
                            };

                            builder.spawn((
                                Name::new(format!(
                                    "ListBox-Option-Text-{}",
                                    list_box.entry
                                )),
                                text,
                                TextColor::default(),
                                TextFont::default(),
                                TextLayout::default(),
                                ZIndex::default(),
                                state.clone(),
                                IgnoreParentState,
                                css_source.clone(),
                                CssClass(vec![String::from("option-text")]),
                                Pickable::IGNORE,
                                RenderLayers::layer(*layer),
                                BindToID(id.0),
                            ));
                        });
                }
            });
    }
}

/// Enables mouse-wheel scrolling within a [`ListBox`].
fn handle_scroll_events(
    mut scroll_events: MessageReader<MouseWheel>,
    mut layout_query: Query<
        (
            Entity,
            &Visibility,
            &Children,
            &mut ScrollPosition,
            &ComputedNode,
        ),
        With<ListBoxBase>,
    >,
    option_query: Query<(&ComputedNode, &ChildOf), With<ListBoxOptionBase>>,
    time: Res<Time>,
) {
    let smooth_factor = 30.0;

    for event in scroll_events.read() {
        for (layout_entity, visibility, children, mut scroll, layout_computed) in
            layout_query.iter_mut()
        {
            let is_visible = matches!(*visibility, Visibility::Visible | Visibility::Inherited);
            if !is_visible {
                continue;
            }

            let inv_sf = layout_computed.inverse_scale_factor.max(f32::EPSILON);
            let delta = -wheel_delta_y(event, inv_sf);

            if children.is_empty() {
                scroll.y = 0.0;
                continue;
            }

            let mut option_height = None;
            for (opt_computed, parent) in option_query.iter() {
                if parent.parent() == layout_entity {
                    let opt_inv_sf = opt_computed.inverse_scale_factor.max(f32::EPSILON);
                    option_height = Some((opt_computed.size().y * opt_inv_sf).max(1.0));
                    break;
                }
            }

            let option_h = option_height.unwrap_or(40.0);
            let measured_viewport = (layout_computed.size().y * inv_sf).max(1.0);
            let content_h = children.len() as f32 * option_h;
            let max_scroll = (content_h - measured_viewport).max(0.0);

            let target = (scroll.y + delta).clamp(0.0, max_scroll);
            let smoothed = scroll.y + (target - scroll.y) * smooth_factor * time.delta_secs();
            scroll.y = smoothed.clamp(0.0, max_scroll);
        }
    }
}

// ===============================================
//                   Intern Events
// ===============================================

/// Handles click events on the [`ListBox`] background (focus tracking).
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<ListBox>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }

    trigger.propagate(false);
}

/// Sets `hovered = true` on a [`ListBox`] when the cursor enters.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<ListBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Sets `hovered = false` on a [`ListBox`] when the cursor leaves.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<ListBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}

/// Handles selection when a list box option is clicked.
///
/// In multiselect mode the option's `checked` state is toggled.
/// In single-select mode all other options are unchecked and the clicked one is checked.
fn on_option_click(
    mut trigger: On<Pointer<Click>>,
    mut option_query: Query<
        (Entity, &mut UIWidgetState, &ChoiceOption, &BindToID, &Children),
        With<ListBoxOptionBase>,
    >,
    mut parent_query: Query<(&UIGenID, &mut ListBox), Without<ListBoxOptionBase>>,
    mut inner_query: Query<&mut UIWidgetState, (Without<ListBoxOptionBase>, Without<ListBox>)>,
) {
    let clicked_entity = trigger.entity;

    let (clicked_parent_id, _clicked_option, was_checked) =
        if let Ok((_, state, option, bind_id, _)) = option_query.get(clicked_entity) {
            (bind_id.0, option.clone(), state.checked)
        } else {
            return;
        };

    // Find the parent list box to read multiselect flag.
    let multiselect = parent_query
        .iter()
        .find(|(id, _)| id.0 == clicked_parent_id)
        .map(|(_, lb)| lb.multiselect)
        .unwrap_or(false);

    // Update checked states on all sibling options.
    for (entity, mut state, _, bind_id, children) in option_query.iter_mut() {
        if bind_id.0 != clicked_parent_id {
            continue;
        }

        let new_checked = if multiselect {
            // toggle only the clicked option
            if entity == clicked_entity {
                !was_checked
            } else {
                state.checked
            }
        } else {
            // single-select: only the clicked option becomes checked
            entity == clicked_entity
        };

        state.checked = new_checked;

        for child in children.iter() {
            if let Ok(mut inner_state) = inner_query.get_mut(child) {
                inner_state.checked = new_checked;
            }
        }
    }

    // Rebuild values on the parent ListBox.
    for (id, mut list_box) in parent_query.iter_mut() {
        if id.0 != clicked_parent_id {
            continue;
        }

        list_box.values = option_query
            .iter()
            .filter(|(_, state, _, bind_id, _)| bind_id.0 == clicked_parent_id && state.checked)
            .map(|(_, _, option, _, _)| option.clone())
            .collect();
    }

    trigger.propagate(false);
}

/// Sets `hovered = true` on a list box option and its visual children.
fn on_option_cursor_entered(
    trigger: On<Pointer<Over>>,
    mut query: Query<(&mut UIWidgetState, &Children), With<ListBoxOptionBase>>,
    mut inner_query: Query<&mut UIWidgetState, Without<ListBoxOptionBase>>,
) {
    if let Ok((mut state, children)) = query.get_mut(trigger.entity) {
        state.hovered = true;

        for child in children.iter() {
            if let Ok(mut inner_state) = inner_query.get_mut(child) {
                inner_state.hovered = true;
            }
        }
    }
}

/// Sets `hovered = false` on a list box option and its visual children.
fn on_option_cursor_leave(
    trigger: On<Pointer<Out>>,
    mut query: Query<(&mut UIWidgetState, &Children), With<ListBoxOptionBase>>,
    mut inner_query: Query<&mut UIWidgetState, Without<ListBoxOptionBase>>,
) {
    if let Ok((mut state, children)) = query.get_mut(trigger.entity) {
        state.hovered = false;

        for child in children.iter() {
            if let Ok(mut inner_state) = inner_query.get_mut(child) {
                inner_state.hovered = false;
            }
        }
    }
}
