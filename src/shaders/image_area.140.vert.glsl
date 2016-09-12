#version 140

uniform mat3 transformation;
in vec2 position;
in vec2 tex_coords;
out vec2 tex_coords_frag;

void main(){
    gl_Position = vec4((transformation * vec3(position,1.0)).xy,0.0,1.0);
    tex_coords_frag = tex_coords;
}
