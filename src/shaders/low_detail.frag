#version 440

/* Input compound */
in vec3 v_color;
in vec3 v_normal;
in vec3 v_position;
in vec3 v_light_dir;
in float v_time;

/* Output */
out vec3 out_albedo;
out vec3 out_normal;
out vec3 out_position;

uniform sampler2D texture_atlas;
uniform sampler2D normal_atlas;

uniform vec3 light_pos0;
uniform vec3 light_dir0;

uniform bool is_shadow_pass;
uniform float time;

void process_shadow();
void shade_standart();

void main() {
    if (is_shadow_pass) {
        process_shadow();
    } else {
        shade_standart();
    }
}

void shade_standart() {
    out_albedo = 1.02 * vec3(
        pow(v_color.r, 0.4545),
        pow(v_color.g, 0.4545),
        pow(v_color.b, 0.4545)
    );
    out_albedo = 0.95 * v_color;
    out_normal = v_normal;
    out_position = v_position;
}

void process_shadow() {
    out_position = v_position;
    out_albedo = vec3(0.0);
    out_normal = vec3(0.0);
}