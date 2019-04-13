#version 150 core

in vec2 v_Coord;
out vec4 Target0;

void main() {
    Target0 = vec4(v_Coord.x, v_Coord.y, 0, 1);
}
