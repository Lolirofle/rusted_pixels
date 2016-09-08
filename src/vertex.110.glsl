#version 110

uniform mat4 transformation;
attribute vec2 position;
attribute vec2 tex_coords;
varying vec2 tex_coords_frag;

void main() {
    gl_Position = transformation * vec4(position,0.0,1.0);
    tex_coords_frag = tex_coords;
}
