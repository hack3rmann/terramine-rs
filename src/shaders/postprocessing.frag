#version 440

in vec2 v_frag_texcoord;

out vec4 color;

uniform sampler2D depth_texture;
uniform sampler2D albedo_texture;
uniform sampler2D normal_texture;
uniform sampler2D position_texture;
uniform float time;
uniform vec3 light_dir;
uniform vec3 cam_pos;

// FIXME: make shared constants with the rust's cfg module
vec3 light_color = vec3(0.4, 0.8, 0.2);
vec4 default_color = vec4(0.01, 0.01, 0.01, 1.0);
float z_near = 0.5;
float z_far = 10000.0;

float linearize_depth(float d, float z_near, float z_far) {
    return z_near * z_far / (z_far + d * (z_near - z_far));
}

float get_depth() {
    vec4 depth = texture(depth_texture, v_frag_texcoord);
    return linearize_depth(depth.r, z_near, z_far);
}

vec3 get_albedo() {
    return texture(albedo_texture, v_frag_texcoord).rgb;
}

vec3 get_normal() {
    vec3 normal_color = texture(normal_texture, v_frag_texcoord).xyz;
    return normal_color;
}

vec3 get_position() {
    return texture(position_texture, v_frag_texcoord).xyz;
}

void main() {
    float depth = get_depth();
    vec3 albedo = get_albedo();
    vec3 normal = get_normal();
    vec3 position = get_position();

    if (normal == vec3(0.0) || depth > z_far * 0.5) {
        color = default_color;
        return;
    }

    vec3 to_light_dir = -light_dir;
    float brightness = max(0.3, dot(normal, to_light_dir));

    vec3 to_cam = normalize(cam_pos - position);
    vec3 reflected_to_cam = reflect(to_cam, normal);

    float specular_power = 12.0;
    float specular_multiplier = 0.05;
    float specular = pow(max(dot(reflected_to_cam, -to_light_dir), 0.0), specular_power) * specular_multiplier;

    float fresnel_power = 12.0;
    float fresnel_multiplier = 0.04;
    float fresnel = pow(1.0 - dot(to_cam, normal), fresnel_power) * fresnel_multiplier;

    color = vec4(brightness * albedo + specular + fresnel, 1.0);
}