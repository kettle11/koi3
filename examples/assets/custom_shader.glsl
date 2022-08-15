#VERTEX 
#INCLUDE standard_vertex
#FRAGMENT

in vec2 TexCoords;
out vec4 color_out;

void main()
{
  color_out = vec4(TexCoords, 0.0, 1.0);
}