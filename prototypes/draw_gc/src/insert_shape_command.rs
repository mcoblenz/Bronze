use bronze_gc::*;

use crate::shape::*;
use crate::command::Command;
use crate::document::*;

pub struct InsertShapeCommand { 
    shape: GcRef<dyn Shape>,
    document: GcRef<Document>,
}

impl InsertShapeCommand {
    pub fn new(shape: GcRef<dyn Shape>, document: GcRef<Document>) -> Self {
        InsertShapeCommand {shape, document}
    }
}

impl Command for InsertShapeCommand {
    fn commit(&mut self) {
        self.document.as_mut().add_shape(self.shape);
    }

    fn undo(&mut self) {
        self.document.as_mut().remove_shape(self.shape);
    }

    fn redo(&mut self) {
        self.commit();
    }
}