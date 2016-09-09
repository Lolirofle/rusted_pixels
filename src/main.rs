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
            println!("{:?} {:?}",event_key.get_keyval(),event_key.get_hardware_keycode());
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
                    indices : indices,
                    program : program,
                    texture : texture,
                });
            });

            //Finalization of draw area
            let _gl_state = gl_state.clone();
            image_area.connect_unrealize(move |_|{
                let mut gl_state = _gl_state.borrow_mut();
                *gl_state = None;
            });

            //Drawing of draw area for every frame
            let _gl_state = gl_state.clone();
            image_area.connect_render(move |_,_|{
                let gl_state = _gl_state.borrow();
                let gl_state = gl_state.as_ref().unwrap();

                let mut target = gl_state.display.draw();
                    let (w,h) = target.get_dimensions();
                    let (tex_w,tex_h) = (gl_state.texture.get_width() as f32,gl_state.texture.get_height().unwrap() as f32);
                    let vertices = glium::VertexBuffer::new(&gl_state.display,&[
                        gl_ext::Vertex{position: [0.0  ,0.0  ],tex_coords: [0.0,0.0]},
                        gl_ext::Vertex{position: [0.0  ,tex_h],tex_coords: [0.0,1.0]},
                        gl_ext::Vertex{position: [tex_w,tex_h],tex_coords: [1.0,1.0]},
                        gl_ext::Vertex{position: [tex_w,0.0  ],tex_coords: [1.0,0.0]},
                    ]).unwrap();
                    target.clear_color(0.3,0.3,0.3,1.0);
                    target.draw(
                        &vertices,
                        &gl_state.indices,
                        &gl_state.program,
                        &uniform!{
                            transformation: [
                                [ 1.0/w as f32, 0.0, 0.0, 0.0],
                                [ 0.0, 1.0/h as f32, 0.0, 0.0],
                                [ 0.0, 0.0, 1.0, 0.0],
                                [ 0.0, 0.0, 0.0, 1.0f32]
                            ],
                            tex: gl_state.texture
                                .sampled()
                                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                        },
                        &Default::default()
                    ).unwrap();
                target.finish().unwrap();

                Inhibit(false)
            });
        paned.add2(&image_area);

        let command_input = gtk::TextView::new();
        vert_layout.pack_end(&command_input,false,false,0);

        window.show_all();
        gtk::main();
    }else{
        println!("Failed to initialize GTK.");
    }


    /*let mut state = State{images: vec![
        ImageBuffer::load_png_image(&path::PathBuf::from("test.png")).unwrap(),
        ImageBuffer::new(32,64)
    ], ..State::new()};

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
