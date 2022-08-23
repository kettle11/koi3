precision mediump float;

struct LightInfo {
    vec3 position;
    float radius;
    vec3 inverse_direction;
    float ambient;
    vec3 color_and_intensity;
    lowp int mode;
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

    uniform float p_exposure;
    lowp uint light_count;

    uniform vec4 spherical_harmonic_weights[9];
    
    uniform LightInfo p_lights[20];
}; 

vec3 read_spherical_harmonics(const vec3 n) {
    return max(
          spherical_harmonic_weights[0].rgb
        + spherical_harmonic_weights[1].rgb * (n.y)
        + spherical_harmonic_weights[2].rgb * (n.z)
        + spherical_harmonic_weights[3].rgb * (n.x)
        + spherical_harmonic_weights[4].rgb * (n.x * n.y)
        + spherical_harmonic_weights[5].rgb * (n.y * n.z)
        + spherical_harmonic_weights[6].rgb * (n.z * n.z - 0.3153915652535312)
        + spherical_harmonic_weights[7].rgb * (n.x * n.z)
        + spherical_harmonic_weights[8].rgb * (n.x * n.x - n.y * n.y)
        , 0.0);
}
