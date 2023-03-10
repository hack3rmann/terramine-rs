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

void main() {
    out_albedo = v_color;
    out_normal = v_normal * 0.5 + vec3(0.5);
    out_position = v_position;
}