use spark_gap::frame_counter::FrameCounter;
use std::sync::Arc;
use glam::Mat4;
use wgpu::TextureView;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard;
use winit::keyboard::NamedKey::Escape;
use winit::window::Window;
use spark_gap::camera::CameraUniform;
use spark_gap::camera_handler::CameraHandler;
use spark_gap::context::Context;
use spark_gap::model::Model;
use crate::model_handler::ModelHandler;
use crate::render::{create_depth_texture_view};

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

struct State {
    camera: CameraUniform,
    camera_handler: CameraHandler,
    model_handler: ModelHandler,
    delta_time: f32,
    frame_time: f32,
    first_mouse: bool,
    mouse_x: f32,
    mouse_y: f32,
}

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = Context::new(window).await;
    let mut frame_counter = FrameCounter::new();

    let model = Model::new(&context);
    let mut camera = CameraUniform::new(&context);
    let camera_handler = CameraHandler::new(&context, &camera);

    let model_handler = ModelHandler::new(&context, &model, &camera_handler);

    // could this be part of the Context?
    let mut depth_texture_view = create_depth_texture_view(&context);


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
                        camera_handler.update_camera(&context, &mut camera);
                        depth_texture_view = create_depth_texture_view(&context);
                        context.window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        draw(
                            &context,
                            &model_render,
                            &depth_texture_view,
                        );

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

pub fn render(
            context: &Context,
            camera_handler: &CameraHandler,
            model: &Model,
            depth_texture_view: &TextureView,
) {
    let frame = context
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        render_pass.set_pipeline(&self.render_pipeline);

        // mesh for vertex shader
        render_pass.set_vertex_buffer(0, model.mesh.vertex_buffer.slice(..));

        // transform for vertex shader
        render_pass.set_bind_group(0, &camera_handler.bind_group, &[]);

        // material for fragment shader
        render_pass.set_bind_group(1, &model.material.bind_group, &[]);

        render_pass.draw(0..36, 0..1);
    }

    context.queue.submit(Some(encoder.finish()));
    frame.present();
}

