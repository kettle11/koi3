#VERTEX 
#INCLUDE standard_vertex
#FRAGMENT

in vec2 f_texture_coordinates;
out vec4 color_out;

void main()
{
  color_out = vec4(f_texture_coordinates, 0.0, 1.0);
}