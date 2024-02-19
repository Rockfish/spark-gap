use spark_gap::frame_counter::FrameCounter;
use std::sync::Arc;
use std::time::Instant;
use glam::{Mat4, Vec3, vec3};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard;
use winit::keyboard::NamedKey::Escape;
use winit::window::Window;
use spark_gap::camera::CameraController;
use spark_gap::camera_handler::CameraHandler;
use spark_gap::context::Context;
use spark_gap::model_builder::ModelBuilder;
use crate::anim_render::{AnimRenderPass, create_depth_texture_view};
use crate::world::World;

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = Context::new(window).await;
    let mut frame_counter = FrameCounter::new();
    let size = context.window.inner_size();
    let aspect_ratio = size.width as f32 / size.height as f32;

    let camera_position = vec3(0.0, 100.0, 300.0);
    let camera_controller = CameraController::new(aspect_ratio, camera_position, 0.0, 0.0);
    let camera_handler = CameraHandler::new(&mut context, &camera_controller);

    let model_path = "examples/animation/vampire/dancing_vampire.dae";
    let model = ModelBuilder::new("model", model_path).build(&mut context).unwrap();

    let model_position = Vec3::ZERO;

    let depth_texture_view = create_depth_texture_view(&context);

    let anim_render = AnimRenderPass::new(&mut context);

    #[allow(unused_mut)]
    let mut model_transform = Mat4::IDENTITY;
    // model *= Mat4::from_rotation_x(-90.0f32.to_radians());
    // model_transform *= Mat4::from_translation(vec3(0.0, -10.4, -200.0));
    // model *= Mat4::from_scale(vec3(0.3, 0.3, 0.3));
    // let mut model = Mat4::from_translation(vec3(0.0, 5.0, 0.0));
    // model_transform *= Mat4::from_scale(vec3(15.0, 15.0, 15.0));
    // model_transform *= Mat4::from_scale(vec3(1.0, 1.0, 1.0));

    let mut world = World {
        camera_controller,
        camera_handler,
        model,
        model_position,
        model_transform,
        depth_texture_view,
        run: true,
        delta_time: 0.0,
        frame_time: 0.0,
        first_mouse: false,
        mouse_x: 0.0,
        mouse_y: 0.0,
    };

    let start_instant = Instant::now();

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(new_size) => {
                        context.resize(new_size);
                        world.camera_controller.resize(&context);
                        world.camera_handler.update_camera(&context, &world.camera_controller);
                        world.depth_texture_view = create_depth_texture_view(&context);
                        context.window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        let current_time = Instant::now().duration_since(start_instant).as_secs_f32();

                        if world.run {
                            world.delta_time = current_time - world.frame_time;
                        } else {
                            world.delta_time = 0.0;
                        }
                        world.frame_time = current_time;

                        world.model.update_animation(world.delta_time);

                        anim_render.render(&context, &world);

                        context.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        // if event.state == ElementState::Pressed {
                        if event.logical_key == keyboard::Key::Named(Escape) {
                            target.exit()
                        } else {

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

// pub fn render(context: &Context, world: &World) {
//
//     let frame = context
//         .surface
//         .get_current_texture()
//         .expect("Failed to acquire next swap chain texture");
//
//     let view = frame
//         .texture
//         .create_view(&wgpu::TextureViewDescriptor::default());
//
//     let mut encoder = context
//         .device
//         .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//
//     {
//         let mut render_pass = encoder.begin_render_pass(
//             &wgpu::RenderPassDescriptor {
//                 label: Some("render pass"),
//                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                     view: &view,
//                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
//                         store: wgpu::StoreOp::Store,
//                     },
//                 })],
//                 depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
//                     view: &world.depth_texture_view,
//                     depth_ops: Some(wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(1.0),
//                         store: wgpu::StoreOp::Store,
//                     }),
//                     stencil_ops: None,
//                 }),
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
//             });
//
//         render_pass.set_pipeline(&self.render_pipeline);
//
//         // mesh for vertex shader
//         render_pass.set_vertex_buffer(0, model.mesh.vertex_buffer.slice(..));
//
//         // transform for vertex shader
//         render_pass.set_bind_group(0, &camera_handler.bind_group, &[]);
//
//         // material for fragment shader
//         render_pass.set_bind_group(1, &model.material.bind_group, &[]);
//
//         render_pass.draw(0..36, 0..1);
//     }
//
//     context.queue.submit(Some(encoder.finish()));
//     frame.present();
// }

