#import bevy_pbr::forward_io::VertexOutput

@group(3) @binding(1) var screen_texture: texture_2d<f32>;
@group(3) @binding(2) var texture_sampler: sampler;
@group(3) @binding(100) var<uniform> width: u32;
@group(3) @binding(101) var<uniform> height: u32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let aspect_ratio: f32 = f32(width) / f32(height);

    var uv: vec2f = in.uv;
    uv.x *= aspect_ratio;

    return vec4f(textureSample(screen_texture, texture_sampler, in.uv));
}