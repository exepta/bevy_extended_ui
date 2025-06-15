use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{CurrentWidgetState, ExtendedUiConfiguration, UIGenID, UIWidgetState, LIB_CORE_CSS_PATH};
use crate::styling::convert::{CssSource, TagName};
use crate::widgets::Paragraph;

#[derive(Component)]
struct ParagraphBase;

pub struct ParagraphWidget;

impl Plugin for ParagraphWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &Paragraph, Option<&CssSource>), (With<Paragraph>, Without<ParagraphBase>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, paragraph, source_opt) in query.iter() {
        let mut css_source = CssSource(LIB_CORE_CSS_PATH.to_string());
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("Paragraph-{}", paragraph.w_count)),
            Node::default(),
            Text::new(paragraph.text.clone()),
            TextColor::default(),
            TextFont::default(),
            TextLayout::default(),
            ZIndex::default(),
            css_source,
            TagName("p".to_string()),
            RenderLayers::layer(*layer),
            ParagraphBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Paragraph>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Paragraph>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Paragraph>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}