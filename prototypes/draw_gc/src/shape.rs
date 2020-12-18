// TODO: split out the view behavior into a separate file, rather than merging model and view.

use bronze::*;
use crate::graphics_context::*;



#[derive(Clone, Copy)]
pub struct Rect {
    pub top_left: Point,
    pub bottom_right: Point,
}

pub trait Shape {
    fn bounds(&self) -> Rect;
    fn draw (&self, graphics_context: &mut GraphicsContext);
}

impl GcTrace for dyn Shape {}