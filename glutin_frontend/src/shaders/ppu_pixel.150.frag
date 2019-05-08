#version 150 core

in vec2 v_Coord;
in vec2 v_PixelCoord;
out vec4 Target0;
uniform samplerBuffer t_PixelColours;

void main() {
    int pixel_colour_index = int(v_PixelCoord.y) * 256 + int(v_PixelCoord.x);
    vec4 pixel_colour = texelFetch(t_PixelColours, pixel_colour_index);
    Target0 = pixel_colour;
}
