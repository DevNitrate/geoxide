#import bevy_pbr::forward_io::VertexOutput

@group(3) @binding(1) var screen_texture: texture_2d<u32>;
@group(3) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(textureSample(screen_texture, texture_sampler, in.uv)*-1);
    // return vec4f(in.uv, 0.0, 1.0);
}