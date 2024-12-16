#version 140

in vec2 v_tex_coords;

out vec4 colour;

uniform sampler2D tex;

void main() {
    colour = texture(tex, v_tex_coords);
}