#version 140

in vec2 position;
in vec4 color;
out vec4 color_frag;

void main(){
    gl_Position = vec4((vec3(position,1.0)).xy,0.0,1.0);
    color_frag = color;
}
