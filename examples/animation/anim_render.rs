use glam::{Mat4, vec3};
use crate::run_loop::BACKGROUND_COLOR;
use crate::world::World;
use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::context::Context;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use spark_gap::model_builder::MODEL_BIND_GROUP_LAYOUT;
use spark_gap::model_mesh::ModelVertex;
use spark_gap::texture_config::TextureType;
use wgpu::{IndexFormat, RenderPass, RenderPipeline, TextureView};
use spark_gap::model::Model;

pub struct AnimRenderPass {
    render_pipeline: RenderPipeline,
}

impl AnimRenderPass {
    pub fn new(context: &Context) -> Self {
        let render_pipeline = create_render_pipeline(context);

        Self { render_pipeline }
    }

    pub fn render(&self, context: &Context, world: &World) {
        let frame = context
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let pass_description = wgpu::RenderPassDescriptor {
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
                view: &world.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&pass_description);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);

            let render_pass = render_model(context, render_pass, &world.model, &world.model_transform);

            let model_transform = Mat4::from_translation(vec3(50.0, 0.0, -100.0));

            render_model(context, render_pass, &world.model_2, &model_transform);
        }

        context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn render_model<'a>(context: &'a Context, mut render_pass: RenderPass<'a>, model: &'a Model, model_transform: &'a Mat4) -> RenderPass<'a> {

    render_pass.set_bind_group(1, &model.bind_group, &[]);

    let animator = model.animator.borrow();

    let final_bones = animator.final_bone_matrices.borrow();
    let final_nodes = animator.final_node_matrices.borrow();

    context.queue.write_buffer(
        &model.model_transform_buffer,
        0,
        bytemuck::cast_slice(&model_transform.to_cols_array()),
    );

    context.queue.write_buffer(
        &model.final_bones_matrices_buffer,
        0,
        bytemuck::cast_slice(final_bones.as_ref()));

    for mesh in model.meshes.iter() {
        let node_transform = &final_nodes[mesh.id as usize].to_cols_array();

        context.queue.write_buffer(
            &model.node_transform_buffer,
            0,
            bytemuck::cast_slice(node_transform));

        let diffuse_material = mesh.materials.iter().find(|m| m.texture_type == TextureType::Diffuse).unwrap();

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);

        render_pass.set_bind_group(2, &diffuse_material.bind_group, &[]);

        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }
    render_pass
}



pub fn create_render_pipeline(context: &Context) -> RenderPipeline {
    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let model_bind_group_layout = context.bind_layout_cache.get(MODEL_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout, model_bind_group_layout, material_bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("animation_shader.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("animation_shader.wgsl").into()),
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

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub fn create_depth_texture_view(context: &Context) -> TextureView {
    let size = context.window.inner_size();

    let size = wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[DEPTH_FORMAT],
    };

    let texture = context.device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
