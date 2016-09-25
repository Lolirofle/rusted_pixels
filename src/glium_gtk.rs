use epoxy;
use glium;
use gtk;
use gtk::prelude::*;
use std::ops;
use std::os::raw::c_void;
use std::rc::Rc;

pub struct GtkGLAreaBackend{
    pub gl_area: gtk::GLArea,
}

unsafe impl glium::backend::Backend for GtkGLAreaBackend{
    fn swap_buffers(&self) -> Result<(),glium::SwapBuffersError>{
        self.gl_area.queue_render();
        Ok(())
    }

    unsafe fn get_proc_address(&self,symbol: &str) -> *const c_void{
        epoxy::get_proc_addr(symbol) as *const c_void
    }

    fn get_framebuffer_dimensions(&self) -> (u32,u32){
        (self.gl_area.get_allocated_width() as u32,self.gl_area.get_allocated_height() as u32)
    }

    fn is_current(&self) -> bool{
        unsafe{self.make_current()};
        true
    }

    unsafe fn make_current(&self){
        if self.gl_area.get_realized(){
            self.gl_area.make_current();
        }
    }
}

pub struct GtkFacade{
    pub context: Rc<glium::backend::Context>,
}

impl glium::backend::Facade for GtkFacade{
    fn get_context(&self) -> &Rc<glium::backend::Context>{
        &self.context
    }
}

impl GtkFacade{
    pub fn draw(&self) -> glium::Frame{
        glium::Frame::new(self.context.clone(),self.get_framebuffer_dimensions())
    }

    pub fn draw_with_dimensions(&self,dims: (u32,u32)) -> glium::Frame{
        glium::Frame::new(self.context.clone(),dims)
    }
}

impl ops::Deref for GtkFacade{
    type Target = glium::backend::Context;

    #[inline]
    fn deref(&self) -> &glium::backend::Context{
        &self.context
    }
}
