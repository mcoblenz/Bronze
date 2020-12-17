use crate::shape::*;

pub struct Square {
    top_left: Point,
    edge_length: f64,
}

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
}