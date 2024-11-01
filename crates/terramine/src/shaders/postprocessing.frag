#version 440

in vec2 frag_uv;

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
const vec4 DEFAULT_COLOR = vec4(0.21, 0.61, 0.61, 1.0);
const float Z_NEAR = 0.5;
const float Z_FAR = 10000.0;

const vec3 LIGHT_COLOR = vec3(0.4, 0.8, 0.2);
const float SHADOW_BRIGHTNESS = 0.05;

float linearize_depth(float d, float z_near, float z_far) {
    return z_near * z_far / (z_far + d * (z_near - z_far));
}

float get_depth(in vec2 uv) {
    vec4 depth = textureLod(depth_texture, uv * 0.5 + 0.5, 0.0);
    return linearize_depth(depth.r, Z_NEAR, Z_FAR);
}

vec3 get_albedo() {
    vec3 albedo = texture(albedo_texture, frag_uv).rgb;
    return vec3(
        pow(albedo.r, 1.0 / (0.4545 * 0.4545)),
        pow(albedo.g, 1.0 / (0.4545 * 0.4545)),
        pow(albedo.b, 1.0 / (0.4545 * 0.4545))
    );
    return albedo;
}

vec3 get_normal() {
    vec3 normal_color = texture(normal_texture, frag_uv).xyz;
    return normal_color;
}

vec3 get_position() {
    return texture(position_texture, frag_uv).xyz;
}

float get_light_depth() {
    float depth = texture(light_depth_texture, frag_uv).r;
    return linearize_depth(depth, 1.0, 200.0);
}

/// Projects `pos` by `projection` matrix and outputs uvs vector in [-1.0; 1.0]
vec3 projection_uvs(in vec3 pos, in mat4 projection) {
    vec4 projected = projection * vec4(pos, 1.0);

    vec3 proj_coords = projected.xyz / projected.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    return proj_coords;
}

float get_shadow(in vec3 pos, float current_depth) {
    float shadow_glitch_brightness_shift = 0.13;

    vec3 proj_coords = projection_uvs(pos, light_proj0 * light_view0);

    if (proj_coords.x < 0.0 || 1.0 < proj_coords.x ||
        proj_coords.y < 0.0 || 1.0 < proj_coords.y ||
        proj_coords.z < 0.0 || 1.0 < proj_coords.z)
    { return 1.0; }

    float closest_depth = texture(light_depth_texture, proj_coords.xy).r;
    closest_depth = linearize_depth(closest_depth, Z_NEAR, Z_FAR);
    current_depth = linearize_depth(proj_coords.z, Z_NEAR, Z_FAR);
    bool is_shadow = current_depth - 0.00003 > closest_depth;

    out_color = vec4(vec3(closest_depth), 1.0);

    return is_shadow
        ? SHADOW_BRIGHTNESS
        : 1.0 + shadow_glitch_brightness_shift;
}

float get_shadow_multisampled(in vec3 frag_pos, float current_depth) {
    vec2 offset = 0.66 * 0.2 * vec2(1, 0);
    //frag_pos = floor(frag_pos * 8.0) / 8.0;

    float nearby[6];
    nearby[0] = get_shadow(frag_pos + offset.xyy, current_depth);
    nearby[1] = get_shadow(frag_pos - offset.xyy, current_depth);
    nearby[2] = get_shadow(frag_pos + offset.yyx, current_depth);
    nearby[3] = get_shadow(frag_pos - offset.yyx, current_depth);
    nearby[4] = get_shadow(frag_pos + offset.yxy, current_depth);
    nearby[5] = get_shadow(frag_pos - offset.yxy, current_depth);

    float centre_weight = 0.6;
    float nearby_weight = 0.1;

    float shadow_summary = centre_weight * get_shadow(frag_pos, current_depth);

    for (uint i = uint(0); i < uint(6); ++i)
        shadow_summary += 0.0;//nearby[i] * nearby_weight;

    return shadow_summary;// / (centre_weight + 6.0 * nearby_weight);
}

bool is_uv_saturated(in vec2 uv) {
    return uv.x * uv.x <= 1.0 && uv.y * uv.y <= 1.0;
}

float ss_shadow(in vec3 pos, in vec3 to_light) {
    const float RAY_MAX_DISTANCE = 0.2;
    const float THICKNESS = 0.025;
    const uint  MAX_N_STEPS = uint(16);
    const float RAY_STEP_LEN = RAY_MAX_DISTANCE / float(MAX_N_STEPS);

    // Compute ray position and direction (in view space)
    vec3 ray_origin = (view * vec4(pos, 1.0)).xyz;
    vec3 ray_dir = (view * vec4(to_light, 0.0)).xyz;

    // Ray march towards the light
    float occlusion = 0.0;

    for (uint i = uint(0); i < MAX_N_STEPS; i++) {
        // Step the ray
        vec3 ray_pos = ray_origin + float(i) * ray_dir * RAY_STEP_LEN;
        vec2 ray_uv = projection_uvs(ray_pos, proj).xy;

        // Ensure the UV coordinates are inside the screen
        if (!is_uv_saturated(ray_uv))
            continue;

        // Compute the difference between the ray's and the camera's depth
        float depth_z = get_depth(ray_uv * 2.0 - 1.0);
        float depth_delta = depth_z - ray_pos.z;

        // Check if the camera can't "see" the ray (ray depth must be larger than the camera depth, so positive depth_delta)
        if (depth_delta > 0.0 && depth_delta < THICKNESS) {
            // Mark as occluded
            occlusion = ray_pos.z;

            // Fade out as we approach the edges of the screen
            // occlusion *= screen_fade(ray_uv);

            //break;
        }
    }

    // Convert to visibility
    return 1.0 - occlusion;
}

bool is_cross() {
    vec2 crosshair_sizes = vec2(3.5, 21.5);

    vec2 pos = (frag_uv.xy * 2.0 - 1.0) * screen_resolution;
    pos = round(pos);

    return abs(pos.x) <= crosshair_sizes.x && abs(pos.y) <= crosshair_sizes.y ||
           abs(pos.x) <= crosshair_sizes.y && abs(pos.y) <= crosshair_sizes.x;
}

float fresnel_multiple(float power, float strength, in vec3 to_cam, in vec3 normal) {
    return strength * pow(1.0 - max(dot(to_cam, normal), 0.0), power);
}

float specular_multiple(float power, float strength, in vec3 to_cam, in vec3 to_light, in vec3 normal) {
    vec3 reflected_to_cam = reflect(to_cam, normal);
    return strength * pow(max(dot(reflected_to_cam, -to_light), 0.0), power);
}

float diffuse_brightness(float min_brightness, in vec3 normal, in vec3 to_light) {
    return max(min_brightness, dot(normal, to_light));
}

void main() {
    float depth = get_depth(frag_uv * 2.0 - 1.0);
    vec3 albedo = get_albedo();
    vec3 normal = get_normal();
    vec3 position = get_position();

    float light_depth = 0.0;
    if (render_shadows)
        light_depth = get_light_depth();

    if (depth > Z_FAR * 0.5)
        out_color = DEFAULT_COLOR;
    else {
        vec3 to_light_dir = -light_dir0;
        vec3 to_cam = normalize(cam_pos - position);

        float diffuse = diffuse_brightness(0.05, normal, to_light_dir);
        float specular = specular_multiple(12.0, 0.01, to_cam, to_light_dir, normal);
        float fresnel = fresnel_multiple(20.0, 0.01, to_cam, normal);

        float shadow = render_shadows
            ? get_shadow_multisampled(position, depth)
            : 0.25;

        out_color = vec4(albedo * diffuse + (fresnel + specular) * DEFAULT_COLOR.rgb, 1.0) * shadow * 4.0;

        /* Simple gamma-correction */
        out_color = vec4(
            pow(out_color.r, 0.4545),
            pow(out_color.g, 0.4545),
            pow(out_color.b, 0.4545),
            1.0
        );
    }

    if (is_cross()) {
        out_color = (1.0 - out_color) * 0.5;
        out_color.a = 1.0;
    }
}