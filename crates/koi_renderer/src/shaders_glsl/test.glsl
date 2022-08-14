#VERTEX
#INCLUDE standard_vertex
#FRAGMENT

in vec2 TexCoords;
in vec3 WorldPosition;  
in vec4 VertexColor;
in vec3 Normal;

out vec4 color_out;

void main()
{
  color_out = vec4(1.0, 0.0, 0.0, 1.0);
}