#import bevy_pbr::forward_io::VertexOutput

@group(3) @binding(1) var screen_texture: texture_2d<f32>;
@group(3) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let resolution = textureDimensions(screen_texture);
    let aspect_ratio: f32 = f32(resolution.x) / f32(resolution.y);

    var uv: vec2f = in.uv;
    uv.x *= aspect_ratio;

    return vec4f(textureSample(screen_texture, texture_sampler, in.uv));
    // return vec4f(in.uv, 0.0, 1.0);
}