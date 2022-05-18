#version 140

/* Vertex buffer inputs */
in vec2 position;
in vec3 color;

/* Output compound */
out vec4 u_Color_Time;

/* Time uniform */
uniform mat4 transform;
uniform float time;

void main() {
    /* Assempling output compound */
    u_Color_Time = vec4(color.rgb, time);

    /* Writing to gl_Position */
    gl_Position = transform * vec4(position, 0.0, 1.0);
}
