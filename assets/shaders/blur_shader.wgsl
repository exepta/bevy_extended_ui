#import bevy_ui::ui_vertex_output::UiVertexOutput
#import bevy_ui::ui_node::draw_uinode_background

struct BackdropBlurUniform {
    blur_radius_px: f32,
    _pad: vec3<f32>,
    tint: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: BackdropBlurUniform;
@group(1) @binding(1)
var screen_texture: texture_2d<f32>;
@group(1) @binding(2)
var screen_sampler: sampler;

const MAX_RADIUS: i32 = 24;

fn gaussian_weight(distance_sq: f32, sigma: f32) -> f32 {
    return exp(-distance_sq / (2.0 * sigma * sigma));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let texture_size = max(vec2<f32>(textureDimensions(screen_texture)), vec2<f32>(1.0));
    let screen_uv = clamp(in.position.xy / texture_size, vec2<f32>(0.0), vec2<f32>(1.0));
    let texel = vec2<f32>(1.0) / texture_size;

    let requested_radius = max(material.blur_radius_px, 0.0);
    let radius = i32(clamp(round(requested_radius), 0.0, f32(MAX_RADIUS)));
    let sample_step = max(requested_radius / max(f32(radius), 1.0), 1.0);
    let sigma = max(requested_radius * 0.5, 1.0);

    var sum = vec4<f32>(0.0);
    var weight_sum = 0.0;

    for (var y = -MAX_RADIUS; y <= MAX_RADIUS; y = y + 1) {
        if (abs(y) > radius) {
            continue;
        }

        for (var x = -MAX_RADIUS; x <= MAX_RADIUS; x = x + 1) {
            if (abs(x) > radius) {
                continue;
            }

            let offset = vec2<f32>(f32(x), f32(y)) * sample_step;
            let sample_uv = clamp(screen_uv + offset * texel, vec2<f32>(0.0), vec2<f32>(1.0));
            let sample_color = textureSample(screen_texture, screen_sampler, sample_uv);
            let weight = gaussian_weight(dot(offset, offset), sigma);
            sum += sample_color * weight;
            weight_sum += weight;
        }
    }

    let base_color = textureSample(screen_texture, screen_sampler, screen_uv);
    let blurred = select(base_color, sum / max(weight_sum, 0.0001), radius > 0);

    let background_alpha = clamp(material.tint.a, 0.0, 1.0);
    let glass_color = mix(blurred.rgb, material.tint.rgb, background_alpha);

    let point = in.uv * in.size - 0.5 * in.size;
    return draw_uinode_background(
        vec4<f32>(glass_color, 1.0),
        point,
        in.size,
        in.border_radius,
        in.border_widths,
    );
}
