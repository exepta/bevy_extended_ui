use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::widget_util::{apply_overlay_state_for_bind, set_z_index_pair};
use crate::widgets::{
    BindToID, ColorPicker, UIGenID, UIWidgetState, WidgetId, WidgetKind, hsv_to_rgb_u8,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::{RelativeCursorPosition, UiGlobalTransform, UiScale};
use bevy::window::PrimaryWindow;

const SV_CANVAS_SIZE: u32 = 192;
const TRACK_WIDTH: u32 = 256;
const TRACK_HEIGHT: u32 = 14;
const MODAL_FALLBACK_WIDTH: f32 = 340.0;
const COLOR_PICKER_MODAL_ROOT_Z: i32 = 40_000;
const COLOR_PICKER_MODAL_Z: i32 = 40_001;

fn set_if_changed<T: PartialEq>(target: &mut T, value: T) {
    if *target != value {
        *target = value;
    }
}

/// Marker component for initialized color picker widgets.
#[derive(Component)]
struct ColorPickerBase;

/// Marker inserted when a color picker changed because of pointer input.
#[derive(Component)]
pub struct ColorPickerUserChanged;

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

/// Tracks the last visual state applied to a color picker.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct ColorPickerVisualSnapshot {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
    hue: f32,
    saturation: f32,
    value: f32,
}

impl From<&ColorPicker> for ColorPickerVisualSnapshot {
    fn from(picker: &ColorPicker) -> Self {
        Self {
            red: picker.red,
            green: picker.green,
            blue: picker.blue,
            alpha: picker.alpha,
            hue: picker.hue,
            saturation: picker.saturation,
            value: picker.value,
        }
    }
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
    /// Handles `build` in the extended UI workflow.
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
        app.add_systems(Last, update_modal_position);
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
            .insert(GlobalZIndex::default())
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
                        ZIndex(COLOR_PICKER_MODAL_Z),
                        Pickable::IGNORE,
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["color-picker-modal".to_string()]),
                        BindToID(id.0),
                        RenderLayers::layer(layer),
                        ColorPickerModal,
                    ))
                    .insert(GlobalZIndex::default())
                    .with_children(|modal| {
                        modal
                            .spawn((
                                Name::new(format!("ColorPicker-Canvas-{}", picker.entry)),
                                Node::default(),
                                ImageNode::new(canvas).with_mode(NodeImageMode::Stretch),
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
                            .observe(on_canvas_release)
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
                                ImageNode::new(hue).with_mode(NodeImageMode::Stretch),
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
                            .observe(on_hue_release)
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
                                ImageNode::new(alpha).with_mode(NodeImageMode::Stretch),
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
                            .observe(on_alpha_release)
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
    mut modal_q: Query<
        (&mut Visibility, &mut ZIndex, &mut GlobalZIndex, &BindToID),
        (With<ColorPickerModal>, Without<ColorPicker>),
    >,
    mut root_z_q: Query<
        (&mut ZIndex, &mut GlobalZIndex, &UIGenID),
        (With<ColorPicker>, Without<ColorPickerModal>),
    >,
) {
    for (mut state, id) in picker_q.iter_mut() {
        if state.disabled || !state.focused {
            state.open = false;
        }

        for (mut z, mut global_z, z_id) in root_z_q.iter_mut() {
            if z_id.0 == id.0 {
                set_z_index_pair(&mut z, &mut global_z, state.open, COLOR_PICKER_MODAL_ROOT_Z);
            }
        }

        apply_overlay_state_for_bind(id.0, state.open, COLOR_PICKER_MODAL_Z, &mut modal_q);
    }
}

/// Positions the modal centered under the trigger with 10px gap.
fn update_modal_position(
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    picker_q: Query<
        (&UIGenID, &UIWidgetState, &ComputedNode, &UiGlobalTransform),
        With<ColorPicker>,
    >,
    trigger_q: Query<(&BindToID, &ComputedNode, &UiGlobalTransform), With<ColorPickerTrigger>>,
    mut modal_q: Query<
        (&mut Node, &BindToID, &ComputedNode, Option<&mut UiStyle>),
        With<ColorPickerModal>,
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let gap = 10.0;
    let margin = 8.0;
    let scale = (window.scale_factor() * ui_scale.0).max(f32::EPSILON);
    let viewport_size = Vec2::new(window.width(), window.height());

    for (id, state, root_node, root_transform) in picker_q.iter() {
        if !state.open {
            continue;
        }

        let Some((trigger_node, trigger_transform)) = trigger_q
            .iter()
            .find(|(bind, _, _)| bind.0 == id.0)
            .map(|(_, node, transform)| (node, transform))
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
        let root_top_left = top_left_ui(root_node, root_transform, scale);
        let trigger_top_left = top_left_ui(trigger_node, trigger_transform, scale);

        let trigger_w = trigger_size.x.max(1.0);
        let trigger_h = trigger_size.y.max(1.0);
        let raw_modal_w = if modal_size.x.is_finite() && modal_size.x > 8.0 {
            modal_size.x
        } else {
            MODAL_FALLBACK_WIDTH
        };
        let raw_modal_h = if modal_size.y.is_finite() && modal_size.y > 8.0 {
            modal_size.y
        } else {
            294.0
        };
        let modal_w = raw_modal_w.min((viewport_size.x - margin * 2.0).max(160.0));
        let modal_h = raw_modal_h.min((viewport_size.y - margin * 2.0).max(180.0));

        let ideal_x = trigger_top_left.x + (trigger_w - modal_w) * 0.5;
        let max_x = (viewport_size.x - modal_w - margin).max(margin);
        let absolute_x = ideal_x.clamp(margin, max_x);

        let below_y = trigger_top_left.y + trigger_h + gap;
        let above_y = trigger_top_left.y - modal_h - gap;
        let absolute_y = if below_y + modal_h <= viewport_size.y - margin || above_y < margin {
            below_y
        } else {
            above_y
        }
        .clamp(margin, (viewport_size.y - modal_h - margin).max(margin));

        let local_left = Val::Px(absolute_x - root_top_left.x);
        let local_top = Val::Px(absolute_y - root_top_left.y);
        let local_width = Val::Px(modal_w);
        let local_max_height = Val::Px(modal_h);

        set_if_changed(&mut modal_node.left, local_left);
        set_if_changed(&mut modal_node.top, local_top);
        set_if_changed(&mut modal_node.width, local_width);
        set_if_changed(&mut modal_node.max_height, local_max_height);

        if let Some(mut styles) = maybe_styles {
            let mut changed = false;
            {
                let styles = styles.bypass_change_detection();
                if let Some(active) = styles.active_style.as_mut() {
                    if active.left != Some(local_left) {
                        active.left = Some(local_left);
                        changed = true;
                    }
                    if active.top != Some(local_top) {
                        active.top = Some(local_top);
                        changed = true;
                    }
                    if active.width != Some(local_width) {
                        active.width = Some(local_width);
                        changed = true;
                    }
                    if active.max_height != Some(local_max_height) {
                        active.max_height = Some(local_max_height);
                        changed = true;
                    }
                }
                for (_, style) in styles.styles.iter_mut() {
                    if style.normal.left != Some(local_left) {
                        style.normal.left = Some(local_left);
                        changed = true;
                    }
                    if style.normal.top != Some(local_top) {
                        style.normal.top = Some(local_top);
                        changed = true;
                    }
                    if style.normal.width != Some(local_width) {
                        style.normal.width = Some(local_width);
                        changed = true;
                    }
                    if style.normal.max_height != Some(local_max_height) {
                        style.normal.max_height = Some(local_max_height);
                        changed = true;
                    }
                }
            }
            if changed {
                styles.set_changed();
            }
        }
    }
}

/// Handles `logical_size` in the extended UI workflow.
fn logical_size(node: &ComputedNode) -> Vec2 {
    let inv = node.inverse_scale_factor.max(f32::EPSILON);
    node.size() * inv
}

fn top_left_ui(node: &ComputedNode, transform: &UiGlobalTransform, scale: f32) -> Vec2 {
    let half = node.size() * 0.5;
    transform.affine().transform_point2(-half) / scale
}

/// Syncs generated textures, preview and labels when the color changes.
fn sync_color_picker_visual_state(
    mut commands: Commands,
    picker_q: Query<
        (
            Entity,
            &ColorPicker,
            &UIGenID,
            &ColorPickerTextures,
            Option<&ColorPickerVisualSnapshot>,
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
    for (entity, picker, id, textures, snapshot) in picker_q.iter() {
        let next_snapshot = ColorPickerVisualSnapshot::from(picker);
        if snapshot.is_some_and(|snapshot| *snapshot == next_snapshot) {
            continue;
        }

        let hue_changed = snapshot.is_none_or(|snapshot| snapshot.hue != picker.hue);
        let rgb_changed = snapshot.is_none_or(|snapshot| {
            snapshot.red != picker.red
                || snapshot.green != picker.green
                || snapshot.blue != picker.blue
        });

        if hue_changed {
            if let Some(mut canvas_image) = images.get_mut(&textures.canvas) {
                *canvas_image = generate_sv_canvas_image(picker.hue);
            }
        }
        if rgb_changed {
            if let Some(mut alpha_image) = images.get_mut(&textures.alpha) {
                *alpha_image = generate_alpha_track_image(picker.red, picker.green, picker.blue);
            }
        }

        {
            let mut trigger_bg_q = params.p0();
            for (mut color, bind, maybe_styles) in trigger_bg_q.iter_mut() {
                if bind.0 == id.0 {
                    let background =
                        Color::srgba_u8(picker.red, picker.green, picker.blue, picker.alpha);
                    set_if_changed(&mut *color, BackgroundColor(background));

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
                    set_if_changed(
                        &mut node.left,
                        Val::Percent((picker.saturation * 100.0).clamp(0.0, 100.0)),
                    );
                    set_if_changed(
                        &mut node.top,
                        Val::Percent(((1.0 - picker.value) * 100.0).clamp(0.0, 100.0)),
                    );
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

                    set_if_changed(&mut node.left, Val::Px(left));
                    set_if_changed(&mut node.top, Val::Px(top));
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

                    set_if_changed(&mut node.left, Val::Px(left));
                    set_if_changed(&mut node.top, Val::Px(top));
                }
            }
        }

        {
            let mut hex_q = params.p4();
            for (mut text, mut text_color, bind, maybe_styles) in hex_q.iter_mut() {
                if bind.0 == id.0 {
                    set_if_changed(&mut text.0, picker.hex());
                    set_if_changed(
                        &mut text_color.0,
                        trigger_text_color(picker.red, picker.green, picker.blue, picker.alpha),
                    );

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
                    set_if_changed(&mut text.0, format!("RGB  {}", picker.rgb_string()));
                }
            }
        }
        {
            let mut rgba_q = params.p6();
            for (mut text, bind) in rgba_q.iter_mut() {
                if bind.0 == id.0 {
                    set_if_changed(&mut text.0, format!("RGBA {}", picker.rgba_string()));
                }
            }
        }

        commands.entity(entity).insert(next_snapshot);
    }
}

/// Handles `perceived_luminance` in the extended UI workflow.
fn perceived_luminance(red: u8, green: u8, blue: u8) -> f32 {
    let r = red as f32 / 255.0;
    let g = green as f32 / 255.0;
    let b = blue as f32 / 255.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Handles `trigger_text_color` in the extended UI workflow.
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
    mut commands: Commands,
    canvas_q: Query<(&BindToID, &RelativeCursorPosition), With<ColorCanvas>>,
    mut picker_q: Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_canvas_pointer(
        trigger.entity,
        &mut commands,
        &canvas_q,
        &mut picker_q,
        &mut current_widget_state,
        true,
    );
    trigger.propagate(false);
}

/// Handles drag interaction on the SV canvas.
fn on_canvas_drag(
    trigger: On<Pointer<Drag>>,
    mut commands: Commands,
    canvas_q: Query<(&BindToID, &RelativeCursorPosition), With<ColorCanvas>>,
    mut picker_q: Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_canvas_pointer(
        trigger.entity,
        &mut commands,
        &canvas_q,
        &mut picker_q,
        &mut current_widget_state,
        false,
    );
}

/// Emits a color change after canvas dragging has finished.
fn on_canvas_release(
    mut trigger: On<Pointer<Release>>,
    mut commands: Commands,
    canvas_q: Query<&BindToID, With<ColorCanvas>>,
    picker_q: Query<(Entity, &UIGenID), With<ColorPicker>>,
) {
    mark_bound_picker_user_changed(trigger.entity, &mut commands, &canvas_q, &picker_q);
    trigger.propagate(false);
}

/// Handles `apply_canvas_pointer` in the extended UI workflow.
fn apply_canvas_pointer(
    entity: Entity,
    commands: &mut Commands,
    canvas_q: &Query<(&BindToID, &RelativeCursorPosition), With<ColorCanvas>>,
    picker_q: &mut Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    current_widget_state: &mut ResMut<CurrentWidgetState>,
    emit_change: bool,
) {
    let Ok((bind, rel)) = canvas_q.get(entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };

    let saturation = (normalized.x + 0.5).clamp(0.0, 1.0);
    let value = 1.0 - (normalized.y + 0.5).clamp(0.0, 1.0);

    for (picker_entity, mut picker, mut state, id) in picker_q.iter_mut() {
        if id.0 != bind.0 {
            continue;
        }
        if state.disabled {
            return;
        }
        state.focused = true;
        current_widget_state.widget_id = id.0;
        let hue = picker.hue;
        let old_saturation = picker.saturation;
        let old_value = picker.value;
        let old_red = picker.red;
        let old_green = picker.green;
        let old_blue = picker.blue;
        picker.set_hsv(hue, saturation, value);
        if emit_change
            && (old_saturation != picker.saturation
                || old_value != picker.value
                || old_red != picker.red
                || old_green != picker.green
                || old_blue != picker.blue)
        {
            commands
                .entity(picker_entity)
                .insert(ColorPickerUserChanged);
        }
        break;
    }
}

/// Handles clicks on the hue track.
fn on_hue_click(
    mut trigger: On<Pointer<Click>>,
    mut commands: Commands,
    hue_q: Query<(&BindToID, &RelativeCursorPosition), With<HueTrack>>,
    mut picker_q: Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_hue_pointer(
        trigger.entity,
        &mut commands,
        &hue_q,
        &mut picker_q,
        &mut current_widget_state,
        true,
    );
    trigger.propagate(false);
}

/// Handles drag interaction on the hue track.
fn on_hue_drag(
    trigger: On<Pointer<Drag>>,
    mut commands: Commands,
    hue_q: Query<(&BindToID, &RelativeCursorPosition), With<HueTrack>>,
    mut picker_q: Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_hue_pointer(
        trigger.entity,
        &mut commands,
        &hue_q,
        &mut picker_q,
        &mut current_widget_state,
        false,
    );
}

/// Emits a color change after hue dragging has finished.
fn on_hue_release(
    mut trigger: On<Pointer<Release>>,
    mut commands: Commands,
    hue_q: Query<&BindToID, With<HueTrack>>,
    picker_q: Query<(Entity, &UIGenID), With<ColorPicker>>,
) {
    mark_bound_picker_user_changed(trigger.entity, &mut commands, &hue_q, &picker_q);
    trigger.propagate(false);
}

/// Handles `apply_hue_pointer` in the extended UI workflow.
fn apply_hue_pointer(
    entity: Entity,
    commands: &mut Commands,
    hue_q: &Query<(&BindToID, &RelativeCursorPosition), With<HueTrack>>,
    picker_q: &mut Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    current_widget_state: &mut ResMut<CurrentWidgetState>,
    emit_change: bool,
) {
    let Ok((bind, rel)) = hue_q.get(entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };

    let hue = (normalized.x + 0.5).clamp(0.0, 1.0) * 360.0;

    for (picker_entity, mut picker, mut state, id) in picker_q.iter_mut() {
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
        let old_hue = picker.hue;
        let old_red = picker.red;
        let old_green = picker.green;
        let old_blue = picker.blue;
        picker.set_hsv(hue, saturation, value);
        if emit_change
            && (old_hue != picker.hue
                || old_red != picker.red
                || old_green != picker.green
                || old_blue != picker.blue)
        {
            commands
                .entity(picker_entity)
                .insert(ColorPickerUserChanged);
        }
        break;
    }
}

/// Handles clicks on the alpha track.
fn on_alpha_click(
    mut trigger: On<Pointer<Click>>,
    mut commands: Commands,
    alpha_q: Query<(&BindToID, &RelativeCursorPosition), With<AlphaTrack>>,
    mut picker_q: Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_alpha_pointer(
        trigger.entity,
        &mut commands,
        &alpha_q,
        &mut picker_q,
        &mut current_widget_state,
        true,
    );
    trigger.propagate(false);
}

/// Handles drag interaction on the alpha track.
fn on_alpha_drag(
    trigger: On<Pointer<Drag>>,
    mut commands: Commands,
    alpha_q: Query<(&BindToID, &RelativeCursorPosition), With<AlphaTrack>>,
    mut picker_q: Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    apply_alpha_pointer(
        trigger.entity,
        &mut commands,
        &alpha_q,
        &mut picker_q,
        &mut current_widget_state,
        false,
    );
}

/// Emits a color change after alpha dragging has finished.
fn on_alpha_release(
    mut trigger: On<Pointer<Release>>,
    mut commands: Commands,
    alpha_q: Query<&BindToID, With<AlphaTrack>>,
    picker_q: Query<(Entity, &UIGenID), With<ColorPicker>>,
) {
    mark_bound_picker_user_changed(trigger.entity, &mut commands, &alpha_q, &picker_q);
    trigger.propagate(false);
}

fn mark_bound_picker_user_changed<T: Component>(
    source_entity: Entity,
    commands: &mut Commands,
    source_q: &Query<&BindToID, With<T>>,
    picker_q: &Query<(Entity, &UIGenID), With<ColorPicker>>,
) {
    let Ok(bind) = source_q.get(source_entity) else {
        return;
    };

    for (picker_entity, id) in picker_q.iter() {
        if id.0 == bind.0 {
            commands
                .entity(picker_entity)
                .insert(ColorPickerUserChanged);
            break;
        }
    }
}

/// Handles `apply_alpha_pointer` in the extended UI workflow.
fn apply_alpha_pointer(
    entity: Entity,
    commands: &mut Commands,
    alpha_q: &Query<(&BindToID, &RelativeCursorPosition), With<AlphaTrack>>,
    picker_q: &mut Query<
        (Entity, &mut ColorPicker, &mut UIWidgetState, &UIGenID),
        With<ColorPicker>,
    >,
    current_widget_state: &mut ResMut<CurrentWidgetState>,
    emit_change: bool,
) {
    let Ok((bind, rel)) = alpha_q.get(entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };

    let alpha = ((normalized.x + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;

    for (picker_entity, mut picker, mut state, id) in picker_q.iter_mut() {
        if id.0 != bind.0 {
            continue;
        }
        if state.disabled {
            return;
        }
        state.focused = true;
        current_widget_state.widget_id = id.0;
        if picker.alpha != alpha {
            picker.alpha = alpha;
            if emit_change {
                commands
                    .entity(picker_entity)
                    .insert(ColorPickerUserChanged);
            }
        }
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
