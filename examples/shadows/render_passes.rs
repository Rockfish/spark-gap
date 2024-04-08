use std::mem;

use glam::Mat4;
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPipeline, Sampler, ShaderModule, TextureView};
use wgpu::util::DeviceExt;

use spark_gap::gpu_context::GpuContext;

use crate::lights::{Lights, LightUniform, MAX_LIGHTS};
use crate::world::{get_projection_view_matrix, get_vertex_buffer_layout};

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;


pub const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 512,
    height: 512,
    depth_or_array_layers: MAX_LIGHTS as u32,
};

// #[repr(C)]
// #[derive(Clone, Copy, Pod, Zeroable)]
// pub struct ShaderParams {
//     pub projection_view: [[f32; 4]; 4],
//     pub num_lights: [u32; 4],
// }

pub struct ShadowPass {
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub projection_view_buffer: Buffer,
}

pub struct ForwardPass {
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub projection_view_buffer: Buffer,
}

pub fn create_shadow_pass(context: &mut GpuContext, entity_bind_group_layout: &BindGroupLayout, shader: &ShaderModule) -> ShadowPass {

    let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0, // projection_view
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(mem::size_of::<Mat4>() as u64),
            },
            count: None,
        }],
    });

    let projection_view_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("shadow projection view buffer"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        size: mem::size_of::<Mat4>() as wgpu::BufferAddress,
        mapped_at_creation: false,
    });

    let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: projection_view_buffer.as_entire_binding(),
        }],
        label: None,
    });

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
            entry_point: "vs_bake",
            buffers: &[get_vertex_buffer_layout()],
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
        bind_group,
        projection_view_buffer,
    }
}

pub fn create_forward_pass(
    context: &mut GpuContext,
    entity_bind_group_layout: &BindGroupLayout,
    lights: &Lights,
    shader: &ShaderModule,
    shadow_view: &TextureView,
    shadow_sampler: &Sampler,
) -> ForwardPass {

    let light_uniform_size = (MAX_LIGHTS * mem::size_of::<LightUniform>()) as wgpu::BufferAddress;

    let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            // projection_view
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(mem::size_of::<Mat4>() as _),
                },
                count: None,
            },
            // number of lights
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(mem::size_of::<u32>() as _),
                },
                count: None,
            },
            // lights
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(light_uniform_size),
                },
                count: None,
            },
            // shadow texture
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            // shadow sampler
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
        label: None,
    });

    let project_view_matrix = get_projection_view_matrix(context.config.width as f32 / context.config.height as f32);

    let projection_view_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("projection_view buffer"),
        contents: bytemuck::cast_slice(&project_view_matrix.to_cols_array()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let num_lights = lights.lights.len() as u32;
    // let num_lights = 1_u32;

    let num_lights_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("num_lights buffer"),
        contents: bytemuck::bytes_of(&num_lights),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Create bind group
    let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_view_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: num_lights_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: lights.light_storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(shadow_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(shadow_sampler),
            },
        ],
        label: None,
    });

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("main"),
        bind_group_layouts: &[&bind_group_layout, &entity_bind_group_layout],
        push_constant_ranges: &[],
    });

    // Create the render pipeline
    let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("forward pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[get_vertex_buffer_layout()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(context.config.view_formats[0].into())],
        }),
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    ForwardPass {
        pipeline,
        bind_group,
        projection_view_buffer,
    }
}

