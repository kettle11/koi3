#VERTEX 
#INCLUDE standard_vertex
#FRAGMENT

in vec2 f_texture_coordinates;
in vec4 f_vertex_color;

uniform vec2 p_texture_coordinate_offset;
uniform vec2 p_texture_coordinate_scale;

uniform vec4 p_base_color;
uniform sampler2D p_base_color_texture;

out vec4 color_out;

void main()
{
  vec4 base_color = (f_vertex_color * p_base_color * texture(p_base_color_texture, f_texture_coordinates));
  color_out = base_color;
}