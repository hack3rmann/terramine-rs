#version 440

/* Vertex buffer inputs */
in vec3 position;
in vec3 color;
in float light;

/* Output compound */
out vec3 a_color;
out float a_light;

uniform float time;
uniform mat4 proj;
uniform mat4 view;

void main() {
    /* Assempling output compound */
    a_color = color;
    a_light = light;

    /* Writing to gl_Position */
    gl_Position = proj * view * vec4(position, 1.0);
}
