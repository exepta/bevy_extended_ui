use crate::ExtendedUiConfiguration;
use crate::styles::paint::Colored;
use crate::styles::{CssSource, TagName};
use crate::widgets::{ToggleButton, FieldKind, FieldSelectionMulti, FieldSet, FieldSelectionSingle, RadioButton, WidgetId, WidgetKind};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Marker component for initialized field set widgets.
#[derive(Component)]
struct FieldSetBase;

/// Tracks whether field set warnings were already logged.
#[derive(Component, Default)]
struct FieldSetWarned {
    mixed: bool,
    unsupported: bool,
}

/// Plugin that registers field set widget behavior.
pub struct FieldSetWidget;

impl Plugin for FieldSetWidget {
    /// Registers systems for field set setup and validation.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
        app.add_systems(Update, detect_fieldset_kind_system);
    }
}

/// Initializes UI nodes for field set widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &FieldSet, Option<&CssSource>), (With<FieldSet>, Without<FieldSetBase>)>,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, fieldset, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands
            .entity(entity)
            .insert((
                Name::new(format!("FieldSet-{}", fieldset.entry)),
                Node::default(),
                WidgetId {
                    id: fieldset.entry,
                    kind: WidgetKind::FieldSet,
                },
                ImageNode::default(),
                BackgroundColor::default(),
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
                css_source,
                TagName("fieldset".to_string()),
                RenderLayers::layer(*layer),
                FieldSetBase,
                FieldSetWarned::default(),
            ))
            .insert(FieldSelectionSingle::default())
            .insert(FieldSelectionMulti::default());
    }
}

/// Detects field set child types and updates selection mode.
fn detect_fieldset_kind_system(
    mut commands: Commands,
    mut fieldsets: Query<(Entity, &mut FieldSet, Option<&Children>, &mut FieldSetWarned)>,
    radio_q: Query<(), With<RadioButton>>,
    toggle_q: Query<(), With<ToggleButton>>,
) {
    for (fs_entity, mut fs, direct_children, mut warned) in fieldsets.iter_mut() {
        let mut radios = 0;
        let mut toggles = 0;
        let mut mixed_found = false;
        let mut unsupported_found = false;

        let mut to_hide: Vec<Entity> = Vec::new();

        if let Some(children) = direct_children {
            for child in children.iter() {
                let is_radio = radio_q.get(child).is_ok();
                let is_toggle = toggle_q.get(child).is_ok();

                if is_radio || is_toggle {
                    if is_radio {
                        radios += 1;
                    }
                    if is_toggle {
                        toggles += 1;
                    }
                } else {
                    unsupported_found = true;
                    to_hide.push(child);
                }
            }
        }

        if radios > 0 && toggles > 0 {
            mixed_found = true;
            // Hide all direct radio/toggle children in mixed case
            if let Some(children) = direct_children {
                for child in children.iter() {
                    if radio_q.get(child).is_ok() || toggle_q.get(child).is_ok() {
                        to_hide.push(child);
                    }
                }
            }
        }

        for e in to_hide {
            commands.entity(e).insert(Visibility::Hidden);
        }

        match (radios > 0, toggles > 0, mixed_found, unsupported_found) {
            (true, false, false, _) => fs.kind = Some(FieldKind::Radio),
            (false, true, false, _) => fs.kind = Some(FieldKind::Toggle),
            (_, _, true, _) => {
                if !warned.mixed {
                    warn!(
                        "FieldSet {:?} has mixed supported children (Radio/Toggle); they were hidden.",
                        fs_entity
                    );
                    warned.mixed = true;
                }
            }
            (false, false, false, true) => {
                if !warned.unsupported {
                    warn!(
                        "FieldSet {:?} has unsupported direct children; they were hidden.",
                        fs_entity
                    );
                    warned.unsupported = true;
                }
            }
            _ => {}
        }
    }
}
