#version 440

/* Vertex buffer inputs */
in vec3 position;
in vec2 tex_coords;

/* Output compound */
out vec2 a_Tex_Coords;

uniform float time;
uniform mat4 proj;
uniform mat4 view;

void main() {
    /* Assempling output compound */
    a_Tex_Coords = tex_coords;

    /* Writing to gl_Position */
    gl_Position = proj * view * vec4(position, 1.0);
}
