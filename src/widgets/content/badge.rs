use crate::ExtendedUiConfiguration;
use crate::styles::{CssClass, CssID, CssSource, TagName};
use crate::widgets::{
    Badge, BadgeAnchor, BindToID, Body, UIGenID, UIWidgetState, WidgetId, WidgetKind,
};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform, UiScale};
use std::collections::HashMap;

/// Marker component for initialized badge widgets.
#[derive(Component)]
struct BadgeBase;

/// Marker component for badge text nodes.
#[derive(Component)]
struct BadgeText;

/// Runtime state for badge target tracking.
#[derive(Component, Default)]
struct BadgeRuntime {
    target: Option<Entity>,
}

/// Base class list for badge root (user classes + internal classes).
#[derive(Component, Clone)]
struct BadgeClassBase(Vec<String>);

/// Plugin that wires badge widget behavior.
pub struct BadgeWidget;

impl Plugin for BadgeWidget {
    /// Registers systems for badge setup and runtime updates.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                resolve_badge_targets,
                update_badge_text,
            )
                .chain(),
        );
        app.add_systems(PostUpdate, update_badge_visuals);
    }
}

/// Initializes internal nodes for badges.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &UIGenID,
            &Badge,
            Option<&CssSource>,
            Option<&CssClass>,
        ),
        (With<Badge>, Without<BadgeBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().copied().unwrap_or(1);

    for (entity, ui_id, badge, source_opt, class_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        let mut root_classes = class_opt.map(|class| class.0.clone()).unwrap_or_default();
        if !root_classes.iter().any(|class| class == "badge") {
            root_classes.push("badge".to_string());
        }

        let mut node = Node::default();
        node.position_type = PositionType::Absolute;
        node.left = Val::Px(0.0);
        node.top = Val::Px(0.0);

        commands
            .entity(entity)
            .insert((
                Name::new(format!("Badge-{}", badge.entry)),
                node,
                WidgetId {
                    id: badge.entry,
                    kind: WidgetKind::Badge,
                },
                BackgroundColor::default(),
                BorderColor::default(),
                ZIndex(20_000),
                Pickable::IGNORE,
                UiTransform::default(),
                css_source.clone(),
                TagName("badge".to_string()),
                CssClass(root_classes.clone()),
                BadgeClassBase(root_classes),
                RenderLayers::layer(layer),
                BadgeBase,
                BadgeRuntime::default(),
            ))
            .with_children(|builder| {
                builder.spawn((
                    Name::new(format!("Badge-Text-{}", badge.entry)),
                    Text::new(format_badge_label(badge.value, badge.max)),
                    TextColor::default(),
                    TextFont::default(),
                    TextLayout::new_with_justify(Justify::Center).with_no_wrap(),
                    UiTransform::default(),
                    Pickable::IGNORE,
                    css_source.clone(),
                    CssClass(vec!["badge-text".to_string()]),
                    BindToID(ui_id.get()),
                    UIWidgetState::default(),
                    RenderLayers::layer(layer),
                    BadgeText,
                ));
            });
    }
}

/// Resolves badge targets from either `for="id"` or the direct parent widget.
fn resolve_badge_targets(
    mut badge_q: Query<
        (&Badge, Option<&ChildOf>, &mut BadgeRuntime),
        (With<Badge>, With<BadgeBase>),
    >,
    parents_q: Query<&ChildOf>,
    id_q: Query<(Entity, &CssID), Without<Badge>>,
    widget_q: Query<(), With<WidgetId>>,
    body_q: Query<(), With<Body>>,
) {
    let mut by_id: HashMap<&str, Entity> = HashMap::new();
    for (entity, css_id) in id_q.iter() {
        if !css_id.0.is_empty() {
            by_id.entry(css_id.0.as_str()).or_insert(entity);
        }
    }

    for (badge, parent_opt, mut runtime) in badge_q.iter_mut() {
        let target = if let Some(for_id) = badge.for_id.as_deref() {
            let normalized = for_id.trim().trim_start_matches('#');
            by_id.get(normalized).copied()
        } else if let Some(parent) = parent_opt {
            let mut current = parent.parent();
            let mut resolved = None;

            loop {
                if body_q.get(current).is_ok() {
                    break;
                }

                if widget_q.get(current).is_ok() {
                    resolved = Some(current);
                    break;
                }

                if let Ok(next) = parents_q.get(current) {
                    current = next.parent();
                } else {
                    break;
                }
            }

            resolved
        } else {
            None
        };

        runtime.target = target;
    }
}

/// Updates badge text content when the component changes.
fn update_badge_text(
    badge_q: Query<(&Badge, &Children), (With<Badge>, With<BadgeBase>, Changed<Badge>)>,
    mut text_q: Query<&mut Text, With<BadgeText>>,
) {
    for (badge, children) in badge_q.iter() {
        let label = format_badge_label(badge.value, badge.max);
        for child in children.iter() {
            if let Ok(mut text) = text_q.get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

fn format_badge_label(value: u32, max: u32) -> String {
    if value > max {
        format!("+{}", max)
    } else {
        value.to_string()
    }
}

fn anchor_to_class(anchor: BadgeAnchor) -> &'static str {
    match anchor {
        BadgeAnchor::TopLeft => "badge-anchor-top-left",
        BadgeAnchor::TopRight => "badge-anchor-top-right",
        BadgeAnchor::BottomLeft => "badge-anchor-bottom-left",
        BadgeAnchor::BottomRight => "badge-anchor-bottom-right",
    }
}

fn set_css_classes(classes: &mut CssClass, base: &[String], dynamic: &[&str]) {
    let mut next = base.to_vec();
    for class in dynamic {
        if !next.iter().any(|existing| existing == class) {
            next.push((*class).to_string());
        }
    }

    if classes.0 != next {
        classes.0 = next;
    }
}

/// Positions badges and updates state classes.
fn update_badge_visuals(
    ui_scale: Res<UiScale>,
    target_geometry_q: Query<(&ComputedNode, &UiGlobalTransform)>,
    parent_q: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut badge_q: Query<
        (
            &Badge,
            &mut Node,
            &mut Visibility,
            &mut CssClass,
            &BadgeClassBase,
            Option<&ChildOf>,
            &BadgeRuntime,
            Option<&ComputedNode>,
        ),
        (With<Badge>, With<BadgeBase>, Without<BadgeText>),
    >,
) {
    let sf = ui_scale.0.max(f32::EPSILON);

    for (
        badge,
        mut node,
        mut visibility,
        mut css_class,
        class_base,
        parent_opt,
        runtime,
        computed,
    ) in badge_q.iter_mut()
    {
        let Some(target) = runtime.target else {
            *visibility = Visibility::Hidden;
            set_css_classes(&mut css_class, &class_base.0, &["badge-closed"]);
            continue;
        };

        let Ok((target_node, target_transform)) = target_geometry_q.get(target) else {
            *visibility = Visibility::Hidden;
            set_css_classes(&mut css_class, &class_base.0, &["badge-closed"]);
            continue;
        };

        *visibility = Visibility::Inherited;

        let mut parent_top_left = Vec2::ZERO;
        if let Some(parent) = parent_opt {
            if let Ok((parent_node, parent_transform)) = parent_q.get(parent.parent()) {
                let half = parent_node.size() * 0.5;
                parent_top_left = parent_transform.affine().transform_point2(-half) / sf;
            }
        }

        let target_size = target_node.size() / sf;
        let target_top_left = target_transform
            .affine()
            .transform_point2(-(target_node.size() * 0.5))
            / sf;

        let (badge_w, badge_h) = if let Some(computed) = computed {
            (
                (computed.size().x / sf).max(10.0),
                (computed.size().y / sf).max(10.0),
            )
        } else {
            (18.0, 18.0)
        };

        let anchor_point = match badge.anchor {
            BadgeAnchor::TopLeft => target_top_left,
            BadgeAnchor::TopRight => target_top_left + Vec2::new(target_size.x, 0.0),
            BadgeAnchor::BottomLeft => target_top_left + Vec2::new(0.0, target_size.y),
            BadgeAnchor::BottomRight => target_top_left + Vec2::new(target_size.x, target_size.y),
        };

        let x = anchor_point.x - badge_w * 0.5;
        let y = anchor_point.y - badge_h * 0.5;

        node.position_type = PositionType::Absolute;
        node.left = Val::Px(x - parent_top_left.x);
        node.top = Val::Px(y - parent_top_left.y);

        set_css_classes(
            &mut css_class,
            &class_base.0,
            &[
                "badge-open",
                anchor_to_class(badge.anchor),
                if badge.value > badge.max {
                    "badge-overflow"
                } else {
                    "badge-normal"
                },
            ],
        );
    }
}
