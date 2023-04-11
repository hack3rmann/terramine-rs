#version 440

/* Vertex buffer inputs */
in vec3 position;
in vec2 tex_coords;
in uint face_idx;

/* Output compound */
out vec2 v_tex_coords;
out vec3 v_normal;
out vec3 v_tangent;
out vec3 v_bitangent;
out vec3 v_position;
out mat3 v_to_world;

uniform float time;
uniform mat4 proj;
uniform mat4 view;

uniform vec3 light_dir0;
uniform vec3 light_pos0;
uniform mat4 light_proj0;
uniform mat4 light_view0;

uniform bool is_shadow_pass;

vec3 normals[] = {
    vec3(1, 0, 0),
    vec3(-1, 0, 0),
    vec3(0, 1, 0),
    vec3(0, -1, 0),
    vec3(0, 0, 1),
    vec3(0, 0, -1)
};

vec3 tangents[] = {
    vec3(0, 1, 0),
    vec3(0, 1, 0),
    vec3(-1, 0, 0),
    vec3(-1, 0, 0),
    vec3(0, 1, 0),
    vec3(0, 1, 0)
};

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
    gl_Position = light_proj0 * light_view0 * vec4(position, 1.0);
}

void shade_standart() {
    /* Assembling output compound */
    v_tex_coords = tex_coords;
    v_normal = normals[face_idx];
    v_tangent = tangents[face_idx];
    v_bitangent = cross(v_normal, v_tangent);
    v_position = position;

    mat3 to_local = mat3(
        v_bitangent.x, v_tangent.x, v_normal.x,
        v_bitangent.y, v_tangent.y, v_normal.y,
        v_bitangent.z, v_tangent.z, v_normal.z
    );
    v_to_world = inverse(to_local);

    /* Writing to gl_Position */
    gl_Position = proj * view * vec4(position, 1.0);
}