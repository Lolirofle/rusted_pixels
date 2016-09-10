use glium;

use glium_ext;

pub struct State{
    pub display : glium_ext::GtkFacade,
    pub vertices: glium::VertexBuffer<Vertex>,
    pub indices : glium::IndexBuffer<u16>,
    pub program : glium::program::Program,
    pub texture : glium::texture::SrgbTexture2d, //The image as a texture
    pub zoom    : f32, //Scaling
    pub translation: [f32; 2], //Offset
    pub translation_previous_pos: Option<(f32,f32)>, //Used for retrieving translation
    pub dimensions: (f32,f32)//Dimensions of image area
}

#[derive(Copy,Clone)]
pub struct Vertex{
    pub position  : [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex,position,tex_coords);
