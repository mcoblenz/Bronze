use bronze::*;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub top_left: Point,
    pub bottom_right: Point,
}

pub trait Shape {
    fn bounds(&self) -> Rect;
}

impl GcTrace for dyn Shape {}