#version 150 core

uniform sampler2D t_InColour;
in vec2 v_Coord;
out vec4 Target0;

void main() {
    Target0 = texture(t_InColour, v_Coord);
}
