#VERTEX 

uniform mat4 p_view;
uniform mat4 p_projection;

in vec3 a_position;

out vec3 local_position;

void main()
{
    local_position = a_position;  
    gl_Position = p_projection * p_view * vec4(a_position, 1.0);
}

#FRAGMENT

// Because the model's transform is untransformed this will be local.
in vec3 local_position;
uniform sampler2D p_equirectangular_texture;

out vec4 color_out;

const vec2 inv_atan = vec2(0.1591, 0.3183);

vec2 sample_spherical_map(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= inv_atan;
    uv += 0.5;
    return uv;
}

void main()
{		
    vec2 uv = sample_spherical_map(normalize(local_position));
    uv.y = 1.0 - uv.y;
    vec3 color = texture(p_equirectangular_texture, uv).rgb;
    color_out = vec4(color, 1.0);
}