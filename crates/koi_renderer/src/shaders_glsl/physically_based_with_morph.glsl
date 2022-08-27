#VERTEX 

#INCLUDE scene_info

in mat4 ia_local_to_world;

in vec3 a_position;
in vec2 a_texture_coordinate;
in vec3 a_normal;
in vec4 a_color;

out vec2 f_texture_coordinates;
out vec3 f_world_position;
out vec3 f_normal;
out vec4 f_vertex_color;

uniform sampler3D u_morph_targets;
in vec3 u_morph_target_influences;

vec3 getMorph( 
    const in int vertexIndex,
    const in int morphTargetIndex,
    const in int offset,
    const in int stride,
    const in int width,
    const in int height )
{
    float texelIndex = float( vertexIndex * stride + offset );
    float y = floor( texelIndex / width );
    float x = texelIndex - y * width;
    vec3 morphUV = vec3( ( x + 0.5 ) / width, y / height, morphTargetIndex );
    return texture( u_morph_targets, morphUV ).xyz;
}

void main() 
{
    f_world_position = vec3(ia_local_to_world * vec4(a_position, 1.0));
    f_normal = mat3(ia_local_to_world) * a_normal;
    f_texture_coordinates = a_texture_coordinate;
    f_vertex_color = a_color;

    vec3 position = a_position;
    ivec3 texture_size = textureSize(u_morph_targets, 0);
    for (int i = 0; i < texture_size.z; i ++) {
        position += getMorph(gl_VertexID, i, 0, 0, texture_size.x, texture_size.y);
    }
    
    gl_Position = p_camera_to_screen * p_world_to_camera * ia_local_to_world * vec4(position, 1.0);
}

#FRAGMENT

#INCLUDE scene_info

precision mediump float;

// This shader is largely based on Filament's implementation:
// https://google.github.io/filament/Filament.md.html

// Inputs from the vertex shader
in vec2 f_texture_coordinates;
in vec3 f_world_position;  
in vec4 f_vertex_color;
in vec3 f_normal;

// Properties and textures
uniform int p_textures_enabled;

uniform vec4 p_base_color;
uniform sampler2D p_base_color_texture;

uniform float p_metallic;

// This value is squared on the CPU side before being passed in.
uniform float p_roughness;
uniform sampler2D p_metallic_roughness_texture;

// How much ambient light is visible to this model.
uniform float p_ambient;
// uniform sampler2D p_ambient_texture;

// Does this item produce its own light?
uniform vec3 p_emissive; 
// uniform sampler2D p_emissive_texture;

// uniform sampler2D p_normal_texture;

uniform float p_reflectance;

out vec4 color_out;

const float PI = 3.14159265359;
const float MEDIUMP_FLT_MAX = 65504.0;
const float MIN_ROUGHNESS = 0.007921;

float D_GGX(float roughness, float NoH, const vec3 n, const vec3 h) {
    vec3 NxH = cross(n, h);
    float a = NoH * roughness;
    float k = roughness / (dot(NxH, NxH) + a * a);
    float d = k * k * (1.0 / PI);
    return min(d, MEDIUMP_FLT_MAX);
}

// As explained on the Filament explainer this version is more accurate, 
// but the two square-roots is probably slower.

float V_SmithGGXCorrelated(float NoV, float NoL, float roughness) {
    float a2 = roughness * roughness;
    float GGXV = NoL * sqrt(NoV * NoV * (1.0 - a2) + a2);
    float GGXL = NoV * sqrt(NoL * NoL * (1.0 - a2) + a2);
    float v = 0.5 / (GGXV + GGXL);
    return min(v, MEDIUMP_FLT_MAX);
}

/*
float V_SmithGGXCorrelated(float roughness, float NoV, float NoL) {
     // Hammon 2017, "PBR Diffuse Lighting for GGX+Smith Microsurfaces"
    float v = 0.5 / mix(2.0 * NoL * NoV, NoL + NoV, roughness);
    return  min(v, MEDIUMP_FLT_MAX);
}
*/


/* 
vec3 F_Schlick(float u, vec3 f0, float f90) {
    return f0 + (vec3(f90) - f0) * pow(1.0 - u, 5.0);
}
*/


const float Fd_Lambert = 1.0 / PI;


// f90 (the reflectivity at 90 degree grazing angles) is set to 1.0
vec3 F_Schlick(float u, vec3 f0) {
    float f = pow(1.0 - u, 5.0);
    return f + f0 * (1.0 - f);
}

// This is significantly more complex than Fd_Lambert 
// but does manifest a nice 'glow' that appears to soften the edge.
// However Filament's docs note that this is not energy-conserving.
/*
float Fd_Burley(float NoV, float NoL, float LoH, float roughness) {
    float f90 = 0.5 + 2.0 * roughness * LoH * LoH;
    float lightScatter = F_Schlick(NoL, 1.0, f90);
    float viewScatter = F_Schlick(NoV, 1.0, f90);
    return lightScatter * viewScatter * (1.0 / PI);
}
*/


vec3 BRDF(vec3 v, vec3 n, vec3 base_color, float roughness, vec3 f0, const LightInfo light) {
    vec3 l;
    float attenuation;

    if (light.mode == 0) {
        l = light.inverse_direction;
        attenuation = 1.0;
    } else {
        vec3 diff = light.position - f_world_position;
        float distance = length(diff);
        l = diff / distance;
        attenuation = 1.0 / (distance * distance);
    };

    vec3 h = normalize(v + l);

    float NoV = abs(dot(n, v)) + 1e-5;
    float NoL = clamp(dot(n, l), 0.0, 1.0);
    float NoH = clamp(dot(n, h), 0.0, 1.0);
    float LoH = clamp(dot(l, h), 0.0, 1.0);

    float D = D_GGX(roughness, NoH, n, h);
    float V = V_SmithGGXCorrelated(roughness, NoV, NoL);
    vec3  F = F_Schlick(LoH, f0);

    // specular BRDF
    vec3 Fr = (D * V) * F;

    // diffuse BRDF
    vec3 Fd = base_color.rgb * Fd_Lambert;

    vec3 color = Fr + Fd;

    // apply lighting...
    return color * NoL * light.color_and_intensity * attenuation;
}

void main()
{
    // Only read from textures if they're set to enabled.
    bool base_color_texture_enabled = (p_textures_enabled & 0x1) > 0;
    bool metallic_roughness_texture_enabled = (p_textures_enabled & 0x2) > 0;

    vec3 n = normalize(f_normal);
    vec3 v = normalize(p_camera_position - f_world_position);

    vec4 base_color = p_base_color * f_vertex_color;
    if (base_color_texture_enabled) {
        base_color *= texture(p_base_color_texture, f_texture_coordinates);
    }

    float roughness = p_roughness;
    float metallic = p_metallic;
    if (metallic_roughness_texture_enabled) {
        vec4 metallic_roughness = texture(p_metallic_roughness_texture, f_texture_coordinates);
        metallic *= metallic_roughness.b;
        roughness *= metallic_roughness.g;
    }
    roughness = max(p_roughness, MIN_ROUGHNESS);

    vec3 diffuse_color = (1.0 - metallic) * base_color.rgb;
    vec3 f0 = 0.16 * vec3(p_reflectance * p_reflectance) * (1.0 - metallic) + base_color.rgb * metallic;

    color_out = vec4(0, 0, 0, 1);

    int count = int(light_count);
   // for (int i = 0; i < 9; i++) {
   //     if (i == count) break;
   //     color_out.rgb += BRDF(v, n, diffuse_color, roughness, f0, p_lights[i]);
   // }

    color_out.rgb += diffuse_color * read_spherical_harmonics(normalize(n));
    
    color_out.rgb = pow(color_out.rgb, vec3(1.0/2.2));
    
    // Clamp because Macs *will* display values outside gamut. 
    color_out = clamp(color_out, 0.0, 1.0);

}