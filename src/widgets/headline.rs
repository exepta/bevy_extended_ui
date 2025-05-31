use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState};
use crate::styling::convert::{CssSource, TagName};
use crate::widgets::Headline;

#[derive(Component)]
struct HeadlineBase;

pub struct HeadlineWidget;

impl Plugin for HeadlineWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Headline, Option<&CssSource>), (With<Headline>, Without<HeadlineBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, headline, source_opt) in query.iter() {
        let mut css_source = CssSource(String::from("assets/css/core.css"));
        if let Some(source) = source_opt {
            css_source = source.clone();
        }
        
        commands.entity(entity).insert((
            Name::new(format!("{}-{}", headline.h_type.to_string().to_uppercase(), headline.w_count)),
            Node::default(),
            Text::new(headline.text.clone()),
            TextColor::default(),
            TextFont::default(),
            TextLayout::default(),
            ZIndex::default(),
            css_source,
            TagName(format!("{}", headline.h_type.to_string())),
            RenderLayers::layer(*layer),
            HeadlineBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Headline>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Headline>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Headline>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}