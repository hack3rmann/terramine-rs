#version 440

/* Vertex buffer inputs */
in vec3 position;
in vec3 color;
in uint face_idx;

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
uniform mat4 light_proj;
uniform mat4 light_view;
uniform bool is_shadow_pass;

void process_shadow();
void shade_standart();

vec3 get_normal(uint face_idx);

void main() {
    if (is_shadow_pass) {
        process_shadow();
    } else {
        shade_standart();
    }
}

void process_shadow() {
    v_position = position;
    gl_Position = light_proj * light_view * vec4(position, 1.0);
}

void shade_standart() {
    /* Assempling output compound */
    v_color = color;
    v_normal = get_normal(face_idx);
    v_position = position;
    v_time = time;
    v_light_dir = light_dir;

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