use bronze::GcTrace;

pub trait Command {
    fn commit(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
}

impl GcTrace for dyn Command {}
