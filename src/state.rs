use image::RgbaImage;

use color::Color;
use input::{Input,Arg};

/*
 * Holds the main state, pretty self explanatory.
 */
pub struct State {
    pub current_color: Color,
    pub left_mouse_down: bool,
    pub right_mouse_down: bool,
    pub images: Vec<RgbaImage>,
    pub palettes: Vec<Color>,//TODO: Multiple palettes
    pub input: Vec<Input>,
    pub args: Vec<Arg>,
}

impl State {
    pub fn new() -> Self{
        State {
            current_color: Color::RGB(255,255,255),
            left_mouse_down: false,
            right_mouse_down: false,
            images: vec![],
            palettes: vec![
                Color::RGB(0,0,0),
                Color::RGB(128,128,128),
                Color::RGB(255,255,255),
                Color::RGB(192,128,112),
            ],
            input: Vec::new(),
            args: Vec::new(),
        }
    }
}
