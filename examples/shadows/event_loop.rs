use std::sync::Arc;

use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard;
use winit::keyboard::NamedKey::Escape;
use winit::window::Window;

use spark_gap::frame_counter::FrameCounter;
use spark_gap::gpu_context::GpuContext;

use crate::world::World;

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = GpuContext::new(window).await;
    let mut frame_counter = FrameCounter::new();

    let mut world = World::new(&mut context);

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent { window_id: _, event } = event {
                match event {
                    WindowEvent::Resized(new_size) => {
                        context.resize(new_size);
                        world.resize(&context);
                        context.window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        world.render(&context);

                        context.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.logical_key == keyboard::Key::Named(Escape) {
                            target.exit()
                        }
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}
