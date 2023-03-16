#version 440

in vec2 v_frag_texcoord;

out vec4 color;

uniform sampler2D depth_texture;
uniform sampler2D albedo_texture;
uniform sampler2D normal_texture;
uniform sampler2D position_texture;
uniform sampler2D light_depth_texture;
uniform float time;
uniform vec3 light_dir;
uniform vec3 light_pos;
uniform mat4 light_proj;
uniform mat4 light_view;
uniform vec3 cam_pos;
uniform mat4 proj;
uniform mat4 view;

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

float get_light_depth() {
    float depth = texture(light_depth_texture, v_frag_texcoord).r;
    return linearize_depth(depth, 1.0, 200.0);
}

bool is_shadow(vec4 frag_pos, float current_depth) {
    vec4 frag_pos_light_space = light_proj * light_view * frag_pos;
    vec3 proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    if (proj_coords.x < 0.0 || 1.0 < proj_coords.x ||
        proj_coords.y < 0.0 || 1.0 < proj_coords.y ||
        proj_coords.z < 0.0 || 1.0 < proj_coords.z)
    { return false; }

    float closest_depth = texture(light_depth_texture, proj_coords.xy).r;
    closest_depth = linearize_depth(closest_depth, z_near, z_far);
    current_depth = linearize_depth(proj_coords.z, z_near, z_far);
    bool is_shadow = current_depth - 0.003 > closest_depth;

    color = vec4(vec3(closest_depth), 1.0);

    return is_shadow;
}

void main() {
    float depth = get_depth();
    vec3 albedo = get_albedo();
    vec3 normal = get_normal();
    vec3 position = get_position();
    float light_depth = get_light_depth();

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
    float fresnel = pow(1.0 - max(dot(to_cam, normal), 0.0), fresnel_power) * fresnel_multiplier;

    bool is_shadow = is_shadow(vec4(position, 1.0), depth);

    color = vec4(albedo * brightness + fresnel + specular, 1.0) * (is_shadow ? 0.2 : 1.0);
    //color = vec4(vec3(light_depth / 160.0), 1.0);
}