extern crate sdl2;
extern crate png;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::BlendMode;
use sdl2::mouse::Mouse;
use std::path;

pub mod image_buffer;
pub mod windows;
pub mod state;
pub mod input;

use input::*;

use image_buffer::ImageBuffer;
use state::State;
use windows::*;

pub fn main() {
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
