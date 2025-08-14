use objc::rc::autoreleasepool;
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::*,
    keyboard::{KeyCode, ModifiersState},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowId},
};

mod command;
mod document;
mod document_window_controller;
mod graphics_context;
mod insert_shape_command;
mod shape;
mod square;
mod undo_manager;

use crate::document_window_controller::DocumentWindowController;

struct App {
    window: Option<Arc<Window>>,
    modifiers_state: ModifiersState,
    mouse_position: winit::dpi::PhysicalPosition<f64>,
    document_controller: DocumentWindowController,
    pixels: Option<Pixels<'static>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        // Pixels is not ideal; I'd rather be using CG APIs.
        // But CG-Rust bindings look like a big hassle.
        // TODO: move this into the DWC once Pixels no longer requires a type parameter that I can't instantiate.
        let window_size = self.window.as_ref().unwrap().inner_size();
        let w = self.window.as_ref().unwrap();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, w.clone());

        let mut pixels = match Pixels::new(window_size.width, window_size.height, surface_texture) {
            Ok(p) => Some(p),
            Err(e) => panic!("{}", e),
        };
        self.pixels = pixels;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_size) => {
                //layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
                ..
            } => {
                self.mouse_position = position;
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
                ..
            } => {
                if state == ElementState::Pressed && button == MouseButton::Left {
                    let logical_mouse_pos = self.mouse_position.to_logical(
                        self.window
                            .as_ref()
                            .expect("window should exist")
                            .scale_factor(),
                    );
                    self.document_controller.mouse_clicked(logical_mouse_pos);
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers_state = state.state();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                // Command key combinations
                if event.state == ElementState::Pressed
                    && self.modifiers_state & ModifiersState::SUPER != Default::default()
                {
                    match event.text {
                        Some(t) => {
                            if t.as_str() == "Z"
                                && self.modifiers_state & ModifiersState::SHIFT
                                    == Default::default()
                            {
                                self.document_controller.undo();
                            } else if t.as_str() == "Z"
                                && self.modifiers_state & ModifiersState::SHIFT
                                    != Default::default()
                            {
                                self.document_controller.redo();
                            }
                        }
                        _k => {}
                    }
                }
            }
            // WindowEvent::MainEventsCleared => {
            //     self.window.unwrap().request_redraw();
            // }
            WindowEvent::RedrawRequested => {
                // For now, we only have one window.
                print!("Redrawing window...\n");
                self.document_controller
                    .redraw(&mut self.pixels.as_mut().unwrap());
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let pixel_width: u32 = 800;
    let pixel_height: u32 = 600;

    let document_controller = DocumentWindowController::new(pixel_width, pixel_height);

    let mouse_position = winit::dpi::PhysicalPosition::new(0.0, 0.0);

    let mut app = App {
        window: None,
        modifiers_state: Default::default(),
        mouse_position,
        document_controller: document_controller,
        pixels: None,
    };
    event_loop.run_app(&mut app);
}
