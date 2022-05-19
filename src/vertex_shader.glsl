#version 440

/* Vertex buffer inputs */
in vec2 position;
in vec2 tex_coords;

/* Output compound */
out vec2 a_Tex_Coords;

void main() {
    /* Assempling output compound */
    a_Tex_Coords = tex_coords;

    /* Writing to gl_Position */
    gl_Position = vec4(position, 0.0, 1.0);
}
