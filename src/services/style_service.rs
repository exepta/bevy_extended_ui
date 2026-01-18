use crate::html::HtmlStyle;
use crate::services::image_service::get_or_load_image;
use crate::services::state_service::update_widget_states;
use crate::styles::components::UiStyle;
use crate::styles::{
    AnimationDirection, AnimationKeyframe, AnimationSpec, FontWeight, Style, TransformStyle,
    TransitionProperty, TransitionSpec,
};
use crate::widgets::UIWidgetState;

use bevy::color::Srgba;
use bevy::math::Rot2;
use bevy::prelude::*;
use bevy::ui::{UiTransform, Val2};
use crate::ImageCache;

pub struct StyleService;

impl Plugin for StyleService {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_widget_styles_system.after(update_widget_states),
        );
        app.add_systems(
            PostUpdate,
            update_style_transitions.after(update_widget_styles_system),
        );
        app.add_systems(
            PostUpdate,
            update_style_animations.after(update_style_transitions),
        );
        app.add_systems(
            PostUpdate,
            sync_last_ui_transform.after(update_style_animations),
        );
    }
}

#[derive(Component, Debug, Clone)]
pub struct StyleTransition {
    from: Style,
    to: Style,
    start_time: f32,
    spec: TransitionSpec,
    from_transform: Option<UiTransform>,
    to_transform: Option<UiTransform>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct LastUiTransform(pub UiTransform);

#[derive(Component, Debug, Clone)]
pub struct StyleAnimation {
    base: Style,
    keyframes: Vec<AnimationKeyframe>,
    spec: AnimationSpec,
    start_time: f32,
}

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

pub fn update_widget_styles_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            Option<&UIWidgetState>,
            Option<&HtmlStyle>,
            &mut UiStyle,
        ),
        Or<(Changed<UiStyle>, Changed<HtmlStyle>, Changed<UIWidgetState>)>,
    >,
    mut transition_query: Query<Option<&mut StyleTransition>>,
    mut animation_query: Query<Option<&mut StyleAnimation>>,

    mut qs: ParamSet<(
        Query<UiStyleComponents>,
        Query<(&UiTransform, Option<&LastUiTransform>)>,
    )>,

    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, state_opt, html_style_opt, mut ui_style) in query.iter_mut() {
        let state = state_opt.cloned().unwrap_or_default();

        let mut base_styles: Vec<(&String, u32)> = vec![];
        let mut pseudo_styles: Vec<(&String, u32)> = vec![];

        for sel in ui_style.styles.keys() {
            if selector_matches_state(sel, &state) {
                let specificity = selector_specificity(sel);
                if sel.contains(':') {
                    pseudo_styles.push((sel, specificity));
                } else {
                    base_styles.push((sel, specificity));
                }
            }
        }

        base_styles.sort_by_key(|&(_, spec)| spec);
        pseudo_styles.sort_by_key(|&(_, spec)| spec);

        let mut final_style = Style::default();

        // 1) base normal
        for (sel, _) in &base_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.normal);
            }
        }

        // 2) base important
        for (sel, _) in &base_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.important);
            }
        }

        // 3) inline html
        if let Some(html_style) = html_style_opt {
            final_style.merge(&html_style.0);
        }

        // 4) pseudo normal
        for (sel, _) in &pseudo_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.normal);
            }
        }

        // 5) pseudo important
        for (sel, _) in &pseudo_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.important);
            }
        }

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

            let (from_transform, to_transform) =
                resolve_transform_transition(&spec, qs.p1().get(entity).ok());

            let transition_state = StyleTransition {
                from,
                to,
                start_time: time.elapsed_secs(),
                spec,
                from_transform,
                to_transform,
            };

            if let Some(existing) = transition.as_mut() {
                **existing = transition_state;
            } else {
                commands.entity(entity).insert(transition_state);
            }
            continue;
        }

        if let Some(transition) = transition.as_mut() {
            if !has_changed {
                continue;
            }
            transition.from = previous_style.unwrap_or_default();
            transition.to = final_style.clone();
            transition.start_time = time.elapsed_secs();
            transition.spec = final_style.transition.clone().unwrap_or_default();

            let (from_transform, to_transform) =
                resolve_transform_transition(&transition.spec, qs.p1().get(entity).ok());
            transition.from_transform = from_transform;
            transition.to_transform = to_transform;

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
    }
}

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

    for (entity, transition) in transitions.iter_mut() {
        let elapsed = now - transition.start_time - transition.spec.delay;
        let duration = transition.spec.duration.max(0.001);
        let t = (elapsed / duration).clamp(0.0, 1.0);
        let eased = transition.spec.timing.apply(t);
        let blended = blend_style(&transition.from, &transition.to, eased, &transition.spec);

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

pub fn update_style_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut animations: Query<(Entity, &mut StyleAnimation)>,
    mut style_query: Query<UiStyleComponents>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let now = time.elapsed_secs();

    for (entity, animation) in animations.iter_mut() {
        let duration = animation.spec.duration.max(0.001);
        let elapsed = now - animation.start_time - animation.spec.delay;

        if elapsed < 0.0 {
            continue;
        }

        if let Some(iterations) = animation.spec.iterations {
            if iterations <= 0.0 {
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
        if style.transform.is_empty() {
            **transform = UiTransform::default();
            return;
        }

        let mut next = **transform;

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

        **transform = next;
    }
}

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

fn blend_animation_style(from: &Style, to: &Style, t: f32) -> Style {
    let mut blended = to.clone();
    blended.color = blend_color(from.color, to.color, t);
    blended.border_color = blend_color(from.border_color, to.border_color, t);
    blended.background = blend_background(from.background.clone(), to.background.clone(), t);
    blended.transform = blend_transform_style(&from.transform, &to.transform, t);
    blended
}

fn resolve_transform_transition(
    spec: &TransitionSpec,
    transform: Option<(&UiTransform, Option<&LastUiTransform>)>,
) -> (Option<UiTransform>, Option<UiTransform>) {
    if !transition_allows_transform(spec) {
        return (None, None);
    }

    let Some((current, last)) = transform else {
        return (None, None);
    };

    let from = last.map(|last| last.0).unwrap_or(*current);
    let to = *current;
    (Some(from), Some(to))
}

fn transition_allows_color(spec: &TransitionSpec) -> bool {
    spec.properties.iter().any(|prop| {
        matches!(prop, TransitionProperty::All) || matches!(prop, TransitionProperty::Color)
    })
}

fn transition_allows_background(spec: &TransitionSpec) -> bool {
    spec.properties.iter().any(|prop| {
        matches!(prop, TransitionProperty::All) || matches!(prop, TransitionProperty::Background)
    })
}

fn transition_allows_transform(spec: &TransitionSpec) -> bool {
    spec.properties.iter().any(|prop| {
        matches!(
            prop,
            TransitionProperty::All | TransitionProperty::Transform
        )
    })
}

fn blend_ui_transform(from: UiTransform, to: UiTransform, t: f32) -> UiTransform {
    UiTransform {
        translation: blend_val2(from.translation, to.translation, t),
        scale: from.scale.lerp(to.scale, t),
        rotation: blend_rot2(from.rotation, to.rotation, t),
    }
}

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

fn blend_val2(from: Val2, to: Val2, t: f32) -> Val2 {
    Val2::new(blend_val(from.x, to.x, t), blend_val(from.y, to.y, t))
}

fn blend_val2_opt(from: Option<Val2>, to: Option<Val2>, t: f32) -> Option<Val2> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_val2(a, b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

fn blend_val(from: Val, to: Val, t: f32) -> Val {
    match (from, to) {
        (Val::Px(a), Val::Px(b)) => Val::Px(lerp(a, b, t)),
        (Val::Percent(a), Val::Percent(b)) => Val::Percent(lerp(a, b, t)),
        _ => to,
    }
}

fn blend_val_opt(from: Option<Val>, to: Option<Val>, t: f32) -> Option<Val> {
    match (from, to) {
        (Some(a), Some(b)) => Some(blend_val(a, b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

fn blend_rot2(from: Rot2, to: Rot2, t: f32) -> Rot2 {
    let from_angle = from.as_radians();
    let to_angle = to.as_radians();
    Rot2::radians(lerp(from_angle, to_angle, t))
}

fn blend_vec2_opt(from: Option<Vec2>, to: Option<Vec2>, t: f32) -> Option<Vec2> {
    match (from, to) {
        (Some(a), Some(b)) => Some(a.lerp(b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

fn blend_f32_opt(from: Option<f32>, to: Option<f32>, t: f32) -> Option<f32> {
    match (from, to) {
        (Some(a), Some(b)) => Some(lerp(a, b, t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

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

fn blend_background(
    from: Option<crate::styles::Background>,
    to: Option<crate::styles::Background>,
    t: f32,
) -> Option<crate::styles::Background> {
    match (from, to) {
        (Some(a), Some(b)) => Some(crate::styles::Background {
            color: blend_color(Some(a.color), Some(b.color), t).unwrap_or(a.color),
            image: if t >= 1.0 { b.image } else { a.image },
        }),
        (Some(value), None) => Some(value),
        (None, Some(value)) => Some(value),
        _ => None,
    }
}

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
    };

    if let Some(existing) = animation.as_mut() {
        if existing.spec != new_animation.spec
            || existing.base != new_animation.base
            || existing.keyframes != new_animation.keyframes
        {
            **existing = new_animation;
        }
    } else {
        commands.entity(entity).insert(new_animation);
    }
}

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

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn selector_matches_state(selector: &str, state: &UIWidgetState) -> bool {
    for part in selector.split_whitespace() {
        let segments: Vec<&str> = part.split(':').collect();
        for pseudo in &segments[1..] {
            match *pseudo {
                "read-only" if !state.readonly => return false,
                "disabled" if !state.disabled => return false,
                "checked" if state.disabled || !state.checked => return false,
                "focus" if state.disabled || !state.focused => return false,
                "hover" if state.disabled || !state.hovered => return false,
                _ => {}
            }
        }
    }
    true
}

fn selector_specificity(selector: &str) -> u32 {
    let mut spec = 0;
    for part in selector.split_whitespace() {
        let segments: Vec<&str> = part.split(':').collect();
        let base = segments[0];

        spec += if base.starts_with('#') {
            100
        } else if base.starts_with('.') {
            10
        } else {
            1
        };

        spec += segments.len().saturating_sub(1) as u32;
    }
    spec
}

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
        match node.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                node.column_gap = style.gap.unwrap_or_default();
                node.row_gap = Val::Auto;
            }
            _ => {
                node.row_gap = style.gap.unwrap_or_default();
                node.column_gap = Val::Auto;
            }
        }

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

fn folder_basename(folder: &str) -> &str {
    folder
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(folder)
}
