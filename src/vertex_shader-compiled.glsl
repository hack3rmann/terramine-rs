#version 140
#define GLSLIFY 1

in vec2 position;
in vec3 color;

out vec4 u_Color_Time;

uniform float time;

void main() {
    u_Color_Time = vec4(color.rgb, time);

    gl_Position = vec4(position, 0.0, 1.0);
}
