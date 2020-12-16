use bronze::*;

#[derive(Clone, Copy)]
pub struct Point {
    x: u32,
    y: u32,
}

#[derive(Clone, Copy)]
pub struct Rect {
    top_left: Point,
    bottom_right: Point,
}

pub trait Shape {
    fn bounds(&self) -> Rect;
}

pub struct Square {
    bounds: Rect,
}

impl Shape for Square {
    fn bounds(&self) -> Rect {
        return self.bounds;
    }
}

impl GcTrace for dyn Shape {}