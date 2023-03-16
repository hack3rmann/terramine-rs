#version 440

/* Vertex buffer inputs */
in vec3 position;
in vec3 color;
in vec3 normal;

/* Output compound */
out vec3 v_color;
out vec3 v_normal;
out vec3 v_position;
out vec3 v_light_dir;
out float v_time;

uniform float time;
uniform mat4 proj;
uniform mat4 view;
uniform vec3 light_dir;
uniform vec3 light_pos;

void main() {
    /* Assempling output compound */
    v_color = color;
    v_normal = normal;
    v_position = position;
    v_time = time;
    v_light_dir = light_dir;

    /* Writing to gl_Position */
    gl_Position = proj * view * vec4(position, 1.0);
}
