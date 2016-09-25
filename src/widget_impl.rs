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
            keys::Alt_L     | keys::Alt_R     |
            keys::Shift_L   | keys::Shift_R   |
            keys::Tab       => {return Inhibit(true);}

            //Clear input buffer or pop input stack
            keys::Escape => {
                if state.input_buffer.is_empty() {
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
                state.input_buffer.push(chr);
                println!("input {:?};{:?}", state.input,state.input_buffer);
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

    pub type ImageStateType = Rc<RefCell<Option<gl_ext::ImageState>>>;

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
            let vertices_draw = glium::VertexBuffer::empty_dynamic(&display,1).unwrap();
            let indices = glium::IndexBuffer::new(
                &display,
                glium::index::PrimitiveType::TriangleStrip,
                &[1,2,0,3u16]
            ).unwrap();
            //GL shaders
            let program = program!(&display,
                140 => {
                    vertex  : include_str!("shaders/image_area.140.vert.glsl"),
                    fragment: include_str!("shaders/image_area.140.frag.glsl"),
                },
                110 => {
                    vertex  : include_str!("shaders/image_area.110.vert.glsl"),
                    fragment: include_str!("shaders/image_area.110.frag.glsl"),
                },
                100 => {
                    vertex  : include_str!("shaders/image_area.100.vert.glsl"),
                    fragment: include_str!("shaders/image_area.100.frag.glsl"),
                },
            ).unwrap();
            let program_draw = program!(&display,
                140 => {
                    vertex  : include_str!("shaders/draw.140.vert.glsl"),
                    fragment: include_str!("shaders/draw.140.frag.glsl"),
                },
            ).unwrap();

            let state = state.borrow();
            let image_dimensions = state.images[0].dimensions();
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                state.images[0].clone().into_raw(),//TODO
                image_dimensions
            );
            let texture = glium::texture::Texture2d::new(&display,image).unwrap();

            let mut gl_state = gl_state.borrow_mut();
            *gl_state = Some(gl_ext::ImageState{
                display                 : display,
                vertices                : vertices,
                vertices_draw           : vertices_draw,
                indices                 : indices,
                program                 : program,
                drawing_program         : program_draw,
                texture                 : texture,
                translation_previous_pos: None,
                dimensions              : (1.0,1.0),
                draw_point_buffer       : Vec::with_capacity(20),
                mouse_image_previous_pos: [0;2],
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
                gl_state.dimensions = (w as f64,h as f64);
            }
        }));

        //Drawing of draw area for every frame
        widget.connect_render(move_fn_with_clones!(state,gl_state; |_,context|{
            let state = state.borrow();
            let mut gl_state = gl_state.borrow_mut();

            if let Some(gl_state) = gl_state.as_mut(){
                use glium::index::*;
                use glium::uniforms::*;

                context.make_current();

                let (tex_w,tex_h) = (
                    gl_state.texture.get_width() as f64,
                    gl_state.texture.get_height().unwrap() as f64
                );

                //Buffered draw
                if !gl_state.draw_point_buffer.is_empty(){
                    //Prepare vertex data
                    gl_state.vertices_draw.write(gl_state.draw_point_buffer.as_ref());
                    gl_state.draw_point_buffer.clear();

                    //Draw to texture
                    let mut target = gl_state.texture.as_surface();
                        target.draw(
                            &gl_state.vertices_draw,
                            NoIndices(PrimitiveType::Points),
                            &gl_state.drawing_program,
                            &uniform!{
                                transformation: [//Translation*Scale transformation matrix
                                    [ (1.0/tex_w/state.zoom) as f32, 0.0, 0.0],
                                    [ 0.0, (-1.0/tex_h/state.zoom) as f32, 0.0],
                                    [ (-(state.translation[0] + gl_state.dimensions.0)/tex_w/state.zoom) as f32, ((state.translation[1] + gl_state.dimensions.1)/tex_h/state.zoom) as f32, 1.0f32]
                                ]
                            },
                            &Default::default()
                        ).unwrap();
                }

                //Image area
                let mut target = gl_state.display.draw();
                    let (scale_x,scale_y) = (
                        (1.0/gl_state.dimensions.0*tex_w*state.zoom) as f32,
                        (1.0/gl_state.dimensions.1*tex_h*state.zoom) as f32,
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
                                [ (state.translation[0]/gl_state.dimensions.0*2.0) as f32, (-state.translation[1]/gl_state.dimensions.1*2.0) as f32, 1.0f32]
                            ],//TODO: The translation seem slightly incorrect (Almost not noticable)
                            tex: gl_state.texture
                                .sampled()
                                .minify_filter(MinifySamplerFilter::Nearest)
                                .magnify_filter(MagnifySamplerFilter::Nearest),
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
        widget.connect_scroll_event(move_fn_with_clones!(state,gl_state; |_,event|{
            let gl_state = gl_state.borrow();
            let mut state = state.borrow_mut();
            if let Some(gl_state) = gl_state.as_ref(){
                let (_,delta) = event.get_delta();

                if delta>0.0{
                    state.zoom/=2.0;//TODO: Zoom with mouse pos as center
                    //let pos = ::window_to_image_pos(event.get_position(),gl_state,&*state);
                    //state.translation[0]-=pos.0;
                    //state.translation[1]-=pos.1;
                }else if delta<0.0{
                    //let pos = ::window_to_image_pos(event.get_position(),gl_state,&*state);
                    //state.translation[0]+=pos.0;
                    //state.translation[1]+=pos.1;
                    state.zoom*=2.0;
                }
            }
            Inhibit(false)
        }));

        //When releasing mouse buttons
        widget.add_events(gdk_sys::GDK_ALL_EVENTS_MASK.bits() as i32);//TODO: Use specific event masks instead
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
            if event.get_state().contains(gdk::BUTTON1_MASK){
                //Translation
                if event.get_state().contains(gdk::SHIFT_MASK){
                    let mut state = state.borrow_mut();
                    let mut gl_state = gl_state.borrow_mut();
                    if let Some(gl_state) = gl_state.as_mut(){
                        let pos = event.get_position();

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
                }else{//Drawing
                    let state = state.borrow();
                    let mut gl_state = gl_state.borrow_mut();
                    if let Some(gl_state) = gl_state.as_mut(){
                        let pos = event.get_position();
                        let image_pos = ::window_to_image_pos(pos,gl_state,&*state,);
                        let image_posi = [image_pos.0 as i32 , image_pos.1 as i32];
                        let gl_pos = ::window_to_gl_pos(pos,gl_state,&state);
                        let gl_pos = [gl_pos.0 as f32 , gl_pos.1 as f32];

                        //Avoid pushing points for the same pixel on the image
                        if image_posi!=gl_state.mouse_image_previous_pos{
                            gl_state.mouse_image_previous_pos = image_posi;

                            if image_posi[0]>=0 && image_posi[0]<gl_state.texture.get_width() as i32
                            && image_posi[1]>=0 && image_posi[1]<gl_state.texture.get_height().unwrap() as i32{
                                gl_state.draw_point_buffer.push(gl_ext::DrawingVertex{
                                    position: gl_pos,
                                    color   : [1.0 , 0.5 , 0.5 , 1.0],
                                });
                            }
                        }
                    }
                }
            }
            Inhibit(false)
        }));
    }

    //TODO: Find everything that's the same as image_area and factor it out
    //TODO: Context sharing with image area
    pub fn preview_area(widget: &gtk::GLArea,image_area: &gtk::GLArea,gl_state: &ImageStateType){
        //Initialization of GL context
        widget.connect_create_context(move_fn_with_clones!(image_area; |_|{
            image_area.get_context().unwrap()
        }));

        //Drawing of draw area for every frame
        widget.connect_render(move_fn_with_clones!(gl_state; |_,context|{
            let gl_state = gl_state.borrow();
            if let Some(gl_state) = gl_state.as_ref(){
                context.make_current();

                let mut target = gl_state.display.draw();
                    let (w,h) = target.get_dimensions();
                    let (tex_w,tex_h) = (
                        gl_state.texture.get_width() as f64,
                        gl_state.texture.get_height().unwrap() as f64
                    );
                    let (scale_x,scale_y) = (
                        (1.0/w as f64*tex_w) as f32,
                        (1.0/h as f64*tex_h) as f32,
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
