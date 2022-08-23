#VERTEX

#INCLUDE scene_info

in vec3 a_position;

out vec3 local_position;

void main()
{
    mat4 view_rotation = mat4(mat3(p_world_to_camera)); // remove translation from the view matrix
    vec4 clip_pos = p_camera_to_screen * view_rotation * vec4(a_position, 1.0);

    gl_Position = clip_pos.xyww;
    local_position = a_position;
}

#FRAGMENT

#INCLUDE scene_info

in vec3 local_position;
out vec4 color_out;

uniform samplerCube p_cube_map;
  
void main()
{
    color_out.rgb = texture(p_cube_map, local_position).rgb * p_exposure;
    color_out.rgb = pow(color_out.rgb, vec3(1.0/2.2));

    //color_out.a = 1.0;
   // color_out = vec4(1.0, 0.0, 0.0, 1.0);
}