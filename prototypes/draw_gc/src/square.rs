use crate::shape::*;
use crate::graphics_context::*;
use bronze::*;
use bronze_derive::*;

pub struct Square {
    top_left: Point,
    edge_length: f64,
}

#[derive(Trace)]
impl Square {
    pub fn new(top_left: Point, edge_length: f64) -> Self {
        Square{top_left, edge_length}
    }
}

impl Shape for Square {
    fn bounds(&self) -> Rect {
        let bottom_right = Point{x: self.top_left.x + self.edge_length, y: self.top_left.y + self.edge_length};
        Rect {top_left: self.top_left, bottom_right: bottom_right}
    }

    fn draw (&self, graphics_context: &mut GraphicsContext) {
        graphics_context.draw_rect(self.top_left, self.edge_length, self.edge_length, [0, 0x0, 0xff, 0xff]);
    }
}
