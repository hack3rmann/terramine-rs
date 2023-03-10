#version 440

/* Vertex buffer inputs */
in vec3 position;
in vec2 tex_coords;
in vec3 normal;

/* Output compound */
out vec2 v_tex_coords;
out vec3 v_normal;
out vec3 v_position;
out vec3 v_light_dir;
out float v_time;

uniform float time;
uniform mat4 proj;
uniform mat4 view;
uniform vec3 light_dir;

void main() {
    /* Assembling output compound */
    v_tex_coords = tex_coords;
    v_normal = normal;
    v_time = time;
    v_light_dir = light_dir;
    v_position = position;

    /* Writing to gl_Position */
    gl_Position = proj * view * vec4(position, 1.0);
}
