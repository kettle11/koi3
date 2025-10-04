precision mediump float;

#INCLUDE scene_info

in mat4 ia_local_to_world;
in vec4 ia_color;

in vec3 a_position;
in vec2 a_texture_coordinate;
in vec3 a_normal;
in vec4 a_color;

out vec2 f_texture_coordinates;
out vec3 f_world_position;
out vec3 f_normal;
out vec4 f_vertex_color;

void main()
{
    f_world_position = vec3(ia_local_to_world * vec4(a_position, 1.0));
    f_normal = mat3(ia_local_to_world) * a_normal;
    f_texture_coordinates = a_texture_coordinate;
    f_vertex_color = a_color * ia_color;
    
    gl_Position = p_camera_to_screen * p_world_to_camera * ia_local_to_world * vec4(a_position, 1.0);
}
