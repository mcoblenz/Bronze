use bronze::GcTrace;

pub trait Command {
    fn commit(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
}

unsafe impl GcTrace for dyn Command {
    unsafe fn trace(&self) {
    }
}