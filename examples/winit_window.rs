#![allow(clippy::single_match)]

use env_logger;
use winit::event::ElementState;
use winit::keyboard::NamedKey::Escape;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    keyboard,
    window::WindowBuilder,
};

// #[path = "wgpu_beginner/fill.rs"]
// mod fill;

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("Simple first window!")
        .with_inner_size(winit::dpi::LogicalSize::new(400.0, 400.0))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, elwt| {
        // println!("{event:?}");

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    // Notify the windowing system that we'll be presenting to the window.
                    window.pre_present_notify();
                    // fill::fill_window(&window);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        if event.logical_key == keyboard::Key::Named(Escape) {
                            elwt.exit()
                        }
                    }
                }
                _ => (),
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        }
    })
}
