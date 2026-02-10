use crate::ExtendedUiConfiguration;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{Divider, DividerAlignment, WidgetId, WidgetKind};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Marker component for initialized divider widgets.
#[derive(Component)]
struct DividerBase;

/// Plugin that registers divider widget behavior.
pub struct DividerWidget;

/// Tracks the previous alignment for change detection.
#[derive(Component, Deref, DerefMut)]
struct PrevDividerAlignment(DividerAlignment);

impl Plugin for DividerWidget {
    /// Registers systems for divider widget setup.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (internal_node_creation_system, update_divider_alignment).chain(),
        );
    }
}

/// Spawns the divider UI node with initial alignment classes.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Divider, Option<&CssSource>), (With<Divider>, Without<DividerBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, divider, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        let align_class = alignment_class(&divider.alignment);

        commands.entity(entity).insert((
            Name::new(format!("Divider-{}", divider.entry)),
            Node::default(),
            WidgetId {
                id: divider.entry,
                kind: WidgetKind::Divider,
            },
            ZIndex::default(),
            Pickable::IGNORE,
            BackgroundColor::default(),
            BorderColor::default(),
            css_source,
            TagName("divider".to_string()),
            CssClass(vec![align_class.to_string()]),
            RenderLayers::layer(*layer),
            DividerBase,
            PrevDividerAlignment(divider.alignment.clone()),
        ));
    }
}

/// Updates CSS classes when the divider alignment changes.
fn update_divider_alignment(
    mut q: Query<
        (&Divider, &mut CssClass, &mut PrevDividerAlignment),
        (With<DividerBase>, Changed<Divider>),
    >,
) {
    for (divider, mut classes, mut prev) in q.iter_mut() {
        if **prev == divider.alignment {
            continue;
        }

        set_alignment_class(&mut classes, &divider.alignment);
        **prev = divider.alignment.clone();

        info!("divider alignment -> {:?}", divider.alignment);
    }
}

/// Returns the CSS class name for a given alignment.
fn alignment_class(a: &DividerAlignment) -> &'static str {
    match a {
        DividerAlignment::Vertical => "divider-vert",
        DividerAlignment::Horizontal => "divider-hori",
    }
}

/// Applies the alignment class to a CSS class list.
fn set_alignment_class(classes: &mut CssClass, a: &DividerAlignment) {
    classes
        .0
        .retain(|c| c != "divider-vert" && c != "divider-hori");
    classes.0.push(alignment_class(a).to_string());
}
