use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::paint::Colored;
use crate::widgets::CheckBox;

#[derive(Component)]
struct CheckBoxBase;

#[derive(Component)]
struct CheckBoxLabel;

#[derive(Component)]
pub struct CheckBoxMark;

pub struct CheckBoxWidget;

impl Plugin for CheckBoxWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &CheckBox, Option<&CssSource>), (With<CheckBox>, Without<CheckBoxBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, checkbox, source_opt) in query.iter() {
        let mut css_source = CssSource(String::from("assets/css/core.css"));
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("CheckBox-{}", checkbox.w_count)),
            Node {
                width: Val::Px(200.0),
                height: Val::Px(40.0),
                display: Display::Flex,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                ..default()
            },
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            css_source.clone(),
            TagName(String::from("input[type=\"checkbox\"]")),
            RenderLayers::layer(*layer),
            CheckBoxBase,
            children![
                (
                    Name::new(format!("Check-Mark-{}", checkbox.w_count)),
                    Node {
                      display: Display::Flex,
                      justify_content: JustifyContent::Center,
                      align_items: AlignItems::Center,
                      ..default()
                    },
                    BackgroundColor::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                    css_source.clone(),
                    UIWidgetState::default(),
                    CssClass(vec!["mark-box".to_string()]),
                    Pickable::IGNORE,
                    BindToID(id.0),
                    RenderLayers::layer(*layer),
                    CheckBoxMark,
                ),
                (
                    Name::new(format!("Check-Label-{}", checkbox.w_count)),
                    Text::new(checkbox.label.clone()),
                    TextColor::default(),
                    TextFont::default(),
                    TextLayout::default(),
                    css_source.clone(),
                    UIWidgetState::default(),
                    CssClass(vec!["check-text".to_string()]),
                    Pickable::IGNORE,
                    BindToID(id.0),
                    RenderLayers::layer(*layer),
                    CheckBoxLabel
                )
            ]
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut query: Query<(&mut UIWidgetState, &UIGenID, &CheckBox, &CssSource), With<CheckBox>>,
    inner_query: Query<(Entity, &BindToID, Option<&Children>, &ComputedNode), With<CheckBoxMark>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    if let Ok((mut state, gen_id, checkbox, css_source)) = query.get_mut(trigger.target) {
        state.focused = true;
        state.checked = !state.checked;
        current_widget_state.widget_id = gen_id.0;

        for (entity, id, children_opt, computed_node) in inner_query.iter() {
            if gen_id.0 != id.0 {
                continue;
            }
            
            let width = computed_node.size.x / 1.5;
            let height = computed_node.size.y / 1.5;

            if state.checked {
                let mut child = None;
                commands.entity(entity).with_children(|builder| {
                    let in_child = builder.spawn((
                        Name::new(format!("Mark-{}", checkbox.w_count)),
                        Node {
                            width: Val::Px(width),
                            height: Val::Px(height),
                            ..default()
                        },
                        Pickable::IGNORE,
                        css_source.clone(),
                        UIWidgetState::default(),
                        CssClass(vec!["mark".to_string()]),
                        RenderLayers::layer(*layer),
                    )).id();
                    child = Some(in_child);
                });

                if let Some(child) = child {
                    if let Some(path) = checkbox.icon_path.clone() {
                        commands.entity(child).insert(ImageNode::new(asset_server.load(path)));
                    }
                }
            } else {
                if let Some(children) = children_opt {
                    for child in children.iter() {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<CheckBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<CheckBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}