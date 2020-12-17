use bronze::*;

use crate::command::Command;


pub struct UndoManager {
    undo_stack: Vec<Box<dyn Command>>
}

impl UndoManager {
    pub fn new() -> Self {
        UndoManager {undo_stack: Vec::new()}
    }

    pub fn push_command(&mut self, cmd: Box<dyn Command>) {
        self.undo_stack.push(cmd);
    }

    pub fn undo(&mut self) {

    }
}