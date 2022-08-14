// layout (std140) uniform ub_SceneInfo
// {
    // Also known as 'view'
   uniform mat4 world_to_camera;
    // Also known as 'projection'
   uniform mat4 camera_to_screen;
// }; 

in mat4 ia_local_to_world;

in vec3 a_position;
in vec2 a_texture_coordinate;
in vec3 a_normal;
in vec4 a_color;

out vec2 TexCoords;
out vec3 WorldPosition;
out vec3 Normal;
out vec4 VertexColor;

void main()
{
    WorldPosition = vec3(ia_local_to_world * vec4(a_position, 1.0));
    Normal = mat3(ia_local_to_world) * a_normal;
    TexCoords = a_texture_coordinate;
    VertexColor = a_color;
    
    gl_Position = camera_to_screen * world_to_camera * ia_local_to_world * vec4(a_position, 1.0);
}
