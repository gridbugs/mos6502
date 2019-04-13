#version 150 core

in vec2 a_CornerZeroToOne;
out vec2 v_Coord;

void main() {
    v_Coord = a_CornerZeroToOne;
    gl_Position = vec4(a_CornerZeroToOne.x * 2 - 1, 1 - a_CornerZeroToOne.y * 2, 0, 1);
}
