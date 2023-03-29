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

vec3 get_normal(uint face_idx);
vec3 get_tangent(uint face_idx);

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
    v_normal = get_normal(face_idx);
    v_tangent = get_tangent(face_idx);
    v_bitangent = cross(v_normal, v_tangent);
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

vec3 get_normal(uint face_idx) {
    switch (face_idx) {
        case 0: return vec3(1, 0, 0);
        case 1: return vec3(-1, 0, 0);
        case 2: return vec3(0, 1, 0);
        case 3: return vec3(0, -1, 0);
        case 4: return vec3(0, 0, 1);
        case 5: return vec3(0, 0, -1);
        
        default:
            return vec3(-1, -1, -1);
    }
}

vec3 get_tangent(uint face_idx) {
    switch (face_idx) {
        case 0:
        case 1:
        case 4:
        case 5:
            return vec3(0, 1, 0);

        case 2:
        case 3:
            return vec3(-1, 0, 0);

        default:
            return vec3(-1, -1, -1);
    }
}