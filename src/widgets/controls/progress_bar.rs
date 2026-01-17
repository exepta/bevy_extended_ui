use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::ExtendedUiConfiguration;
use crate::styles::{CssClass, CssSource, TagName};
use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::widgets::{BindToID, ProgressBar, UIGenID, UIWidgetState, WidgetId, WidgetKind};

#[derive(Component)]
struct ProgressBarBase;

#[derive(Component)]
struct ProgressBarTrack;

#[derive(Component)]
struct ProgressBarNeedInit;

pub struct ProgressBarWidget;

impl Plugin for ProgressBarWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (internal_node_creation_system, update_progress_bars, initialize_progress_bar_visual_state)
                .chain(),
        );
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &ProgressBar, Option<&CssSource>), (With<ProgressBar>, Without<ProgressBarBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().copied().unwrap_or(1);
    for (entity, id, progress_bar, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands
            .entity(entity)
            .insert((
                Name::new(format!("ProgressBar-{}", progress_bar.entry)),
                Node::default(),
                WidgetId {
                    id: progress_bar.entry,
                    kind: WidgetKind::ProgressBar,
                },
                BackgroundColor::default(),
                ImageNode::default(),
                BorderColor::default(),
                BorderRadius::default(),
                BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                ZIndex::default(),
                Pickable::IGNORE,
                css_source.clone(),
                TagName(String::from("progressbar")),
                RenderLayers::layer(layer),
                ProgressBarBase,
                ProgressBarNeedInit,
            ))
            .insert(Visibility::Inherited)
            .with_children(|builder| {
                builder.spawn((
                    Name::new(format!("ProgressBar-Fill-{}", progress_bar.entry)),
                    Node::default(),
                    BackgroundColor::default(),
                    ImageNode::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    RenderLayers::layer(layer),
                    css_source.clone(),
                    CssClass(vec!["progress".to_string()]),
                    Pickable::IGNORE,
                    ProgressBarTrack,
                    BindToID(id.0),
                ));
            });
    }
}

fn update_progress_bars(
    ui_scale: Res<UiScale>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&ProgressBar, &ComputedNode, &Children, &UIGenID), With<ProgressBarBase>>,
    mut track_query: Query<(&mut Node, &BindToID, &mut UiStyle), With<ProgressBarTrack>>,
) {
    let Ok(window) = window_query.single() else { return };
    let sf = window.scale_factor() * ui_scale.0;

    for (progress_bar, computed_node, children, ui_id) in query.iter_mut() {
        let progress = ((progress_bar.value - progress_bar.min) / (progress_bar.max - progress_bar.min)).clamp(0.0, 1.0);

        let base_width = computed_node.size().x / sf;
        let fill_width = progress * base_width;

        for child in children.iter() {
            if let Ok((mut node, bind, mut style)) = track_query.get_mut(child) {
                if bind.0 != ui_id.0 {
                    continue;
                }
                node.width = Val::Px(fill_width);
                for (_, styles) in style.styles.iter_mut() {
                    styles.normal.width = Some(Val::Px(fill_width));
                }
            }
        }
    }
}

fn initialize_progress_bar_visual_state(
    mut commands: Commands,
    ui_scale: Res<UiScale>,
    mut query: Query<(Entity, &ProgressBar, &ComputedNode, &Children, &UIGenID, Option<&ProgressBarNeedInit>)>,
    mut track_query: Query<(&mut Node, &mut UiStyle, &BindToID), With<ProgressBarTrack>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else { return };
    let sf = window.scale_factor() * ui_scale.0;

    for (entity, progress_bar, computed_node, children, ui_id, needs_init) in query.iter_mut() {
        if needs_init.is_none() {
            continue;
        }

        let total_width = computed_node.size().x / sf;
        if total_width <= 1.0 {
            continue;
        }

        let progress = (progress_bar.value - progress_bar.min) / (progress_bar.max - progress_bar.min);
        let fill_width = progress.clamp(0.0, 1.0) * total_width;

        for child in children.iter() {
            if let Ok((mut node, mut style, bind_to)) = track_query.get_mut(child) {
                if bind_to.0 != ui_id.0 {
                    continue;
                }
                node.width = Val::Px(fill_width);
                for (_, styles) in style.styles.iter_mut() {
                    styles.normal.width = Some(Val::Px(fill_width));
                }
            }
        }

        commands.entity(entity).remove::<ProgressBarNeedInit>();
    }
}