#version 140
#define GLSLIFY 1

/* Vertex buffer inputs */
in vec2 position;
in vec3 color;

/* Output compound */
out vec4 u_Color_Time;

/* Time uniform */
uniform float time;

void main() {
    /* Assempling output compound */
    u_Color_Time = vec4(color.rgb, time);

    /* Writing to gl_Position */
    gl_Position = vec4(position, 0.0, 1.0);
}
