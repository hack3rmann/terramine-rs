#version 140

/* Input compound */
in float a_Time;
in vec2 a_Tex_Coords;

/* Output */
out vec4 color;

uniform sampler2D tex;

void main() {
    /* Time stuff */
    //float time = a_Time;
    //float time_sine = (sin(time) + 1.0f) / 2.0f;
    //float time_cosine = (cos(time) + 1.0f) / 2.0f;

    /* Export color */
    //color = vec4(u_Color_Time.r + time_sine, u_Color_Time.g + time_cosine, u_Color_Time.b + (time_sine + time_cosine) / 2.0f, 1.0);
    color = texture(tex, a_Tex_Coords);
}