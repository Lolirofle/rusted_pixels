#![allow(dead_code)]

extern crate core;
extern crate epoxy;
extern crate image;
extern crate gl;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate shared_library;
#[macro_use]extern crate glium;

#[macro_use]mod macros;
mod color;
mod gl_ext;
mod glium_gtk;
mod image_ext;
mod input;
mod state;
mod util;
mod widget_impl;
mod x11_keymap;

use core::cell::RefCell;
use core::ptr;
use gtk::prelude::*;
use shared_library::dynamic_library::DynamicLibrary;
use std::{fs,io};
use std::rc::Rc;

use state::State;

pub fn main() {
    if let Ok(_) = gtk::init(){
        //Window initialization
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
            window.set_title("Rusted Pixels");
            window.set_border_width(4);
            window.set_position(gtk::WindowPosition::Center);
            window.set_default_size(800,600);
            window.connect_delete_event(|_,_|{
                gtk::main_quit();
                Inhibit(false)
            });

        //GL initialization (Loading symbols)
        epoxy::load_with(|s| unsafe{
            match DynamicLibrary::open(None).unwrap().symbol(s){
                Ok(v) => v,
                Err(e) => {
                    println!("{:?}",e);
                    ptr::null()
                },
            }
        });

        //Data initialization
        let gl_state = Rc::new(RefCell::new(None));
        let state    = Rc::new(RefCell::new(State{
            images: vec![
                image::load(
                    io::BufReader::new(fs::File::open("test.png").unwrap()),
                    image::PNG
                ).unwrap().to_rgba()
            ],
            ..state::State::new()
        }));

        //Window layout
        let vert_layout = gtk::Box::new(gtk::Orientation::Vertical,0);
            window.add(&vert_layout);

            let menu_bar = gtk::MenuBar::new();
                vert_layout.pack_start(&menu_bar,false,false,0);

                let menu_item = gtk::MenuItem::new_with_label("File");
                    menu_bar.append(&menu_item);

                let menu_item = gtk::MenuItem::new_with_label("Edit");
                    menu_bar.append(&menu_item);

            let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
                vert_layout.pack_start(&paned,true,true,0);
                widget_impl::input_system(&vert_layout,&state,input::get_commands());

                let image_area = gtk::GLArea::new();
                let preview_area = gtk::GLArea::new();
                    paned.add2(&image_area);
                    widget_impl::image::image_area(&image_area,&preview_area,&gl_state,&state);

                let split_left = gtk::Box::new(gtk::Orientation::Vertical,0);
                    paned.add1(&split_left);

                    let grid = gtk::Grid::new();
                        split_left.pack_start(&grid,false,true,2);
                        widget_impl::color_chooser(&grid,&state);


                    widget_impl::image::preview_area(&preview_area,&image_area,&gl_state);

            let command_input = gtk::TextView::new();
                vert_layout.pack_end(&command_input,false,false,0);
                widget_impl::command_input(&command_input);

        window.show_all();

        split_left.pack_end(&preview_area,true,true,0);
        preview_area.show();

        gtk::main();
    }else{
        println!("Failed to initialize GTK.");
    }
}

fn window_to_image_pos(pos: (f64,f64),gl_state: &gl_ext::ImageState,state: &State) -> (f64,f64){
    let (tex_w,tex_h) = (
        gl_state.texture.get_width() as f64,
        gl_state.texture.get_height().unwrap() as f64
    );

    (
        (pos.0 - state.translation[0]-(gl_state.dimensions.0-tex_w*state.zoom)/2.0)/state.zoom,
        (pos.1 - state.translation[1]-(gl_state.dimensions.1-tex_h*state.zoom)/2.0)/state.zoom
    )
}

fn image_to_gl_pos(pos: (f64,f64),gl_state: &gl_ext::ImageState) -> (f64,f64){
    let (tex_w,tex_h) = (
        gl_state.texture.get_width() as f64,
        gl_state.texture.get_height().unwrap() as f64
    );

    (
         (pos.0/(tex_w/2.0) - 1.0),
        -(pos.1/(tex_h/2.0) - 1.0),
    )
}


fn window_to_gl_pos(pos: (f64,f64),gl_state: &gl_ext::ImageState,state: &State) -> (f64,f64){
    let (tex_w,tex_h) = (
        gl_state.texture.get_width() as f64,
        gl_state.texture.get_height().unwrap() as f64
    );

    //±(window_to_image_pos(x,y)/(tex_dim/2.0) - 1.0)
    (
         ((pos.0 - state.translation[0])*2.0 - gl_state.dimensions.0)/tex_w/state.zoom,
        -((pos.1 - state.translation[1])*2.0 - gl_state.dimensions.1)/tex_h/state.zoom,
    )
}
