use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::widgets::{ProgressBar, WidgetId, WidgetKind};

#[derive(Component)]
struct ProgressBarBase;

#[derive(Component)]
struct ProgressBarTrack;

#[derive(Component)]
struct ProgressBarNeedInit;

pub struct ProgressBarWidget;

impl Plugin for ProgressBarWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            internal_node_creation_system,
            update_progress_bars,
            initialize_progress_bar_visual_state
        ).chain());
    }
}

/// Spawns internal nodes for a `ProgressBar` UI widget.
///
/// This system detects all `ProgressBar` entities that haven't been initialized
/// yet (i.e., lack the `ProgressBarBase` component) and constructs their internal
/// node structure, including the progress track.
///
/// Inserts:
/// - `ProgressBarBase` to mark the entity as initialized
/// - `ProgressBarNeedInit` to trigger visual setup in a later system
/// - Child node with `ProgressBarTrack`, bound to the parent ID
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &ProgressBar, Option<&CssSource>), (With<ProgressBar>, Without<ProgressBarBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, progress_bar, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("ProgressBar-{}", progress_bar.w_count)),
            Node::default(),
            WidgetId {
                id: progress_bar.w_count,
                kind: WidgetKind::ProgressBar
            },
            BackgroundColor::default(),
            ImageNode::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            Pickable::default(),
            css_source.clone(),
            TagName(String::from("progressbar")),
            RenderLayers::layer(*layer),
            ProgressBarBase,
            ProgressBarNeedInit
        )) .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {
                builder.spawn((
                    Name::new(format!("ProgressBar-Fill-{}", progress_bar.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    ImageNode::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    RenderLayers::layer(*layer),
                    css_source.clone(),
                    CssClass(vec!["progress".to_string()]),
                    Pickable::IGNORE,
                    ProgressBarTrack,
                    BindToID(id.0)));
            });
    }
}

/// Updates the visual size of all progress bars during runtime.
///
/// This system runs every frame and recalculates the width of each progress track
/// based on the current value of the progress bar. It ensures the UI stays in sync
/// with any logical value updates on the `ProgressBar` component.
fn update_progress_bars(
    mut query: Query<(&ProgressBar, &ComputedNode, &Children, &UIGenID), With<ProgressBarBase>>,
    mut track_query: Query<(&mut Node, &BindToID, &mut WidgetStyle), With<ProgressBarTrack>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = match window_query.single() {
        Ok(window) => window,
        Err(_) => return,
    };

    for (progress_bar, computed_node, children, ui_id) in query.iter_mut() {
        let progress = ((progress_bar.value - progress_bar.min) / (progress_bar.max - progress_bar.min))
            .clamp(0.0, 1.0);

        let base_width = computed_node.size().x / window.scale_factor();
        let fill_width = progress * base_width;

        for child in children.iter() {
            if let Ok((mut node, bind, mut style)) = track_query.get_mut(child) {
                if bind.0 != ui_id.0 {
                    continue;
                }

                node.width = Val::Px(fill_width);

                for (_, styles) in style.styles.iter_mut() {
                    styles.width = Some(Val::Px(fill_width));
                }
            }
        }
    }
}

/// Sets the initial visual state of a progress bar after creation.
///
/// This system is triggered for any `ProgressBar` entity that has a `ProgressBarNeedInit`
/// marker component. It calculates the correct width of the progress track based on
/// the current progress value and sets the visual size accordingly.
///
/// Removes the `ProgressBarNeedInit` component once initialization is complete.
fn initialize_progress_bar_visual_state(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &ProgressBar,
        &ComputedNode,
        &Children,
        &UIGenID,
        Option<&ProgressBarNeedInit>,
    )>,
    mut track_query: Query<(&mut Node, &mut WidgetStyle, &BindToID), With<ProgressBarTrack>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else { return };

    for (entity, progress_bar, computed_node, children, ui_id, needs_init) in query.iter_mut() {
        if needs_init.is_none() {
            continue;
        }

        let total_width = computed_node.size().x / window.scale_factor();
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
                    styles.width = Some(Val::Px(fill_width));
                }
            }
        }

        commands.entity(entity).remove::<ProgressBarNeedInit>();
    }
}

/// Called when the progress bar is clicked.
///
/// This will set the `focused` state of the progress bar and update
/// the currently active widget state in the global UI state resource.
fn on_internal_click(
    trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<ProgressBar>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

/// Called when the mouse enters the progress bar.
///
/// Sets the `hovered` flag to true in the widget state.
fn on_internal_cursor_entered(
    trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<ProgressBar>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
}

/// Called when the mouse leaves the progress bar.
///
/// Resets the `hovered` flag to false in the widget state.
fn on_internal_cursor_leave(
    trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<ProgressBar>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
}
