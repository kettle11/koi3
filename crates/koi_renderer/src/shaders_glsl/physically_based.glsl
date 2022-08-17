#VERTEX 

#INCLUDE standard_vertex

#FRAGMENT

#INCLUDE scene_info

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

void main()
{
  color_out = vec4(p_lights[0].color_and_intensity, 1.0);
}