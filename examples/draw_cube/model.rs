use wgpu::BindingType::Texture;
use wgpu::RenderPipeline;
use wgpu::util::DeviceExt;
use crate::context::Context;
use crate::cube::Cube;
use crate::texture;
use crate::texture::{get_texture, get_texture_bind_group};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    // pub normal: [f32; 3],
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    // pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Model {
    pub mesh: Mesh,
    pub material: Material,
}

impl Model {
    pub fn new(context: &Context) -> Self {

        let cube = Cube::new();

        let vertex_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&cube.vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mesh = Mesh {
            name: String::from("Cube"),
            vertex_buffer,
            num_elements: 36,
        };

        let diffuse_texture = get_texture(context);
        let (bind_group_layout, bind_group) = get_texture_bind_group(context);

        let material = Material {
            name: String::from("Container Texture"),
            diffuse_texture,
            bind_group,
            bind_group_layout,
        };

        Model {
            mesh,
            material
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // For indices if used.
                // wgpu::VertexAttribute {
                //     offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                //     shader_location: 2,
                //     format: wgpu::VertexFormat::Float32x3,
                // },
            ],
        }
    }
}
