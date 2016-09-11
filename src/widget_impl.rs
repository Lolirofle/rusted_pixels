use core::cell::RefCell;
use gtk;
use gtk::prelude::*;
use std::rc::Rc;

use state::State;

pub type StateType = Rc<RefCell<State>>;

pub fn input_system<W>(widget: &W,state: &StateType,commands: ::input::Commands)
    where W: gtk::WidgetExt
{
    use input::*;

    widget.connect_key_press_event(move_fn_with_clones!(state; |_,event|{
        //println!("{:?} {:?} {:?}",event_key.get_keyval(),event_key.get_hardware_keycode(),event_key.get_state());
        let mut state = state.borrow_mut();

        let (keycode,keymod) = (event.get_keyval(),event.get_state());

        match keycode{
            keys::Control_L | keys::Control_R |
            keys::Shift_L   | keys::Shift_R   |
            keys::Tab       => {return Inhibit(true);}

            //Clear input buffer or pop input stack
            keys::Escape => {
                if !state.input_buffer.is_empty() {
                    state.input.pop();
                    println!("pop stack {:?};{:?}",state.input,state.input_buffer);
                } else {
                    state.input_buffer.clear();
                    println!("reset input {:?};{:?}",state.input,state.input_buffer);
                }
            },

            //Execute the input stack
            keys::Return => {
                println!("execute stack {:?};{:?}", state.input,state.input_buffer);

                //Parse the input buffer, and push it to the input stack
                let (input_type,arg) = parse_input(&state.input_buffer);
                state.input.push(input_type);
                if let Some(arg) = arg {
                    state.args.push(arg);
                }

                state.input_buffer.clear();

                match execute_command(&mut state, &commands) {
                    CommandResult::Quit => { gtk::main_quit(); },
                    _ => {}
                }
            },

           //Every command begins with a single key
            keycode if state.input.is_empty() => {
                state.input.push(Input::Char(keycode,keymod));
                println!("execute once {:?};{:?}", state.input,state.input_buffer);
                match execute_command(&mut state, &commands) {
                    CommandResult::Quit => { gtk::main_quit(); },
                    _ => {}
                }
            },

            //Input to the input buffer
            keycode => if let Some(chr) = keycode_to_char(keycode) {
                println!("input {:?};{:?}", state.input,state.input_buffer);
                state.input_buffer.push(chr);
            }else{
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }));
}

pub fn command_input(widget: &gtk::TextView){
    use x11_keymap::*;

    widget.set_monospace(true);
    widget.set_wrap_mode(gtk::WrapMode::None);
    widget.connect_key_press_event(|widget,event_key|{
        if event_key.get_hardware_keycode() == ENTER{
            widget.set_buffer(None);
            widget.set_editable(false);
        }
        Inhibit(false)
    });
}

pub mod image{
    use core::cell::RefCell;
    use gdk;
    use gdk_sys;
    use glium;
    use glium::Surface;
    use gtk;
    use gtk::prelude::*;
    use std::rc::Rc;

    use gl_ext;
    use glium_ext;

    pub type ImageStateType   = Rc<RefCell<Option<gl_ext::ImageState>>>;
    pub type PreviewStateType = Rc<RefCell<Option<gl_ext::PreviewState>>>;

    /**
     * Implements an GLArea to be an image area
     */
    pub fn image_area(widget: &gtk::GLArea,gl_state: &ImageStateType,state: &super::StateType){
        //Initialization of draw area
        widget.connect_realize(move_fn_with_clones!(state,gl_state; |widget|{
            widget.make_current();

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
            *gl_state = Some(gl_ext::ImageState{
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
        widget.connect_unrealize(move_fn_with_clones!(gl_state; |_|{
            let mut gl_state = gl_state.borrow_mut();
            *gl_state = None;
        }));

        //Resize of draw area
        widget.connect_resize(move_fn_with_clones!(gl_state; |_,w,h|{
            let mut gl_state = gl_state.borrow_mut();
            if let Some(gl_state) = gl_state.as_mut(){
                gl_state.dimensions = (w as f32,h as f32);
            }
        }));

        //Drawing of draw area for every frame
        widget.connect_render(move_fn_with_clones!(state,gl_state; |_,widget|{
            let state = state.borrow();
            let gl_state = gl_state.borrow();
            if let Some(gl_state) = gl_state.as_ref(){
                widget.make_current();

                let mut target = gl_state.display.draw();
                    let (tex_w,tex_h) = (
                        gl_state.texture.get_width() as f32,
                        gl_state.texture.get_height().unwrap() as f32
                    );
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
        widget.add_events(gdk_sys::GDK_SCROLL_MASK.bits() as i32);
        widget.add_events(gdk_sys::GDK_SMOOTH_SCROLL_MASK.bits() as i32);
        widget.connect_scroll_event(move_fn_with_clones!(state; |_,event|{
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
        widget.add_events(gdk_sys::GDK_ALL_EVENTS_MASK.bits() as i32);
        widget.connect_button_release_event(move_fn_with_clones!(gl_state; |_,event|{
            let mut gl_state = gl_state.borrow_mut();
            if let Some(gl_state) = gl_state.as_mut(){
                if event.get_state().contains(gdk::BUTTON1_MASK){
                    gl_state.translation_previous_pos = None;
                }
            }
            Inhibit(false)
        }));

        //When moving mouse cursor
        widget.connect_motion_notify_event(move_fn_with_clones!(state,gl_state; |_,event|{
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
    }

    pub fn preview_area(widget: &gtk::GLArea,gl_state: &PreviewStateType,state: &super::StateType){
        //Initialization of draw area
        widget.connect_realize(move_fn_with_clones!(state,gl_state; |widget|{
            widget.make_current();

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
            *gl_state = Some(gl_ext::PreviewState{
                display : display,
                vertices: vertices,
                indices : indices,
                program : program,
                texture : texture,
                dimensions: (1.0,1.0),
            });
        }));

        //Finalization of draw area
        widget.connect_unrealize(move_fn_with_clones!(gl_state; |_|{
            let mut gl_state = gl_state.borrow_mut();
            *gl_state = None;
        }));

        //Drawing of draw area for every frame
        widget.connect_render(move_fn_with_clones!(gl_state; |_,widget|{
            let gl_state = gl_state.borrow();
            if let Some(gl_state) = gl_state.as_ref(){
                widget.make_current();

                let mut target = gl_state.display.draw();
                    let (w,h) = target.get_dimensions();
                    let (tex_w,tex_h) = (gl_state.texture.get_width() as f32,gl_state.texture.get_height().unwrap() as f32);
                    let (scale_x,scale_y) = (
                        1.0/w as f32*tex_w,
                        1.0/h as f32*tex_h,
                    );
                    target.clear_color(0.1,0.1,0.1,1.0);
                    target.draw(
                        &gl_state.vertices,
                        &gl_state.indices,
                        &gl_state.program,
                        &uniform!{
                            transformation: [
                                [ scale_x, 0.0, 0.0],
                                [ 0.0, scale_y, 0.0],
                                [ 0.0, 0.0, 1.0f32]
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
        }));
    }
}
