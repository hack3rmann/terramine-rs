struct CommonUniforms {
    resolution: vec2<f32>,
    time: f32,
}

@group(0)
@binding(0)
var<uniform> common_uniforms: CommonUniforms;

@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    let aspect_ratio = common_uniforms.resolution.y / common_uniforms.resolution.x;
    return vec4<f32>(pos.x * aspect_ratio, pos.y, pos.z, 1.0);
}

@fragment
fn fs_main(@builtin(position) clip_pos: vec4<f32>) -> @location(0) vec4<f32>  {
    return vec4<f32>(0.5, 0.1, 0.5, 1.0);
}