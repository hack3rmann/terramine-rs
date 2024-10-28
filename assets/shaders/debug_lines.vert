#version 440

/* Shader inputs */
in vec3 pos;
in vec4 color;

/* Shader output */
out vec4 v_color;
out vec3 v_position;

/* Uniforms */
uniform float time;
uniform mat4 proj;
uniform mat4 view;

void main() {
    v_color = color;

    gl_Position = proj * view * vec4(pos, 1.0);
}