#version 140

uniform mat4 transformation;
in vec2 position;
in vec2 tex_coords;
out vec2 tex_coords_frag;

void main(){
    gl_Position = transformation * vec4(position,0.0,1.0);
    tex_coords_frag = tex_coords;
}
