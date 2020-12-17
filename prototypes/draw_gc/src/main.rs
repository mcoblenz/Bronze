use objc::{rc::autoreleasepool};
use winit::{
    event::{Event, WindowEvent, ElementState, MouseButton},
    event_loop::ControlFlow,
};

mod document_window_controller;
mod document;
mod document_view;
mod shape;
mod square;
mod undo_manager;
mod command;
mod insert_shape_command;

use crate::document_window_controller::DocumentWindowController;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();

    let mut doc = DocumentWindowController::new(&event_loop);


    let mut mouse_position = winit::dpi::PhysicalPosition::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { window_id, event } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        //layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                    },
                    WindowEvent::CursorMoved {device_id, position, ..} => {
                        mouse_position = position;
                    }
                    WindowEvent::MouseInput {device_id, state, button, ..} => {
                        if state == ElementState::Pressed && button == MouseButton::Left {
                            doc.mouse_clicked(mouse_position);
                        }
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    // window.request_redraw();
                },
                
                _ => {}
            }
        });
    });
}
