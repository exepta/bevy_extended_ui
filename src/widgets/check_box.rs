use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::global::{BindToID, UiGenID};
use crate::resources::ExtendedUiConfiguration;
use crate::styles::{BaseStyle, InternalStyle, Style};
use crate::styles::css_types::Background;
use crate::utils::Radius;
use crate::widgets::{CheckBox};

#[derive(Component)]
struct CheckBoxRoot;

#[derive(Component)]
struct CheckBoxLabel;

#[derive(Component)]
struct CheckBoxMark;

pub struct CheckBoxWidget;

impl Plugin for CheckBoxWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_generate_component_system);
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &CheckBox, Option<&BaseStyle>), (Without<CheckBoxRoot>, With<CheckBox>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, gen_id, checkbox, option_base_style) in query.iter() {
        let default_style = default_style(option_base_style);
        commands.entity(entity).insert((
            Name::new(format!("CheckBox-{}", gen_id.0)),
            Node::default(),
            default_style.clone(),
            RenderLayers::layer(*layer),
            CheckBoxRoot
        )).with_children(|builder| {
            builder.spawn((
                Name::new(format!("Check-Mark-{}", gen_id.0)),
                Node {
                    width: Val::Px(default_style.0.check_size),
                    height: Val::Px(default_style.0.check_size),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: default_style.0.check_border,
                    ..default()
                },
                RenderLayers::layer(*layer),
                BorderRadius {
                    top_left: default_style.0.check_border_radius.top_left,
                    top_right: default_style.0.check_border_radius.top_right,
                    bottom_left: default_style.0.check_border_radius.bottom_left,
                    bottom_right: default_style.0.check_border_radius.bottom_right,
                },
                BorderColor(default_style.0.check_border_color),
                BackgroundColor(if checkbox.checked { default_style.0.check_background_color } else { Color::srgba(0.0, 0.0, 0.0, 0.0) }),
                CheckBoxMark,
                BindToID(gen_id.0)
            )).with_children(|builder| {
                builder.spawn((
                    Name::new(format!("Mark-{}", gen_id.0)),
                ));
            });

            builder.spawn((
                Name::new(format!("Check-Label-{}", gen_id.0)),
                Text::new(checkbox.label.clone()),
                TextFont {
                    font_size: default_style.0.font_size,
                    ..default()
                },
                PickingBehavior::IGNORE,
                TextColor(default_style.0.color),
                RenderLayers::layer(*layer),
                CheckBoxLabel,
                BindToID(gen_id.0)
            ));
        });
    }
}

/*fn on_internal_click(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(&mut CheckBox, &Children), With<CheckBox>>,
) {
    
}*/

fn default_style(overwrite: Option<&BaseStyle>) -> InternalStyle {
    let mut internal_style = InternalStyle(Style {
        width: Val::Px(200.),
        min_width: Val::Px(100.),
        height: Val::Px(50.),
        display: Display::Flex,
        justify_content: JustifyContent::FlexStart,
        gap_row: Val::Px(20.),
        align_items: AlignItems::Center,
        background: Background { color: Color::srgba(1.0, 1.0, 1.0, 1.0), ..default() },
        border: UiRect::all(Val::Px(0.)),
        border_radius: Radius::all(Val::Px(0.)),
        ..default()
    });

    if let Some(style) = overwrite {
        internal_style.merge_styles(&style.0);
    }
    internal_style
}

