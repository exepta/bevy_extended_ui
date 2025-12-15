use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::ExtendedUiConfiguration;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{Divider, DividerAlignment, WidgetId, WidgetKind};

#[derive(Component)]
struct DividerBase;

pub struct DividerWidget;

#[derive(Component, Deref, DerefMut)]
struct PrevDividerAlignment(DividerAlignment);

impl Plugin for DividerWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                update_divider_alignment,
            )
                .chain(),
        );
    }
}
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Divider, Option<&CssSource>), (With<Divider>, Without<DividerBase>)>,
    config: Res<ExtendedUiConfiguration>
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
                kind: WidgetKind::Divider
            },
            ZIndex::default(),
            Pickable::IGNORE,
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            css_source,
            TagName("divider".to_string()),
            CssClass(vec![align_class.to_string()]),
            RenderLayers::layer(*layer),
            DividerBase,
            PrevDividerAlignment(divider.alignment.clone())
        ));
    }
}

fn update_divider_alignment(
    mut q: Query<(&Divider, &mut CssClass, &mut PrevDividerAlignment), (With<DividerBase>, Changed<Divider>)>,
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

fn alignment_class(a: &DividerAlignment) -> &'static str {
    match a {
        DividerAlignment::Vertical => "divider-vert",
        DividerAlignment::Horizontal => "divider-hori",
    }
}

fn set_alignment_class(classes: &mut CssClass, a: &DividerAlignment) {
    classes.0.retain(|c| c != "divider-vert" && c != "divider-hori");
    classes.0.push(alignment_class(a).to_string());
}