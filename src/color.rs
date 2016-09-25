#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Color{
    RGB(u8,u8,u8),
    RGBA(u8,u8,u8,u8)
}

impl Color{
    pub fn to_gdk(self) -> ::gdk_sys::GdkRGBA{
        match self{
            Color::RGB(r,g,b) => ::gdk_sys::GdkRGBA{
                red  : r as f64/255.0,
                green: g as f64/255.0,
                blue : b as f64/255.0,
                alpha: 1.0,
            },
            Color::RGBA(r,g,b,a) => ::gdk_sys::GdkRGBA{
                red  : r as f64/255.0,
                green: g as f64/255.0,
                blue : b as f64/255.0,
                alpha: a as f64/255.0,
            }
        }
    }
}
