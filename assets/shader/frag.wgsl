#import bevy_pbr::forward_io::VertexOutput

@group(3) @binding(1) var terrain_texture: texture_2d<f32>;
@group(3) @binding(2) var terrain_sampler: sampler;
@group(3) @binding(100) var<uniform> width: f32;
@group(3) @binding(101) var<uniform> height: f32;
@group(3) @binding(102) var<uniform> forward_vec: vec3f;
@group(3) @binding(103) var<uniform> up_vec: vec3f;
@group(3) @binding(104) var<uniform> camera_pos: vec3f;
@group(3) @binding(105) var<uniform> max_height: f32;
@group(3) @binding(106) var<uniform> min_height: f32;
@group(3) @binding(107) var<uniform> scale_factor: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let terrain_dimensions: vec2f = vec2f(textureDimensions(terrain_texture));
    let aspect_ratio: f32 = width / height;

    var uv: vec2f = in.uv * 2.0 - 1.0;
    uv.x *= aspect_ratio;
    uv.y *= -1.0;

    let camera_forward: vec3f = normalize(forward_vec);
    let camera_right: vec3f = normalize(cross(camera_forward, normalize(up_vec)));
    let camera_up: vec3f = cross(camera_right, camera_forward);

    let ray_origin: vec3f = camera_pos;
    let ray_direction: vec3f = project(uv, camera_forward, camera_up, camera_right, 70.0);

    let l: vec3f = vec3f(0.0, min_height * scale_factor, 0.0);
    let h: vec3f = vec3f(terrain_dimensions.x, max_height * scale_factor, terrain_dimensions.y);
    
    let bound_check: vec2f = ray_aabb(ray_origin, ray_direction, l, h);

    if bound_check.y >= max(bound_check.x, 0.0) {
        return terrain_intersect(ray_origin, ray_direction, bound_check.x, l, h);
    } else {
        return vec4f(0.0);
    }

    // return vec4f(uv, 0.0, 1.0);

    // return raymarch(ray_origin, ray_direction);
}

fn project(uv: vec2f, forward: vec3f, up: vec3f, right: vec3f, fov: f32) -> vec3f {
    let focal_length: f32 = 1.0 / tan(radians(fov) * 0.5);
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

fn sample_pixel(x: i32, y: i32) -> vec4f {
    return textureLoad(terrain_texture, vec2<i32>(x, y), 0);
}

fn terrain_intersect(origin: vec3f, dir: vec3f, entry_t: f32, l: vec3f, h: vec3f) -> vec4f {
    var t: f32 = ceil(entry_t);
    let max_steps = 1024;

    for (var step = 0; step < max_steps; step++) {
        let p: vec3f = origin + t * dir;

        if p.x <= l.x || p.x >= h.x || p.z <= l.z || p.z >= h.z || p.y < l.y || p.y > h.y {
            return vec4f(0.0, 0.0, 0.0, 1.0);
        }

        let sample: vec4f = sample_pixel(i32(p.x), i32(p.z)) * scale_factor;

        if p.y <= sample.r {
            return vec4f(sample.r / max_height);
        }

        if (p + sample.a * dir).y > sample.g {
            t += sample.a;
        } else {
            t += floor(clamp(((p.y - sample.r) / (sample.b - dir.y)), 1.0, sample.a));
        }

    }

    return vec4f(0.0, 0.0, 0.0, 1.0);
}