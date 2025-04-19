use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use crate::global::{UiGenID, UiElementState, BindToID};
use crate::styles::{BaseStyle, HoverStyle, SelectedStyle, InternalStyle, Style};
use crate::resources::{CurrentElementSelected, ExtendedUiConfiguration};
use crate::styles::css_types::Background;
use crate::utils::Radius;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, BaseStyle, HoverStyle, SelectedStyle, InternalStyle)]
pub struct Slider {
    pub value: i32,
    pub step: i32,
    pub min: i32,
    pub max: i32,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: 0,
            max: 100,
        }
    }
}

#[derive(Component)]
struct SliderRoot;

#[derive(Component)]
struct SliderTrack;

#[derive(Component, Reflect, Debug, Clone)]
struct SliderThumb {
    current_x: f32,
}

pub struct SliderWidget;

impl Plugin for SliderWidget {
    fn build(&self, app: &mut App) {
        app.register_type::<Slider>();
        app.register_type::<SliderThumb>();
        app.add_systems(Update, (
            internal_generate_component_system,
            detect_change_slider_values
        ));
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &InternalStyle, Option<&BaseStyle>), (Without<SliderRoot>, With<Slider>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, gen_id, style, option_base_style) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("Slider-{}", gen_id.0)),
            Node::default(),
            default_style(option_base_style),
            BoxShadow {
                color: Color::BLACK,
                spread_radius: Val::Px(3.),
                blur_radius: Val::Px(3.),
                y_offset: Val::Px(0.),
                x_offset: Val::Px(0.),
                ..default()
            },
            RenderLayers::layer(*layer),
            SliderRoot
        ))
            .observe(on_click_track)
            .observe(on_internal_mouse_click)
            .with_children(|builder| {

            // Slider Track
            builder.spawn((
                Name::new(format!("Slider-Track-{}", gen_id.0)),
                Node {
                    width: Val::Px(0.),
                    height: Val::Percent(100.),
                    ..default()
                },
                BorderRadius {
                    top_left: style.0.border_radius.top_left,
                    top_right: style.0.border_radius.top_right,
                    bottom_left: style.0.border_radius.bottom_left,
                    bottom_right: style.0.border_radius.bottom_right,
                },
                BackgroundColor(style.0.track_color),
                RenderLayers::layer(*layer),
                PickingBehavior::IGNORE,
                SliderTrack,
                BindToID(gen_id.0)
            ));

            // Slider Thumb
            builder.spawn((
                Name::new(format!("Slider-Thumb-{}", gen_id.0)),
                Node {
                    width: style.0.thumb_width,
                    height: style.0.thumb_height,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BorderRadius {
                    top_left: style.0.thumb_border_radius.top_left,
                    top_right: style.0.thumb_border_radius.top_right,
                    bottom_left: style.0.thumb_border_radius.bottom_left,
                    bottom_right: style.0.thumb_border_radius.bottom_right,
                },
                BackgroundColor(style.0.thumb_color),
                style.0.thumb_box_shadow,
                RenderLayers::layer(*layer),
                SliderThumb { current_x: 0. },
                BindToID(gen_id.0)
            )).observe(on_move_thumb);

        });
    }
}

fn detect_change_slider_values(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Slider, &UiElementState, &ComputedNode, &Children, &UiGenID), With<Slider>>,
    mut track_query: Query<(&mut Node, &BindToID), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID), (With<SliderThumb>, Without<SliderTrack>)>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    for (mut slider, state, computed_node, children, ui_id) in query.iter_mut() {
        // Skip unfocused sliders
        if !state.selected {
            continue;
        }
        
        let old_value = slider.value;

        // Value change via arrow keys
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            slider.value = (slider.value + if shift {slider.step * 10} else {slider.step}).min(slider.max);
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            slider.value = (slider.value - if shift {slider.step * 10} else {slider.step}).max(slider.min);
        }

        if slider.value != old_value {
            let slider_width = computed_node.size().x / 1.5;
            let percent = (slider.value - slider.min) as f32 / (slider.max - slider.min) as f32;
            let clamped_x = percent * slider_width;

            for child in children.iter() {
                if let Ok((mut thumb_node, mut thumb, bind_to_thumb)) = thumb_query.get_mut(*child) {
                    if bind_to_thumb.0 != ui_id.0 {
                        continue;
                    }
                    thumb.current_x = clamped_x;
                    thumb_node.left = Val::Px(clamped_x - 8.0);
                }
                if let Ok((mut track_node, bind_to_track)) = track_query.get_mut(*child) {
                    if bind_to_track.0 != ui_id.0 {
                        continue;
                    }
                    track_node.width = Val::Px(clamped_x);
                }
            }
        }
    }
}

fn on_move_thumb(
    event: Trigger<Pointer<Drag>>,
    mut query: Query<(&mut Slider, &ComputedNode, &Children), With<Slider>>,
    mut track_query: Query<&mut Node, (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb), (With<SliderThumb>, Without<SliderTrack>)>,
) {
    for (mut slider, computed_node, children) in query.iter_mut() {
        let slider_width = computed_node.size().x / 1.5;
        for child in children.iter() {
            if event.target.eq(child) {
                let next_child = children.iter().next();
                if let Ok((mut thumb_node, mut slider_thumb)) = thumb_query.get_mut(*child) {
                    slider_thumb.current_x += event.event.delta.x;
                    slider_thumb.current_x = slider_thumb.current_x.clamp(0.0, slider_width);

                    thumb_node.left = Val::Px(slider_thumb.current_x - 8.0);

                    if let Some(track_child) = next_child {
                        if let Ok(mut track_node) = track_query.get_mut(*track_child) {
                            track_node.width = Val::Px(slider_thumb.current_x);
                        }
                    }

                    let percent = slider_thumb.current_x / slider_width;
                    let range = (slider.max - slider.min) as f32;
                    let raw_value = slider.min as f32 + percent * range;
                    let stepped_value = ((raw_value / slider.step as f32).round() * slider.step as f32) as i32;
                    slider.value = stepped_value;
                }
            }
        }
    }
}

fn on_click_track(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(Entity, &mut Slider, &ComputedNode, &UiGenID, &Children), With<Slider>>,
    mut track_query: Query<(&mut Node, &BindToID), (With<SliderTrack>, Without<SliderThumb>)>,
    mut thumb_query: Query<(&mut Node, &mut SliderThumb, &BindToID), (With<SliderThumb>, Without<SliderTrack>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    for (entity, mut slider, computed_node, ui_id, children) in query.iter_mut() {
        if event.target.eq(&entity) {
            let slider_width = computed_node.size().x / 1.5;
            let track_left = (window.width() - slider_width) / 2.0;
            let click_x = event.pointer_location.position.x - track_left;
            let clamped_x = click_x.clamp(0.0, slider_width);

            for child in children.iter() {
                let next_child = children.iter().next();
                if let Some(track_child) = next_child {
                    if let Ok((mut thumb_node, mut slider_thumb, bind_to_thumb)) = thumb_query.get_mut(*child) {
                        if bind_to_thumb.0 != ui_id.0 {
                            continue;
                        }
                        slider_thumb.current_x = clamped_x;
                        thumb_node.left = Val::Px(clamped_x - 8.0);

                        if let Ok((mut track_node, bind_to_track)) = track_query.get_mut(*track_child) {
                            if bind_to_track.0 != ui_id.0 {
                                continue;
                            }
                            track_node.width = Val::Px(clamped_x);
                        }

                        let percent = slider_thumb.current_x / slider_width;
                        let range = (slider.max - slider.min) as f32;
                        let raw_value = slider.min as f32 + percent * range;
                        let stepped_value = ((raw_value / slider.step as f32).round() * slider.step as f32) as i32;
                        slider.value = stepped_value;
                    }
                }
            }
        }
    }
}

fn on_internal_mouse_click(
    event: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UiElementState, &UiGenID), With<Slider>>,
    mut current_element_selected: ResMut<CurrentElementSelected>
) {
    if let Ok((mut state, gen_id)) = query.get_mut(event.target) {
        state.selected = true;
        current_element_selected.0 = gen_id.0;
    }
}


fn default_style(overwrite: Option<&BaseStyle>) -> InternalStyle {
    let mut internal_style = InternalStyle(Style {
        width: Val::Px(400.),
        min_width: Val::Px(100.),
        height: Val::Px(8.),
        display: Display::Flex,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        background: Background { color: Color::srgba(1.0, 1.0, 1.0, 1.0), ..default() },
        border: UiRect::all(Val::Px(0.)),
        border_radius: Radius::all(Val::Px(5.)),
        ..default()
    });

    if let Some(style) = overwrite {
        internal_style.merge_styles(&style.0);
    }
    internal_style
}

