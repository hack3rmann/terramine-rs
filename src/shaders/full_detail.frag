#version 440

/* Input compound */
in vec2 a_Tex_Coords;

/* Output */
out vec4 color;
in float u_light;

/* Texture samplter */
uniform sampler2D tex;

void main() {
    /* Export color */
    vec4 tex_color = texture(tex, a_Tex_Coords);

    if (tex_color.a < 0.001)
        discard;

    color = vec4(tex_color.rgb * u_light, 1.0);
}