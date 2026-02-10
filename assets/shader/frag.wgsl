#import bevy_pbr::forward_io::VertexOutput

@group(3) @binding(1) var terrain_texture: texture_2d<f32>;
@group(3) @binding(100) var<uniform> uniforms: ScreenUniforms;

struct ScreenUniforms {
    width: f32,
    height: f32,
    aspect_ratio: f32,
    focal_length: f32,

    camera_forward: vec3f,
    camera_up: vec3f,
    camera_right: vec3f,
    camera_pos: vec3f,

    min_height: f32,
    max_height: f32,
    scale_factor: f32
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let width: f32 = uniforms.width;
    let height: f32 = uniforms.height;
    let camera_forward: vec3f = uniforms.camera_forward;
    let camera_up: vec3f = uniforms.camera_up;
    let camera_right: vec3f = uniforms.camera_right;
    let camera_pos: vec3f = uniforms.camera_pos;
    let max_height: f32 = uniforms.max_height;
    let min_height: f32 = uniforms.min_height;
    let scale_factor: f32 = uniforms.scale_factor;
    let aspect_ratio: f32 = uniforms.aspect_ratio;
    let focal_length: f32 = uniforms.focal_length;
    let terrain_dimensions: vec2f = vec2f(textureDimensions(terrain_texture));

    var uv: vec2f = in.uv * 2.0 - 1.0;
    uv.x *= aspect_ratio;
    uv.y *= -1.0;

    let ray_origin: vec3f = camera_pos;
    let ray_direction: vec3f = project(uv, camera_forward, camera_up, camera_right, focal_length);

    let l: vec3f = vec3f(0.0, min_height * scale_factor, 0.0);
    let h: vec3f = vec3f(terrain_dimensions.x, max_height * scale_factor, terrain_dimensions.y);
    
    let bound_check: vec2f = ray_aabb(ray_origin, ray_direction, l, h);

    if bound_check.y >= max(bound_check.x, 0.0) {
        return terrain_intersect(ray_origin, ray_direction, bound_check.x, l, h);
    } else {
        return vec4f(0.0);
    }
}

fn project(uv: vec2f, forward: vec3f, up: vec3f, right: vec3f, focal_length: f32) -> vec3f {
    return normalize(forward * focal_length + right * uv.x + up * uv.y);
}

fn ray_aabb(ray_origin: vec3f, ray_dir: vec3f, aabb_min: vec3f, aabb_max: vec3f) -> vec2f {
    let inv_dir = 1.0 / ray_dir;

    let t0 = (aabb_min - ray_origin) * inv_dir;
    let t1 = (aabb_max - ray_origin) * inv_dir;

    let t_min = max(max(min(t0.x, t1.x), min(t0.y, t1.y)), min(t0.z, t1.z));
    let t_max = min(min(max(t0.x, t1.x), max(t0.y, t1.y)), max(t0.z, t1.z));

    return vec2f(t_min, t_max);
}

fn unpack_i16(x: u32) -> i32 {
    return i32(x << 16) >> 16;
}

fn sample_pixel(x: i32, y: i32) -> vec4f {
    let raw: vec4u = bitcast<vec4u>(textureLoad(terrain_texture, vec2i(x, y), 0));

    let r_i: i32 = unpack_i16(raw.x >> 16);
    let g_i: i32 = unpack_i16(raw.x & 0xFFFFu);
    let b_i: i32 = unpack_i16(raw.y >> 16);
    let a_i: i32 = unpack_i16(raw.y & 0xFFFFu);

    return vec4f(
        f32(r_i),
        f32(g_i),
        f32(b_i),
        f32(a_i),
    );
}

fn terrain_intersect(origin: vec3f, dir: vec3f, entry_t: f32, l: vec3f, h: vec3f) -> vec4f {
    var t: f32 = ceil(entry_t);
    let max_steps = 1024;

    for (var step = 0; step < max_steps; step++) {
        let p: vec3f = origin + t * dir;

        if p.x <= l.x || p.x >= h.x || p.z <= l.z || p.z >= h.z || p.y < l.y || p.y > h.y {
            return vec4f(0.0, 0.0, 0.0, 1.0);
        }

        let sample: vec4f = sample_pixel(i32(p.x), i32(p.z)) * uniforms.scale_factor;

        if p.y <= sample.r {
            return vec4f(sample.r / uniforms.max_height);
        }

        if (p + sample.a * dir).y > sample.g {
            t += sample.a;
        } else {
            t += floor(clamp(((p.y - sample.r) / (sample.b - dir.y)), 1.0, sample.a));
        }
    }

    return vec4f(0.0, 0.0, 0.0, 1.0);
}