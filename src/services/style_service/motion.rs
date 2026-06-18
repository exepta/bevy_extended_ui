use std::collections::HashMap;

use bevy::math::Rot2;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui::{UiTransform, Val2};

use crate::ImageCache;
use crate::styles::components::UiStyle;
use crate::styles::{
    AnimationDirection, AnimationKeyframe, AnimationSpec, Radius, Style, TransformStyle,
    TransitionProperty, TransitionSpec,
};

use super::{UiStyleComponents, apply_style_components, apply_transform_style};

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

/// Component storing an active style animation.
#[derive(Component, Debug, Clone)]
pub struct StyleAnimation {
    pub base: Style,
    pub keyframes: Vec<AnimationKeyframe>,
    pub spec: AnimationSpec,
    pub start_time: f32,
    pub current_style: Option<Style>,
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

/// Handles `apply_transform_style_if_blocked` in the extended UI workflow.
pub(super) fn apply_transform_style_if_blocked(
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

/// Resolves transform blending with optional cached transforms.
pub(super) fn resolve_transform_transition(
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

/// Computes the current animation style state and applies it.
pub(super) fn update_style_animation_state(
    commands: &mut Commands,
    entity: Entity,
    final_style: &Style,
    keyframes: &HashMap<String, Vec<AnimationKeyframe>>,
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
        blended.outline_color = blend_color(from.outline_color, to.outline_color, t);
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
    blended.outline_color = blend_color(from.outline_color, to.outline_color, t);
    blended.background = blend_background(from.background.clone(), to.background.clone(), t);
    blended.transform = blend_transform_style(&from.transform, &to.transform, t);
    blended.width = blend_val_opt(from.width, to.width, t);
    blended.height = blend_val_opt(from.height, to.height, t);
    blended.min_width = blend_val_opt(from.min_width, to.min_width, t);
    blended.max_width = blend_val_opt(from.max_width, to.max_width, t);
    blended.min_height = blend_val_opt(from.min_height, to.min_height, t);
    blended.max_height = blend_val_opt(from.max_height, to.max_height, t);
    blended.padding = blend_ui_rect_opt(&from.padding, &to.padding, t);
    blended.margin = blend_ui_rect_opt(&from.margin, &to.margin, t);
    blended.font_size = blend_font_size_opt(&from.font_size, &to.font_size, t);
    blended.border_radius = blend_radius_opt(&from.border_radius, &to.border_radius, t);
    blended
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
        left: blend_val(from.left, to.left, t),
        right: blend_val(from.right, to.right, t),
        top: blend_val(from.top, to.top, t),
        bottom: blend_val(from.bottom, to.bottom, t),
    }
}

/// Blends optional `UiRect` values when both are set.
fn blend_ui_rect_opt(from: &Option<UiRect>, to: &Option<UiRect>, t: f32) -> Option<UiRect> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_ui_rect(a, b, t)),
        (None, Some(b)) => Some(*b),
        (Some(a), None) => Some(*a),
        _ => None,
    }
}

/// Blends two border radius values.
fn blend_radius(from: &Radius, to: &Radius, t: f32) -> Radius {
    Radius {
        top_left: blend_val(from.top_left, to.top_left, t),
        top_right: blend_val(from.top_right, to.top_right, t),
        bottom_left: blend_val(from.bottom_left, to.bottom_left, t),
        bottom_right: blend_val(from.bottom_right, to.bottom_right, t),
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
fn blend_font_size(from: FontSize, to: FontSize, t: f32) -> FontSize {
    match (from, to) {
        (FontSize::Px(a), FontSize::Px(b)) => FontSize::Px(lerp(a, b, t)),
        (FontSize::Rem(a), FontSize::Rem(b)) => FontSize::Rem(lerp(a, b, t)),
        (FontSize::Vw(a), FontSize::Vw(b)) => FontSize::Vw(lerp(a, b, t)),
        (FontSize::Vh(a), FontSize::Vh(b)) => FontSize::Vh(lerp(a, b, t)),
        (FontSize::VMin(a), FontSize::VMin(b)) => FontSize::VMin(lerp(a, b, t)),
        (FontSize::VMax(a), FontSize::VMax(b)) => FontSize::VMax(lerp(a, b, t)),
        _ => to,
    }
}

/// Blends optional font sizes when both are set.
fn blend_font_size_opt(from: &Option<FontSize>, to: &Option<FontSize>, t: f32) -> Option<FontSize> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_font_size(*a, *b, t)),
        (None, Some(b)) => Some(*b),
        (Some(a), None) => Some(*a),
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
