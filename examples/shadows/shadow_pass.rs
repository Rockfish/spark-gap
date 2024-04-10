use std::mem;

use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPipeline, ShaderModule};
use wgpu::util::DeviceExt;

use spark_gap::gpu_context::GpuContext;

use crate::lights::{Lights, LightUniform, MAX_LIGHTS};
use crate::world::get_vertex_buffer_layout;

pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;



pub struct ShadowPass {
    pub pipeline: RenderPipeline,
    pub instance_ids_buffer: Buffer,
    pub bind_group: BindGroup,
}

pub fn create_shadow_pass(
    context: &mut GpuContext,
    lights: &Lights,
    entity_bind_group_layout: &BindGroupLayout,
    shader: &ShaderModule,
) -> ShadowPass {
    let light_uniform_size = (MAX_LIGHTS * mem::size_of::<LightUniform>()) as wgpu::BufferAddress;

    let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            // lights
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(light_uniform_size),
                },
                count: None,
            },
        ],
    });

    let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: lights.light_storage_buffer.as_entire_binding(),
        }],
        label: None,
    });

    let mut instance_ids: Vec<u32> = (0_u32..MAX_LIGHTS as u32).collect();

    let instance_ids_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("light indexes"),
        contents: bytemuck::cast_slice(instance_ids.as_slice()),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let instance_ids_description = wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<u32>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 2,
            format: wgpu::VertexFormat::Uint32,
        }],
    };

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow pipeline layout"),
        bind_group_layouts: &[&bind_group_layout, &entity_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_shadow",
            buffers: &[get_vertex_buffer_layout(), instance_ids_description],
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: context.device.features().contains(wgpu::Features::DEPTH_CLIP_CONTROL),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: SHADOW_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // corresponds to bilinear filtering
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    ShadowPass {
        pipeline,
        instance_ids_buffer,
        bind_group,
    }
}
