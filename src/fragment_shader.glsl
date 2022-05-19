#version 140

/* Input compound */
in float a_Time;
in vec2 a_Tex_Coords;

/* Output */
out vec4 color;

/* Texture samplter */
uniform sampler2D tex;

void main() {
    /* Export color */
    color = texture(tex, a_Tex_Coords);
}