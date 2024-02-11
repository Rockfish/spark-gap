use std::borrow::Cow;
use wgpu::{BindGroup, BindGroupLayout, RenderPipeline};
use wgpu::util::DeviceExt;
use crate::context::Context;
use crate::cube::Cube;
use crate::model::Model;
use crate::texture::get_texture;

pub fn create_bind_group_layout(context: &Context) -> BindGroupLayout {
    let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            },
            // wgpu::BindGroupLayoutEntry {
            //     binding: 1,
            //     visibility: wgpu::ShaderStages::FRAGMENT,
            //     ty: wgpu::BindingType::Texture {
            //         multisampled: false,
            //         sample_type: wgpu::TextureSampleType::Uint,
            //         view_dimension: wgpu::TextureViewDimension::D2,
            //     },
            //     count: None,
            // },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
        ],
    });

    bind_group_layout
}

// pub fn create_bind_group(context: &Context, bind_group_layout: &BindGroupLayout) -> BindGroup {
//     let pv_matrix = Cube::get_projection_view_matrix(400.0f32 / 400.0f32);
//
//     let uniform_buf = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("PV Uniform Buffer"),
//         contents: bytemuck::cast_slice(pv_matrix.as_ref()),
//         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//     });
//
//     let texture_view = get_texture(context);
//
//     // Create bind group
//     let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
//         layout: bind_group_layout,
//         entries: &[
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: uniform_buf.as_entire_binding(),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 1,
//                 resource: wgpu::BindingResource::TextureView(&texture_view),
//             },
//         ],
//         label: None,
//     });
//
//     bind_group
// }



