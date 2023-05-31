use objc::{rc::autoreleasepool};
use winit::{
    event::*,
    event_loop::*,
};
use pixels::{Pixels, SurfaceTexture};

mod document_window_controller;
mod document;
mod shape;
mod square;
mod undo_manager;
mod command;
mod insert_shape_command;
mod graphics_context;

use crate::document_window_controller::DocumentWindowController;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();

    let pixel_width: u32 = 320;
    let pixel_height: u32 = 240;

    let mut document_controller = DocumentWindowController::new(&event_loop, pixel_width, pixel_height);

     // Pixels is not ideal; I'd rather be using CG APIs.
    // But CG-Rust bindings look like a big hassle.
    // TODO: move this into the DWC once Pixels no longer requires a type parameter that I can't instantiate.
    let window_size = document_controller.window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &document_controller.window);


    let mut pixels = match 
        Pixels::new(pixel_width, pixel_height, surface_texture) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        };


    let mut mouse_position = winit::dpi::PhysicalPosition::new(0.0, 0.0);
    let mut modifiers_state = Default::default();

    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { window_id: _, event } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(_size) => {
                        //layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                    },
                    WindowEvent::CursorMoved {device_id: _, position, ..} => {
                        mouse_position = position;
                    },
                    WindowEvent::MouseInput {device_id: _, state, button, ..} => {
                        if state == ElementState::Pressed && button == MouseButton::Left {
                            let logical_mouse_pos = mouse_position.to_logical(document_controller.window.scale_factor());
                            document_controller.mouse_clicked(logical_mouse_pos);
                        }
                    },
                    WindowEvent::ModifiersChanged(state) => {
                        modifiers_state = state;
                    }
                    WindowEvent::KeyboardInput {device_id: _, input, is_synthetic: _} => {
                        // Command key combinations
                        if input.state == ElementState::Pressed && modifiers_state & ModifiersState::LOGO != Default::default(){
                            match input.virtual_keycode {
                                Some(VirtualKeyCode::Z) => {
                                    if modifiers_state & ModifiersState::SHIFT == Default::default() {
                                        document_controller.undo();
                                    }
                                    else {
                                        document_controller.redo();
                                    }
                                },
                                Some (_k) => {}
                                None => {}
                            }
                        }
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    document_controller.window.request_redraw();
                },
                Event::RedrawRequested(_window_id) => {
                    // For now, we only have one window.
                    document_controller.redraw(&mut pixels);
                }
                _ => {}
            }
        });
    });
}
