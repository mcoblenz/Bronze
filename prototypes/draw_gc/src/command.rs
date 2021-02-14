use bronze::GcTrace;
use bronze_derive::*;

#[derive(Trace)]
pub trait Command {
    fn commit(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
}

