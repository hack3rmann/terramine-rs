struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) face_idx: u32,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
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

    out.clip_pos = camera.proj * camera.view * vec4<f32>(in.position, 1.0);
    out.color = in.color;

    return out;
}



struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.albedo = vec4<f32>(in.color, 1.0);

    return out;
}