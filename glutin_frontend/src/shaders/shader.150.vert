#version 150 core

in vec2 a_CornerZeroToOne;
out vec2 v_Coord;
out vec2 v_PixelCoord;

void main() {
    v_Coord = a_CornerZeroToOne;
    v_PixelCoord = a_CornerZeroToOne * vec2(256., 240.);
    gl_Position = vec4(a_CornerZeroToOne.x * 2 - 1, 1 - a_CornerZeroToOne.y * 2, 0, 1);
}
