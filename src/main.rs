extern crate core;
extern crate epoxy;
extern crate gl;
extern crate glium;
extern crate gtk;
extern crate libc;
extern crate png;
extern crate sdl2;
extern crate shared_library;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::BlendMode;
use sdl2::mouse::Mouse;
use std::path;

mod glium_ext;
mod image_buffer;
mod input;
mod state;
mod windows;

use core::cell::RefCell;
use core::ptr;
use glium::Surface;
use gtk::prelude::*;
use shared_library::dynamic_library::DynamicLibrary;
use std::rc::Rc;

use image_buffer::ImageBuffer;
use input::*;
use state::State;
use windows::*;

pub fn main() {
    if let Ok(_) = gtk::init().is_err(){
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


        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        window.add(&paned);

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

            let display: Rc<RefCell<Option<glium_ext::GtkFacade>>> = Rc::new(RefCell::new(None));
            let display2 = display.clone();
            image_area.connect_realize(move |widget|{
                let mut display = display2.borrow_mut();
                *display = Some(
                    glium_ext::GtkFacade{
                        context: unsafe{
                            glium::backend::Context::new::<_,()>(
                                glium_ext::GtkBackend{gl_area: widget.clone()},
                                true,
                                Default::default()
                            )
                        }.unwrap(),
                    }
                );
            });

            let display2 = display.clone();
            image_area.connect_render(move |_, _|{
                let display = display2.borrow();
                let display = display.as_ref().unwrap();

                let mut target = display.draw();
                target.clear_color(0.7, 0.3, 0.3, 1.0);
                //target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,&Default::default()).unwrap();
                target.finish().unwrap();

                Inhibit(false)
            });
        paned.add2(&image_area);

        window.show_all();
        gtk::main();
    }else{
        println!("Failed to initialize GTK.");
    }



    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rusted pixels", 800, 600)
        .resizable()
        .build()
        .unwrap();

    let mut renderer = window.renderer().present_vsync().build().unwrap();

    // this is the most intuitive blend mode.
    renderer.set_blend_mode(BlendMode::Blend);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut state = State{images: vec![
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

        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGB(255,255,255));

        for window in &windows {
            window.draw(&mut renderer, &state);
        }

        renderer.present();
    }
}
