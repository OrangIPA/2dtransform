#version 330 core
layout (location=0) in vec3 aPos;

uniform mat4 transform;
uniform mat4 cam_transform;

void main()
{
    gl_Position = transform * cam_transform * vec4(aPos, 1.0);
}