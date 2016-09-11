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

mod color;
mod gl_ext;
mod glium_ext;
mod image_ext;
mod input;
mod state;
mod x11_keymap;
//mod windows;

use core::cell::RefCell;
use core::ptr;
use glium::Surface;
use gtk::prelude::*;
use shared_library::dynamic_library::DynamicLibrary;
use std::{fs,io,path};
use std::rc::Rc;

use input::*;
use state::State;

macro_rules! move_fn_with_clones{
    ($($n:ident),+; || $body:block) => (
        {
            $( let $n = $n.clone(); )+
            move || { $body }
        }
    );
    ($($n:ident),+; |$($p:pat),+| $body:block) => (
        {
            $( let $n = $n.clone(); )+
            move |$($p),+| { $body }
        }
    );
}

pub fn main() {
    if let Ok(_) = gtk::init(){

        //Window initialization
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
            window.set_title("Rusted Pixels");
            window.set_border_width(4);
            window.set_position(gtk::WindowPosition::Center);
            window.set_default_size(800,600);
            window.connect_key_press_event(|_,event_key|{
                println!("{:?} {:?} {:?}",event_key.get_keyval(),event_key.get_hardware_keycode(),event_key.get_state());
                //ALT+X: 120 53 MOD1_MASK
                Inhibit(false)
            });
            window.connect_delete_event(|_,_|{
                gtk::main_quit();
                Inhibit(false)
            });

        //Data initialization
        let gl_state: Rc<RefCell<Option<gl_ext::State>>> = Rc::new(RefCell::new(None));
        let state: Rc<RefCell<State>> = Rc::new(RefCell::new(State{
            images: vec![
                image::load(
                    io::BufReader::new(fs::File::open("test.png").unwrap()),
                    image::PNG
                ).unwrap().to_rgba()
            ],
            ..state::State::new()
        }));
		let commands = input::get_commands();

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

                paned.connect_key_press_event(move_fn_with_clones!(state; |_,event|{
                    let mut state = state.borrow_mut();
                    use gdk::enums::key::{
                        plus        as PLUS,
                        minus       as MINUS,
                        KP_Add      as KP_PLUS,
                        KP_Subtract as KP_MINUS,
                    };
                    match event.get_keyval(){
                        KEY_PLUS  | KEY_KP_PLUS  => {state.zoom*=2.0;},
                        KEY_MINUS | KEY_KP_MINUS => {state.zoom/=2.0;},
                        key  => {
                            /*// every command begins with a single key
				            if state.input.is_empty() {
				                state.input.push(Input::Char(keycode,keymod));
				                match execute_command(&mut state, &commands) {
				                    CommandResult::Quit => { break 'main_loop },
				                    _ => {}
				                }
				            }
				            // If escape is pressed, clear input buffer or pop
				            // input stack
				            else if keycode == Keycode::Escape {
				                if !state.input_buffer.is_empty() {
				                    state.input_buffer = String::new();
				                } else {
				                    state.input.pop();
				                }
				            }
				            else if keycode == Keycode::Return {
				                let (input_type, arg)
				                    = input::parse_input(&state.input_buffer);
				                state.input.push(input_type);
				                if let Some(arg) = arg {
				                    state.args.push(arg);
				                }
				                state.input_buffer = String::new();
				                match execute_command(&mut state, &commands) {
				                    CommandResult::Quit => { break 'main_loop },
				                    _ => {}
				                }
				            }
				            else {
				                if let Some(chr) = input::keycode_to_char(keycode) {
				                    state.input_buffer.push(chr);
				                    println!("{:?}", state.input_buffer.as_str());
				                }
				            }*/
                        }
                    };
                    Inhibit(false)
                }));

                {let button = gtk::Button::new_with_label("Click me!");
                    paned.add1(&button);
                }

                let image_area = gtk::GLArea::new();
                    paned.add2(&image_area);

                    //Load GL symbols
                    epoxy::load_with(|s| unsafe{
                        match DynamicLibrary::open(None).unwrap().symbol(s){
                            Ok(v) => v,
                            Err(_) => ptr::null(),
                        }
                    });

                    //Initialization of draw area
                    image_area.connect_realize(move_fn_with_clones!(state,gl_state; |widget|{
                        //Wrapper struct for glium
                        let display = glium_ext::GtkFacade{
                            context: unsafe{
                                glium::backend::Context::new::<_,()>(
                                    glium_ext::GtkBackend{gl_area: widget.clone()},
                                    true,
                                    Default::default()
                                )
                            }.unwrap(),
                        };
                        //GL shader data for an image
                        let vertices = glium::VertexBuffer::new(&display,&[
                            gl_ext::Vertex{position: [-1.0,-1.0],tex_coords: [0.0,0.0]},
                            gl_ext::Vertex{position: [-1.0, 1.0],tex_coords: [0.0,1.0]},
                            gl_ext::Vertex{position: [ 1.0, 1.0],tex_coords: [1.0,1.0]},
                            gl_ext::Vertex{position: [ 1.0,-1.0],tex_coords: [1.0,0.0]},
                        ]).unwrap();
                        let indices = glium::IndexBuffer::new(
                            &display,
                            glium::index::PrimitiveType::TriangleStrip,
                            &[1,2,0,3u16]
                        ).unwrap();
                        //GL shaders
                        let program = program!(&display,
                            140 => {
                                vertex  : include_str!(  "vertex.140.glsl"),
                                fragment: include_str!("fragment.140.glsl"),
                            },
                            110 => {
                                vertex  : include_str!(  "vertex.110.glsl"),
                                fragment: include_str!("fragment.110.glsl"),
                            },
                            100 => {
                                vertex  : include_str!(  "vertex.100.glsl"),
                                fragment: include_str!("fragment.100.glsl"),
                            },
                        ).unwrap();

                        let state = state.borrow();
                        let image_dimensions = state.images[0].dimensions();
                        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                            state.images[0].clone().into_raw(),//TODO
                            image_dimensions
                        );
                        let texture = glium::texture::SrgbTexture2d::new(&display,image).unwrap();

                        let mut gl_state = gl_state.borrow_mut();
                        *gl_state = Some(gl_ext::State{
                            display : display,
                            vertices: vertices,
                            indices : indices,
                            program : program,
                            texture : texture,
                            translation_previous_pos: None,
                            dimensions: (1.0,1.0),
                        });
                    }));

                    //Finalization of draw area
                    image_area.connect_unrealize(move_fn_with_clones!(gl_state; |_|{
                        let mut gl_state = gl_state.borrow_mut();
                        *gl_state = None;
                    }));

                    //Resize of draw area
                    image_area.connect_resize(move_fn_with_clones!(gl_state; |_,w,h|{
                        let mut gl_state = gl_state.borrow_mut();
                        if let Some(gl_state) = gl_state.as_mut(){
                            gl_state.dimensions = (w as f32,h as f32);
                        }
                    }));

                    //Drawing of draw area for every frame
                    image_area.connect_render(move_fn_with_clones!(state,gl_state; |_,_|{
                        let state = state.borrow();
                        let gl_state = gl_state.borrow();
                        if let Some(gl_state) = gl_state.as_ref(){
                            let mut target = gl_state.display.draw();
                                let (tex_w,tex_h) = (gl_state.texture.get_width() as f32,gl_state.texture.get_height().unwrap() as f32);
                                let (scale_x,scale_y) = (
                                    1.0/gl_state.dimensions.0*tex_w*state.zoom,
                                    1.0/gl_state.dimensions.1*tex_h*state.zoom,
                                );
                                target.clear_color(0.3,0.3,0.3,1.0);
                                target.draw(
                                    &gl_state.vertices,
                                    &gl_state.indices,
                                    &gl_state.program,
                                    &uniform!{
                                        transformation: [//Translation*Scale transformation matrix
                                            [ scale_x, 0.0, 0.0],
                                            [ 0.0, scale_y, 0.0],
                                            [ state.translation[0]/gl_state.dimensions.0*2.0, -state.translation[1]/gl_state.dimensions.1*2.0, 1.0f32]
                                        ],//TODO: The translation seem slightly incorrect (Almost not noticable)
                                        tex: gl_state.texture
                                            .sampled()
                                            .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                                            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                                    },
                                    &Default::default()
                                ).unwrap();
                            target.finish().unwrap();
                        }

                        Inhibit(false)
                    }));

                    //Scroll to zoom
                    image_area.add_events(gdk_sys::GDK_SCROLL_MASK.bits() as i32);
                    image_area.add_events(gdk_sys::GDK_SMOOTH_SCROLL_MASK.bits() as i32);
                    image_area.connect_scroll_event(move_fn_with_clones!(state; |_,event|{
                        let mut state = state.borrow_mut();
                        let (_,delta) = event.get_delta();
                        if delta>0.0{
                            state.zoom/=2.0;
                        }else if delta<0.0{
                            state.zoom*=2.0;
                        }
                        Inhibit(false)
                    }));

                    //When releasing mouse buttons
                    image_area.add_events(gdk_sys::GDK_ALL_EVENTS_MASK.bits() as i32);
                    image_area.connect_button_release_event(move_fn_with_clones!(gl_state; |_,event|{
                        let mut gl_state = gl_state.borrow_mut();
                        if let Some(gl_state) = gl_state.as_mut(){
                            if event.get_state().contains(gdk::BUTTON1_MASK){
                                gl_state.translation_previous_pos = None;
                            }
                        }
                        Inhibit(false)
                    }));

                    //When moving mouse cursor
                    image_area.connect_motion_notify_event(move_fn_with_clones!(state,gl_state; |_,event|{
                        let mut state = state.borrow_mut();
                        let mut gl_state = gl_state.borrow_mut();
                        if let Some(gl_state) = gl_state.as_mut(){
                            if event.get_state().contains(gdk::BUTTON1_MASK){
                                let pos = event.get_position();
                                let pos = (pos.0 as f32,pos.1 as f32);

                                match &mut gl_state.translation_previous_pos{
                                    &mut Some(ref mut previous_pos) => {
                                        state.translation = [
                                            state.translation[0] + pos.0-previous_pos.0,
                                            state.translation[1] + pos.1-previous_pos.1
                                        ];
                                        *previous_pos = (pos.0,pos.1);
                                    },
                                    option => {
                                        *option = Some(pos);
                                    }
                                }
                            }
                        }
                        Inhibit(false)
                    }));

            let command_input = gtk::TextView::new();
                vert_layout.pack_end(&command_input,false,false,0);

                command_input.set_monospace(true);
                command_input.set_wrap_mode(gtk::WrapMode::None);
                command_input.connect_key_press_event(|widget,event_key|{
                    if event_key.get_hardware_keycode() == x11_keymap::ENTER{
                        widget.set_buffer(None);
                        widget.set_editable(false);
                    }
                    Inhibit(false)
                });

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
