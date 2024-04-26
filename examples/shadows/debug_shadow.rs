use std::borrow::Cow;

use glam::{Mat4, vec3};
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPass, RenderPipeline, Sampler, Texture, TextureView};

use spark_gap::buffers::{create_buffer_bind_group, create_mat4_buffer_init, create_uniform_bind_group_layout, TRANSFORM_BIND_GROUP_LAYOUT};
use spark_gap::gpu_context::{get_or_create_bind_group_layout, GpuContext};
use spark_gap::small_mesh::{create_unit_square, SmallMesh};

use crate::lights::MAX_LIGHTS;
use crate::world::World;

pub const SHADOW_WIDTH: u32 = 6 * 1024;
pub const SHADOW_HEIGHT: u32 = 6 * 1024;

pub const SHADOW_BIND_GROUP_LAYOUT: &str = "shadow comparison bind group layout";
pub const SHADOW_COMPARISON_BIND_GROUP_LAYOUT: &str = "shadow comparison bind group layout";
pub const SHADOW_FILTER_BIND_GROUP_LAYOUT: &str = "shadow filter bind group layout";

// two choices, let world create the texture, or have it created here.

pub struct ShadowMaterial {
    pub quad_mesh: SmallMesh,
    pub texture: Texture,
    pub texture_view: TextureView,
    pub texture_sampler: Sampler,
    pub filter_bind_group: BindGroup,
    pub projection_view_buffer: Buffer,
    pub projection_view_bind_group: BindGroup,
    pub transform_buffer: Buffer,
    pub transform_bind_group: BindGroup,
    pub debug_texture_pipeline: RenderPipeline,
}


pub fn create_shadow_map_material(context: &mut GpuContext) -> ShadowMaterial {

    let texture_size = wgpu::Extent3d {
        width: 2048,
        height: 2048,
        depth_or_array_layers: MAX_LIGHTS as u32,
    };

    let texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size:texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("shadow map texture"),
        view_formats: &[],
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    if !context.bind_layout_cache.contains_key(SHADOW_FILTER_BIND_GROUP_LAYOUT) {
        let layout = create_shadow_filter_bind_group_layout(context);
        context
            .bind_layout_cache
            .insert(String::from(SHADOW_FILTER_BIND_GROUP_LAYOUT), layout.into());
    }

    let shadow_filter_layout = context.bind_layout_cache.get(SHADOW_FILTER_BIND_GROUP_LAYOUT).unwrap();

    let filter_bind_group = create_texture_bind_group(context, &shadow_filter_layout, &texture_view, &texture_sampler);

    let transform_layout = get_or_create_bind_group_layout(context, TRANSFORM_BIND_GROUP_LAYOUT, create_uniform_bind_group_layout);

    let scale = 7.0f32;
    let mut model_transform = Mat4::from_scale(vec3(scale, scale, scale));
    model_transform *= Mat4::from_rotation_z(180.0f32.to_radians());

    let projection_view_buffer = create_mat4_buffer_init(context, &model_transform, "shadow debug projection view");
    let projection_view_bind_group = create_buffer_bind_group(context, &transform_layout, &projection_view_buffer, "shadow debug projection view bind group");

    let transform_buffer = create_mat4_buffer_init(context, &model_transform, "shadow debug transform");
    let transform_bind_group = create_buffer_bind_group(context, &transform_layout, &transform_buffer, "shadow debug transform bind group");

    let quad_mesh = create_unit_square(context);

    let debug_texture_pipeline = create_debug_depth_render_pipeline(context);

    ShadowMaterial {
        quad_mesh,
        texture,
        texture_view,
        texture_sampler,
        filter_bind_group,
        projection_view_buffer,
        projection_view_bind_group,
        transform_buffer,
        transform_bind_group,
        debug_texture_pipeline,
    }
}

fn create_shadow_filter_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            // 0: texture
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            // 1: sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
        label: Some(SHADOW_FILTER_BIND_GROUP_LAYOUT),
    })
}

fn create_texture_bind_group(
    context: &GpuContext,
    bind_group_layout: &BindGroupLayout,
    texture_view: &TextureView,
    texture_sampler: &Sampler,
) -> BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("shadow filter bind group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(texture_sampler),
            },
        ],
    })
}

pub fn create_debug_depth_render_pipeline(gpu_context: &GpuContext) -> RenderPipeline {

    let shader = gpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("debug_depth_shader.wgsl"))),
    });

    // let camera_bind_group_layout = gpu_context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let transform_bind_group_layout = gpu_context.bind_layout_cache.get(TRANSFORM_BIND_GROUP_LAYOUT).unwrap();
    let shadow_filter_bind_group_layout = gpu_context.bind_layout_cache.get(SHADOW_FILTER_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = gpu_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            transform_bind_group_layout, // projection view
            transform_bind_group_layout, // model transform
            shadow_filter_bind_group_layout
        ],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = gpu_context.surface.get_capabilities(&gpu_context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = gpu_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("debug texture pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[SmallMesh::vertex_description()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
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

pub fn shadow_render_debug<'a>(
    mut render_pass: RenderPass<'a>,
    shadow_map: &'a ShadowMaterial,
) -> RenderPass<'a> {
    render_pass.set_bind_group(0, &shadow_map.projection_view_bind_group, &[]);
    render_pass.set_bind_group(1, &shadow_map.transform_bind_group, &[]);

    render_pass.set_bind_group(2, &shadow_map.filter_bind_group, &[]);
    // render_pass.set_bind_group(2, &shadow_map.test_material.bind_group, &[]);

    render_pass.set_vertex_buffer(0, shadow_map.quad_mesh.vertex_buffer.slice(..));
    render_pass.draw(0..6, 0..1);

    render_pass
}