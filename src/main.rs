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
mod glium_ext;
mod image_ext;
mod input;
mod state;
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

                let button = gtk::Button::new_with_label("Click me!");
                    paned.add1(&button);

                let image_area = gtk::GLArea::new();
                    paned.add2(&image_area);
                    widget_impl::image::image_area(&image_area,&gl_state,&state);
                    widget_impl::input_system(&image_area,&state,input::get_commands());

            let command_input = gtk::TextView::new();
                vert_layout.pack_end(&command_input,false,false,0);
                widget_impl::command_input(&command_input);

        window.show_all();
        gtk::main();
    }else{
        println!("Failed to initialize GTK.");
    }
}

fn window_to_image_pos(pos: (f32,f32),gl_state: &gl_ext::State,state: &State) -> (f32,f32){
    let (tex_w,tex_h) = (//TODO: May be possible to replace this with state.image.dimensions() and not needing gl_state
        gl_state.texture.get_width() as f32,
        gl_state.texture.get_height().unwrap() as f32
    );

    (
        (pos.0 as f32-state.translation[0]-((gl_state.dimensions.0-tex_w*state.zoom)/2.0))/state.zoom,
        (pos.1 as f32-state.translation[1]-((gl_state.dimensions.1-tex_h*state.zoom)/2.0))/state.zoom
    )
}
