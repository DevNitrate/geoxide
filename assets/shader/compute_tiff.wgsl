@group(0) @binding(0) var<storage, read_write> ssbo: array<i32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) invocation_id: vec3u) {
    ssbo[invocation_id.x] = 0;
}