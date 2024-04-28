use std::sync::Arc;

use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode::{Digit1, Digit2, Escape, Space, KeyC};
use winit::keyboard::PhysicalKey;
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
                        if event.state == ElementState::Pressed {
                            match event.physical_key {
                                PhysicalKey::Code(Escape) => target.exit(),
                                PhysicalKey::Code(Space) => world.show_shadows = !world.show_shadows,
                                PhysicalKey::Code(Digit1) => world.layer_number = 0,
                                PhysicalKey::Code(Digit2) => world.layer_number = 1,
                                PhysicalKey::Code(KeyC) => {
                                    world.camera_position += 1;
                                    if world.camera_position > 2 { world.camera_position = 0; }
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}
