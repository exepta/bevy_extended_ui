use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::ExtendedUiConfiguration;
use crate::styles::{CssSource, TagName};
use crate::styles::paint::Colored;
use crate::widgets::{FieldSelectionMulti, FieldSet, FiledSelectionSingle, WidgetId, WidgetKind};

#[derive(Component)]
struct FieldSetBase;

pub struct FieldSetWidget;

impl Plugin for FieldSetWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &FieldSet, Option<&CssSource>), (With<FieldSet>, Without<FieldSetBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, fieldset, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("FieldSet-{}", fieldset.entry)),
            Node::default(),
            WidgetId {
                id: fieldset.entry,
                kind: WidgetKind::FieldSet
            },
            ImageNode::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            Pickable::default(),
            css_source,
            TagName("fieldset".to_string()),
            RenderLayers::layer(*layer),
            FieldSetBase
        ))
            .insert(FiledSelectionSingle::default())
            .insert(FieldSelectionMulti::default());
    }
}