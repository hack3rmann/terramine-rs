struct VertexInput {
    @location(0)
    pos: vec3<f32>,

    @location(1)
    tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position)
    clip_pos: vec4<f32>,

    @location(1)
    tex_coords: vec2<f32>,
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

@group(2) @binding(0) 
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = in.tex_coords;
    out.clip_pos = camera.proj * camera.view * vec4<f32>(in.pos, 1.0);

    return out;
}



struct FragmentOutput {
    @location(0)
    frag_color: vec4<f32>,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.frag_color = textureSample(texture, tex_sampler, in.tex_coords);

    return out;
}