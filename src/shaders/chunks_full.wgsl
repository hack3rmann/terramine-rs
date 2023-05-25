struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) voxel_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
}

struct Transform {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: Transform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_pos = transform.proj * transform.view * vec4<f32>(in.position, 1.0);

    return out;
}



struct FragmentOutput {
    @location(0) albedo: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) position: vec3<f32>
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.albedo = vec3<f32>(1.0);
    out.normal = vec3<f32>(0.0);
    out.position = vec3<f32>(0.0);

    return out;
}