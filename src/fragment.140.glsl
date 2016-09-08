#version 140

uniform sampler2D tex;
in vec2 tex_coords_frag;
out vec4 color_frag;

void main() {
    color_frag = texture(tex,tex_coords_frag);
}
