#version 140

in vec2 position;
in vec3 color;

out vec3 u_Color;

void main() {
    u_Color = color;

    gl_Position = vec4(position, 0.0, 1.0);
}
