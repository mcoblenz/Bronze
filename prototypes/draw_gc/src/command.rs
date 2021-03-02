use bronze::*;

pub trait Command {
    fn commit(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
}

simple_empty_finalize_trace![dyn Command];