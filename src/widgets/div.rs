use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{ExtendedUiConfiguration, UIWidgetState};
use crate::styling::convert::TagName;
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::widgets::Div;

#[derive(Component)]
struct DivBase;

pub struct DivWidget;

impl Plugin for DivWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Div), (With<Div>, Without<DivBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    let css_internal = "assets/css/core.css";
    for (entity, div) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Div-{}", div.0)),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            WidgetStyle::load_from_file(css_internal),
            TagName("div".to_string()),
            RenderLayers::layer(*layer),
            DivBase
        )).observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Div>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}