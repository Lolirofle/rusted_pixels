#version 100

uniform lowp mat3 transformation;
attribute lowp vec2 position;
attribute lowp vec2 tex_coords;
varying lowp vec2 tex_coords_frag;

void main() {
    gl_Position = vec4((transformation * vec3(position,1.0)).xy,0.0,1.0);
    tex_coords_frag = tex_coords;
}
