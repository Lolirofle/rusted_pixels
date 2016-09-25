use image::RgbaImage;

use color::Color;
use input::{Input,Arg};

/*
 * Holds the main state, pretty self explanatory.
 */
pub struct State {
    pub current_color: Color,
    pub images: Vec<RgbaImage>,
    pub current_palette_index: usize,
    pub palettes: Vec<Vec<Color>>,//TODO: Multiple palettes
    pub input: Vec<Input>,
    pub args: Vec<Arg>,
    pub zoom: f64,
    pub translation: [f64; 2],
	pub input_buffer: String,
}

impl State {
    pub fn new() -> Self{
        State {
            current_color: Color::RGB(255,255,255),
            images: vec![],
            current_palette_index: 0,
            palettes: vec![vec![
                Color::RGB(0,0,0),
                Color::RGB(128,128,128),
                Color::RGB(255,255,255),
                Color::RGB(192,128,112),
            ]],
            input: Vec::new(),
            args: Vec::new(),
            zoom: 1.0,
            translation: [0.0,0.0],
			input_buffer: String::new(),
        }
    }

    #[inline(always)]
    pub fn current_palette<'a>(&'a self) -> &'a [Color] {
        &self.palettes[self.current_palette_index]
    }
}
