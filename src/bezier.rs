
#[repr(C)]
#[derive(Default, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}


#[repr(C)]
#[derive(Default, Clone)]
pub struct Bezier2 {
    pub e0: Vec2,
    pub e1: Vec2,
    pub c: Vec2,
}