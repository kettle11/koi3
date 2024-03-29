#VERTEX 
#INCLUDE standard_vertex
#FRAGMENT

in vec2 f_texture_coordinates;
in vec4 f_vertex_color;

uniform vec2 p_texture_coordinate_offset;
uniform vec2 p_texture_coordinate_scale;

uniform vec4 p_base_color;
uniform sampler2D sp0_base_color_texture;

out vec4 color_out;

void main()
{
  vec4 base_color = (f_vertex_color * p_base_color * texture(sp0_base_color_texture, f_texture_coordinates));
  if (base_color.a == 0.0) {
    discard;
  }
  color_out = base_color;
}