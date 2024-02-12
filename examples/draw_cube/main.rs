mod context;
mod cube;
mod texture;
mod bindings;
mod model;
mod camera;

use std::borrow::Cow;
use crate::context::Context;
use crate::cube::{Cube};
use spark_gap::frame_counter::FrameCounter;
use std::sync::Arc;
use wgpu::{BindGroupLayout, RenderPipeline};
use winit::keyboard::NamedKey::Escape;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    keyboard,
    window::Window,
};
use crate::camera::{Camera, CameraHandler};
use crate::model::Model;

async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = Context::new(window).await;
    let mut frame_counter = FrameCounter::new();


    let model = Model::new(&context);
    let camera = Camera::new();
    let camera_handler = CameraHandler::new(&context, &camera);

    let render_pipeline = create_render_pipeline(&context, &model.material.bind_group_layout, &camera_handler.bind_group_layout);

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
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        draw(&context, &render_pipeline, &camera_handler, &model);

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

pub fn create_render_pipeline(context: &Context, texture_bind_group_layout: &BindGroupLayout, camera_bind_group_layout: &BindGroupLayout) -> RenderPipeline {

    let pipeline_layout = context.device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, texture_bind_group_layout],
            push_constant_ranges: &[],
        });

    let shader = context.device.create_shader_module(
        wgpu::ShaderModuleDescriptor {
            label: Some("shader.wgsl"),
            // source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context.device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Model::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

    render_pipeline
}

pub fn draw(context: &Context, render_pipeline: &RenderPipeline, camera_handler: &CameraHandler, model: &Model) {
    let frame = context
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = context.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { label: None }
    );

    {
        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
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

pub fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    #[allow(unused_mut)]
    let mut builder = winit::window::WindowBuilder::new();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        builder = builder.with_canvas(Some(canvas));
    }

    let window = Arc::new(
        builder
            .with_title("A triangle.")
            .with_inner_size(winit::dpi::LogicalSize::new(400.0, 400.0))
            .build(&event_loop)
            .unwrap(),
    );

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run(event_loop, window));
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
