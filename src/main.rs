extern crate core;
extern crate epoxy;
extern crate image;
extern crate gl;
extern crate gtk;
extern crate shared_library;
#[macro_use]extern crate glium;

mod color;
mod gl_ext;
mod glium_ext;
mod image_ext;
mod input;
mod state;
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
                    gl_ext::Vertex{position: [0.0,0.0],tex_coords: [0.0,0.0]},
                    gl_ext::Vertex{position: [0.0,1.0],tex_coords: [0.0,1.0]},
                    gl_ext::Vertex{position: [1.0,1.0],tex_coords: [1.0,1.0]},
                    gl_ext::Vertex{position: [1.0,0.0],tex_coords: [1.0,0.0]},
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
                    zoom : 1.0,
                });
            });

            //Finalization of draw area
            let _gl_state = gl_state.clone();
            image_area.connect_unrealize(move |_|{
                let mut gl_state = _gl_state.borrow_mut();
                *gl_state = None;
            });

            let _gl_state = gl_state.clone();
            image_area.connect_scroll_event(move |_,event|{
                println!("Scroll: {:?}",event.get_delta());
                let mut gl_state = _gl_state.borrow_mut();
                if let Some(gl_state) = gl_state.as_mut(){
                    let (_,delta) = event.get_delta();
                    if delta>0.0{
                        gl_state.zoom*=2.0;
                    }else if delta<0.0{
                        gl_state.zoom/=2.0;
                    }
                }
                Inhibit(false)
            });


            let _gl_state = gl_state.clone();
            image_area.connect_key_press_event(move |_,event|{
                //Plus: 43 20 
                //Minus: 45 61
                println!("Zoom");
                let mut gl_state = _gl_state.borrow_mut();
                if let Some(gl_state) = gl_state.as_mut(){
                    match event.get_keyval(){
                        43 => {gl_state.zoom*=2.0;},
                        45 => {gl_state.zoom/=2.0;},
                        _  => ()
                    };
                }
                Inhibit(false)
            });

            //Drawing of draw area for every frame
            let _gl_state = gl_state.clone();
            image_area.connect_render(move |_,_|{
                let gl_state = _gl_state.borrow();
                if let Some(gl_state) = gl_state.as_ref(){
                    let mut target = gl_state.display.draw();
                        let (w,h) = target.get_dimensions();
                        let (tex_w,tex_h) = (gl_state.texture.get_width() as f32,gl_state.texture.get_height().unwrap() as f32);
                        target.clear_color(0.3,0.3,0.3,1.0);
                        target.draw(
                            &gl_state.vertices,
                            &gl_state.indices,
                            &gl_state.program,
                            &uniform!{
                                transformation: [
                                    [ 2.0/w as f32*tex_w as f32*gl_state.zoom, 0.0, 0.0, 0.0],
                                    [ 0.0, 2.0/h as f32*tex_h as f32*gl_state.zoom, 0.0, 0.0],
                                    [ 0.0, 0.0, 1.0, 0.0],
                                    [ 0.0, 0.0, 0.0, 1.0f32]
                                ],
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

        let command_input = gtk::TextView::new();
        vert_layout.pack_end(&command_input,false,false,0);

        command_input.set_monospace(true);
        command_input.set_wrap_mode(gtk::WrapMode::None);
        command_input.connect_key_press_event(|_,event_key|{
            println!("Input: {:?} {:?} {:?}",event_key.get_keyval(),event_key.get_hardware_keycode(),event_key.get_state());
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
