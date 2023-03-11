#version 440

/* Input compound */
in vec2 v_tex_coords;
in vec3 v_normal;
in vec3 v_tangent;
in vec3 v_bitangent;
in vec3 v_position;
in vec3 v_light_dir;
in float v_time;

/* Output */
out vec3 out_albedo;
out vec3 out_normal;
out vec3 out_position;

/* Texture samplter */
uniform sampler2D texture_atlas;
uniform sampler2D normal_atlas;

void main() {
    vec4 tex_color = texture(texture_atlas, v_tex_coords);

    /* load normal from normal map and unexpose it */
    vec3 local_normal_exp = texture(normal_atlas, v_tex_coords).xyz;
    vec3 local_normal = vec3(
        pow(local_normal_exp.x, 1.0 / 0.4545),
        pow(local_normal_exp.y, 1.0 / 0.4545),
        pow(local_normal_exp.z, 1.0 / 0.4545)
    );

    mat3 to_local = mat3(
        v_bitangent.x, v_tangent.x, v_normal.x,
        v_bitangent.y, v_tangent.y, v_normal.y,
        v_bitangent.z, v_tangent.z, v_normal.z
    );
    mat3 to_world = inverse(to_local);

    if (tex_color.a < 0.001)
        discard;

    out_albedo = tex_color.rgb;
    out_normal = to_world * local_normal;
    out_position = v_position;
}