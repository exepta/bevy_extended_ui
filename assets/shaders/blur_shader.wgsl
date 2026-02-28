#import bevy_ui::ui_vertex_output::UiVertexOutput
#import bevy_ui::ui_node::draw_uinode_background

struct BackdropBlurUniform {
    blur_radius_px: f32,
    overlay_alpha: f32,
    feedback_compensation: f32,
    viewport_size: vec2<f32>,
    tint: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: BackdropBlurUniform;
@group(1) @binding(1)
var screen_texture: texture_2d<f32>;
@group(1) @binding(2)
var screen_sampler: sampler;
@group(1) @binding(3)
var overlay_texture: texture_2d<f32>;
@group(1) @binding(4)
var overlay_sampler: sampler;

const MAX_RADIUS: i32 = 24;

fn gaussian_weight(distance_sq: f32, sigma: f32) -> f32 {
    return exp(-distance_sq / (2.0 * sigma * sigma));
}

fn undo_tint_feedback(color: vec3<f32>, tint: vec4<f32>) -> vec3<f32> {
    let a = clamp(tint.a, 0.0, 0.95);
    if (a <= 0.0001) {
        return color;
    }
    let recovered = (color - tint.rgb * a) / max(1.0 - a, 0.001);
    return clamp(recovered, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let texture_size = max(vec2<f32>(textureDimensions(screen_texture)), vec2<f32>(1.0));
    let viewport_size = max(material.viewport_size, vec2<f32>(1.0));
    let screen_uv = clamp(in.position.xy / viewport_size, vec2<f32>(0.0), vec2<f32>(1.0));
    let texel = vec2<f32>(1.0) / texture_size;

    let requested_radius = max(material.blur_radius_px, 0.0);
    let node_area = max(in.size.x * in.size.y, 1.0);
    let viewport_area = max(viewport_size.x * viewport_size.y, 1.0);
    let coverage = clamp(node_area / viewport_area, 0.0, 1.0);
    let fullscreen_penalty = smoothstep(0.55, 0.95, coverage);
    let dynamic_max_radius = mix(f32(MAX_RADIUS), 10.0, fullscreen_penalty);
    let radius = i32(clamp(round(requested_radius), 0.0, dynamic_max_radius));
    // Keep fullscreen blur cleaner (less mushy) by softly compressing effective radius.
    let effective_radius = requested_radius * mix(1.0, 0.72, fullscreen_penalty);
    let sample_step = max(effective_radius / max(f32(radius), 1.0), 1.0);
    let sigma = max(effective_radius * 0.42, 1.0);
    let radius_sq = radius * radius;
    let ring_core = i32(floor(f32(radius) * 0.35));
    let ring_mid = i32(floor(f32(radius) * 0.58));
    let ring_outer = i32(floor(f32(radius) * 0.82));
    let ring_core_sq = ring_core * ring_core;
    let ring_mid_sq = ring_mid * ring_mid;
    let ring_outer_sq = ring_outer * ring_outer;
    let sparse_sampling = radius >= 10;
    var base_stride = 1;
    if (requested_radius > 95.0) {
        base_stride = 2;
    }
    if (requested_radius > 165.0) {
        base_stride = 3;
    }
    var coverage_stride_bonus = 0;
    if (requested_radius > 110.0) {
        coverage_stride_bonus = i32(round(mix(0.0, 2.0, fullscreen_penalty)));
    }

    var sum = vec4<f32>(0.0);
    var weight_sum = 0.0;

    for (var y = -radius; y <= radius; y = y + 1) {
        for (var x = -radius; x <= radius; x = x + 1) {
            let dist_sq_i = x * x + y * y;
            if (dist_sq_i > radius_sq) {
                continue;
            }
            if (sparse_sampling) {
                var stride = 1;
                if (dist_sq_i > ring_core_sq) {
                    stride = base_stride + coverage_stride_bonus;
                }
                if (dist_sq_i > ring_mid_sq) {
                    stride = base_stride + coverage_stride_bonus + 1;
                }
                if (dist_sq_i > ring_outer_sq) {
                    stride = base_stride + coverage_stride_bonus + 2;
                }
                let lattice = abs(x) * 7 + abs(y) * 11;
                if ((lattice % stride) != 0) {
                    continue;
                }
            }

            let offset = vec2<f32>(f32(x), f32(y)) * sample_step;
            let sample_uv = clamp(screen_uv + offset * texel, vec2<f32>(0.0), vec2<f32>(1.0));
            var sample_color = textureSample(screen_texture, screen_sampler, sample_uv);
            if (material.feedback_compensation > 0.5) {
                sample_color = vec4<f32>(
                    undo_tint_feedback(sample_color.rgb, material.tint),
                    sample_color.a,
                );
            }
            let weight = gaussian_weight(dot(offset, offset), sigma);
            sum += sample_color * weight;
            weight_sum += weight;
        }
    }

    var base_color = textureSample(screen_texture, screen_sampler, screen_uv);
    if (material.feedback_compensation > 0.5) {
        base_color = vec4<f32>(
            undo_tint_feedback(base_color.rgb, material.tint),
            base_color.a,
        );
    }
    let blurred = select(base_color, sum / max(weight_sum, 0.0001), radius > 0);

    let background_alpha = clamp(material.tint.a, 0.0, 1.0);
    let tinted_glass = mix(blurred.rgb, material.tint.rgb, background_alpha);
    let overlay_sample = textureSample(overlay_texture, overlay_sampler, clamp(in.uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    let overlay_alpha = clamp(overlay_sample.a * material.overlay_alpha, 0.0, 1.0);
    let glass_color = mix(tinted_glass, overlay_sample.rgb, overlay_alpha);

    let point = in.uv * in.size - 0.5 * in.size;
    return draw_uinode_background(
        vec4<f32>(glass_color, 1.0),
        point,
        in.size,
        in.border_radius,
        in.border_widths,
    );
}
