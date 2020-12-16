use bronze::*;

trait Command {
    fn undo(&self);
    fn redo(&self);
}

impl GcTrace for Command {}

pub struct UndoManager {
    undo_stack: Vec<GcHandle<Box<dyn Command>>>
}

impl UndoManager {
    fn undo(&mut self) {

    }
}