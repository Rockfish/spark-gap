use std::sync::Arc;

use glam::vec3;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard;
use winit::keyboard::NamedKey::Escape;
use winit::window::Window;

use spark_gap::camera::camera_handler::CameraHandler;
use spark_gap::camera::fly_camera_controller::FlyCameraController;
use spark_gap::frame_counter::FrameCounter;
use spark_gap::gpu_context::GpuContext;
use spark_gap::texture::create_depth_texture;

use crate::model::Model;
use crate::render::World;

const VIEW_PORT_WIDTH: i32 = 1500;
const VIEW_PORT_HEIGHT: i32 = 1000;

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = GpuContext::new(window).await;
    let mut frame_counter = FrameCounter::new();

    let model = Model::new(&context);

    let aspect_ratio = VIEW_PORT_WIDTH as f32 / VIEW_PORT_HEIGHT as f32;
    let camera_position = vec3(0.0, 100.0, 300.0);
    let mut camera_controller = FlyCameraController::new(aspect_ratio, camera_position, 0.0, 0.0);
    let camera_handler = CameraHandler::new(&mut context, &camera_controller);

    let mut depth_texture = create_depth_texture(&context);

    let mut world = World::new(&mut context);

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent { window_id: _, event } = event {
                match event {
                    WindowEvent::Resized(new_size) => {
                        context.resize(new_size);
                        camera_handler.update_camera(&context, &mut camera_controller);
                        depth_texture = create_depth_texture(&context);
                        context.window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        world.render(&context);

                        context.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        // if event.state == ElementState::Pressed {
                        if event.logical_key == keyboard::Key::Named(Escape) {
                            target.exit()
                        }
                        // }
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}
