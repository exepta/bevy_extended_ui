use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use crate::styles::{CssClass, CssSource, IconPlace, TagName};
use crate::styles::paint::Colored;
use crate::widgets::{BindToID, FieldMode, FieldSelectionMulti, FieldSet, FieldSelectionSingle, InFieldSet, ToggleButton, UIGenID, UIWidgetState, WidgetId, WidgetKind};
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
    parents: Query<&ChildOf>,
    fieldset_q: Query<(), With<FieldSet>>,
    mut single_sel_q: Query<Option<&mut FieldSelectionSingle>, With<FieldSet>>,
    mut multi_sel_q: Query<Option<&mut FieldSelectionMulti>, With<FieldSet>>,
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

        commands.entity(entity).insert(UIWidgetState {
            checked: toggle_button.selected,
            ..default()
        });

        let fs_entity_opt = find_fieldset_ancestor_optional(entity, &parents, &fieldset_q);
        if let Some(fs_ent) = fs_entity_opt {
            commands.entity(entity).insert(InFieldSet(fs_ent));
            if toggle_button.selected {
                if let Ok(Some(mut sel)) = single_sel_q.get_mut(fs_ent) {
                    if sel.0.is_none() {
                        sel.0 = Some(entity);
                    }
                }
                if let Ok(Some(mut selection_multi)) = multi_sel_q.get_mut(fs_ent) {
                    if !selection_multi.0.contains(&entity) {
                        selection_multi.0.push(entity);
                    }
                }
            }
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
                TagName("toggle".to_string()),
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
    mut toggles_q: Query<
        (
            Entity,
            &mut UIWidgetState,
            &UIGenID,
            &mut ToggleButton,
        ),
        With<ToggleButton>,
    >,
    parents: Query<&ChildOf>,
    fieldset_tag_q: Query<(), With<FieldSet>>,
    mut fs_q: Query<(
        &FieldSet,
        Option<&mut FieldSelectionSingle>,
        Option<&mut FieldSelectionMulti>,
    )>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let clicked = trigger.entity;

    let fs_entity_opt = find_fieldset_ancestor_optional(clicked, &parents, &fieldset_tag_q);

    // Standalone Toggle
    if fs_entity_opt.is_none() {
        if let Ok((_e, mut st, gen_id, mut tb)) = toggles_q.get_mut(clicked) {
            current_widget_state.widget_id = gen_id.0;
            st.checked = !st.checked;
            tb.selected = st.checked;
        }
        trigger.propagate(false);
        return;
    }

    let fs_entity = fs_entity_opt.unwrap();
    let Ok((fieldset, mut sel_single, mut sel_multi)) = fs_q.get_mut(fs_entity) else {
        trigger.propagate(false);
        return;
    };

    // Fetch clicked toggle
    let Ok((_e, mut st, gen_id, mut tb)) = toggles_q.get_mut(clicked) else {
        trigger.propagate(false);
        return;
    };
    current_widget_state.widget_id = gen_id.0;

    let mut should_check = false;
    let mut should_uncheck = false;

    match fieldset.field_mode {
        FieldMode::Single => {
            if st.checked {
                if fieldset.allow_none {
                    st.checked = false;
                    tb.selected = false;
                    should_uncheck = true;
                } // else: stay checked, no change
            } else {
                st.checked = true;
                tb.selected = true;
                should_check = true;
            }
        }
        FieldMode::Multi | FieldMode::Count(_) => {
            st.checked = !st.checked;
            tb.selected = st.checked;
            if st.checked {
                should_check = true;
            } else {
                should_uncheck = true;
            }
        }
    }

    // Update selections
    let mut sel_single_ref = sel_single.as_mut();
    let mut sel_multi_ref = sel_multi.as_mut();

    if should_check {
        if let Some(ss) = sel_single_ref.as_mut() {
            ss.0 = Some(clicked);
        }
        if let Some(sm) = sel_multi_ref.as_mut() {
            if !sm.0.contains(&clicked) {
                sm.0.push(clicked);
            }
        }
    }
    if should_uncheck {
        if let Some(ss) = sel_single_ref.as_mut() {
            if ss.0 == Some(clicked) {
                ss.0 = None;
            }
        }
        if let Some(sm) = sel_multi_ref.as_mut() {
            sm.0.retain(|&e| e != clicked);
        }
    }

    // Enforce single mode: uncheck others in the same FieldSet
    if fieldset.field_mode == FieldMode::Single && should_check {
        let toggle_entities: Vec<Entity> = toggles_q.iter().map(|(e, _, _, _)| e).collect();
        for e in toggle_entities {
            if e == clicked {
                continue;
            }
            let Some(fs_other) = find_fieldset_ancestor_optional(e, &parents, &fieldset_tag_q) else {
                continue;
            };
            if fs_other != fs_entity {
                continue;
            }
            if let Ok((_oe, mut st_o, _other_gen, mut tb_o)) = toggles_q.get_mut(e) {
                if st_o.checked {
                    st_o.checked = false;
                    tb_o.selected = false;
                }
            }
        }
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

fn find_fieldset_ancestor_optional(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    fieldsets: &Query<(), With<FieldSet>>,
) -> Option<Entity> {
    loop {
        let Ok(p) = parents.get(entity) else { return None };
        let parent = p.parent();
        if fieldsets.get(parent).is_ok() {
            return Some(parent);
        }
        entity = parent;
    }
}
