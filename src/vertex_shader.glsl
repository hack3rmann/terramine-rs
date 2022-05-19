#version 140

/* Vertex buffer inputs */
in vec2 position;
in vec2 tex_coords;

/* Output compound */
out float a_Time;
out vec2 a_Tex_Coords;

/* Time uniform */
uniform float time;

void main() {
    /* Assempling output compound */
    a_Time = time;
    a_Tex_Coords = tex_coords;

    /* Writing to gl_Position */
    gl_Position = vec4(position, 0.0, 1.0);
}
