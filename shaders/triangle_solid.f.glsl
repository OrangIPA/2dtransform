#version 330 core
out vec4 FragColor;

uniform vec4 rgba;

void main()
{
    FragColor = vec4(rgba);
}