#version 100

uniform lowp mat4 transformation;
attribute lowp vec2 position;
attribute lowp vec2 tex_coords;
varying lowp vec2 tex_coords_frag;

void main() {
    gl_Position = transformation * vec4(position,0.0,1.0);
    tex_coords_frag = tex_coords;
}
