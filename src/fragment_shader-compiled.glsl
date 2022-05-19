#version 140
#define GLSLIFY 1

/* Input compound */
in vec2 a_Tex_Coords;

/* Output */
out vec4 color;

/* Texture samplter */
uniform sampler2D tex;

void main() {
    /* Export color */
    color = texture(tex, a_Tex_Coords);
}