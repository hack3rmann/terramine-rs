#version 440

/* Input compound */
in vec3 a_color;
in float a_light;

/* Output */
out vec4 color;

void main() {
    color = vec4(a_color * a_light, 1.0);
}