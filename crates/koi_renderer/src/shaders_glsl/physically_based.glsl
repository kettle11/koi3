#VERTEX 

#INCLUDE standard_vertex

#FRAGMENT

in vec2 f_texture_coordinates;
in vec3 f_world_position;  
in vec4 f_vertex_color;
in vec3 f_normal;

uniform vec2 p_texture_coordinate_offset;
uniform vec2 p_texture_coordinate_scale;

uniform vec4 p_base_color;
uniform sampler2D p_base_color_texture;

out vec4 color_out;

void main()
{
  vec4 base_color = (f_vertex_color * p_base_color * texture(p_base_color_texture, f_texture_coordinates * p_texture_coordinate_scale + p_texture_coordinate_offset));
  color_out = vec4(1.0, 0.0, 1.0, 1.0);
}