#version 440

in vec4 position;
in vec2 texcoord;

out vec2 v_frag_texcoord;

void main() {
    v_frag_texcoord = texcoord;
    gl_Position = position;
}