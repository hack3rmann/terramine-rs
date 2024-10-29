#version 440

in vec2 v_frag_texcoord;

out vec4 out_color;

uniform sampler2D depth_texture;
uniform sampler2D albedo_texture;
uniform sampler2D normal_texture;
uniform sampler2D position_texture;
uniform sampler2D light_depth_texture;
uniform float time;

uniform vec3 light_dir0;
uniform vec3 light_pos0;
uniform mat4 light_proj0;
uniform mat4 light_view0;

uniform vec3 light_dir1;
uniform vec3 light_pos1;
uniform mat4 light_proj1;
uniform mat4 light_view1;

uniform vec2 screen_resolution;
uniform vec3 cam_pos;
uniform mat4 proj;
uniform mat4 view;
uniform bool render_shadows;

/// These constants are shared. See cfg module.
vec4 default_color = vec4(0.01, 0.01, 0.01, 1.0);
float z_near = 0.5;
float z_far = 10000.0;

vec3 light_color = vec3(0.4, 0.8, 0.2);
float shadow_brightness = 0.05;

float linearize_depth(float d, float z_near, float z_far) {
    return z_near * z_far / (z_far + d * (z_near - z_far));
}

float get_depth() {
    vec4 depth = texture(depth_texture, v_frag_texcoord);
    return linearize_depth(depth.r, z_near, z_far);
}

vec3 get_albedo() {
    vec3 albedo = texture(albedo_texture, v_frag_texcoord).rgb;
    return vec3(
        pow(albedo.r, 1.0 / (0.4545 * 0.4545)),
        pow(albedo.g, 1.0 / (0.4545 * 0.4545)),
        pow(albedo.b, 1.0 / (0.4545 * 0.4545))
    );
    return albedo;
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

float get_shadow_small(vec4 frag_pos, float current_depth) {
    float shadow_glitch_brightness_shift = 0.13;

    vec4 frag_pos_light_space = light_proj0 * light_view0 * frag_pos;
    vec3 proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    if (proj_coords.x < 0.0 || 1.0 < proj_coords.x ||
        proj_coords.y < 0.0 || 1.0 < proj_coords.y ||
        proj_coords.z < 0.0 || 1.0 < proj_coords.z)
    { return 1.0; }

    float closest_depth = texture(light_depth_texture, proj_coords.xy).r;
    closest_depth = linearize_depth(closest_depth, z_near, z_far);
    current_depth = linearize_depth(proj_coords.z, z_near, z_far);
    bool is_shadow = current_depth - 0.0003 > closest_depth;

    out_color = vec4(vec3(closest_depth), 1.0);

    return is_shadow
        ? shadow_brightness
        : 1.0 + shadow_glitch_brightness_shift;
}

float get_shadow(vec4 frag_pos, float current_depth) {
    vec2 offset = 0.2 * vec2(1, 0);
    frag_pos = floor(frag_pos * 8.0) * 0.125;

    float nearby[6];
    nearby[0] = get_shadow_small(frag_pos + offset.xyyy, current_depth);
    nearby[1] = get_shadow_small(frag_pos - offset.xyyy, current_depth);
    nearby[2] = get_shadow_small(frag_pos + offset.yyxy, current_depth);
    nearby[3] = get_shadow_small(frag_pos - offset.yyxy, current_depth);
    nearby[4] = get_shadow_small(frag_pos + offset.yxyy, current_depth);
    nearby[5] = get_shadow_small(frag_pos - offset.yxyy, current_depth);

    float centre_weight = 0.6;
    float nearby_weight = 0.1;

    float shadow_summary = centre_weight * get_shadow_small(frag_pos, current_depth);

    for (uint i = uint(0); i < uint(6); ++i)
        shadow_summary += nearby[i] * nearby_weight;

    return shadow_summary / (centre_weight + 6.0 * nearby_weight);
}

bool is_cross() {
    vec2 crosshair_sizes = vec2(3.5, 21.5);

    vec2 pos = (v_frag_texcoord.xy * 2.0 - 1.0) * screen_resolution;
    pos = round(pos);

    return abs(pos.x) <= crosshair_sizes.x && abs(pos.y) <= crosshair_sizes.y ||
           abs(pos.x) <= crosshair_sizes.y && abs(pos.y) <= crosshair_sizes.x;
}

void main() {
    float depth = get_depth();
    vec3 albedo = get_albedo();
    vec3 normal = get_normal();
    vec3 position = get_position();

    float light_depth = 0.0;
    if (render_shadows)
        light_depth = get_light_depth();

    if (normal == vec3(0.0) || depth > z_far * 0.5) {
        out_color = default_color;
        if (is_cross()) {
            out_color = (1.0 - out_color) * 0.5;
            out_color.a = 1.0;
        }
        return;
    }

    vec3 to_light_dir = -light_dir0;
    float brightness = max(0.05, dot(normal, to_light_dir));

    vec3 to_cam = normalize(cam_pos - position);
    vec3 reflected_to_cam = reflect(to_cam, normal);

    float specular_power = 12.0;
    float specular_multiplier = 0.01;
    float specular = pow(max(dot(reflected_to_cam, -to_light_dir), 0.0), specular_power) * specular_multiplier;

    float fresnel_power = 20.0;
    float fresnel_multiplier = 0.002;
    float fresnel = pow(1.0 - max(dot(to_cam, normal), 0.0), fresnel_power) * fresnel_multiplier;

    float shadow = 0.25;
    if (render_shadows)
        shadow = get_shadow(vec4(position, 1.0), depth);

    out_color = vec4(albedo * brightness + fresnel + specular, 1.0) * shadow * 4.0;

    /* Simple gamma-correction */
    out_color = vec4(
        pow(out_color.r, 0.4545),
        pow(out_color.g, 0.4545),
        pow(out_color.b, 0.4545),
        1.0
    );

    if (is_cross()) {
        out_color = (1.0 - out_color) * 0.5;
        out_color.a = 1.0;
    }
}
