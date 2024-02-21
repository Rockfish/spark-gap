use crate::model::Model;
use crate::texture::{create_depth_texture, Texture};
use glam::vec3;
use spark_gap::camera::camera_handler::{CameraHandler, CAMERA_BIND_GROUP_LAYOUT};
use spark_gap::camera::fly_camera_controller::FlyCameraController;
use spark_gap::frame_counter::FrameCounter;
use spark_gap::gpu_context::GpuContext;
use spark_gap::model_mesh::ModelVertex;
use std::sync::Arc;
use wgpu::{BindGroupLayout, RenderPipeline};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard;
use winit::keyboard::NamedKey::Escape;
use winit::window::Window;

const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = GpuContext::new(window).await;
    let mut frame_counter = FrameCounter::new();

    let size = context.window.inner_size();
    let aspect_ratio = size.width as f32 / size.height as f32;

    let camera_position = vec3(1.5, 1.5, 5.0);
    let camera_controller = FlyCameraController::new(aspect_ratio, camera_position, 15.0, -15.0);
    let camera_handler = CameraHandler::new(&mut context, &camera_controller);

    let model = Model::new(&context);

    let mut depth_texture = create_depth_texture(&context);

    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();

    let render_pipeline = create_render_pipeline(&context, &model.material.bind_group_layout, &camera_bind_group_layout);

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent { window_id: _, event } = event {
                match event {
                    WindowEvent::Resized(new_size) => {
                        context.resize(new_size);
                        camera_handler.update_camera(&context, &camera_controller);
                        depth_texture = create_depth_texture(&context);
                        context.window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        draw(&context, &render_pipeline, &camera_handler, &model, &depth_texture);

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

pub fn draw(
    context: &GpuContext,
    render_pipeline: &RenderPipeline,
    camera_handler: &CameraHandler,
    model: &Model,
    depth_texture: &Texture,
) {
    let frame = context
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                view: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(render_pipeline);

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

pub fn create_render_pipeline(
    context: &GpuContext,
    texture_bind_group_layout: &BindGroupLayout,
    camera_bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout, texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[ModelVertex::vertex_description()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    render_pipeline
}
