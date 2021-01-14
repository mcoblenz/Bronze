use crate::command::Command;


pub struct UndoManager {
    // We own these commands, so there's no need for GC.
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl UndoManager {
    pub fn new() -> Self {
        UndoManager {undo_stack: Vec::new(), redo_stack: Vec::new()}
    }

    pub fn push_command(&mut self, cmd: Box<dyn Command>) {
        self.undo_stack.push(cmd);
    }

    pub fn undo(&mut self) {
        let command = self.undo_stack.pop();

        command.map(|mut c| {
            c.as_mut().undo(); 
            self.redo_stack.push(c);
        });
    }

    pub fn redo(&mut self) {
        let command = self.redo_stack.pop();

        command.map(|mut c| {
            c.as_mut().redo(); 
            self.undo_stack.push(c);
        });
    }
}