#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Color{
    RGB(u8,u8,u8),
    RGBA(u8,u8,u8,u8)
}
