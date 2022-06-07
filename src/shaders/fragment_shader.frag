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
    color = vec4(texture(tex, a_Tex_Coords).rgb * u_light, 1.0);
}