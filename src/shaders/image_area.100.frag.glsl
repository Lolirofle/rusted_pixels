#version 100

uniform lowp sampler2D tex;
varying lowp vec2 tex_coords_frag;

void main(){
    gl_FragColor = texture2D(tex,tex_coords_frag);
}
