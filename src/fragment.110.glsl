#version 110

uniform sampler2D tex;
varying vec2 tex_coords_frag;

void main() {
    gl_FragColor = texture2D(tex,tex_coords_frag);
}
