#version 440

/* Shader input */
in vec4 v_color;
in vec3 v_position;

/* Shader output */
out vec3 out_albedo;
out vec3 out_normal;
out vec3 out_position;
out float out_light_depth;

void main() {
    out_albedo = v_color.rgb;
    out_normal = vec3(1.0);
    out_position = v_position;
    
    // FIXME:
    out_light_depth = 0.0;
}