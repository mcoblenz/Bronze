use bronze::*;
use crate::document::*;
use crate::shape::*;
use crate::square::*;
use crate::insert_shape_command::*;
use crate::undo_manager::*;
use crate::command::Command;

pub struct DocumentWindowController {
    window: winit::window::Window,
    document: GcHandle<Document>,
    undo_manager: UndoManager,
}

impl DocumentWindowController {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let size = winit::dpi::LogicalSize::new(800, 600);
        
        let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Metal Window Example".to_string())
        .build(&event_loop)
        .unwrap();

        let document = Document::new();
        let document_ref = Gc::new(document);
        let document_handle = GcHandle::new(document_ref);
        let undo_manager = UndoManager::new();

        DocumentWindowController{window, document: document_handle, undo_manager}
    }

    pub fn mouse_clicked(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        println!("mouse clicked at position: {}, {}", position.x, position.y);
        // Make a square centered at the point that was clicked.
        const edge_length: f64 = 10.0;
        let top_left = Point{x: position.x - (edge_length / 2.0), y: position.y - (edge_length / 2.0)};

        let square = Square::new(top_left, edge_length);
        let square_box = Box::new(square);

        // Note usage of handle rather than a ref, because the command is in the Rust heap
        let mut insert_command = Box::new(InsertShapeCommand::new(Gc::new(square_box), self.document.gc_ref()));
        (*insert_command.as_mut()).commit();

        self.undo_manager.push_command(insert_command);

    }


}