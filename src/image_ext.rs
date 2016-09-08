use image::{Rgba,RgbaImage,Pixel};
use image::png::PNGEncoder;
use std::{fs,io,path};

pub fn save_png_image<P: AsRef<path::Path>>(image: &RgbaImage,path: P) -> io::Result<()>{
    PNGEncoder::new(try!(fs::File::create(path))).encode(
    	&*image,
    	image.width(),
    	image.height(),
    	<Rgba<u8> as Pixel>::color_type(),
    )
}
