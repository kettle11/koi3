struct LightInfo {
    vec3 position;
    float radius;
    vec3 inverse_direction;
    float ambient;
    vec3 color_and_intensity;
    int mode;
    mat4 world_to_light; 
};

layout (std140) uniform ub0_scene_info
{
    // Also known as 'view'
    uniform mat4 p_world_to_camera;
    // Also known as 'projection'
    uniform mat4 p_camera_to_screen;

    uniform vec3 p_camera_position;
    uniform float p_dither_scale;

    uniform float p_fog_start;
    uniform float p_fog_end;

    uniform float __padding;
    uniform uint light_count;

    uniform LightInfo p_lights[100];
}; 
