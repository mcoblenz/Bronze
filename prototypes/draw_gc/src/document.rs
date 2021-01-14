use crate::shape::*;
use bronze::*;
use std::vec::Vec;

pub struct Document {
    pub shapes: Vec<GcRef<dyn Shape>>,
}

impl Document {
    pub fn new() -> Self {
        Document {shapes: Vec::new()}
    }

    pub fn add_shape(&mut self, shape: GcRef<dyn Shape>) {
        self.shapes.push(shape);
    }

    pub fn remove_shape(&mut self, shape: GcRef<dyn Shape>) {
        let pos = self.shapes.iter().position(|x| *x == shape);
        match pos {
            Some(index) => {self.shapes.remove(index);},
            None => println!("error: shape not found."),
        }
    }
}

impl GcTrace for Document {}