#version 440

in vec4 position;
in vec2 texcoord;

out vec2 frag_uv;

void main() {
    frag_uv = texcoord;
    gl_Position = position;
}