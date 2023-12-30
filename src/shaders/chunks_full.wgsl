struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) face_idx: u32,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,

    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) face_idx: u32,
}

struct CommonUniforms {
    resolution: vec2<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> common_uniforms: CommonUniforms;

struct CameraUniform {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
}

@group(1) @binding(0) 
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_pos = camera.proj * camera.view * vec4(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    out.face_idx = in.face_idx;
    out.position = in.position;

    return out;
}



@group(2) @binding(0)
var albedo_atlas: texture_2d<f32>;

@group(2) @binding(1)
var albedo_sampler: sampler;

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.albedo = textureSample(albedo_atlas, albedo_sampler, in.tex_coords);

    return out;
}