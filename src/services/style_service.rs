use crate::ExtendedUiConfiguration;
use crate::ImageCache;
use crate::html::HtmlStyle;
use crate::services::image_service::get_or_load_image;
use crate::services::state_service::update_widget_states;
use crate::styles::components::UiStyle;
use crate::styles::{
    AnimationDirection, AnimationKeyframe, AnimationSpec, BackdropFilter, BackgroundAttachment,
    BackgroundPosition, BackgroundPositionValue, BackgroundSize, BackgroundSizeValue, CalcContext,
    CalcExpr, CursorStyle, FontVal, FontWeight, GradientStopPosition, LinearGradient, Radius,
    Style, TransformStyle, TransitionProperty, TransitionSpec,
};
use crate::widgets::UIWidgetState;
use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::color::Srgba;
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::image::ImageSampler;
use bevy::math::Rot2;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::RenderApp;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{
    NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_phase::{
    DrawFunctions, PhaseItem, SortedRenderPhase, ViewSortedRenderPhases,
};
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, Origin3d, ShaderType, TexelCopyTextureInfo, TextureAspect,
    TextureDimension, TextureFormat,
};
use bevy::render::texture::GpuImage;
use bevy::render::view::{ExtractedView, RetainedViewEntity, ViewTarget};
use bevy::shader::{Shader, ShaderRef};
use bevy::ui::{
    ComputedNode, ComputedUiRenderTargetInfo, UiGlobalTransform, UiSystems, UiTransform, Val2,
};
use bevy::ui_render::graph::NodeUi;
use bevy::ui_render::{DrawUiMaterial, TransparentUi, UiCameraView};
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage, PrimaryWindow, SystemCursorIcon};

const BACKDROP_BLUR_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("9d04a8bb-b6cf-4758-bca8-30706480973f");

/// Plugin that applies CSS styles, transitions, and animations to UI nodes.
pub struct StyleService;

impl Plugin for StyleService {
    /// Registers style update systems and resources.
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            BACKDROP_BLUR_SHADER_HANDLE,
            "../../assets/shaders/blur_shader.wgsl",
            Shader::from_wgsl
        );
        app.init_resource::<CssCursorState>();
        app.init_resource::<BackdropCaptureState>();
        app.add_plugins(ExtractResourcePlugin::<BackdropCaptureState>::default());
        app.add_plugins(UiMaterialPlugin::<BackdropBlurMaterial>::default());
        #[cfg(not(all(feature = "wasm-default", target_arch = "wasm32")))]
        app.add_systems(
            PostUpdate,
            mark_new_nodes_for_style_refresh.before(update_widget_styles_system),
        );
        app.add_systems(
            PostUpdate,
            (
                update_widget_styles_system.after(update_widget_states),
                update_style_transitions.after(update_widget_styles_system),
                update_style_animations.after(update_style_transitions),
                apply_calc_styles_system.after(update_style_animations),
                apply_background_gradients_system
                    .after(apply_calc_styles_system)
                    .after(UiSystems::Layout),
                apply_background_images_system
                    .after(apply_background_gradients_system)
                    .after(UiSystems::Layout),
                ensure_backdrop_capture_texture_system.after(apply_background_images_system),
                sync_backdrop_blur_materials_system
                    .after(ensure_backdrop_capture_texture_system)
                    .after(apply_background_images_system)
                    .after(UiSystems::Layout),
                propagate_style_inheritance.after(apply_calc_styles_system),
                sync_last_ui_transform.after(propagate_style_inheritance),
                update_css_cursor_icons.after(update_widget_styles_system),
            ),
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<DeferredBackdropUiPhases>()
            .add_systems(
                bevy::render::Render,
                split_backdrop_ui_phase_items_system
                    .in_set(bevy::render::RenderSystems::Prepare)
                    .after(bevy::ui_render::prepare_uimaterial_nodes::<BackdropBlurMaterial>),
            )
            .add_render_graph_node::<ViewNodeRunner<BackdropCaptureCopyNode>>(
                Core2d,
                BackdropCaptureCopyPass,
            )
            .add_render_graph_node::<ViewNodeRunner<BackdropDeferredDrawNode>>(
                Core2d,
                BackdropDeferredDrawPass,
            )
            .add_render_graph_edges(
                Core2d,
                (
                    NodeUi::UiPass,
                    BackdropCaptureCopyPass,
                    BackdropDeferredDrawPass,
                    Node2d::Upscaling,
                ),
            );
    }
}

/// Component storing an active style transition.
#[derive(Component, Debug, Clone)]
pub struct StyleTransition {
    pub from: Style,
    pub to: Style,
    pub start_time: f32,
    pub spec: TransitionSpec,
    pub from_transform: Option<UiTransform>,
    pub to_transform: Option<UiTransform>,
    pub current_style: Option<Style>,
}

/// Component caching the last computed UI transform.
#[derive(Component, Debug, Clone, Copy)]
pub struct LastUiTransform(pub UiTransform);

/// Component storing an active style animation.
#[derive(Component, Debug, Clone)]
pub struct StyleAnimation {
    pub base: Style,
    pub keyframes: Vec<AnimationKeyframe>,
    pub spec: AnimationSpec,
    pub start_time: f32,
    pub current_style: Option<Style>,
}

/// Resource tracking the currently applied CSS cursor state.
#[derive(Resource, Default)]
struct CssCursorState {
    active: bool,
    previous: Option<CursorIcon>,
}

/// Marker to force a style application pass when a UI node was just created.
#[derive(Component)]
pub(crate) struct StyleRefreshOnNodeAdded;

#[derive(ShaderType, Clone, Copy, Debug)]
struct BackdropBlurUniform {
    blur_radius_px: f32,
    overlay_alpha: f32,
    feedback_compensation: f32,
    viewport_size: Vec2,
    tint: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BackdropBlurMaterial {
    #[uniform(0)]
    uniform: BackdropBlurUniform,
    #[texture(1)]
    #[sampler(2)]
    screen_texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    overlay_texture: Handle<Image>,
}

impl UiMaterial for BackdropBlurMaterial {
    fn fragment_shader() -> ShaderRef {
        BACKDROP_BLUR_SHADER_HANDLE.into()
    }
}

#[derive(Resource, Default, Clone, ExtractResource)]
struct BackdropCaptureState {
    screen_texture: Option<Handle<Image>>,
    captured_size: UVec2,
    captured_format: Option<TextureFormat>,
    warmup_frames: u8,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct BackdropCaptureCopyPass;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct BackdropDeferredDrawPass;

#[derive(Default)]
struct BackdropCaptureCopyNode;

#[derive(Default)]
struct BackdropDeferredDrawNode;

#[derive(Resource, Default)]
struct DeferredBackdropUiPhases(HashMap<RetainedViewEntity, SortedRenderPhase<TransparentUi>>);

/// Convenience alias for mutable style-related UI component access.
type UiStyleComponents<'w, 's> = (
    Option<Mut<'w, Node>>,
    Option<Mut<'w, BackgroundColor>>,
    Option<Mut<'w, BorderColor>>,
    Option<Mut<'w, BoxShadow>>,
    Option<Mut<'w, TextColor>>,
    Option<Mut<'w, TextFont>>,
    Option<Mut<'w, TextLayout>>,
    Option<Mut<'w, ImageNode>>,
    Option<Mut<'w, ZIndex>>,
    Option<Mut<'w, Pickable>>,
    Option<Mut<'w, UiTransform>>,
);

/// Updates the OS cursor icon based on hovered widget styles.
fn update_css_cursor_icons(
    mut commands: Commands,
    mut cursor_state: ResMut<CssCursorState>,
    mut window_q: Query<(Entity, Option<&mut CursorIcon>), With<PrimaryWindow>>,
    hovered_q: Query<(&UiStyle, &UIWidgetState)>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok((window_entity, mut cursor_opt)) = window_q.single_mut() else {
        return;
    };

    let mut desired_cursor: Option<(CursorStyle, i32)> = None;
    let mut best_z = i32::MIN;

    for (ui_style, state) in hovered_q.iter() {
        if !state.hovered {
            continue;
        }

        let Some(active) = ui_style.active_style.as_ref() else {
            continue;
        };

        let Some(cursor) = active.cursor.clone() else {
            continue;
        };

        let z = active.z_index.unwrap_or(0);
        if desired_cursor.is_none() || z > best_z {
            desired_cursor = Some((cursor, z));
            best_z = z;
        }
    }

    if let Some((cursor_style, _)) = desired_cursor {
        let new_icon = match cursor_style {
            CursorStyle::System(system_icon) => CursorIcon::from(system_icon),
            CursorStyle::Custom(path) => {
                let handle =
                    get_or_load_image(path.as_str(), &mut image_cache, &mut images, &asset_server);
                if images.get(handle.id()).is_none() {
                    return;
                }
                CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                    handle,
                    hotspot: (0, 0),
                    ..default()
                }))
            }
        };
        if !cursor_state.active {
            cursor_state.previous = cursor_opt.as_deref().cloned();
            cursor_state.active = true;
        }

        if let Some(cursor) = cursor_opt.as_deref_mut() {
            if *cursor != new_icon {
                *cursor = new_icon;
            }
        } else {
            commands.entity(window_entity).insert(new_icon);
        }
    } else if cursor_state.active {
        cursor_state.active = false;
        let restore_icon = cursor_state
            .previous
            .take()
            .unwrap_or_else(|| CursorIcon::from(SystemCursorIcon::Default));

        if let Some(cursor) = cursor_opt.as_deref_mut() {
            if *cursor != restore_icon {
                *cursor = restore_icon;
            }
        } else {
            commands.entity(window_entity).insert(restore_icon);
        }
    }
}

#[cfg(not(all(feature = "wasm-default", target_arch = "wasm32")))]
fn mark_new_nodes_for_style_refresh(
    mut commands: Commands,
    query: Query<Entity, (Added<Node>, With<UiStyle>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(StyleRefreshOnNodeAdded);
    }
}

/// Computes active styles for widgets and applies them to UI components.
pub fn update_widget_styles_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            Option<&UIWidgetState>,
            Option<&HtmlStyle>,
            Option<&StyleRefreshOnNodeAdded>,
            &mut UiStyle,
        ),
        Or<(
            Changed<UiStyle>,
            Changed<HtmlStyle>,
            Changed<UIWidgetState>,
            Added<StyleRefreshOnNodeAdded>,
        )>,
    >,
    mut transition_query: Query<Option<&mut StyleTransition>>,
    mut animation_query: Query<Option<&mut StyleAnimation>>,
    mut qs: ParamSet<(Query<UiStyleComponents>,)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, state_opt, html_style_opt, refresh_on_node_added, mut ui_style) in query.iter_mut()
    {
        let state = state_opt.cloned().unwrap_or_default();
        let node_added = refresh_on_node_added.is_some();

        let mut base_styles: Vec<(&String, u32, usize)> = vec![];
        let mut pseudo_styles: Vec<(&String, u32, usize)> = vec![];

        for (key, style_pair) in &ui_style.styles {
            let selector = if style_pair.selector.is_empty() {
                key.as_str()
            } else {
                style_pair.selector.as_str()
            };

            if selector.contains("::") {
                continue;
            }
            if selector_matches_state(selector, &state) {
                let specificity = selector_specificity(selector);
                if selector.contains(':') {
                    pseudo_styles.push((key, specificity, style_pair.origin));
                } else {
                    base_styles.push((key, specificity, style_pair.origin));
                }
            }
        }

        // Sort by origin (ascending) then specificity (ascending)
        // Later origin overrides earlier. Higher specificity overrides lower.
        sort_style_candidates(&mut base_styles);
        sort_style_candidates(&mut pseudo_styles);

        let mut final_style = Style::default();

        // 1) base normal
        merge_style_candidates(&mut final_style, &ui_style, &base_styles, false);

        // 2) base important
        merge_style_candidates(&mut final_style, &ui_style, &base_styles, true);

        // 3) inline html
        if let Some(html_style) = html_style_opt {
            final_style.merge(&html_style.0);
        }

        // 4) pseudo normal
        merge_style_candidates(&mut final_style, &ui_style, &pseudo_styles, false);

        // 5) pseudo important
        merge_style_candidates(&mut final_style, &ui_style, &pseudo_styles, true);

        let previous_style = ui_style.active_style.clone();
        let has_changed = previous_style.as_ref() != Some(&final_style);
        if has_changed {
            ui_style.active_style = Some(final_style.clone());
        }

        update_style_animation_state(
            &mut commands,
            entity,
            &final_style,
            &ui_style.keyframes,
            time.elapsed_secs(),
            &mut animation_query,
        );

        let mut transition = transition_query.get_mut(entity).ok().flatten();
        let should_transition =
            has_changed && final_style.transition.is_some() && previous_style.is_some();

        if should_transition {
            let spec = final_style.transition.clone().unwrap_or_default();
            let from = previous_style.unwrap_or_default();
            let to = final_style.clone();
            let copy_spec = spec.clone();

            let (from_transform, to_transform) = resolve_transform_transition(&spec, &from, &to);

            let transition_state = StyleTransition {
                from,
                to,
                start_time: time.elapsed_secs(),
                spec,
                from_transform,
                to_transform,
                current_style: None,
            };

            if let Some(existing) = transition.as_mut() {
                **existing = transition_state;
            } else {
                commands.entity(entity).insert(transition_state);
            }

            apply_transform_style_if_blocked(&mut qs, entity, &final_style, &copy_spec);
            if node_added {
                commands.entity(entity).remove::<StyleRefreshOnNodeAdded>();
            }
            continue;
        }

        if let Some(transition) = transition.as_mut() {
            if !has_changed {
                if node_added {
                    commands.entity(entity).remove::<StyleRefreshOnNodeAdded>();
                }
                continue;
            }

            transition.from = previous_style.unwrap_or_default();
            transition.to = final_style.clone();
            transition.start_time = time.elapsed_secs();
            transition.spec = final_style.transition.clone().unwrap_or_default();

            let (from_transform, to_transform) =
                resolve_transform_transition(&transition.spec, &transition.from, &transition.to);
            transition.from_transform = from_transform;
            transition.to_transform = to_transform;

            apply_transform_style_if_blocked(&mut qs, entity, &final_style, &transition.spec);
            if node_added {
                commands.entity(entity).remove::<StyleRefreshOnNodeAdded>();
            }
            continue;
        }

        if !has_changed && !node_added {
            continue;
        }

        if let Ok(mut components) = qs.p0().get_mut(entity) {
            apply_style_components(
                &final_style,
                &mut components,
                &asset_server,
                &mut image_cache,
                &mut images,
            );
        }

        if node_added {
            commands.entity(entity).remove::<StyleRefreshOnNodeAdded>();
        }
    }
}

/// Advances and applies style transitions based on time.
pub fn update_style_transitions(
    mut commands: Commands,
    time: Res<Time>,
    mut transitions: Query<(Entity, &mut StyleTransition)>,
    mut style_query: Query<UiStyleComponents>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let now = time.elapsed_secs();

    for (entity, mut transition) in transitions.iter_mut() {
        let elapsed = now - transition.start_time - transition.spec.delay;
        let duration = transition.spec.duration.max(0.001);
        let t = (elapsed / duration).clamp(0.0, 1.0);
        let eased = transition.spec.timing.apply(t);
        let blended = blend_style(&transition.from, &transition.to, eased, &transition.spec);
        transition.current_style = Some(blended.clone());

        if let Ok(mut components) = style_query.get_mut(entity) {
            apply_style_components(
                &blended,
                &mut components,
                &asset_server,
                &mut image_cache,
                &mut images,
            );

            if transition_allows_transform(&transition.spec) {
                if let (Some(from), Some(to)) = (transition.from_transform, transition.to_transform)
                {
                    if let Some(transform) = components.10.as_mut() {
                        **transform = blend_ui_transform(from, to, eased);
                    }
                }
            }

            if elapsed >= duration {
                apply_style_components(
                    &transition.to,
                    &mut components,
                    &asset_server,
                    &mut image_cache,
                    &mut images,
                );

                if transition_allows_transform(&transition.spec) {
                    if let Some(target) = transition.to_transform {
                        if let Some(transform) = components.10.as_mut() {
                            **transform = target;
                        }
                    }
                }

                commands.entity(entity).remove::<StyleTransition>();
            }
        }
    }
}

/// Advances and applies style animations based on time.
pub fn update_style_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut animations: Query<(Entity, &mut StyleAnimation)>,
    mut style_query: Query<UiStyleComponents>,
    ui_style_query: Query<&UiStyle>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let now = time.elapsed_secs();

    for (entity, mut animation) in animations.iter_mut() {
        let desired_style = ui_style_query
            .get(entity)
            .ok()
            .and_then(|ui| ui.active_style.as_ref());

        let desired_animation_name = desired_style
            .and_then(|s| s.animation.as_ref())
            .map(|a| a.name.as_str());

        if desired_animation_name != Some(animation.spec.name.as_str()) {
            if let (Some(style), Ok(mut components)) = (desired_style, style_query.get_mut(entity))
            {
                apply_style_components(
                    style,
                    &mut components,
                    &asset_server,
                    &mut image_cache,
                    &mut images,
                );
            }
            commands.entity(entity).remove::<StyleAnimation>();
            continue;
        }

        let duration = animation.spec.duration.max(0.001);
        let elapsed = now - animation.start_time - animation.spec.delay;

        if elapsed < 0.0 {
            continue;
        }

        if let Some(iterations) = animation.spec.iterations {
            if iterations <= 0.0 {
                if let Ok(mut components) = style_query.get_mut(entity) {
                    apply_style_components(
                        &animation.base,
                        &mut components,
                        &asset_server,
                        &mut image_cache,
                        &mut images,
                    );
                }
                commands.entity(entity).remove::<StyleAnimation>();
                continue;
            }

            let total = duration * iterations;
            if elapsed >= total {
                let final_cycle = (iterations - 1.0).max(0.0).floor() as u32;
                let progress = animation_progress(&animation.spec, final_cycle, 1.0);
                if let Ok(mut components) = style_query.get_mut(entity) {
                    let blended = sample_animation_style(&animation.keyframes, progress);
                    apply_style_components(
                        &blended,
                        &mut components,
                        &asset_server,
                        &mut image_cache,
                        &mut images,
                    );
                }
                commands.entity(entity).remove::<StyleAnimation>();
                continue;
            }
        }

        let cycles = (elapsed / duration).floor().max(0.0) as u32;
        let cycle_progress = (elapsed / duration).fract();
        let progress = animation_progress(&animation.spec, cycles, cycle_progress);

        if let Ok(mut components) = style_query.get_mut(entity) {
            let blended = sample_animation_style(&animation.keyframes, progress);
            animation.current_style = Some(blended.clone());
            apply_style_components(
                &blended,
                &mut components,
                &asset_server,
                &mut image_cache,
                &mut images,
            );
        }
    }
}

fn apply_calc_styles_system(
    mut query: Query<(
        Entity,
        &UiStyle,
        Option<&StyleTransition>,
        Option<&StyleAnimation>,
        Option<&ChildOf>,
        Option<&mut Node>,
    )>,
    computed_query: Query<&ComputedNode>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let viewport = resolve_layout_viewport(&window_q).unwrap_or_default();

    for (_entity, ui_style, transition_opt, animation_opt, parent_opt, node_opt) in query.iter_mut()
    {
        let Some(mut node) = node_opt else {
            continue;
        };

        let style = if let Some(transition) = transition_opt {
            transition.current_style.as_ref().unwrap_or(&transition.to)
        } else if let Some(animation) = animation_opt {
            animation.current_style.as_ref().unwrap_or(&animation.base)
        } else {
            let Some(active) = ui_style.active_style.as_ref() else {
                continue;
            };
            active
        };

        let (content_w, content_h, box_w, box_h) = if let Some(parent) = parent_opt {
            if let Ok(parent_node) = computed_query.get(parent.parent()) {
                // ComputedNode sizes are in physical pixels; convert to logical for Val::Px.
                let inv_sf = parent_node.inverse_scale_factor.max(f32::EPSILON);
                let size = parent_node.size;
                let border = parent_node.border;
                let padding = parent_node.padding;
                let box_size = Vec2::new(
                    shrink_axis(size.x, border.min_inset.x, border.max_inset.x),
                    shrink_axis(size.y, border.min_inset.y, border.max_inset.y),
                );
                let content = Vec2::new(
                    shrink_axis(box_size.x, padding.min_inset.x, padding.max_inset.x),
                    shrink_axis(box_size.y, padding.min_inset.y, padding.max_inset.y),
                );
                let content = content * inv_sf;
                let box_size = box_size * inv_sf;
                (content.x, content.y, box_size.x, box_size.y)
            } else {
                (viewport.x, viewport.y, viewport.x, viewport.y)
            }
        } else {
            (viewport.x, viewport.y, viewport.x, viewport.y)
        };

        let ctx_content_w = CalcContext {
            base: content_w,
            viewport,
        };
        let ctx_content_h = CalcContext {
            base: content_h,
            viewport,
        };
        let ctx_box_w = CalcContext {
            base: box_w,
            viewport,
        };
        let ctx_box_h = CalcContext {
            base: box_h,
            viewport,
        };

        apply_calc_length(style.width_calc.as_ref(), ctx_content_w, &mut node.width);
        apply_calc_length(
            style.min_width_calc.as_ref(),
            ctx_content_w,
            &mut node.min_width,
        );
        apply_calc_length(
            style.max_width_calc.as_ref(),
            ctx_content_w,
            &mut node.max_width,
        );
        apply_calc_length(style.height_calc.as_ref(), ctx_content_h, &mut node.height);
        apply_calc_length(
            style.min_height_calc.as_ref(),
            ctx_content_h,
            &mut node.min_height,
        );
        apply_calc_length(
            style.max_height_calc.as_ref(),
            ctx_content_h,
            &mut node.max_height,
        );

        apply_calc_length(style.left_calc.as_ref(), ctx_box_w, &mut node.left);
        apply_calc_length(style.right_calc.as_ref(), ctx_box_w, &mut node.right);
        apply_calc_length(style.top_calc.as_ref(), ctx_box_h, &mut node.top);
        apply_calc_length(style.bottom_calc.as_ref(), ctx_box_h, &mut node.bottom);

        if let Some(expr) = style.flex_basis_calc.as_ref() {
            let base_main = match node.flex_direction {
                FlexDirection::Row | FlexDirection::RowReverse => content_w,
                _ => content_h,
            };
            let ctx_main = CalcContext {
                base: base_main,
                viewport,
            };
            if let Some(px) = expr.eval_length(ctx_main) {
                node.flex_basis = Val::Px(px);
            }
        }

        let mut row_gap_val = None;
        let mut column_gap_val = None;

        if let Some(expr) = style.row_gap_calc.as_ref() {
            if let Some(px) = expr.eval_length(ctx_content_w) {
                row_gap_val = Some(Val::Px(px));
            }
        }

        if let Some(expr) = style.column_gap_calc.as_ref() {
            if let Some(px) = expr.eval_length(ctx_content_w) {
                column_gap_val = Some(Val::Px(px));
            }
        }

        let needs_row_gap = style.row_gap.is_none() && row_gap_val.is_none();
        let needs_column_gap = style.column_gap.is_none() && column_gap_val.is_none();

        if (needs_row_gap || needs_column_gap) && style.gap_calc.is_some() {
            if let Some(expr) = style.gap_calc.as_ref() {
                if let Some(px) = expr.eval_length(ctx_content_w) {
                    let gap_val = Val::Px(px);
                    if needs_row_gap {
                        row_gap_val = Some(gap_val);
                    }
                    if needs_column_gap {
                        column_gap_val = Some(gap_val);
                    }
                }
            }
        }

        if let Some(val) = row_gap_val {
            node.row_gap = val;
        }
        if let Some(val) = column_gap_val {
            node.column_gap = val;
        }
    }
}

#[cfg(all(feature = "wasm-default", target_arch = "wasm32"))]
fn resolve_layout_viewport(window_q: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    let window = window_q.single().ok()?;
    Some(window.resolution.size())
}

#[cfg(all(
    feature = "wasm-breakpoints",
    not(feature = "wasm-default"),
    target_arch = "wasm32"
))]
fn resolve_layout_viewport(_window_q: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    let window = web_sys::window()?;
    let width = window.inner_width().ok()?.as_f64()? as f32;
    let height = window.inner_height().ok()?.as_f64()? as f32;
    Some(Vec2::new(width, height))
}

#[cfg(all(feature = "wasm-breakpoints", not(target_arch = "wasm32")))]
fn resolve_layout_viewport(window_q: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    let window = window_q.single().ok()?;
    Some(window.resolution.size())
}

#[cfg(all(not(feature = "wasm-breakpoints"), feature = "css-breakpoints"))]
fn resolve_layout_viewport(window_q: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    let window = window_q.single().ok()?;
    Some(window.resolution.size())
}

#[cfg(all(not(feature = "wasm-breakpoints"), not(feature = "css-breakpoints")))]
fn resolve_layout_viewport(_window_q: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    None
}

fn apply_background_gradients_system(
    mut query: Query<(
        Entity,
        &UiStyle,
        Option<&StyleTransition>,
        Option<&StyleAnimation>,
        &ComputedNode,
        Option<&mut ImageNode>,
    )>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for (_entity, ui_style, transition_opt, animation_opt, computed, img_node_opt) in
        query.iter_mut()
    {
        let Some(mut img_node) = img_node_opt else {
            continue;
        };

        let style = if let Some(transition) = transition_opt {
            transition.current_style.as_ref().unwrap_or(&transition.to)
        } else if let Some(animation) = animation_opt {
            animation.current_style.as_ref().unwrap_or(&animation.base)
        } else {
            let Some(active) = ui_style.active_style.as_ref() else {
                continue;
            };
            active
        };

        let Some(background) = style.background.as_ref() else {
            continue;
        };
        let Some(gradient) = background.gradient.as_ref() else {
            continue;
        };

        if img_node.color != Color::WHITE {
            img_node.color = Color::WHITE;
        }

        let size = computed.size;
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }

        let width = size.x.round().max(1.0) as u32;
        let height = size.y.round().max(1.0) as u32;
        let size = UVec2::new(width, height);
        let inv_sf = computed.inverse_scale_factor.max(f32::EPSILON);
        let scale_factor = inv_sf.recip();

        let key = gradient_cache_key(gradient, size);
        let handle = if let Some(handle) = image_cache.map.get(&key) {
            handle.clone()
        } else {
            let image = render_linear_gradient_image(gradient, size, scale_factor);
            let handle = images.add(image);
            image_cache.map.insert(key, handle.clone());
            handle
        };

        if img_node.image != handle {
            img_node.image = handle;
        }
    }
}

fn apply_background_images_system(
    mut query: Query<(
        Entity,
        &UiStyle,
        Option<&StyleTransition>,
        Option<&StyleAnimation>,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        Option<&UiGlobalTransform>,
        Option<&mut ImageNode>,
    )>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for (
        _entity,
        ui_style,
        transition_opt,
        animation_opt,
        computed,
        render_target,
        global_transform,
        img_node_opt,
    ) in query.iter_mut()
    {
        let Some(mut img_node) = img_node_opt else {
            continue;
        };

        let style = if let Some(transition) = transition_opt {
            transition.current_style.as_ref().unwrap_or(&transition.to)
        } else if let Some(animation) = animation_opt {
            animation.current_style.as_ref().unwrap_or(&animation.base)
        } else {
            let Some(active) = ui_style.active_style.as_ref() else {
                continue;
            };
            active
        };

        let Some(background) = style.background.as_ref() else {
            continue;
        };
        if background.gradient.is_some() {
            continue;
        }
        let Some(image_path) = background.image.as_ref() else {
            continue;
        };

        if img_node.color != Color::WHITE {
            img_node.color = Color::WHITE;
        }

        let size = computed.size;
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }

        let container_size = UVec2::new(
            size.x.round().max(1.0) as u32,
            size.y.round().max(1.0) as u32,
        );
        let inv_sf = computed.inverse_scale_factor.max(f32::EPSILON);
        let scale_factor = inv_sf.recip();

        let attachment = style.background_attachment.clone().unwrap_or_default();
        let position = style.background_position.clone().unwrap_or_default();
        let bg_size = style.background_size.clone().unwrap_or_default();

        let source_handle = get_or_load_image(
            image_path.as_str(),
            &mut image_cache,
            &mut images,
            &asset_server,
        );
        let Some(source_image) = images.get(source_handle.id()) else {
            continue;
        };

        let source_size = source_image.size();
        if source_size.x == 0 || source_size.y == 0 {
            continue;
        }

        let viewport_size = render_target.physical_size();
        let viewport_size = if viewport_size.x == 0 || viewport_size.y == 0 {
            container_size
        } else {
            viewport_size
        };
        let positioning_size = if matches!(attachment, BackgroundAttachment::Fixed) {
            viewport_size
        } else {
            container_size
        };

        let draw_size =
            resolve_background_draw_size(&bg_size, source_size, positioning_size, scale_factor);
        if draw_size.x == 0 || draw_size.y == 0 {
            continue;
        }

        let mut offset =
            resolve_background_position(&position, positioning_size, draw_size, scale_factor);
        if matches!(attachment, BackgroundAttachment::Fixed) {
            if let Some(transform) = global_transform {
                let half = size * 0.5;
                let top_left = transform.affine().transform_point2(-half);
                offset -= top_left;
            }
        } else if matches!(attachment, BackgroundAttachment::Local) {
            offset -= computed.scroll_position;
        }

        let cache_key = background_image_cache_key(
            &source_handle,
            container_size,
            draw_size,
            offset,
            attachment,
        );
        let handle = if let Some(handle) = image_cache.map.get(&cache_key) {
            handle.clone()
        } else {
            let Some(image) =
                render_background_image(source_image, container_size, draw_size, offset)
            else {
                continue;
            };
            let handle = images.add(image);
            image_cache.map.insert(cache_key, handle.clone());
            handle
        };

        if img_node.image != handle {
            img_node.image = handle;
        }
    }
}

fn resolved_active_style<'a>(
    ui_style: &'a UiStyle,
    transition_opt: Option<&'a StyleTransition>,
    animation_opt: Option<&'a StyleAnimation>,
) -> Option<&'a Style> {
    if let Some(transition) = transition_opt {
        return Some(transition.current_style.as_ref().unwrap_or(&transition.to));
    }
    if let Some(animation) = animation_opt {
        return Some(animation.current_style.as_ref().unwrap_or(&animation.base));
    }
    ui_style.active_style.as_ref()
}

fn sync_backdrop_blur_materials_system(
    mut commands: Commands,
    window_q: Query<&Window, With<PrimaryWindow>>,
    query: Query<(
        Entity,
        &UiStyle,
        Option<&StyleTransition>,
        Option<&StyleAnimation>,
        Option<&ImageNode>,
        Option<&MaterialNode<BackdropBlurMaterial>>,
    )>,
    capture_state: Res<BackdropCaptureState>,
    mut materials: ResMut<Assets<BackdropBlurMaterial>>,
) {
    let viewport_size = window_q
        .single()
        .map(|w| w.physical_size().as_vec2())
        .unwrap_or(Vec2::ONE)
        .max(Vec2::ONE);
    for (entity, ui_style, transition_opt, animation_opt, image_node_opt, material_node_opt) in
        query.iter()
    {
        let Some(style) = resolved_active_style(ui_style, transition_opt, animation_opt) else {
            if material_node_opt.is_some() {
                commands
                    .entity(entity)
                    .remove::<MaterialNode<BackdropBlurMaterial>>();
            }
            continue;
        };

        let blur_radius = match style.backdrop_filter.as_ref() {
            Some(BackdropFilter::Blur(radius)) if *radius > 0.0 => *radius,
            _ => {
                if material_node_opt.is_some() {
                    commands
                        .entity(entity)
                        .remove::<MaterialNode<BackdropBlurMaterial>>();
                }
                continue;
            }
        };

        let tint = style
            .background
            .as_ref()
            .map(|background| background.color.to_linear().to_vec4())
            .unwrap_or(Vec4::new(1.0, 1.0, 1.0, 0.0));
        let has_overlay = style
            .background
            .as_ref()
            .map(|background| background.gradient.is_some() || background.image.is_some())
            .unwrap_or(false);
        let uniform = BackdropBlurUniform {
            blur_radius_px: blur_radius,
            overlay_alpha: if has_overlay { 1.0 } else { 0.0 },
            // The capture texture contains previously composited content.
            // Keep compensation enabled to reduce tint feedback drift.
            feedback_compensation: 1.0,
            viewport_size,
            tint,
        };

        let texture_ready =
            capture_state.screen_texture.is_some() && capture_state.warmup_frames == 0;
        if !texture_ready && material_node_opt.is_none() {
            continue;
        }

        if let Some(material_node) = material_node_opt {
            if let Some(material) = materials.get_mut(material_node.id()) {
                material.uniform = uniform;
                if let Some(screen_texture) = capture_state.screen_texture.clone() {
                    let overlay_texture = if has_overlay {
                        image_node_opt
                            .map(|img| img.image.clone())
                            .unwrap_or_else(|| screen_texture.clone())
                    } else {
                        screen_texture.clone()
                    };
                    material.screen_texture = screen_texture;
                    material.overlay_texture = overlay_texture;
                }
            } else if let Some(screen_texture) = capture_state.screen_texture.clone() {
                let overlay_texture = if has_overlay {
                    image_node_opt
                        .map(|img| img.image.clone())
                        .unwrap_or_else(|| screen_texture.clone())
                } else {
                    screen_texture.clone()
                };
                let handle = materials.add(BackdropBlurMaterial {
                    uniform,
                    screen_texture,
                    overlay_texture,
                });
                commands.entity(entity).insert(MaterialNode(handle));
            }
            continue;
        }

        let Some(screen_texture) = capture_state.screen_texture.clone() else {
            continue;
        };
        let overlay_texture = if has_overlay {
            image_node_opt
                .map(|img| img.image.clone())
                .unwrap_or_else(|| screen_texture.clone())
        } else {
            screen_texture.clone()
        };
        let handle = materials.add(BackdropBlurMaterial {
            uniform,
            screen_texture,
            overlay_texture,
        });
        commands.entity(entity).insert(MaterialNode(handle));
    }
}

fn ensure_backdrop_capture_texture_system(
    window_q: Query<&Window, With<PrimaryWindow>>,
    blur_query: Query<(&UiStyle, Option<&StyleTransition>, Option<&StyleAnimation>)>,
    configuration: Res<ExtendedUiConfiguration>,
    mut capture_state: ResMut<BackdropCaptureState>,
    mut images: ResMut<Assets<Image>>,
) {
    let has_backdrop_blur = blur_query
        .iter()
        .any(|(ui_style, transition_opt, animation_opt)| {
            resolved_active_style(ui_style, transition_opt, animation_opt).is_some_and(|style| {
                matches!(
                    style.backdrop_filter.as_ref(),
                    Some(BackdropFilter::Blur(radius)) if *radius > 0.0
                )
            })
        });

    if !has_backdrop_blur {
        capture_state.screen_texture = None;
        capture_state.captured_size = UVec2::ZERO;
        capture_state.captured_format = None;
        capture_state.warmup_frames = 0;
        return;
    }

    let Ok(window) = window_q.single() else {
        return;
    };
    let window_size = window.physical_size();
    if window_size == UVec2::ZERO {
        return;
    }

    let texture_format = if configuration.hdr_support {
        TextureFormat::Rgba16Float
    } else {
        TextureFormat::Rgba8UnormSrgb
    };

    if capture_state.captured_size == window_size
        && capture_state.captured_format == Some(texture_format)
    {
        capture_state.warmup_frames = capture_state.warmup_frames.saturating_sub(1);
        return;
    }

    let reuse_existing_texture = capture_state.screen_texture.is_some();
    let mut image = Image::new_uninit(
        Extent3d {
            width: window_size.x,
            height: window_size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_format,
        RenderAssetUsages::default(),
    );
    image.copy_on_resize = reuse_existing_texture;
    image.sampler = ImageSampler::linear();

    let handle = if let Some(handle) = capture_state.screen_texture.clone() {
        if let Some(existing) = images.get_mut(handle.id()) {
            *existing = image;
            handle
        } else {
            images.add(image)
        }
    } else {
        images.add(image)
    };

    capture_state.screen_texture = Some(handle);
    capture_state.captured_size = window_size;
    capture_state.captured_format = Some(texture_format);
    // Keep the regular translucent background for one frame until the first GPU copy happened.
    capture_state.warmup_frames = 1;
}

impl ViewNode for BackdropCaptureCopyNode {
    type ViewQuery = (
        Entity,
        &'static ExtractedCamera,
        &'static ViewTarget,
        Option<&'static UiCameraView>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        (view_entity, camera, view_target, ui_camera_view): bevy::ecs::query::QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let ui_view_entity = ui_camera_view.map(|v| v.0).unwrap_or(view_entity);
        let Some(ui_view) = world.get::<ExtractedView>(ui_view_entity) else {
            return Ok(());
        };
        let deferred_phases = world.resource::<DeferredBackdropUiPhases>();
        let Some(phase) = deferred_phases.0.get(&ui_view.retained_view_entity) else {
            return Ok(());
        };
        if phase.items.is_empty() {
            return Ok(());
        }

        let Some(target_size) = camera.physical_target_size else {
            return Ok(());
        };
        if target_size == UVec2::ZERO {
            return Ok(());
        }

        let capture_state = world.resource::<BackdropCaptureState>();
        let Some(screen_texture) = capture_state.screen_texture.clone() else {
            return Ok(());
        };

        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let Some(gpu_image) = gpu_images.get(screen_texture.id()) else {
            return Ok(());
        };

        if view_target.main_texture_format() != gpu_image.texture_format {
            return Ok(());
        }

        let copy_size = Extent3d {
            width: target_size.x.min(gpu_image.size.width),
            height: target_size.y.min(gpu_image.size.height),
            depth_or_array_layers: 1,
        };
        if copy_size.width == 0 || copy_size.height == 0 {
            return Ok(());
        }

        render_context.command_encoder().copy_texture_to_texture(
            TexelCopyTextureInfo {
                texture: view_target.main_texture(),
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyTextureInfo {
                texture: &gpu_image.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            copy_size,
        );

        Ok(())
    }
}

fn split_backdrop_ui_phase_items_system(
    mut phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
    draw_functions: Res<DrawFunctions<TransparentUi>>,
    mut deferred_phases: ResMut<DeferredBackdropUiPhases>,
) {
    deferred_phases.0.clear();
    let backdrop_draw_function = draw_functions
        .read()
        .id::<DrawUiMaterial<BackdropBlurMaterial>>();

    for (retained_view, phase) in phases.0.iter_mut() {
        if phase.items.is_empty() {
            continue;
        }

        // Keep all items before the first backdrop material in the regular UI pass.
        // Defer everything from the first backdrop onward so text / overlays that
        // should appear above blur remain above it in the deferred pass ordering.
        let first_backdrop_index = phase
            .items
            .iter()
            .position(|item| item.draw_function() == backdrop_draw_function);
        let Some(first_backdrop_index) = first_backdrop_index else {
            continue;
        };

        let deferred = phase.items.split_off(first_backdrop_index);
        if !deferred.is_empty() {
            deferred_phases
                .0
                .insert(*retained_view, SortedRenderPhase { items: deferred });
        }
    }
}

impl ViewNode for BackdropDeferredDrawNode {
    type ViewQuery = (
        Entity,
        &'static ViewTarget,
        &'static ExtractedCamera,
        Option<&'static UiCameraView>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        (view_entity, target, camera, ui_camera_view): bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let ui_view_entity = ui_camera_view.map(|v| v.0).unwrap_or(view_entity);

        let Some(ui_view) = world.get::<ExtractedView>(ui_view_entity) else {
            return Ok(());
        };
        let deferred_phases = world.resource::<DeferredBackdropUiPhases>();
        let Some(phase) = deferred_phases.0.get(&ui_view.retained_view_entity) else {
            return Ok(());
        };
        if phase.items.is_empty() {
            return Ok(());
        }

        let mut render_pass = render_context.begin_tracked_render_pass(
            bevy::render::render_resource::RenderPassDescriptor {
                label: Some("backdrop_deferred_ui"),
                color_attachments: &[Some(target.get_unsampled_color_attachment())],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            },
        );
        if let Some(viewport) = camera.viewport.as_ref() {
            render_pass.set_camera_viewport(viewport);
        }

        let _ = phase.render(&mut render_pass, world, ui_view_entity);
        Ok(())
    }
}

type StyleCandidate<'a> = (&'a String, u32, usize);

fn sort_style_candidates(candidates: &mut [StyleCandidate<'_>]) {
    candidates.sort_by(|a, b| match a.2.cmp(&b.2) {
        std::cmp::Ordering::Equal => a.1.cmp(&b.1),
        other => other,
    });
}

fn merge_style_candidates(
    final_style: &mut Style,
    ui_style: &UiStyle,
    candidates: &[StyleCandidate<'_>],
    important: bool,
) {
    for (sel, _, _) in candidates {
        if let Some(pair) = ui_style.styles.get(*sel) {
            if important {
                final_style.merge(&pair.important);
            } else {
                final_style.merge(&pair.normal);
            }
        }
    }
}

fn apply_transform_style_if_blocked(
    qs: &mut ParamSet<(Query<UiStyleComponents>,)>,
    entity: Entity,
    style: &Style,
    spec: &TransitionSpec,
) {
    if transition_allows_transform(spec) {
        return;
    }

    if let Ok(mut components) = qs.p0().get_mut(entity) {
        if let Some(transform) = components.10.as_mut() {
            apply_transform_style(style, transform);
        }
    }
}

fn shrink_axis(size: f32, min_inset: f32, max_inset: f32) -> f32 {
    (size - min_inset - max_inset).max(0.0)
}

fn gradient_cache_key(gradient: &LinearGradient, size: UVec2) -> String {
    let mut key = format!("__linear-gradient__:{:.4}", gradient.angle);
    for stop in &gradient.stops {
        let color = stop.color.to_srgba();
        let r = (color.red * 255.0).round() as u8;
        let g = (color.green * 255.0).round() as u8;
        let b = (color.blue * 255.0).round() as u8;
        let a = (color.alpha * 255.0).round() as u8;
        key.push_str(&format!(":{r:02x}{g:02x}{b:02x}{a:02x}"));
        match stop.position {
            Some(GradientStopPosition::Percent(value)) => {
                key.push_str(&format!("@{value:.4}%"));
            }
            Some(GradientStopPosition::Px(value)) => {
                key.push_str(&format!("@{value:.4}px"));
            }
            None => key.push_str("@auto"),
        }
    }
    key.push_str(&format!(":{}x{}", size.x, size.y));
    key
}

fn background_image_cache_key(
    source: &Handle<Image>,
    container_size: UVec2,
    draw_size: UVec2,
    offset: Vec2,
    attachment: BackgroundAttachment,
) -> String {
    let id = format!("{:?}", source.id());
    let offset_x = offset.x.round();
    let offset_y = offset.y.round();
    format!(
        "__background-image__:{id}:{}x{}:{}x{}:{offset_x:.1}:{offset_y:.1}:{attachment:?}",
        container_size.x, container_size.y, draw_size.x, draw_size.y
    )
}

fn resolve_background_draw_size(
    size: &BackgroundSize,
    source: UVec2,
    area: UVec2,
    scale_factor: f32,
) -> UVec2 {
    let source_w = source.x.max(1) as f32;
    let source_h = source.y.max(1) as f32;
    let area_w = area.x.max(1) as f32;
    let area_h = area.y.max(1) as f32;

    let (target_w, target_h) = match size {
        BackgroundSize::Auto => (source_w, source_h),
        BackgroundSize::Cover => {
            let scale = (area_w / source_w).max(area_h / source_h);
            (source_w * scale, source_h * scale)
        }
        BackgroundSize::Contain => {
            let scale = (area_w / source_w).min(area_h / source_h);
            (source_w * scale, source_h * scale)
        }
        BackgroundSize::Explicit(width, height) => {
            let mut w = resolve_background_size_value(width, area_w, scale_factor);
            let mut h = resolve_background_size_value(height, area_h, scale_factor);
            if w.is_none() && h.is_none() {
                w = Some(source_w);
                h = Some(source_h);
            } else if w.is_none() {
                let ratio = source_w / source_h;
                w = Some(h.unwrap_or(source_h) * ratio);
            } else if h.is_none() {
                let ratio = source_h / source_w;
                h = Some(w.unwrap_or(source_w) * ratio);
            }
            (w.unwrap_or(source_w), h.unwrap_or(source_h))
        }
    };

    UVec2::new(
        target_w.max(1.0).round() as u32,
        target_h.max(1.0).round() as u32,
    )
}

fn resolve_background_size_value(
    value: &BackgroundSizeValue,
    area: f32,
    scale_factor: f32,
) -> Option<f32> {
    match value {
        BackgroundSizeValue::Auto => None,
        BackgroundSizeValue::Percent(percent) => Some(area * percent / 100.0),
        BackgroundSizeValue::Px(px) => Some(px * scale_factor),
    }
}

fn resolve_background_position(
    position: &BackgroundPosition,
    area: UVec2,
    image: UVec2,
    scale_factor: f32,
) -> Vec2 {
    let area_w = area.x as f32;
    let area_h = area.y as f32;
    let image_w = image.x as f32;
    let image_h = image.y as f32;

    let x = resolve_background_position_axis(&position.x, area_w, image_w, scale_factor);
    let y = resolve_background_position_axis(&position.y, area_h, image_h, scale_factor);
    Vec2::new(x, y)
}

fn resolve_background_position_axis(
    value: &BackgroundPositionValue,
    area: f32,
    image: f32,
    scale_factor: f32,
) -> f32 {
    match value {
        BackgroundPositionValue::Percent(percent) => (area - image) * percent / 100.0,
        BackgroundPositionValue::Px(px) => px * scale_factor,
    }
}

fn render_background_image(
    source: &Image,
    container_size: UVec2,
    draw_size: UVec2,
    offset: Vec2,
) -> Option<Image> {
    let source_format = source.texture_descriptor.format;
    let supported = matches!(
        source_format,
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb
    );
    if !supported {
        return None;
    }

    let data = source.data.as_ref()?;
    let source_size = source.size();
    let src_w = source_size.x as usize;
    let src_h = source_size.y as usize;
    if src_w == 0 || src_h == 0 {
        return None;
    }

    let out_w = container_size.x as usize;
    let out_h = container_size.y as usize;
    let mut out = vec![0u8; out_w * out_h * 4];

    let draw_w = draw_size.x as usize;
    let draw_h = draw_size.y as usize;
    if draw_w == 0 || draw_h == 0 {
        return None;
    }

    let offset_x = offset.x.round() as i32;
    let offset_y = offset.y.round() as i32;

    for y in 0..draw_h {
        let dest_y = offset_y + y as i32;
        if dest_y < 0 || dest_y >= out_h as i32 {
            continue;
        }
        let src_y = ((y as f32 + 0.5) * src_h as f32 / draw_h as f32)
            .floor()
            .clamp(0.0, (src_h - 1) as f32) as usize;

        for x in 0..draw_w {
            let dest_x = offset_x + x as i32;
            if dest_x < 0 || dest_x >= out_w as i32 {
                continue;
            }
            let src_x = ((x as f32 + 0.5) * src_w as f32 / draw_w as f32)
                .floor()
                .clamp(0.0, (src_w - 1) as f32) as usize;

            let src_idx = (src_y * src_w + src_x) * 4;
            let dst_idx = (dest_y as usize * out_w + dest_x as usize) * 4;
            out[dst_idx..dst_idx + 4].copy_from_slice(&data[src_idx..src_idx + 4]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: out_w as u32,
            height: out_h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        out,
        source_format,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    Some(image)
}

fn render_linear_gradient_image(
    gradient: &LinearGradient,
    size: UVec2,
    scale_factor: f32,
) -> Image {
    let width = size.x.max(1) as usize;
    let height = size.y.max(1) as usize;
    let width_f = width as f32;
    let height_f = height as f32;

    let angle = gradient.angle.to_radians();
    let direction = Vec2::new(angle.sin(), -angle.cos());
    let line_length = (direction.x.abs() * width_f + direction.y.abs() * height_f).max(1.0);
    let stops = resolve_gradient_stops(gradient, line_length, scale_factor);

    let mut data = Vec::with_capacity(width * height * 4);
    let half_w = width_f / 2.0;
    let half_h = height_f / 2.0;

    for y in 0..height {
        let fy = y as f32 + 0.5 - half_h;
        for x in 0..width {
            let fx = x as f32 + 0.5 - half_w;
            let projection = fx * direction.x + fy * direction.y;
            let t = ((projection + line_length / 2.0) / line_length).clamp(0.0, 1.0);
            let color = sample_gradient_color(&stops, t);
            data.push((color.red * 255.0).round() as u8);
            data.push((color.green * 255.0).round() as u8);
            data.push((color.blue * 255.0).round() as u8);
            data.push((color.alpha * 255.0).round() as u8);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
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

#[derive(Clone, Copy)]
struct ResolvedGradientStop {
    color: Srgba,
    position: f32,
}

fn resolve_gradient_stops(
    gradient: &LinearGradient,
    line_length: f32,
    scale_factor: f32,
) -> Vec<ResolvedGradientStop> {
    let mut stops: Vec<(Srgba, Option<f32>)> = gradient
        .stops
        .iter()
        .map(|stop| {
            let position = match stop.position {
                Some(GradientStopPosition::Percent(value)) => Some(value / 100.0),
                Some(GradientStopPosition::Px(value)) => Some((value * scale_factor) / line_length),
                None => None,
            };
            (stop.color.to_srgba(), position)
        })
        .collect();

    if stops.is_empty() {
        return Vec::new();
    }

    if stops.len() == 1 {
        let color = stops[0].0;
        return vec![
            ResolvedGradientStop {
                color,
                position: 0.0,
            },
            ResolvedGradientStop {
                color,
                position: 1.0,
            },
        ];
    }

    if stops.first().and_then(|stop| stop.1).is_none() {
        stops[0].1 = Some(0.0);
    }
    if stops.last().and_then(|stop| stop.1).is_none() {
        let last = stops.len() - 1;
        stops[last].1 = Some(1.0);
    }

    let mut i = 0usize;
    while i < stops.len() {
        if stops[i].1.is_some() {
            i += 1;
            continue;
        }

        let start = i - 1;
        let mut end = i;
        while end < stops.len() && stops[end].1.is_none() {
            end += 1;
        }

        let start_pos = stops[start].1.unwrap();
        let end_pos = stops[end].1.unwrap_or(start_pos);
        let span = (end - start) as f32;
        let step = if span > 0.0 {
            (end_pos - start_pos) / span
        } else {
            0.0
        };

        for idx in i..end {
            stops[idx].1 = Some(start_pos + step * (idx - start) as f32);
        }

        i = end;
    }

    let mut resolved: Vec<ResolvedGradientStop> = Vec::with_capacity(stops.len());
    let mut prev = stops[0].1.unwrap_or(0.0);
    resolved.push(ResolvedGradientStop {
        color: stops[0].0,
        position: prev,
    });

    for (color, pos_opt) in stops.into_iter().skip(1) {
        let mut pos = pos_opt.unwrap_or(prev);
        if pos < prev {
            pos = prev;
        }
        resolved.push(ResolvedGradientStop {
            color,
            position: pos,
        });
        prev = pos;
    }

    resolved
}

fn sample_gradient_color(stops: &[ResolvedGradientStop], t: f32) -> Srgba {
    if stops.is_empty() {
        return Srgba::new(0.0, 0.0, 0.0, 0.0);
    }
    if t <= stops[0].position {
        return stops[0].color;
    }

    for window in stops.windows(2) {
        let left = window[0];
        let right = window[1];
        if t <= right.position {
            let span = (right.position - left.position).max(f32::EPSILON);
            let local = ((t - left.position) / span).clamp(0.0, 1.0);
            return lerp_srgba(left.color, right.color, local);
        }
    }

    stops.last().map(|stop| stop.color).unwrap_or_default()
}

fn lerp_srgba(a: Srgba, b: Srgba, t: f32) -> Srgba {
    Srgba {
        red: lerp(a.red, b.red, t),
        green: lerp(a.green, b.green, t),
        blue: lerp(a.blue, b.blue, t),
        alpha: lerp(a.alpha, b.alpha, t),
    }
}

fn apply_calc_length(expr: Option<&CalcExpr>, ctx: CalcContext, target: &mut Val) {
    if let Some(expr) = expr {
        if let Some(px) = expr.eval_length(ctx) {
            *target = Val::Px(px);
        }
    }
}

/// Updates the cached UI transform after styles are applied.
pub fn sync_last_ui_transform(
    mut commands: Commands,
    mut query: Query<(Entity, &UiTransform, Option<&mut LastUiTransform>)>,
) {
    for (entity, transform, last_opt) in query.iter_mut() {
        if let Some(mut last) = last_opt {
            last.0 = *transform;
        } else {
            commands.entity(entity).insert(LastUiTransform(*transform));
        }
    }
}

/// Applies a `Style` to Bevy UI components.
fn apply_style_components(
    style: &Style,
    components: &mut UiStyleComponents,
    asset_server: &AssetServer,
    image_cache: &mut ImageCache,
    images: &mut Assets<Image>,
) {
    // Node
    if let Some(node) = components.0.as_mut() {
        apply_style_to_node(style, Some(node.as_mut()));
    } else {
        apply_style_to_node(style, None);
    }

    // BackgroundColor
    if let Some(bg) = components.1.as_mut() {
        bg.0 = style
            .background
            .as_ref()
            .map(|b| b.color)
            .unwrap_or(Color::NONE);
    }

    // BorderColor
    if let Some(bc) = components.2.as_mut() {
        bc.set_all(style.border_color.unwrap_or(Color::NONE));
    }

    // BoxShadow
    if let Some(bs) = components.3.as_mut() {
        bs.0 = style.box_shadow.as_ref().cloned().unwrap_or_default().0;
    }

    // TextColor
    if let Some(tc) = components.4.as_mut() {
        tc.0 = style.color.unwrap_or(Color::WHITE);
    }

    // TextFont
    if let Some(tf) = components.5.as_mut() {
        if let Some(font_size) = style.font_size.clone() {
            tf.font_size = font_size.get(None);
        }

        if let Some(font_family) = style.font_family.as_ref() {
            let font_path_str = font_family.0.to_string();

            if font_path_str.eq_ignore_ascii_case("default") {
                tf.font = Default::default();
            } else if font_path_str.ends_with(".ttf") {
                tf.font = asset_server.load(font_path_str);
            } else {
                let folder = font_path_str.trim().trim_matches('"').trim_matches('\'');

                if folder.is_empty() {
                    tf.font = Default::default();
                } else {
                    let weight_opt = style.font_weight.clone();
                    tf.font = load_weighted_font_from_folder(asset_server, folder, weight_opt);
                }
            }
        }
    }

    // TextLayout
    if let Some(tl) = components.6.as_mut() {
        if let Some(text_wrap) = style.text_wrap {
            tl.linebreak = text_wrap;
        }
    }

    // ImageNode
    if let Some(img_node) = components.7.as_mut() {
        img_node.color = style.color.unwrap_or(Color::WHITE);

        if let Some(bg) = style.background.as_ref() {
            if let Some(path) = bg.image.as_ref() {
                let handle = get_or_load_image(path.as_str(), image_cache, images, asset_server);
                img_node.image = handle;
            }
        }
    }

    // ZIndex
    if let Some(zi) = components.8.as_mut() {
        zi.0 = style.z_index.unwrap_or(0);
    }

    // Pickable
    if let Some(pick) = components.9.as_mut() {
        let old_pick = pick.clone();
        let new_pick = style.pointer_events.as_ref().cloned().unwrap_or(Pickable {
            is_hoverable: old_pick.is_hoverable,
            should_block_lower: old_pick.should_block_lower,
        });

        **pick = new_pick;
    }

    if let Some(transform) = components.10.as_mut() {
        apply_transform_style(style, transform);
    }
}

/// Applies transform-related style fields to a `UiTransform`.
fn apply_transform_style(style: &Style, transform: &mut UiTransform) {
    if style.transform.is_empty() {
        *transform = UiTransform::default();
        return;
    }

    let mut next = UiTransform::default();

    if let Some(translation) = style.transform.translation {
        next.translation = translation;
    }

    if let Some(x) = style.transform.translation_x {
        next.translation.x = x;
    }

    if let Some(y) = style.transform.translation_y {
        next.translation.y = y;
    }

    if let Some(scale) = style.transform.scale {
        next.scale = scale;
    }

    if let Some(scale_x) = style.transform.scale_x {
        next.scale.x = scale_x;
    }

    if let Some(scale_y) = style.transform.scale_y {
        next.scale.y = scale_y;
    }

    if let Some(rotation) = style.transform.rotation {
        next.rotation = Rot2::radians(rotation);
    }

    *transform = next;
}

/// Builds a `UiTransform` from the style's transform fields.
fn ui_transform_from_style(style: &Style) -> UiTransform {
    let mut transform = UiTransform::default();
    apply_transform_style(style, &mut transform);
    transform
}

/// Blends two styles based on a transition specification.
fn blend_style(from: &Style, to: &Style, t: f32, spec: &TransitionSpec) -> Style {
    let mut blended = to.clone();

    if transition_allows_color(spec) {
        blended.color = blend_color(from.color, to.color, t);
        blended.border_color = blend_color(from.border_color, to.border_color, t);
    }

    if transition_allows_background(spec) {
        blended.background = blend_background(from.background.clone(), to.background.clone(), t);
    }

    blended
}

/// Blends two styles for animation interpolation.
fn blend_animation_style(from: &Style, to: &Style, t: f32) -> Style {
    let mut blended = to.clone();
    blended.color = blend_color(from.color, to.color, t);
    blended.border_color = blend_color(from.border_color, to.border_color, t);
    blended.background = blend_background(from.background.clone(), to.background.clone(), t);
    blended.transform = blend_transform_style(&from.transform, &to.transform, t);
    blended.width = blend_val_opt(from.width.clone(), to.width.clone(), t);
    blended.height = blend_val_opt(from.height.clone(), to.height.clone(), t);
    blended.min_width = blend_val_opt(from.min_width.clone(), to.min_width.clone(), t);
    blended.max_width = blend_val_opt(from.max_width.clone(), to.max_width.clone(), t);
    blended.min_height = blend_val_opt(from.min_height.clone(), to.min_height.clone(), t);
    blended.max_height = blend_val_opt(from.max_height.clone(), to.max_height.clone(), t);
    blended.padding = blend_ui_rect_opt(&from.padding, &to.padding, t);
    blended.margin = blend_ui_rect_opt(&from.margin, &to.margin, t);
    blended.font_size = blend_font_val_opt(&from.font_size, &to.font_size, t);
    blended.border_radius = blend_radius_opt(&from.border_radius, &to.border_radius, t);
    blended
}

/// Resolves transform blending with optional cached transforms.
fn resolve_transform_transition(
    spec: &TransitionSpec,
    from: &Style,
    to: &Style,
) -> (Option<UiTransform>, Option<UiTransform>) {
    if !transition_allows_transform(spec) {
        return (None, None);
    }

    let from_transform = ui_transform_from_style(from);
    let to_transform = ui_transform_from_style(to);
    (Some(from_transform), Some(to_transform))
}

/// Returns true if the transition includes color changes.
fn transition_allows_color(spec: &TransitionSpec) -> bool {
    spec.properties.iter().any(|prop| {
        matches!(prop, TransitionProperty::All) || matches!(prop, TransitionProperty::Color)
    })
}

/// Returns true if the transition includes background changes.
fn transition_allows_background(spec: &TransitionSpec) -> bool {
    spec.properties.iter().any(|prop| {
        matches!(prop, TransitionProperty::All) || matches!(prop, TransitionProperty::Background)
    })
}

/// Returns true if the transition includes transform changes.
fn transition_allows_transform(spec: &TransitionSpec) -> bool {
    spec.properties.iter().any(|prop| {
        matches!(
            prop,
            TransitionProperty::All | TransitionProperty::Transform
        )
    })
}

/// Linearly interpolates between two UI transforms.
fn blend_ui_transform(from: UiTransform, to: UiTransform, t: f32) -> UiTransform {
    UiTransform {
        translation: blend_val2(from.translation, to.translation, t),
        scale: from.scale.lerp(to.scale, t),
        rotation: blend_rot2(from.rotation, to.rotation, t),
    }
}

/// Blends transform style fields based on a factor.
fn blend_transform_style(from: &TransformStyle, to: &TransformStyle, t: f32) -> TransformStyle {
    TransformStyle {
        translation: blend_val2_opt(from.translation, to.translation, t),
        translation_x: blend_val_opt(from.translation_x, to.translation_x, t),
        translation_y: blend_val_opt(from.translation_y, to.translation_y, t),
        scale: blend_vec2_opt(from.scale, to.scale, t),
        scale_x: blend_f32_opt(from.scale_x, to.scale_x, t),
        scale_y: blend_f32_opt(from.scale_y, to.scale_y, t),
        rotation: blend_f32_opt(from.rotation, to.rotation, t),
    }
}

/// Blends two `Val2` values.
fn blend_val2(from: Val2, to: Val2, t: f32) -> Val2 {
    Val2::new(blend_val(from.x, to.x, t), blend_val(from.y, to.y, t))
}

/// Blends optional `Val2` values when both are set.
fn blend_val2_opt(from: Option<Val2>, to: Option<Val2>, t: f32) -> Option<Val2> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_val2(a, b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

/// Blends two `Val` values.
fn blend_val(from: Val, to: Val, t: f32) -> Val {
    match (from, to) {
        (Val::Px(a), Val::Px(b)) => Val::Px(lerp(a, b, t)),
        (Val::Percent(a), Val::Percent(b)) => Val::Percent(lerp(a, b, t)),
        _ => to,
    }
}

/// Blends optional `Val` values when both are set.
fn blend_val_opt(from: Option<Val>, to: Option<Val>, t: f32) -> Option<Val> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_val(a, b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

/// Blends two `UiRect` values.
fn blend_ui_rect(from: &UiRect, to: &UiRect, t: f32) -> UiRect {
    UiRect {
        left: blend_val(from.left.clone(), to.left.clone(), t),
        right: blend_val(from.right.clone(), to.right.clone(), t),
        top: blend_val(from.top.clone(), to.top.clone(), t),
        bottom: blend_val(from.bottom.clone(), to.bottom.clone(), t),
    }
}

/// Blends optional `UiRect` values when both are set.
fn blend_ui_rect_opt(from: &Option<UiRect>, to: &Option<UiRect>, t: f32) -> Option<UiRect> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_ui_rect(a, b, t)),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), None) => Some(a.clone()),
        _ => None,
    }
}

/// Blends two border radius values.
fn blend_radius(from: &Radius, to: &Radius, t: f32) -> Radius {
    Radius {
        top_left: blend_val(from.top_left.clone(), to.top_left.clone(), t),
        top_right: blend_val(from.top_right.clone(), to.top_right.clone(), t),
        bottom_left: blend_val(from.bottom_left.clone(), to.bottom_left.clone(), t),
        bottom_right: blend_val(from.bottom_right.clone(), to.bottom_right.clone(), t),
    }
}

/// Blends optional radius values when both are set.
fn blend_radius_opt(from: &Option<Radius>, to: &Option<Radius>, t: f32) -> Option<Radius> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_radius(a, b, t)),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), None) => Some(a.clone()),
        _ => None,
    }
}

/// Blends two font size values.
fn blend_font_val(from: &FontVal, to: &FontVal, t: f32) -> FontVal {
    match (from, to) {
        (FontVal::Px(a), FontVal::Px(b)) => FontVal::Px(lerp(*a, *b, t)),
        (FontVal::Rem(a), FontVal::Rem(b)) => FontVal::Rem(lerp(*a, *b, t)),
        _ => to.clone(),
    }
}

/// Blends optional font sizes when both are set.
fn blend_font_val_opt(from: &Option<FontVal>, to: &Option<FontVal>, t: f32) -> Option<FontVal> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_font_val(a, b, t)),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), None) => Some(a.clone()),
        _ => None,
    }
}

/// Blends two rotations.
fn blend_rot2(from: Rot2, to: Rot2, t: f32) -> Rot2 {
    let from_angle = from.as_radians();
    let to_angle = to.as_radians();
    Rot2::radians(lerp(from_angle, to_angle, t))
}

/// Blends optional vectors when both are set.
fn blend_vec2_opt(from: Option<Vec2>, to: Option<Vec2>, t: f32) -> Option<Vec2> {
    match (from, to) {
        (Some(a), Some(b)) => Some(a.lerp(b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

/// Blends optional floats when both are set.
fn blend_f32_opt(from: Option<f32>, to: Option<f32>, t: f32) -> Option<f32> {
    match (from, to) {
        (Some(a), Some(b)) => Some(lerp(a, b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

/// Blends optional colors when both are set.
fn blend_color(from: Option<Color>, to: Option<Color>, t: f32) -> Option<Color> {
    match (from, to) {
        (Some(a), Some(b)) => {
            let a = a.to_srgba();
            let b = b.to_srgba();
            Some(Color::Srgba(Srgba {
                red: lerp(a.red, b.red, t),
                green: lerp(a.green, b.green, t),
                blue: lerp(a.blue, b.blue, t),
                alpha: lerp(a.alpha, b.alpha, t),
            }))
        }
        (Some(value), None) => Some(value),
        (None, Some(value)) => Some(value),
        _ => None,
    }
}

/// Blends background colors and images.
fn blend_background(
    from: Option<crate::styles::Background>,
    to: Option<crate::styles::Background>,
    t: f32,
) -> Option<crate::styles::Background> {
    match (from, to) {
        (Some(a), Some(b)) => Some(crate::styles::Background {
            color: blend_color(Some(a.color), Some(b.color), t).unwrap_or(a.color),
            image: if t >= 1.0 { b.image } else { a.image },
            gradient: if t >= 1.0 { b.gradient } else { a.gradient },
        }),
        (Some(value), None) => Some(value),
        (None, Some(value)) => Some(value),
        _ => None,
    }
}

/// Computes the current animation style state and applies it.
fn update_style_animation_state(
    commands: &mut Commands,
    entity: Entity,
    final_style: &Style,
    keyframes: &std::collections::HashMap<String, Vec<AnimationKeyframe>>,
    now: f32,
    animation_query: &mut Query<Option<&mut StyleAnimation>>,
) {
    let mut animation = animation_query.get_mut(entity).ok().flatten();

    let Some(spec) = final_style.animation.clone() else {
        if animation.is_some() {
            commands.entity(entity).remove::<StyleAnimation>();
        }
        return;
    };

    if spec.name.is_empty() {
        if animation.is_some() {
            commands.entity(entity).remove::<StyleAnimation>();
        }
        return;
    }

    let Some(frames) = keyframes.get(&spec.name) else {
        if animation.is_some() {
            commands.entity(entity).remove::<StyleAnimation>();
        }
        return;
    };

    if frames.is_empty() {
        if animation.is_some() {
            commands.entity(entity).remove::<StyleAnimation>();
        }
        return;
    }

    let mut computed = Vec::with_capacity(frames.len());
    for frame in frames {
        let mut style = final_style.clone();
        style.merge(&frame.style);
        computed.push(AnimationKeyframe {
            progress: frame.progress,
            style,
        });
    }

    let new_animation = StyleAnimation {
        base: final_style.clone(),
        keyframes: computed,
        spec,
        start_time: now,
        current_style: None,
    };

    if let Some(existing) = animation.as_mut() {
        if existing.spec != new_animation.spec {
            **existing = new_animation;
        } else if existing.base != new_animation.base
            || existing.keyframes != new_animation.keyframes
        {
            existing.base = new_animation.base;
            existing.keyframes = new_animation.keyframes;
        }
    } else {
        commands.entity(entity).insert(new_animation);
    }
}

/// Computes eased animation progress based on spec and cycle.
fn animation_progress(spec: &AnimationSpec, cycle_index: u32, cycle_progress: f32) -> f32 {
    let mut progress = cycle_progress.clamp(0.0, 1.0);
    let is_odd = cycle_index % 2 == 1;
    match spec.direction {
        AnimationDirection::Normal => {}
        AnimationDirection::Reverse => progress = 1.0 - progress,
        AnimationDirection::Alternate => {
            if is_odd {
                progress = 1.0 - progress;
            }
        }
        AnimationDirection::AlternateReverse => {
            if !is_odd {
                progress = 1.0 - progress;
            }
        }
    }
    spec.timing.apply(progress)
}

/// Samples a style from keyframes at the given progress.
fn sample_animation_style(keyframes: &[AnimationKeyframe], progress: f32) -> Style {
    if keyframes.is_empty() {
        return Style::default();
    }

    let mut prev = &keyframes[0];
    if progress <= prev.progress {
        return prev.style.clone();
    }

    for next in keyframes.iter().skip(1) {
        if progress <= next.progress {
            if (next.progress - prev.progress).abs() < f32::EPSILON {
                return next.style.clone();
            }
            let local_t = (progress - prev.progress) / (next.progress - prev.progress);
            return blend_animation_style(&prev.style, &next.style, local_t);
        }
        prev = next;
    }

    prev.style.clone()
}

/// Linearly interpolates between two floats.
fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

/// Returns true if the selector's pseudo state matches the widget state.
fn selector_matches_state(selector: &str, state: &UIWidgetState) -> bool {
    for part in selector.replace('>', " > ").split_whitespace() {
        if part == ">" {
            continue;
        }
        let segments: Vec<&str> = part.split(':').collect();
        for pseudo in &segments[1..] {
            match *pseudo {
                "read-only" if !state.readonly => return false,
                "disabled" if !state.disabled => return false,
                "checked" if state.disabled || !state.checked => return false,
                "focus" if state.disabled || !state.focused => return false,
                "hover" if state.disabled || !state.hovered => return false,
                "invalid" if !state.invalid => return false,
                _ => {}
            }
        }
    }
    true
}

/// Computes a simple specificity score for a selector.
fn selector_specificity(selector: &str) -> u32 {
    let mut spec = 0;
    for part in selector.replace('>', " > ").split_whitespace() {
        if part == ">" {
            continue;
        }
        let segments: Vec<&str> = part.split(':').collect();
        let base = segments[0];

        spec += if base.starts_with('#') {
            100
        } else if base.starts_with('.') {
            10
        } else if base == "*" {
            0
        } else {
            1
        };

        spec += segments.len().saturating_sub(1) as u32;
    }
    spec
}

/// Applies layout-related style fields to a Bevy `Node`.
fn apply_style_to_node(style: &Style, node: Option<&mut Node>) {
    if let Some(node) = node {
        node.width = style.width.unwrap_or_default();
        node.min_width = style.min_width.unwrap_or_default();
        node.max_width = style.max_width.unwrap_or_default();
        node.height = style.height.unwrap_or_default();
        node.min_height = style.min_height.unwrap_or_default();
        node.max_height = style.max_height.unwrap_or_default();
        node.display = style.display.unwrap_or_default();
        node.position_type = style.position_type.unwrap_or_default();
        node.left = style.left.unwrap_or_default();
        node.top = style.top.unwrap_or_default();
        node.right = style.right.unwrap_or_default();
        node.bottom = style.bottom.unwrap_or_default();
        node.padding = style.padding.unwrap_or_default();
        node.margin = style.margin.unwrap_or_default();
        node.border = style.border.unwrap_or_default();

        let mut br = node.border_radius;

        if let Some(radius) = style.border_radius.clone() {
            br.top_left = radius.top_left;
            br.top_right = radius.top_right;
            br.bottom_left = radius.bottom_left;
            br.bottom_right = radius.bottom_right;
        } else {
            br.top_left = Val::ZERO;
            br.top_right = Val::ZERO;
            br.bottom_left = Val::ZERO;
            br.bottom_right = Val::ZERO;
        }

        node.border_radius = br;
        node.justify_content = style.justify_content.unwrap_or_default();
        node.align_items = style.align_items.unwrap_or_default();
        node.overflow = style.overflow.unwrap_or_default();

        node.flex_direction = style.flex_direction.unwrap_or(FlexDirection::Row);
        let row_gap = style.row_gap.or(style.gap).unwrap_or_default();
        let column_gap = style.column_gap.or(style.gap).unwrap_or_default();
        node.row_gap = row_gap;
        node.column_gap = column_gap;

        node.flex_grow = style.flex_grow.unwrap_or_default();
        node.flex_basis = style.flex_basis.unwrap_or_default();
        node.flex_shrink = style.flex_shrink.unwrap_or_default();
        node.flex_wrap = style.flex_wrap.unwrap_or_default();

        node.grid_row = style.grid_row.unwrap_or_default();
        node.grid_column = style.grid_column.unwrap_or_default();
        node.grid_auto_flow = style.grid_auto_flow.unwrap_or_default();
        node.grid_template_rows = style.grid_template_rows.clone().unwrap_or_default();
        node.grid_template_columns = style.grid_template_columns.clone().unwrap_or_default();
        node.grid_auto_columns = style.grid_auto_columns.clone().unwrap_or_default();
        node.grid_auto_rows = style.grid_auto_rows.clone().unwrap_or_default();
    }
}

/// Loads a font asset from a folder based on weight tokens.
fn load_weighted_font_from_folder(
    asset_server: &AssetServer,
    folder: &str,
    weight: Option<FontWeight>,
) -> Handle<Font> {
    let folder = folder
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches('/');
    if folder.is_empty() {
        return Default::default();
    }

    let family = folder_basename(folder);

    let w = weight.unwrap_or(FontWeight::Normal);

    let token = weight_token_exact(w);
    let path_primary = format!("{folder}/{family}-{token}.ttf");

    asset_server.load::<Font>(path_primary)
}

/// Maps a font weight to its exact token used in filenames.
fn weight_token_exact(weight: FontWeight) -> &'static str {
    match weight {
        FontWeight::Thin => "Thin",
        FontWeight::ExtraLight => "ExtraLight",
        FontWeight::Light => "Light",
        FontWeight::Normal => "Regular",
        FontWeight::Medium => "Medium",
        FontWeight::SemiBold => "SemiBold",
        FontWeight::Bold => "Bold",
        FontWeight::ExtraBold => "ExtraBold",
        FontWeight::Black => "Black",
    }
}

/// Returns the last path segment of a folder path.
fn folder_basename(folder: &str) -> &str {
    folder
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(folder)
}

/// Propagates inheritable style fields down the widget tree.
pub fn propagate_style_inheritance(
    root_query: Query<Entity, (With<UiStyle>, Without<ChildOf>)>,
    children_query: Query<&Children>,
    style_query: Query<&UiStyle>,
    mut target_query: Query<(
        Option<&mut TextColor>,
        Option<&mut TextFont>,
        Option<&mut ImageNode>,
        Option<&StyleTransition>,
        Option<&StyleAnimation>,
    )>,
    asset_server: Res<AssetServer>,
) {
    for root_entity in root_query.iter() {
        propagate_recursive(
            root_entity,
            None,
            &children_query,
            &style_query,
            &mut target_query,
            &asset_server,
        );
    }
}

/// Recursively applies inherited styles to children.
fn propagate_recursive(
    entity: Entity,
    inherited_style: Option<&Style>,
    children_query: &Query<&Children>,
    style_query: &Query<&UiStyle>,
    target_query: &mut Query<(
        Option<&mut TextColor>,
        Option<&mut TextFont>,
        Option<&mut ImageNode>,
        Option<&StyleTransition>,
        Option<&StyleAnimation>,
    )>,
    asset_server: &Res<AssetServer>,
) {
    // 1. Determine the effective style for THIS entity
    //    If this entity has its own active_style, we use it.
    //    Otherwise, we might use inherited values.
    //    BUT: The original requirement is about parents passing styles to children.
    //    So we look at what we should pass down vs what we should apply here.

    let my_style_comp = style_query.get(entity).ok();
    let my_active_style = my_style_comp.and_then(|s| s.active_style.as_ref());

    // 2. Prepare the style to pass down to children.
    //    This is effective: inherited_style merged with my_active_style
    //    (where my_active_style takes precedence).
    let mut style_to_propagate = if let Some(inherited) = inherited_style {
        inherited.clone()
    } else {
        Style::default()
    };

    if let Some(mine) = my_active_style {
        style_to_propagate.merge(mine);
    }

    // 2.1 Check for active animations or transitions on this entity
    // and use their current style if available.
    if let Ok(components) = target_query.get(entity) {
        if let Some(transition) = components.3 {
            if let Some(current) = &transition.current_style {
                style_to_propagate.merge(current);
            }
        }
        if let Some(animation) = components.4 {
            if let Some(current) = &animation.current_style {
                style_to_propagate.merge(current);
            }
        }
    }

    // 3. Apply styles to THIS entity's components if strictly inherited (no local override)
    if let Ok(components) = target_query.get_mut(entity) {
        let (mut text_color_opt, mut text_font_opt, mut image_node_opt, _, _) = components;
        // --- COLOR ---
        // Apply inherited color if I don't have my own color
        let has_local_color = my_active_style.map_or(false, |s| s.color.is_some());
        if !has_local_color {
            if let Some(parent_color) = inherited_style.and_then(|s| s.color) {
                if let Some(text_color) = text_color_opt.as_mut() {
                    if text_color.0 != parent_color {
                        text_color.0 = parent_color;
                    }
                }
                if let Some(image_node) = image_node_opt.as_mut() {
                    if image_node.color != parent_color {
                        image_node.color = parent_color;
                    }
                }
            }
        }

        // --- FONT SIZE ---
        let has_local_size = my_active_style.map_or(false, |s| s.font_size.is_some());
        if !has_local_size {
            if let Some(parent_size_val) = inherited_style.and_then(|s| s.font_size.as_ref()) {
                if let Some(text_font) = text_font_opt.as_mut() {
                    // 12.0 is default base, could be configurable
                    let size_px = parent_size_val.get(Some(12.0));
                    if text_font.font_size != size_px {
                        text_font.font_size = size_px;
                    }
                }
            }
        }

        // --- FONT FAMILY & WEIGHT ---
        // Note: Logic similar to apply_text_style needed here to resolve handle.
        // If we strictly inherit family/weight, we need to load the font.
        // This is complex because we need the folder structure logic from apply_text_style.
        // For now, a simpler approach: if we have a resolved font handle from parent logic?
        // Actually, style_service resolves fonts every time active_style changes.
        // Providing the full path logic again here might be duplicative.
        // A better approach: The `inherited_style` now contains the family/weight.
        // We can re-use the standard `apply_text_style` logic if we synthesize a style?
        // OR: Just implement the specific property application here.

        let has_local_family = my_active_style.map_or(false, |s| s.font_family.is_some());
        let has_local_weight = my_active_style.map_or(false, |s| s.font_weight.is_some());

        if !has_local_family && !has_local_weight {
            // We need to form the font path from inherited values
            if let Some(inherited) = inherited_style {
                if let Some(family) = &inherited.font_family {
                    let weight = inherited.font_weight.unwrap_or(FontWeight::Normal);
                    let folder = &family.0; // Assuming FontFamily is struct(String)
                    let weight_str = weight_token_exact(weight);
                    // This assumes a standard naming convention "Family-Weight.ttf"
                    let filename = format!("{}-{}.ttf", folder_basename(folder), weight_str);
                    let full_path = format!("{}/{}", folder, filename);

                    let handle = asset_server.load(full_path);
                    if let Some(text_font) = text_font_opt.as_mut() {
                        if text_font.font != handle {
                            text_font.font = handle;
                        }
                    }
                }
            }
        }
    }

    // 4. Recurse to children
    if let Ok(children) = children_query.get(entity) {
        for child_entity in children {
            propagate_recursive(
                *child_entity,
                Some(&style_to_propagate), // Pass down the merged style
                children_query,
                style_query,
                target_query,
                asset_server,
            );
        }
    }
}
