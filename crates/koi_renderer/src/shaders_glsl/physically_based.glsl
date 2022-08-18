#VERTEX 

#INCLUDE standard_vertex

#FRAGMENT

#INCLUDE scene_info

// This shader is largely based on Filament's implementation:
// https://google.github.io/filament/Filament.md.html

// TODO: 
// Bitpack if a texture is assigned and conditionally avoid 
// texture reads.

// Inputs from the vertex shader
in vec2 f_texture_coordinates;
in vec3 f_world_position;  
in vec4 f_vertex_color;
in vec3 f_normal;

// Properties and textures
uniform vec4 p_base_color;
// uniform sampler2D p_base_color_texture;

uniform float p_metallic;
uniform float p_roughness;
// uniform sampler2D p_metallic_roughness_texture;

// How much ambient light is visible to this model.
uniform float p_ambient;
// uniform sampler2D p_ambient_texture;

// Does this item produce its own light?
uniform vec3 p_emissive; 
// uniform sampler2D p_emissive_texture;

// uniform sampler2D p_normal_texture;

out vec4 color_out;

const float PI = 3.14159265359;
const float MEDIUMP_FLT_MAX = 65504.0;

float D_GGX(float roughness, float NoH, const vec3 n, const vec3 h) {
    vec3 NxH = cross(n, h);
    float a = NoH * roughness;
    float k = roughness / (dot(NxH, NxH) + a * a);
    float d = k * k * (1.0 / PI);
    return min(d, MEDIUMP_FLT_MAX);
}

// As explained on the Filament explainer this version is more accurate, 
// but the two square-roots is probably slower.
/*
float V_SmithGGXCorrelated(float NoV, float NoL, float roughness) {
    float a2 = roughness * roughness;
    float GGXV = NoL * sqrt(NoV * NoV * (1.0 - a2) + a2);
    float GGXL = NoV * sqrt(NoL * NoL * (1.0 - a2) + a2);
    return 0.5 / (GGXV + GGXL);
}
*/

float V_SmithGGXCorrelated(float NoV, float NoL, float roughness) {
    float a = roughness;
    float GGXV = NoL * (NoV * (1.0 - a) + a);
    float GGXL = NoV * (NoL * (1.0 - a) + a);
    return 0.5 / (GGXV + GGXL);
}

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


// TODO: Make roughness perceptually linear by using Filament's square approach.
vec3 BRDF(vec3 v, vec3 n, float roughness, vec3 f0, const LightInfo light) {
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

    float D = D_GGX(NoH, roughness, n, h);
    float V = V_SmithGGXCorrelated(NoV, NoL, roughness);
    vec3  F = F_Schlick(LoH, f0);

    // specular BRDF
    vec3 Fr = (D * V) * F;

    // diffuse BRDF
    vec3 Fd = p_base_color.rgb * Fd_Lambert;

    vec3 color = Fr + Fd;

    // apply lighting...
    return color * NoL * light.color_and_intensity * attenuation;
}

void main()
{
    vec3 n = normalize(f_normal);
    vec3 v = normalize(p_camera_position - f_world_position);

    vec3 f0 = vec3(0.16 * 0.25);

    color_out = vec4(0, 0, 0, 1);
    for (int i = 0; i < light_count; i++) {
      color_out.rgb += BRDF(v, n, p_roughness, f0, p_lights[i]);
    }
}