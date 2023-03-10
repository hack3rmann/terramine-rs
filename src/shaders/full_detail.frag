#version 440

/* Input compound */
in vec2 v_tex_coords;
in vec3 v_normal;
in vec3 v_position;
in vec3 v_light_dir;
in float v_time;

/* Output */
out vec3 out_albedo;
out vec3 out_normal;
out vec3 out_position;

/* Texture samplter */
uniform sampler2D tex;

void main() {
    vec4 tex_color = texture(tex, v_tex_coords);

    if (tex_color.a < 0.001)
        discard;

    out_albedo = tex_color.rgb;
    out_normal = v_normal * 0.5 + vec3(0.5);
    out_position = v_position;
}