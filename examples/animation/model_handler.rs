use std::rc::Rc;
use wgpu::{BindGroupLayout, RenderPipeline, TextureView};
use spark_gap::camera_handler::CameraHandler;
use spark_gap::context::Context;
use spark_gap::model::Model;
use crate::run_loop::BACKGROUND_COLOR;

pub struct ModelHandler {
    model: Model,
    render_pipeline: RenderPipeline,
}

impl ModelHandler {
    pub fn new(context: &Context, model: Model, camera_handler: &CameraHandler) -> Self {

        let render_pipeline = create_model_render_pipeline(
            &context,
            &model.meshes[0].materials[0].bind_group_layout,  // hack for now
            &camera_handler.bind_group_layout,
        );

        ModelHandler {
            model,
            render_pipeline

        }
    }

    pub fn draw(&self,
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
}

pub fn create_model_render_pipeline(
    context: &Context,
    texture_bind_group_layout: &BindGroupLayout,
    camera_bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let pipeline_layout = context
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, texture_bind_group_layout],
            push_constant_ranges: &[],
        });

    let shader = context
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("animation_shader.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("animation_shader.wgsl").into()),
        });

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context
        .device
        .create_render_pipeline(
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
