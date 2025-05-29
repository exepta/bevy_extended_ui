use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, IgnoreParentState, ImageCache, UIGenID, UIWidgetState};
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::widgets::{ChoiceBox, ChoiceOption};

#[derive(Component)]
struct ChoiceBase;

#[derive(Component)]
struct ChoiceOptionBase;

#[derive(Component)]
struct SelectedOptionBase;

#[derive(Component)]
struct DropBase;

#[derive(Component)]
struct ChoiceLayoutBoxBase;

pub struct ChoiceBoxWidget;

impl Plugin for ChoiceBoxWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_content_box_visibility,
            internal_node_creation_system,
            handle_scroll_events
        ).chain());
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &ChoiceBox, Option<&CssSource>), (With<ChoiceBox>, Without<ChoiceBase>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, choice_box, source_opt) in query.iter() {
        let mut css_source = CssSource(String::from("assets/css/core.css"));
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("Choice-Box-{}", choice_box.w_count)),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            css_source.clone(),
            TagName("select".to_string()),
            RenderLayers::layer(*layer),
            ChoiceBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {

                generate_child_selected_option(builder, &css_source.clone(), choice_box, layer, &id.0, &mut image_cache, &asset_server);
                
                builder.spawn((
                    Name::new(format!("Choice-Content-{}", choice_box.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    css_source.clone(),
                    CssClass(vec![String::from("choice-content-box")]),
                    RenderLayers::layer(*layer),
                    Visibility::Hidden,
                    ChoiceLayoutBoxBase,
                    BindToID(id.0)
                )).with_children(|builder| {
                    for option in choice_box.options.iter() {
                        builder.spawn((
                            Name::new(format!("Option-{}", choice_box.w_count)),
                            Node::default(),
                            BackgroundColor::default(),
                            BorderColor::default(),
                            BorderRadius::default(),
                            ZIndex::default(),
                            UIWidgetState::default(),
                            IgnoreParentState,
                            option.clone(),
                            css_source.clone(),
                            CssClass(vec![String::from("choice-option")]),
                            RenderLayers::layer(*layer),
                            ChoiceOptionBase,
                            BindToID(id.0)
                        )).observe(on_internal_option_click)
                            .observe(on_internal_option_cursor_entered)
                            .observe(on_internal_option_cursor_leave)
                            .with_children(|builder| {
                            builder.spawn((
                                Name::new(format!("Option-Text-{}", choice_box.w_count)),
                                Text::new(option.text.clone()),
                                TextColor::default(),
                                TextFont::default(),
                                TextLayout::default(),
                                ZIndex::default(),
                                UIWidgetState::default(),
                                IgnoreParentState,
                                css_source.clone(),
                                CssClass(vec![String::from("option-text")]),
                                Pickable::IGNORE,
                                RenderLayers::layer(*layer),
                                BindToID(id.0)
                            ));
                        });
                    }
                });

        });
    }
}

// ===============================================
//             Intern Functions
// ===============================================

fn update_content_box_visibility(
    mut query: Query<(&mut UIWidgetState, &UIGenID), (With<ChoiceBox>, Changed<UIWidgetState>)>,
    mut content_query: Query<(&mut Visibility, &BindToID), With<ChoiceLayoutBoxBase>>,
) {
    for (mut state, gen_id) in query.iter_mut() {
        for (mut visibility, bind_to_id) in content_query.iter_mut() {
            if bind_to_id.0 != gen_id.0 {
                continue;
            }

            if !state.focused {
                state.open = false;
            }

            if state.open {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn handle_scroll_events(
    mut scroll_events: EventReader<MouseWheel>,
    mut layout_query: Query<(Entity, &Visibility, &Children), With<ChoiceLayoutBoxBase>>,
    mut option_query: Query<(&mut Node, &mut WidgetStyle, &ChildOf), With<ChoiceOptionBase>>,
    time: Res<Time>,
) {
    let mut max_scroll = -0.0;
    let min_scroll = 0.0;

    let smooth_factor = 30.;

    for event in scroll_events.read() {
        for (layout_entity, visibility, children) in layout_query.iter_mut() {
            if *visibility != Visibility::Visible {
                continue;
            }

            if children.len() > 3 {
                max_scroll = -50.0 * (children.len() - 3) as f32;
            }

            let scroll_amount = match event.unit {
                MouseScrollUnit::Line => event.y * 25.0,
                MouseScrollUnit::Pixel => event.y,
            };

            let inverted_scroll_amount = scroll_amount;

            for (mut style, mut widget_style, parent) in option_query.iter_mut() {
                if parent.parent() != layout_entity {
                    continue;
                }

                let current_offset = match style.top {
                    Val::Px(val) => val,
                    _ => 0.0,
                };

                let target_offset = (current_offset + inverted_scroll_amount)
                    .clamp(max_scroll, min_scroll);

                let smoothed_offset = current_offset + (target_offset - current_offset) * smooth_factor * time.delta_secs();

                style.top = Val::Px(smoothed_offset);
                for (_, styles) in widget_style.styles.iter_mut() {
                    styles.top = Some(Val::Px(smoothed_offset));
                }
            }
        }
    }
}

// ===============================================
//                   Intern Events
// ===============================================

// Main Component

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<ChoiceBox>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        state.open = !state.open;
        current_widget_state.widget_id = gen_id.0;
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<ChoiceBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<ChoiceBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}

// Option Component

fn on_internal_option_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(Entity, &mut UIWidgetState, &ChoiceOption, &BindToID), (With<ChoiceOptionBase>, Without<ChoiceBox>)>,
    mut parent_query: Query<(Entity, &mut UIWidgetState, &UIGenID, &mut ChoiceBox), (With<ChoiceBox>, Without<ChoiceOptionBase>)>,
    mut selected_query: Query<(&BindToID, &Children), With<SelectedOptionBase>>,
    mut text_query: Query<&mut Text>,
) {
    let clicked_entity = trigger.target;

    let (clicked_parent_id, clicked_option) = if let Ok((_, _, option, bind_id)) = query.get_mut(clicked_entity) {
        (bind_id.0.clone(), option.clone())
    } else {
        return;
    };

    for (entity, mut state, _, bind_id) in query.iter_mut() {
        if bind_id.0 == clicked_parent_id {
            state.checked = entity == clicked_entity;
        }
    }

    for (_, mut parent_state, id, mut choice_box) in parent_query.iter_mut() {
        if id.0 == clicked_parent_id {
            choice_box.value = clicked_option.clone();
            parent_state.open = false;
            
            for (bind_id, selected_children) in selected_query.iter_mut() {
                if bind_id.0 == clicked_parent_id {
                    for child in selected_children.iter() {
                        if let Ok(mut text) = text_query.get_mut(child) {
                            text.0 = clicked_option.text.clone();
                        }
                    }
                }
            }
        }
    }
}

fn on_internal_option_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<(&mut UIWidgetState, &Children), With<ChoiceOptionBase>>,
    mut inner_query: Query<&mut UIWidgetState, Without<ChoiceOptionBase>>,
) {
    if let Ok((mut state, children)) = query.get_mut(trigger.target) {
        state.hovered = true;
        
        for child in children.iter() {
            if let Ok(mut inner_state) = inner_query.get_mut(child) {
                inner_state.hovered = true;
            }
        }
    }
}

fn on_internal_option_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<(&mut UIWidgetState, &Children), With<ChoiceOptionBase>>,
    mut inner_query: Query<&mut UIWidgetState, Without<ChoiceOptionBase>>,
) {
    if let Ok((mut state, children)) = query.get_mut(trigger.target) {
        state.hovered = false;

        for child in children.iter() {
            if let Ok(mut inner_state) = inner_query.get_mut(child) {
                inner_state.hovered = false;
            }
        }
    }
}

// ===============================================
//                   Child Builder
// ===============================================

fn generate_child_selected_option(
    builder: &mut RelatedSpawnerCommands<ChildOf>, 
    css_source: &CssSource, 
    choice_box: &ChoiceBox, 
    layer: &usize, id: &usize,
    image_cache: &mut ImageCache,
    asset_server: &Res<AssetServer>,
) {
    
    // Selected Container
    builder.spawn((
        Name::new(format!("Option-Selected-{}", choice_box.w_count)),
        Node::default(),
        BackgroundColor::default(),
        BorderColor::default(),
        BorderRadius::default(),
        UIWidgetState::default(),
        css_source.clone(),
        CssClass(vec![String::from("option-selected")]),
        RenderLayers::layer(*layer),
        Pickable::IGNORE,
        BindToID(*id),
        SelectedOptionBase
    )).with_children(|builder| {
        
        // Selected Text
        builder.spawn((
            Name::new(format!("Option-Sel-Text-{}", choice_box.w_count)),
            Text::new(choice_box.value.text.clone()),
            TextColor::default(),
            TextFont::default(),
            TextLayout::default(),
            ZIndex::default(),
            UIWidgetState::default(),
            IgnoreParentState,
            css_source.clone(),
            CssClass(vec![String::from("option-sel-text")]),
            Pickable::IGNORE,
            RenderLayers::layer(*layer),
            BindToID(*id)
        ));
    });
    
    builder.spawn((
        Name::new(format!("Arrow-Box-{}", choice_box.w_count)),
        Node::default(),
        BackgroundColor::default(),
        BorderColor::default(),
        BorderRadius::default(),
        UIWidgetState::default(),
        css_source.clone(),
        CssClass(vec![String::from("option-drop-box")]),
        RenderLayers::layer(*layer),
        Pickable::IGNORE,
        BindToID(*id),
        DropBase
    )).with_children(|builder| {
        if let Some(drop_icon) = choice_box.icon_path.clone() {
            let handle = image_cache.map.entry(drop_icon.clone())
                .or_insert_with(|| asset_server.load(drop_icon.as_str()))
                .clone();
            
            builder.spawn((
                Name::new(format!("Drop-Icon-{}", choice_box.w_count)),
                ImageNode {
                    image: handle,
                    ..default()
                },
                ZIndex::default(),
                UIWidgetState::default(),
                css_source.clone(),
                CssClass(vec![String::from("option-drop-icon")]),
                Pickable::IGNORE,
                RenderLayers::layer(*layer),
                BindToID(*id)
            ));
        }
    });
}
