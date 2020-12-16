use crate::shape::*;
use bronze::*;
use std::vec::Vec;

pub struct Document {
    shapes: Vec<GcRef<dyn Shape>>,
}

impl Document {
    pub fn new() -> Self {
        Document {shapes: Vec::new()}
    }
}

impl GcTrace for Document {}