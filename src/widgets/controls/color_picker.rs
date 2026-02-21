use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::{
    BindToID, ColorPicker, UIGenID, UIWidgetState, WidgetId, WidgetKind, hsv_to_rgb_u8,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::RelativeCursorPosition;

const SV_CANVAS_SIZE: u32 = 192;
const TRACK_WIDTH: u32 = 256;
const TRACK_HEIGHT: u32 = 14;
const MODAL_FALLBACK_WIDTH: f32 = 340.0;

/// Marker component for initialized color picker widgets.
#[derive(Component)]
struct ColorPickerBase;

/// Marker component for the picker trigger button.
#[derive(Component)]
struct ColorPickerTrigger;

/// Marker component for the picker modal container.
#[derive(Component)]
struct ColorPickerModal;

/// Stores generated texture handles for one color picker.
#[derive(Component, Clone)]
struct ColorPickerTextures {
    canvas: Handle<Image>,
    alpha: Handle<Image>,
}

/// Marker component for the saturation/value canvas.
#[derive(Component)]
struct ColorCanvas;

/// Marker component for the canvas thumb.
#[derive(Component)]
struct ColorCanvasThumb;

/// Marker component for the hue track.
#[derive(Component)]
struct HueTrack;

/// Marker component for the hue thumb.
#[derive(Component)]
struct HueThumb;

/// Marker component for the alpha track.
#[derive(Component)]
struct AlphaTrack;

/// Marker component for the alpha thumb.
#[derive(Component)]
struct AlphaThumb;

/// Marker component for HEX output text.
#[derive(Component)]
struct HexText;

/// Marker component for RGB output text.
#[derive(Component)]
struct RgbText;

/// Marker component for RGBA output text.
#[derive(Component)]
struct RgbaText;

/// Plugin that registers color picker widget behavior.
pub struct ColorPickerWidget;

impl Plugin for ColorPickerWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                update_modal_visibility,
                update_modal_position,
                sync_color_picker_visual_state,
            )
                .chain(),
        );
    }
}

/// Initializes UI nodes for color picker widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &ColorPicker, Option<&CssSource>),
        (With<ColorPicker>, Without<ColorPickerBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for (entity, id, picker, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        let canvas = images.add(generate_sv_canvas_image(picker.hue));
        let hue = images.add(generate_hue_track_image());
        let alpha = images.add(generate_alpha_track_image(
            picker.red,
            picker.green,
            picker.blue,
        ));

        commands
            .entity(entity)
            .insert((
                Name::new(format!("ColorPicker-{}", picker.entry)),
                Node::default(),
                WidgetId {
                    id: picker.entry,
                    kind: WidgetKind::ColorPicker,
                },
                BackgroundColor::default(),
                BorderColor::default(),
                ImageNode::default(),
                BoxShadow::new(
                    Colored::TRANSPARENT,
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(0.0),
                ),
                Pickable::default(),
                css_source.clone(),
                TagName("colorpicker".to_string()),
                RenderLayers::layer(layer),
                CssClass(vec!["color-picker-root".to_string()]),
                ColorPickerTextures {
                    canvas: canvas.clone(),
                    alpha: alpha.clone(),
                },
                ColorPickerBase,
            ))
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {
                builder
                    .spawn((
                        Name::new(format!("ColorPicker-Trigger-{}", picker.entry)),
                        Node::default(),
                        BackgroundColor(Color::srgb_u8(picker.red, picker.green, picker.blue)),
                        BorderColor::default(),
                        ImageNode::default(),
                        Pickable::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["color-picker-trigger".to_string()]),
                        BindToID(id.0),
                        RenderLayers::layer(layer),
                        ColorPickerTrigger,
                    ))
                    .observe(on_trigger_click)
                    .with_children(|head| {
                        head.spawn((
                            Name::new(format!("ColorPicker-Trigger-Hex-{}", picker.entry)),
                            Text::new(format!("HEX  {}", picker.hex())),
                            TextColor(trigger_text_color(
                                picker.red,
                                picker.green,
                                picker.blue,
                                picker.alpha,
                            )),
                            TextFont::default(),
                            TextLayout::default(),
                            Pickable::IGNORE,
                            UIWidgetState::default(),
                            css_source.clone(),
                            CssClass(vec!["color-value-hex".to_string()]),
                            BindToID(id.0),
                            RenderLayers::layer(layer),
                            HexText,
                        ));
                    });

                builder
                    .spawn((
                        Name::new(format!("ColorPicker-Modal-{}", picker.entry)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(52.0),
                            ..default()
                        },
                        BackgroundColor::default(),
                        BorderColor::default(),
                        ImageNode::default(),
                        Visibility::Hidden,
                        ZIndex(20),
                        Pickable::IGNORE,
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["color-picker-modal".to_string()]),
                        BindToID(id.0),
                        RenderLayers::layer(layer),
                        ColorPickerModal,
                    ))
                    .with_children(|modal| {
                        modal
                            .spawn((
                                Name::new(format!("ColorPicker-Canvas-{}", picker.entry)),
                                Node::default(),
                                ImageNode::new(canvas),
                                BackgroundColor::default(),
                                BorderColor::default(),
                                RelativeCursorPosition::default(),
                                Pickable::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["color-canvas".to_string()]),
                                BindToID(id.0),
                                RenderLayers::layer(layer),
                                ColorCanvas,
                            ))
                            .observe(on_canvas_click)
                            .observe(on_canvas_drag)
                            .with_children(|canvas_builder| {
                                canvas_builder.spawn((
                                    Name::new(format!("ColorPicker-Canvas-Thumb-{}", picker.entry)),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Percent(0.0),
                                        top: Val::Percent(100.0),
                                        margin: UiRect::all(Val::Px(-7.0)),
                                        ..default()
                                    },
                                    BackgroundColor::default(),
                                    BorderColor::default(),
                                    ImageNode::default(),
                                    Pickable::IGNORE,
                                    UIWidgetState::default(),
                                    css_source.clone(),
                                    CssClass(vec!["color-canvas-thumb".to_string()]),
                                    BindToID(id.0),
                                    RenderLayers::layer(layer),
                                    ColorCanvasThumb,
                                ));
                            });

                        modal
                            .spawn((
                                Name::new(format!("ColorPicker-Hue-{}", picker.entry)),
                                Node::default(),
                                ImageNode::new(hue),
                                BackgroundColor::default(),
                                BorderColor::default(),
                                RelativeCursorPosition::default(),
                                Pickable::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["color-hue-track".to_string()]),
                                BindToID(id.0),
                                RenderLayers::layer(layer),
                                HueTrack,
                            ))
                            .observe(on_hue_click)
                            .observe(on_hue_drag)
                            .with_children(|track| {
                                track.spawn((
                                    Name::new(format!("ColorPicker-Hue-Thumb-{}", picker.entry)),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        ..default()
                                    },
                                    BackgroundColor::default(),
                                    BorderColor::default(),
                                    ImageNode::default(),
                                    Pickable::IGNORE,
                                    UIWidgetState::default(),
                                    css_source.clone(),
                                    CssClass(vec!["color-track-thumb".to_string()]),
                                    BindToID(id.0),
                                    RenderLayers::layer(layer),
                                    HueThumb,
                                ));
                            });

                        modal
                            .spawn((
                                Name::new(format!("ColorPicker-Alpha-{}", picker.entry)),
                                Node::default(),
                                ImageNode::new(alpha),
                                BackgroundColor::default(),
                                BorderColor::default(),
                                RelativeCursorPosition::default(),
                                Pickable::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["color-alpha-track".to_string()]),
                                BindToID(id.0),
                                RenderLayers::layer(layer),
                                AlphaTrack,
                            ))
                            .observe(on_alpha_click)
                            .observe(on_alpha_drag)
                            .with_children(|track| {
                                track.spawn((
                                    Name::new(format!("ColorPicker-Alpha-Thumb-{}", picker.entry)),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        ..default()
                                    },
                                    BackgroundColor::default(),
                                    BorderColor::default(),
                                    ImageNode::default(),
                                    Pickable::IGNORE,
                                    UIWidgetState::default(),
                                    css_source.clone(),
                                    CssClass(vec!["color-track-thumb".to_string()]),
                                    BindToID(id.0),
                                    RenderLayers::layer(layer),
                                    AlphaThumb,
                                ));
                            });

                        modal
                            .spawn((
                                Name::new(format!("ColorPicker-Values-{}", picker.entry)),
                                Node::default(),
                                BackgroundColor::default(),
                                BorderColor::default(),
                                ImageNode::default(),
                                Pickable::IGNORE,
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["color-values".to_string()]),
                                BindToID(id.0),
                                RenderLayers::layer(layer),
                            ))
                            .with_children(|values| {
                                values.spawn((
                                    Name::new(format!("ColorPicker-Rgb-{}", picker.entry)),
                                    Text::new(String::new()),
                                    TextColor::default(),
                                    TextFont::default(),
                                    TextLayout::default(),
                                    Pickable::IGNORE,
                                    UIWidgetState::default(),
                                    css_source.clone(),
                                    CssClass(vec![
                                        "color-value".to_string(),
                                        "color-value-rgb".to_string(),
                                    ]),
                                    BindToID(id.0),
                                    RenderLayers::layer(layer),
                                    RgbText,
                                ));

                                values.spawn((
                                    Name::new(format!("ColorPicker-Rgba-{}", picker.entry)),
                                    Text::new(String::new()),
                                    TextColor::default(),
                                    TextFont::default(),
                                    TextLayout::default(),
                                    Pickable::IGNORE,
                                    UIWidgetState::default(),
                                    css_source.clone(),
                                    CssClass(vec![
                                        "color-value".to_string(),
                                        "color-value-rgba".to_string(),
                                    ]),
                                    BindToID(id.0),
                                    RenderLayers::layer(layer),
                                    RgbaText,
                                ));
                            });
                    });
            });
    }
}

/// Opens/closes the modal based on picker focus/open state.
fn update_modal_visibility(
    mut picker_q: Query<
        (&mut UIWidgetState, &UIGenID),
        (With<ColorPicker>, Changed<UIWidgetState>),
    >,
    mut modal_q: Query<(&mut Visibility, &BindToID), With<ColorPickerModal>>,
    mut root_z_q: Query<(&mut ZIndex, &UIGenID), With<ColorPicker>>,
) {
    for (mut state, id) in picker_q.iter_mut() {
        if state.disabled || !state.focused {
            state.open = false;
        }

        for (mut z, z_id) in root_z_q.iter_mut() {
            if z_id.0 == id.0 {
                z.0 = if state.open { 20 } else { 0 };
            }
        }

        for (mut visibility, bind) in modal_q.iter_mut() {
            if bind.0 != id.0 {
                continue;
            }
            *visibility = if state.open {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// Positions the modal centered under the trigger with 10px gap.
fn update_modal_position(
    picker_q: Query<(&UIGenID, &UIWidgetState), With<ColorPicker>>,
    trigger_q: Query<(&BindToID, &ComputedNode), With<ColorPickerTrigger>>,
    mut modal_q: Query<
        (&mut Node, &BindToID, &ComputedNode, Option<&mut UiStyle>),
        With<ColorPickerModal>,
    >,
) {
    let gap = 10.0;

    for (id, state) in picker_q.iter() {
        if !state.open {
            continue;
        }

        let Some(trigger_node) = trigger_q
            .iter()
            .find(|(bind, _)| bind.0 == id.0)
            .map(|(_, node)| node)
        else {
            continue;
        };

        let Some((mut modal_node, modal_computed, maybe_styles)) = modal_q
            .iter_mut()
            .find(|(_, bind, _, _)| bind.0 == id.0)
            .map(|(node, _, computed, styles)| (node, computed, styles))
        else {
            continue;
        };

        let trigger_size = logical_size(trigger_node);
        let modal_size = logical_size(modal_computed);

        let trigger_w = trigger_size.x.max(1.0);
        let trigger_h = trigger_size.y.max(1.0);
        let modal_w = if modal_size.x.is_finite() && modal_size.x > 8.0 {
            modal_size.x
        } else {
            MODAL_FALLBACK_WIDTH
        };

        let local_left = Val::Px((trigger_w - modal_w) * 0.5);
        let local_top = Val::Px(trigger_h + gap);

        modal_node.left = local_left;
        modal_node.top = local_top;

        if let Some(mut styles) = maybe_styles {
            for (_, style) in styles.styles.iter_mut() {
                style.normal.left = Some(local_left);
                style.normal.top = Some(local_top);
            }
        }
    }
}

fn logical_size(node: &ComputedNode) -> Vec2 {
    let inv = node.inverse_scale_factor.max(f32::EPSILON);
    node.size() * inv
}

/// Syncs generated textures, preview and labels when the color changes.
fn sync_color_picker_visual_state(
    picker_q: Query<
        (
            &ColorPicker,
            Ref<ColorPicker>,
            &UIGenID,
            &ColorPickerTextures,
        ),
        With<ColorPickerBase>,
    >,
    mut images: ResMut<Assets<Image>>,
    hue_track_q: Query<(&BindToID, &ComputedNode), With<HueTrack>>,
    alpha_track_q: Query<(&BindToID, &ComputedNode), With<AlphaTrack>>,
    mut params: ParamSet<(
        Query<(&mut BackgroundColor, &BindToID, Option<&mut UiStyle>), With<ColorPickerTrigger>>,
        Query<(&mut Node, &BindToID), With<ColorCanvasThumb>>,
        Query<(&mut Node, &BindToID, &ComputedNode), With<HueThumb>>,
        Query<(&mut Node, &BindToID, &ComputedNode), With<AlphaThumb>>,
        Query<(&mut Text, &mut TextColor, &BindToID, Option<&mut UiStyle>), With<HexText>>,
        Query<(&mut Text, &BindToID), With<RgbText>>,
        Query<(&mut Text, &BindToID), With<RgbaText>>,
    )>,
) {
    for (picker, picker_ref, id, textures) in picker_q.iter() {
        if picker_ref.is_added() || picker_ref.is_changed() {
            if let Some(canvas_image) = images.get_mut(&textures.canvas) {
                *canvas_image = generate_sv_canvas_image(picker.hue);
            }
            if let Some(alpha_image) = images.get_mut(&textures.alpha) {
                *alpha_image = generate_alpha_track_image(picker.red, picker.green, picker.blue);
            }
        }

        {
            let mut trigger_bg_q = params.p0();
            for (mut color, bind, maybe_styles) in trigger_bg_q.iter_mut() {
                if bind.0 == id.0 {
                    let background =
                        Color::srgba_u8(picker.red, picker.green, picker.blue, picker.alpha);
                    *color = BackgroundColor(background);

                    if let Some(mut styles) = maybe_styles {
                        for (_, style) in styles.styles.iter_mut() {
                            style.normal.background = Some(crate::styles::Background {
                                color: background,
                                ..default()
                            });
                        }
                    }
                }
            }
        }

        {
            let mut canvas_thumb_q = params.p1();
            for (mut node, bind) in canvas_thumb_q.iter_mut() {
                if bind.0 == id.0 {
                    node.left = Val::Percent((picker.saturation * 100.0).clamp(0.0, 100.0));
                    node.top = Val::Percent(((1.0 - picker.value) * 100.0).clamp(0.0, 100.0));
                }
            }
        }

        {
            let mut hue_thumb_q = params.p2();
            for (mut node, bind, thumb_computed) in hue_thumb_q.iter_mut() {
                if bind.0 == id.0 {
                    let Some((_, track_computed)) = hue_track_q
                        .iter()
                        .find(|(track_bind, _)| track_bind.0 == id.0)
                    else {
                        continue;
                    };
                    let track_size = logical_size(track_computed);
                    let thumb_size = logical_size(thumb_computed);

                    let max_left = (track_size.x - thumb_size.x).max(0.0);
                    let left = (picker.hue / 360.0).clamp(0.0, 1.0) * max_left;
                    let top = ((track_size.y - thumb_size.y) * 0.5).max(0.0);

                    node.left = Val::Px(left);
                    node.top = Val::Px(top);
                }
            }
        }

        {
            let mut alpha_thumb_q = params.p3();
            for (mut node, bind, thumb_computed) in alpha_thumb_q.iter_mut() {
                if bind.0 == id.0 {
                    let Some((_, track_computed)) = alpha_track_q
                        .iter()
                        .find(|(track_bind, _)| track_bind.0 == id.0)
                    else {
                        continue;
                    };
                    let track_size = logical_size(track_computed);
                    let thumb_size = logical_size(thumb_computed);

                    let max_left = (track_size.x - thumb_size.x).max(0.0);
                    let left = (picker.alpha as f32 / 255.0).clamp(0.0, 1.0) * max_left;
                    let top = ((track_size.y - thumb_size.y) * 0.5).max(0.0);

                    node.left = Val::Px(left);
                    node.top = Val::Px(top);
                }
            }
        }

        {
            let mut hex_q = params.p4();
            for (mut text, mut text_color, bind, maybe_styles) in hex_q.iter_mut() {
                if bind.0 == id.0 {
                    text.0 = format!("{}", picker.hex());
                    text_color.0 =
                        trigger_text_color(picker.red, picker.green, picker.blue, picker.alpha);

                    if let Some(mut styles) = maybe_styles {
                        for (_, style) in styles.styles.iter_mut() {
                            style.normal.color = Some(text_color.0);
                        }
                    }
                }
            }
        }
        {
            let mut rgb_q = params.p5();
            for (mut text, bind) in rgb_q.iter_mut() {
                if bind.0 == id.0 {
                    text.0 = format!("RGB  {}", picker.rgb_string());
                }
            }
        }
        {
            let mut rgba_q = params.p6();
            for (mut text, bind) in rgba_q.iter_mut() {
                if bind.0 == id.0 {
                    text.0 = format!("RGBA {}", picker.rgba_string());
                }
            }
        }
    }
}

fn perceived_luminance(red: u8, green: u8, blue: u8) -> f32 {
    let r = red as f32 / 255.0;
    let g = green as f32 / 255.0;
    let b = blue as f32 / 255.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn trigger_text_color(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    // If trigger background is translucent, force dark text for readability
    // against likely bright surfaces behind it.
    if alpha < 150 {
        return Color::BLACK;
    }

    // Hard guarantee for bright/white backgrounds.
    if red >= 245 && green >= 245 && blue >= 245 {
        return Color::BLACK;
    }

    if perceived_luminance(red, green, blue) < 0.5 {
        Color::WHITE
    } else {
        Color::BLACK
    }
}

/// Handles click on the trigger button and toggles modal visibility.
fn on_trigger_click(
    mut trigger: On<Pointer<Click>>,
    trigger_q: Query<&BindToID, With<ColorPickerTrigger>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Ok(bind) = trigger_q.get(trigger.entity) else {
        return;
    };

    for (mut state, gen_id) in query.iter_mut() {
        if gen_id.0 != bind.0 {
            continue;
        }
        if state.disabled {
            trigger.propagate(false);
            return;
        }
        state.focused = true;
        state.open = !state.open;
        current_widget_state.widget_id = gen_id.0;
        break;
    }
    trigger.propagate(false);
}

/// Sets hovered state when the cursor enters a color picker.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<ColorPicker>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

/// Clears hovered state when the cursor leaves a color picker.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<ColorPicker>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

/// Handles clicks on the SV canvas.
fn on_canvas_click(
    mut trigger: On<Pointer<Click>>,
    canvas_q: Query<(&BindToID, &RelativeCursorPosition), With<ColorCanvas>>,
    mut picker_q: Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_canvas_pointer(
        trigger.entity,
        &canvas_q,
        &mut picker_q,
        &mut current_widget_state,
    );
    trigger.propagate(false);
}

/// Handles drag interaction on the SV canvas.
fn on_canvas_drag(
    trigger: On<Pointer<Drag>>,
    canvas_q: Query<(&BindToID, &RelativeCursorPosition), With<ColorCanvas>>,
    mut picker_q: Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_canvas_pointer(
        trigger.entity,
        &canvas_q,
        &mut picker_q,
        &mut current_widget_state,
    );
}

fn apply_canvas_pointer(
    entity: Entity,
    canvas_q: &Query<(&BindToID, &RelativeCursorPosition), With<ColorCanvas>>,
    picker_q: &mut Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    current_widget_state: &mut ResMut<CurrentWidgetState>,
) {
    let Ok((bind, rel)) = canvas_q.get(entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };

    let saturation = (normalized.x + 0.5).clamp(0.0, 1.0);
    let value = 1.0 - (normalized.y + 0.5).clamp(0.0, 1.0);

    for (mut picker, mut state, id) in picker_q.iter_mut() {
        if id.0 != bind.0 {
            continue;
        }
        if state.disabled {
            return;
        }
        state.focused = true;
        current_widget_state.widget_id = id.0;
        let hue = picker.hue;
        picker.set_hsv(hue, saturation, value);
        break;
    }
}

/// Handles clicks on the hue track.
fn on_hue_click(
    mut trigger: On<Pointer<Click>>,
    hue_q: Query<(&BindToID, &RelativeCursorPosition), With<HueTrack>>,
    mut picker_q: Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_hue_pointer(
        trigger.entity,
        &hue_q,
        &mut picker_q,
        &mut current_widget_state,
    );
    trigger.propagate(false);
}

/// Handles drag interaction on the hue track.
fn on_hue_drag(
    trigger: On<Pointer<Drag>>,
    hue_q: Query<(&BindToID, &RelativeCursorPosition), With<HueTrack>>,
    mut picker_q: Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_hue_pointer(
        trigger.entity,
        &hue_q,
        &mut picker_q,
        &mut current_widget_state,
    );
}

fn apply_hue_pointer(
    entity: Entity,
    hue_q: &Query<(&BindToID, &RelativeCursorPosition), With<HueTrack>>,
    picker_q: &mut Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    current_widget_state: &mut ResMut<CurrentWidgetState>,
) {
    let Ok((bind, rel)) = hue_q.get(entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };

    let hue = (normalized.x + 0.5).clamp(0.0, 1.0) * 360.0;

    for (mut picker, mut state, id) in picker_q.iter_mut() {
        if id.0 != bind.0 {
            continue;
        }
        if state.disabled {
            return;
        }
        state.focused = true;
        current_widget_state.widget_id = id.0;
        let saturation = picker.saturation;
        let value = picker.value;
        picker.set_hsv(hue, saturation, value);
        break;
    }
}

/// Handles clicks on the alpha track.
fn on_alpha_click(
    mut trigger: On<Pointer<Click>>,
    alpha_q: Query<(&BindToID, &RelativeCursorPosition), With<AlphaTrack>>,
    mut picker_q: Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_alpha_pointer(
        trigger.entity,
        &alpha_q,
        &mut picker_q,
        &mut current_widget_state,
    );
    trigger.propagate(false);
}

/// Handles drag interaction on the alpha track.
fn on_alpha_drag(
    trigger: On<Pointer<Drag>>,
    alpha_q: Query<(&BindToID, &RelativeCursorPosition), With<AlphaTrack>>,
    mut picker_q: Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_alpha_pointer(
        trigger.entity,
        &alpha_q,
        &mut picker_q,
        &mut current_widget_state,
    );
}

fn apply_alpha_pointer(
    entity: Entity,
    alpha_q: &Query<(&BindToID, &RelativeCursorPosition), With<AlphaTrack>>,
    picker_q: &mut Query<(&mut ColorPicker, &mut UIWidgetState, &UIGenID), With<ColorPicker>>,
    current_widget_state: &mut ResMut<CurrentWidgetState>,
) {
    let Ok((bind, rel)) = alpha_q.get(entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };

    let alpha = ((normalized.x + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;

    for (mut picker, mut state, id) in picker_q.iter_mut() {
        if id.0 != bind.0 {
            continue;
        }
        if state.disabled {
            return;
        }
        state.focused = true;
        current_widget_state.widget_id = id.0;
        picker.alpha = alpha;
        break;
    }
}

/// Generates the saturation/value canvas image for a given hue.
fn generate_sv_canvas_image(hue: f32) -> Image {
    let mut data = vec![0_u8; (SV_CANVAS_SIZE * SV_CANVAS_SIZE * 4) as usize];

    for y in 0..SV_CANVAS_SIZE {
        for x in 0..SV_CANVAS_SIZE {
            let s = x as f32 / SV_CANVAS_SIZE.saturating_sub(1) as f32;
            let v = 1.0 - y as f32 / SV_CANVAS_SIZE.saturating_sub(1) as f32;
            let (r, g, b) = hsv_to_rgb_u8(hue, s, v);
            let idx = ((y * SV_CANVAS_SIZE + x) * 4) as usize;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SV_CANVAS_SIZE,
            height: SV_CANVAS_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}

/// Generates a static hue gradient track image.
fn generate_hue_track_image() -> Image {
    let mut data = vec![0_u8; (TRACK_WIDTH * TRACK_HEIGHT * 4) as usize];

    for y in 0..TRACK_HEIGHT {
        for x in 0..TRACK_WIDTH {
            let hue = x as f32 / TRACK_WIDTH.saturating_sub(1) as f32 * 360.0;
            let (r, g, b) = hsv_to_rgb_u8(hue, 1.0, 1.0);
            let idx = ((y * TRACK_WIDTH + x) * 4) as usize;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: TRACK_WIDTH,
            height: TRACK_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}

/// Generates an alpha gradient track image for the selected RGB color.
fn generate_alpha_track_image(red: u8, green: u8, blue: u8) -> Image {
    let mut data = vec![0_u8; (TRACK_WIDTH * TRACK_HEIGHT * 4) as usize];

    for y in 0..TRACK_HEIGHT {
        for x in 0..TRACK_WIDTH {
            let alpha = x as f32 / TRACK_WIDTH.saturating_sub(1) as f32;
            let checker = ((x / 8 + y / 8) % 2) as u8;
            let base = if checker == 0 { 232_u8 } else { 206_u8 };

            let blend = |channel: u8| -> u8 {
                let ch = channel as f32;
                (base as f32 * (1.0 - alpha) + ch * alpha).round() as u8
            };

            let idx = ((y * TRACK_WIDTH + x) * 4) as usize;
            data[idx] = blend(red);
            data[idx + 1] = blend(green);
            data[idx + 2] = blend(blue);
            data[idx + 3] = 255;
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: TRACK_WIDTH,
            height: TRACK_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}
