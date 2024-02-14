use crate::context::Context;
use crate::texture::Texture;
use wgpu::{BindGroup, BindGroupLayout};

pub struct Material {
    pub name: String,
    pub texture: Texture,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl Material {
    pub fn new(context: &Context, name: impl Into<String>, texture: Texture) -> Self {
        let bind_group_layout = create_texture_bind_group_layout(context);
        let bind_group = create_texture_bind_group(context, &texture, &bind_group_layout);

        Material {
            name: name.into(),
            texture,
            bind_group,
            bind_group_layout,
        }
    }
}

pub fn create_texture_bind_group_layout(context: &Context) -> BindGroupLayout {
    context
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 0: texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 1: sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
}

pub fn create_texture_bind_group(
    context: &Context,
    texture: &Texture,
    bind_group_layout: &BindGroupLayout,
) -> BindGroup {
    context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        })
}
