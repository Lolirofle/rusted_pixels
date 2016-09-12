#version 110

uniform mat3 transformation;
attribute vec2 position;
attribute vec2 tex_coords;
varying vec2 tex_coords_frag;

void main() {
    gl_Position = vec4((transformation * vec3(position,1.0)).xy,0.0,1.0);
    tex_coords_frag = tex_coords;
}
