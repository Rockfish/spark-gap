use std::mem;
use bytemuck::{Pod, Zeroable};

use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPipeline, Sampler, ShaderModule, TextureView};

use spark_gap::gpu_context::GpuContext;

use crate::lights::{Light, Lights, LightUniform};
use crate::world::{get_projection_view_matrix, get_vertex_buffer_layout};

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const MAX_LIGHTS: usize = 10;

pub const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 512,
    height: 512,
    depth_or_array_layers: MAX_LIGHTS as u32,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub proj: [[f32; 4]; 4],
    pub num_lights: [u32; 4],
}

pub struct Pass {
    pub(crate) pipeline: RenderPipeline,
    pub(crate) bind_group: BindGroup,
    pub(crate) uniform_buf: Buffer,
}

pub fn create_forward_pass(
    context: &mut GpuContext,
    local_bind_group_layout: &BindGroupLayout,
    lights: &Lights,
    shader: &ShaderModule,
    shadow_view: &TextureView,
    shadow_sampler: &Sampler,
) -> Pass {
    let supports_storage_resources = context
        .adapter
        .get_downlevel_capabilities()
        .flags
        .contains(wgpu::DownlevelFlags::VERTEX_STORAGE)
        && context.device.limits().max_storage_buffers_per_shader_stage > 0;

    let light_uniform_size = (MAX_LIGHTS * mem::size_of::<LightUniform>()) as wgpu::BufferAddress;

    // Create pipeline layout
    let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // global
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(mem::size_of::<GlobalUniforms>() as _),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1, // lights
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: if supports_storage_resources {
                        wgpu::BufferBindingType::Storage { read_only: true }
                    } else {
                        wgpu::BufferBindingType::Uniform
                    },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(light_uniform_size),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
        label: None,
    });

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("main"),
        bind_group_layouts: &[&bind_group_layout, &local_bind_group_layout],
        push_constant_ranges: &[],
    });

    let mx_total = get_projection_view_matrix(context.config.width as f32 / context.config.height as f32);

    let forward_uniforms = GlobalUniforms {
        proj: mx_total.to_cols_array_2d(),
        num_lights: [lights.lights.len() as u32, 0, 0, 0],
    };

    let uniform_buf = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(&forward_uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Create bind group
    let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: lights.light_storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(shadow_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(shadow_sampler),
            },
        ],
        label: None,
    });

    // Create the render pipeline
    let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("main"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[get_vertex_buffer_layout()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: if supports_storage_resources {
                "fs_main"
            } else {
                "fs_main_without_storage"
            },
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

    Pass {
        pipeline,
        bind_group,
        uniform_buf,
    }
}

pub fn create_shadow_pass(context: &mut GpuContext, local_bind_group_layout: &BindGroupLayout, shader: &ShaderModule) -> Pass {
    let uniform_size = mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress;

    // Create pipeline layout
    let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0, // global
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(uniform_size),
            },
            count: None,
        }],
    });

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow"),
        bind_group_layouts: &[&bind_group_layout, &local_bind_group_layout],
        push_constant_ranges: &[],
    });

    let uniform_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: uniform_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create bind group
    let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buf.as_entire_binding(),
        }],
        label: None,
    });

    // Create the render pipeline
    let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow"),
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

    Pass {
        pipeline,
        bind_group,
        uniform_buf,
    }
}
