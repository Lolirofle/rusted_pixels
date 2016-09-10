#![allow(dead_code)]

extern crate core;
extern crate epoxy;
extern crate image;
extern crate gl;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate shared_library;
extern crate vecmath;
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
//use windows::*;

pub fn main() {
    if let Ok(_) = gtk::init(){
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
            epoxy::load_with(|s| {
                unsafe {
                    match DynamicLibrary::open(None).unwrap().symbol(s) {
                        Ok(v) => v,
                        Err(_) => ptr::null(),
                    }
                }
            });

            image_area.connect_realize(|widget|{
                widget.make_current();
            });

            let gl_state: Rc<RefCell<Option<gl_ext::State>>> = Rc::new(RefCell::new(None));

            //Initialization of draw area
            let _gl_state = gl_state.clone();
            image_area.connect_realize(move |widget|{
                let display = glium_ext::GtkFacade{
                    context: unsafe{
                        glium::backend::Context::new::<_,()>(
                            glium_ext::GtkBackend{gl_area: widget.clone()},
                            true,
                            Default::default()
                        )
                    }.unwrap(),
                };
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

                let image = image::load(
                    io::BufReader::new(fs::File::open("test.png").unwrap()),
                    image::PNG
                ).unwrap().to_rgba();
                let image_dimensions = image.dimensions();
                let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                    image.into_raw(),
                    image_dimensions
                );
                let texture = glium::texture::SrgbTexture2d::new(&display,image).unwrap();

                let mut gl_state = _gl_state.borrow_mut();
                *gl_state = Some(gl_ext::State{
                    display : display,
                    vertices: vertices,
                    indices : indices,
                    program : program,
                    texture : texture,
                    zoom    : 1.0,
                    translation: [0.0,0.0],
                    translation_previous_pos: None,
                    dimensions: (1.0,1.0),
                });
            });

            //Finalization of draw area
            let _gl_state = gl_state.clone();
            image_area.connect_unrealize(move |_|{
                let mut gl_state = _gl_state.borrow_mut();
                *gl_state = None;
            });

            //Resize of draw area
            let _gl_state = gl_state.clone();
            image_area.connect_resize(move |_,w,h|{
                let mut gl_state = _gl_state.borrow_mut();
                if let Some(gl_state) = gl_state.as_mut(){
                    gl_state.dimensions = (w as f32,h as f32);
                }
            });

            //Drawing of draw area for every frame
            let _gl_state = gl_state.clone();
            image_area.connect_render(move |_,_|{
                let gl_state = _gl_state.borrow();
                if let Some(gl_state) = gl_state.as_ref(){
                    let mut target = gl_state.display.draw();
                        let (tex_w,tex_h) = (gl_state.texture.get_width() as f32,gl_state.texture.get_height().unwrap() as f32);
                        let (scale_x,scale_y) = (
                            1.0/gl_state.dimensions.0*tex_w*gl_state.zoom,
                            1.0/gl_state.dimensions.1*tex_h*gl_state.zoom,
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
                                    [ gl_state.translation[0]/gl_state.dimensions.0*2.0, -gl_state.translation[1]/gl_state.dimensions.1*2.0, 1.0f32]
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
            });
        paned.add2(&image_area);

        let _gl_state = gl_state.clone();
        paned.connect_key_press_event(move |_,event|{
            let mut gl_state = _gl_state.borrow_mut();
            if let Some(gl_state) = gl_state.as_mut(){
                use gdk_sys::{
                    GDK_KEY_plus        as KEY_PLUS,
                    GDK_KEY_minus       as KEY_MINUS,
                    GDK_KEY_KP_Add      as KEY_KP_PLUS,
                    GDK_KEY_KP_Subtract as KEY_KP_MINUS,
                };
                match event.get_keyval() as i32{
                    KEY_PLUS  | KEY_KP_PLUS  => {gl_state.zoom*=2.0;},
                    KEY_MINUS | KEY_KP_MINUS => {gl_state.zoom/=2.0;},
                    _  => ()
                };
            }
            Inhibit(false)
        });

        let _gl_state = gl_state.clone();
        image_area.add_events(gdk_sys::GDK_SCROLL_MASK.bits() as i32);
        image_area.add_events(gdk_sys::GDK_SMOOTH_SCROLL_MASK.bits() as i32);
        image_area.connect_scroll_event(move |_,event|{
            let mut gl_state = _gl_state.borrow_mut();
            if let Some(gl_state) = gl_state.as_mut(){
                let (_,delta) = event.get_delta();
                if delta>0.0{
                    gl_state.zoom/=2.0;
                }else if delta<0.0{
                    gl_state.zoom*=2.0;
                }
            }
            Inhibit(false)
        });
        image_area.add_events(gdk_sys::GDK_ALL_EVENTS_MASK.bits() as i32);
        let _gl_state = gl_state.clone();
        image_area.connect_button_release_event(move |_,event|{
            let mut gl_state = _gl_state.borrow_mut();
            if let Some(gl_state) = gl_state.as_mut(){
                if event.get_state().contains(gdk::BUTTON1_MASK){
                    gl_state.translation_previous_pos = None;
                }
                
            }
            Inhibit(false)
        });

        let _gl_state = gl_state.clone();
        image_area.connect_motion_notify_event(move |_,event|{
            let mut gl_state = _gl_state.borrow_mut();
            if let Some(gl_state) = gl_state.as_mut(){
                if event.get_state().contains(gdk::BUTTON1_MASK){
                    let pos = event.get_position();
                    let pos = (pos.0 as f32,pos.1 as f32);

                    match &mut gl_state.translation_previous_pos{
                        &mut Some(ref mut previous_pos) => {
                            gl_state.translation = [
                                gl_state.translation[0] + pos.0-previous_pos.0,
                                gl_state.translation[1] + pos.1-previous_pos.1
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
        });

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

    /*
    let mut windows: Vec<Box<Window>> =
        vec![Box::new(DrawingWindow::new(50, 50, 8,
                                         Color::RGB(100, 100, 100), 0)),
             Box::new(PreviewWindow(
                 DrawingWindow::new(400, 50, 1,
                                    Color::RGB(50,50,50), 0))),
             Box::new(DrawingWindow::new(400, 400, 2,
                                         Color::RGB(50,50,50), 0)),
             Box::new(PaletteWindow{x: 400,y: 100,palette_id: 0}),
            ];

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'main_loop
                },
                Event::MouseButtonDown { mouse_btn: Mouse::Left,
                                         x, y, .. } => {
                    state.left_mouse_down = true;
                    for window in &windows {
                        window
                            .handle_mouse_down(&mut state, x, y);
                    }
                },
                Event::MouseMotion { x, y, .. } => {
                    state.mouse_x = x;
                    state.mouse_y = y;
                    if state.left_mouse_down {
                        for window in &windows {
                            window
                                .handle_mouse_down(&mut state, x, y);
                        }
                    }
                },
                Event::MouseButtonUp { mouse_btn: Mouse::Left, .. } => {
                    state.left_mouse_down = false;
                },
                /*Event::KeyDown { keycode: Some(Keycode::S), keymod: sdl2::keyboard::LCTRLMOD, .. } => {
                    state.images[0].save_png_image("test_out.png").unwrap();
                },*/
                Event::KeyDown { keycode: Some(keycode), keymod, .. } => {
                    use sdl2::keyboard::{LCTRLMOD, LALTMOD};

                    // every command begins with a single key
                    if state.input.is_empty() {
                        match keymod {
                            LCTRLMOD => {
                                state.input.push(
                                    Input::Char(ExtendedChar::CtrlModified(keycode)));
                            },
                            LALTMOD => {
                                state.input.push(
                                    Input::Char(ExtendedChar::AltModified(keycode)));
                            },
                            _ => {
                                state.input.push(
                                    Input::Char(ExtendedChar::NonModified(keycode)));
                            }
                        }
                        match execute_command(&mut state) {
                            CommandResult::Quit => { break 'main_loop },
                            _ => {}
                        }
                    } else {
                        // add the keycode char to some buffer,
                        // then add code to interpret that buffer when
                        // RET is pressed
                    }
                    
                },
                _ => {}
            }
        }
    }*/
}

fn window_to_image_pos(pos: (f32,f32),gl_state: &GlState) -> (f32,f32){
    let (tex_w,tex_h) = (
        gl_state.texture.get_width() as f32,
        gl_state.texture.get_height().unwrap() as f32
    );

    (
        pos.0 as f32-gl_state.translation[0]-((gl_state.dimensions.0-tex_w*gl_state.zoom)/2.0))/gl_state.zoom,
        pos.1 as f32-gl_state.translation[1]-((gl_state.dimensions.1-tex_h*gl_state.zoom)/2.0))/gl_state.zoom
    )
}
