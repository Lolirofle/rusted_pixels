use glium;

use glium_ext;

pub struct State{
    pub display : glium_ext::GtkFacade,
    pub indices : glium::IndexBuffer<u16>,
    pub program : glium::program::Program,
    pub texture : glium::texture::SrgbTexture2d,
}

#[derive(Copy,Clone)]
pub struct Vertex{
    pub position  : [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex,position,tex_coords);
