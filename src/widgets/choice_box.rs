use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, IgnoreParentState, UIGenID, UIWidgetState};
use crate::styling::paint::Colored;
use crate::widgets::{ChoiceBox};

#[derive(Component)]
struct ChoiceBase;

#[derive(Component)]
struct ChoiceOptionBase;

#[derive(Component)]
struct SelectedOptionBase;

#[derive(Component)]
struct ChoiceLayoutBoxBase;

pub struct ChoiceBoxWidget;

impl Plugin for ChoiceBoxWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_content_box_visibility,
            internal_node_creation_system,
        ).chain());
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &ChoiceBox, Option<&CssSource>), (With<ChoiceBox>, Without<ChoiceBase>)>,
    config: Res<ExtendedUiConfiguration>
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
            TagName("choice-box".to_string()),
            RenderLayers::layer(*layer),
            ChoiceBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {

                generate_child_selected_option(builder, &css_source.clone(), choice_box, layer, &id.0);
                
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
                        ))
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

fn generate_child_selected_option(builder: &mut RelatedSpawnerCommands<ChildOf>, css_source: &CssSource, choice_box: &ChoiceBox, layer: &usize, id: &usize) {
    
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
}
