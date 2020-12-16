use winit::window::*;
use winit::event_loop::ControlFlow;
use winit::event::Event;

use bronze::*;
use crate::document::*;

pub struct DocumentWindowController {
    window: winit::window::Window,
    document: GcHandle<Document>,
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

        DocumentWindowController{window, document: document_handle}
    }

    pub fn mouse_clicked(&self, position: winit::dpi::PhysicalPosition<f64>) {
        println!("mouse clicked at position: {}, {}", position.x, position.y);

    }
}