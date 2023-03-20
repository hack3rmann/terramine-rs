#version 440

/* Vertex buffer inputs */
in vec3 position;
in vec2 tex_coords;
in vec3 normal;
in vec3 tangent;

/* Output compound */
out vec2 v_tex_coords;
out vec3 v_normal;
out vec3 v_tangent;
out vec3 v_bitangent;
out vec3 v_position;
out mat3 to_world;

uniform float time;
uniform mat4 proj;
uniform mat4 view;
uniform vec3 light_dir;
uniform vec3 light_pos;
uniform mat4 light_proj;
uniform mat4 light_view;
uniform bool is_shadow_pass;

void process_shadow();
void shade_standart();

void main() {
    if (is_shadow_pass) {
        process_shadow();
    } else {
        shade_standart();
    }
}

void process_shadow() {
    /* Assembling output compound */
    v_position = position;

    /* Writing to gl_Position */
    gl_Position = light_proj * light_view * vec4(position, 1.0);
}

void shade_standart() {
    /* Assembling output compound */
    v_tex_coords = tex_coords;
    v_normal = normal;
    v_tangent = tangent;
    v_bitangent = cross(normal, tangent);
    v_position = position;

    mat3 to_local = mat3(
        v_bitangent.x, v_tangent.x, v_normal.x,
        v_bitangent.y, v_tangent.y, v_normal.y,
        v_bitangent.z, v_tangent.z, v_normal.z
    );
    to_world = inverse(to_local);

    /* Writing to gl_Position */
    gl_Position = proj * view * vec4(position, 1.0);
}