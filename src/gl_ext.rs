use glium;

use glium_ext;

pub struct State{
    pub display : glium_ext::GtkFacade,
    pub vertices: glium::VertexBuffer<Vertex>,
    pub indices : glium::IndexBuffer<u16>,
    pub program : glium::program::Program,
}

#[derive(Copy,Clone)]
pub struct Vertex{
    pub position  : [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex,position,tex_coords);
