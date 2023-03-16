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
out float out_light_depth;

uniform vec3 light_pos;

void main() {
    out_albedo = v_color;
    out_normal = v_normal;
    out_position = v_position;
    out_light_depth = length(v_position - light_pos);
}