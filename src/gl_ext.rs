use glium;

use glium_ext;

pub struct ImageState{
    pub display                 : glium_ext::GtkFacade,
    pub vertices                : glium::VertexBuffer<Vertex>,
    pub vertices_draw           : glium::VertexBuffer<DrawingVertex>,
    pub indices                 : glium::IndexBuffer<u16>,
    pub program                 : glium::program::Program,
    pub drawing_program         : glium::program::Program,
    pub texture                 : glium::texture::Texture2d, //The image as a texture
    pub translation_previous_pos: Option<(f32,f32)>, //Used for retrieving translation
    pub dimensions              : (f32,f32)//Dimensions of image area
}

pub struct PreviewState{
    pub display : glium_ext::GtkFacade,
    pub vertices: glium::VertexBuffer<Vertex>,
    pub indices : glium::IndexBuffer<u16>,
    pub program : glium::program::Program,
    pub texture : glium::texture::SrgbTexture2d, //The image as a texture
    pub dimensions: (f32,f32)//Dimensions of image area
}

#[derive(Copy,Clone)]
pub struct Vertex{
    pub position  : [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex,position,tex_coords);


#[derive(Copy,Clone)]
pub struct DrawingVertex{
    pub position: [f32; 2],
    pub color   : [f32; 4],
}
implement_vertex!(DrawingVertex,position,color);
