#version 440
#define GLSLIFY 1

/* Shader inputs */
in vec3 pos;
in vec4 color;

/* Shader output */
out vec4 a_color;

/* Uniforms */
uniform float time;
uniform mat4 proj;
uniform mat4 view;

void main() {
	a_color = color;

	gl_Position = proj * view * vec4(pos, 1.0);
}