#version 150 core

uniform sampler2D t_InColour;
in vec2 v_Coord;
out vec4 Target0;

void main() {
    // cancel-out gamma correction
    float gamma = 2.2;
    Target0 = pow(texture(t_InColour, v_Coord), vec4(gamma));
}
