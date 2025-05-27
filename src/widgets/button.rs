use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, ImageCache, UIGenID, UIWidgetState};
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::styling::IconPlace;
use crate::styling::paint::Colored;
use crate::widgets::Button;

#[derive(Component)]
struct ButtonBase;

#[derive(Component)]
struct ButtonText;

#[derive(Component)]
struct ButtonImage;

pub struct ButtonWidget;

impl Plugin for ButtonWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
    }
}

fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &Button, Option<&CssSource>), (With<Button>, Without<ButtonBase>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, button, source_opt) in query.iter() {
        let mut css_source = CssSource(String::from("assets/css/core.css"));
        if let Some(source) = source_opt {
            css_source = source.clone();
        }
        
        commands.entity(entity).insert((
            Name::new(format!("Button-{}", button.w_count)),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            css_source.clone(),
            TagName("button".to_string()),
            RenderLayers::layer(*layer),
            ButtonBase
        )).with_children(|builder| {
            if button.icon_place == IconPlace::Left {
                place_icon(builder, button, &asset_server, &mut image_cache, id.0, *layer, css_source.clone());
            }
            
            builder.spawn((                    
                Name::new(format!("Button-Text-{}", button.w_count)),
                Text::new(button.text.clone()),
                TextColor::default(),
                TextFont::default(),
                TextLayout::default(),
                css_source.clone(),
                UIWidgetState::default(),
                ZIndex::default(),
                CssClass(vec!["button-text".to_string()]),
                Pickable::IGNORE,
                BindToID(id.0),
                RenderLayers::layer(*layer),
                ButtonText
            ));

            if button.icon_place == IconPlace::Right {
                place_icon(builder, button, &asset_server, &mut image_cache, id.0, *layer, css_source.clone());
            }
        }).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<Button>>,
    mut current_widget_state: ResMut<CurrentWidgetState>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.0;
    }
}

fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<Button>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<Button>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}

fn place_icon(
    builder: &mut RelatedSpawnerCommands<ChildOf>, 
    btn: &Button, 
    asset_server: &Res<AssetServer>,
    image_cache: &mut ResMut<ImageCache>,
    id: usize, 
    layer: usize,
    css_source: CssSource,
) {
    if let Some(icon) = btn.icon_path.clone() {
        let handle = image_cache.map.entry(icon.clone())
            .or_insert_with(|| asset_server.load(icon.as_str()))
            .clone();
        
        builder.spawn((
            Name::new(format!("Button-Icon-{}", btn.w_count)),
            ImageNode::new(handle),
            RenderLayers::layer(layer),
            Pickable::IGNORE,
            ButtonImage,
            UIWidgetState::default(),
            css_source.clone(),
            CssClass(vec!["button-text".to_string()]),
            BindToID(id),
            ZIndex(1)
        ));
    }
}