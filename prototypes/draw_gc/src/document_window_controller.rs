use bronze_gc::*;
use crate::document::*;
use crate::square::*;
use crate::insert_shape_command::*;
use crate::undo_manager::*;
use crate::command::Command;
use crate::graphics_context::*;

use raw_window_handle::HasRawWindowHandle;
// use winit::platform::macos::WindowExtMacOS;
// use cocoa::{appkit::NSView, base::id as cocoa_id};
// use objc::{runtime::YES};
use pixels::{Pixels};

// use std::mem;

pub struct DocumentWindowController {
    pub window: winit::window::Window, // TODO: make this private
    window_size: winit::dpi::LogicalSize<f64>,
    pixel_width: u32,
    pixel_height: u32,
    document: GcRef<Document>,
    undo_manager: UndoManager,
}

impl DocumentWindowController {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>, pixel_width: u32, pixel_height: u32) -> Self {
        let window_size = winit::dpi::LogicalSize::new(800.0, 600.0);
        
        let window = winit::window::WindowBuilder::new()
        .with_inner_size(window_size)
        .with_title("Metal Window Example".to_string())
        .build(&event_loop)
        .unwrap();

        // let device = Device::system_default().expect("no device found");

        // let layer = metal::MetalLayer::new();
        // layer.set_device(&device);
        // layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        // layer.set_presents_with_transaction(false);
    

        // unsafe {
        //     let view = window.ns_view() as cocoa_id;
        //     view.setWantsLayer(YES);
        //     // view.setLayer(mem::transmute(layer.as_ref()));
        // }

        // let view = window.ns_view() as cocoa_id;


        let document = Document::new();
        let document_ref = Gc::new(document);
        let undo_manager = UndoManager::new();

        DocumentWindowController{window, window_size, pixel_width, pixel_height, document: document_ref, undo_manager}
    }

    pub fn mouse_clicked(&mut self, position: winit::dpi::LogicalPosition<f64>) {
        // println!("mouse clicked at position: {}, {}", position.x, position.y);
        // Make a square centered at the point that was clicked.
        const EDGE_LENGTH: f64 = 10.0;
        let top_left = Point{x: position.x - (EDGE_LENGTH / 2.0), y: position.y - (EDGE_LENGTH / 2.0)};

        let square = Square::new(top_left, EDGE_LENGTH);
        
        let mut insert_command = Box::new(InsertShapeCommand::new(Gc::new(square), self.document));
        (*insert_command.as_mut()).commit();

        self.undo_manager.push_command(insert_command);
    }

    pub fn redraw<W: HasRawWindowHandle>(&self, pixels: &mut Pixels<W>) {
        // let view = self.window.ns_view();
        // let layer = view.layer();
        // println!("drawing in layer {:?}", layer);

        let frame = pixels.get_frame();
        let mut count = 0;
        for (_i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let color = [0xff, 0xff, 0xff, 0xff];
            pixel.copy_from_slice(&color);
            count = count + 4;
        }

        let mut graphics_context = GraphicsContext::new(frame, self.pixel_width, self.pixel_height, self.window_size);

        // Use the painter's algorithm on the shapes.
        for shape in &self.document.borrow().shapes {
            shape.draw(&mut graphics_context);
        }

        let render_err = pixels.render();
        assert!(render_err.is_ok())
    }

    pub fn undo(&mut self) {
        self.undo_manager.undo();
    }
    
    pub fn redo(&mut self) {
        self.undo_manager.redo();
    }
}