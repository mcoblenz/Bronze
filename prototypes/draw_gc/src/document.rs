use crate::shape::*;
use bronze::*;
use std::vec::Vec;

pub struct Document {
    shapes: Vec<GcRef<Box<dyn Shape>>>,
}

impl Document {
    pub fn new() -> Self {
        Document {shapes: Vec::new()}
    }

    pub fn add_shape(&mut self, shape: GcRef<Box<dyn Shape>>) {
        self.shapes.push(shape);
    }
}

impl GcTrace for Document {}