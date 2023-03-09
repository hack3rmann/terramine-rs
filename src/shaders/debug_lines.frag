#version 440

/* Shader input */
in vec4 a_color;

/* Shader output */
out vec4 color;

void main() {
    color = a_color;
}